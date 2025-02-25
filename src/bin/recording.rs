#![allow(unused, unreachable_code)]

use evdev::{Device, KeyCode};
use xkbcommon::xkb::{self, Context, KeyDirection, Keycode};

use std::{
    fs,
    io::Write,
    process::{Child, ChildStdin, Stdio},
    thread,
    time::Duration,
};

// TODO: chars that can be combined are named DEAD_X, example DEAD_CIRCUMFLEX for ^
// or DEAD_TILDE for ~, this is because they can be combined to create â or ñ,
// therefore they dont send chars, fix this later by catching the variant for "dead keys"

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// char offset to convert from evdev to xkbcommon
const XKB_OFFSET: u16 = 8;

// Used to implement the drop trait to kill the process
struct AppChild {
    child: Child,
    pipe: ChildStdin,
}

impl Drop for AppChild {
    fn drop(&mut self) {
        self.child.kill().expect("failed to kill child process");
        self.child.wait().expect("failed to wait for child exit");
    }
}

fn main() -> Result<()> {
    // pick a device
    let mut dev = pick_device();

    // Initialize xkbcommon for key translation
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);

    // TODO:
    // dump layout and pipe through and load the keymap with std::command.
    // check https://www.youtube.com/watch?v=6jFSr_a_xEQ&t=2337s
    //
    // xkb::Keymap::new_from_string()
    // https://xkbcommon.org/doc/current/group__keymap.html#ga502717aa7148fd17d4970896f1e9e06f
    let keymap = xkb::Keymap::new_from_names(
        &context,
        "",
        "105",
        "se",
        "",
        None, // Default layout
        xkb::COMPILE_NO_FLAGS,
    )
    .expect("Failed to load keymap");

    let mut state = xkb::State::new(&keymap);

    let mut child = spawn_window();

    // get all the event and print them.
    'outer: loop {
        for event in dev.fetch_events().expect("failed to get events") {
            if let evdev::EventSummary::Key(event, code, value) = event.destructure() {
                // HACK: Escape is doing something to our state,
                //       So we remove it for now
                if code == KeyCode::KEY_ESC {
                    continue;
                }

                let keycode = (code.0 + XKB_OFFSET).into();
                let Some(dir) = direction(value) else {
                    continue;
                };

                state.update_key(keycode, dir);

                // check if we should get more syms for the logging?
                // how should the logging look?
                // maybe should be more, where is the debug?
                let sym = state.key_get_one_sym(keycode);
                if !sym.is_modifier_key() && value == 1 {
                    let x = state.key_get_utf8((code.0 + 8).into());
                    /// this doesnt print '"'
                    ///
                    child.pipe.write_all(x.as_bytes());
                    println!("sym {sym:?} you sent: {x}");
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

const fn direction(i: i32) -> Option<KeyDirection> {
    match i {
        1 => Some(KeyDirection::Down),
        0 => Some(KeyDirection::Up),
        _ => None,
    }
}

pub fn pick_device() -> evdev::Device {
    use std::io::prelude::*;
    // TODO: Make this into a config file to load settings for the
    // windows to record and also the devices
    let mut args = std::env::args_os();
    args.next();
    if let Some(dev_file) = args.next() {
        let dev_string = format!("/dev/input/{}", &dev_file.to_str().unwrap());
        println!("{dev_string}");
        evdev::Device::open(dev_string).unwrap()
    } else {
        //TODO: Make this into its own function to be able to use the command line
        let mut devices: Vec<_> = evdev::enumerate().map(|t| t.1).collect();
        devices.reverse();
        for (i, d) in devices.iter().enumerate() {
            println!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
        }
        print!("Select the device [0-{}]: ", devices.len() - 1);
        let _ = std::io::stdout().flush();

        let mut chosen = String::new();
        let n = loop {
            chosen.clear();
            std::io::stdin().read_line(&mut chosen).unwrap();

            match chosen.trim().parse::<usize>() {
                Ok(n) if n < devices.len() => break n,
                _ => {
                    eprintln!(
                        "ERROR: failed to parse number, enter a number between [0-{}]",
                        devices.len() - 1
                    );
                }
            }
        };
        devices.into_iter().nth(n).unwrap()
    }
}

// HACK: Egui must run i the mainthread, this is also a requirement for
// xkbcommon or evdev to work properly, dont remember which.
// Therefore we spawn another process instead of using a thread.
fn spawn_window() -> AppChild {
    let mut command = std::process::Command::new("./target/debug/gui");
    let mut child = command
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to spawn UI process");

    let mut pipe = match child.stdin.take() {
        Some(pipe) => pipe,
        None => {
            // needed to not not leave a ghost process
            // implemented in the drop function of AppChild
            child.kill().expect("failed to kill child process");
            child.wait().expect("failed to await child process");
            panic!("failed to take standard in from gui process")
        }
    };

    AppChild { child, pipe }
}

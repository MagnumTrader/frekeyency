#![allow(unused, unreachable_code)]

use evdev::{Device, KeyCode};
use xkbcommon::xkb::{self, Context, KeyDirection, Keycode};

use std::{fs, thread, time::Duration};

// TODO: chars that can be combined are named DEAD_X, example DEAD_CIRCUMFLEX for ^ 
// or DEAD_TILDE for ~, this is because they can be combined to create â or ñ, 
// therefore they dont send chars, fix this later by catching the variant for "dead keys"
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// char offset to convert from evdev to xkbcommon
const XKB_OFFSET: u16 = 8;

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
        let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
        devices.reverse();
        for (i, d) in devices.iter().enumerate() {
            println!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
        }
        print!("Select the device [0-{}]: ", devices.len());
        let _ = std::io::stdout().flush();
        let mut chosen = String::new();
        std::io::stdin().read_line(&mut chosen).unwrap();
        let n = chosen.trim().parse::<usize>().unwrap();
        devices.into_iter().nth(n).unwrap()
    }
}

fn spawn_window() {
    // spawn a window for our amazing things
    //
    //
    //


}

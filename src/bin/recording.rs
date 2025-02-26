// TODO: next i need the gui to display the presses in a list instead of what it is now
// TODO: Add small DSL/arguements to specify what type of symbols you want

use evdev::{EventSummary, KeyCode as EvDevKeyCode};
use xkbcommon::xkb::{
    self, KeyDirection, Keycode as XkbKeyCode, Keysym, MOD_NAME_ALT, MOD_NAME_CTRL, MOD_NAME_SHIFT,
    STATE_MODS_DEPRESSED,
};

use std::{process::Stdio, thread, time::Duration};

// NOTE: chars that can be combined are named DEAD_X, example DEAD_CIRCUMFLEX for ^
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
    let keymap =
        xkb::Keymap::new_from_names(&context, "", "105", "se", "", None, xkb::COMPILE_NO_FLAGS)
            .expect("Failed to load keymap");

    let mut state = xkb::State::new(&keymap);

    //let mut child = spawn_gui();

    'outer: loop {
        for event in dev.fetch_events().expect("failed to get events") {
            let EventSummary::Key(_, code, value) = event.destructure() else {
                continue;
            };

            let keycode: XkbKeyCode = (code.0 + XKB_OFFSET).into();

            let Some(dir) = direction(value) else {
                continue;
            };

            state.update_key(keycode, dir);

            // HACK: Escape is doing something to our state,
            //       So we remove it for now
            if value != 1 || code == EvDevKeyCode::KEY_ESC {
                continue;
            }

            let symbol = match state.key_get_one_sym(keycode) {
                Keysym::dead_circumflex => '^',
                Keysym::dead_grave => '´',
                Keysym::dead_acute => '`',
                Keysym::dead_tilde => '~',
                sym => {
                    let Some(c) = sym.key_char() else {
                        continue;
                    };

                    if c == 'q' && state.mod_name_is_active(MOD_NAME_CTRL, STATE_MODS_DEPRESSED) {
                        break 'outer;
                    }
                    c
                }
            };
            println!("{symbol}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

#[inline]
const fn direction(i: i32) -> Option<KeyDirection> {
    match i {
        1 => Some(KeyDirection::Down),
        0 => Some(KeyDirection::Up),
        // 2 can be returned if we are holding a key,
        // dont know if
        _ => None,
    }
}

// Could be used later, right now im interested in all the symbols i use
#[allow(unused)]
fn get_mod_string(state: &xkb::State) -> String {
    let mods = [
        (MOD_NAME_SHIFT, "Shift"),
        (MOD_NAME_CTRL, "Ctrl"),
        (MOD_NAME_ALT, "Alt"),
    ];

    let mut mod_string = String::new();

    for m in mods.iter() {
        if state.mod_name_is_active(m.0, STATE_MODS_DEPRESSED) {
            if !mod_string.is_empty() {
                mod_string.push_str(" + ");
            }
            mod_string.push_str(m.1);
        };
    }
    mod_string
}

fn pick_device() -> evdev::Device {
    use std::io::prelude::*;

    let mut args = std::env::args_os();
    args.next();
    if let Some(dev_file) = args.next() {
        let dev_string = format!("/dev/input/{}", &dev_file.to_str().unwrap());
        println!("{dev_string}");
        evdev::Device::open(dev_string).unwrap()
    } else {
        // TODO:
        // Make this into its own function to be able to use in multiple places,
        // like the gui for example.
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

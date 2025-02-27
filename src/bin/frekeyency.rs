use evdev::{EventSummary, KeyCode as EvDevKeyCode};
use xkbcommon::xkb::{
    self, KeyDirection, Keycode as XkbKeyCode, Keysym, MOD_NAME_ALT, MOD_NAME_CTRL, MOD_NAME_SHIFT,
    STATE_MODS_DEPRESSED,
};

use std::{thread, time::Duration};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// char offset to convert from evdev to xkbcommon
const XKB_OFFSET: u16 = 8;

fn main() -> Result<()> {
    let mut dev = frekeyency::pick_device(std::env::args());
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

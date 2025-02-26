use evdev::Device;

pub fn pick_device(mut args: std::env::Args) -> evdev::Device {
    use std::io::prelude::*;

    // Skip the current folder of the program, maybe use later
    args.next();
    if let Some(dev_file) = args.next() {
        let dev_string = format!("/dev/input/{}", &dev_file);
        evdev::Device::open(dev_string).unwrap()
    } else {
        let devices = list_devices();

        if devices.len() <= 0 {
            panic!("No devices found, did you the program with sudo?")
        }

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

/// Lists all the devices in /dev/input
pub fn list_devices() -> Vec<Device> {
    evdev::enumerate().map(|t| t.1).collect()
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use eframe::App;
use evdev::Device;

use std::{
    io::Read,
    process::{Child, ChildStdout, Stdio},
    sync::mpsc::{self, Receiver},
};

/*
* TODO: Make a running bool so that we can cancel logging
*       should that exit the process. Stop
* TODO: print symbols in a row instead of one at the time
* TODO: create db connnection and start logging
*       should we start logging in batches, with combos?
*
*
*
*
*/

struct Frekeyency {
    paused: bool,
    device_index: Option<usize>,
    devices: Vec<Device>,
    rx: Option<Receiver<String>>, // current_combo
    last_string: String,
}

impl App for Frekeyency {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                let l = if self.paused { "record" } else { "pause" };
                if ui.button(l).clicked() {
                    self.paused = !self.paused;
                };

                let d = if let Some(index) = self.device_index {
                    &self.devices[index].name().unwrap()
                } else {
                    "select a device!"
                };
                egui::containers::ComboBox::from_label(d).show_ui(ui, |ui| {
                    for (i, device) in self.devices.iter().enumerate() {
                        if let Some(name) = device.name() {
                            if ui
                                .selectable_value(
                                    &mut self.device_index,
                                    Some(i),
                                    device.name().unwrap_or(""),
                                )
                                .clicked()
                            {
                                ctx.request_repaint();
                                self.rx = Some(spawn_frekeyency(&format!("event{}", i)));
                                println!("clicked: {name}, selected: {:?}", &self.device_index);
                            }
                        }
                    }
                });
            });

            if let Some(s) = self.rx.as_mut() {
                if let Ok(s) = s.try_recv() {
                    if self.paused {
                        // Explicit drop of string, no recording during pause.
                        drop(s);
                    } else {
                        self.last_string.push_str(s.trim());
                    }
                }
            }
            let s =
                egui::RichText::new(&self.last_string.chars().rev().collect::<String>()).size(32.0);
            ui.label(s);
        });
    }
}

fn main() -> Result<()> {
    // TODO: handle choosing device from the command line here
    let app = Frekeyency {
        paused: false,
        device_index: None,
        devices: frekeyency::list_devices(),
        rx: None,
        last_string: String::new(),
    };

    let _ = eframe::run_native(
        "dev",
        eframe::NativeOptions::default(),
        Box::new(|_ctx| Ok(Box::new(app))),
    );
    Ok(())
}

// HACK:
// I would like to spawn this as a seperate thread,
// but egui must run i the mainthread, this is also a requirement for
// xkbcommon or evdev to work properly, dont remember which.
// Therefore we spawn another process instead of using a thread and
// listen to Stdout of that process
fn spawn_frekeyency(device_id: &str) -> mpsc::Receiver<String> {
    // TODO: Handle compilation so that we get gui and recorder in the same folder
    let mut command = std::process::Command::new("./target/debug/recording");
    let mut child = command
        .arg(device_id) // my current keyboard
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn frekeyency");

    // Needed to not not leave a ghost process if we crash between
    // spawning the child and creating stdout
    let pipe = match child.stdout.take() {
        Some(pipe) => pipe,
        None => {
            // We are crashing the program anyway, so ignoring results of kill and wait.
            let _ = child.kill();
            let _ = child.wait();
            panic!("")
        }
    };

    let mut child = AppChild { child, pipe };
    let (tx, rx) = mpsc::channel();
    let mut buf = [0; 64];

    std::thread::spawn(move || loop {
        let read = child.pipe.read(&mut buf).unwrap();
        if tx
            .send(String::from_utf8_lossy(&buf[..read]).to_string())
            .is_err()
        {
            break;
        };
    });

    rx
}

/// Wrapper of the child process and a handle to it's Stdout
/// required to implement the drop trait to kill the process
/// if the main program crashes.
struct AppChild {
    child: Child,
    //TODO: Make this a bufreader for more efficient reading
    pipe: ChildStdout,
}

impl Drop for AppChild {
    fn drop(&mut self) {
        self.child.kill().expect("failed to kill child process");
        self.child.wait().expect("failed to wait for child exit");
    }
}

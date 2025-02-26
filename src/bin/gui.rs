type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use evdev::Device;

use std::{
    io::Read,
    process::{Child, ChildStdout, Stdio},
    sync::mpsc,
};

fn main() -> Result<()> {
    let rx = spawn_reader();
    let mut last_string = String::default();
    let devices = frekeyency::list_devices();
    let _device: Option<Device> = None;

    let _ = eframe::run_simple_native("dev", eframe::NativeOptions::default(), move |ctx, _| {
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::containers::ComboBox::from_label("Select a device!").show_ui(ui, |ui| {
                for device in &devices {
                    if let Some(name) = device.name() {
                        if ui.button(name).clicked() {
                            println!("clicked: {name}");
                        }
                    }
                }
            });
            if let Ok(s) = rx.try_recv() {
                last_string = s;
            }
            let s = egui::RichText::new(&last_string).size(32.0);
            ui.label(s);
        });
    });
    Ok(())
}

fn spawn_reader() -> mpsc::Receiver<String> {
    let mut child = spawn_key_reader();
    let (tx, rx) = mpsc::channel();
    let mut buf = [0; 64];
    std::thread::spawn(move || loop {
        let read = child.pipe.read(&mut buf).unwrap();
        if tx
            .send(String::from_utf8_lossy(&buf[..read]).to_string())
            .is_err()
        {
            // Errors only when parent proces have closed
            break;
        };
    });
    rx
}

// HACK:
// I would like to spawn this as a seperate thread,
// but egui must run i the mainthread, this is also a requirement for
// xkbcommon or evdev to work properly, dont remember which.
// Therefore we spawn another process instead of using a thread and
// listen to Stdout of that process
fn spawn_key_reader() -> AppChild {
    // TODO: Handle compilation so that we get gui and recorder in the same folder
    let mut command = std::process::Command::new("./target/debug/recording");

    let mut child = command
        .arg("event15") // my current keyboard
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn key reader");

    // Needed to not not leave a ghost process if we crash between
    // spawning the child and creating stdout
    let pipe = match child.stdout.take() {
        Some(pipe) => pipe,
        None => {
            child.kill().expect("failed to kill child process");
            child.wait().expect("failed to await child process");
            panic!("failed to take standard in from gui process")
        }
    };

    AppChild { child, pipe }
}
/// Wrapper of the child process and a handle to it's Stdout
/// required to implement the drop trait to kill the process
/// if the main program crashes.
struct AppChild {
    child: Child,
    pipe: ChildStdout,
}

impl Drop for AppChild {
    fn drop(&mut self) {
        self.child.kill().expect("failed to kill child process");
        self.child.wait().expect("failed to wait for child exit");
    }
}

#![allow(unused, unreachable_code)]
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use egui;

use std::{
    io::Read,
    process::{Child, ChildStdout, Stdio},
};

fn main() -> Result<()> {

    let mut buf = [0; 8];
    let mut child = spawn_key_reader();

    eframe::run_simple_native(
        "dev",
        eframe::NativeOptions::default(),
        move |ctx, frame| {
            let read = child.pipe.read(&mut buf).unwrap();
            ctx.request_repaint();

            egui::CentralPanel::default().show(ctx, |ui| {
                let s = format!("{}", String::from_utf8_lossy(&buf[..read]));
                let s = egui::RichText::new(s).size(32.0);
                ui.label(s);
            });
        },
    );
    Ok(())
}

// HACK:
// I would like to spawn this as a seperate thread,
// Egui must run i the mainthread, this is also a requirement for
// xkbcommon or evdev to work properly, dont remember which.
// Therefore we spawn another process instead of using a thread and
// listen to Stdout of that process
fn spawn_key_reader() -> AppChild {
    // TODO: Handle compilation so that we get gui and recorder in the same folder
    let mut command = std::process::Command::new("./target/debug/recording");

    let mut child = command
        .arg("event15")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn UI process");

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

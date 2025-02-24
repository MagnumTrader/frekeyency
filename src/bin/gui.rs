#![allow(unused, unreachable_code)]
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use std::io::Read;

fn main() -> Result<()> {
    let mut buf = [0; 8];

    eframe::run_simple_native(
        "FreKeyency",
        eframe::NativeOptions::default(),
        move |ctx, frame| {
            let read = std::io::stdin()
                .read(&mut buf)
                .expect("failed to read stdin");
            ctx.request_repaint();

            egui::CentralPanel::default().show(ctx, |ui| {
                let s = format!("{}", String::from_utf8_lossy(&buf[..read]));
                ui.label(s);
            });
        },
    );
    Ok(())
}

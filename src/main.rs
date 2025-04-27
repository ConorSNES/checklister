use eframe::egui;

mod app;

use app::App;

fn main() -> eframe::Result {
    println!("Program start");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native("El World", options, Box::new(|_| Ok(Box::<App>::default())))
}

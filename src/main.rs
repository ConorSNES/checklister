use eframe::egui;

mod app;

use app::App;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id("checklister"),
        ..Default::default()
    };

    eframe::run_native("Checklister", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

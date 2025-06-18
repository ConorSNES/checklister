#![windows_subsystem = "windows"]

use eframe::{egui::{self}, icon_data::from_png_bytes};

mod app;

use app::App;

const ICON: &[u8] = include_bytes!("..\\icon\\32_checklister.png");

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
			.with_app_id("checklister")
			.with_icon(from_png_bytes(ICON).expect("Missing Icon File!!")),
        ..Default::default()
    };

    eframe::run_native("Checklister", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

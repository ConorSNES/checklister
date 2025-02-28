use eframe::egui;

mod app;

use app::App;




fn main() -> eframe::Result {
    println!("Hello, world!");

	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([256.0, 256.0]),
		..Default::default()
	};

	eframe::run_native("El World", options, Box::new(| _ | { Ok(Box::<App>::default()) }))
}



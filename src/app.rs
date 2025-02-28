use eframe::egui;
use model::{Entry, Model};
use serde::{Deserialize, Serialize};
mod model;

enum Action {
	Add{loc: Vec<isize>, dat: Entry},
	Remove{loc: Vec<isize>}
}

// declaration of the application
#[derive(Serialize, Deserialize)]
pub struct App {
	body : String,
	data : Model
}

impl Default for App {
	fn default() -> Self {
		Self {
			body : "Hello, world".to_owned(),
			data : Model::default()
		}
	}
}

impl App {
	fn tableview() {
		
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		// construct navpanel
		egui::SidePanel::left("navigation").show(ctx, |ui| {
			ui.vertical_centered(|ui| {
				if ui.button("Table view").clicked() {
					self.menu = CurrentMenu::TableView;
				}
				if ui.button("New entry").clicked() {
					self.menu = CurrentMenu::Create;
				};
				ui.set_width(32.0);
			});
		});

		// construct body
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading(self.body.clone());
		});
	}
}
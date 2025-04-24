use eframe::egui::{self, Color32, Frame, Layout, ScrollArea, Ui};
use model::{Entry, EntryHost, EntrySwitch, Model};
use serde::{Deserialize, Serialize};
mod model;

enum Action {
    Add { loc: Vec<isize>, dat: Entry },
    Remove { loc: Vec<isize> },
}

#[derive(Serialize, Deserialize)]
enum CurrentMenu {
    TableView,
    Create,
}

// declaration of the application
#[derive(Serialize, Deserialize)]
pub struct App {
    body: String,
    data: Model,
    menu: CurrentMenu,
}

impl Default for App {
    fn default() -> Self {
        Self {
            body: "Hello, world".to_owned(),
            data: model::make_sample_set(),
            menu: CurrentMenu::TableView,
        }
    }
}

impl App {
    const COL_WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF); //::from_hex("#fff").expect("White is invalid");
    const COL_GREY10: Color32 = Color32::from_rgb(0xEE, 0xEE, 0xEE); //::from_hex("#eee").expect("Grey10 is invalid");

    // Constructs view of all elements.
    fn tableview(&self, ui: &mut Ui) {
        ui.with_layout(
            Layout::top_down(egui::Align::Min).with_main_justify(true),
            |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    Frame::new().show(ui, |ui| {
                        Self::drawentryhost(ui, &self.data.entry, false);
                    })
                });
            },
        );
    }

    // Wraps many drawentry(s) together.
    fn drawentryhost(ui: &mut Ui, subject: &EntryHost, displayeven: bool) -> (usize, usize) {
        let mut o: (usize, usize) = (0, 0);

        for v in &subject.subelements {
            let pending = Self::drawentry(ui, v, displayeven);
            o.0 += pending.0;
            o.1 += pending.1;
        }

        o
    }

    // Dictates how entry should be drawn.
    fn drawentry(ui: &mut Ui, subject: &Entry, displayeven: bool) -> (usize, usize) {
        let mut o = (0, 0);

        Frame::new()
            .fill(match displayeven {
                false => Self::COL_WHITE,
                true => Self::COL_GREY10,
            })
            .corner_radius(4)
            .inner_margin(4)
            .show(ui, |ui| {
                // Switch depending on subject.
                if let EntrySwitch::End(v) = &subject.data {
                    // This is a node of the tree with full data.
                    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                        // Display full data.
                        if ui
                            .button(match v.completed {
                                None => " ",
                                Some(_) => "•",
                            })
                            .clicked()
                        {
                            // When clicked, toggle completed state.
                            // DAMMIT
                        }

                        // Add to o.
                        if let Some(comp) = v.completed {
                            o.0 += 1;
                            ui.label(comp.format("%d/%m/%Y").to_string());
                        } else {
                            o.1 += 1;
                        }

                        ui.label(subject.title.to_owned());
                    });
                } else if let EntrySwitch::Host(v) = &subject.data {
                    // This is a node of the tree with partial data.
                    ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                        // Draw header with title.
                        ui.label(subject.title.to_owned());

                        // Go recursive using host.
                        let totals = Self::drawentryhost(ui, v, !displayeven);

                        // Merge pending totals with output totals.
                        o.0 += totals.0;
                        o.1 += totals.1;

                        // Draw footer in small text.
                        ui.label("footer");
                    });
                }
            });

        o
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
            self.tableview(ui);
        });
    }
}

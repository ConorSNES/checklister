use chrono::Local;
use eframe::egui::{self, Button, Color32, Frame, Label, Layout, RichText, ScrollArea, Theme, Ui};
use model::{Entry, EntryHost, EntrySwitch, Model};
use serde::{Deserialize, Serialize};
mod model;

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
    /* const COL_WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
    const COL_GREY10: Color32 = Color32::from_rgb(0xF8, 0xF8, 0xF8); */
    const COL_GREYDARK: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
    const COL_DARK: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1B);

    // Constructs view of all elements.
    fn tableview(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            Frame::new().show(ui, |ui| {
                // Start the first drawentryhost on internal (root) data.
                Self::drawentryhost(ui, &mut self.data.entry, false);
            })
        });
    }

    // Wraps many drawentry(s) together.
    fn drawentryhost(ui: &mut Ui, subject: &mut EntryHost, displayeven: bool) -> (usize, usize) {
        let mut o: (usize, usize) = (0, 0);

        for v in &mut subject.subelements {
            let pending = Self::drawentry(ui, v, displayeven);
            o.0 += pending.0;
            o.1 += pending.1;
        }

        o
    }

    // Dictates how entry should be drawn.
    fn drawentry(ui: &mut Ui, subject: &mut Entry, displayeven: bool) -> (usize, usize) {
        let mut o = (0, 0);
        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            Frame::new()
                .fill(match displayeven {
                    false => Self::COL_GREYDARK,
                    true => Self::COL_DARK,
                })
                .corner_radius(4)
                .inner_margin(4)
                .show(ui, |ui| {
                    // Switch depending on subject.
                    if let EntrySwitch::End(v) = &mut subject.data {
                        // This is a node of the tree with full data.
                        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                            // Display full data.

                            let mainbutton = Button::new(match v.completed {
                                None => " ",
                                Some(_) => "✔",
                            });
                            if ui.add_sized((20.0, 20.0), mainbutton).clicked() {
                                // When clicked, toggle completed state.
                                v.completed = match v.completed {
                                    None => Some(Local::now().naive_local()),
                                    Some(_) => None,
                                };
                            }

                            // Add to o.
                            if let Some(comp) = v.completed {
                                o.0 += 1;
                                ui.label(
                                    RichText::new(comp.format("%d/%m/%Y").to_string())
                                        .size(10.0)
                                        .weak(),
                                );
                            } else {
                                o.1 += 1;
                            }
							// hacky solution: draw right elements before left ones to allow filling center. should make a recipe for this
                            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                                if ui
                                    .add_sized(
                                        [20.0, 20.0],
                                        Button::new(
                                            RichText::new("...").weak(),
                                        )
                                        .fill(Color32::TRANSPARENT),
                                    )
                                    .clicked()
                                {}

                                ui.vertical_centered(|ui| {
                                    ui.label(subject.title.to_owned());
                                });
                            });
                        });
                    } else if let EntrySwitch::Host(v) = &mut subject.data {
                        // This is a node of the tree with partial data.
                        ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                            ui.horizontal_top(|ui| {
                                // Draw header with title.
                                ui.add(Label::new(subject.title.to_owned()));

                                if ui.add_sized((20.0, 20.0), Button::new("+")).clicked() {
                                    // Raise add entry dialog on this entry host
                                }
                            });

                            ui.add_space(8.0);

                            // Go recursive using host.
                            let totals = Self::drawentryhost(ui, v, !displayeven);

                            // Merge pending totals with output totals.
                            o.0 += totals.0;
                            o.1 += totals.1;

                            ui.add_space(8.0);

                            // Draw footer in small text.
                            ui.label("footer");
                        });
                    }
                });
        });

        o
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for hotkeys
        ctx.input(|i| {
            if i.key_down(egui::Key::F4) {
                // Terminate program if f4 is down.
                std::process::exit(0);
            }
        });

        ctx.set_theme(Theme::Dark);

        // construct navpanel
        egui::TopBottomPanel::top("navigation").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("Table view").clicked() {
                    self.menu = CurrentMenu::TableView;
                }
                if ui.add_sized((20.0, 20.0), Button::new("+")).clicked() {
                    // Raise add entry dialog on this entry host
                }
            });
        });

        // construct body
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.heading(self.body.clone());
            self.tableview(ui);
        });
    }
}

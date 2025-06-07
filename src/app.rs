use chrono::Local;
use eframe::egui::{self, Button, Color32, Frame, Label, Layout, RichText, ScrollArea, Theme, Ui};
use model::{Entry, EntryHost, EntrySwitch, Model};
use serde::{Deserialize, Serialize};
mod model;

#[derive(Serialize, Deserialize, PartialEq)]
enum CurrentDialog {
    None,
    Create(Option<Vec<usize>>, String),
    Edit(Vec<usize>),
}

// declaration of the application
#[derive(Serialize, Deserialize)]
pub struct App {
    data: Model,
    dialog: CurrentDialog,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: model::make_sample_set(),
            dialog: CurrentDialog::None,
        }
    }
}

impl App {
    /* const COL_WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
    const COL_GREY10: Color32 = Color32::from_rgb(0xF8, 0xF8, 0xF8); */
    const COL_GREYDARK: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
    const COL_DARK: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1B);

    const TASK_NAME_DEFAULT: &str = "New Task";

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
                        Self::drawtriple(
                            ui,
                            |ui| {
                                // Draw the check button. (you can't add onclick events to checkboxes)
                                if ui
                                    .add_sized(
                                        (20.0, 20.0),
                                        Button::new(match v.completed {
                                            None => " ",
                                            Some(_) => "✔",
                                        }),
                                    )
                                    .clicked()
                                {
                                    // When clicked, toggle completed state.
                                    v.completed = match v.completed {
                                        None => Some(Local::now().naive_local()),
                                        Some(_) => None,
                                    };
                                }

                                // Add to completion stats.
                                // If this entry is complete, add label for date completed.
                                if let Some(comp) = v.completed {
                                    o.0 += 1;
                                    ui.add(Label::new(
                                        RichText::new(comp.format("%d/%m/%Y").to_string())
                                            .size(10.0)
                                            .weak(),
                                    ));
                                } else {
                                    o.1 += 1;
                                }
                            },
                            |ui| {
                                // Add title.
                                ui.label(subject.title.to_owned());
                            },
                            |ui| {
                                // Add edit button.
                                if ui
                                    .add_sized(
                                        [20.0, 20.0],
                                        Button::new(RichText::new("...").weak())
                                            .fill(Color32::TRANSPARENT),
                                    )
                                    .clicked()
                                {
                                    // This is where you put the switch edit code snoosk
                                }
                            },
                        );
                    } else if let EntrySwitch::Host(v) = &mut subject.data {
                        // This is a node of the tree with partial data.
                        ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                            // Draw the header of the box.
                            Self::drawtriple(
                                ui,
                                |ui| {
                                    ui.add_space(20.0);
                                },
                                |ui| {
                                    // Draw title.
                                    ui.add(Label::new(subject.title.to_owned()));
                                },
                                |ui| {
                                    if ui.add_sized((20.0, 20.0), Button::new("+")).clicked() {
                                        // Raise add entry dialog on this entry host
                                    }
                                },
                            );

                            ui.add_space(4.0);

                            // Go recursive using inner entry host.
                            let totals = Self::drawentryhost(ui, v, !displayeven);

                            // Merge pending totals with output totals.
                            o.0 += totals.0;
                            o.1 += totals.1;

                            ui.add_space(4.0);

                            // Draw footer in small text.
                            ui.horizontal(|ui| {
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!(
                                        "{0}/{1} completed",
                                        totals.0,
                                        totals.1 + totals.0
                                    ))
                                    .weak(),
                                );
                            });
                        });
                    }
                });
        });

        o
    }

    // Constructs the current inline edit dialog.
    fn dialogview(&mut self, ui: &mut Ui) {
        if self.dialog == CurrentDialog::None {
            return;
        }
        Frame::new().inner_margin(8).show(ui, |ui| {
            match &mut self.dialog {
                CurrentDialog::None => {}
                CurrentDialog::Create(v, newtaskname) => {
                    match v {
                        None => {
                            ui.heading("Create task");
                        }
                        Some(v) => {
                            ui.heading(format!(
                                "Create subtask for {}",
                                self.data.entry.deepget(v).title
                            ));
                        }
                    }

                    ui.text_edit_singleline(newtaskname);

					let mut close: bool = false;

                    ui.horizontal(|ui| {
                        // Close buttons (with some extra logic to handle the label)
						let canceled = ui.button("Cancel").clicked();
						let confirmed = ui.button("Done").clicked();

                        if confirmed {
                            // add task to location with new content
                            match v {
                                None => {
									// We're adding an entry to the root
                                    self.data.entry.subelements.push(Entry::new_end(newtaskname.to_string(), "".to_string()));
									self.data.entry.sort();
                                }
                                Some(v) => {
									// We're adding an entry to a child entry
									match &mut self.data.entry.deepget(v).data {
										EntrySwitch::Host(w) => {
											w.subelements.push(Entry::new_end(newtaskname.to_string(), "".to_string()));
											w.sort();
										},
										EntrySwitch::End(_) => {
											panic!("Attempted to add an entry to a non-entryhost!");
										}
									}
                                }
                            };
                        }

						close = canceled | confirmed;
                    });

					if close {
						self.dialog = CurrentDialog::None;
					}
                }
                CurrentDialog::Edit(v) => {
                    ui.heading("Edit task");
                    ui.text_edit_singleline(&mut self.data.entry.deepget(v).title);
                    // Close button
                    if ui.button("Done").clicked() {
                        self.dialog = CurrentDialog::None;
                    }
                }
            };
        });
    }

    // Recipe for drawing a "triple";
    // Left, right aligned content with centred content between.
    fn drawtriple<R>(
        ui: &mut Ui,
        left: impl FnOnce(&mut Ui) -> R,
        centre: impl FnOnce(&mut Ui) -> R,
        right: impl FnOnce(&mut Ui) -> R,
    ) {
        ui.horizontal(|ui| {
            // Draw left
            left(ui);

            // Create rtl box
            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                // Draw right
                right(ui);

                // Draw centred box with content
                ui.vertical_centered(centre);
            });
        });
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
                // Wrap these elements in a panel for improved layout
                Frame::new().inner_margin(4.0).show(ui, |ui| {
                    if ui.button("Table view").clicked() {
                        // I don't know what this will do
                    }
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.add_sized((20.0, 20.0), Button::new("+")).clicked() {
                            // Raise add entry dialog on this entry host
                            self.dialog =
                                CurrentDialog::Create(None, App::TASK_NAME_DEFAULT.to_owned());
                        }
                    });
                });
            });
        });

        // construct body
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show current edit modal
            self.dialogview(ui);

            // Show all tasks
            self.tableview(ui);
        });
    }
}

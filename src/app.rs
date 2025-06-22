use core::f32;
use std::{fs::File, hash::{Hash, Hasher}, io::Write, path::Path, time::Duration};
use eframe::egui::{
    self, popup_below_widget, Button, Color32, FontSelection, Frame, Id, KeyboardShortcut, Label, Layout, Modal, Modifiers, RichText, ScrollArea, TextEdit, Ui
};
use model::{Entry, EntryHost, EntrySwitch, Model};
use serde::{Deserialize, Serialize};
use crate::app::{currentact::CurrentAct, displaytyped::{DisplayType, DisplayTyped}, model::{traits::Filterable, EntryEnd}, recipes::{drawtriple_mutpass, togglepopup}, xorhasher::XorHasher};

pub mod model;
pub mod xorhasher;
mod recipes;
mod menubar;
mod displaytyped;
mod currentact;

// declaration of the application
#[derive(Serialize, Deserialize, Default)]
pub struct App {
    data: Model,
    #[serde(skip)]
    action: CurrentAct,
}

impl App {
	const PERSISTENCE_KEY: &str = "main";

    const COL_WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
    const COL_GREY10: Color32 = Color32::from_rgb(0xF8, 0xF8, 0xF8);
    const COL_GREYDARK: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
    const COL_DARK: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1B);

    const TASK_NAME_DEFAULT: &str = "New Task";
    const NOTES_TEXT_DEFAULT: &str = "Add notes...";

    pub const KEYCOMBO_NOTES_NEWLINE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::Enter);

    pub const KEYCOMBO_SUBMIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, egui::Key::Enter);
    pub const KEYCOMBO_DISMISS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, egui::Key::Escape);
	pub const KEYCOMBO_EXIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, egui::Key::F4);
    pub const KEYCOMBO_FIND: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::F);
	pub const KEYCOMBO_ADD: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::N);
	pub const KEYCOMBO_ADDLIST: KeyboardShortcut = KeyboardShortcut::new(Modifiers { alt: false, ctrl: false, shift: true, mac_cmd: false, command: true }, egui::Key::N);

	pub fn new(cc: &eframe::CreationContext) -> Self {
		match cc.storage {
			None => Self::default(),
			Some(storage) => {
				match eframe::get_value(storage, Self::PERSISTENCE_KEY) {
					None => Self::default(),
					Some(v) => v
				}
			}
		}
	}

	// Returns a finished hash of the current elements.
	fn model_hash(&self) -> u8 {
		let mut hashobj = XorHasher::default();
		self.data.hash(&mut hashobj);
		hashobj.finish() as u8
	}

	fn export_model(&self, path : &Path) -> Option<String> {
		// We need to handle error states for failing to create the target, serialize or writing to the new file.
		match File::create(path) {
			Err(err) => Some(format!("Could not create file: {}", err)),
			Ok(mut outfile) => {
				match serde_json::to_vec_pretty(&self.data) {
					Err(err) => Some(format!("Could not serialize file: {}", err)),
					Ok(stringified) => {
						match outfile.write_all(&stringified ) {
							Err(err) => Some(format!("Failed to write to file: {}", err)),
							Ok(_) => None
						}
					}
				}
			}
		}
	}

	fn import_model(&mut self, path : &Path) -> Option<String> {
		// We need to handle error states for both failing to open the file and failing to parse the contents.
		match std::fs::read(path) {
			Result::Err(err) => Some(format!("Could not read file: {}", err)),
			Result::Ok(infile) => {
				match serde_json::from_slice(&infile) {
					Result::Err(err) => Some(format!("Could not parse imported file: {}", err)),
					Result::Ok(data) => {
						self.data = data;
						None
					}
				}
			}
		}
	}

	fn new_model(&mut self) {
		self.data = Model::default();
	}

    // Constructs view of all elements.
    fn show_tableview(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            Frame::new().show(ui, |ui| {
                // Start the first drawentryhost on internal (root) data.
				let filter = if let CurrentAct::Find(v) = &self.action { v } else { &"".to_owned() };
                let result = Self::drawinnerentryhost(ui, &mut self.data.entry, vec![], filter);

                // If we have a new action from drawing, set the current action
                if result.2.some() {
                    self.action = result.2;
                };

                // Draw footer in small text if there's something to show.
                if result.0 + result.1 >= 8 {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "{0}/{1} completed",
                                result.0,
                                result.1 + result.0
                            ))
                            .weak(),
                        );
                    });
                }
            })
        });
    }

	// Constructs an end check button for an element.
	fn show_checkbutton(ui: &mut Ui, subject: &mut EntryEnd) {
		if ui
			.add_sized(
				(20.0, 20.0),
				Button::new(match subject.completed {
					None => " ",
					Some(_) => "✔",
				}),
			)
			.clicked()
		{
			// When clicked, toggle completed state.
			subject.toggle();
		}
	}

	fn show_addbutton(ui: &mut Ui, index: Option<Vec<usize>>, popupid: Id) -> CurrentAct {
		let addbutton = ui.add_sized((20.0, 20.0), Button::new("+"));
		let mut action = CurrentAct::None;

		if addbutton.clicked() {
			// Raise add entry dialog on this entry host
			action = CurrentAct::Create(index.clone(), String::new());
		}

		if addbutton.secondary_clicked() {
			// Raise popup menu for adding items
			togglepopup(ui, popupid);
		}

		// Create popup
		popup_below_widget(
			ui,
			popupid,
			&addbutton,
			egui::PopupCloseBehavior::CloseOnClick,
			|ui| {
				ui.set_min_width(128.0);
				if ui.button("Add subtask").clicked() {
					action = CurrentAct::Create(index.clone(), String::new());
				}
				if ui.button("Add sublist").clicked() {
					action = CurrentAct::CreateHost(
						index.clone(),
						String::new(),
					);
				}
				if let Some(i) = &index {
                    if ui.button("Info").clicked() {
                        action = CurrentAct::Info(i.clone());
                    }

					if ui.button("Delete").clicked() {
						action = CurrentAct::Remove(i.clone());
					}
				}
			},
		);
		action
	}

    // Wraps many drawentry(s) together.
    fn drawinnerentryhost(
        ui: &mut Ui,
        subject: &mut EntryHost,
        index: Vec<usize>,
		filter: &str,
    ) -> (usize, usize, CurrentAct) {
        let mut o = (0, 0, CurrentAct::None);

        // If there are no subelements, show the placeholder.
        if subject.subelements.len() == 0 {
            ui.vertical_centered(|ui| ui.label(RichText::new("No content.").italics()));
        }
        for i in 0..subject.subelements.len() {
            // Collect the nested subject
            let subsubject = &mut subject.subelements[i];
            // Construct the nested index
            let mut newidx = index.to_owned();
            newidx.push(i);

            // Draw entry
            let pending = Self::drawentry(ui, subsubject, newidx, filter);
            o.0 += pending.0;
            o.1 += pending.1;
            // The newest action is returned if it is not an else.
			if pending.2.some() {o.2 = pending.2};
        }

        o
    }

	fn drawentryleft(ui: &mut Ui, subject: &mut EntrySwitch) -> (usize, usize) {
		match subject {
			EntrySwitch::End(v) => {
				// Draw the check button. (you can't add onclick events to checkboxes)
                Self::show_checkbutton(ui, v);

                // Add to completion stats.
                // If this entry is complete, add label for date completed.
                if let Some(comp) = v.completed {
                    ui.add(Label::new(
                        RichText::new(comp.format("%d/%m/%Y").to_string())
                            .size(10.0)
                            .weak(),
                    ));
                    (1, 0)
                } else {
                    (0, 1)
                }
			},
			EntrySwitch::Host(_) => {
				ui.add_space(20.0);
				(0, 0)
			}
		}
	}

	fn drawentryright(ui: &mut Ui, subject: &mut EntrySwitch, index: &Vec<usize>, ) -> CurrentAct {
		let mut action = CurrentAct::None;
		let popupid = Id::new(index.clone());
		match subject {
			EntrySwitch::End(v) => {
				// Add edit button.
                // online egui release was **too new**. detailled popup tech gets added 1.32

                let menuresp = ui.add_sized(
                    [20.0, 20.0],
                    Button::new(RichText::new("...").weak()).fill(Color32::TRANSPARENT),
                );

                if menuresp.clicked() | menuresp.secondary_clicked() {
                    // Raise the popup upon right clicking
                    togglepopup(ui, popupid);
                }

                popup_below_widget(
                    ui,
                    popupid,
                    &menuresp,
                    egui::PopupCloseBehavior::CloseOnClick,
                    |ui| {
                        ui.set_min_width(128.0);
                        if ui.button("Edit").clicked() {
                            action = CurrentAct::Edit(index.clone());
                        }
                        if ui.button("Info").clicked() {
                            action = CurrentAct::Info(index.clone());
                        }
                        if ui.button("Delete").clicked() {
                            action = 
                                // You need to confirm deletion of incomplete tasks
                                match v.completed {
                                    None => CurrentAct::Confirm(Box::new(CurrentAct::Remove(
                                        index.clone(),
                                    ))),
                                    Some(_) => CurrentAct::Remove(index.clone()),
                                }
                            ;
                        }
                    },
                );
			},
			EntrySwitch::Host(_) => {
                action = Self::show_addbutton(ui, Some(index.clone()), popupid);
			}
		}
		action
	}

    // Dictates how any entry should be drawn.
    fn drawentry(
        ui: &mut Ui,
        subject: &mut Entry,
        index: Vec<usize>,
		filter: &str,
    ) -> (usize, usize, CurrentAct) {
		let mut o = (0, 0, CurrentAct::None);

		// Skip if the filter cannot be applied.
		if !subject.visible(filter) {return o;}
        
		let displayeven = index.len() % 2 == 0; // Entries alternate between colours depending on depth-even-ness.
        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            // Collect fill colour
            let dispfill = match ui.style().visuals.dark_mode {
                true => match displayeven {
                    false => Self::COL_GREYDARK,
                    true => Self::COL_DARK,
                },
                false => match displayeven {
                    false => Self::COL_WHITE,
                    true => Self::COL_GREY10,
                },
            };
            // Draw surrounding box
            Frame::new()
                .fill(dispfill)
                .corner_radius(4)
                .inner_margin(4)
                .show(ui, |ui| {
					ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
						let mut subjcompletion = (0, 0);
						let mut subjaction = CurrentAct::default();

						// First, draw our header triple.
						drawtriple_mutpass(ui, subject,  
							|ui, v| {
							subjcompletion = Self::drawentryleft(ui, &mut v.data);
						}, 
						|ui, v| {
							// Add title.
							if ui.label(v.title.clone()).double_clicked() {
								// When doubleclicked, start editing this entry
								o.2 = CurrentAct::Edit(index.clone());
							};
						}, 
						|ui, v| {
							subjaction = Self::drawentryright(ui, &mut v.data, &index);
						});

						// Merge subjcompletion and subaction with o
						if subjaction.some() {
							o.2 = subjaction;
						}
						o.0 = subjcompletion.0;
						o.1 = subjcompletion.1;

						// If this is an entry host, draw the inside
						if let EntrySwitch::Host(v) = &mut subject.data {
							let res = Self::drawinnerentryhost(ui, v, index, filter);
							o.0 += res.0;
							o.1 += res.1;
							if res.2.some() {o.2 = res.2};

							ui.add_space(4.0);

							// Draw footer in small text if there's something to show.
							if res.0 + res.1 >= 8 {
								ui.horizontal(|ui| {
									ui.add_space(8.0);
									ui.label(
										RichText::new(format!("{0}/{1} completed", res.0, res.1 + res.0))
											.weak(),
									);
								});
							}
						}
					});
                });
        });

        o
    }

    // Constructs current inline dialog for actions that use this method.
    fn show_action(&mut self, ui: &mut Ui) {
        Frame::new().inner_margin(8).show(ui, |ui| {
            match &mut self.action {
                CurrentAct::Confirm(v) => {
                    let mut canceled = false;
                    let mut confirmed = false;

					ui.vertical_centered(|ui| {
						ui.label(format!(
                            "Are sure you want to {}?",
                            v.humantext(&self.data.entry)
                        ));

						ui.add_space(4.0);

						ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
							ui.set_max_width(128.0);

							canceled = ui.button("No").clicked();
							confirmed = ui.button("Yes").clicked();
						});
					});

                    if confirmed {
                        self.applyaction();
                    }

                    if canceled {
                        self.closeaction();
                    }
                }
				CurrentAct::Error(v) => {
					// An error simply displays the current error.
					ui.heading("Error");
					ui.label(v.to_owned());
					if ui.button("Dismiss").clicked() {
						self.closeaction();
					}
				}
				CurrentAct::Create(v, title) => {
                    // Draw heading
                    ui.heading(match v {
                        None => "Create task".to_owned(),
                        Some(v) => {
                            format!(
                                "Create subtask for {}",
                                self.data.entry.deepget_mut(v).title
                            )
                        }
                    });

					ui.add_space(4.0);

                    // Block borrowing is weird, so we store the boolflags for cancelling and confirming out here (it doesn't like )
                    let mut canceled = false;
                    let mut confirmed = false;

                    // Use rtl recipe to have a correctly filled formbox
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        // Close buttons (with some extra logic to handle the label)
                        confirmed = ui.button("Done").clicked();
                        canceled = ui.button("Cancel").clicked();

                        // Text edit box
                        ui.add(
                            TextEdit::singleline(title)
                                .hint_text(Self::TASK_NAME_DEFAULT.to_owned())
                                .desired_width(f32::INFINITY),
                        )
                        .request_focus();
                    });

                    if confirmed {
                        self.applyaction();
                    }

                    if canceled | confirmed {
                        self.closeaction();
                    }
                }
                CurrentAct::CreateHost(v, title) => {
                    // Draw heading
                    ui.heading(match v {
                        None => "Create list".to_owned(),
                        Some(v) => {
                            format!(
                                "Create sublist for {}",
                                self.data.entry.deepget_mut(v).title
                            )
                        }
                    });

					ui.add_space(4.0);

                    // Block borrowing is weird, so we store the boolflags for cancelling and confirming out here (it doesn't like )
                    let mut canceled = false;
                    let mut confirmed = false;

                    // Use rtl recipe to have a correctly filled formbox
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        // Close buttons (with some extra logic to handle the label)
                        confirmed = ui.button("Done").clicked();
                        canceled = ui.button("Cancel").clicked();

                        // Text edit box
                        ui.add(
                            TextEdit::singleline(title)
                                .hint_text(Self::TASK_NAME_DEFAULT.to_owned())
                                .desired_width(f32::INFINITY),
                        )
                        .request_focus();
                    });

                    if confirmed {
                        self.applyaction();
                    }

                    if canceled | confirmed {
                        self.closeaction();
                    }
                }
                CurrentAct::Edit(v) => {
                    ui.heading("Edit task");

					ui.add_space(4.0);

                    let mut confirmed = false;

                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        // Close button
                        confirmed = ui.button("Done").clicked();

                        // Main text box
                        ui.add(
                            TextEdit::singleline(&mut self.data.entry.deepget_mut(v).title)
                                .hint_text(Self::TASK_NAME_DEFAULT)
                                .desired_width(f32::INFINITY),
                        )
                        .request_focus();
                    });

                    if confirmed {
                        self.action = CurrentAct::None;
                    }
                }
                CurrentAct::Info(v) => {
                    let value = self.data.entry.deepget_mut(v);

                    ui.add(
                        TextEdit::singleline(&mut value.title).desired_width(f32::INFINITY).font(FontSelection::Style(egui::TextStyle::Heading)).hint_text(Self::TASK_NAME_DEFAULT),
                    );
                    ui.separator();

                    // change layout depending on hosting or ending state
                    match &mut value.data {
                        EntrySwitch::End(subval) => {
                            //ui.label("Notes:");
                            ui.add(
                                TextEdit::multiline(&mut subval.body)
                                .desired_rows(3)
                                .desired_width(f32::INFINITY)
                                .return_key(Self::KEYCOMBO_NOTES_NEWLINE)
                                .hint_text(Self::NOTES_TEXT_DEFAULT)
                            );
                            ui.add(Label::new(
                                RichText::new(
                                    format!(
                                        "You can add new lines using {}.", ui.ctx().format_shortcut(&Self::KEYCOMBO_NOTES_NEWLINE) 
                                    )).italics()));

                            ui.add_space(8.0);

                            ui.label(format!("Created on: {}", subval.added.format("%d/%m/%Y %H:%M:%S")));
                            match &subval.completed {
                                None => {
                                    ui.label("Task has not been completed.");
                                }
                                Some(v) => {
                                    let delta = v.signed_duration_since(subval.added);
                                    //let deltastring = format!("{}d, {}:{}:{}:{}", delta.num_days(), delta.num_hours(), delta.num_minutes(), delta.num_seconds(), delta.num_milliseconds());
                                    ui.label(format!("Completed on: {}\nTime to complete: {}s", v.format("%d/%m/%Y %H:%M:%S"), delta.num_seconds()));
                                }
                            }
                        },
                        EntrySwitch::Host(subval) => {
                            let len = subval.subelements.len();
                            if len > 0 {
                                // Get some numbers
                                let date = subval.date().format("%d/%m/%Y %H:%M:%S");
                                let completed = subval.totalcompleted();
                                ui.label(format!(
                                    "Newest date: {}\n\nTotal tasks: {}\nComplete: {}\nIncomplete: {}", 
                                    date,
                                    len, 
                                    completed, 
                                    len-completed
                                ));
                            }
                            else {
                                ui.label("No newest date.\n\nNo contained tasks.");
                            }
                            
                        }
                    }

                    // Handle footer/exit button
                    ui.separator();
                    let mut canceled = false;
                    ui.horizontal(|ui| {
                        match &value.data {
                            EntrySwitch::End(_) => ui.label("Type: Task"),
                            EntrySwitch::Host(_) => ui.label("Type: Sublist"),
                        };
                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            canceled = ui.button("Close").clicked();
                        });
                    });
                    
                    if canceled {
                        self.action = CurrentAct::None;
                    }
                }
                CurrentAct::Find(v) => {
                    let mut close = false;

                    // Use rtl recipe to have a correctly filled formbox
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        close = ui.button("Close").clicked();

                        // Text edit box
                        ui.add(
                            TextEdit::singleline(v)
                                .hint_text("Find...")
                                .desired_width(f32::INFINITY),
                        )
                        .request_focus();
                    });

                    if close {
                        self.closeaction();
                    }
                }
                _ => {}
            };
        });
    }

    // Apply an open dialog pane
    fn applyaction(&mut self) {
        match &mut self.action {
            CurrentAct::Confirm(subject) => {
                self.action = *subject.clone();
            }
            CurrentAct::Create(subject, title) => {
                // Add task to location with new content
                let host = match subject {
                    None => &mut self.data.entry, // We're adding an entry to the root
                    Some(v) => {
                        // We're adding an entry to a child entry
                        match &mut self.data.entry.deepget_mut(v).data {
                            EntrySwitch::Host(w) => w,
                            EntrySwitch::End(_) => {
                                panic!("Attempted to add an entry to a non-entryhost!")
                            }
                        }
                    }
                };
                let titlefilter = (if title == "" {
                    Self::TASK_NAME_DEFAULT
                } else {
                    title
                })
                .to_string();
                host.subelements
                    .push(Entry::new_end(titlefilter, "".to_string()));
                self.closeaction();
            }
            CurrentAct::CreateHost(subject, title) => {
                // Add list to location with new title
                let host = match subject {
                    None => &mut self.data.entry,
                    Some(v) => match &mut self.data.entry.deepget_mut(v).data {
                        EntrySwitch::Host(w) => w,
                        EntrySwitch::End(_) => {
                            panic!("Attempted to add an entryhost to a non-entryhost!")
                        }
                    },
                };
                host.subelements.push(Entry::new_host(title.to_string()));
                self.closeaction();
            }
            CurrentAct::Remove(v) => {
                let len = v.len();

                // Special case: empty array will remove all members of root but leave root intact
                if len == 0 {
                    self.data.entry.subelements.clear();
                }
                // Otherwise, try to remove at allocated index.
                else if len == 1 {
                    // Remove entry from root
                    self.data.entry.subelements.remove(v[0]);
                } else if let EntrySwitch::Host(w) =
                    &mut self.data.entry.deepget_mut(&v[..len - 1]).data
                {
                    // Remove entry from nested entry host
                    w.subelements.remove(v[len - 1]);
                } else {
                    // An invalid index has been presented.
                    panic!("An invalid index was on a removal action.")
                };
                // Reset the action
                self.closeaction();
            }
			CurrentAct::Cleanup => {
				self.data.entry.cleanup();
				self.closeaction();
			}
			CurrentAct::Sort => {
				self.data.entry.sort();
				self.closeaction();
			}
			CurrentAct::Import(v) => {
				
				let v2 = v.clone(); // would have resulted in a cyclic borrow
				if let Some(err) = self.import_model(v2.as_path()) {
					self.action = CurrentAct::Error(err);
				}
				self.closeaction();
			}
			CurrentAct::Export(v) => {
				let v2 = v.clone();
				if let Some(err) = self.export_model(v2.as_path()) {
					self.action = CurrentAct::Error(err);
				}
				self.closeaction();
			}
			CurrentAct::New => {
				self.new_model();
				self.closeaction();
			}
			_ => self.closeaction(),
        }
    }

    // Close an open dialog pane
    fn closeaction(&mut self) {
        self.action = CurrentAct::None;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Collect pre-op hash of the model.
		let hbefore = self.model_hash();

        // construct navpanel
        egui::TopBottomPanel::top("navigation").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Wrap these elements in a panel for improved layout
                Frame::new().inner_margin(4.0).show(ui, |ui| {
					// Draw menubar/navbar
                    menubar::draw_menubar(self, ctx, ui);

					// Draw add button
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        let popupid = Id::new("addbuttonID");
                        let action = Self::show_addbutton(ui, None, popupid);
						if action.some() {self.action = action}
                    });
                });
            });
        });

        // Show current edit modal, if one is present
		let dt = self.action.display();
		match dt {
			DisplayType::Modal => {
				Modal::new("info".into()).show(ctx, |ui| {self.show_action(ui);});
			},
			DisplayType::Inline => {
				egui::TopBottomPanel::top("info").show(ctx, |ui| {self.show_action(ui);});
			},
			DisplayType::None => {}
		}

        // Check for hotkeys
        ctx.input_mut(|i| {
			if i.consume_shortcut(&Self::KEYCOMBO_EXIT) {
				// Terminate program if f4 is down.
				self.action = CurrentAct::Confirm(Box::new(CurrentAct::Exit));
			}

            // Consume newlines for no action.
            let _ = i.consume_shortcut(&Self::KEYCOMBO_NOTES_NEWLINE);

            if i.consume_shortcut(&Self::KEYCOMBO_SUBMIT) {
                // Apply the current dialog if enter is down.
                self.applyaction();
            }

            if i.consume_shortcut(&Self::KEYCOMBO_DISMISS) {
                // Close the current dialog if enter is down.
                self.closeaction();
            }

            if i.consume_shortcut(&Self::KEYCOMBO_FIND) {
                self.action = CurrentAct::Find(String::new());
            }

			if i.consume_shortcut(&Self::KEYCOMBO_ADDLIST) {
				self.action = CurrentAct::CreateHost(None, "".to_owned());
			}

			if i.consume_shortcut(&Self::KEYCOMBO_ADD) {
				self.action = CurrentAct::Create(None, "".to_owned());
			}
        });

        // Construct main body
        egui::CentralPanel::default().show(ctx, |ui| {

            // Show all tasks
            self.show_tableview(ui);
        });

        // Account for any applicable actions after layout paint
		match self.action {
			CurrentAct::Exit => {
				ctx.send_viewport_cmd(egui::ViewportCommand::Close);
			},
			CurrentAct::Remove(_) 
			| CurrentAct::Cleanup 
			| CurrentAct::Sort 
			| CurrentAct::Import(_) 
			| CurrentAct::Export(_) 
			| CurrentAct::New => self.applyaction(),
			_ => {}
		}
		
		// Collect post-op hash of main model.
		let hnow = self.model_hash();

		if hbefore != hnow {
			// Perform automatic change actions when we have no pending action and there are changes to record.
			self.data.entry.sort();
		}
    }

	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, Self::PERSISTENCE_KEY, &self);
	}

	fn auto_save_interval(&self) -> std::time::Duration {
		Duration::new(5, 0)
	}
}

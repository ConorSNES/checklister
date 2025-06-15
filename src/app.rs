use std::hash::{Hash, Hasher};
use eframe::egui::{
    self, popup_below_widget, Button, Color32, Frame, Id, KeyboardShortcut, Label, Layout, Modal, Modifiers, RichText, ScrollArea, TextEdit, Ui
};
use model::{Entry, EntryHost, EntrySwitch, Model};
use serde::{Deserialize, Serialize};
use crate::app::{model::{traits::Filterable, EntryEnd}, recipes::{drawtriple_mutpass, togglepopup}, xorhasher::XorHasher};

pub mod model;
pub mod xorhasher;
mod recipes;
mod menubar;

enum DisplayType {
	None,
	Modal,
	Inline,
}

#[derive(PartialEq, Clone)]
enum CurrentAct {
    None,
    Confirm(Box<CurrentAct>),
    // Originally (and ideally), we use references to represent the target. This resulted in lifetime complications, so we use a (slightly more expensive?) index based method now.
    Create(Option<Vec<usize>>, String), // Create an ending member of targeted vector
    CreateHost(Option<Vec<usize>>, String), // Create a host member of targeted vector
    Edit(Vec<usize>),
    Remove(Vec<usize>),
    Cleanup,
	Sort,
	Exit,
    Find(String),
}

// pretty simple current dialog
impl Default for CurrentAct {
    fn default() -> Self {
        Self::None
    }
}

impl CurrentAct {
	fn some(&self) -> bool {
		*self != Self::None
	}

	fn display(&self) -> DisplayType {
		match self {
			Self::None | Self::Cleanup | Self::Sort => DisplayType::None,
			Self::Confirm(_) | Self::Create(_, _) | Self::CreateHost(_, _) => DisplayType::Modal,
			Self::Find(_) | Self::Edit(_) => DisplayType::Inline,
			_ => DisplayType::None
		}
	}
}

impl CurrentAct {
    // Function that provides rich translation of current action to english text
    fn humantext(&self, subject: &EntryHost) -> String {
        match self {
            Self::None => "do nothing".to_owned(),
            Self::Confirm(v) => v.as_ref().humantext(subject),
            Self::Remove(v) => format!("remove '{}'", subject.deepget(v).title.to_owned()),
            Self::Cleanup => "remove all completed tasks".to_owned(),
			Self::Exit => "exit".to_owned(),
            _ => "undefined".to_owned(),
        }
    }
}

// declaration of the application
#[derive(Serialize, Deserialize)]
pub struct App {
    data: Model,
    #[serde(skip)]
    action: CurrentAct,
}

impl Default for App {
    fn default() -> Self {
		let mut datamodel = model::make_sample_set();
		datamodel.entry.sort();
        Self {
            data: datamodel,
            action: CurrentAct::None,
        }
    }
}

impl App {
    const COL_WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
    const COL_GREY10: Color32 = Color32::from_rgb(0xF8, 0xF8, 0xF8);
    const COL_GREYDARK: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
    const COL_DARK: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1B);

    const TASK_NAME_DEFAULT: &str = "New Task";

    const KEYCOMBO_FIND: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::F);
	const KEYCOMBO_ADD: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::N);
	const KEYCOMBO_ADDLIST: KeyboardShortcut = KeyboardShortcut::new(Modifiers { alt: false, ctrl: false, shift: true, mac_cmd: false, command: true }, egui::Key::N);

	// Returns a finished hash of the current elements.
	fn model_hash(&self) -> u8 {
		let mut hashobj = XorHasher::default();
		self.data.hash(&mut hashobj);
		hashobj.finish() as u8
	}

    // Constructs view of all elements.
    fn show_tableview(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            Frame::new().show(ui, |ui| {
                // Start the first drawentryhost on internal (root) data.
				let filter = if let CurrentAct::Find(v) = &self.action { v } else { &"".to_owned() };
                let result = Self::drawinnerentryhost(ui, &mut self.data.entry, vec![], filter);

                // If we have a new action from drawing, set the current action
                if let Some(newact) = result.2 {
                    self.action = newact;
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
					if ui.button("Delete this list").clicked() {
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
    ) -> (usize, usize, Option<CurrentAct>) {
        let mut o = (0, 0, None);

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
            o.2 = if pending.2 != None { pending.2 } else { o.2 };
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
                Self::show_addbutton(ui, Some(index.clone()), popupid);
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
    ) -> (usize, usize, Option<CurrentAct>) {
		let mut o = (0, 0, None);

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
								o.2 = Some(CurrentAct::Edit(index.clone()));
							};
						}, 
						|ui, v| {
							subjaction = Self::drawentryright(ui, &mut v.data, &index);
						});

						// Merge subjcompletion and subaction with o
						if subjaction.some() {
							o.2 = Some(subjaction);
						}
						o.0 = subjcompletion.0;
						o.1 = subjcompletion.1;

						// If this is an entry host, draw the inside
						if let EntrySwitch::Host(v) = &mut subject.data {
							let res = Self::drawinnerentryhost(ui, v, index, filter);
							o.0 += res.0;
							o.1 += res.1;

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
    fn dialogview(&mut self, ui: &mut Ui) {
        if self.action == CurrentAct::None {
            return;
        }
        Frame::new().inner_margin(8).show(ui, |ui| {
            match &mut self.action {
                CurrentAct::Confirm(v) => {
                    let mut canceled = false;
                    let mut confirmed = false;

                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        canceled = ui.button("No").clicked();
                        confirmed = ui.button("Yes").clicked();

                        ui.label(format!(
                            "Are sure you want to {}?",
                            v.humantext(&self.data.entry)
                        ));
                    });

                    if confirmed {
                        self.applyaction();
                    }

                    if canceled {
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

                    let mut confirmed = false;

                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        // Close button
                        confirmed = ui.button("Done").clicked();

                        // Main text box
                        ui.add(
                            TextEdit::singleline(&mut self.data.entry.deepget_mut(v).title)
                                .desired_width(f32::INFINITY),
                        )
                        .request_focus();
                    });

                    if confirmed {
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

        // Check for hotkeys
        ctx.input_mut(|i| {
            if i.key_down(egui::Key::F4) {
                // Terminate program if f4 is down.
				self.action = CurrentAct::Confirm(Box::new(CurrentAct::Exit));
            }

            if i.key_down(egui::Key::Enter) {
                // Apply the current dialog if enter is down.
                self.applyaction();
            }

            if i.key_down(egui::Key::Escape) {
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
				Modal::new("info".into()).show(ctx, |ui| {self.dialogview(ui);});
			},
			DisplayType::Inline => {
				egui::TopBottomPanel::top("info").show(ctx, |ui| {self.dialogview(ui);});
			},
			DisplayType::None => {}
		}

        // construct body
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show all tasks
            self.show_tableview(ui);
			
        });

        // account for any applicable actions after layout paint
		match self.action {
			CurrentAct::Exit => {std::process::exit(0);},
			CurrentAct::Remove(_) | CurrentAct::Cleanup | CurrentAct::Sort => self.applyaction(),
			_ => {}
		}
		
		// Collect post-op hash of main model.
		let hnow = self.model_hash();

		if hbefore != hnow {
			// Perform automatic change actions when we have no pending action and there are changes to record.
			self.data.entry.sort();
			// Automatically save the list.

		}
    }
}

use eframe::egui::{
    self, popup_below_widget, Button, Color32, Frame, Id, Label, Layout, RichText, ScrollArea,
    TextEdit, Theme, Ui,
};
use model::{Entry, EntryHost, EntrySwitch, Model};
use serde::{Deserialize, Serialize};

use crate::app::{model::EntryEnd, recipes::togglepopup};
mod model;
mod recipes;

#[derive(PartialEq)]
enum CurrentAct {
    None,
    // Originally (and ideally), we use references to represent the target. This resulted in lifetime complications, so we use a (slightly more expensive) index based method now.
    Create(Option<Vec<usize>>, String),
    CreateHost(Option<Vec<usize>>, String),
    Edit(Vec<usize>),
    Remove(Vec<usize>),
}

// pretty simple current dialog
impl Default for CurrentAct {
    fn default() -> Self {
        Self::None
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
        Self {
            data: model::make_sample_set(),
            action: CurrentAct::None,
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
                let result = Self::drawentryhost(ui, &mut self.data.entry, vec![], false);
                // If we have a new action, set the current action
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

    // Wraps many drawentry(s) together.
    fn drawentryhost(
        ui: &mut Ui,
        subject: &mut EntryHost,
        index: Vec<usize>,
        displayeven: bool,
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
            let pending = Self::drawentry(ui, subsubject, newidx, displayeven);
            o.0 += pending.0;
            o.1 += pending.1;
            // The newest action is returned if it is not an else.
            o.2 = if pending.2 != None { pending.2 } else { o.2 };
        }

        o
    }

    // Shorthand for drawing an entry end
    fn drawentryend(
        ui: &mut Ui,
        title: String,
        subject: &mut EntryEnd,
        index: Vec<usize>,
    ) -> (usize, usize, Option<CurrentAct>) {
        // This is a node of the tree with full data.
        let mut o = (0, 0, None);
        let mut acts = (None, None);
        recipes::drawtriple(
            ui,
            |ui| {
                // Draw the check button. (you can't add onclick events to checkboxes)
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

                // Add to completion stats.
                // If this entry is complete, add label for date completed.
                o = if let Some(comp) = subject.completed {
                    ui.add(Label::new(
                        RichText::new(comp.format("%d/%m/%Y").to_string())
                            .size(10.0)
                            .weak(),
                    ));
                    (1, 0, None)
                } else {
                    (0, 1, None)
                };
            },
            |ui| {
                // Add title.
                if ui.label(title).double_clicked() {
                    // When doubleclicked, start editing this entry
                    acts.0 = Some(CurrentAct::Edit(index.clone()));
                };
            },
            |ui| {
                // Add edit button.
                // online egui release was **too new**. detailled popup tech gets added 1.32

                let popupid = Id::new(index.clone());

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
                        // This is where you put the switch edit code snoosk
                        ui.set_min_width(128.0);
                        if ui.button("Edit").clicked() {
                            acts.1 = Some(CurrentAct::Edit(index.clone()));
                        }
                        if ui.button("Delete").clicked() {
                            // uhhh wip, need to add a delete action
                            acts.1 = Some(CurrentAct::Remove(index.clone()));
                        }
                    },
                );
            },
        );
        // Collapse the pending actions.
        o.2 = acts.0;
        if o.2 == None {
            o.2 = acts.1
        }

        o
    }

    // Dictates how any entry should be drawn.
    fn drawentry(
        ui: &mut Ui,
        subject: &mut Entry,
        index: Vec<usize>,
        displayeven: bool,
    ) -> (usize, usize, Option<CurrentAct>) {
        let mut o = (0, 0, None);
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
                        o = Self::drawentryend(ui, subject.title.to_owned(), v, index);
                    } else if let EntrySwitch::Host(v) = &mut subject.data {
                        // This is a node of the tree with partial data.
                        ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                            // Draw the header of the box.

                            // Workaround for "waah you're going to start race conditions by writing to the same place";
                            // Prioritise certain actions through a tuple.
                            let mut actions = (None, None);
                            recipes::drawtriple(
                                ui,
                                |ui| {
                                    ui.add_space(20.0);
                                },
                                |ui| {
                                    // Draw title.
                                    if ui
                                        .add(Label::new(subject.title.to_owned()))
                                        .double_clicked()
                                    {
                                        // When double clicked, edit this
                                        actions.0 = Some(CurrentAct::Edit(index.clone()))
                                    }
                                },
                                |ui| {
                                    let popupid = Id::new(index.clone());

                                    let addbutton = ui.add_sized((20.0, 20.0), Button::new("+"));

                                    if addbutton.clicked() {
                                        // Raise add entry dialog on this entry host
                                        actions.1 = Some(CurrentAct::Create(
                                            Some(index.clone()),
                                            String::new(),
                                        ));
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
                                                actions.1 = Some(CurrentAct::Create(
                                                    Some(index.clone()),
                                                    String::new(),
                                                ));
                                            }
                                            if ui.button("Add sublist").clicked() {
                                                actions.1 = Some(CurrentAct::CreateHost(
                                                    Some(index.clone()),
                                                    String::new(),
                                                ));
                                            }
                                        },
                                    );
                                },
                            );
                            o.2 = actions.0;
                            if o.2 == None {
                                o.2 = actions.1
                            }

                            ui.add_space(4.0);

                            // Go recursive using inner entry host.
                            let totals = Self::drawentryhost(ui, v, index, !displayeven);

                            // Merge pending totals with output totals.
                            o.0 += totals.0;
                            o.1 += totals.1;

                            // If we are not raising a new entry on this host, replace the current action with the one collected from totals
                            // (technically wastes processing since we never have to even consider this when we're raising add entry. might be negligible but please consider later)
                            if o.2 == None {
                                o.2 = totals.2;
                            }

                            ui.add_space(4.0);

                            // Draw footer in small text if there's something to show.
                            if totals.0 + totals.1 >= 8 {
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
                            }
                        });
                    }
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
                CurrentAct::Create(v, title) => {
                    // Draw heading
                    ui.heading(match v {
                        None => "Create task".to_owned(),
                        Some(v) => {
                            format!("Create subtask for {}", self.data.entry.deepget(v).title)
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
                        self.applydialog();
                    }

                    if canceled | confirmed {
                        self.closedialog();
                    }
                }
                CurrentAct::CreateHost(v, title) => {
                    // Draw heading
                    ui.heading(match v {
                        None => "Create list".to_owned(),
                        Some(v) => {
                            format!("Create sublist for {}", self.data.entry.deepget(v).title)
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
                        self.applydialog();
                    }

                    if canceled | confirmed {
                        self.closedialog();
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
                            TextEdit::singleline(&mut self.data.entry.deepget(v).title)
                                .desired_width(f32::INFINITY),
                        )
                        .request_focus();
                    });

                    if confirmed {
                        self.action = CurrentAct::None;
                    }
                }
                _ => {}
            };
        });
    }

    // Apply an open dialog pane
    fn applydialog(&mut self) {
        match &mut self.action {
            CurrentAct::Create(subject, title) => {
                // Add task to location with new content
                let host = match subject {
                    None => &mut self.data.entry, // We're adding an entry to the root
                    Some(v) => {
                        // We're adding an entry to a child entry
                        match &mut self.data.entry.deepget(v).data {
                            EntrySwitch::Host(w) => w,
                            EntrySwitch::End(_) => {
                                panic!("Attempted to add an entry to a non-entryhost!")
                            }
                        }
                    }
                };
                host.subelements
                    .push(Entry::new_end(title.to_string(), "".to_string()));
                host.sort();
                self.closedialog();
            }
            CurrentAct::CreateHost(subject, title) => {
                // Add list to location with new title
                let host = match subject {
                    None => &mut self.data.entry,
                    Some(v) => match &mut self.data.entry.deepget(v).data {
                        EntrySwitch::Host(w) => w,
                        EntrySwitch::End(_) => {
                            panic!("Attempted to add an entryhost to a non-entryhost!")
                        }
                    },
                };
                host.subelements.push(Entry::new_host(title.to_string()));
                host.sort();
                self.closedialog();
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
                    &mut self.data.entry.deepget(&v[..len - 1]).data
                {
                    // Remove entry from nested entry host
                    w.subelements.remove(v[len - 1]);
                } else {
                    // An invalid index has been presented.
                    panic!("An invalid index was on a removal action.")
                };
                // Reset the action
                self.action = CurrentAct::None;
            }
            _ => self.closedialog(),
        }
    }

    // Close an open dialog pane
    fn closedialog(&mut self) {
        self.action = CurrentAct::None;
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

            if i.key_down(egui::Key::Enter) {
                // Apply the current dialog if enter is down.
                self.applydialog();
            }

            if i.key_down(egui::Key::Escape) {
                // Close the current dialog if enter is down.
                self.closedialog();
            }
        });

        ctx.set_theme(Theme::Dark);

        // construct navpanel
        egui::TopBottomPanel::top("navigation").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Wrap these elements in a panel for improved layout
                Frame::new().inner_margin(4.0).show(ui, |ui| {
                    if ui.button("Table view").clicked() {
                        // I don't know what this is going to do
                    }
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        let popupid = Id::new("addbuttonID");
                        let addbutton = ui.add_sized((20.0, 20.0), Button::new("+"));
                        if addbutton.clicked() {
                            // Raise add entry dialog on this entry host
                            self.action = CurrentAct::Create(None, String::new());
                        }

                        if addbutton.secondary_clicked() {
                            // Raise the popup if right clicked
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
                                    self.action = CurrentAct::Create(None, String::new());
                                }
                                if ui.button("Add sublist").clicked() {
                                    self.action = CurrentAct::CreateHost(None, String::new());
                                }
                            },
                        );
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

        // account for any removal actions after layout paint
        if let CurrentAct::Remove(_) = self.action {
            self.applydialog();
        }
    }
}

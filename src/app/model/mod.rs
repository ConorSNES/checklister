// This file defines the in-memory data model.

pub mod traits;

use std::cmp::Ordering;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::app::model::traits::Filterable;

pub fn make_sample_set() -> Model {
    Model {
        entry: EntryHost {
            subelements: vec![
                Entry {
                    title: "snusk".to_owned(),
                    data: EntrySwitch::End(EntryEnd {
                        body: "bjarg".to_owned(),
                        added: NaiveDateTime::default(),
                        completed: None,
                    }),
                },
                Entry {
                    title: "snusk2".to_owned(),
                    data: EntrySwitch::End(EntryEnd {
                        body: "bjarg".to_owned(),
                        added: NaiveDateTime::default(),
                        completed: Some(NaiveDateTime::default()),
                    }),
                },
                Entry {
                    title: "snusk3".to_owned(),
                    data: EntrySwitch::End(EntryEnd {
                        body: "bjarg".to_owned(),
                        added: NaiveDateTime::default(),
                        completed: None,
                    }),
                },
                Entry {
                    title: "snusklist".to_owned(),
                    data: EntrySwitch::Host(EntryHost {
                        subelements: vec![
                            Entry {
                                title: "subsnusk1".to_owned(),
                                data: EntrySwitch::End(EntryEnd {
                                    body: "bjarg".to_owned(),
                                    added: NaiveDateTime::default(),
                                    completed: Some(NaiveDateTime::default()),
                                }),
                            },
                            Entry {
                                title: "subsnusk2".to_owned(),
                                data: EntrySwitch::End(EntryEnd {
                                    body: "bjarg".to_owned(),
                                    added: NaiveDateTime::default(),
                                    completed: None,
                                }),
                            },
                            Entry {
                                title: "subsnusk3".to_owned(),
                                data: EntrySwitch::End(EntryEnd {
                                    body: "bjarg".to_owned(),
                                    added: NaiveDateTime::default(),
                                    completed: Some(NaiveDateTime::default()),
                                }),
                            },
                        ],
                    }),
                },
                Entry {
                    title: "snusk4".to_owned(),
                    data: EntrySwitch::End(EntryEnd {
                        body: "bjarg".to_owned(),
                        added: NaiveDateTime::default(),
                        completed: None,
                    }),
                },
            ],
        },
    }
}

// The data model contains only one entry host.
#[derive(Default, Serialize, Deserialize)]
pub struct Model {
    pub entry: EntryHost,
}

// Used to describe the switch of entries.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum EntrySwitch {
    End(EntryEnd),
    Host(EntryHost),
}

impl Filterable for EntrySwitch {
	fn visible(&self, filter: &str) -> bool {
		match self {
			Self::End(v) => v.visible(filter),
			Self::Host(v) => v.visible(filter)
		}
	}
}

// An entry contains either an array of entries or the main entry data.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Entry {
    pub title: String,
    pub data: EntrySwitch,
}

impl Entry {
    // Two functions that construct new entry ends and entry hosts respectively.
    pub fn new_end(title: String, body: String) -> Self {
        Entry {
            title: title,
            data: EntrySwitch::End(EntryEnd {
                body: body,
                added: Local::now().naive_local(),
                completed: None,
            }),
        }
    }

    pub fn new_host(title: String) -> Self {
        Entry {
            title: title,
            data: EntrySwitch::Host(EntryHost {
                subelements: vec![],
            }),
        }
    }

    // Collects added date of Entry, regardless of switched content. As a result, this is a copy value.
    pub fn added(&self) -> NaiveDateTime {
        match &self.data {
            EntrySwitch::End(v) => v.added,
            EntrySwitch::Host(v) => v.date(),
        }
    }

    // Collects completed date of Entry, regardless of switched content. As a result, this is a copy value.
    pub fn completed(&self) -> Option<NaiveDateTime> {
        match &self.data {
            EntrySwitch::End(v) => v.completed,
            EntrySwitch::Host(v) => v.completed(),
        }
    }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Handle completed task logic.
        let cmp = self
            .completed()
            .is_none()
            .cmp(&other.completed().is_none())
            .reverse();
        if cmp != Ordering::Equal {
            return cmp;
        }

        // Otherwise, return the comparison of the two dates.
        self.added().cmp(&other.added()).reverse()
    }
}

impl Filterable for Entry {
	fn visible(&self, filter: &str) -> bool {
		// Test filter on entry title
		if self.title.contains(filter) {return true;}
		// Otherwise, return result of nested
		self.data.visible(filter)
	}
}

// The main content of an entry.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct EntryEnd {
    pub body: String,
    pub added: NaiveDateTime,
    pub completed: Option<NaiveDateTime>,
}

impl EntryEnd {
    // Toggle completed state.
    pub fn toggle(&mut self) {
        self.completed = match self.completed {
            None => Some(Local::now().naive_local()),
            Some(_) => None,
        };
    }
}

impl Filterable for EntryEnd {
    fn visible(&self, _filter: &str) -> bool {
        false
    }
}

// An array of entries.
#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct EntryHost {
    pub subelements: Vec<Entry>,
}

impl EntryHost {
    // Function that performs recursive traversal of entry hosts
    pub fn deepget_mut(&mut self, index: &[usize]) -> &mut Entry {
        // If the vector is empty, panic
        if index.len() == 0 {
            panic!("No indexes provided!")
        };

        // If this is the most shallow the index goes, return the current item.
        if index.len() == 1 {
            &mut self.subelements[index[0]]
        }
        // If this is not, try and access the deeper one
        else {
            let EntrySwitch::Host(v) = &mut self.subelements[index[0]].data else {
                panic!("Entryhost deepget ended early! Index: {:#?}", index);
            };
            // Perform recursive deepget with slice excluding current index
            v.deepget_mut(&index[1..])
        }
    }

    pub fn deepget(&self, index: &[usize]) -> &Entry {
        // If the vector is empty, panic
        if index.len() == 0 {
            panic!("No indexes provided!")
        };

        // If we're just fetching from this, return contained entry from this
        if index.len() == 1 {
            &self.subelements[index[0]]
        }
        // Go deeper
        else {
            let EntrySwitch::Host(v) = &self.subelements[index[0]].data else {
                panic!("Entryhost deepget ended early! Index: {:#?}", index);
            };
            // Perform recursive deepget with slice excluding current index
            v.deepget(&index[1..])
        }
    }

    // Collect newest date of host
    pub fn date(&self) -> NaiveDateTime {
        // basically a max aggregator for one class value, with some fanangling
        let mut max = NaiveDateTime::MIN;
        for v in &self.subelements {
            max = NaiveDateTime::max(max, v.added());
        }
        max
    }

    // Collect newest completed date of host
    // Terminates early as None if not all subelements are complete
    pub fn completed(&self) -> Option<NaiveDateTime> {
        let mut max = NaiveDateTime::MIN;

        for v in &self.subelements {
            match v.completed() {
                None => return None,
                Some(w) => max = max.max(w),
            }
        }

        Some(max)
    }

    // Recursive sort for entryhost contents
    pub fn sort(&mut self) {
        // Sort the internals if we're able
        for v in &mut self.subelements {
            if let EntrySwitch::Host(w) = &mut v.data {
                w.sort();
            }
        }
        // Now sort this entry
        self.subelements.sort();
    }

    // Clean the completed entries within this host
    pub fn cleanup(&mut self) {
        // Iteration is done in reverse order to account for shrinking array.
        for i in (0..self.subelements.len()).rev() {
            match &mut self.subelements[i].data {
                EntrySwitch::Host(w) => {
                    w.cleanup();
                    if w.subelements.len() == 0 {
                        self.subelements.remove(i);
                    }
                }
                EntrySwitch::End(w) => {
                    if w.completed != None {
                        self.subelements.remove(i);
                    }
                }
            }
        }
    }
}

impl Filterable for EntryHost {
	// Visible status of an entry host (if this host contains a visible entry, this host is visible)
	fn visible(&self, filter: &str) -> bool {
		for v in &self.subelements {
            if v.visible(filter) {
                return true;
            }
        }
        false
	}
}

// This file defines the in-memory data model.

use std::cmp::Ordering;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize)]
pub enum EntrySwitch {
    End(EntryEnd),
    Host(EntryHost),
}

// An entry contains either an array of entries or the main entry data.
#[derive(Serialize, Deserialize)]
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

	// currently dead- need a menu to add a host
    pub fn _new_host(title: String) -> Self {
        Entry {
            title: title,
            data: EntrySwitch::Host(EntryHost {
                subelements: vec![],
            }),
        }
    }

	// Collects added date of Entry, regardless of it being a Host or End.
	pub fn added(&self) -> NaiveDateTime {
		match &self.data {
			EntrySwitch::End(v) => v.added,
			EntrySwitch::Host(v) => v.date()
		}
	}
}

// The main content of an entry.
#[derive(Serialize, Deserialize)]
pub struct EntryEnd {
    pub body: String,
    pub added: NaiveDateTime,
    pub completed: Option<NaiveDateTime>,
}

// An array of entries.
#[derive(Serialize, Deserialize, Default)]
pub struct EntryHost {
    pub subelements: Vec<Entry>,
}

impl EntryHost {
    // Function that performs recursive traversal of entry hosts
    pub fn deepget(&mut self, index: &[usize]) -> &mut Entry {
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
            match &mut self.subelements[index[0]].data {
                EntrySwitch::End(_) => panic!("Entryhost deepget ended early! Index: {:#?}", index),
                // Otherwise, go recursive
                EntrySwitch::Host(v) => v.deepget(&index[1..]),
            }
        }
    }

	// Collect newest date of host
	pub fn date(&self) -> NaiveDateTime {
		// basically a max aggregator for one class value, with some fanangling
		let mut max = NaiveDateTime::MIN;
		for v in &self.subelements {
			max = NaiveDateTime::max(max, v.added());
		};
		max
	}

	// Collect newest completed date of host
	// (it's a little more nuanced than that, but that covers the basic idea)
	// unf todo

	// Recursive sort for entryhost contents
	pub fn sort(&mut self) {
		// Sort the internals if we're able
		for v in &mut self.subelements {
			if let EntrySwitch::Host(w) = &mut v.data { w.sort(); }
		}
		// Now sort this entry
		self.subelements.sort_by(|subj1, subj2| { 
			let s1date = subj1.added();
			let s2date = subj2.added();
			if s1date == s2date { return Ordering::Equal };
			return if !(s1date < s2date) { Ordering::Less } else { Ordering::Greater }
		});
	}
}

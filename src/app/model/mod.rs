// This file defines the in-memory data model.

use chrono::NaiveDateTime;
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
							}
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

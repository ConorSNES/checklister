// This file defines the in-memory data model.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Model {
	pub entry : EntryHost
}

#[derive(Serialize, Deserialize)]
pub enum EntrySwitch {
	End(EntryEnd),
	Host(EntryHost),
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
	pub title: String,
	pub data: EntrySwitch
}

#[derive(Serialize, Deserialize)]
pub struct EntryEnd {
	pub body: String,
	pub added: NaiveDateTime,
	pub completed: Option<NaiveDateTime>
}

#[derive(Serialize, Deserialize, Default)]
pub struct EntryHost {
	pub subelements: Vec<Entry>
}
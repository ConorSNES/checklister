use crate::app::{displaytyped::DisplayType, model::EntryHost, DisplayTyped};
use std::path::PathBuf;

// Define the CurrentAct enum

#[derive(PartialEq, Clone)]
pub enum CurrentAct {
    None,
    Confirm(Box<CurrentAct>),
	Error(String),
    // Originally (and ideally), we use references to represent the target. This resulted in lifetime complications, so we use a (slightly more expensive?) index based method now.
    Create(Option<Vec<usize>>, String), // Create an ending member of targeted vector
    CreateHost(Option<Vec<usize>>, String), // Create a host member of targeted vector
    Edit(Vec<usize>),
    Remove(Vec<usize>),
	New,
	Import(PathBuf),
	Export(PathBuf),
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
	pub fn some(&self) -> bool {
		*self != Self::None
	}
    
    // Function that provides rich translation of current action to english text
    pub fn humantext(&self, subject: &EntryHost) -> String {
        match self {
            Self::None => "do nothing".to_owned(),
            Self::Confirm(v) => v.as_ref().humantext(subject),
            Self::Remove(v) => format!("remove '{}'", subject.deepget(v).title.to_owned()),
            Self::Cleanup => "remove all completed tasks".to_owned(),
			Self::New => "start a new task list (previous data will be lost)".to_owned(),
			Self::Import(v) => format!("import the list '{}' (previous data will be lost)", v.display()),
			Self::Exit => "exit".to_owned(),
            _ => "undefined".to_owned(),
        }
    }
}

impl DisplayTyped for CurrentAct {
    fn display(&self) -> DisplayType {
        match self {
			Self::None | Self::Cleanup | Self::Sort => DisplayType::None,
			Self::Error(_) | Self::Confirm(_) | Self::Create(_, _) | Self::CreateHost(_, _) => DisplayType::Modal,
			Self::Find(_) | Self::Edit(_) => DisplayType::Inline,
			_ => DisplayType::None
		}
    }
}
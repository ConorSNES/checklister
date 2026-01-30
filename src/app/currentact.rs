use crate::app::{displaytyped::DisplayType, model::EntryHost, DisplayTyped};
use std::path::PathBuf;

// Define the CurrentAct enum

#[derive(PartialEq, Clone, Hash)]
pub enum CurrentAct {
    None,
    Confirm(Box<CurrentAct>),
    Error(String),
    Message(String, String), // title, body
    // Originally (and ideally), we use references to represent the target. This resulted in lifetime complications, so we use a (slightly more expensive?) index based method now.
    Create(Option<Vec<usize>>, String), // Create an ending member of targeted vector
    CreateHost(Option<Vec<usize>>, String), // Create a host member of targeted vector
    Edit(Vec<usize>), // Edit title of an entry
    Info(Vec<usize>), // View info of an entry
    Remove(Vec<usize>), // Remove entry
    New, // Create a new list in memory (flush all)
    Import(PathBuf), // Import list from file
    Export(PathBuf), // Export list from file
    Cleanup, // Run cleanup cycle on list in memory (see EntryHost.cleanup())
    Sort, // Run sort on lise (see EntryHost.sort())
    Exit, // Exit program
    Find(String), // Find utility (string is subject)
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
            Self::Import(v) => format!(
                "import the list '{}' (previous data will be lost)",
                v.display()
            ),
            Self::Exit => "exit".to_owned(),
            _ => "undefined".to_owned(),
        }
    }
}

impl DisplayTyped for CurrentAct {
    fn display(&self) -> DisplayType {
        match self {
            Self::None | Self::Cleanup | Self::Sort => DisplayType::None,

            Self::Error(_)
            | Self::Message(_, _)
            | Self::Confirm(_)
            | Self::Create(_, _)
            | Self::CreateHost(_, _)
            | Self::Info(_) => DisplayType::Modal,

            Self::Find(_) | Self::Edit(_) => DisplayType::Inline,

            _ => DisplayType::None,
        }
    }
}

// Define the consts
pub fn appinfo() -> CurrentAct {
    // Collect values
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let licence = "Apache-2.0";

    CurrentAct::Message("Application Info".to_owned(), 
        format!("{}

Version:  {}
Authored by Conor SS 2025-2026
Software licence: {}",
        name, version, licence
    ))
}

pub fn hints() -> CurrentAct {
    // Return application hints
    CurrentAct::Message("Hints".to_owned(), 
    "Your changes are saved automatically alongside user preferences.

Add buttons (\"+\") can be right-clicked for more options.

The list is sorted by completion date, then by creation date (newest first).

Task lists can help keep related tasks together through sorting.

Any submenu can be accepted using Enter, and closed using Escape.".to_owned())
}

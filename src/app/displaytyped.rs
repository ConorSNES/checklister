
// How is this enum (mainly the ) to be visually represented?
pub enum DisplayType {
	None,
	Modal,
	Inline,
}

pub trait DisplayTyped {
    fn display(&self) -> DisplayType;
}


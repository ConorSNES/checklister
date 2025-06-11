pub trait Filterable {
	// Validates if this particular item is visible when filtered by this string
    fn visible(&self, filter: &str) -> bool;
}

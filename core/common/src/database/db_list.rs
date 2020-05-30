use crate::serde::Serialize;

/// List of a Database Object
#[derive(Debug, Clone, Serialize, Hash)]
pub struct DbList<T> {
    /// Internal Data List
    pub data: Vec<T>,
    /// Number of entires.
    pub count: usize,
    /// The page associated with the list.
    /// May be bigger then page_max if requested from the user
    pub page: usize,
    /// The number of pages available
    pub page_max: usize,
}

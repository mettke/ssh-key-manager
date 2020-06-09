use crate::{
    database::{Database, DbList, DbResult},
    sec::Auth,
};

/// Allows fetching multiple Objects using a Filter
pub trait FetchAll<'a, A: Auth, T, F: ?Sized, D: Database>: Sized {
    /// Fetches multiple objects using a Filter.
    /// Returns an empty array if there are no objects
    /// or when the user is not allowed to view given object.
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    fn fetch_all(
        &self,
        filter: &'a F,
        auth: &'a A,
        page: usize,
    ) -> DbResult<DbList<T>, D>;
}

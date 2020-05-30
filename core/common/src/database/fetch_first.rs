use crate::{
    database::{Database, DbResult},
    sec::Auth,
};

/// Allows fetching the first Object matching a Filter
pub trait FetchFirst<A: Auth, T, F, D: Database>: Sized {
    /// Fetches the first object matching a Filter.
    /// Returns `Ok(None)` if there is no object with that id
    /// or when the user is not allowed to view given objects.
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    fn fetch_first(&self, filter: &F, auth: &A) -> DbResult<Option<T>, D>;
}

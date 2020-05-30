use crate::{
    database::{Database, DbList, DbResult},
    sec::Auth,
};

/// Allows fetching multiple Objects using a Filter
pub trait FetchAllFor<A: Auth, T, F, D: Database>: Sized {
    /// Fetches multiple objects using a Filter.
    /// Returns an empty array if there are no objects
    /// or when the user is not allowed to view given object.
    /// Difference to `FetchAll` is that admins are treated like users
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    fn fetch_all_for(
        &self,
        filter: &F,
        auth: &A,
        page: usize,
    ) -> DbResult<DbList<T>, D>;
}

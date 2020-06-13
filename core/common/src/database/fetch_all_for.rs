use crate::{
    async_trait::async_trait,
    database::{Database, DbList, DbResult},
    sec::Auth,
};
use std::marker::PhantomData;

/// Allows fetching multiple Objects using a Filter
#[async_trait]
pub trait FetchAllFor<'a, 'b, A: Auth, T, F, D: Database>: Sized {
    /// Fetches multiple objects using a Filter.
    /// Returns an empty array if there are no objects
    /// or when the user is not allowed to view given object.
    /// Difference to `FetchAll` is that admins are treated like users
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    async fn fetch_all_for(
        &self,
        filter: &F,
        auth: &A,
        page: usize,
        _: PhantomData<(&'a (), &'b ())>,
    ) -> DbResult<DbList<T>, D>;
}

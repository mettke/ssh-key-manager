use crate::{
    async_trait::async_trait,
    database::{Database, DbResult},
    sec::Auth,
};
use std::marker::PhantomData;

/// Allows fetching the first Object matching a Filter
#[async_trait]
pub trait FetchFirst<'a, 'b, A: Auth, T, F, D: Database>: Sized {
    /// Fetches the first object matching a Filter.
    /// Returns `Ok(None)` if there is no object with that id
    /// or when the user is not allowed to view given objects.
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    async fn fetch_first(
        &self,
        filter: &F,
        auth: &A,
        _: PhantomData<(&'a (), &'b ())>,
    ) -> DbResult<Option<T>, D>;
}

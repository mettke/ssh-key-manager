use crate::{
    async_trait::async_trait,
    database::{Database, DbResult},
    sec::Auth,
    types::Id,
};
use std::marker::PhantomData;

/// Allows fetching a single Object
#[async_trait]
pub trait FetchById<'a, A: Auth, T, D: Database>: Sized {
    /// Fetches a single Object using its Id.
    /// Returns `Ok(None)` if there is no object with that id
    /// or when the user is not allowed to view given object.
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    async fn fetch(
        &self,
        id: &Id,
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<Option<T>, D>;
}

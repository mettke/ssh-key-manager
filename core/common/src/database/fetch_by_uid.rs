use crate::{
    async_trait::async_trait,
    database::{Database, DbResult},
    sec::Auth,
};
use std::marker::PhantomData;

/// Allows fetching a single Object
#[async_trait]
pub trait FetchByUid<'a, A: Auth, T, D: Database>: Sized {
    /// Fetches a single Object using its uid.
    /// Returns `Ok(None)` if there is no object with that uid
    /// or when the user is not allowed to view given object.
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    async fn fetch_by_uid(
        &self,
        uid: &str,
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<Option<T>, D>;
}

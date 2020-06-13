use crate::{
    async_trait::async_trait,
    database::{Database, DbResult},
    sec::Auth,
    types::Id,
};
use std::marker::PhantomData;

/// Allows creating new objects
#[async_trait]
pub trait Delete<'a, A: Auth, T, D: Database>: Sized {
    /// Deletes one or more objects.
    /// Does not error if objects do not exist
    ///
    /// # Errors
    /// Fails on
    /// * Connection Errors
    async fn delete(
        &self,
        ids: &[Id],
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), D>;
}

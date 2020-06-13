use crate::{
    async_trait::async_trait,
    database::{Database, DbResult},
    sec::Auth,
};
use std::marker::PhantomData;

/// Allows saving changes made to an object
#[async_trait]
pub trait Save<'a, A: Auth, T, D: Database>: Sized {
    /// Save modifications made to an Object.
    /// Does not fail if object does not exist
    ///
    /// # Errors
    /// Fails on
    /// * Connection Errors
    /// * Unique Constraints
    async fn save(
        &self,
        object: &T,
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), D>;
}

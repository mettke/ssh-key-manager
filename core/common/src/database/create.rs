use crate::{
    async_trait::async_trait,
    database::{Database, DbResult},
    sec::Auth,
};
use std::marker::PhantomData;

/// Allows creating new objects
#[async_trait]
pub trait Create<'a, A: Auth, T, D: Database>: Sized {
    /// Creates a new Object.
    ///
    /// # Errors
    /// Fails on
    /// * Connection Errors
    /// * Unique Constraints
    async fn create(
        &self,
        object: &T,
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), D>;
}

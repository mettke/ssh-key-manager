use crate::{
    async_trait::async_trait,
    database::{Database, DbResult},
    sec::Auth,
};
use std::marker::PhantomData;

/// Allows deleting given object
#[async_trait]
pub trait DeleteObj<'a, A: Auth, T, D: Database>: Sized {
    /// Deletes the given object.
    /// Does not error if objects do not exist
    ///
    /// # Errors
    /// Fails on
    /// * Connection Errors
    async fn delete_obj(
        &self,
        object: &T,
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), D>;
}

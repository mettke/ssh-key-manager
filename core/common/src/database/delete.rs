use crate::{
    database::{Database, DbResult},
    sec::Auth,
    types::Id,
};

/// Allows creating new objects
pub trait Delete<A: Auth, T, D: Database>: Sized {
    /// Deletes one or more objects.
    /// Does not error if objects do not exist
    ///
    /// # Errors
    /// Fails on
    /// * Connection Errors
    fn delete(&self, ids: &[Id], auth: &A) -> DbResult<(), D>;
}

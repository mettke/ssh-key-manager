use crate::{
    database::{Database, DbResult},
    sec::Auth,
};

/// Allows saving changes made to an object
pub trait Save<A: Auth, T, D: Database>: Sized {
    /// Save modifications made to an Object.
    /// Does not fail if object does not exist
    ///
    /// # Errors
    /// Fails on
    /// * Connection Errors
    /// * Unique Constraints
    fn save(&self, object: &T, auth: &A) -> DbResult<(), D>;
}

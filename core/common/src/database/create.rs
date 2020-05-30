use crate::{
    database::{Database, DbResult},
    sec::Auth,
};

/// Allows creating new objects
#[allow(single_use_lifetimes)]
pub trait Create<A: Auth, T, D: Database>: Sized {
    /// Creates a new Object.
    ///
    /// # Errors
    /// Fails on
    /// * Connection Errors
    /// * Unique Constraints
    fn create(&self, object: &T, auth: &A) -> DbResult<(), D>;
}

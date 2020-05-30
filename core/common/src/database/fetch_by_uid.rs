use crate::{
    database::{Database, DbResult},
    sec::Auth,
};

/// Allows fetching a single Object
#[allow(single_use_lifetimes)]
pub trait FetchByUid<A: Auth, T, D: Database>: Sized {
    /// Fetches a single Object using its uid.
    /// Returns `Ok(None)` if there is no object with that uid
    /// or when the user is not allowed to view given object.
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    fn fetch_by_uid(&self, uid: &str, auth: &A) -> DbResult<Option<T>, D>;
}

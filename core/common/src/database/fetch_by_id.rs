use crate::{
    database::{Database, DbResult},
    sec::Auth,
    types::Id,
};

/// Allows fetching a single Object
#[allow(single_use_lifetimes)]
pub trait FetchById<'a, A: Auth, T, D: Database>: Sized {
    /// Fetches a single Object using its Id.
    /// Returns `Ok(None)` if there is no object with that id
    /// or when the user is not allowed to view given object.
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// May not fail on input errors.
    fn fetch(&self, id: &'a Id, auth: &A) -> DbResult<Option<T>, D>;
}

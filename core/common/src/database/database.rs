use crate::{database::DatabaseError, types::Id};
use std::{borrow::Cow, error, fmt::Debug};

/// Common methods for Database Implementations
pub trait Database: Sized + Send + Sync + Debug {
    /// the custom error for the template engine.
    type DatabaseError: error::Error;

    /// Generates an id
    ///
    /// # Errors
    /// Fails on connection or deserialisation errors.
    fn generate_id(&self) -> Result<Id, DatabaseError<Self>>;

    /// Fetches a permission id array from the database for an entity
    /// The Array contains the id of the entity itself and all the
    /// groups the entity is a member in.
    ///
    /// # Errors
    /// Fails on connection or deserialisation errors.
    fn fetch_permission_ids<'a>(
        &self,
        entity_id: Cow<'a, Id>,
    ) -> Result<Vec<Cow<'a, Id>>, DatabaseError<Self>>;

    /// Applies outstanding migrations
    ///
    /// # Errors
    /// Fails on connection or deserialisation errors.
    fn migrate(&self) -> Result<(), DatabaseError<Self>>;
}

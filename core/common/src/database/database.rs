use crate::{async_trait::async_trait, database::DatabaseError, types::Id};
use std::{borrow::Cow, error, fmt::Debug};

/// Common methods for Database Implementations
#[async_trait]
pub trait Database: Sized + Send + Sync + Debug {
    /// the custom error for the template engine.
    type DatabaseError: error::Error + Send + Sync;

    /// Generates an id
    ///
    /// # Errors
    /// Fails on connection or deserialisation errors.
    async fn generate_id(&self) -> Result<Id, DatabaseError<Self>>;

    /// Fetches a permission id array from the database for an entity
    /// The Array contains the id of the entity itself and all the
    /// groups the entity is a member in.
    ///
    /// # Errors
    /// Fails on connection or deserialisation errors.
    async fn fetch_permission_ids<'a>(
        &self,
        entity_id: Cow<'a, Id>,
    ) -> Result<Vec<Cow<'a, Id>>, DatabaseError<Self>>;

    /// Applies outstanding migrations
    ///
    /// # Errors
    /// Fails on connection or deserialisation errors.
    async fn migrate(&self) -> Result<(), DatabaseError<Self>>;
}

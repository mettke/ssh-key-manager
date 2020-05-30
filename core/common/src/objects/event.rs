use crate::{
    chrono::NaiveDateTime,
    serde::Serialize,
    types::{EventTypes, Id},
};
use std::borrow::Cow;

/// Defines an Event which description an Action take by a user
/// inside the Application
#[derive(Debug, Clone, Hash, Serialize)]
pub struct Event<'a> {
    /// The id which uniquely identifies the Event
    pub id: Cow<'a, Id>,
    /// The entity who did the change
    pub actor_id: Option<Cow<'a, Id>>,
    /// The date and time when the change was registered
    pub date: Option<NaiveDateTime>,
    /// Further details about the change
    pub details: Cow<'a, str>,
    /// The type of the object the change was made on
    pub type_: EventTypes,
    /// The id of the object the change was made on
    pub object_id: Option<Cow<'a, Id>>,
}

/// Provides fields to filter when searching for multiple
/// objects
#[derive(Debug, Clone, Hash, Serialize)]
pub struct EventFilter<'a> {
    /// The entity who did the change must equal to this id
    pub actor_id: Option<Cow<'a, Id>>,
    /// The Event information must be like
    pub details: Option<Cow<'a, str>>,
    /// The object the operation was made on must equal
    pub object_id: Option<Cow<'a, Id>>,
}

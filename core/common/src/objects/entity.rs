use crate::{
    serde::Serialize,
    types::{EntityTypes, Id},
};
use std::borrow::Cow;

#[derive(Debug, Clone, Hash, Serialize)]
/// Generic Representation of a `User`, a `Group` or a `Serveraccount`
pub struct Entity<'a> {
    /// The id which uniquely identifies the user
    pub entity_id: Cow<'a, Id>,
    /// The name of the user
    pub name: Option<Cow<'a, str>>,
    /// Id of the associated server. Only available when this is a `Serveraccount`
    pub server_id: Option<Cow<'a, Id>>,
    /// Hostname of the associated server. Only available when this is a `Serveraccount`
    pub server_name: Option<Cow<'a, str>>,
    /// The type of the user
    pub type_: Option<EntityTypes>,
}

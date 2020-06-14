use crate::{serde::Serialize, types::Id};
use std::borrow::Cow;

#[derive(Debug, Clone, Hash, Serialize)]
/// Allows querying group members
pub struct GroupMember<'a, Inner> {
    /// Id of the group which is associated with this membership
    pub group_id: Cow<'a, Id>,
    /// Inner object representing the member
    pub member: Inner,
    /// Time when the object was added to the group
    pub add_date: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Hash, Serialize)]
/// Allows querying group members
pub struct GroupMemberEntry<'a> {
    /// Id of the group which is associated with this membership
    pub group_id: Cow<'a, Id>,
    /// Inner object representing the member
    pub member: Cow<'a, Id>,
}

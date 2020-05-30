use crate::{serde::Serialize, types::Id};
use std::borrow::Cow;

#[derive(Debug, Clone, Hash, Serialize)]
/// Defines the Group structure in the database
pub struct Group<'a> {
    /// The id which uniquely identifies the group
    pub entity_id: Cow<'a, Id>,
    /// The name of the group
    pub name: Cow<'a, str>,
    /// Whether it is a system group. System groups cannot be deleted
    pub system: bool,
    /// The oauth_scope which automatically adds a user with that scope to the group
    pub oauth_scope: Option<Cow<'a, str>>,
    /// The ldap_group which automatically adds a user of that group to this group
    pub ldap_group: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Hash, Serialize)]
/// Provides fields to filter when searching for multiple
/// objects
pub struct GroupFilter<'a> {
    /// The name of the group must be like this value
    pub name: Option<Cow<'a, str>>,
}

impl Default for GroupFilter<'_> {
    #[inline]
    fn default() -> Self {
        Self { name: None }
    }
}

impl<'a, I: Iterator<Item = (Cow<'a, str>, Cow<'a, str>)>> From<I>
    for GroupFilter<'a>
{
    #[inline]
    fn from(iter: I) -> Self {
        let mut filter = Self::default();
        for (key, val) in iter {
            if val.is_empty() {
                continue;
            }
            if let "type" = key.as_ref() {
                filter.name = Some(val);
            }
        }
        filter
    }
}

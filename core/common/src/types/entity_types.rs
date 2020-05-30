use crate::serde::{Deserialize, Serialize};
use core_macros::EnumFrom;
use std::convert::TryFrom;

/// The Type of an entity. Defines in which table
/// the program should search for the object
#[derive(
    Debug, Copy, Clone, Hash, EnumFrom, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum EntityTypes {
    /// Entity represents a user
    User,
    /// Entity represents a server account
    ServerAccount,
    /// Entity represents a group
    Group,
}

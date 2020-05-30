use crate::serde::{Deserialize, Serialize};
use core_macros::EnumFrom;
use std::convert::TryFrom;

/// The Type of an event. Represents which object the id refers to
#[derive(
    Debug, Copy, Clone, Hash, EnumFrom, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum EventTypes {
    /// Id refers to an server object
    Server,
    /// Id refers to an entity object
    Entity,
}

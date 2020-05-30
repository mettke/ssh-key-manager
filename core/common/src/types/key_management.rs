use crate::serde::{Deserialize, Serialize};
use core_macros::EnumFrom;
use std::convert::TryFrom;

/// Key Management used for a given Server
/// The Application does not make a difference between `KeyManagement::None` and
/// `KeyManagement::Other`. Its only cosmetical.
#[derive(
    Debug, Copy, Clone, Hash, EnumFrom, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum KeyManagement {
    /// No key management in place. Made for systems who only provide users
    None,
    /// Keys are managed by the Application and get overriden on deployment
    Keys,
    /// Keys are managed by another system.
    Other,
}

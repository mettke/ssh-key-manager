use crate::serde::{Deserialize, Serialize};
use core_macros::EnumFrom;
use std::convert::TryFrom;

/// The Types a user can be
///
/// Admin tiering must prevent someone
/// from being a superuser and an admin at once.
#[derive(
    Debug, Copy, Clone, Hash, EnumFrom, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum UserTypes {
    /// Normal user - might be a local admin for a server or a group
    User,
    /// Admin user which has full access to the application
    /// (but not its configuration)
    Admin,
    /// Superuser which has full access to the application configuration
    /// but is not allowed to use the application itself
    Superuser,
}

use crate::serde::{Deserialize, Serialize};
use core_macros::EnumFrom;
use std::convert::TryFrom;

/// Authorization Type used for a given Server
#[derive(
    Debug, Copy, Clone, Hash, EnumFrom, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum AuthorizationType {
    /// Each User must be manually authorised to an server_account
    Manual,
    /// Each user is automatically assigned to a server_account with the same name
    Automatic,
}

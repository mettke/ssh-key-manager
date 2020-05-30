use crate::serde::{Deserialize, Serialize};
use core_macros::EnumFrom;
use std::convert::TryFrom;

/// Sync Status for a Server or a Server Account
#[derive(
    Debug, Copy, Clone, Hash, EnumFrom, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum SyncStatusType {
    /// So far no synchronisation took place. Default status for new servers
    NotSyncedYet,
    /// Synchronisation was successfully. One can expect the server configuration to
    /// match the one on the application
    SyncSuccess,
    /// Synchronisation failed and did not complete. Details are found in the
    /// Activity Table
    SyncFailure,
    /// Synchronsation encountered a warning. Server should work as expected but
    /// might not be ideally configured
    SyncWarning,
}

impl Default for SyncStatusType {
    #[inline]
    fn default() -> Self {
        Self::NotSyncedYet
    }
}

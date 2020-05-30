use crate::serde::{Deserialize, Serialize};
use core_macros::EnumFrom;
use std::convert::TryFrom;

/// The Access restriction Types of an ssh key
#[derive(
    Debug, Copy, Clone, Hash, EnumFrom, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum AccessOption {
    /// Restrict to a set of commands
    Command,
    /// Restrict to a set of ips
    From,
    /// Set environment variables
    Environment,
    /// Disable Agent forwarding
    NoAgentForwarding,
    /// Disable port forwarding
    NoPortForwarding,
    /// Disable pty creation
    NoPty,
    /// Disable X11 forwarding
    NoX11Forwarding,
    /// Disable the usage of the user rc file
    NoUserRc,
}

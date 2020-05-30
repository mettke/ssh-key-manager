use crate::{
    serde::Serialize,
    types::{AuthorizationType, Id, KeyManagement, SyncStatusType},
};
use std::{borrow::Cow, convert::TryFrom};

#[derive(Debug, Clone, Hash, Serialize)]
/// Defines the Server structure in the database
pub struct Server<'a> {
    /// The id which uniquely identifies the key
    pub id: Cow<'a, Id>,
    /// The hostname of the server
    pub hostname: Cow<'a, str>,
    /// The ip address associated with the server
    pub ip_address: Option<Cow<'a, str>>,
    /// The display name
    pub name: Option<Cow<'a, str>>,
    /// The Key Management configuration
    pub key_management: KeyManagement,
    /// The Authorisation Configuration
    pub authorization: AuthorizationType,
    /// The last synchronistaion Status
    pub sync_status: SyncStatusType,
    /// The SSH Rsa Key Fingerprint
    pub rsa_key_fingerprint: Option<Cow<'a, str>>,
    /// The ssh port
    pub port: i32,
}

#[derive(Debug, Clone, Hash, Serialize)]
/// Provides fields to filter when searching for multiple
/// objects
pub struct ServerFilter<'a> {
    /// Server Hostname must be like this value
    pub hostname: Option<Cow<'a, str>>,
    /// Ip Address must be equal to this value
    pub ip_address: Option<Cow<'a, str>>,
    /// Name must be like this value
    pub name: Option<Cow<'a, str>>,
    /// Key Management must be equal to any of these values
    pub key_management: Option<Cow<'a, [KeyManagement]>>,
    /// Sync Status must be equal to any of these values
    pub sync_status: Option<Cow<'a, [SyncStatusType]>>,
    /// Contains the id of the user and all of its groups.
    /// Limits server list to those who either are admined or
    /// can be accessed by any of these ids
    pub permission_ids: Option<Cow<'a, [Cow<'a, Id>]>>,
}

impl Default for ServerFilter<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            hostname: None,
            ip_address: None,
            name: None,
            key_management: None,
            sync_status: None,
            permission_ids: None,
        }
    }
}

impl<'a, I: Iterator<Item = (Cow<'a, str>, Cow<'a, str>)>> From<I>
    for ServerFilter<'a>
{
    #[inline]
    fn from(iter: I) -> Self {
        let mut filter = Self::default();
        let mut key_management = vec![];
        let mut sync_status = vec![];
        for (key, val) in iter {
            if val.is_empty() {
                continue;
            }
            match key.as_ref() {
                "hostname" => {
                    filter.hostname = Some(val);
                }
                "ip_address" => {
                    filter.ip_address = Some(val);
                }
                "name" => {
                    filter.name = Some(val);
                }
                "key_management" => {
                    if let Ok(v) = KeyManagement::try_from(val.as_ref()) {
                        key_management.push(v);
                    }
                }
                "sync_status" => {
                    if let Ok(v) = SyncStatusType::try_from(val.as_ref()) {
                        sync_status.push(v);
                    }
                }
                _ => {}
            }
        }
        if !key_management.is_empty() {
            filter.key_management = Some(Cow::Owned(key_management));
        }
        if !sync_status.is_empty() {
            filter.sync_status = Some(Cow::Owned(sync_status));
        }
        filter
    }
}

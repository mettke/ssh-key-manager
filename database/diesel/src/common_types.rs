use crate::{DbFrom, DbName, DbTo, DieselError};
use core_common::types::{
    AccessOption, AuthorizationType, EntityTypes, EventTypes, FingerprintMd5,
    FingerprintSha256, Id, KeyManagement, SyncStatusType, UserTypes,
};
use std::borrow::Cow;

impl DbFrom for Id {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        Self::from_slice(v).map_err(|_| {
            let data = String::from_utf8_lossy(v).into();
            DieselError::TransformationError(data)
        })
    }
}

impl DbTo for Id {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        self.bytes.as_bytes()
    }
}

impl DbTo for &Id {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        self.bytes.as_bytes()
    }
}

impl DbFrom for FingerprintMd5<'_> {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        Ok(Self::from_bytes(Cow::Owned(v.into())))
    }
}

impl DbTo for FingerprintMd5<'_> {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        self.get_bytes()
    }
}

impl DbTo for &FingerprintMd5<'_> {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        self.get_bytes()
    }
}

impl DbFrom for FingerprintSha256<'_> {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        Ok(Self::from_bytes(Cow::Owned(v.into())))
    }
}

impl DbTo for FingerprintSha256<'_> {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        self.get_bytes()
    }
}

impl DbTo for &FingerprintSha256<'_> {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        self.get_bytes()
    }
}

impl DbName for UserTypes {
    #[inline]
    fn db_type_name() -> &'static str {
        "user_types"
    }
}

impl DbFrom for UserTypes {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        match v {
            b"user" | b"User" | b"USER" => Ok(Self::User),
            b"admin" | b"Admin" | b"ADMIN" => Ok(Self::Admin),
            b"superuser" | b"Superuser" | b"SUPERUSER" => Ok(Self::Superuser),
            _ => {
                let data = String::from_utf8_lossy(v).into();
                Err(DieselError::TransformationError(data))
            }
        }
    }
}

impl DbTo for UserTypes {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            Self::User => b"user",
            Self::Admin => b"admin",
            Self::Superuser => b"superuser",
        }
    }
}

impl DbTo for &UserTypes {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match *self {
            UserTypes::User => b"user",
            UserTypes::Admin => b"admin",
            UserTypes::Superuser => b"superuser",
        }
    }
}

impl DbName for AccessOption {
    #[inline]
    fn db_type_name() -> &'static str {
        "access_option"
    }
}

impl DbFrom for AccessOption {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        match v {
            b"command" => Ok(Self::Command),
            b"from" => Ok(Self::From),
            b"environment" => Ok(Self::Environment),
            b"no-agent-forwarding" => Ok(Self::NoAgentForwarding),
            b"no-port-forwarding" => Ok(Self::NoPortForwarding),
            b"no-pty" => Ok(Self::NoPty),
            b"no-X11-forwarding" => Ok(Self::NoX11Forwarding),
            b"no-user-rc" => Ok(Self::NoUserRc),
            _ => {
                let data = String::from_utf8_lossy(v).into();
                Err(DieselError::TransformationError(data))
            }
        }
    }
}

impl DbTo for AccessOption {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            Self::Command => b"command",
            Self::From => b"from",
            Self::Environment => b"environment",
            Self::NoAgentForwarding => b"no-agent-forwarding",
            Self::NoPortForwarding => b"no-port-forwarding",
            Self::NoPty => b"no-pty",
            Self::NoX11Forwarding => b"no-X11-forwarding",
            Self::NoUserRc => b"no-user-rc",
        }
    }
}

impl DbTo for &AccessOption {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match *self {
            AccessOption::Command => b"command",
            AccessOption::From => b"from",
            AccessOption::Environment => b"environment",
            AccessOption::NoAgentForwarding => b"no-agent-forwarding",
            AccessOption::NoPortForwarding => b"no-port-forwarding",
            AccessOption::NoPty => b"no-pty",
            AccessOption::NoX11Forwarding => b"no-X11-forwarding",
            AccessOption::NoUserRc => b"no-user-rc",
        }
    }
}

impl DbName for EntityTypes {
    #[inline]
    fn db_type_name() -> &'static str {
        "entity_types"
    }
}

impl DbFrom for EntityTypes {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        match v {
            b"user" => Ok(Self::User),
            b"server account" => Ok(Self::ServerAccount),
            b"group" => Ok(Self::Group),
            _ => {
                let data = String::from_utf8_lossy(v).into();
                Err(DieselError::TransformationError(data))
            }
        }
    }
}

impl DbTo for EntityTypes {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            Self::User => b"user",
            Self::ServerAccount => b"server account",
            Self::Group => b"group",
        }
    }
}

impl DbTo for &EntityTypes {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match *self {
            EntityTypes::User => b"user",
            EntityTypes::ServerAccount => b"server account",
            EntityTypes::Group => b"group",
        }
    }
}

impl DbName for EventTypes {
    #[inline]
    fn db_type_name() -> &'static str {
        "event_types"
    }
}

impl DbFrom for EventTypes {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        match v {
            b"server" => Ok(Self::Server),
            b"entity" => Ok(Self::Entity),
            _ => {
                let data = String::from_utf8_lossy(v).into();
                Err(DieselError::TransformationError(data))
            }
        }
    }
}

impl DbTo for EventTypes {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            Self::Server => b"server",
            Self::Entity => b"entity",
        }
    }
}

impl DbTo for &EventTypes {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match *self {
            EventTypes::Server => b"server",
            EventTypes::Entity => b"entity",
        }
    }
}

impl DbName for KeyManagement {
    #[inline]
    fn db_type_name() -> &'static str {
        "key_management"
    }
}

impl DbFrom for KeyManagement {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        match v {
            b"none" => Ok(Self::None),
            b"keys" => Ok(Self::Keys),
            b"other" => Ok(Self::Other),
            _ => {
                let data = String::from_utf8_lossy(v).into();
                Err(DieselError::TransformationError(data))
            }
        }
    }
}

impl DbTo for KeyManagement {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            Self::None => b"none",
            Self::Keys => b"keys",
            Self::Other => b"other",
        }
    }
}

impl DbTo for &KeyManagement {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match *self {
            KeyManagement::None => b"none",
            KeyManagement::Keys => b"keys",
            KeyManagement::Other => b"other",
        }
    }
}

impl DbName for AuthorizationType {
    #[inline]
    fn db_type_name() -> &'static str {
        "authorization_type"
    }
}

impl DbFrom for AuthorizationType {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        match v {
            b"manual" => Ok(Self::Manual),
            b"automatic" => Ok(Self::Automatic),
            _ => {
                let data = String::from_utf8_lossy(v).into();
                Err(DieselError::TransformationError(data))
            }
        }
    }
}

impl DbTo for AuthorizationType {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            Self::Manual => b"manual",
            Self::Automatic => b"automatic",
        }
    }
}

impl DbTo for &AuthorizationType {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match *self {
            AuthorizationType::Manual => b"manual",
            AuthorizationType::Automatic => b"automatic",
        }
    }
}

impl DbName for SyncStatusType {
    #[inline]
    fn db_type_name() -> &'static str {
        "sync_status_type"
    }
}

impl DbFrom for SyncStatusType {
    #[inline]
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError> {
        match v {
            b"not synced yet" => Ok(Self::NotSyncedYet),
            b"sync success" => Ok(Self::SyncSuccess),
            b"sync failure" => Ok(Self::SyncFailure),
            b"sync warning" => Ok(Self::SyncWarning),
            _ => {
                let data = String::from_utf8_lossy(v).into();
                Err(DieselError::TransformationError(data))
            }
        }
    }
}

impl DbTo for SyncStatusType {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            Self::NotSyncedYet => b"not synced yet",
            Self::SyncSuccess => b"sync success",
            Self::SyncFailure => b"sync failure",
            Self::SyncWarning => b"sync warning",
        }
    }
}

impl DbTo for &SyncStatusType {
    #[inline]
    fn convert_back(&self) -> &'_ [u8] {
        match self {
            SyncStatusType::NotSyncedYet => b"not synced yet",
            SyncStatusType::SyncSuccess => b"sync success",
            SyncStatusType::SyncFailure => b"sync failure",
            SyncStatusType::SyncWarning => b"sync warning",
        }
    }
}

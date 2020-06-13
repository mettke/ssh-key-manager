use core_common::tokio::task::JoinError;
use std::{error, fmt};

/// Diesel Error happening when communication with a database
#[derive(Debug)]
pub enum DieselError {
    /// Unable to transform binary representation to type
    TransformationError(String),
    /// Not server available which response to connection attempts
    NoServerAvailable,
    /// Internal Error from Diesel
    DieselError(diesel::result::Error),
    /// Internal Error from R2D2,
    R2D2Error(r2d2::Error),
    /// Error while trying to migrate database
    MigrationError(diesel_migrations::RunMigrationsError),
    /// Internal Async Execution failed
    JoinError(JoinError),
}

impl fmt::Display for DieselError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransformationError(data) => {
                write!(f, "Unable to convert {}", data)
            }
            Self::NoServerAvailable => {
                write!(f, "No database connections available.")
            }
            Self::DieselError(err) => write!(f, "Inner Error from Diesel: {}", err),
            Self::R2D2Error(err) => write!(f, "Inner Error from R2D2: {}", err),
            Self::MigrationError(err) => {
                write!(f, "Error while migrating Database: {}", err)
            }
            Self::JoinError(err) => {
                write!(f, "Failed while trying to execute blocking thread: {}", err)
            }
        }
    }
}

impl error::Error for DieselError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::TransformationError(_) | Self::NoServerAvailable => None,
            Self::DieselError(err) => Some(err),
            Self::R2D2Error(err) => Some(err),
            Self::MigrationError(err) => Some(err),
            Self::JoinError(err) => Some(err),
        }
    }
}

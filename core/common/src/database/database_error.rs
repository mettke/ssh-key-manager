use crate::database::Database;
use std::{error, fmt};

/// Database encountered an Error
#[derive(Debug)]
pub enum DatabaseError<D: Database> {
    /// Object fails unique constraints
    NonUnique,
    /// Custom Error from the underlying database management system
    Custom(D::DatabaseError),
}

impl<D: Database> fmt::Display for DatabaseError<D> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUnique => write!(f, "Object already exists"),
            Self::Custom(err) => write!(f, "Custom Database Error: {}", err),
        }
    }
}

impl<D> error::Error for DatabaseError<D>
where
    D: Database,
    <D as Database>::DatabaseError: 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::NonUnique => None,
            Self::Custom(err) => Some(err),
        }
    }
}

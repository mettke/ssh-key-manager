use crate::error::DieselError;

/// Type definition required to map Rust Type to Database Type
pub trait DbName: std::fmt::Debug + Sized {
    /// Name of the type inside the database
    fn db_type_name() -> &'static str;
}

/// Type definition required to map Rust Type to Database Type
pub trait DbFrom: std::fmt::Debug + Sized {
    /// Converts a binary represention to its corresponding type.
    ///
    /// # Errors
    ///
    /// Fails if binary cannot be mapped to type
    fn convert(v: &'_ [u8]) -> Result<Self, DieselError>;
}

/// Type definition required to map Rust Type to Database Type
pub trait DbTo: std::fmt::Debug + Sized {
    /// Converts a type to its binary representation
    fn convert_back(&self) -> &'_ [u8];
}

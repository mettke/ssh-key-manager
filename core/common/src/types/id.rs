use crate::serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    error, fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};
use uuid::Uuid;

/// Conversion Error happening when trying to parse an invalid uuid
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum UuidConversionError {
    /// Format does not match xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    InvalidFormat,
    /// Size does not equal 16
    InvalidSize,
}

impl fmt::Display for UuidConversionError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(
                f,
                "uuid must have the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
            ),
            Self::InvalidSize => write!(f, "uuid must be 16 bytes long"),
        }
    }
}

impl error::Error for UuidConversionError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// Id used inside the whole application. Resolves either to a uuid string
/// representation in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx or to
/// its byte representation [u8; 16]
/// TODO: Swich to `Cow`
#[derive(Debug, Clone, Eq)]
pub struct Id {
    /// Raw bytes of the UUID
    pub bytes: Uuid,
    /// Text Representation of the UUID. Only available when parsing from string
    pub text: Option<String>,
}

impl Id {
    /// Convertes an array to a UUID
    ///
    /// # Errors
    ///
    /// Will fail with `UuidConversionError::InvalidSize` if Array length is not 16
    #[inline]
    #[allow(single_use_lifetimes)]
    pub fn from_slice<'a, B: Into<&'a [u8]>>(
        bytes: B,
    ) -> Result<Self, UuidConversionError> {
        let bytes = bytes.into();
        Uuid::from_slice(bytes)
            .map_err(|_| UuidConversionError::InvalidSize)
            .map(|uuid| Self {
                bytes: uuid,
                text: None,
            })
    }

    /// Tries to parse a string as uuid.
    ///
    /// # Errors
    ///
    /// Will fail with `UuidConversionError::InvalidFormat` if string format does not
    /// equal xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    #[inline]
    pub fn from_string<S: Into<String>>(
        text: S,
    ) -> Result<Self, UuidConversionError> {
        let text = text.into();
        Uuid::from_str(&text)
            .map_err(|_| UuidConversionError::InvalidFormat)
            .map(|uuid| Self {
                bytes: uuid,
                text: Some(text),
            })
    }

    /// Converts an uuid to its string representation. Requires a
    /// buffer it the text field is not set
    #[inline]
    pub fn to_str<'a>(&'a self, buffer: &'a mut [u8; 45]) -> &'a str {
        match &self.text {
            Some(text) => text,
            None => self.bytes.to_hyphenated().encode_lower(buffer),
        }
    }
}

impl From<&Id> for Id {
    #[inline]
    fn from(id: &Self) -> Self {
        id.clone()
    }
}

impl PartialEq for Id {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Hash for Id {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl FromStr for Id {
    type Err = UuidConversionError;
    /// Tries to parse a string as uuid.
    ///
    /// # Errors
    ///
    /// Will fail with `UuidConversionError::InvalidFormat` if string format does not
    /// equal xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_string())
    }
}

impl Serialize for Id {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buffer = Uuid::encode_buffer();
        let text = self.to_str(&mut buffer);
        serializer.serialize_str(text)
    }
}

impl<'de> Deserialize<'de> for Id {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_string(string).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{Id, UuidConversionError};
    use uuid::Uuid;

    #[test]
    fn from_slice() {
        let bytes = &[
            0x5f, 0x5a, 0x85, 0x94, 0x4c, 0xb9, 0x45, 0x15, 0x97, 0xd0, 0x62, 0x20,
            0xb5, 0x96, 0x34, 0xeb,
        ];
        let id = Id::from_slice(&bytes[..]).expect("Unable to convert Id");
        assert_eq!(id.bytes.as_bytes(), bytes, "Id Bytes do not match");
    }

    #[test]
    fn from_slice_invalid() {
        let bytes = &[
            0x5f, 0x5a, 0x85, 0x94, 0x4c, 0xb9, 0x45, 0x15, 0x97, 0xd0, 0x62, 0x20,
            0xb5, 0x96, 0x34, 0xeb, 0x00,
        ];
        let id = Id::from_slice(&bytes[..]);
        assert_eq!(
            id,
            Err(UuidConversionError::InvalidSize),
            "Conversion should not work"
        );
    }

    #[test]
    fn from_string() {
        let bytes = &[
            0x5f, 0x5a, 0x85, 0x94, 0x4c, 0xb9, 0x45, 0x15, 0x97, 0xd0, 0x62, 0x20,
            0xb5, 0x96, 0x34, 0xeb,
        ];

        let uuid = "5f5a8594-4cb9-4515-97d0-6220b59634eb";
        let id = Id::from_string(uuid).expect("Unable to convert Id");
        assert_eq!(id.bytes.as_bytes(), bytes, "Id Bytes do not match");

        let uuid = "5f5a85944cb9451597d06220b59634eb";
        let id = Id::from_string(uuid).expect("Unable to convert Id");
        assert_eq!(id.bytes.as_bytes(), bytes, "Id Bytes do not match");
    }

    #[test]
    fn from_string_invalid() {
        let uuid = "5f5a85944cb9451597d06220b59634ebe";
        let id = Id::from_string(uuid).expect_err("Should error");
        assert_eq!(
            id,
            UuidConversionError::InvalidFormat,
            "Conversion should not work"
        );

        let uuid = "5f5a8594-4cb9-4515-97d0-6220b59634ebe";
        let id = Id::from_string(uuid).expect_err("Should error");
        assert_eq!(
            id,
            UuidConversionError::InvalidFormat,
            "Conversion should not work"
        );
    }

    #[test]
    fn to_str() {
        let mut buffer = Uuid::encode_buffer();
        let uuid = "5f5a8594-4cb9-4515-97d0-6220b59634eb";
        let id = Id::from_string(uuid).expect("Unable to convert Id");
        assert_eq!(id.to_str(&mut buffer), uuid, "Id String does not match");
    }
}

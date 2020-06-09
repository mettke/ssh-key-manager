use crate::serde::{Serialize, Serializer};
use base64::DecodeError;
use std::{
    borrow::{Borrow, Cow},
    error, fmt,
    hash::{Hash, Hasher},
};

/// Conversion Error happening when trying to parse an invalid fingerprint
#[derive(Debug, Clone)]
pub enum FingerprintConversionError {
    /// Input is not valid base64
    InvalidBase64(DecodeError),
}

impl fmt::Display for FingerprintConversionError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBase64(_) => write!(f, "input must be base64 encoded"),
        }
    }
}

impl error::Error for FingerprintConversionError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidBase64(err) => Some(err),
        }
    }
}

/// Sha256 Fingerprint of a Server. The string representation contains bytes in the
/// hexadecimal format. Any non valid hex character may be used as seperator
#[allow(single_use_lifetimes)]
#[derive(Debug, Clone, Eq)]
pub struct FingerprintMd5<'a> {
    bytes: Cow<'a, [u8]>,
    text: Option<Cow<'a, str>>,
}

impl PartialEq for FingerprintMd5<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Hash for FingerprintMd5<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<'a> FingerprintMd5<'a> {
    /// Returns the raw bytes of the fingerprint
    #[inline]
    #[must_use]
    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Creates a fingerprint using raw bytes
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: Cow<'a, [u8]>) -> Self {
        Self { bytes, text: None }
    }

    /// Creates a fingerprint from a string.
    #[inline]
    #[must_use]
    pub fn from_string(text: Cow<'a, str>) -> Self {
        let mut bytes = Vec::with_capacity(text.len());
        let mut tmp: u8 = 0;
        let mut fst: bool = true;
        for c in text.chars() {
            let val: u8 = match c {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                'a' | 'A' => 10,
                'b' | 'B' => 11,
                'c' | 'C' => 12,
                'd' | 'D' => 13,
                'e' | 'E' => 14,
                'f' | 'F' => 15,
                _ => continue,
            };
            if fst {
                tmp = val.saturating_mul(16);
                fst = false;
            } else {
                bytes.push(tmp.saturating_add(val));
                fst = true;
            }
        }
        Self {
            bytes: Cow::Owned(bytes),
            text: Some(text),
        }
    }

    /// Converts the given fingerprint to its string representation
    #[must_use]
    #[inline]
    #[allow(clippy::integer_arithmetic)]
    pub fn to_str(&'a self) -> Cow<'a, str> {
        if let Some(text) = self.text.as_ref() {
            Cow::Borrowed(text.borrow())
        } else {
            let mut text = String::with_capacity(self.bytes.len().saturating_mul(3));
            let mut tmp: char = ' ';
            let mut fst: bool = true;
            let mut prev: bool = false;
            for b in self.bytes.as_ref() {
                for i in &[1_u8, 0_u8] {
                    let val = match (*b >> (i * 4)) & 0xF {
                        0 => '0',
                        1 => '1',
                        2 => '2',
                        3 => '3',
                        4 => '4',
                        5 => '5',
                        6 => '6',
                        7 => '7',
                        8 => '8',
                        9 => '9',
                        10 => 'a',
                        11 => 'b',
                        12 => 'c',
                        13 => 'd',
                        14 => 'e',
                        15 => 'f',
                        _ => continue,
                    };
                    if fst {
                        tmp = val;
                        fst = false;
                    } else {
                        if prev {
                            text.push(':');
                        }
                        text.push(tmp);
                        text.push(val);
                        fst = true;
                        prev = true;
                    }
                }
            }
            Cow::Owned(text)
        }
    }
}

impl Serialize for FingerprintMd5<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let text = self.to_str();
        serializer.serialize_str(&text)
    }
}

/// Sha256 Fingerprint of a Server. The string representation
/// is base64 encoded
#[allow(single_use_lifetimes)]
#[derive(Debug, Clone, Eq)]
pub struct FingerprintSha256<'a> {
    bytes: Cow<'a, [u8]>,
    text: Option<Cow<'a, str>>,
}

impl PartialEq for FingerprintSha256<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Hash for FingerprintSha256<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<'a> FingerprintSha256<'a> {
    /// Returns the raw bytes of the fingerprint
    #[must_use]
    #[inline]
    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Creates a fingerprint using raw bytes
    #[must_use]
    #[inline]
    pub const fn from_bytes(bytes: Cow<'a, [u8]>) -> Self {
        Self { bytes, text: None }
    }

    /// Tries to parse a string as fingerprint.
    ///
    /// # Errors
    ///
    /// Will fail with `FingerprintConversionError::InvalidBase64` if string is
    /// not base64 encoded
    #[inline]
    pub fn from_string(
        mut text: Cow<'a, str>,
    ) -> Result<Self, FingerprintConversionError> {
        while text.len().checked_rem(2).unwrap_or_default() != 0 {
            text.to_mut().push('=');
        }
        base64::decode(text.as_ref())
            .map(|bytes| Self {
                bytes: Cow::Owned(bytes),
                text: Some(text),
            })
            .map_err(FingerprintConversionError::InvalidBase64)
    }
    /// Converts the given fingerprint to its string representation
    #[must_use]
    #[inline]
    pub fn to_str(&'a self) -> Cow<'a, str> {
        if let Some(text) = self.text.as_ref() {
            Cow::Borrowed(text.borrow())
        } else {
            let text = base64::encode(&self.bytes);
            let text = text.trim_end_matches('=');
            Cow::Owned(text.into())
        }
    }
}

impl Serialize for FingerprintSha256<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.text {
            Some(text) => serializer.serialize_str(text),
            None => {
                let text = self.to_str();
                serializer.serialize_str(&text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FingerprintMd5, FingerprintSha256};
    use std::borrow::Cow;

    #[test]
    fn md5_equal() {
        let string = "01:23:45:67:89:AB:CD:EF".into();
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let f_str = FingerprintMd5::from_string(string);
        let f_bytes = FingerprintMd5::from_bytes(bytes.into());
        assert_eq!(f_str, f_bytes, "Fingerprints do not match");
    }

    #[test]
    fn md5_from_bytes() {
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let f_bytes = FingerprintMd5::from_bytes(bytes.clone().into());
        assert_eq!(
            f_bytes.get_bytes(),
            &bytes[..],
            "Fingerprint bytes do not match"
        );
    }

    #[test]
    fn md5_from_string() {
        let string = "01:23:45:67:89:AB:CD:EF".into();
        let bytes = &[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let f_bytes = FingerprintMd5::from_string(string);
        assert_eq!(f_bytes.get_bytes(), bytes, "Fingerprint bytes do not match");
    }

    #[test]
    fn md5_to_string() {
        let string: String = "01:23:45:67:89:AB:CD:EF".into();
        let f_bytes = FingerprintMd5::from_string(string.clone().into());
        assert_eq!(
            f_bytes.to_str(),
            string,
            "Fingerprint String does not match"
        );
    }

    #[test]
    fn sha256_equal() {
        let string = "ASNFZ4mrze8=".into();
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let f_str =
            FingerprintSha256::from_string(string).expect("Unable to convert");
        let f_bytes = FingerprintSha256::from_bytes(bytes.into());
        assert_eq!(f_str, f_bytes, "Fingerprints do not match");
    }

    #[test]
    fn sha256_from_bytes() {
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let f_bytes = FingerprintSha256::from_bytes(bytes.clone().into());
        assert_eq!(
            f_bytes.get_bytes(),
            &bytes[..],
            "Fingerprint bytes do not match"
        );
    }

    #[test]
    fn sha256_from_string() {
        let bytes = &[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let string = "ASNFZ4mrze8=".into();
        let f_bytes =
            FingerprintSha256::from_string(string).expect("Unable to convert");
        assert_eq!(f_bytes.get_bytes(), bytes, "Fingerprint bytes do not match");
        let string = "ASNFZ4mrze8".into();
        let f_bytes =
            FingerprintSha256::from_string(string).expect("Unable to convert");
        assert_eq!(f_bytes.get_bytes(), bytes, "Fingerprint bytes do not match");

        let bytes = &[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD];
        let string = "ASNFZ4mrzQ==".into();
        let f_bytes =
            FingerprintSha256::from_string(string).expect("Unable to convert");
        assert_eq!(f_bytes.get_bytes(), bytes, "Fingerprint bytes do not match");
        let string = "ASNFZ4mrzQ=".into();
        let f_bytes =
            FingerprintSha256::from_string(string).expect("Unable to convert");
        assert_eq!(f_bytes.get_bytes(), bytes, "Fingerprint bytes do not match");
        let string = "ASNFZ4mrzQ".into();
        let f_bytes =
            FingerprintSha256::from_string(string).expect("Unable to convert");
        assert_eq!(f_bytes.get_bytes(), bytes, "Fingerprint bytes do not match");

        let string = "ASNFZ4mr".into();
        let bytes = &[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB];
        let f_bytes =
            FingerprintSha256::from_string(string).expect("Unable to convert");
        assert_eq!(f_bytes.get_bytes(), bytes, "Fingerprint bytes do not match");
    }

    #[test]
    fn sha256_to_string() {
        let string: String = "ASNFZ4mrze8=".into();
        let f_bytes = FingerprintSha256::from_string(Cow::Borrowed(&string))
            .expect("Unable to convert");
        assert_eq!(
            f_bytes.to_str(),
            string,
            "Fingerprint String does not match"
        );

        let string: String = "ASNFZ4mr".into();
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB];
        let f_bytes = FingerprintSha256::from_bytes(bytes.into());
        assert_eq!(
            f_bytes.to_str(),
            string,
            "Fingerprint String does not match"
        );
    }
}

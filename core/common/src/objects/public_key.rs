use crate::{
    chrono::NaiveDateTime,
    database::{Database, DatabaseError},
    serde::Serialize,
    types::Id,
    types::{FingerprintMd5, FingerprintSha256},
};
use bishop::{BishopArt, DrawingOptions};
use std::{borrow::Cow, convert::TryFrom};

/// Conversion Error happening when trying to parse an invalid public key
#[derive(Debug)]
pub enum PublicKeyConversionError<D: Database> {
    /// Database encountered an error
    DatabaseError(DatabaseError<D>),
    /// OpenSSH was unable to parse the key
    OpenSshError(openssh_keys::errors::Error),
}

#[derive(Debug, Clone, Hash, Serialize)]
/// Defines the Public Key structure in the database
pub struct PublicKey<'a> {
    /// The id which uniquely identifies the key
    pub id: Cow<'a, Id>,
    /// The entity which owns the key
    pub entity_id: Cow<'a, Id>,
    /// The key type
    pub type_: Cow<'a, str>,
    /// The key data
    pub keydata: Cow<'a, str>,
    /// An optional comment supplied by the owner
    pub comment: Option<Cow<'a, str>>,
    /// The size of the key
    pub keysize: Option<i32>,
    /// The keys md5 fingerprint
    pub fingerprint_md5: Option<Cow<'a, FingerprintMd5<'a>>>,
    /// The keys sha256 fingerprint
    pub fingerprint_sha256: Option<Cow<'a, FingerprintSha256<'a>>>,
    /// The keys md5 randomart
    pub randomart_md5: Option<Cow<'a, str>>,
    /// The keys sha256 randomart
    pub randomart_sha256: Option<Cow<'a, str>>,
    /// The time when the user uploaded the key
    pub upload_date: Option<NaiveDateTime>,
}

impl PublicKey<'_> {
    #[must_use]
    #[inline]
    /// Converts the key to its openssh authorized keys representation
    pub fn to_plain(&self) -> String {
        format!(
            "{} {} {}",
            self.type_,
            self.keydata,
            self.comment.as_deref().unwrap_or_default()
        )
    }

    /// Tries to parse a `PublicKey` from String. String must be in the openssh
    /// `authorized_keys` format. This method does not save the public key to the
    /// database.
    ///
    /// # Errors
    /// Fails if key is not in the openssh `authorized_keys` format or when the
    /// database cannot create an uuid
    #[allow(clippy::needless_lifetimes)]
    #[inline]
    pub async fn parse<'a, D: Database>(
        data: &str,
        owner: &'a Id,
        db: &D,
    ) -> Result<PublicKey<'a>, PublicKeyConversionError<D>> {
        let key = openssh_keys::PublicKey::parse(data)
            .map_err(PublicKeyConversionError::OpenSshError)?;
        Self::from_openssh(&key, owner, db)
            .await
            .map_err(PublicKeyConversionError::DatabaseError)
    }

    #[allow(clippy::needless_lifetimes)]
    async fn from_openssh<'a, D: Database>(
        value: &openssh_keys::PublicKey,
        owner: &'a Id,
        db: &D,
    ) -> Result<PublicKey<'a>, DatabaseError<D>> {
        let id = Cow::Owned(db.generate_id().await?);
        let entity_id = Cow::Borrowed(owner);
        let type_ = Cow::Owned(value.keytype().into());
        let keydata = Cow::Owned(base64::encode(value.data()));
        let comment = value.comment.clone().map(Cow::Owned);
        let keysize = i32::try_from(value.size()).ok();
        let fingerprint_md5 = Some(Cow::Owned(FingerprintMd5::from_string(
            Cow::Owned(value.fingerprint_md5()),
        )));
        let fingerprint_sha256 =
            FingerprintSha256::from_string(Cow::Owned(value.fingerprint()))
                .ok()
                .map(Cow::Owned);
        let randomart_md5 = fingerprint_md5
            .as_ref()
            .map(|f| Self::create_randomart_md5(value, f))
            .map(Cow::Owned);
        let randomart_sha256 = fingerprint_sha256
            .as_ref()
            .map(|f| Self::create_randomart_sha256(value, f))
            .map(Cow::Owned);

        Ok(PublicKey {
            id,
            entity_id,
            type_,
            keydata,
            comment,
            keysize,
            fingerprint_md5,
            fingerprint_sha256,
            randomart_md5,
            randomart_sha256,
            upload_date: None,
        })
    }

    fn create_randomart_md5(
        pk: &openssh_keys::PublicKey,
        fingerprint: &FingerprintMd5<'_>,
    ) -> String {
        let top_text = format!("{} {}", pk.keytype(), pk.size());
        let options = DrawingOptions {
            top_text,
            bottom_text: "MD5".into(),
            ..DrawingOptions::default()
        };
        let mut field = BishopArt::new();
        field.input(fingerprint.get_bytes());
        let mut text = field.draw_with_opts(&options);
        let _ = text.pop();
        text
    }

    fn create_randomart_sha256(
        pk: &openssh_keys::PublicKey,
        fingerprint: &FingerprintSha256<'_>,
    ) -> String {
        let top_text = format!("{} {}", pk.keytype(), pk.size());
        let options = DrawingOptions {
            top_text,
            bottom_text: "SHA256".into(),
            ..DrawingOptions::default()
        };
        let mut field = BishopArt::new();
        field.input(fingerprint.get_bytes());
        let mut text = field.draw_with_opts(&options);
        let _ = text.pop();
        text
    }
}

#[derive(Debug, Clone, Hash, Serialize)]
/// Provides fields to filter when searching for multiple
/// objects
pub struct PublicKeyFilter<'a> {
    /// The owner of the key must equal to this id
    pub entity_id: Option<Cow<'a, Id>>,
    /// The key must equal this value
    pub type_: Option<Cow<'a, str>>,
    /// The comment must be like this value
    pub comment: Option<Cow<'a, str>>,
    /// The keysize must be greater or equal then this value
    pub keysize_ge: Option<i32>,
    /// The keysize must be lower or equal then this value
    pub keysize_le: Option<i32>,
    /// The md5 fingerprint must be equal to this value
    pub fingerprint_md5: Option<Cow<'a, FingerprintMd5<'a>>>,
    /// The sha256 fingerprint must be equal to this value
    pub fingerprint_sha256: Option<Cow<'a, FingerprintSha256<'a>>>,
}

impl Default for PublicKeyFilter<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            entity_id: None,
            type_: None,
            comment: None,
            keysize_ge: None,
            keysize_le: None,
            fingerprint_md5: None,
            fingerprint_sha256: None,
        }
    }
}

impl<'a, I: Iterator<Item = (Cow<'a, str>, Cow<'a, str>)>> From<I>
    for PublicKeyFilter<'a>
{
    #[inline]
    fn from(iter: I) -> Self {
        let mut filter = Self::default();
        for (key, val) in iter {
            if val.is_empty() {
                continue;
            }
            match key.as_ref() {
                "type" => {
                    filter.type_ = Some(val);
                }
                "comment" => {
                    filter.comment = Some(val);
                }
                "keysize-min" => {
                    filter.keysize_ge = val.parse::<i32>().ok();
                }
                "keysize-max" => {
                    filter.keysize_le = val.parse::<i32>().ok();
                }
                "fingerprint" => {
                    filter.fingerprint_md5 =
                        Some(Cow::Owned(FingerprintMd5::from_string(val.clone())));
                    filter.fingerprint_sha256 =
                        FingerprintSha256::from_string(val).ok().map(Cow::Owned);
                }
                _ => {}
            }
        }
        filter
    }
}

#[cfg(test)]
mod tests {
    use super::{PublicKey, PublicKeyFilter};
    use crate::{
        database::{Database, DatabaseError},
        types::{FingerprintMd5, FingerprintSha256, Id},
    };
    use std::borrow::Cow;

    struct TestDb;

    impl Database for TestDb {
        fn generate_id(&self) -> Result<Id, DatabaseError> {
            let key_id: String = r"00bd8c06-daf7-47e6-8c96-8d467587b6dc".into();
            Ok(Id::from_string(key_id).expect("Invalid Id"))
        }
        fn fetch_permission_ids<'a>(
            &self,
            _entity_id: Cow<'a, Id>,
        ) -> Result<Vec<Cow<'a, Id>>, DatabaseError> {
            unimplemented!()
        }
    }

    #[test]
    fn parse() {
        let owner_id = r"c6efb44e-9b67-4dc0-a31b-6482476ed8b7";
        let key_str = r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDS6o9i9w5eFXEUcQMhOvupIhPFdb1evYoYPmTDSkoejpZF+u7PHfPanSXc/95UbsOuBLENgIGnr/1gN9Vvok/XqZc+UnODyAKztdGx8za9Zhxe3BBxs1R1UJ5Ri5U+WQkvedUYJs2vvl67ZkMFOV49gILjHD8Lq43lU3pyPupmKtq3dRzCyFQk9smx4eyW9vWaPgKHHMeFvoO2coMg9vF06vuFb5H/KqEO58GYgy45Zc+sePOWA6i4z9uBWQyTUzpHrT8TpQABunIfx6KGwyt+7y8LzCbsks7R/HE67PNJz9bb7lBXraqBMFfFhciiHDgnppt8BY/MCeF7OLcsyhztaBJyz6v04c4jiHX32FfsL8w57fPU9paCj6RnSbCcB4hrsuqpCnAEClLSBhrFa/3agucst7VP6Z+pabzh+lNjuwWh9FR7/zB3sBNhQDpMwJyuOcwLKj+uThZGfzIpRSIfUK7WX2msCqlgCnP7ELkinj8fETXEFg1mL66VgpYuFHM= testkey";
        let fingerprint_md5 = FingerprintMd5::from_string(
            r"cb:18:d9:7d:f6:c8:c9:1b:e6:e9:7d:b5:32:4e:b8:1d".into(),
        );
        let fingerprint_sha256 = FingerprintSha256::from_string(Cow::Borrowed(
            r"fb6rYJ5d3p2IRUKjdlb4RYXPCZvjUTSd9IstLY0NhXo",
        ))
        .expect("Unable to convert fingerprint");
        let random_art_md5 = r#"+-[ssh-rsa 3072]--+
|                 |
|                 |
|                 |
|       o .       |
|      o S . o    |
|       + . =.+  .|
|      . o  .BE. o|
|           o+B...|
|           o*o+. |
+------[MD5]------+"#;
        let random_art_sha256 = r#"+-[ssh-rsa 3072]--+
|              +B=|
|            .o+o+|
|           +.o=+o|
|         .o.+EX.+|
|        So.++Bo* |
|        . ooo.o  |
|        o   o.   |
|       o + ooo...|
|        o oo+o...|
+----[SHA256]-----+"#;

        let db = TestDb;
        let id = db.generate_id().expect("Unable to generate id");

        let entity_id = Id::from_string(owner_id).expect("Invalid Id");
        let key =
            PublicKey::parse(key_str, &entity_id, &db).expect("Unable to parse key");

        assert_eq!(key.id, Cow::Owned(id));
        assert_eq!(key.entity_id, Cow::Borrowed(&entity_id));
        assert_eq!(key.type_, "ssh-rsa");
        assert_eq!(
            key.keydata,
            Cow::Borrowed(
                r"AAAAB3NzaC1yc2EAAAADAQABAAABgQDS6o9i9w5eFXEUcQMhOvupIhPFdb1evYoYPmTDSkoejpZF+u7PHfPanSXc/95UbsOuBLENgIGnr/1gN9Vvok/XqZc+UnODyAKztdGx8za9Zhxe3BBxs1R1UJ5Ri5U+WQkvedUYJs2vvl67ZkMFOV49gILjHD8Lq43lU3pyPupmKtq3dRzCyFQk9smx4eyW9vWaPgKHHMeFvoO2coMg9vF06vuFb5H/KqEO58GYgy45Zc+sePOWA6i4z9uBWQyTUzpHrT8TpQABunIfx6KGwyt+7y8LzCbsks7R/HE67PNJz9bb7lBXraqBMFfFhciiHDgnppt8BY/MCeF7OLcsyhztaBJyz6v04c4jiHX32FfsL8w57fPU9paCj6RnSbCcB4hrsuqpCnAEClLSBhrFa/3agucst7VP6Z+pabzh+lNjuwWh9FR7/zB3sBNhQDpMwJyuOcwLKj+uThZGfzIpRSIfUK7WX2msCqlgCnP7ELkinj8fETXEFg1mL66VgpYuFHM="
            )
        );
        assert_eq!(key.comment, Some("testkey".into()));
        assert_eq!(key.keysize, Some(3072));
        assert_eq!(key.fingerprint_md5, Some(Cow::Owned(fingerprint_md5)));
        assert_eq!(key.fingerprint_sha256, Some(Cow::Owned(fingerprint_sha256)));
        assert_eq!(key.randomart_md5, Some(random_art_md5.into()));
        assert_eq!(key.randomart_sha256, Some(random_art_sha256.into()));
        assert_eq!(key.upload_date, None);
        assert_eq!(key.to_plain(), key_str);
    }

    #[test]
    fn filter() {
        let type_ = "test_type";
        let comment = "test_comment";
        let keysize_min = "20";
        let kesize_max = "2000";
        let fingerprint_md5 = "20:20";
        let iter = vec![
            ("other", "other"),
            ("type", type_),
            ("comment", comment),
            ("keysize-min", keysize_min),
            ("other2", "other2"),
            ("keysize-max", kesize_max),
            ("fingerprint", fingerprint_md5),
            ("other3", "other3"),
        ]
        .into_iter()
        .map(|(k, v)| (Cow::Borrowed(k), Cow::Borrowed(v)));
        let filter = PublicKeyFilter::from(iter);

        assert_eq!(filter.entity_id, None);
        assert_eq!(filter.type_, Some(type_.into()));
        assert_eq!(filter.comment, Some(comment.into()));
        assert_eq!(filter.keysize_ge, Some(20));
        assert_eq!(filter.keysize_le, Some(2000));
        assert_eq!(
            filter.fingerprint_md5,
            Some(Cow::Owned(FingerprintMd5::from_string(
                fingerprint_md5.into()
            )))
        );
        assert_eq!(
            filter.fingerprint_sha256,
            FingerprintSha256::from_string(fingerprint_md5.into())
                .map(Cow::Owned)
                .ok()
        );
    }
}

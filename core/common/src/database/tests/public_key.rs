use super::auth::admin::AdminAuth;
use crate::{
    chrono::offset::Utc,
    database::{Create, Database},
    objects::PublicKey,
    types::{FingerprintMd5, FingerprintSha256},
};
use std::{borrow::Cow, marker::PhantomData, sync::Arc};

pub(crate) async fn create_with_admin<D>(db: Arc<D>)
where
    for<'a> D: Database + Create<'a, AdminAuth, PublicKey<'a>, D>,
{
    let entity_id = db.generate_id().await.expect("Unable to generate id");
    let auth = AdminAuth {
        id: entity_id.clone(),
    };
    let id = db.generate_id().await.expect("Unable to generate id");
    let fingerprint_md5 = FingerprintMd5::from_bytes(Cow::Borrowed(&[20, 20, 20]));
    let fingerprint_sha256 =
        FingerprintSha256::from_bytes(Cow::Owned(id.bytes.as_bytes().to_vec()));
    let upload_date = Utc::now().naive_utc();
    let key = PublicKey {
        id: Cow::Owned(id),
        entity_id: Cow::Owned(entity_id),
        type_: "ssh-rsa".into(),
        keydata: "test keydata".into(),
        comment: Some("test comment".into()),
        keysize: Some(12),
        fingerprint_md5: Some(Cow::Owned(fingerprint_md5)),
        fingerprint_sha256: Some(Cow::Owned(fingerprint_sha256)),
        randomart_md5: Some("test".into()),
        randomart_sha256: Some("test2".into()),
        upload_date: Some(upload_date),
    };
    db.create(&key, &auth, PhantomData)
        .await
        .expect("Unable to create key");
}

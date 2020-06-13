use core_common::{
    database::{
        Create, Database, DatabaseError, DbList, FetchAll, FetchById, FetchByUid,
    },
    log,
    objects::{Entity, PublicKey, PublicKeyConversionError, PublicKeyFilter, User},
    sec::{Auth, CsrfToken},
    serde::Serialize,
    types::Id,
    web::{AppError, Notification, Request, TemplateEngine},
};
use std::{borrow::Cow, marker::PhantomData};

/// A List of public keys ready to be presented
#[derive(Debug)]
pub struct PublicKeyListView<'a>(pub DbList<PublicKey<'a>>);

impl<'a> PublicKeyListView<'a> {
    /// Fetches all public keys visible to the given user
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes)]
    pub async fn fetch<A, D, T, R>(
        req: &R,
        filter: &PublicKeyFilter<'_>,
    ) -> Result<PublicKeyListView<'a>, AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b> D:
            Database + FetchAll<'a, 'b, A, PublicKey<'a>, PublicKeyFilter<'b>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let auth = req.get_auth();
        let db = req.get_database();

        db.fetch_all(filter, auth, 0, PhantomData)
            .await
            .map(Self)
            .map_err(AppError::DatabaseError)
    }

    /// Creates a `PublicKey` using the information in the request body
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes, clippy::needless_lifetimes)]
    pub async fn create<'e, A, D, T, R>(
        req: &mut R,
        data: Option<Cow<'_, str>>,
        uid: Option<&'e User<'_>>,
        csrf: &CsrfToken,
    ) -> Result<[Notification<'e>; 1], AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b, 'c> D: Database
            + FetchAll<'b, 'c, A, PublicKey<'b>, PublicKeyFilter<'c>, D>
            + FetchByUid<'b, A, User<'b>, D>
            + Create<'b, A, PublicKey<'b>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        if !csrf.valid {
            return Ok([Notification::Error {
                name: "Public Key",
                para: "csrf",
                help: "../help/#pubkey_err",
            }]);
        }
        let db = req.get_database();
        let auth = req.get_auth();
        let body = if let (Some(data), Some(uid)) = (data.as_ref(), uid) {
            match PublicKey::parse(data, &uid.entity_id, db).await {
                Err(PublicKeyConversionError::DatabaseError(err)) => {
                    return Err(AppError::DatabaseError(err));
                }
                Err(PublicKeyConversionError::OpenSshError(err)) => {
                    log::warn!("Error while tring to convert publickey: {}", err);
                    (None, Some(()))
                }
                Ok(key) => (Some(key), Some(())),
            }
        } else {
            (None, None)
        };
        match body {
            (Some(key), Some(_)) => match db.create(&key, auth, PhantomData).await {
                Err(err @ DatabaseError::Custom(_)) => {
                    Err(AppError::DatabaseError(err))
                }
                Err(DatabaseError::NonUnique) => Ok([Notification::Unique {
                    name: "Public Key",
                    para: "fingerprint",
                    help: "../help/#pubkey_err",
                }]),
                Ok(()) => Ok([Notification::Info {
                    name: "Public Key",
                    url: ".",
                    id: key.id,
                }]),
            },
            (None, _) => Ok([Notification::Error {
                name: "Public Key",
                para: "Public Key Data",
                help: "../help/#pubkey_err",
            }]),
            (_, None) => Ok([Notification::Error {
                name: "Public Key",
                para: "Uid",
                help: "../help/#pubkey_err",
            }]),
        }
    }
}

/// A public key ready to be presented
#[derive(Debug, Clone, Hash, Serialize)]
pub struct PublicKeyView<'a> {
    /// The public key to show to the user
    pub public_key: PublicKey<'a>,
    /// The entity which owns the public key
    pub owner: Entity<'a>,
    /// Whether the current user is the owner
    pub is_owner: bool,
}

#[allow(clippy::unimplemented)]
impl PublicKeyView<'_> {
    /// Fetches all public keys visible to the given user
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes)]
    pub async fn fetch<'a, A, D, T, R>(
        req: &R,
        key: &str,
    ) -> Result<Option<PublicKeyView<'a>>, AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b> D: Database
            + FetchById<'a, A, PublicKey<'a>, D>
            + FetchById<'a, A, Entity<'a>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let auth = req.get_auth();
        let db = req.get_database();

        let id = match Id::from_string(key) {
            Err(_) => {
                return Ok(None);
            }
            Ok(id) => id,
        };

        let public_key: Option<PublicKey<'_>> =
            db.fetch(&id, auth, PhantomData).await?;
        if let Some(public_key) = public_key {
            let owner: Option<Entity<'_>> =
                db.fetch(&public_key.entity_id, auth, PhantomData).await?;
            let owner = owner.unwrap_or_else(|| Entity {
                entity_id: public_key.entity_id.clone(),
                name: None,
                server_id: None,
                server_name: None,
                type_: None,
            });
            let is_owner = *owner.entity_id == *auth.get_id();
            Ok(Some(PublicKeyView {
                public_key,
                owner,
                is_owner,
            }))
        } else {
            Ok(None)
        }
    }
}

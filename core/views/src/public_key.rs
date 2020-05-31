use core_common::{
    database::{Create, Database, DatabaseError, DbList, FetchAll, FetchByUid},
    log,
    objects::{PublicKey, PublicKeyConversionError, PublicKeyFilter, User},
    sec::{Auth, CsrfToken},
    serde::Serialize,
    types::{EntityTypes, Id},
    web::{AppError, Notification, Request, TemplateEngine},
};
use std::borrow::Cow;

/// A List of public keys ready to be presented
#[derive(Debug)]
pub struct PublicKeyListView<'a>(pub DbList<PublicKey<'a>>);

impl<'a> PublicKeyListView<'a> {
    /// Fetches all public keys visible to the given user
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes, single_use_lifetimes)]
    pub async fn fetch<A, D, T, R>(
        req: &R,
        filter: &PublicKeyFilter<'_>,
    ) -> Result<PublicKeyListView<'a>, AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b, 'c> D:
            Database + FetchAll<'b, A, PublicKey<'a>, PublicKeyFilter<'c>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let auth = req.get_auth();
        let db = req.get_database();

        db.fetch_all(filter, auth, 0)
            .map(Self)
            .map_err(AppError::DatabaseError)
    }

    /// Creates a `PublicKey` using the information in the request body
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes, single_use_lifetimes, clippy::needless_lifetimes)]
    pub async fn create<'e, A, D, T, R>(
        req: &mut R,
        data: Option<Cow<'_, str>>,
        uid: Option<&'e User<'_>>,
        csrf: &CsrfToken,
    ) -> Result<[Notification<'e>; 1], AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b, 'c, 'd> D: Database
            + FetchAll<'b, A, PublicKey<'d>, PublicKeyFilter<'c>, D>
            + FetchByUid<A, User<'d>, D>
            + Create<A, PublicKey<'d>, D>,
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
            match PublicKey::parse(data, &uid.entity_id, db) {
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
            (Some(key), Some(_)) => match db.create(&key, auth) {
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
                    url: "publickeys",
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

/// The owner associated with the `PublicKey`
#[derive(Debug, Clone, Hash, Serialize)]
pub struct PublicKeyEntityView {
    /// The id of the entity
    pub entity_id: Id,
    /// The type of the entity
    pub entity_type: EntityTypes,
    /// The name of the entity
    pub name: Option<String>,
}

/// A public key ready to be presented
#[derive(Debug, Clone, Hash, Serialize)]
pub struct PublicKeyView<'a> {
    /// The public key to show to the user
    pub public_key: PublicKey<'a>,
    /// The entity which owns the public key
    pub owner: PublicKeyEntityView,
    /// Whether the current user is the owner
    pub is_owner: bool,
}

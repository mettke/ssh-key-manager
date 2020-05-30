use crate::{
    async_trait::async_trait,
    database::{Create, Database, FetchByUid, Save},
    http::response,
    objects::User,
    types::{Id, UserTypes},
    web::{AppError, Request, TemplateEngine, UserContainer},
};
use std::{error, fmt};

/// Defines methods of authentication which the application supports.
#[derive(Debug, Clone, Copy)]
pub enum AuthMethod {
    /// Authentication via OAuth
    OAuth,
}

/// Provides methods to check the authentication and authorisation of
/// the current user
#[allow(clippy::type_repetition_in_bounds)]
#[async_trait]
pub trait Auth: Sized + Send + Sync + fmt::Debug {
    /// the custom error for the template engine.
    type AuthError: error::Error;

    /// Authenticates the current user
    async fn authenticate<D, T, R>(
        req: &R,
        res: &mut response::Builder,
    ) -> Option<Self>
    where
        for<'a> D: Database
            + FetchByUid<PreAuth, User<'a>, D>
            + Create<PreAuth, User<'a>, D>
            + Save<PreAuth, User<'a>, D>,
        T: TemplateEngine,
        R: Request<Self, D, T> + Sync;

    /// Creates an `Auth` Object using the given values.
    /// Used by oauth to create a user from the token
    /// response. May respond with `None` if oauth is
    /// not supported
    ///
    /// # Errors
    /// Fail when the communication with the database fails
    fn create<D, T, R>(
        req: &R,
        username: String,
        name: &str,
        email: &str,
        exp: Option<u64>,
        type_: UserTypes,
    ) -> Result<Option<Self>, AppError<Self, D, T, R>>
    where
        for<'a> D: Database
            + FetchByUid<PreAuth, User<'a>, D>
            + Create<PreAuth, User<'a>, D>
            + Save<PreAuth, User<'a>, D>,
        T: TemplateEngine,
        R: Request<Self, D, T>;

    /// Returns `true` if the current user can be safely considered as an
    /// administrator
    fn is_admin(&self) -> bool;

    /// Returns the uid of the user. Result must not be empty as a user
    /// must have an uid.
    fn get_uid(&self) -> &str;

    /// Returns the internal database id of the user.
    fn get_id(&self) -> &Id;

    /// Get a `UserContainer` from the current Authentication mechanism
    fn get_user_container(&self) -> UserContainer<'_>;

    /// Returns whether or not a given Method is supported
    fn is_supported(method: AuthMethod) -> bool;

    /// Creates a string form the current Authentication Objects.
    /// Used to store the string in a cookie.
    /// May return `Ok(None)` if OAuth is not supported
    ///
    /// # Errors
    /// Fails on signing or encryption operations
    fn get_str<D, T, R>(
        &self,
        req: &R,
    ) -> Result<Option<String>, AppError<Self, D, T, R>>
    where
        D: Database,
        T: TemplateEngine,
        R: Request<Self, D, T>;
}

#[derive(Debug, Clone, Copy)]
pub struct PreAuthError;

impl fmt::Display for PreAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PreAuthError should never appear")
    }
}

impl error::Error for PreAuthError {}

/// Special struct which allows database interaction while authenticating
#[derive(Debug, Clone, Copy)]
pub struct PreAuth;

#[allow(clippy::panic)]
#[allow(clippy::type_repetition_in_bounds)]
#[async_trait]
impl Auth for PreAuth {
    type AuthError = PreAuthError;

    #[inline]
    async fn authenticate<D, T, R>(_: &R, _: &mut response::Builder) -> Option<Self>
    where
        for<'a> D: Database
            + FetchByUid<PreAuth, User<'a>, D>
            + Create<PreAuth, User<'a>, D>
            + Save<PreAuth, User<'a>, D>,
        T: TemplateEngine,
        R: Request<Self, D, T> + Sync,
    {
        Some(Self {})
    }

    #[inline]
    fn create<D, T, R>(
        _req: &R,
        _username: String,
        _name: &str,
        _email: &str,
        _exp: Option<u64>,
        _type_: UserTypes,
    ) -> Result<Option<Self>, AppError<Self, D, T, R>>
    where
        D: Database,
        T: TemplateEngine,
        R: Request<Self, D, T>,
    {
        Ok(None)
    }

    #[inline]
    fn is_admin(&self) -> bool {
        false
    }

    #[inline]
    fn get_uid(&self) -> &str {
        panic!("Should never be called")
    }

    #[inline]
    fn get_id(&self) -> &Id {
        panic!("Should never be called");
    }

    #[inline]
    fn get_user_container(&self) -> UserContainer<'_> {
        UserContainer {
            id: None,
            uid: None,
            name: None,
            is_admin: false,
            is_superuser: false,
        }
    }

    #[inline]
    fn is_supported(_: AuthMethod) -> bool {
        false
    }

    #[inline]
    fn get_str<D, T, R>(
        &self,
        _: &R,
    ) -> Result<Option<String>, AppError<Self, D, T, R>>
    where
        D: Database,
        T: TemplateEngine,
        R: Request<Self, D, T>,
    {
        Ok(None)
    }
}

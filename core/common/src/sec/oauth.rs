use crate::{
    database::{Create, Database, FetchByUid, Save},
    objects,
    sec::{async_http_client, Auth, PreAuth},
    types::UserTypes,
    url,
    web::{create_cookie, delete_cookie, AppError, Request, TemplateEngine},
};
use failure::{Compat, Fail};
use jsonwebtoken::dangerous_unsafe_decode;
use openidconnect::{
    core::{
        CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreJsonWebKeyType,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
        CoreProviderMetadata, CoreTokenType,
    },
    AccessToken, AsyncCodeTokenRequest, AsyncRefreshTokenRequest, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, EmptyAdditionalClaims, EmptyExtraTokenFields,
    ExtraTokenFields, IdToken, IdTokenClaims, IdTokenFields, IssuerUrl, Nonce,
    NonceVerifier, OAuth2TokenResponse, RedirectUrl, RefreshToken, Scope,
    StandardTokenResponse, SubjectIdentifier, TokenResponse, UserInfoClaims,
};
use serde::Deserialize;
use std::{convert::TryFrom, error, fmt};

type LTokenResponse = StandardTokenResponse<
    IdTokenFields<
        EmptyAdditionalClaims,
        EmptyExtraTokenFields,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
        CoreJsonWebKeyType,
    >,
    CoreTokenType,
>;

#[derive(Debug, Clone, Hash)]
struct User {
    username: String,
    name: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize, Hash)]
struct JwtPayload {
    sub: String,
    exp: u64,
}

struct LocalNonceVerify(Option<Nonce>);

impl NonceVerifier for LocalNonceVerify {
    fn verify(self, nonce: Option<&Nonce>) -> Result<(), String> {
        if let Some(n) = self.0 {
            n.verify(nonce)
        } else {
            Ok(())
        }
    }
}

/// Failures happening when trying to talk to the external OAuth Provider
#[derive(Debug)]
#[allow(variant_size_differences)]
pub enum OAuthError {
    /// Unable to parse url
    Url(url::ParseError),
    /// Unable to fetch Metadata
    Discover(Compat<openidconnect::DiscoveryError<reqwest::Error>>),
    /// Unable to get token for code
    RequestToken(
        Compat<
            openidconnect::RequestTokenError<
                reqwest::Error,
                openidconnect::StandardErrorResponse<
                    openidconnect::BasicErrorResponseType,
                >,
            >,
        >,
    ),
    /// Endpoint is missing user info
    NoUserEndpoint(Compat<openidconnect::NoUserInfoEndpoint>),
    /// Endpoint is missing user info
    UserInfo(Compat<openidconnect::UserInfoError<reqwest::Error>>),
    /// Token verification failed
    ClaimVerification(Compat<openidconnect::ClaimsVerificationError>),
    /// Token claim extraction failed
    ClaimExtraction(jsonwebtoken::errors::Error),
    /// Token does not contain a username
    TokenMissesUsername,
    /// Token does not contain a mail address
    TokenMissesEmail,
    /// Token does not contain a name
    TokenMissesName,
    /// Token is not authorised for user scope
    TokenMissesUserScope,
}

impl fmt::Display for OAuthError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(err) => err.fmt(f),
            Self::Discover(err) => err.fmt(f),
            Self::RequestToken(err) => err.fmt(f),
            Self::NoUserEndpoint(err) => err.fmt(f),
            Self::UserInfo(err) => err.fmt(f),
            Self::ClaimVerification(err) => err.fmt(f),
            Self::ClaimExtraction(err) => err.fmt(f),
            Self::TokenMissesUsername => write!(f, "Username is missing in Token"),
            Self::TokenMissesEmail => write!(f, "Email is missing in Token"),
            Self::TokenMissesName => write!(f, "Name is missing in Token"),
            Self::TokenMissesUserScope => write!(f, "UserScope is missing in Token"),
        }
    }
}

impl error::Error for OAuthError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Url(err) => Some(err),
            Self::Discover(err) => Some(err),
            Self::RequestToken(err) => Some(err),
            Self::NoUserEndpoint(err) => Some(err),
            Self::UserInfo(err) => Some(err),
            Self::ClaimVerification(err) => Some(err),
            Self::ClaimExtraction(err) => Some(err),
            Self::TokenMissesUsername
            | Self::TokenMissesEmail
            | Self::TokenMissesName
            | Self::TokenMissesUserScope => None,
        }
    }
}

/// Struct allowing communication with the external OAuth Provider
#[derive(Debug, Clone)]
pub struct OAuth2 {
    /// Internal OAuth Client
    client: CoreClient,
    /// Scope required for a user to use the application
    pub user_scope: String,
    /// Scope required for a user to be admin
    pub admin_scope: String,
    /// Scope required for a user to be superuser
    pub super_user_scope: String,
}

impl OAuth2 {
    /// Create a new `OAuth2` object
    ///
    /// # Errors
    /// Fails if the oauth client is not able to get provider metadata
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub async fn new<D: Database>(
        client_id: String,
        client_secret: Option<String>,
        metdata: String,
        redirect_url: &str,
        user_scope: String,
        admin_scope: String,
        super_user_scope: String,
    ) -> Result<Self, OAuthError> {
        let client_secret = client_secret.map(ClientSecret::new);
        let issuer_url = IssuerUrl::new(metdata).map_err(OAuthError::Url)?;
        let redirect_url =
            RedirectUrl::new(redirect_url.to_string()).map_err(OAuthError::Url)?;
        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url, async_http_client)
                .await
                .map_err(Fail::compat)
                .map_err(OAuthError::Discover)?;
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id),
            client_secret,
        )
        .set_redirect_uri(redirect_url);
        Ok(Self {
            client,
            user_scope,
            admin_scope,
            super_user_scope,
        })
    }

    /// Get authorize url for openid authentication
    #[inline]
    pub fn authorize_url(&self) -> (url::Url, CsrfToken, Nonce) {
        self.client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .url()
    }

    /// Tries to convert a `LTokenResponse` to List of Cookies, a Token and
    /// String representation of the Token
    ///
    /// # Errors
    /// Fails on database and token creation issues
    #[inline]
    pub async fn handle_token<A, D, T, R>(
        &self,
        token_result: &LTokenResponse,
        req: &R,
    ) -> Result<(Vec<String>, Option<(A, String)>), AppError<A, D, T, R>>
    where
        A: Auth,
        for<'a> D: Database
            + FetchByUid<'a, PreAuth, objects::User<'a>, D>
            + Create<'a, PreAuth, objects::User<'a>, D>
            + Save<'a, PreAuth, objects::User<'a>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let (user, a_exp) = if let Some(id_token) = token_result.id_token() {
            let claims = self.validate::<D>(id_token)?;
            let username = claims
                .preferred_username()
                .ok_or(OAuthError::TokenMissesUsername)?
                .to_string();
            let email = claims
                .email()
                .ok_or(OAuthError::TokenMissesEmail)?
                .to_string();
            let name = claims
                .name()
                .and_then(|n| n.get(None))
                .ok_or(OAuthError::TokenMissesName)?
                .to_string();
            let a_exp = u64::try_from(claims.expiration().timestamp()).ok();
            (
                User {
                    username,
                    email,
                    name,
                },
                a_exp,
            )
        } else {
            self.get_user_info::<D, _, _>(token_result).await?
        };

        let (access_cookie, refresh_cookie, scopes) =
            Self::create_token_cookies(req, token_result);

        let (is_user, is_admin, is_superuser) = scopes.map_or_else(
            || (false, false, false),
            |scopes| {
                scopes
                    .iter()
                    .fold((false, false, false), |(u, a, s), scope| {
                        (
                            u || scope.as_str() == self.user_scope,
                            a || scope.as_str() == self.admin_scope,
                            s || scope.as_str() == self.super_user_scope,
                        )
                    })
            },
        );
        let type_ = if is_admin {
            UserTypes::Admin
        } else if is_superuser {
            UserTypes::Superuser
        } else {
            UserTypes::User
        };
        if !is_user {
            return Err(AppError::OAuth(OAuthError::TokenMissesUserScope));
        }

        let mut cookies = Vec::with_capacity(3);
        let auth =
            Auth::create(req, user.username, &user.name, &user.email, a_exp, type_)
                .await?
                .and_then(|auth| {
                    Some(auth.get_str(req).transpose()?.map(|auth_plain| {
                        let cookie = Self::create_token_cookie(req, &auth_plain);
                        cookies.push(cookie);
                        (auth, auth_plain)
                    }))
                })
                .transpose()?;

        cookies.push(access_cookie);
        if let Some(refresh_cookie) = refresh_cookie {
            cookies.push(refresh_cookie);
        }
        // Ok((cookies, token, token_plain))
        Ok((cookies, auth))
    }

    async fn get_user_info<
        D: Database,
        EF: ExtraTokenFields,
        TT: openidconnect::TokenType,
    >(
        &self,
        token: &StandardTokenResponse<EF, TT>,
    ) -> Result<(User, Option<u64>), OAuthError> {
        let access_token = token.access_token().clone();
        let claims: JwtPayload = dangerous_unsafe_decode(access_token.secret())
            .map_err(OAuthError::ClaimExtraction)?
            .claims;
        let user_info: UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim> = self
            .client
            .user_info(access_token, Some(SubjectIdentifier::new(claims.sub)))
            .map_err(Fail::compat)
            .map_err(OAuthError::NoUserEndpoint)?
            .request_async(async_http_client)
            .await
            .map_err(Fail::compat)
            .map_err(OAuthError::UserInfo)?;
        let username = user_info
            .preferred_username()
            .ok_or(OAuthError::TokenMissesUsername)?
            .to_string();
        let email = user_info
            .email()
            .ok_or(OAuthError::TokenMissesEmail)?
            .to_string();
        let name = user_info
            .name()
            .and_then(|n| n.get(None))
            .ok_or(OAuthError::TokenMissesName)?
            .to_string();
        Ok((
            User {
                username,
                email,
                name,
            },
            Some(claims.exp),
        ))
    }

    fn create_token_cookies<'a, A, D, T, R, EF, TT>(
        req: &R,
        token: &'a StandardTokenResponse<EF, TT>,
    ) -> (String, Option<String>, Option<&'a Vec<Scope>>)
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
        EF: ExtraTokenFields,
        TT: openidconnect::TokenType,
    {
        let access_token = token.access_token();
        let access_cookie = Self::create_access_cookie(req, access_token);

        let refresh_token = token
            .refresh_token()
            .map(|token| Self::create_refresh_cookie(req, token));
        (access_cookie, refresh_token, token.scopes())
    }

    /// Fetches a Token using the code given by the openid callback
    ///
    /// # Errors
    /// Fails if the oauth client is unable to communicate with the provider
    #[inline]
    pub async fn get_token(
        &self,
        code: String,
    ) -> Result<LTokenResponse, OAuthError> {
        self.client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(Fail::compat)
            .map_err(OAuthError::RequestToken)
    }

    /// Fetches a Token using a refresh token
    ///
    /// # Errors
    /// Fails if the oauth client is unable to communicate with the provider
    #[inline]
    pub async fn get_token_by_refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<LTokenResponse, OAuthError> {
        self.client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(Fail::compat)
            .map_err(OAuthError::RequestToken)
    }

    // /// Fetches a Token using a username and password
    // #[inline]
    // pub fn get_token_by_password(
    //     &self,
    //     username: String,
    //     password: String,
    // ) -> Result<LTokenResponse, OAuthError> {
    //     self.client
    //         .exchange_password(
    //             &ResourceOwnerUsername::new(username),
    //             &ResourceOwnerPassword::new(password),
    //         )
    //         .request(http_client)
    //         .map_err(|err| match err {
    //             RequestTokenError::ServerResponse(ref inner)
    //                 if inner.error() == &CoreErrorResponseType::InvalidGrant =>
    //             {
    //                 IronError::new(err.compat(), status::Unauthorized)
    //             }
    //             _ => IronError::new(err.compat(), status::InternalServerError),
    //         })
    // }

    /// Tries to validate the given token
    ///
    /// # Errors
    ///
    /// Fails when the token fails validation
    #[inline]
    pub fn validate<'a, D: Database>(
        &self,
        token: &'a IdToken<
            EmptyAdditionalClaims,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
            CoreJsonWebKeyType,
        >,
    ) -> Result<&'a IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>, OAuthError>
    {
        let nonce = LocalNonceVerify(None);
        token
            .claims(&self.client.id_token_verifier(), nonce)
            .map_err(Fail::compat)
            .map_err(OAuthError::ClaimVerification)
    }

    /// Creates the state Token used in oauth authentication. Requires the
    /// `CsrfToken` from the `OAuthClient`
    #[inline]
    pub fn create_state_cookie<A, D, T, R>(req: &R, csrf_token: &CsrfToken) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        create_cookie(
            req,
            "state",
            csrf_token.secret(),
            Some(300),
            None,
            Some("/"),
        )
    }

    /// Fetches the state cookie value
    #[inline]
    pub fn get_state_cookie<A, D, T, R>(req: &R) -> Option<&str>
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        req.get_cookie("state=")
    }

    /// Deletes the state Token used in oauth authentication.
    #[inline]
    pub fn delete_state_cookie<A, D, T, R>(req: &R) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        delete_cookie(req, "state", Some("/"))
    }

    /// Creates the access token cookie
    #[inline]
    pub fn create_token_cookie<A, D, T, R>(req: &R, data: &str) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        create_cookie(req, "token", data, None, None, Some("/"))
    }

    /// Fetches the access token cookie value
    #[inline]
    pub fn get_token_cookie<A, D, T, R>(req: &R) -> Option<&str>
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        req.get_cookie("token=")
    }

    /// Deletes the access token cookie.
    #[inline]
    pub fn delete_token_cookie<A, D, T, R>(req: &R) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        delete_cookie(req, "token", Some("/"))
    }

    /// Creates the access token cookie
    #[inline]
    pub fn create_access_cookie<A, D, T, R>(
        req: &R,
        access_token: &AccessToken,
    ) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        create_cookie(
            req,
            "access_token",
            access_token.secret(),
            None,
            None,
            Some("/"),
        )
    }

    /// Fetches the access token cookie value
    #[inline]
    pub fn get_access_cookie<A, D, T, R>(req: &R) -> Option<&str>
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        req.get_cookie("access_token=")
    }

    /// Deletes the access token cookie.
    #[inline]
    pub fn delete_access_cookie<A, D, T, R>(req: &R) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        delete_cookie(req, "access_token", Some("/"))
    }

    /// Creates the refresh token cookie
    #[inline]
    pub fn create_refresh_cookie<A, D, T, R>(
        req: &R,
        refresh_token: &RefreshToken,
    ) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        create_cookie(
            req,
            "refresh_token",
            refresh_token.secret(),
            None,
            None,
            Some("/"),
        )
    }

    /// Fetches the refresh token cookie value
    #[inline]
    pub fn get_refresh_cookie<A, D, T, R>(req: &R) -> Option<&str>
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        req.get_cookie("refresh_token=")
    }

    /// Deletes the refresh token cookie.
    #[inline]
    pub fn delete_refresh_cookie<A, D, T, R>(req: &R) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        delete_cookie(req, "refresh_token", Some("/"))
    }

    /// Creates the redirect Cookie which allows redirecting after authentication
    #[inline]
    pub fn create_redirect_cookie<A, D, T, R>(req: &R, url: &str) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        create_cookie(req, "redirect", url, None, None, Some("/"))
    }

    /// Fetches the redirect cookie value
    #[inline]
    pub fn get_redirect_cookie<A, D, T, R>(req: &R) -> Option<&str>
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        req.get_cookie("redirect=")
    }

    /// Deletes the redirect cookie after usage.
    #[inline]
    pub fn delete_redirect_cookie<A, D, T, R>(req: &R) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        delete_cookie(req, "redirect", Some("/"))
    }
}

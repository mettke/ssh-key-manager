use aes::Aes128;
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use core_common::{
    async_trait::async_trait,
    base64,
    database::{Create, Database, FetchByUid, Save},
    http::{
        header::{HeaderValue, SET_COOKIE},
        response,
    },
    jsonwebtoken::{
        self, decode, encode, Algorithm, DecodingKey, EncodingKey, Header,
        Validation,
    },
    log,
    objects::User,
    sec::{Auth, AuthMethod, OAuth2, PreAuth},
    serde::{Deserialize, Serialize},
    types::{Id, UserTypes},
    web::{
        get_current_time_and_add, AppError, Request, TemplateEngine, UserContainer,
    },
};
use rand::{rngs::OsRng, RngCore};
use std::{error, fmt, string};

/// Database encountered an Error
#[derive(Debug)]
pub enum TokenError {
    /// Error while trying to decode token
    Base64(base64::DecodeError),
    /// IV is not valid
    InvalidIvKey(block_modes::InvalidKeyIvLength),
    /// BlockMode invalid
    InvalidBlockMode(block_modes::BlockModeError),
    /// Result is not valid utf8
    InvalidEncoding(string::FromUtf8Error),
    /// Unable to encode token
    TokenEncoding(jsonwebtoken::errors::Error),
    /// Errors while gathering Random Data
    RandomGathering(rand::Error),
    /// Token is not the expected size
    Misaligned,
    /// Secret is not large enough
    InvalidSecret,
}

impl fmt::Display for TokenError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base64(err) => err.fmt(f),
            Self::InvalidIvKey(err) => err.fmt(f),
            Self::InvalidBlockMode(err) => err.fmt(f),
            Self::InvalidEncoding(err) => err.fmt(f),
            Self::TokenEncoding(err) => err.fmt(f),
            Self::RandomGathering(err) => err.fmt(f),
            Self::Misaligned => write!(f, "Token misaligned"),
            Self::InvalidSecret => {
                write!(f, "App Secret must be at least 128 bits long")
            }
        }
    }
}

impl error::Error for TokenError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Base64(err) => Some(err),
            Self::InvalidIvKey(err) => Some(err),
            Self::InvalidBlockMode(err) => Some(err),
            Self::InvalidEncoding(err) => Some(err),
            Self::TokenEncoding(err) => Some(err),
            Self::RandomGathering(err) => Some(err),
            Self::Misaligned | Self::InvalidSecret => None,
        }
    }
}

/// User Token containing general Information about to user to reduce
/// database queries
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Token {
    /// By whom token was issued
    pub iss: String,
    /// Expiration Time of the Token
    pub exp: u64,
    /// User Id inside of the Database
    pub id: Id,
    /// Unique user uid
    pub uid: String,
    /// Full Username if set
    pub name: Option<String>,
    /// Type of the user account
    pub type_: UserTypes,
}

impl Token {
    fn decode_token_string<D, T, R>(req: &R, token: &str) -> Option<Self>
    where
        D: Database,
        T: TemplateEngine,
        R: Request<Self, D, T>,
    {
        let secret = req.get_base_data();
        let validation = Validation {
            algorithms: vec![Algorithm::HS256],
            ..Validation::default()
        };
        let key = DecodingKey::from_secret(&secret.app_secret);
        Self::decrypt::<D, T, R>(token, &secret.app_secret)
            .map_err(|err| {
                log::warn!("Unable to decrypt cookie: {}", err);
            })
            .ok()
            .and_then(|token| {
                decode(&token, &key, &validation)
                    .map_err(|err| {
                        log::warn!("Unable to decode cookie: {}", err);
                    })
                    .ok()
            })
            .map(|token| token.claims)
    }

    fn decrypt<D: Database, T: TemplateEngine, R: Request<Self, D, T>>(
        data: &str,
        key: &[u8],
    ) -> Result<String, AppError<Self, D, T, R>> {
        let data = base64::decode(data.as_bytes())
            .map_err(TokenError::Base64)
            .map_err(AppError::AuthError)?;

        if key.len() >= 16 {
            if let (Some(data), Some(iv)) = (
                data.get(..data.len().saturating_sub(16)),
                data.get(data.len().saturating_sub(16)..),
            ) {
                #[allow(clippy::indexing_slicing)]
                let cvc: Cbc<Aes128, Pkcs7> = Cbc::new_var(&key[..16], iv)
                    .map_err(TokenError::InvalidIvKey)
                    .map_err(AppError::AuthError)?;
                let data = cvc
                    .decrypt_vec(data)
                    .map_err(TokenError::InvalidBlockMode)
                    .map_err(AppError::AuthError)?;
                String::from_utf8(data)
                    .map_err(TokenError::InvalidEncoding)
                    .map_err(AppError::AuthError)
            } else {
                Err(AppError::AuthError(TokenError::Misaligned))
            }
        } else {
            Err(AppError::AuthError(TokenError::InvalidSecret))
        }
    }

    fn encrypt<D: Database, T: TemplateEngine, R: Request<Self, D, T>>(
        data: &str,
        key: &[u8],
    ) -> Result<String, AppError<Self, D, T, R>> {
        let mut rng = OsRng::default();
        if key.len() >= 16 {
            let mut iv = [0_u8; 16];
            rng.try_fill_bytes(&mut iv)
                .map_err(TokenError::RandomGathering)
                .map_err(AppError::AuthError)?;
            #[allow(clippy::indexing_slicing)]
            let cvc: Cbc<Aes128, Pkcs7> = Cbc::new_var(&key[..16], &iv)
                .map_err(TokenError::InvalidIvKey)
                .map_err(AppError::AuthError)?;
            let mut data = cvc.encrypt_vec(data.as_bytes());
            data.extend_from_slice(&iv);
            let data = base64::encode(data);
            Ok(data)
        } else {
            Err(AppError::AuthError(TokenError::InvalidSecret))
        }
    }
}

#[allow(clippy::type_repetition_in_bounds)]
#[async_trait]
impl Auth for Token {
    type AuthError = TokenError;

    #[inline]
    async fn authenticate<D, T, R>(
        req: &R,
        res: &mut response::Builder,
    ) -> Option<Self>
    where
        for<'a> D: Database
            + FetchByUid<'a, PreAuth, User<'a>, D>
            + Create<'a, PreAuth, User<'a>, D>
            + Save<'a, PreAuth, User<'a>, D>,
        T: TemplateEngine,
        R: Request<Self, D, T> + Sync,
    {
        let mut auth = OAuth2::get_token_cookie(req)
            .and_then(|token| Self::decode_token_string(req, token));
        if auth.is_none() {
            log::warn!("Using refresh token");
            if let Some(refresh_token) = OAuth2::get_refresh_cookie(req) {
                let client = &req.get_base_data().oauth;
                if let Ok(token) = client
                    .get_token_by_refresh_token(refresh_token.into())
                    .await
                {
                    auth = client
                        .handle_token(&token, req)
                        .await
                        .ok()
                        .and_then(|(cookies, auth)| {
                            if let Some(header) = res.headers_mut() {
                                for cookie in cookies {
                                    let value =
                                        HeaderValue::from_str(&cookie).ok()?;
                                    let _ = header.append(SET_COOKIE, value);
                                }
                            }
                            auth
                        })
                        .map(|(auth, _)| auth);
                }
            }
        }
        auth
    }

    #[inline]
    async fn create<D, T, R>(
        req: &R,
        username: String,
        name: &str,
        email: &str,
        exp: Option<u64>,
        type_: UserTypes,
    ) -> Result<Option<Self>, AppError<Self, D, T, R>>
    where
        for<'a> D: Database
            + FetchByUid<'a, PreAuth, User<'a>, D>
            + Create<'a, PreAuth, User<'a>, D>
            + Save<'a, PreAuth, User<'a>, D>,
        T: TemplateEngine,
        R: Request<Self, D, T>,
    {
        let t_exp = exp
            .unwrap_or_else(|| get_current_time_and_add(15_u64.saturating_mul(60)));
        let db = req.get_database();
        let user =
            User::update_or_create_user(db, &PreAuth, &username, name, email, type_)
                .await
                .map_err(AppError::DatabaseError)?;
        let token = Self {
            iss: "SSH Key Authority".to_string(),
            exp: t_exp,
            id: user.entity_id.into_owned(),
            uid: username,
            name: None,
            type_,
        };
        Ok(Some(token))
    }

    #[inline]
    fn is_admin(&self) -> bool {
        self.type_ == UserTypes::Admin
    }

    #[inline]
    fn get_uid(&self) -> &str {
        &self.uid
    }

    #[inline]
    fn get_id(&self) -> &Id {
        &self.id
    }

    #[inline]
    fn get_user_container(&self) -> UserContainer<'_> {
        UserContainer {
            id: Some(&self.id),
            uid: Some(&self.uid),
            name: self.name.as_deref(),
            is_admin: self.type_ == UserTypes::Admin,
            is_superuser: self.type_ == UserTypes::Superuser,
        }
    }

    #[inline]
    fn is_supported(method: AuthMethod) -> bool {
        match method {
            AuthMethod::OAuth => true,
        }
    }

    #[inline]
    fn get_str<D, T, R>(
        &self,
        req: &R,
    ) -> Result<Option<String>, AppError<Self, D, T, R>>
    where
        D: Database,
        T: TemplateEngine,
        R: Request<Self, D, T>,
    {
        let data = req.get_base_data();
        let header = Header::new(Algorithm::HS256);
        let token_str =
            encode(&header, self, &EncodingKey::from_secret(&data.app_secret))
                .map_err(TokenError::TokenEncoding)
                .map_err(AppError::AuthError)?;
        Self::encrypt(&token_str, &data.app_secret).map(Some)
    }
}

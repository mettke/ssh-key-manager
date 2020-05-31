use crate::{
    database::Database,
    http::{
        header::{HeaderValue, SET_COOKIE},
        response,
    },
    sec::Auth,
    web::{create_cookie, delete_cookie, AppError, Request, TemplateEngine},
};
use base64::decode;
use csrf::{ChaCha20Poly1305CsrfProtection, CsrfProtection, UnencryptedCsrfCookie};
use std::fmt;

/// Internal Strucutre for Csrf verification and generation
pub struct CsrfToken {
    /// Whether the csrf token is valid
    pub valid: bool,
    cookie: Option<UnencryptedCsrfCookie>,
    gen: ChaCha20Poly1305CsrfProtection,
}

impl fmt::Debug for CsrfToken {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CsrfToken")
            .field("valid", &self.valid)
            .field("cookie", &self.cookie)
            .field("gen", &"[...]".to_string())
            .finish()
    }
}

impl CsrfToken {
    /// Creates a `CsrfToken` from an incoming request
    #[inline]
    pub fn from<A, D, T, R>(req: &R) -> Self
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let app_secret = req.get_base_data().app_secret;
        let gen = ChaCha20Poly1305CsrfProtection::from_key(app_secret);
        Self {
            valid: false,
            cookie: None,
            gen,
        }
    }

    /// Verifies whether the given token and the csrf cookie inside the request
    /// are valid
    #[inline]
    pub fn verify<A, D, T, R>(req: &R, csrf_token: Option<&str>) -> Self
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let app_secret = req.get_base_data().app_secret;
        let gen = ChaCha20Poly1305CsrfProtection::from_key(app_secret);
        let token = csrf_token
            .and_then(|token| decode(token).ok())
            .and_then(|data| gen.parse_token(&data).ok());
        let cookie = Self::get_state_cookie(req)
            .and_then(|cookie| decode(cookie).ok())
            .and_then(|data| gen.parse_cookie(&data).ok());
        let valid = match (token, cookie.as_ref()) {
            (Some(token), Some(cookie)) => gen.verify_token_pair(&token, cookie),
            _ => false,
        };
        Self { valid, cookie, gen }
    }

    /// Generates a new csrf token
    ///
    /// # Errors
    /// Fails when a csrf token could not be created
    #[inline]
    pub fn generate<A, D, T, R>(
        self,
        req: &R,
        res: &mut response::Builder,
    ) -> Result<String, AppError<A, D, T, R>>
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let old_cookie = self.cookie.and_then(|c| {
            let c = c.value();
            if c.len() < 64 {
                None
            } else {
                let mut buf = [0; 64];
                buf.copy_from_slice(c);
                Some(buf)
            }
        });
        let (token, cookie) = self
            .gen
            .generate_token_pair(old_cookie.as_ref(), 15_i64.wrapping_mul(60))?;
        if let Some(header) = res.headers_mut() {
            let cookie = Self::create_state_cookie(req, &cookie);
            let value =
                HeaderValue::from_str(&cookie).map_err(AppError::HttpHeader)?;
            let _ = header.append(SET_COOKIE, value);
        }
        Ok(token.b64_string())
    }

    /// Creates the state Token used in oauth authentication. Requires the
    /// `CsrfToken` from the `OAuthClient`
    #[inline]
    pub fn create_state_cookie<A, D, T, R>(
        req: &R,
        cookie: &csrf::CsrfCookie,
    ) -> String
    where
        A: Auth,
        D: Database,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        create_cookie(req, "csrf", &cookie.b64_string(), None, None, Some("/"))
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
        req.get_cookie("csrf=")
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
        delete_cookie(req, "csrf", Some("/"))
    }
}

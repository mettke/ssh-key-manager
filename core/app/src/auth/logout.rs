use core_common::{
    database::Database,
    http::{
        header::{HeaderValue, SET_COOKIE},
        method::Method,
        response::Response,
    },
    sec::{Auth, CsrfToken, OAuth2},
    url::form_urlencoded,
    web::{
        invalid_method, redirect, AppError, Request, ResponseType, TemplateEngine,
    },
};
use std::borrow::Cow;

/// Servers the logout.
///
/// # Errors
/// Fails when the response creation failed
#[inline]
pub async fn index<A, D, T, R>(
    req: &R,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    #[allow(indirect_structural_match)]
    match *req.get_method() {
        Method::GET => index_get(req).await,
        _ => invalid_method(&[Method::GET]),
    }
}

async fn index_get<A, D, T, R>(
    req: &R,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let mut res = Response::builder();
    let uri = req.get_uri();
    let red_url: Cow<'_, str> = uri
        .query()
        .map(str::as_bytes)
        .and_then(|bytes| {
            form_urlencoded::parse(bytes)
                .find(|(key, value)| key == "red" && !value.is_empty())
        })
        .map_or(Cow::Borrowed("/app"), |(_, red)| red);

    if let Some(header) = res.headers_mut() {
        let cookie = OAuth2::delete_token_cookie(req);
        let value = HeaderValue::from_str(&cookie).map_err(AppError::HttpHeader)?;
        let _ = header.append(SET_COOKIE, value);

        let cookie = OAuth2::delete_access_cookie(req);
        let value = HeaderValue::from_str(&cookie).map_err(AppError::HttpHeader)?;
        let _ = header.append(SET_COOKIE, value);

        let cookie = OAuth2::delete_refresh_cookie(req);
        let value = HeaderValue::from_str(&cookie).map_err(AppError::HttpHeader)?;
        let _ = header.append(SET_COOKIE, value);

        let cookie = CsrfToken::delete_state_cookie(req);
        let value = HeaderValue::from_str(&cookie).map_err(AppError::HttpHeader)?;
        let _ = header.append(SET_COOKIE, value);
    }
    redirect(req, res, &red_url, false, true, false)
}

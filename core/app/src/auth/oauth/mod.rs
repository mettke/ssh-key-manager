use core_common::{
    database::Database,
    http::{
        header::{HeaderValue, SET_COOKIE},
        method::Method,
        response::Response,
    },
    sec::{Auth, OAuth2},
    url::form_urlencoded,
    web::{
        invalid_method, redirect, AppError, Request, ResponseType, TemplateEngine,
    },
};

/// Callback route for the oauth flow
pub mod callback;

/// Servers the oauth route.
///
/// # Errors
/// Fails when the communication with the provider fails
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
    let client = &req.get_base_data().oauth;
    let (auth_url, csrf_token, _nonce) = client.authorize_url();
    let cookie = OAuth2::create_state_cookie(req, &csrf_token);

    let uri = req.get_uri();
    let red = uri
        .query()
        .map(str::as_bytes)
        .and_then(|bytes| {
            form_urlencoded::parse(bytes)
                .find(|(key, value)| key == "red" && !value.is_empty())
        })
        .map(|(_, red)| OAuth2::create_redirect_cookie(req, &red));

    let mut res = Response::builder();
    if let Some(header) = res.headers_mut() {
        let value = HeaderValue::from_str(&cookie).map_err(AppError::HttpHeader)?;
        let _ = header.append(SET_COOKIE, value);

        if let Some(red) = red {
            let value = HeaderValue::from_str(&red).map_err(AppError::HttpHeader)?;
            let _ = header.append(SET_COOKIE, value);
        }
    }
    redirect(req, res, auth_url.as_str(), false, false, false)
}

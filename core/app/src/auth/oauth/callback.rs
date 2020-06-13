use core_common::{
    database::{Create, Database, FetchByUid, Save},
    http::{
        header::{HeaderValue, SET_COOKIE},
        method::Method,
        response::Response,
    },
    objects::User,
    sec::{Auth, OAuth2, PreAuth},
    web::{
        get_query_parameters, invalid_method, redirect, AppError, Request,
        ResponseType, TemplateEngine,
    },
};

/// Servers the oauth callback route.
///
/// # Errors
/// Fails when the communication with the provider fails
#[inline]
pub async fn index<A, D, T, R>(
    req: &R,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a> D: Database
        + FetchByUid<'a, PreAuth, User<'a>, D>
        + Create<'a, PreAuth, User<'a>, D>
        + Save<'a, PreAuth, User<'a>, D>,
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
    for<'a> D: Database
        + FetchByUid<'a, PreAuth, User<'a>, D>
        + Create<'a, PreAuth, User<'a>, D>
        + Save<'a, PreAuth, User<'a>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let mut red_url = OAuth2::get_redirect_cookie(req).unwrap_or("/app/");
    let parameters = get_query_parameters(req);

    let mut res = Response::builder();
    let params =
        parameters.fold((None, None), |params, (key, value)| match key.as_ref() {
            "code" => (Some(value), params.1),
            "state" => (params.0, Some(value)),
            _ => params,
        });
    if let (Some(code), Some(state)) = params {
        let state_cookie = OAuth2::get_state_cookie(req);
        if Some(&state[..]) == state_cookie {
            let client = &req.get_base_data().oauth;
            let token_result = client.get_token(code.to_string()).await?;
            let (cookies, _) = client.handle_token(&token_result, req).await?;
            if let Some(header) = res.headers_mut() {
                for cookie in cookies {
                    let value = HeaderValue::from_str(&cookie)
                        .map_err(AppError::HttpHeader)?;
                    let _ = header.append(SET_COOKIE, value);
                }
            }
        } else {
            red_url = "/app/?error=2"
        }
    } else {
        red_url = "/app/?error=1"
    }
    let delete_state = OAuth2::delete_state_cookie(req);
    let delete_red = OAuth2::delete_redirect_cookie(req);
    if let Some(header) = res.headers_mut() {
        let value =
            HeaderValue::from_str(&delete_state).map_err(AppError::HttpHeader)?;
        let _ = header.append(SET_COOKIE, value);
        let value =
            HeaderValue::from_str(&delete_red).map_err(AppError::HttpHeader)?;
        let _ = header.append(SET_COOKIE, value);
    }
    redirect(req, res, red_url, false, true, false)
}

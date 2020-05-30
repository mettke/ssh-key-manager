use crate::{
    database::{Database, DatabaseError},
    http::{
        self,
        header::{HeaderValue, ALLOW, LOCATION},
        method::Method,
        response::{self, Response},
        status::StatusCode,
        uri,
    },
    sec::{Auth, OAuthError},
    serde::Serialize,
    web::{self, BaseContainer, Request, ResponseType, TemplateEngine},
};
use std::{convert::TryFrom, error, fmt};

/// Database encountered an Error
#[derive(Debug)]
pub enum AppError<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>> {
    /// Error when creating a Response
    Http(http::Error),
    /// Error when creating a HeaderValue for a Response
    HttpHeader(http::header::InvalidHeaderValue),
    /// Error when trying to render a template
    Render(web::RenderError<T>),
    /// Error trying to convert uri
    UriParts(http::uri::InvalidUriParts),
    /// Error trying to convert path
    Uri(http::uri::InvalidUri),
    /// Error while trying to authenticate via OAuth
    OAuth(OAuthError),
    /// Unable to fetch from Database
    DatabaseError(DatabaseError<D>),
    /// Error inside of the Authentication
    AuthError(A::AuthError),
    /// Error inside of the Request
    RequestError(R::RequestError),
    /// Error while creating csrf
    CsrfError(csrf::CsrfError),
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>> fmt::Display
    for AppError<A, D, T, R>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(err) => write!(f, "Error while crating response: {}", err),
            Self::HttpHeader(err) => write!(
                f,
                "Error while crating a header value for a response: {}",
                err
            ),
            Self::Render(err) => {
                write!(f, "Error while rendering template: {}", err)
            }
            Self::UriParts(err) => write!(f, "Error while converting uri: {}", err),
            Self::Uri(err) => write!(f, "Error while converting path: {}", err),
            Self::OAuth(err) => {
                write!(f, "Error while trying to authenticate via oauth: {}", err)
            }
            Self::DatabaseError(err) => write!(
                f,
                "Error while trying to communicate with the database: {}",
                err
            ),
            Self::AuthError(err) => {
                write!(f, "Error while trying to authenticate: {}", err)
            }
            Self::RequestError(err) => {
                write!(f, "Error while trying load request: {}", err)
            }
            Self::CsrfError(err) => write!(f, "Error while creating csrf: {}", err),
        }
    }
}

impl<
        A: 'static + Auth,
        D: 'static + Database,
        T: 'static + TemplateEngine,
        R: 'static + Request<A, D, T>,
    > error::Error for AppError<A, D, T, R>
{
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Http(err) => Some(err),
            Self::HttpHeader(err) => Some(err),
            Self::Render(err) => Some(err),
            Self::UriParts(err) => Some(err),
            Self::Uri(err) => Some(err),
            Self::OAuth(err) => Some(err),
            Self::DatabaseError(err) => Some(err),
            Self::AuthError(err) => Some(err),
            Self::RequestError(err) => Some(err),
            Self::CsrfError(err) => Some(err),
        }
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>> From<http::Error>
    for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: http::Error) -> Self {
        Self::Http(err)
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>>
    From<http::header::InvalidHeaderValue> for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: http::header::InvalidHeaderValue) -> Self {
        Self::HttpHeader(err)
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>>
    From<web::RenderError<T>> for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: web::RenderError<T>) -> Self {
        Self::Render(err)
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>>
    From<http::uri::InvalidUriParts> for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: http::uri::InvalidUriParts) -> Self {
        Self::UriParts(err)
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>>
    From<http::uri::InvalidUri> for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: http::uri::InvalidUri) -> Self {
        Self::Uri(err)
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>> From<OAuthError>
    for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: OAuthError) -> Self {
        Self::OAuth(err)
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>>
    From<DatabaseError<D>> for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: DatabaseError<D>) -> Self {
        Self::DatabaseError(err)
    }
}

impl<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>>
    From<csrf::CsrfError> for AppError<A, D, T, R>
{
    #[inline]
    fn from(err: csrf::CsrfError) -> Self {
        Self::CsrfError(err)
    }
}

/// Serves the login page. Should be used when the user is not authenticated
/// and authentication is required
///
/// # Errors
/// Fails when the login template could not be rendered
#[inline]
pub fn serve_login<A, D, T, R>(
    req: &R,
    res: response::Builder,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let url = req.get_uri().path();
    let container = BaseContainer {
        ..BaseContainer::new(req.get_base_view(), &(), &(), url)
    };
    serve_template(req, res, "site_login", &container)
}

/// Serves the template with the given name using the given value
///
/// # Errors
/// Fails when the template could not be rendered
#[inline]
pub fn serve_template<A, D, T, R, S, P>(
    req: &R,
    res: response::Builder,
    name: &str,
    data: &BaseContainer<'_, S, P>,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
    S: Serialize,
    P: Serialize,
{
    let template_engine = req.get_template_engine();
    let content = template_engine.render(name, data)?;
    res.header("Content-Type", "text/html; charset=UTF-8")
        .status(StatusCode::OK)
        .body(ResponseType::String(content))
        .map_err(AppError::Http)
}

/// Serves the 404 page
///
/// # Errors
/// Fails when the reponse could not be created
#[inline]
pub fn not_found<A: Auth, D: Database, T: TemplateEngine, R: Request<A, D, T>>(
) -> Result<Response<ResponseType>, AppError<A, D, T, R>> {
    let builder = Response::builder().status(StatusCode::NOT_FOUND);
    builder.body(ResponseType::Empty).map_err(AppError::Http)
}

/// Serves the 405 method
///
/// # Errors
/// Fails when the reponse could not be created
#[inline]
pub fn invalid_method<
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
>(
    allowed_methods: &[Method],
) -> Result<Response<ResponseType>, AppError<A, D, T, R>> {
    let mut builder = Response::builder().status(StatusCode::METHOD_NOT_ALLOWED);
    let allow = allowed_methods
        .iter()
        .map(Method::as_str)
        .collect::<Vec<&str>>()
        .join(", ");
    if let Some(header) = builder.headers_mut() {
        let value = HeaderValue::from_str(&allow).map_err(AppError::HttpHeader)?;
        let _ = header.append(ALLOW, value);
    }
    builder.body(ResponseType::Empty).map_err(AppError::Http)
}

/// Returns a redirect to the home page
///
/// # Errors
/// Fails when the reponse could not be created
#[inline]
pub fn redirect_home<A, D, T, R>(
    req: &R,
    res: response::Builder,
    permanent: bool,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    redirect(req, res, "/app/", permanent, true, false)
}

/// Redirects to the given path
///
/// # Errors
/// Fails when the reponse could not be created
#[inline]
pub fn redirect<A, D, T, R>(
    req: &R,
    res: response::Builder,
    url: &str,
    permanent: bool,
    path_only: bool,
    get_next: bool,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let uri_str;
    let location = if path_only {
        let mut parts = req.get_uri().clone().into_parts();
        parts.path_and_query = Some(uri::PathAndQuery::try_from(url)?);
        let uri = uri::Uri::from_parts(parts)?;
        uri_str = format!("{}", uri);
        &uri_str
    } else {
        url
    };
    let status = if permanent {
        StatusCode::PERMANENT_REDIRECT
    } else if get_next {
        StatusCode::SEE_OTHER
    } else {
        StatusCode::TEMPORARY_REDIRECT
    };
    let mut res = res.status(status);
    if let Some(header) = res.headers_mut() {
        let value = HeaderValue::from_str(location).map_err(AppError::HttpHeader)?;
        let _ = header.append(LOCATION, value);
    }
    res.body(ResponseType::Empty).map_err(AppError::Http)
}

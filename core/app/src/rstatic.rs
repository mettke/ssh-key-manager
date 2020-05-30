use core_common::{
    database::Database,
    http::{
        header::{HeaderValue, CACHE_CONTROL, CONTENT_TYPE},
        method::Method,
        response::Response,
        status::StatusCode,
    },
    sec::Auth,
    tokio::fs::File,
    web::{
        invalid_method, not_found, AppError, Request, ResponseType, TemplateEngine,
    },
};
use std::path::Path;

/// Servers the static file route.
///
/// # Errors
/// Fails on file io
#[inline]
pub async fn index<A, D, T, R>(
    req: &R,
    path: &[String],
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    #[allow(indirect_structural_match)]
    match *req.get_method() {
        Method::GET => index_get(path).await,
        _ => invalid_method(&[Method::GET]),
    }
}

async fn index_get<A, D, T, R>(
    path: &[String],
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let path: String = path
        .iter()
        .skip(2)
        .map(|s| s.as_ref())
        .collect::<Vec<&str>>()
        .join("/");
    let path = format!("./static/{}", path);
    let path = Path::new(&path);
    if let Ok(file) = File::open(&path).await {
        let mut builder = Response::builder().status(StatusCode::OK);
        if let Some(h) = builder.headers_mut() {
            let _ = h.append(
                CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=86400"),
            );
            if let Some(mime) = mime_guess::from_path(path).first() {
                let _ = h.append(
                    CONTENT_TYPE,
                    HeaderValue::from_str(mime.essence_str())
                        .map_err(AppError::HttpHeader)?,
                );
            }
        }
        builder
            .body(ResponseType::File(file))
            .map_err(AppError::Http)
    } else {
        not_found()
    }
}

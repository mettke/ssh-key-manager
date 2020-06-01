use crate::{
    async_trait::async_trait,
    database::Database,
    http::response::Response,
    sec::Auth,
    web::{Request, ResponseType, TemplateEngine},
};
use std::{
    convert::Infallible, error, fmt, future::Future, net::AddrParseError, path::Path,
};

/// Database encountered an Error
#[derive(Debug)]
pub enum ServerError<A, D, T, R, S>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: 'static + Request<A, D, T>,
    S: Server<A, D, T, R>,
{
    /// Unable to parse given address
    AddrError(AddrParseError),
    /// Custom Error from the underlying server system
    Custom(S::ServerError),
}

impl<A, D, T, R, S> fmt::Display for ServerError<A, D, T, R, S>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: 'static + Request<A, D, T>,
    S: Server<A, D, T, R>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddrError(err) => write!(f, "Unable to parse address: {}", err),
            Self::Custom(err) => write!(f, "Custom Server Error: {}", err),
        }
    }
}

impl<A, D, T, R, S> error::Error for ServerError<A, D, T, R, S>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: 'static + Request<A, D, T>,
    S: Server<A, D, T, R>,
    <S as Server<A, D, T, R>>::ServerError: 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::AddrError(err) => Some(err),
            Self::Custom(err) => Some(err),
        }
    }
}

impl<A, D, T, R, S> From<AddrParseError> for ServerError<A, D, T, R, S>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: 'static + Request<A, D, T>,
    S: Server<A, D, T, R>,
{
    #[inline]
    fn from(err: AddrParseError) -> Self {
        Self::AddrError(err)
    }
}

/// Required methods for a server
#[allow(clippy::type_repetition_in_bounds)]
#[async_trait]
pub trait Server<
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: 'static + Request<A, D, T>,
>: Sized + fmt::Debug
{
    /// the custom error for the server.
    type ServerError: error::Error;

    /// Starts the Server. Server must stop using Ctrl+C
    ///
    /// # Errors
    /// Fails when the server is unable to start.
    async fn start_server<F, H, S>(
        self,
        addr: &str,
        port: u16,
        signal: S,
        handler: H,
        tls_cfg: Option<(&Path, &Path)>,
    ) -> Result<(), ServerError<A, D, T, R, Self>>
    where
        S: Send + Future<Output = ()>,
        H: 'static + Send + Sync + Fn(R) -> F,
        F: Send + Future<Output = Result<Response<ResponseType>, Infallible>>;
}

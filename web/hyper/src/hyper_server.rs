use crate::hyper_request::HyperRequest;
use core_common::{
    async_trait::async_trait,
    database::{Create, Database, FetchByUid, Save},
    http::response::Response,
    objects::User,
    sec::{Auth, PreAuth},
    tokio::{
        io::{AsyncRead, AsyncWrite},
        net::{TcpListener, TcpStream},
        stream::{Stream, StreamExt},
    },
    web::{BaseData, BaseView, ResponseType, Server, ServerError, TemplateEngine},
};
use futures::{future::FutureExt, stream::TryStreamExt};
use hyper::{
    server::{accept::Accept, Server as HServer},
    service::{make_service_fn, service_fn},
    Body, Request as HRequest, Response as HResponse,
};
use rustls::{internal::pemfile, ServerConfig};
use std::{
    convert::Infallible,
    error, fmt, fs,
    future::Future,
    io,
    marker::PhantomData,
    net::{IpAddr, SocketAddr},
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio_rustls::{server::TlsStream, TlsAcceptor};
use tokio_util::codec::{BytesCodec, FramedRead};

/// Errors that might happen inside the `HyperServer`
#[allow(variant_size_differences)]
#[derive(Debug)]
pub enum HyperError {
    /// Error from hyper itself
    Hyper(hyper::Error),
    /// Error while trying to setup tls
    Tls(rustls::TLSError),
    /// Error while trying to listen to addr
    Io(std::io::Error),
    /// Error while loading certificates
    Certificate,
    /// Error while loading privatekey
    PrivateKey,
}

impl fmt::Display for HyperError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hyper(err) => err.fmt(f),
            Self::Tls(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
            Self::Certificate => write!(f, "Unable to load certificates from file"),
            Self::PrivateKey => write!(f, "Unable to load private key from file"),
        }
    }
}

impl error::Error for HyperError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Hyper(err) => Some(err),
            Self::Tls(err) => Some(err),
            Self::Io(err) => Some(err),
            Self::Certificate | Self::PrivateKey => None,
        }
    }
}

/// Server using hyper as framework
#[derive(Debug)]
pub struct HyperServer<
    A: 'static + Auth,
    D: 'static + Database,
    T: 'static + TemplateEngine,
> {
    base_view: Arc<BaseView<'static>>,
    base_data: Arc<BaseData>,
    database: Arc<D>,
    templates: Arc<T>,
    a: PhantomData<A>,
}

impl<A: 'static + Auth, D: 'static + Database, T: 'static + TemplateEngine>
    HyperServer<A, D, T>
{
    #[must_use]
    #[inline]
    /// Create a new `HyperServer`
    pub fn new(
        database: Arc<D>,
        templates: Arc<T>,
        base_view: Arc<BaseView<'static>>,
        base_data: Arc<BaseData>,
    ) -> Self {
        Self {
            base_view,
            base_data,
            database,
            templates,
            a: PhantomData,
        }
    }
}

impl<A: 'static + Auth, D: 'static + Database, T: 'static + TemplateEngine> Clone
    for HyperServer<A, D, T>
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            base_view: Arc::clone(&self.base_view),
            base_data: Arc::clone(&self.base_data),
            database: Arc::<D>::clone(&self.database),
            templates: Arc::<T>::clone(&self.templates),
            a: self.a,
        }
    }
}

impl<A: 'static + Auth, D: 'static + Database, T: 'static + TemplateEngine>
    HyperServer<A, D, T>
{
    async fn handle_req<H, F>(
        self,
        remote_addr: SocketAddr,
        req: HRequest<Body>,
        handler: Arc<H>,
    ) -> Result<HResponse<Body>, Infallible>
    where
        H: Send + Sync + Fn(HyperRequest<A, D, T>) -> F,
        F: Send + Future<Output = Result<Response<ResponseType>, Infallible>>,
    {
        let (header, body) = req.into_parts();
        let request = HyperRequest {
            base_view: self.base_view,
            base_data: self.base_data,
            database: self.database,
            templates: self.templates,
            remote_addr,
            header,
            body: Some(body),
            auth: None,
        };
        let res = handler(request).await?;
        Ok::<_, Infallible>(res.map(|body| match body {
            ResponseType::Empty => Body::empty(),
            ResponseType::File(file) => {
                let stream = FramedRead::new(file, BytesCodec::new());
                Body::wrap_stream(stream)
            }
            ResponseType::String(string) => Body::from(string),
        }))
    }
}

#[allow(clippy::type_repetition_in_bounds)]
#[async_trait]
impl<A, D, T> Server<A, D, T, HyperRequest<A, D, T>> for HyperServer<A, D, T>
where
    A: Auth,
    for<'a> D: Database
        + FetchByUid<'a, PreAuth, User<'a>, D>
        + Create<'a, PreAuth, User<'a>, D>
        + Save<'a, PreAuth, User<'a>, D>,
    T: 'static + TemplateEngine,
{
    type ServerError = HyperError;

    #[inline]
    async fn start_server<F, H, S>(
        self,
        addr: &str,
        port: u16,
        signal: S,
        handler: H,
        tls_cfg: Option<(&Path, &Path)>,
    ) -> Result<(), ServerError<A, D, T, HyperRequest<A, D, T>, Self>>
    where
        S: Send + Future<Output = ()>,
        H: 'static + Send + Sync + Fn(HyperRequest<A, D, T>) -> F,
        F: Send + Future<Output = Result<Response<ResponseType>, Infallible>>,
    {
        let ip: IpAddr = addr.parse()?;
        let addr = SocketAddr::from((ip, port));

        let mut listener = TcpListener::bind(addr)
            .await
            .map_err(HyperError::Io)
            .map_err(ServerError::Custom)?;
        let acceptor: Pin<
            Box<dyn Stream<Item = Result<HttpStream, io::Error>> + '_ + Send>,
        > = if let Some(tls_cfg) = tls_cfg {
            let tls_cfg = {
                let certs = load_certs(tls_cfg.0).map_err(ServerError::Custom)?;
                let key =
                    load_private_key(tls_cfg.1).map_err(ServerError::Custom)?;
                let mut cfg = ServerConfig::new(rustls::NoClientAuth::new());
                cfg.set_single_cert(certs, key)
                    .map_err(HyperError::Tls)
                    .map_err(ServerError::Custom)?;
                cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
                Arc::new(cfg)
            };
            let tls_acceptor = TlsAcceptor::from(tls_cfg);
            Box::pin(
                listener
                    .incoming()
                    .and_then(move |s| {
                        tls_acceptor
                            .accept(s)
                            .map(|s| {
                                s.and_then(|s| {
                                    HttpStream::new(MaybeHttpsStream::Https(s))
                                })
                            })
                            .map(|s| Ok(s.ok()))
                    })
                    .filter_map(Result::transpose),
            )
        } else {
            Box::pin(
                listener.incoming().map(|s| {
                    s.and_then(|s| HttpStream::new(MaybeHttpsStream::Http(s)))
                }),
            )
        };

        let handler = Arc::new(handler);
        let make_svc = make_service_fn(move |stream: &HttpStream| {
            let remote_addr = stream.addr;
            let inner = self.clone();
            let inner_handler = Arc::<H>::clone(&handler);
            async move {
                Ok::<_, Infallible>(service_fn(move |req: HRequest<Body>| {
                    let inner = inner.clone();
                    let inner_handler = Arc::<H>::clone(&inner_handler);
                    async move {
                        let inner = inner.clone();
                        let inner_handler = Arc::<H>::clone(&inner_handler);
                        Self::handle_req(inner, remote_addr, req, inner_handler)
                            .await
                    }
                }))
            }
        });

        HServer::builder(HyperAcceptor { acceptor })
            .serve(make_svc)
            .with_graceful_shutdown(signal)
            .await
            .map_err(HyperError::Hyper)
            .map_err(ServerError::Custom)
    }
}

struct HyperAcceptor<'a> {
    acceptor: Pin<Box<dyn Stream<Item = Result<HttpStream, io::Error>> + 'a + Send>>,
}

impl Accept for HyperAcceptor<'_> {
    type Conn = HttpStream;
    type Error = io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.acceptor).poll_next(cx)
    }
}

enum MaybeHttpsStream<T> {
    Http(T),
    Https(TlsStream<T>),
}

struct HttpStream {
    stream: MaybeHttpsStream<TcpStream>,
    addr: SocketAddr,
}

impl HttpStream {
    pub(crate) fn new(
        stream: MaybeHttpsStream<TcpStream>,
    ) -> Result<Self, io::Error> {
        let addr = match stream {
            MaybeHttpsStream::Http(ref s) => s.peer_addr()?,
            MaybeHttpsStream::Https(ref s) => {
                let (inner_stream, _) = s.get_ref();
                inner_stream.peer_addr()?
            }
        };
        Ok(Self { stream, addr })
    }
}

impl AsyncRead for HttpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.stream {
            MaybeHttpsStream::Http(ref mut s) => Pin::new(s).poll_read(cx, buf),
            MaybeHttpsStream::Https(ref mut s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for HttpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.stream {
            MaybeHttpsStream::Http(ref mut s) => Pin::new(s).poll_write(cx, buf),
            MaybeHttpsStream::Https(ref mut s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        match self.stream {
            MaybeHttpsStream::Http(ref mut s) => Pin::new(s).poll_flush(cx),
            MaybeHttpsStream::Https(ref mut s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        match self.stream {
            MaybeHttpsStream::Http(ref mut s) => Pin::new(s).poll_shutdown(cx),
            MaybeHttpsStream::Https(ref mut s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

// Load public certificate from file.
fn load_certs(filename: &Path) -> Result<Vec<rustls::Certificate>, HyperError> {
    // Open certificate file.
    let certfile = fs::File::open(filename).map_err(|_| HyperError::Certificate)?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    pemfile::certs(&mut reader).map_err(|_| HyperError::Certificate)
}

// Load private key from file.
#[allow(clippy::unimplemented)]
fn load_private_key(filename: &Path) -> Result<rustls::PrivateKey, HyperError> {
    // Open keyfile.
    let keyfile = fs::File::open(filename).map_err(|_| HyperError::PrivateKey)?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| HyperError::PrivateKey)?;
    if keys.len() != 1 {
        unimplemented!();
    }
    #[allow(clippy::indexing_slicing)]
    Ok(keys[0].clone())
}

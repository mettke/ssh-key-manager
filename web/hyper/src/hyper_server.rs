use crate::hyper_request::HyperRequest;
use core_common::{
    async_trait::async_trait,
    database::{Create, Database, FetchByUid, Save},
    http::response::Response,
    objects::User,
    sec::{Auth, PreAuth},
    web::{BaseData, BaseView, ResponseType, Server, ServerError, TemplateEngine},
};
use hyper::{
    server::{conn::AddrStream, Server as HServer},
    service::{make_service_fn, service_fn},
    Body, Request as HRequest, Response as HResponse,
};
use std::{
    convert::Infallible,
    future::Future,
    marker::PhantomData,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tokio_util::codec::{BytesCodec, FramedRead};

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
        H: Send + Fn(HyperRequest<A, D, T>) -> F,
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
        + FetchByUid<PreAuth, User<'a>, D>
        + Create<PreAuth, User<'a>, D>
        + Save<PreAuth, User<'a>, D>,
    T: 'static + TemplateEngine,
{
    type ServerError = hyper::Error;

    #[inline]
    async fn start_server<F, H, S>(
        self,
        addr: &str,
        port: u16,
        signal: S,
        handler: H,
    ) -> Result<(), ServerError<A, D, T, HyperRequest<A, D, T>, Self>>
    where
        S: Send + Future<Output = ()>,
        H: 'static + Send + Sync + Fn(HyperRequest<A, D, T>) -> F,
        F: Send + Future<Output = Result<Response<ResponseType>, Infallible>>,
    {
        let ip: IpAddr = addr.parse()?;
        let addr = SocketAddr::from((ip, port));
        let handler = Arc::new(handler);

        let make_svc = make_service_fn(move |socket: &AddrStream| {
            let remote_addr = socket.remote_addr();
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
        HServer::bind(&addr)
            .serve(make_svc)
            .with_graceful_shutdown(signal)
            .await
            .map_err(ServerError::Custom)
    }
}

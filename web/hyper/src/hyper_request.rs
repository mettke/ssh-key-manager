use core_common::{
    async_trait::async_trait,
    database::{Create, Database, FetchByUid, Save},
    http::{method::Method, request::Parts, response, uri::Uri, version::Version},
    objects::User,
    sec::{Auth, PreAuth},
    web::{AppError, BaseData, BaseView, Request, TemplateEngine},
};
use hyper::{
    body::to_bytes,
    header::{CONTENT_LENGTH, COOKIE, REFERER, USER_AGENT},
    Body,
};
use std::{net::SocketAddr, sync::Arc};

/// Request from the hyper framework
#[derive(Debug)]
pub struct HyperRequest<
    A: 'static + Auth,
    D: 'static + Database,
    T: 'static + TemplateEngine,
> {
    /// The basic data used on all templates
    pub base_view: Arc<BaseView<'static>>,
    /// The basic data used inside the application
    pub base_data: Arc<BaseData>,
    /// Database Engine
    pub database: Arc<D>,
    /// Template Engine
    pub templates: Arc<T>,
    /// Remote Address associated with the Request
    pub remote_addr: SocketAddr,
    /// Inner Hyper Request Header
    pub header: Parts,
    /// Inner Hyper Request Body
    pub body: Option<Body>,
    /// Authentication provided after calling `Self::authenticate`
    pub auth: Option<A>,
}

#[async_trait]
impl<A, D, T> Request<A, D, T> for HyperRequest<A, D, T>
where
    A: Auth,
    for<'a> D: 'static
        + Database
        + FetchByUid<PreAuth, User<'a>, D>
        + Create<PreAuth, User<'a>, D>
        + Save<PreAuth, User<'a>, D>,
    T: 'static + TemplateEngine,
{
    type RequestError = hyper::error::Error;

    #[inline]
    fn get_template_engine(&self) -> &T {
        &self.templates
    }

    #[inline]
    fn get_base_view(&self) -> &BaseView<'_> {
        &self.base_view
    }

    #[inline]
    fn get_base_data(&self) -> &BaseData {
        &self.base_data
    }

    #[inline]
    async fn authenticate(&mut self, res: &mut response::Builder) -> bool {
        let auth = A::authenticate(self, res).await;
        let res = auth.is_some();
        if let Some(auth) = auth {
            self.auth = Some(auth);
        }
        res
    }

    #[inline]
    fn get_auth(&self) -> &A {
        self.auth
            .as_ref()
            .expect("Self::authenticate must be called first")
    }

    #[inline]
    fn get_database(&self) -> &D {
        &self.database
    }

    #[inline]
    fn get_remote_addr(&self) -> &SocketAddr {
        &self.remote_addr
    }

    #[inline]
    fn get_method(&self) -> &Method {
        &self.header.method
    }

    #[inline]
    fn get_version(&self) -> Version {
        self.header.version
    }

    #[inline]
    fn get_content_length(&self) -> Option<&str> {
        self.header
            .headers
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
    }

    #[inline]
    fn get_referer(&self) -> Option<&str> {
        self.header
            .headers
            .get(REFERER)
            .and_then(|v| v.to_str().ok())
    }

    #[inline]
    fn get_user_agent(&self) -> Option<&str> {
        self.header
            .headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
    }

    #[inline]
    fn get_uri(&self) -> &Uri {
        &self.header.uri
    }

    #[inline]
    fn get_cookie(&self, name: &str) -> Option<&str> {
        #[allow(clippy::filter_map)]
        self.header
            .headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|cookie| cookie.to_str().ok())
            .flat_map(|entry| entry.split(';'))
            .map(str::trim_start)
            .filter(|cookie| cookie.starts_with(name))
            .find_map(|cookie| cookie.splitn(2, '=').nth(1))
    }

    #[inline]
    async fn body_as_bytes(&mut self) -> Result<Vec<u8>, AppError<A, D, T, Self>> {
        let body = self
            .body
            .take()
            .expect("body_as_bytes was called a second time");
        to_bytes(body)
            .await
            .map(|b| b.to_vec())
            .map_err(AppError::RequestError)
    }
}

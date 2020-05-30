use crate::{
    async_trait::async_trait,
    database::Database,
    http::{method::Method, response, uri::Uri, version::Version},
    sec::Auth,
    web::{AppError, BaseData, BaseView, TemplateEngine},
};
use std::{error, fmt::Debug, net::SocketAddr};

/// Required methods for an incoming Request
#[async_trait]
pub trait Request<A: Auth, D: Database, T: TemplateEngine>: Debug + Sized {
    /// the custom error for the request.
    type RequestError: error::Error;

    /// Get the template engine to create the response
    fn get_template_engine(&self) -> &T;

    /// Get the base view for the base template
    fn get_base_view(&self) -> &BaseView<'_>;

    /// Get the base view for the base template
    fn get_base_data(&self) -> &BaseData;

    /// Authenticate the user. Allows storing user auth by
    /// providing mutable access to itself
    async fn authenticate(&mut self, res: &mut response::Builder) -> bool;

    /// Get authentication.
    /// May fail if `authenticate` wasn't called before
    fn get_auth(&self) -> &A;

    /// Get Database Pool
    fn get_database(&self) -> &D;

    /// Returns the `SocketAddr` of the Request
    fn get_remote_addr(&self) -> &SocketAddr;

    /// Returns the `Method` of the Request
    fn get_method(&self) -> &Method;

    /// Returns the `Version` of the Request
    fn get_version(&self) -> Version;

    /// Returns the Content length
    fn get_content_length(&self) -> Option<&str>;

    /// Returns the referer
    fn get_referer(&self) -> Option<&str>;

    /// Returns the user agent
    fn get_user_agent(&self) -> Option<&str>;

    /// Returns the current path as str
    fn get_uri(&self) -> &Uri;

    /// Get the value from the cookie with the given name
    /// if it does exist
    fn get_cookie(&self, name: &str) -> Option<&str>;

    /// Loads the body into memory and returns its bytes
    ///
    /// # Panics
    /// One time usage. Panics when called a second time
    async fn body_as_bytes(&mut self) -> Result<Vec<u8>, AppError<A, D, T, Self>>;
}

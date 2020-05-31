//! This module contains various types the web server objects
mod base;
mod request;
mod response_type;
mod routes;
mod server;
mod template_engine;

pub use self::{
    base::{BaseContainer, BaseData, BaseView, Notification, UserContainer},
    request::Request,
    response_type::ResponseType,
    routes::{
        invalid_method, not_found, redirect, redirect_home, serve_login,
        serve_template, AppError,
    },
    server::{Server, ServerError},
    template_engine::{RenderError, TemplateEngine},
};

use crate::{
    chrono::{naive::NaiveDateTime, DateTime, Utc},
    database::Database,
    sec::Auth,
    url::form_urlencoded,
};
use std::{borrow::Cow, convert::TryFrom, fmt, time::SystemTime};

/// Creates a generic cookie and assings secure when the connection
/// was made using https
///
/// # Arguments
///
/// * `req`: Current request
/// * `name`: Name of the cookie
/// * `content`: Cookie Content
/// * `max_date`: How long the cookie should life (seconds)
/// * `expiration`: Timestamp after which the cookie expires
/// * `path`: Path under which the cookie is valid
#[inline]
pub fn create_cookie<A, D, T, R>(
    req: &R,
    name: &str,
    content: &str,
    max_date: Option<u64>,
    expiration: Option<u64>,
    path: Option<&str>,
) -> String
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let secure = if req.get_uri().scheme_str() == Some("https") {
        "Secure;"
    } else {
        ""
    };
    let path = path.map_or_else(String::new, |v| format!("Path={};", v));
    let max_date = max_date.map_or_else(String::new, |v| format!("Max-Age={};", v));
    let expiration = expiration
        .map(convert_to_cookie_time)
        .map_or_else(String::new, |v| format!("Expires={};", v));
    format!(
        "{}={}; {} {} {} {} HttpOnly; SameSite=Lax;",
        name, content, max_date, expiration, path, secure
    )
}

/// Increases the current time by the given offset in seconds and returns it as timestamp
#[must_use]
#[inline]
pub fn get_current_time_and_add(extend: u64) -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Unable to compute current timestamp")
        .as_secs()
        .saturating_add(extend)
}

fn convert_to_cookie_time(unix: u64) -> String {
    let unix = i64::try_from(unix).unwrap_or_default();
    let time = NaiveDateTime::from_timestamp(unix, 0);
    let dt: DateTime<Utc> = DateTime::from_utc(time, Utc);
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Deletes a cookie with the given name
///
/// # Arguments
///
/// * `req`: Current request
/// * `name`: Name of the cookie
/// * `path`: Path under which the cookie is valid
#[inline]
pub fn delete_cookie<A, D, T, R>(req: &R, name: &str, path: Option<&str>) -> String
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let secure = if req.get_uri().scheme_str() == Some("https") {
        "Secure;"
    } else {
        ""
    };
    let path = path.map_or_else(String::new, |v| format!("Path={};", v));
    format!(
        "{}=; Max-Age=0; {} {} HttpOnly; SameSite=Lax;",
        name, path, secure
    )
}

/// Parameter iterator which deals with internally deals with no parameters
pub struct ParameterIter<'a>(Option<form_urlencoded::Parse<'a>>);

impl<'a> Iterator for ParameterIter<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(Iterator::next)
    }
}

impl fmt::Debug for ParameterIter<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParameterIter")
            .field(
                "inner",
                &self
                    .0
                    .as_ref()
                    .map_or_else(|| String::from("empty"), |_| String::from("[..]")),
            )
            .finish()
    }
}

/// Gets an iterator over the url encoded query parameters
#[inline]
pub fn get_query_parameters<A, D, T, R>(req: &R) -> ParameterIter<'_>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let uri = req.get_uri();
    let param = uri.query().map(str::as_bytes).map(form_urlencoded::parse);
    ParameterIter(param)
}

/// Extracts the path at the given index
#[must_use]
#[inline]
pub fn route_at(path: &[String], i: usize) -> Option<&str> {
    path.get(i).map(|s| s.as_ref())
}

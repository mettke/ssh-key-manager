mod app;
mod auth;

use core_app::rstatic;
use core_common::{
    database::{Create, Database, Delete, FetchAll, FetchById, FetchByUid, Save},
    http::{method::Method, response::Response, status::StatusCode},
    objects::{
        Entity, Group, GroupFilter, GroupMember, PublicKey, PublicKeyFilter, User,
    },
    sec::{Auth, PreAuth},
    types::Id,
    web::{
        invalid_method, not_found, redirect_home, route_at, AppError, Request,
        ResponseType, TemplateEngine,
    },
};
use std::{borrow::Cow, convert::Infallible};
use time::OffsetDateTime;

pub async fn handler<A, D, T, R>(
    mut req: R,
) -> Result<Response<ResponseType>, Infallible>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchByUid<'a, PreAuth, User<'a>, D>
        + FetchByUid<'a, A, User<'a>, D>
        + FetchById<'a, A, PublicKey<'a>, D>
        + FetchById<'a, A, Entity<'a>, D>
        + FetchById<'a, A, Group<'a>, D>
        + Create<'a, PreAuth, User<'a>, D>
        + Create<'a, A, PublicKey<'a>, D>
        + Create<'a, A, Group<'a>, D>
        + Create<'a, A, GroupMember<'a, Cow<'a, Id>>, D>
        + Delete<'a, A, PublicKey<'a>, D>
        + Delete<'a, A, Group<'a>, D>
        + Save<'a, PreAuth, User<'a>, D>
        + FetchAll<'a, 'b, A, PublicKey<'a>, PublicKeyFilter<'b>, D>
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + FetchAll<'a, 'b, A, GroupMember<'a, Entity<'a>>, Id, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let entry_time = OffsetDateTime::now_utc();
    let pre_log_1 = format!(
        "{} - - [{}] \"{} {} {:?}\"",
        req.get_remote_addr().ip(),
        entry_time.format("%Y-%m-%dT%H:%M:%S.%NZ%z"),
        req.get_method(),
        req.get_uri().path(),
        req.get_version(),
    );
    let pre_log_2 = format!(
        "{} \"{}\" \"{}\"",
        req.get_content_length().unwrap_or("-"),
        req.get_referer().unwrap_or("-"),
        req.get_user_agent().unwrap_or("-"),
    );
    let res = index(&mut req).await.unwrap_or_else(|err| {
        log::error!("Unable to create Response: {}", err);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(ResponseType::Empty)
            .expect("Unable to set failure response")
    });
    let response_time = OffsetDateTime::now_utc() - entry_time;
    let response_time_ms = response_time.whole_milliseconds();
    log::info!(
        "{} {} {} ({} ms)",
        pre_log_1,
        res.status().as_u16(),
        pre_log_2,
        response_time_ms,
    );
    log::trace!("Responding with: {:?}", res);
    Ok(res)
}

pub async fn index<A, D, T, R>(
    req: &mut R,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchByUid<'a, PreAuth, User<'a>, D>
        + FetchByUid<'a, A, User<'a>, D>
        + FetchById<'a, A, PublicKey<'a>, D>
        + FetchById<'a, A, Entity<'a>, D>
        + FetchById<'a, A, Group<'a>, D>
        + Create<'a, PreAuth, User<'a>, D>
        + Create<'a, A, PublicKey<'a>, D>
        + Create<'a, A, Group<'a>, D>
        + Create<'a, A, GroupMember<'a, Cow<'a, Id>>, D>
        + Delete<'a, A, PublicKey<'a>, D>
        + Delete<'a, A, Group<'a>, D>
        + Save<'a, PreAuth, User<'a>, D>
        + FetchAll<'a, 'b, A, PublicKey<'a>, PublicKeyFilter<'b>, D>
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + FetchAll<'a, 'b, A, GroupMember<'a, Entity<'a>>, Id, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let path: Vec<String> =
        req.get_uri().path().split('/').map(Into::into).collect();
    match route_at(&path, 1) {
        None | Some("") => index_method(req),
        Some("app") => app::index(req, &path).await,
        Some("auth") => auth::index(req, &path).await,
        Some("static") => rstatic::index(req, &path).await,
        _ => not_found(),
    }
}

pub fn index_method<A, D, T, R>(
    req: &mut R,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    D: Database,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    #[allow(indirect_structural_match)]
    match *req.get_method() {
        Method::GET => {
            let res = Response::builder();
            redirect_home(req, res, true)
        }
        _ => invalid_method(&[Method::GET]),
    }
}

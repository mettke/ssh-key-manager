use core_app::{groups, public_keys};
use core_common::{
    database::{Create, Database, Delete, FetchAll, FetchById, FetchByUid, Save},
    http::response::Response,
    objects::{
        Entity, Group, GroupFilter, GroupMember, PublicKey, PublicKeyFilter, User,
    },
    sec::{Auth, PreAuth},
    types::Id,
    web::{
        not_found, redirect, route_at, serve_login, AppError, Request, ResponseType,
        TemplateEngine,
    },
};
use std::borrow::Cow;

pub async fn index<A, D, T, R>(
    req: &mut R,
    path: &[String],
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
    let mut res = Response::builder();
    if path.last().map(|s| s.as_ref()) != Some("") {
        let path = req.get_uri().path();
        let path = format!("{}/", path);
        let res = Response::builder();
        redirect(req, res, &path, true, true, false)
    } else if req.authenticate(&mut res).await {
        match route_at(path, 2) {
            // Some("") => index_method(req),
            Some("groups") => groups::index(req, res, path).await,
            Some("publickeys") => public_keys::index(req, res, path).await,
            _ => not_found(),
        }
    } else {
        serve_login(req, res)
    }
}

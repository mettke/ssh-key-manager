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

#[allow(single_use_lifetimes)]
pub async fn index<A, D, T, R>(
    req: &mut R,
    path: &[String],
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b, 'c> D: Database
        + FetchByUid<PreAuth, User<'a>, D>
        + FetchByUid<A, User<'a>, D>
        + FetchById<'b, A, PublicKey<'a>, D>
        + FetchById<'b, A, Entity<'a>, D>
        + FetchById<'b, A, Group<'a>, D>
        + Create<PreAuth, User<'a>, D>
        + Create<A, PublicKey<'a>, D>
        + Create<A, Group<'a>, D>
        + Create<A, GroupMember<'a, Cow<'a, Id>>, D>
        + Delete<A, PublicKey<'a>, D>
        + Delete<A, Group<'a>, D>
        + Save<PreAuth, User<'a>, D>
        + FetchAll<'b, A, PublicKey<'a>, PublicKeyFilter<'c>, D>
        + FetchAll<'b, A, Group<'a>, GroupFilter<'c>, D>
        + FetchAll<'b, A, GroupMember<'a, Entity<'a>>, Id, D>,
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

use core_common::{
    database::{
        Create, Database, Delete, DeleteObj, FetchAll, FetchById, FetchByUid,
    },
    http::{
        method::Method,
        response::{self, Response},
    },
    objects::{Entity, Group, GroupFilter, GroupMember, GroupMemberEntry, User},
    sec::{Auth, CsrfToken},
    types::Id,
    url::form_urlencoded,
    web::{
        invalid_method, not_found, route_at, serve_template, AppError,
        BaseContainer, Notification, Request, ResponseType, TemplateEngine,
    },
};
use core_macros::FromForm;
use core_views::{GroupListView, GroupView};
use std::{borrow::Cow, marker::PhantomData};

/// Serves the groups route
///
/// # Errors
/// Fails when the communication with the database fails
#[inline]
pub async fn index<A, D, T, R>(
    req: &mut R,
    res: response::Builder,
    path: &[String],
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + FetchAll<'a, 'b, A, GroupMember<'a, Entity<'a>>, Id, D>
        + FetchByUid<'a, A, User<'a>, D>
        + FetchById<'a, A, Group<'a>, D>
        + Create<'a, A, Group<'a>, D>
        + Create<'a, A, GroupMember<'a, Cow<'a, Id>>, D>
        + Delete<'a, A, Group<'a>, D>
        + DeleteObj<'b, A, GroupMemberEntry<'b>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    #[allow(clippy::wildcard_enum_match_arm)]
    match route_at(path, 3) {
        Some("") => index_method(req, res).await,
        Some(group) => group_method(req, res, group).await,
        _ => not_found(),
    }
}

#[inline]
async fn index_method<A, D, T, R>(
    req: &mut R,
    res: response::Builder,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + Create<'a, A, Group<'a>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    #[allow(indirect_structural_match)]
    match *req.get_method() {
        Method::GET => index_get(req, res, None, CsrfToken::from(req)).await,
        Method::POST => index_post(req, res).await,
        _ => invalid_method(&[Method::GET, Method::POST]),
    }
}

async fn index_get<A, D, T, R>(
    req: &R,
    mut res: response::Builder,
    noti: Option<&[Notification<'_>]>,
    csrf_token: CsrfToken,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let uri = req.get_uri();
    let filter = uri
        .query()
        .map(str::as_bytes)
        .map(form_urlencoded::parse)
        .map_or_else(GroupFilter::default, GroupFilter::from);
    let view = GroupListView::fetch(req, &filter).await?;
    let user = req.get_auth().get_user_container();
    let url = req.get_uri().path();
    let csrf = csrf_token.generate(req, &mut res)?;
    let container = BaseContainer {
        csrf: Some(csrf),
        base: Cow::Borrowed("../"),
        user,
        noti,
        ..BaseContainer::new(req.get_base_view(), &view.0, &filter, url)
    };
    serve_template(req, res, "site_groups", &container)
}

async fn index_post<A, D, T, R>(
    req: &mut R,
    res: response::Builder,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + Create<'a, A, Group<'a>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let bytes = req.body_as_bytes().await?;
    let body = form_urlencoded::parse(&bytes).fold(
        (None, None, None, None),
        |acc, (k, v)| {
            if v.is_empty() {
                return acc;
            }
            match k.as_ref() {
                "name" => (Some(v), acc.1, acc.2, acc.3),
                "oauth_scope" => (acc.0, Some(v), acc.2, acc.3),
                "ldap_group" => (acc.0, acc.1, Some(v), acc.3),
                "csrf" => (acc.0, acc.1, acc.2, Some(v)),
                _ => acc,
            }
        },
    );
    let (body, csrf) = (
        (body.0, body.1, body.2),
        CsrfToken::verify(req, body.3.as_deref()),
    );
    let noti = GroupListView::create(req, body.0, body.1, body.2, &csrf).await?;
    index_get(req, res, Some(&noti), csrf).await
}

#[inline]
async fn group_method<A, D, T, R>(
    req: &mut R,
    res: response::Builder,
    group: &str,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + FetchAll<'a, 'b, A, GroupMember<'a, Entity<'a>>, Id, D>
        + FetchByUid<'a, A, User<'a>, D>
        + FetchById<'a, A, Group<'a>, D>
        + Create<'a, A, Group<'a>, D>
        + Create<'a, A, GroupMember<'a, Cow<'a, Id>>, D>
        + Delete<'a, A, Group<'a>, D>
        + DeleteObj<'b, A, GroupMemberEntry<'b>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    #[allow(indirect_structural_match)]
    match *req.get_method() {
        Method::GET => group_get(req, res, group, None, CsrfToken::from(req)).await,
        Method::POST => group_post(req, res, group).await,
        _ => invalid_method(&[Method::GET, Method::POST]),
    }
}

async fn group_get<A, D, T, R>(
    req: &R,
    mut res: response::Builder,
    group: &str,
    noti: Option<&[Notification<'_>]>,
    csrf_token: CsrfToken,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + FetchAll<'a, 'b, A, GroupMember<'a, Entity<'a>>, Id, D>
        + FetchById<'a, A, Group<'a>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let view = match GroupView::fetch(req, group).await? {
        Some(view) => view,
        None => return not_found(),
    };
    let user = req.get_auth().get_user_container();
    let url = req.get_uri().path();
    let csrf = csrf_token.generate(req, &mut res)?;
    let container = BaseContainer {
        csrf: Some(csrf),
        base: Cow::Borrowed("../../"),
        user,
        noti,
        ..BaseContainer::new(req.get_base_view(), &view, &(), url)
    };
    serve_template(req, res, "site_group", &container)
}

#[derive(FromForm, Default)]
struct GroupData<'a> {
    username: Option<Cow<'a, str>>,
    add_member: Option<Cow<'a, str>>,
    delete_member: Option<Cow<'a, str>>,
    csrf: Option<Cow<'a, str>>,
}

async fn group_post<A, D, T, R>(
    req: &mut R,
    res: response::Builder,
    group: &str,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b> D: Database
        + FetchAll<'a, 'b, A, Group<'a>, GroupFilter<'b>, D>
        + FetchAll<'a, 'b, A, GroupMember<'a, Entity<'a>>, Id, D>
        + FetchByUid<'a, A, User<'a>, D>
        + FetchById<'a, A, Group<'a>, D>
        + Delete<'a, A, Group<'a>, D>
        + DeleteObj<'b, A, GroupMemberEntry<'b>, D>
        + Create<'a, A, GroupMember<'a, Cow<'a, Id>>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let id = match Id::from_string(group) {
        Err(_) => {
            return not_found();
        }
        Ok(id) => id,
    };
    let bytes = req.body_as_bytes().await?;
    let data = GroupData::from(form_urlencoded::parse(&bytes));

    let auth = req.get_auth();
    let db = req.get_database();
    let csrf = CsrfToken::verify(req, data.csrf.as_deref());
    let noti = if data.add_member.as_ref().map(|v| v.as_ref()) == Some("1") {
        let username = if let Some(uid) = data.username {
            db.fetch_by_uid(&uid, auth, PhantomData).await?
        } else {
            None
        };
        GroupView::add_member_user(req, id, username.as_ref(), &csrf).await?
    } else if let Some(uid) = data.delete_member {
        GroupView::del_member_user(req, id, &uid, &csrf).await?
    } else {
        [Notification::Error {
            name: "Group",
            para: "operation",
            help: "../../help/#group_err",
        }]
    };
    group_get(req, res, group, Some(&noti), csrf).await
}

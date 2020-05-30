use core_common::{
    database::{Create, Database, FetchAll, FetchByUid},
    http::{
        method::Method,
        response::{self, Response},
    },
    objects::{PublicKey, PublicKeyFilter, User},
    sec::{Auth, CsrfToken},
    url::form_urlencoded,
    web::{
        invalid_method, serve_template, AppError, BaseContainer, Notification,
        Request, ResponseType, TemplateEngine,
    },
};
use core_views::PublicKeyListView;
use std::borrow::Cow;

/// Serves the public keys route
///
/// # Errors
/// Fails when the communication with the database fails
#[inline]
#[allow(single_use_lifetimes)]
pub async fn index<A, D, T, R>(
    req: &mut R,
    res: response::Builder,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b, 'c> D: Database
        + FetchAll<'b, A, PublicKey<'a>, PublicKeyFilter<'c>, D>
        + FetchByUid<A, User<'a>, D>
        + Create<A, PublicKey<'a>, D>,
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

#[allow(single_use_lifetimes)]
async fn index_get<A, D, T, R>(
    req: &R,
    mut res: response::Builder,
    noti: Option<&[Notification<'_>]>,
    csrf_token: CsrfToken,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b, 'c> D:
        Database + FetchAll<'b, A, PublicKey<'a>, PublicKeyFilter<'c>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let uri = req.get_uri();
    let filter = uri
        .query()
        .map(str::as_bytes)
        .map(form_urlencoded::parse)
        .map_or_else(PublicKeyFilter::default, PublicKeyFilter::from);
    let view = PublicKeyListView::fetch(req, &filter).await?;
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
    serve_template(req, res, "site_publickeys", &container)
}

#[allow(single_use_lifetimes)]
async fn index_post<A, D, T, R>(
    req: &mut R,
    res: response::Builder,
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a, 'b, 'c> D: Database
        + FetchAll<'b, A, PublicKey<'a>, PublicKeyFilter<'c>, D>
        + FetchByUid<A, User<'a>, D>
        + Create<A, PublicKey<'a>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    let bytes = req.body_as_bytes().await?;
    let body =
        form_urlencoded::parse(&bytes).fold((None, None, None), |acc, (k, v)| {
            if v.is_empty() {
                return acc;
            }
            match k.as_ref() {
                "data" => (Some(v), acc.1, acc.2),
                "uid" => (acc.0, Some(v), acc.2),
                "csrf" => (acc.0, acc.1, Some(v)),
                _ => acc,
            }
        });
    let (body, csrf) = ((body.0, body.1), CsrfToken::verify(req, body.2.as_deref()));
    let db = req.get_database();
    let auth = req.get_auth();
    let body = if let (data, Some(uid)) = body {
        let user = db.fetch_by_uid(&uid, auth)?;
        (data, user)
    } else {
        (body.0, None)
    };
    let noti =
        PublicKeyListView::create(req, body.0, body.1.as_ref(), &csrf).await?;
    index_get(req, res, Some(&noti), csrf).await
}

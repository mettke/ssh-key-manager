use crate::routes::{index_method, not_found};
use core_app::auth::{logout, oauth};
use core_common::{
    database::{Create, Database, FetchByUid, Save},
    http::response::Response,
    objects::User,
    sec::{Auth, AuthMethod, PreAuth},
    web::{route_at, AppError, Request, ResponseType, TemplateEngine},
};

pub async fn index<A, D, T, R>(
    req: &mut R,
    path: &[String],
) -> Result<Response<ResponseType>, AppError<A, D, T, R>>
where
    A: Auth,
    for<'a> D: Database
        + FetchByUid<'a, PreAuth, User<'a>, D>
        + Create<'a, PreAuth, User<'a>, D>
        + Save<'a, PreAuth, User<'a>, D>,
    T: TemplateEngine,
    R: Request<A, D, T>,
{
    match route_at(path, 2) {
        None | Some("") => index_method(req),
        Some("logout") => logout::index(req).await,
        Some("callback") if A::is_supported(AuthMethod::OAuth) => {
            oauth::callback::index(req).await
        }
        Some("oauth2") if A::is_supported(AuthMethod::OAuth) => {
            oauth::index(req).await
        }
        _ => not_found(),
    }
}

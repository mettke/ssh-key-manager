use super::err::TestErr;
use crate::{
    async_trait::async_trait,
    database::{Create, Database, FetchByUid, Save},
    http::response,
    objects::User,
    sec::{Auth, AuthMethod, PreAuth},
    types::{Id, UserTypes},
    web::{AppError, Request, TemplateEngine, UserContainer},
};

#[derive(Debug, Clone)]
pub struct AdminAuth {
    pub(crate) id: Id,
}

#[async_trait]
impl Auth for AdminAuth {
    type AuthError = TestErr;

    #[inline]
    async fn authenticate<D, T, R>(_: &R, _: &mut response::Builder) -> Option<Self>
    where
        for<'a> D: Database
            + FetchByUid<'a, PreAuth, User<'a>, D>
            + Create<'a, PreAuth, User<'a>, D>
            + Save<'a, PreAuth, User<'a>, D>,
        T: TemplateEngine,
        R: Request<Self, D, T> + Sync,
    {
        panic!("Should never be called")
    }

    #[inline]
    async fn create<D, T, R>(
        _req: &R,
        _username: String,
        _name: &str,
        _email: &str,
        _exp: Option<u64>,
        _type_: UserTypes,
    ) -> Result<Option<Self>, AppError<Self, D, T, R>>
    where
        D: Database,
        T: TemplateEngine,
        R: Request<Self, D, T>,
    {
        panic!("Should never be called")
    }

    #[inline]
    fn is_admin(&self) -> bool {
        true
    }

    #[inline]
    fn get_uid(&self) -> &str {
        panic!("Should never be called")
    }

    #[inline]
    fn get_id(&self) -> &Id {
        &self.id
    }

    #[inline]
    fn get_user_container(&self) -> UserContainer<'_> {
        panic!("Should never be called")
    }

    #[inline]
    fn is_supported(_: AuthMethod) -> bool {
        panic!("Should never be called")
    }

    #[inline]
    fn get_str<D, T, R>(
        &self,
        _: &R,
    ) -> Result<Option<String>, AppError<Self, D, T, R>>
    where
        D: Database,
        T: TemplateEngine,
        R: Request<Self, D, T>,
    {
        panic!("Should never be called")
    }
}

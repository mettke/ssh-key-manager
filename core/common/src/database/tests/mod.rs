mod auth {
    pub(crate) mod admin;
    pub(crate) mod err;
}
mod public_key;

use self::auth::admin::AdminAuth;
use crate::{
    database::{Create, Database},
    objects::PublicKey,
};
use std::{future::Future, pin::Pin, sync::Arc};

type AsyncFn<D> = Box<dyn Fn(Arc<D>) -> Pin<Box<dyn Future<Output = ()>>>>;

pub fn get_tests<D>() -> Vec<(&'static str, AsyncFn<D>)>
where
    for<'a> D: 'static + Database + Create<'a, AdminAuth, PublicKey<'a>, D>,
{
    vec![("pk_create_admin", b(public_key::create_with_admin))]
}

fn b<D, F, Fut>(f: F) -> AsyncFn<D>
where
    for<'a> D: 'static + Database + Create<'a, AdminAuth, PublicKey<'a>, D>,
    F: 'static + Fn(Arc<D>) -> Fut,
    Fut: 'static + Future<Output = ()>,
{
    Box::new(move |db| Box::pin(f(db)))
}

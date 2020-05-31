use crate::{
    database::coalesce4,
    error::DieselError,
    exec_opt,
    migrate::Migrate,
    schema::{entity, groups, server, server_account, users},
    BinaryWrapper, DbWrapper, DieselDB,
};
use core_common::{
    database::{DatabaseError, DbResult, FetchById},
    objects::Entity,
    sec::Auth,
    types::{EntityTypes, Id},
};
use diesel::{
    backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax},
    connection::AnsiTransactionManager,
    expression::nullable::Nullable,
    sql_types::HasSqlType,
    Connection, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, QueryDsl, Queryable, RunQueryDsl,
};
use std::borrow::Cow;

#[derive(Debug, Clone, Queryable)]
struct InnerEntity<'a> {
    entity_id: BinaryWrapper<Cow<'a, Id>>,
    name: Option<Cow<'a, str>>,
    server_id: Option<BinaryWrapper<Cow<'a, Id>>>,
    server_name: Option<Cow<'a, str>>,
    type_: DbWrapper<EntityTypes>,
}

type SelectType = (
    entity::id,
    coalesce4::coalesce4<
        Nullable<server_account::name>,
        Nullable<groups::name>,
        Nullable<users::name>,
        Nullable<users::uid>,
    >,
    Nullable<server_account::server_id>,
    Nullable<server::hostname>,
    entity::type_,
);

impl InnerEntity<'_> {
    fn keys() -> SelectType {
        (
            entity::id,
            coalesce4(
                server_account::name.nullable(),
                groups::name.nullable(),
                users::name.nullable(),
                users::uid.nullable(),
            ),
            server_account::server_id.nullable(),
            server::hostname.nullable(),
            entity::type_,
        )
    }
}

impl<'a> Into<Entity<'a>> for InnerEntity<'a> {
    fn into(self) -> Entity<'a> {
        Entity {
            entity_id: self.entity_id.0,
            name: self.name,
            server_id: self.server_id.map(|v| v.0),
            server_name: self.server_name,
            type_: Some(self.type_.0),
        }
    }
}

#[allow(clippy::type_repetition_in_bounds)]
impl<'a, B, C, A> FetchById<'_, A, Entity<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + SupportsDefaultKeyword
        + HasSqlType<DbWrapper<EntityTypes>>,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    DbWrapper<EntityTypes>: Queryable<DbWrapper<EntityTypes>, B>,
{
    #[inline]
    fn fetch(&self, id: &Id, _auth: &A) -> DbResult<Option<Entity<'a>>, Self> {
        let res: Option<InnerEntity<'a>>;
        let conn = self.get()?;

        let query = entity::dsl::entity
            .left_join(users::dsl::users.on(users::entity_id.eq(entity::id)))
            .left_join(groups::dsl::groups.on(groups::entity_id.eq(entity::id)))
            .left_join(
                server_account::dsl::server_account
                    .on(server_account::entity_id.eq(entity::id)),
            )
            .left_join(
                server::dsl::server.on(server::id.eq(server_account::server_id)),
            )
            .select(InnerEntity::keys())
            .filter(entity::id.eq(BinaryWrapper(id)));
        res = exec_opt!(query, conn, first)?;
        Ok(res.map(|v| v.into()))
    }
}

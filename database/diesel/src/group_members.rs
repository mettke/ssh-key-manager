use crate::{
    database::{coalesce4, UniqueExtension},
    error::DieselError,
    exec, exec_unique,
    migrate::Migrate,
    schema::{entity, group_member, groups, server, server_account, users},
    BinaryWrapper, DbWrapper, DieselDB,
};
use core_common::{
    async_trait::async_trait,
    chrono::NaiveDateTime,
    database::{Create, DatabaseError, DbList, DbResult, DeleteObj, FetchAll},
    objects::{Entity, GroupMember, GroupMemberEntry},
    sec::Auth,
    tokio::task,
    types::{EntityTypes, Id},
};
use diesel::{
    backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax},
    connection::AnsiTransactionManager,
    deserialize::FromSql,
    expression::nullable::Nullable,
    insert_into,
    serialize::ToSql,
    sql_types::{Bool, HasSqlType, Timestamp},
    Connection, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
    Queryable, RunQueryDsl,
};
use std::{borrow::Cow, marker::PhantomData};

#[derive(Debug, Clone, Queryable)]
struct InnerGroupMember<'a> {
    group_id: BinaryWrapper<Cow<'a, Id>>,
    entity_id: BinaryWrapper<Cow<'a, Id>>,
    name: Option<Cow<'a, str>>,
    server_id: Option<BinaryWrapper<Cow<'a, Id>>>,
    server_name: Option<Cow<'a, str>>,
    type_: DbWrapper<EntityTypes>,
    add_date: NaiveDateTime,
}

type SelectType = (
    group_member::group_id,
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
    group_member::add_date,
);

impl InnerGroupMember<'_> {
    fn keys() -> SelectType {
        (
            group_member::group_id,
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
            group_member::add_date,
        )
    }
}

impl<'a> Into<GroupMember<'a, Entity<'a>>> for InnerGroupMember<'a> {
    fn into(self) -> GroupMember<'a, Entity<'a>> {
        GroupMember {
            group_id: self.group_id.0,
            member: Entity {
                entity_id: self.entity_id.0,
                name: self.name,
                server_id: self.server_id.map(|v| v.0),
                server_name: self.server_name,
                type_: Some(self.type_.0),
            },
            add_date: self.add_date,
        }
    }
}

#[allow(clippy::type_repetition_in_bounds, unused_lifetimes)]
#[async_trait]
impl<'a, 'b, B, C, A> FetchAll<'a, 'b, A, GroupMember<'a, Entity<'a>>, Id, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<EntityTypes>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B> + FromSql<Bool, B>,
    DbWrapper<EntityTypes>: Queryable<DbWrapper<EntityTypes>, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
{
    #[inline]
    async fn fetch_all(
        &self,
        filter: &Id,
        _auth: &A,
        _page: usize,
        _: PhantomData<(&'a (), &'b ())>,
    ) -> DbResult<DbList<GroupMember<'a, Entity<'a>>>, Self> {
        let res: Vec<InnerGroupMember<'a>>;

        let query = group_member::dsl::group_member
            .inner_join(
                entity::dsl::entity.on(group_member::member_id.eq(entity::id)),
            )
            .left_join(users::dsl::users.on(users::entity_id.eq(entity::id)))
            .left_join(groups::dsl::groups.on(groups::entity_id.eq(entity::id)))
            .left_join(
                server_account::dsl::server_account
                    .on(server_account::entity_id.eq(entity::id)),
            )
            .left_join(
                server::dsl::server.on(server::id.eq(server_account::server_id)),
            )
            .select(InnerGroupMember::keys())
            .filter(group_member::group_id.eq(BinaryWrapper(filter)));

        res = task::block_in_place(|| {
            let conn = self.get()?;
            exec!(query, conn, load)
        })?;

        let count = res.len();
        Ok(DbList {
            data: res.into_iter().map(|v| v.into()).collect(),
            count,
            page: 1,
            page_max: 1,
        })
    }
}

#[async_trait]
#[allow(unused_lifetimes)]
impl<'a, A, B, C: 'static + Connection>
    Create<'a, A, GroupMember<'a, Cow<'a, Id>>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + SupportsDefaultKeyword
        + UsesAnsiSavepointSyntax,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    NaiveDateTime: ToSql<Timestamp, B>,
{
    #[inline]
    async fn create(
        &self,
        object: &GroupMember<'a, Cow<'a, Id>>,
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), Self> {
        let query = insert_into(group_member::dsl::group_member).values((
            group_member::group_id.eq(BinaryWrapper(&object.group_id)),
            group_member::member_id.eq(BinaryWrapper(&object.member)),
            group_member::add_date.eq(&object.add_date),
            group_member::added_by.eq(BinaryWrapper(auth.get_id())),
        ));
        task::block_in_place(|| {
            let conn = self.get()?;
            exec_unique!(query, conn, execute).map(|_| ())
        })
        // if let DbResult::Ok(_) = res {
        //     let details = Cow::Owned(
        //         json!({
        //             "action": "Pubkey add",
        //             "value": object.fingerprint_md5,
        //             "id": &object.id
        //         })
        //         .to_string(),
        //     );
        //     let event: Event<'_> = Event {
        //         id: Cow::Owned(self.generate_id()?),
        //         actor_id: Some(Cow::Borrowed(auth.get_id())),
        //         date: None,
        //         details,
        //         type_: EventTypes::Entity,
        //         object_id: Some(Cow::Borrowed(&object.entity_id)),
        //     };
        //     self.create(&event, auth)?;
        // }
    }
}

#[async_trait]
#[allow(unused_lifetimes)]
impl<'a, A, B, C> DeleteObj<'a, A, GroupMemberEntry<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + Backend
        + UsesAnsiSavepointSyntax
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
{
    #[inline]
    async fn delete_obj(
        &self,
        object: &GroupMemberEntry<'a>,
        _auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), Self> {
        let query = diesel::delete(group_member::dsl::group_member)
            .filter(group_member::group_id.eq(BinaryWrapper(&object.group_id)))
            .filter(group_member::member_id.eq(BinaryWrapper(&object.member)));
        let _ = task::block_in_place(|| {
            let conn = self.get()?;
            exec!(query, conn, execute)
        })?;
        Ok(())
    }
}

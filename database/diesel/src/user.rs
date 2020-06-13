use crate::{
    error::DieselError,
    exec, exec_opt, exec_unique,
    migrate::Migrate,
    schema::{entity, users},
    BinaryWrapper, DbWrapper, DieselDB, UniqueExtension,
};
use core_common::{
    async_trait::async_trait,
    database::{
        Create, DatabaseError, DbList, DbResult, Delete, FetchAll, FetchById,
        FetchByUid, Save,
    },
    objects::{User, UserFilter},
    sec::Auth,
    tokio::task,
    types::{EntityTypes, Id, UserTypes},
};
use diesel::{
    backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax},
    connection::AnsiTransactionManager,
    deserialize::FromSql,
    dsl::count_star,
    insert_into,
    query_builder::BoxedSelectStatement,
    serialize::ToSql,
    sql_types::{Bool, HasSqlType},
    update, Connection, ExpressionMethods, OptionalExtension, QueryDsl, Queryable,
    RunQueryDsl, TextExpressionMethods,
};
use std::{borrow::Cow, marker::PhantomData};

#[derive(Debug, Clone, Queryable)]
struct InnerUser<'a> {
    entity_id: BinaryWrapper<Cow<'a, Id>>,
    uid: Cow<'a, str>,
    name: Option<Cow<'a, str>>,
    email: Option<Cow<'a, str>>,
    password: Option<Cow<'a, str>>,
    type_: DbWrapper<UserTypes>,
}

type SelectType = (
    users::entity_id,
    users::uid,
    users::name,
    users::email,
    users::password,
    users::type_,
);

impl InnerUser<'_> {
    const fn keys() -> SelectType {
        (
            users::entity_id,
            users::uid,
            users::name,
            users::email,
            users::password,
            users::type_,
        )
    }

    fn filter<'a, B, T>(
        mut query: BoxedSelectStatement<'a, T, users::table, B>,
        filter: &'a UserFilter<'_>,
        types: Option<&'a [DbWrapper<UserTypes>]>,
    ) -> BoxedSelectStatement<'a, T, users::table, B>
    where
        B: 'a
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + HasSqlType<Bool>
            + HasSqlType<DbWrapper<UserTypes>>,
        bool: ToSql<Bool, B>,
    {
        if let Some(ref v) = filter.uid {
            query = query.filter(users::uid.like(v));
        }
        if let Some(ref v) = filter.name {
            query = query.filter(users::name.like(v));
        }
        if let Some(ref v) = filter.email {
            query = query.filter(users::email.like(v));
        }
        if let Some(v) = types {
            query = query.filter(users::type_.eq_any(v));
        }

        query
    }
}

impl<'a> Into<User<'a>> for InnerUser<'a> {
    fn into(self) -> User<'a> {
        User {
            entity_id: self.entity_id.0,
            uid: self.uid,
            name: self.name,
            email: self.email,
            password: self.password,
            type_: self.type_.0,
        }
    }
}

#[allow(clippy::type_repetition_in_bounds)]
#[async_trait]
impl<'a, B, C, A> FetchById<'a, A, User<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<UserTypes>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B> + FromSql<Bool, B>,
    DbWrapper<UserTypes>: Queryable<DbWrapper<UserTypes>, B>,
{
    #[inline]
    async fn fetch(
        &self,
        id: &Id,
        _auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<Option<User<'a>>, Self> {
        let res: Option<InnerUser<'a>>;

        let query = users::dsl::users
            .select(InnerUser::keys())
            .find(BinaryWrapper(id));
        res = task::block_in_place(|| {
            let conn = self.get()?;
            exec_opt!(query, conn, first)
        })?;
        Ok(res.map(|v| v.into()))
    }
}

#[allow(clippy::type_repetition_in_bounds)]
#[async_trait]
impl<'a, B, C, A> FetchByUid<'a, A, User<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<UserTypes>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B> + FromSql<Bool, B>,
    DbWrapper<UserTypes>: Queryable<DbWrapper<UserTypes>, B>,
{
    #[inline]
    async fn fetch_by_uid(
        &self,
        uid: &str,
        _auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<Option<User<'a>>, Self> {
        let res: Option<InnerUser<'a>>;

        let query = users::dsl::users
            .select(InnerUser::keys())
            .filter(users::uid.eq(uid));
        res = task::block_in_place(|| {
            let conn = self.get()?;
            exec_opt!(query, conn, first)
        })?;
        Ok(res.map(|v| v.into()))
    }
}

#[async_trait]
#[allow(clippy::type_repetition_in_bounds, unused_lifetimes)]
impl<'a, 'b, B, C, A> FetchAll<'a, 'b, A, User<'a>, UserFilter<'b>, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<UserTypes>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B> + FromSql<Bool, B>,
    DbWrapper<UserTypes>: Queryable<DbWrapper<UserTypes>, B>,
{
    #[inline]
    async fn fetch_all(
        &self,
        filter: &UserFilter<'b>,
        auth: &A,
        page: usize,
        _: PhantomData<(&'a (), &'b ())>,
    ) -> DbResult<DbList<User<'a>>, Self> {
        let offset = Self::compute_offset(page);
        let types: Option<Vec<DbWrapper<UserTypes>>> = if auth.is_admin() {
            filter
                .type_
                .as_ref()
                .map(|v| v.iter().copied().map(DbWrapper).collect())
        } else {
            filter.type_.as_ref().map(|v| {
                v.iter()
                    .copied()
                    .filter_map(|v| {
                        if v == UserTypes::Superuser {
                            None
                        } else {
                            Some(DbWrapper(v))
                        }
                    })
                    .collect()
            })
        };
        let types_ref = types.as_ref().map(|v| &v[..]);

        let count_query = users::dsl::users.select(count_star()).into_boxed::<B>();
        let count_query = InnerUser::filter(count_query, filter, types_ref);

        let query = users::dsl::users
            .select(InnerUser::keys())
            .limit(25)
            .offset(offset)
            .into_boxed::<B>();
        let query = InnerUser::filter(query, filter, types_ref);

        let (count, res) = task::block_in_place(|| {
            let conn = self.get()?;
            let count = exec!(count_query, conn, first)?;
            let res: Vec<InnerUser<'a>> = exec!(query, conn, load)?;
            Ok((count, res))
        })?;

        let count = Self::compute_count(count);
        let page_max = Self::compute_page_max(count);

        Ok(DbList {
            data: res.into_iter().map(|v| v.into()).collect(),
            count,
            page,
            page_max,
        })
    }
}

#[async_trait]
#[allow(unused_lifetimes)]
impl<'a, A, B, C: 'static + Connection> Create<'a, A, User<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + SupportsDefaultKeyword
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<EntityTypes>>
        + HasSqlType<DbWrapper<UserTypes>>,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    DbWrapper<UserTypes>: Queryable<DbWrapper<UserTypes>, B>,
{
    #[inline]
    async fn create(
        &self,
        object: &User<'a>,
        _auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), Self> {
        let entity_query = insert_into(entity::dsl::entity).values((
            entity::id.eq(BinaryWrapper(&object.entity_id)),
            entity::type_.eq(DbWrapper(EntityTypes::User)),
        ));
        let users_query = insert_into(users::dsl::users).values((
            users::entity_id.eq(BinaryWrapper(&object.entity_id)),
            users::uid.eq(&object.uid),
            users::name.eq(&object.name),
            users::email.eq(&object.email),
            users::password.eq(&object.password),
            users::type_.eq(DbWrapper(object.type_)),
        ));

        task::block_in_place(|| {
            let conn = self.get()?;
            let _ = exec_unique!(entity_query, conn, execute)?;
            exec_unique!(users_query, conn, execute).map(|_| ())
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
impl<'a, A, B, C: 'static + Connection> Save<'a, A, User<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + SupportsDefaultKeyword
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<UserTypes>>,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    DbWrapper<UserTypes>: Queryable<DbWrapper<UserTypes>, B>,
{
    #[inline]
    async fn save(
        &self,
        object: &User<'a>,
        _auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), Self> {
        let query = update(users::dsl::users.find(BinaryWrapper(&object.entity_id)))
            .set((
                users::entity_id.eq(BinaryWrapper(&object.entity_id)),
                users::uid.eq(&object.uid),
                users::name.eq(&object.name),
                users::email.eq(&object.email),
                users::password.eq(&object.password),
                users::type_.eq(DbWrapper(object.type_)),
            ));
        task::block_in_place(|| {
            let conn = self.get()?;
            exec_unique!(query, conn, execute).map(|_| ())
        })
    }
}

#[async_trait]
#[allow(unused_lifetimes)]
impl<'a, A, B, C> Delete<'a, A, User<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
{
    #[inline]
    async fn delete(
        &self,
        ids: &[Id],
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), Self> {
        if auth.is_admin() {
            let ids: Vec<BinaryWrapper<&Id>> =
                ids.iter().map(BinaryWrapper).collect();
            let query = diesel::delete(users::dsl::users)
                .filter(users::entity_id.eq_any(&ids))
                .into_boxed::<B>();
            let _ = task::block_in_place(|| {
                let conn = self.get()?;
                exec!(query, conn, execute)
            })?;
        }
        Ok(())
    }
}

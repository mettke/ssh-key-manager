use crate::{
    error::DieselError, exec, exec_opt, exec_unique, migrate::Migrate,
    schema::event, BinaryWrapper, DbWrapper, DieselDB, UniqueExtension,
};
use core_common::{
    async_trait::async_trait,
    chrono::NaiveDateTime,
    database::{
        Create, DatabaseError, DbList, DbResult, FetchAll, FetchById, FetchFirst,
    },
    objects::{Event, EventFilter},
    sec::Auth,
    tokio::task,
    types::{EventTypes, Id},
};
use diesel::{
    backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax},
    connection::AnsiTransactionManager,
    deserialize::FromSql,
    dsl::count_star,
    expression::nullable::Nullable,
    insert_into,
    query_builder::BoxedSelectStatement,
    serialize::ToSql,
    sql_types::{Bool, HasSqlType, Timestamp},
    Connection, ExpressionMethods, NullableExpressionMethods, OptionalExtension,
    QueryDsl, Queryable, RunQueryDsl, TextExpressionMethods,
};
use std::{borrow::Cow, marker::PhantomData};

#[derive(Debug, Clone, Queryable)]
struct InnerEvent<'a> {
    id: BinaryWrapper<Cow<'a, Id>>,
    actor_id: Option<BinaryWrapper<Cow<'a, Id>>>,
    date: Option<NaiveDateTime>,
    details: Cow<'a, str>,
    type_: DbWrapper<EventTypes>,
    object_id: Option<BinaryWrapper<Cow<'a, Id>>>,
}

type SelectType = (
    event::id,
    event::actor_id,
    Nullable<event::date>,
    event::details,
    event::type_,
    event::object_id,
);

impl InnerEvent<'_> {
    fn keys() -> SelectType {
        (
            event::id,
            event::actor_id,
            event::date.nullable(),
            event::details,
            event::type_,
            event::object_id,
        )
    }

    fn filter<'a, B, T>(
        mut query: BoxedSelectStatement<'a, T, event::table, B>,
        filter: &'a EventFilter<'_>,
    ) -> BoxedSelectStatement<'a, T, event::table, B>
    where
        B: 'a
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + HasSqlType<Bool>,
        bool: ToSql<Bool, B>,
    {
        if let Some(ref v) = filter.actor_id {
            query = query.filter(event::actor_id.eq(BinaryWrapper(v)));
        }
        if let Some(ref details) = filter.details {
            query = query.filter(event::details.like(details));
        }
        if let Some(ref v) = filter.object_id {
            query = query.filter(event::object_id.eq(BinaryWrapper(v)));
        }
        query
    }
}

impl<'a> Into<Event<'a>> for InnerEvent<'a> {
    fn into(self) -> Event<'a> {
        Event {
            id: self.id.0,
            actor_id: self.actor_id.map(|v| v.0),
            date: self.date,
            details: self.details,
            type_: self.type_.0,
            object_id: self.object_id.map(|v| v.0),
        }
    }
}

#[async_trait]
impl<'a, B, C, A> FetchById<'a, A, Event<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<EventTypes>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
    DbWrapper<EventTypes>: Queryable<DbWrapper<EventTypes>, B>,
{
    #[inline]
    async fn fetch(
        &self,
        id: &Id,
        auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<Option<Event<'a>>, Self> {
        if auth.is_admin() {
            let query = event::dsl::event
                .select(InnerEvent::keys())
                .find(BinaryWrapper(id));

            let res: Option<InnerEvent<'_>> = task::block_in_place(|| {
                let conn = self.get()?;
                exec_opt!(query, conn, first)
            })?;
            Ok(res.map(|v| v.into()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
#[allow(unused_lifetimes)]
impl<'a, 'b, B, C, A> FetchFirst<'a, 'b, A, Event<'a>, EventFilter<'b>, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<EventTypes>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
    DbWrapper<EventTypes>: Queryable<DbWrapper<EventTypes>, B>,
{
    #[inline]
    async fn fetch_first(
        &self,
        filter: &EventFilter<'b>,
        auth: &A,
        _: PhantomData<(&'a (), &'b ())>,
    ) -> DbResult<Option<Event<'a>>, Self> {
        if auth.is_admin() {
            let query = event::dsl::event
                .select(InnerEvent::keys())
                .order_by(event::date.desc())
                .into_boxed::<B>();
            let query = InnerEvent::filter(query, filter);
            let res: Option<InnerEvent<'_>> = task::block_in_place(|| {
                let conn = self.get()?;
                exec_opt!(query, conn, first)
            })?;
            Ok(res.map(|v| v.into()))
        } else {
            Ok(None)
        }
    }
}

#[allow(unused_lifetimes)]
#[async_trait]
impl<'a, 'b, B, C, A> FetchAll<'a, 'b, A, Event<'a>, EventFilter<'b>, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<EventTypes>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
    DbWrapper<EventTypes>: Queryable<DbWrapper<EventTypes>, B>,
{
    #[inline]
    async fn fetch_all(
        &self,
        filter: &EventFilter<'b>,
        auth: &A,
        page: usize,
        _: PhantomData<(&'a (), &'b ())>,
    ) -> DbResult<DbList<Event<'a>>, Self> {
        if auth.is_admin() {
            let offset = Self::compute_offset(page);
            let count_query =
                event::dsl::event.select(count_star()).into_boxed::<B>();
            let count_query = InnerEvent::filter(count_query, filter);

            let query = event::dsl::event
                .select(InnerEvent::keys())
                .limit(25)
                .offset(offset)
                .order_by(event::date.desc())
                .into_boxed::<B>();
            let query = InnerEvent::filter(query, filter);

            let (count, res) = task::block_in_place(|| {
                let conn = self.get()?;
                let count = exec!(count_query, conn, first)?;
                let res: Vec<InnerEvent<'_>> = exec!(query, conn, load)?;
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
        } else {
            Ok(DbList {
                data: vec![],
                count: 0,
                page: 0,
                page_max: 0,
            })
        }
    }
}

#[async_trait]
#[allow(unused_lifetimes)]
impl<'a, A, B, C: 'static + Connection> Create<'a, A, Event<'a>, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + SupportsDefaultKeyword
        + UsesAnsiSavepointSyntax
        + HasSqlType<DbWrapper<EventTypes>>,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
{
    #[inline]
    async fn create(
        &self,
        object: &Event<'a>,
        _auth: &A,
        _: PhantomData<&'a ()>,
    ) -> DbResult<(), Self> {
        let query = insert_into(event::dsl::event).values((
            event::id.eq(BinaryWrapper(&object.id)),
            event::actor_id.eq(object.actor_id.as_ref().map(BinaryWrapper)),
            event::details.eq(&object.details),
            event::type_.eq(DbWrapper(object.type_)),
            event::object_id.eq(object.object_id.as_ref().map(BinaryWrapper)),
        ));
        task::block_in_place(|| {
            let conn = self.get()?;
            exec_unique!(query, conn, execute).map(|_| ())
        })
    }
}

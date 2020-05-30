use crate::{
    error::DieselError, exec, exec_opt, exec_unique, migrate::Migrate,
    schema::event, BinaryWrapper, DbWrapper, DieselDB, UniqueExtension,
};
use core_common::{
    chrono::NaiveDateTime,
    database::{
        Create, DatabaseError, DbList, DbResult, FetchAll, FetchById, FetchFirst,
    },
    objects::{Event, EventFilter},
    sec::Auth,
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
use std::borrow::Cow;

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

impl<'a, B, C, A> FetchById<'_, A, Event<'a>, Self> for DieselDB<C>
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
    fn fetch(&self, id: &Id, auth: &A) -> DbResult<Option<Event<'a>>, Self> {
        if auth.is_admin() {
            let conn = self.get()?;
            let query = event::dsl::event
                .select(InnerEvent::keys())
                .find(BinaryWrapper(id));
            let res: Option<InnerEvent<'_>> = exec_opt!(query, conn, first)?;
            Ok(res.map(|v| v.into()))
        } else {
            Ok(None)
        }
    }
}

impl<'a, B, C, A> FetchFirst<A, Event<'a>, EventFilter<'_>, Self> for DieselDB<C>
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
    fn fetch_first(
        &self,
        filter: &EventFilter<'_>,
        auth: &A,
    ) -> DbResult<Option<Event<'a>>, Self> {
        if auth.is_admin() {
            let conn = self.get()?;
            let query = event::dsl::event
                .select(InnerEvent::keys())
                .order_by(event::date.desc())
                .into_boxed::<B>();
            let query = InnerEvent::filter(query, filter);
            let res: Option<InnerEvent<'_>> = exec_opt!(query, conn, first)?;
            Ok(res.map(|v| v.into()))
        } else {
            Ok(None)
        }
    }
}

impl<'a, 'b, B, C, A> FetchAll<'b, A, Event<'a>, EventFilter<'_>, Self>
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
    fn fetch_all(
        &self,
        filter: &'b EventFilter<'_>,
        auth: &'b A,
        page: usize,
    ) -> DbResult<DbList<Event<'a>>, Self> {
        if auth.is_admin() {
            let res: Vec<InnerEvent<'_>>;
            let conn = self.get()?;

            let offset = Self::compute_offset(page);
            let count_query =
                event::dsl::event.select(count_star()).into_boxed::<B>();
            let count_query = InnerEvent::filter(count_query, filter);
            let count = Self::compute_count(exec!(count_query, conn, first)?);
            let page_max = Self::compute_page_max(count);

            let query = event::dsl::event
                .select(InnerEvent::keys())
                .limit(25)
                .offset(offset)
                .order_by(event::date.desc())
                .into_boxed::<B>();
            let query = InnerEvent::filter(query, filter);
            res = exec!(query, conn, load)?;

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

impl<A, B, C: 'static + Connection> Create<A, Event<'_>, Self> for DieselDB<C>
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
    fn create(&self, object: &Event<'_>, _auth: &A) -> DbResult<(), Self> {
        let conn = self.get()?;
        let query = insert_into(event::dsl::event).values((
            event::id.eq(BinaryWrapper(&object.id)),
            event::actor_id.eq(object.actor_id.as_ref().map(BinaryWrapper)),
            event::details.eq(&object.details),
            event::type_.eq(DbWrapper(object.type_)),
            event::object_id.eq(object.object_id.as_ref().map(BinaryWrapper)),
        ));
        exec_unique!(query, conn, execute).map(|_| ())
    }
}

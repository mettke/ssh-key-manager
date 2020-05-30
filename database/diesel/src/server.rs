use crate::{
    error::DieselError,
    exec, exec_opt, exec_unique,
    migrate::Migrate,
    schema::{access, server, server_account, server_admin},
    BinaryWrapper, DbWrapper, DieselDB, UniqueExtension,
};
use core_common::{
    chrono::NaiveDateTime,
    database::{
        Create, Database, DatabaseError, DbList, DbResult, Delete, FetchAll,
        FetchAllFor, FetchById,
    },
    objects::{Server, ServerFilter},
    sec::Auth,
    types::{AuthorizationType, Id, KeyManagement, SyncStatusType},
};
use diesel::{
    backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax},
    connection::AnsiTransactionManager,
    deserialize::FromSql,
    dsl::count_star,
    insert_into,
    query_builder::BoxedSelectStatement,
    serialize::ToSql,
    sql_types::{Bool, HasSqlType, Timestamp},
    BoolExpressionMethods, Connection, ExpressionMethods, OptionalExtension,
    QueryDsl, Queryable, RunQueryDsl, TextExpressionMethods,
};
use std::borrow::{Borrow, Cow};

#[derive(Debug, Clone, Queryable)]
struct InnerServer<'a> {
    id: BinaryWrapper<Cow<'a, Id>>,
    hostname: Cow<'a, str>,
    ip_address: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
    key_management: DbWrapper<KeyManagement>,
    authorization: DbWrapper<AuthorizationType>,
    sync_status: DbWrapper<SyncStatusType>,
    rsa_key_fingerprint: Option<Cow<'a, str>>,
    port: i32,
}

impl InnerServer<'_> {
    fn permission_filter<'a, B, T>(
        query: BoxedSelectStatement<'a, T, server::table, B>,
        ids: &'a [BinaryWrapper<Cow<'a, Id>>],
    ) -> BoxedSelectStatement<'a, T, server::table, B>
    where
        B: 'a
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + HasSqlType<Bool>
            + HasSqlType<DbWrapper<KeyManagement>>
            + HasSqlType<DbWrapper<SyncStatusType>>,
        bool: ToSql<Bool, B>,
    {
        let access_query = access::dsl::access
            .select(access::dest_id)
            .filter(access::source_id.eq_any(ids));
        let server_account_query = server_account::dsl::server_account
            .select(server_account::server_id)
            .filter(server_account::entity_id.eq_any(access_query));
        let server_admin_query = server_admin::dsl::server_admin
            .select(server_admin::server_id)
            .filter(server_admin::entity_id.eq_any(ids));
        query.filter(
            server::id
                .eq_any(server_admin_query)
                .or(server::id.eq_any(server_account_query)),
        )
    }

    fn filter<'a, B, T>(
        mut query: BoxedSelectStatement<'a, T, server::table, B>,
        filter: &'a ServerFilter<'_>,
        ids: Option<&'a [BinaryWrapper<Cow<'a, Id>>]>,
    ) -> BoxedSelectStatement<'a, T, server::table, B>
    where
        B: 'a
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + HasSqlType<Bool>
            + HasSqlType<DbWrapper<KeyManagement>>
            + HasSqlType<DbWrapper<SyncStatusType>>,
        bool: ToSql<Bool, B>,
    {
        if let Some(ids) = ids {
            query = Self::permission_filter(query, ids);
        }
        if let Some(ref v) = filter.hostname {
            query = query.filter(server::hostname.like(v));
        }
        if let Some(ref v) = filter.ip_address {
            query = query.filter(server::ip_address.like(v));
        }
        if let Some(ref v) = filter.name {
            query = query.filter(server::name.like(v));
        }

        if let Some(ref v) = filter.key_management {
            let v: Vec<DbWrapper<KeyManagement>> =
                v.iter().copied().map(DbWrapper).collect();
            query = query.filter(server::key_management.eq_any(v));
        }
        if let Some(ref v) = filter.sync_status {
            let v: Vec<DbWrapper<SyncStatusType>> =
                v.iter().copied().map(DbWrapper).collect();
            query = query.filter(server::sync_status.eq_any(v));
        }

        query
    }
}

impl<'a> Into<Server<'a>> for InnerServer<'a> {
    fn into(self) -> Server<'a> {
        Server {
            id: self.id.0,
            hostname: self.hostname,
            ip_address: self.ip_address,
            name: self.name,
            key_management: self.key_management.0,
            authorization: self.authorization.0,
            sync_status: self.sync_status.0,
            rsa_key_fingerprint: self.rsa_key_fingerprint,
            port: self.port,
        }
    }
}

#[allow(clippy::type_repetition_in_bounds)]
impl<'a, 'b, B, C, A> FetchById<'a, A, Server<'b>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<KeyManagement>>
        + HasSqlType<DbWrapper<AuthorizationType>>
        + HasSqlType<DbWrapper<SyncStatusType>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
    DbWrapper<KeyManagement>: Queryable<DbWrapper<KeyManagement>, B>,
    DbWrapper<AuthorizationType>: Queryable<DbWrapper<AuthorizationType>, B>,
    DbWrapper<SyncStatusType>: Queryable<DbWrapper<SyncStatusType>, B>,
{
    #[inline]
    fn fetch(&self, id: &'a Id, auth: &A) -> DbResult<Option<Server<'b>>, Self> {
        let ids;
        let res: Option<InnerServer<'_>>;
        let conn = self.get()?;

        let query = server::dsl::server.find(BinaryWrapper(id));
        res = if auth.is_admin() {
            exec_opt!(query, conn, first)
        } else {
            ids = self.fetch_permission_ids(Cow::Borrowed(id))?;
            let ids: Vec<BinaryWrapper<Cow<'_, Id>>> = ids
                .iter()
                .map(Borrow::borrow)
                .map(Cow::Borrowed)
                .map(BinaryWrapper)
                .collect();
            let mut query = query.into_boxed::<B>();
            query = InnerServer::permission_filter(query, &ids);
            exec_opt!(query, conn, first)
        }?;
        Ok(res.map(|v| v.into()))
    }
}

#[allow(clippy::type_repetition_in_bounds)]
impl<'a, 'b, B: 'b, C, A> FetchAll<'b, A, Server<'a>, ServerFilter<'b>, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<KeyManagement>>
        + HasSqlType<DbWrapper<AuthorizationType>>
        + HasSqlType<DbWrapper<SyncStatusType>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
    DbWrapper<KeyManagement>: Queryable<DbWrapper<KeyManagement>, B>,
    DbWrapper<AuthorizationType>: Queryable<DbWrapper<AuthorizationType>, B>,
    DbWrapper<SyncStatusType>: Queryable<DbWrapper<SyncStatusType>, B>,
{
    #[inline]
    fn fetch_all(
        &self,
        filter: &'b ServerFilter<'b>,
        auth: &'b A,
        page: usize,
    ) -> DbResult<DbList<Server<'a>>, Self> {
        let ids: Vec<Cow<'_, Id>>;
        let id_ref: &'_ [Cow<'_, Id>];
        let res: Vec<InnerServer<'a>>;
        let conn = self.get()?;

        let offset = Self::compute_offset(page);

        let permission_ids = if auth.is_admin() {
            filter
                .permission_ids
                .as_ref()
                .map(Borrow::borrow)
                .map(Cow::Borrowed)
        } else {
            ids = self.fetch_permission_ids(Cow::Borrowed(auth.get_id()))?;
            id_ref = ids.as_slice();
            Some(Cow::Borrowed(id_ref))
        };
        let permission_ids: Option<Vec<BinaryWrapper<Cow<'_, Id>>>> =
            permission_ids.as_ref().map(|ids| {
                ids.iter()
                    .map(Borrow::borrow)
                    .map(Cow::Borrowed)
                    .map(BinaryWrapper)
                    .collect()
            });

        let count_query = server::dsl::server.select(count_star()).into_boxed::<B>();
        let count_query =
            InnerServer::filter(count_query, filter, permission_ids.as_deref());
        let count = Self::compute_count(exec!(count_query, conn, first)?);
        let page_max = Self::compute_page_max(count);

        let query = server::dsl::server
            .limit(25)
            .offset(offset)
            .into_boxed::<B>();
        let query = InnerServer::filter(query, filter, permission_ids.as_deref());
        res = exec!(query, conn, load)?;

        Ok(DbList {
            data: res.into_iter().map(|v| v.into()).collect(),
            count,
            page,
            page_max,
        })
    }
}

#[allow(clippy::type_repetition_in_bounds)]
impl<'a, B, C, A> FetchAllFor<A, Server<'a>, ServerFilter<'_>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<KeyManagement>>
        + HasSqlType<DbWrapper<AuthorizationType>>
        + HasSqlType<DbWrapper<SyncStatusType>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
    DbWrapper<KeyManagement>: Queryable<DbWrapper<KeyManagement>, B>,
    DbWrapper<AuthorizationType>: Queryable<DbWrapper<AuthorizationType>, B>,
    DbWrapper<SyncStatusType>: Queryable<DbWrapper<SyncStatusType>, B>,
{
    #[inline]
    fn fetch_all_for(
        &self,
        filter: &ServerFilter<'_>,
        auth: &A,
        page: usize,
    ) -> DbResult<DbList<Server<'a>>, Self> {
        let res: Vec<InnerServer<'a>>;
        let conn = self.get()?;

        let offset = Self::compute_offset(page);
        let id = auth.get_id();
        let permission_ids = self.fetch_permission_ids(Cow::Borrowed(id))?;
        let permission_ids: Option<Vec<BinaryWrapper<Cow<'_, Id>>>> = Some(
            permission_ids
                .iter()
                .map(Borrow::borrow)
                .map(Cow::Borrowed)
                .map(BinaryWrapper)
                .collect(),
        );

        let count_query = server::dsl::server.select(count_star()).into_boxed::<B>();
        let count_query =
            InnerServer::filter(count_query, filter, permission_ids.as_deref());
        let count = Self::compute_count(exec!(count_query, conn, first)?);
        let page_max = Self::compute_page_max(count);

        let query = server::dsl::server
            .limit(25)
            .offset(offset)
            .into_boxed::<B>();
        let query = InnerServer::filter(query, filter, permission_ids.as_deref());
        res = exec!(query, conn, load)?;

        Ok(DbList {
            data: res.into_iter().map(|v| v.into()).collect(),
            count,
            page,
            page_max,
        })
    }
}

impl<'a, A, B, C: 'static + Connection> Create<A, Server<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + SupportsDefaultKeyword
        + UsesAnsiSavepointSyntax
        + HasSqlType<DbWrapper<KeyManagement>>
        + HasSqlType<DbWrapper<AuthorizationType>>
        + HasSqlType<DbWrapper<SyncStatusType>>,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
{
    #[inline]
    fn create(&self, object: &Server<'a>, _auth: &A) -> DbResult<(), Self> {
        let conn = self.get()?;
        let query = insert_into(server::dsl::server).values((
            server::id.eq(BinaryWrapper(&object.id)),
            server::hostname.eq(&object.hostname),
            server::ip_address.eq(&object.ip_address),
            server::name.eq(&object.name),
            server::key_management.eq(DbWrapper(object.key_management)),
            server::authorization.eq(DbWrapper(object.authorization)),
            server::sync_status.eq(DbWrapper(object.sync_status)),
            server::name.eq(&object.rsa_key_fingerprint),
            server::port.eq(object.port),
        ));
        exec_unique!(query, conn, execute).map(|_| ())
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

impl<A, B, C> Delete<A, Server<'_>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<KeyManagement>>
        + HasSqlType<DbWrapper<AuthorizationType>>
        + HasSqlType<DbWrapper<SyncStatusType>>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
{
    #[inline]
    fn delete(&self, ids: &[Id], auth: &A) -> DbResult<(), Self> {
        if auth.is_admin() {
            let conn = self.get()?;
            let ids: Vec<BinaryWrapper<&Id>> =
                ids.iter().map(BinaryWrapper).collect();
            let query = diesel::delete(server::dsl::server)
                .filter(server::id.eq_any(&ids))
                .into_boxed::<B>();
            let _ = exec!(query, conn, execute)?;
        }
        Ok(())
    }
}

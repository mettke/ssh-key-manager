use crate::{
    error::DieselError, exec, exec_opt, exec_unique, migrate::Migrate,
    schema::public_key, BinaryWrapper, DbWrapper, DieselDB, UniqueExtension,
};
use core_common::{
    chrono::NaiveDateTime,
    database::{
        Create, Database, DatabaseError, DbList, DbResult, Delete, FetchAll,
        FetchAllFor, FetchById,
    },
    objects::{Event, PublicKey, PublicKeyFilter},
    sec::Auth,
    serde_json::json,
    types::{EventTypes, FingerprintMd5, FingerprintSha256, Id},
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
    QueryDsl, RunQueryDsl, TextExpressionMethods,
};
use std::{borrow::Cow, convert::AsRef};

#[derive(Debug, Clone, Queryable)]
struct InnerPublicKey<'a> {
    id: BinaryWrapper<Cow<'a, Id>>,
    entity_id: BinaryWrapper<Cow<'a, Id>>,
    type_: Cow<'a, str>,
    keydata: Cow<'a, str>,
    comment: Option<Cow<'a, str>>,
    keysize: Option<i32>,
    fingerprint_md5: Option<BinaryWrapper<Cow<'a, FingerprintMd5<'a>>>>,
    fingerprint_sha256: Option<BinaryWrapper<Cow<'a, FingerprintSha256<'a>>>>,
    randomart_md5: Option<Cow<'a, str>>,
    randomart_sha256: Option<Cow<'a, str>>,
    upload_date: Option<NaiveDateTime>,
}

type SelectType = (
    public_key::id,
    public_key::entity_id,
    public_key::type_,
    public_key::keydata,
    public_key::comment,
    public_key::keysize,
    public_key::fingerprint_md5,
    public_key::fingerprint_sha256,
    public_key::randomart_md5,
    public_key::randomart_sha256,
    Nullable<public_key::upload_date>,
);

impl InnerPublicKey<'_> {
    fn keys() -> SelectType {
        (
            public_key::id,
            public_key::entity_id,
            public_key::type_,
            public_key::keydata,
            public_key::comment,
            public_key::keysize,
            public_key::fingerprint_md5,
            public_key::fingerprint_sha256,
            public_key::randomart_md5,
            public_key::randomart_sha256,
            public_key::upload_date.nullable(),
        )
    }

    fn filter<'a, B, T>(
        mut query: BoxedSelectStatement<'a, T, public_key::table, B>,
        filter: &'a PublicKeyFilter<'_>,
        entity_id: Option<&'a Id>,
    ) -> BoxedSelectStatement<'a, T, public_key::table, B>
    where
        B: 'a
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + HasSqlType<Bool>,
        bool: ToSql<Bool, B>,
    {
        if let Some(ref v) = filter.fingerprint_md5 {
            query = query.filter(public_key::fingerprint_md5.eq(BinaryWrapper(v)));
        }
        if let Some(ref v) = filter.fingerprint_sha256 {
            query =
                query.or_filter(public_key::fingerprint_sha256.eq(BinaryWrapper(v)));
        }

        if let Some(v) = entity_id {
            query = query.filter(public_key::entity_id.eq(BinaryWrapper(v)));
        }
        if let Some(ref type_) = filter.type_ {
            query = query.filter(public_key::type_.eq(type_));
        }
        if let Some(ref comment) = filter.comment {
            query = query.filter(public_key::comment.like(comment));
        }
        if let Some(ref keysize_ge) = filter.keysize_ge {
            query = query.filter(public_key::keysize.ge(keysize_ge));
        }
        if let Some(ref keysize_le) = filter.keysize_le {
            query = query.filter(public_key::keysize.le(keysize_le));
        }

        query = query.filter(public_key::active.eq(true));
        query
    }
}

impl<'a> Into<PublicKey<'a>> for InnerPublicKey<'a> {
    fn into(self) -> PublicKey<'a> {
        PublicKey {
            id: self.id.0,
            entity_id: self.entity_id.0,
            type_: self.type_,
            keydata: self.keydata,
            comment: self.comment,
            keysize: self.keysize,
            fingerprint_md5: self.fingerprint_md5.map(|v| v.0),
            fingerprint_sha256: self.fingerprint_sha256.map(|v| v.0),
            randomart_md5: self.randomart_md5,
            randomart_sha256: self.randomart_sha256,
            upload_date: self.upload_date,
        }
    }
}

impl<'a, B, C, A> FetchById<'_, A, PublicKey<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
{
    #[inline]
    fn fetch(&self, id: &Id, auth: &A) -> DbResult<Option<PublicKey<'a>>, Self> {
        let res: Option<InnerPublicKey<'_>>;
        let conn = self.get()?;

        let query = public_key::dsl::public_key
            .select(InnerPublicKey::keys())
            .find(BinaryWrapper(id))
            .filter(public_key::active.eq(true));
        res = if auth.is_admin() {
            exec_opt!(query, conn, first)
        } else {
            let eid = BinaryWrapper(auth.get_id());
            let query = query.filter(public_key::entity_id.eq(eid));
            exec_opt!(query, conn, first)
        }?;
        Ok(res.map(|v| v.into()))
    }
}

impl<'a, 'b, B, C, A> FetchAll<'a, A, PublicKey<'b>, PublicKeyFilter<'_>, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
{
    #[inline]
    fn fetch_all(
        &self,
        filter: &'a PublicKeyFilter<'_>,
        auth: &'a A,
        page: usize,
    ) -> DbResult<DbList<PublicKey<'b>>, Self> {
        let res: Vec<InnerPublicKey<'b>>;
        let conn = self.get()?;

        let offset = Self::compute_offset(page);

        let entity_id = if auth.is_admin() {
            filter.entity_id.as_ref().map(AsRef::as_ref)
        } else {
            Some(auth.get_id())
        };

        let count_query = public_key::dsl::public_key
            .select(count_star())
            .into_boxed::<B>();
        let count_query = InnerPublicKey::filter(count_query, filter, entity_id);
        let count = Self::compute_count(exec!(count_query, conn, first)?);
        let page_max = Self::compute_page_max(count);

        let query = public_key::dsl::public_key
            .select(InnerPublicKey::keys())
            .limit(25)
            .offset(offset)
            .into_boxed::<B>();
        let query = InnerPublicKey::filter(query, filter, entity_id);
        res = exec!(query, conn, load)?;

        Ok(DbList {
            data: res.into_iter().map(|v| v.into()).collect(),
            count,
            page,
            page_max,
        })
    }
}

impl<'a, B, C, A> FetchAllFor<A, PublicKey<'a>, PublicKeyFilter<'_>, Self>
    for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
    NaiveDateTime: FromSql<Timestamp, B>,
{
    #[inline]
    fn fetch_all_for(
        &self,
        filter: &PublicKeyFilter<'_>,
        auth: &A,
        page: usize,
    ) -> DbResult<DbList<PublicKey<'a>>, Self> {
        let res: Vec<InnerPublicKey<'a>>;
        let conn = self.get()?;
        let entity_id = Some(auth.get_id());

        let offset = Self::compute_offset(page);

        let count_query = public_key::dsl::public_key
            .select(count_star())
            .into_boxed::<B>();
        let count_query = InnerPublicKey::filter(count_query, filter, entity_id);
        let count = Self::compute_count(exec!(count_query, conn, first)?);
        let page_max = Self::compute_page_max(count);

        let query = public_key::dsl::public_key
            .select(InnerPublicKey::keys())
            .limit(25)
            .offset(offset)
            .into_boxed::<B>();
        let query = InnerPublicKey::filter(query, filter, entity_id);
        res = exec!(query, conn, load)?;

        Ok(DbList {
            data: res.into_iter().map(|v| v.into()).collect(),
            count,
            page,
            page_max,
        })
    }
}

impl<'a, A, B, C: 'static + Connection> Create<A, PublicKey<'a>, Self>
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
    fn create(&self, object: &PublicKey<'a>, auth: &A) -> DbResult<(), Self> {
        let conn = self.get()?;
        let query = insert_into(public_key::dsl::public_key).values((
            public_key::id.eq(BinaryWrapper(&object.id)),
            public_key::entity_id.eq(BinaryWrapper(&object.entity_id)),
            public_key::type_.eq(&object.type_),
            public_key::keydata.eq(&object.keydata),
            public_key::comment.eq(&object.comment),
            public_key::keysize.eq(&object.keysize),
            public_key::fingerprint_md5
                .eq(object.fingerprint_md5.as_ref().map(BinaryWrapper)),
            public_key::fingerprint_sha256
                .eq(object.fingerprint_sha256.as_ref().map(BinaryWrapper)),
            public_key::randomart_md5.eq(&object.randomart_md5),
            public_key::randomart_sha256.eq(&object.randomart_sha256),
        ));
        let res = exec_unique!(query, conn, execute).map(|_| ());
        if let DbResult::Ok(_) = res {
            let details = Cow::Owned(
                json!({
                    "action": "Pubkey add",
                    "value": object.fingerprint_md5,
                    "id": &object.id
                })
                .to_string(),
            );
            let event: Event<'_> = Event {
                id: Cow::Owned(self.generate_id()?),
                actor_id: Some(Cow::Borrowed(auth.get_id())),
                date: None,
                details,
                type_: EventTypes::Entity,
                object_id: Some(Cow::Borrowed(&object.entity_id)),
            };
            self.create(&event, auth)?;
        }
        res
    }
}

impl<A, B, C> Delete<A, PublicKey<'_>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + Backend
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + SupportsDefaultKeyword,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
{
    #[inline]
    fn delete(&self, ids: &[Id], auth: &A) -> DbResult<(), Self> {
        let conn = self.get()?;
        let ids: Vec<BinaryWrapper<&Id>> = ids.iter().map(BinaryWrapper).collect();

        let mut query = diesel::update(public_key::dsl::public_key)
            .set(public_key::active.eq(false))
            .filter(public_key::id.eq_any(&ids))
            .into_boxed::<B>();

        if !auth.is_admin() {
            query =
                query.filter(public_key::entity_id.eq(BinaryWrapper(auth.get_id())));
        }
        let _ = exec!(query, conn, execute)?;
        Ok(())
    }
}

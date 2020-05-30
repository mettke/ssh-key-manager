use crate::{
    error::DieselError,
    exec, exec_opt, exec_unique,
    migrate::Migrate,
    schema::{entity, groups},
    BinaryWrapper, DbWrapper, DieselDB, UniqueExtension,
};
use core_common::{
    database::{
        Create, DatabaseError, DbList, DbResult, Delete, FetchAll, FetchById,
    },
    objects::{Group, GroupFilter},
    sec::Auth,
    types::{EntityTypes, Id},
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
    Connection, ExpressionMethods, OptionalExtension, QueryDsl, Queryable,
    RunQueryDsl, TextExpressionMethods,
};
use std::borrow::Cow;

#[derive(Debug, Clone, Queryable)]
struct InnerGroup<'a> {
    entity_id: BinaryWrapper<Cow<'a, Id>>,
    name: Cow<'a, str>,
    system: bool,
    oauth_scope: Option<Cow<'a, str>>,
    ldap_group: Option<Cow<'a, str>>,
}

impl InnerGroup<'_> {
    fn filter<'a, B, T>(
        mut query: BoxedSelectStatement<'a, T, groups::table, B>,
        filter: &'a GroupFilter<'_>,
    ) -> BoxedSelectStatement<'a, T, groups::table, B>
    where
        B: 'a
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + HasSqlType<Bool>,
        bool: ToSql<Bool, B>,
    {
        if let Some(ref v) = filter.name {
            query = query.filter(groups::name.like(v));
        }

        query
    }
}

impl<'a> Into<Group<'a>> for InnerGroup<'a> {
    fn into(self) -> Group<'a> {
        Group {
            entity_id: self.entity_id.0,
            name: self.name,
            system: self.system,
            oauth_scope: self.oauth_scope,
            ldap_group: self.ldap_group,
        }
    }
}

#[allow(clippy::type_repetition_in_bounds)]
impl<'a, B, C, A> FetchById<'_, A, Group<'a>, Self> for DieselDB<C>
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
    bool: ToSql<Bool, B> + FromSql<Bool, B>,
{
    #[inline]
    fn fetch(&self, id: &Id, _auth: &A) -> DbResult<Option<Group<'a>>, Self> {
        let res: Option<InnerGroup<'a>>;
        let conn = self.get()?;

        let query = groups::dsl::groups.find(BinaryWrapper(id));
        res = exec_opt!(query, conn, first)?;
        Ok(res.map(|v| v.into()))
    }
}

#[allow(clippy::type_repetition_in_bounds)]
impl<'a, B, C, A> FetchAll<'_, A, Group<'a>, GroupFilter<'_>, Self> for DieselDB<C>
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
    bool: ToSql<Bool, B> + FromSql<Bool, B>,
{
    #[inline]
    fn fetch_all(
        &self,
        filter: &GroupFilter<'_>,
        _auth: &A,
        page: usize,
    ) -> DbResult<DbList<Group<'a>>, Self> {
        let res: Vec<InnerGroup<'a>>;
        let conn = self.get()?;

        let offset = Self::compute_offset(page);

        let count_query = groups::dsl::groups.select(count_star()).into_boxed::<B>();
        let count_query = InnerGroup::filter(count_query, filter);
        let count = Self::compute_count(exec!(count_query, conn, first)?);
        let page_max = Self::compute_page_max(count);

        let query = groups::dsl::groups
            .limit(25)
            .offset(offset)
            .into_boxed::<B>();
        let query = InnerGroup::filter(query, filter);
        res = exec!(query, conn, load)?;

        Ok(DbList {
            data: res.into_iter().map(|v| v.into()).collect(),
            count,
            page,
            page_max,
        })
    }
}

impl<'a, A, B, C: 'static + Connection> Create<A, Group<'a>, Self> for DieselDB<C>
where
    A: Auth,
    B: 'static
        + Backend<RawValue = [u8]>
        + SupportsDefaultKeyword
        + UsesAnsiSavepointSyntax
        + HasSqlType<Bool>
        + HasSqlType<DbWrapper<EntityTypes>>,
    C: 'static
        + Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
    bool: ToSql<Bool, B>,
{
    #[inline]
    fn create(&self, object: &Group<'a>, _auth: &A) -> DbResult<(), Self> {
        let conn = self.get()?;
        let query = insert_into(entity::dsl::entity).values((
            entity::id.eq(BinaryWrapper(&object.entity_id)),
            entity::type_.eq(DbWrapper(EntityTypes::Group)),
        ));
        let _ = exec_unique!(query, conn, execute)?;

        let query = insert_into(groups::dsl::groups).values((
            groups::entity_id.eq(BinaryWrapper(&object.entity_id)),
            groups::name.eq(&object.name),
            groups::system.eq(object.system),
            groups::oauth_scope.eq(&object.oauth_scope),
            groups::ldap_group.eq(&object.ldap_group),
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

impl<A, B, C> Delete<A, Group<'_>, Self> for DieselDB<C>
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
    bool: ToSql<Bool, B>,
{
    #[inline]
    fn delete(&self, ids: &[Id], auth: &A) -> DbResult<(), Self> {
        if auth.is_admin() {
            let conn = self.get()?;
            let ids: Vec<BinaryWrapper<&Id>> =
                ids.iter().map(BinaryWrapper).collect();
            let query = diesel::delete(groups::dsl::groups)
                .filter(groups::entity_id.eq_any(&ids))
                .into_boxed::<B>();
            let _ = exec!(query, conn, execute)?;
        }
        Ok(())
    }
}

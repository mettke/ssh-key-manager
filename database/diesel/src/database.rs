use crate::{
    error::DieselError, exec, migrate::Migrate, schema::group_member, BinaryWrapper,
};
use core_common::{
    async_trait::async_trait,
    database::{Database, DatabaseError, DbResult},
    tokio::task,
    types::Id,
};
use diesel::{
    backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax},
    connection::AnsiTransactionManager,
    prelude::QueryResult,
    r2d2::{ConnectionManager, NopErrorHandler, Pool, PooledConnection},
    result::DatabaseErrorKind,
    select,
    sql_types::{Binary, Nullable, Text},
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use std::{
    borrow::Cow,
    collections::HashSet,
    convert::{TryFrom, TryInto},
    error::Error,
    fmt,
    time::Duration,
};

sql_function! {
    #[sql_name = "coalesce"]
    fn coalesce4(a: Nullable<Text>, b: Nullable<Text>, c: Nullable<Text>, d: Nullable<Text>) -> Nullable<Text>
}

no_arg_sql_function!(GEN_UUID, Binary, "Represents the GEN_UUID() function");

/// Catches a `diesel::result::DatabaseErrorKind::UniqueViolation`
pub trait UniqueExtension<T, B, C>
where
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + SupportsDefaultKeyword,
    C: Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
{
    /// Converts a `DbResult<T>` to a `DbValue<T>`
    ///
    /// # Errors
    /// Other database errors are kept intact and may occur
    fn unique(self) -> DbResult<T, DieselDB<C>>;
}

impl<T, B, C> UniqueExtension<T, B, C> for QueryResult<T>
where
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + SupportsDefaultKeyword,
    C: Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
{
    #[inline]
    fn unique(self) -> DbResult<T, DieselDB<C>> {
        match self {
            Ok(value) => Ok(value),
            Err(diesel::result::Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _,
            )) => Err(DatabaseError::NonUnique),
            Err(e) => Err(DatabaseError::Custom(DieselError::DieselError(e))),
        }
    }
}

/// `DieselPool` wraps a R2D2 Connection Pool
type DieselPool<C> = Pool<ConnectionManager<C>>;
/// `DieselPooledConnection` wraps a R2D2 Connection from a Pool
pub type DieselPooledConnection<T> = PooledConnection<ConnectionManager<T>>;

/// Diesel Database containing the connection pool
pub struct DieselDB<C: 'static + Connection> {
    /// Connection Pool which may contain multiple databases and multiple
    /// connections to each datbase
    pub pools: Vec<DieselPool<C>>,
}

impl<T: 'static + Connection> Clone for DieselDB<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            pools: self.pools.clone(),
        }
    }
}

impl<T: 'static + Connection> fmt::Debug for DieselDB<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
            .field("pools", &String::from("<hidden>"))
            .finish()
    }
}

#[async_trait]
impl<B, C> Database for DieselDB<C>
where
    B: 'static
        + Backend<RawValue = [u8]>
        + UsesAnsiSavepointSyntax
        + SupportsDefaultKeyword,
    C: Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
{
    type DatabaseError = DieselError;

    #[inline]
    async fn generate_id(&self) -> Result<Id, DatabaseError<Self>> {
        let uuid = select(GEN_UUID);
        task::block_in_place(|| {
            let conn = self.get()?;
            exec!(uuid, conn, first).map(|id: BinaryWrapper<Id>| id.0)
        })
    }

    #[inline]
    async fn fetch_permission_ids<'a>(
        &self,
        entity_id: Cow<'a, Id>,
    ) -> Result<Vec<Cow<'a, Id>>, DatabaseError<Self>> {
        task::block_in_place(|| {
            let conn = self.get()?;
            let mut set = HashSet::new();
            let mut vec = vec![BinaryWrapper(entity_id)];
            while !vec.is_empty() {
                let i = 0;
                while i < vec.len() {
                    #[allow(clippy::indexing_slicing)]
                    let id = &vec[i];
                    if set.contains(&id.0) {
                        let _ = vec.swap_remove(i);
                    } else {
                        let _ = set.insert(id.0.clone());
                    }
                }
                let query = group_member::dsl::group_member
                    .select(group_member::group_id)
                    .filter(group_member::member_id.eq_any(vec));
                vec = exec!(query, conn, load)?;
            }
            Ok(set.into_iter().collect())
        })
    }

    #[inline]
    fn migrate(&self) -> Result<(), DatabaseError<Self>> {
        let mut err = DatabaseError::Custom(DieselError::NoServerAvailable);
        let mut migrated = false;
        for pool in &self.pools {
            if let Err(inner) = pool
                .get_timeout(Duration::from_millis(1000))
                .map_err(|err| DatabaseError::Custom(DieselError::R2D2Error(err)))
                .and_then(|conn| {
                    C::migrate(&conn)
                        .map_err(DieselError::MigrationError)
                        .map_err(DatabaseError::Custom)
                })
                .map(|_| migrated = true)
            {
                err = inner;
            }
        }
        if migrated {
            Ok(())
        } else {
            Err(err)
        }
    }
}

impl<B, C> DieselDB<C>
where
    B: 'static
        + Backend<RawValue = [u8]>
        + Backend
        + UsesAnsiSavepointSyntax
        + SupportsDefaultKeyword,
    C: Connection<Backend = B, TransactionManager = AnsiTransactionManager>
        + Migrate,
{
    /// Creates a new pooled connection to the given sql server. The URL is in the format:
    ///
    /// ```{none}
    /// postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]]
    /// ```
    ///
    /// # Errors
    /// Returns `Err(err)` if there are any errors connecting to the sql database.
    #[inline]
    pub fn new(urls: impl Iterator<Item = String>) -> Result<Self, Box<dyn Error>> {
        let pools = urls
            .map(|s| {
                let manager = ConnectionManager::<C>::new(&s);
                Pool::builder()
                    .max_size(20)
                    .min_idle(Some(2))
                    .error_handler(Box::new(NopErrorHandler {}))
                    .build_unchecked(manager)
            })
            .collect();
        Ok(Self::new_with_pool(pools))
    }

    /// Creates a instance of the middleware with the ability to provide a preconfigured pool.
    #[must_use]
    #[inline]
    pub fn new_with_pool(pools: Vec<Pool<ConnectionManager<C>>>) -> Self {
        Self { pools }
    }

    /// Gets a connection from the pool
    ///
    /// # Errors
    /// Returns an error if there are no connections in the pool
    #[inline]
    pub fn get(&self) -> DbResult<PooledConnection<ConnectionManager<C>>, Self> {
        let mut err = None;
        for pool in &self.pools {
            let state = pool.state();
            if state.connections != 0 {
                match pool.get_timeout(Duration::from_millis(100)) {
                    Ok(conn) => return Ok(conn),
                    Err(e) => err = Some(e),
                }
            }
        }
        Err(DatabaseError::Custom(
            err.map_or(DieselError::NoServerAvailable, |err| {
                DieselError::R2D2Error(err)
            }),
        ))
    }

    pub(crate) fn compute_count(count: i64) -> usize {
        usize::try_from(count).unwrap_or_default()
    }

    pub(crate) fn compute_offset(page: usize) -> i64 {
        let page_i64: i64 = page.saturating_sub(1).try_into().unwrap_or_default();
        page_i64.checked_mul(25).unwrap_or_default()
    }

    pub(crate) fn compute_page_max(count: usize) -> usize {
        let mut page_max = count.wrapping_div(25);
        if count.checked_rem(25).unwrap_or_default() != 0 {
            page_max = page_max.wrapping_add(1);
        }
        page_max
    }
}

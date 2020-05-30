use crate::{DbFrom, DbName, DbTo};
use core_common::serde::Serialize;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::{bound::Bound, AsExpression},
    mysql::{Mysql, MysqlType},
    pg::Pg,
    query_builder::QueryId,
    row::Row,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{HasSqlType, Nullable, SingleValue},
    types::NotNull,
    Queryable,
};
use std::{fmt::Debug, io::Write, ops::Deref};

/// Wrapper for a type to be usable with Diesel Query Logic
#[derive(Debug, Clone, Hash, Serialize, PartialEq, Eq)]
pub struct DbWrapper<T: Debug>(pub T);

impl<T: Debug> Deref for DbWrapper<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static + Debug> QueryId for DbWrapper<T> {
    type QueryId = Self;
    const HAS_STATIC_QUERY_ID: bool = true;
}
impl<T: Debug> NotNull for DbWrapper<T> {}
impl<T: Debug> SingleValue for DbWrapper<T> {}

impl<T: Debug> AsExpression<DbWrapper<T>> for DbWrapper<T> {
    type Expression = Bound<Self, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

#[allow(clippy::use_self)]
impl<T: Debug> AsExpression<DbWrapper<T>> for DbWrapper<&'_ T> {
    type Expression = Bound<DbWrapper<T>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<Nullable<DbWrapper<T>>> for DbWrapper<T> {
    type Expression = Bound<Nullable<Self>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

#[allow(clippy::use_self)]
impl<T: Debug> AsExpression<Nullable<DbWrapper<T>>> for DbWrapper<&'_ T> {
    type Expression = Bound<Nullable<DbWrapper<T>>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<DbWrapper<T>> for &'_ DbWrapper<T> {
    type Expression = Bound<DbWrapper<T>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<DbWrapper<T>> for &'_ DbWrapper<&'_ T> {
    type Expression = Bound<DbWrapper<T>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<Nullable<DbWrapper<T>>> for &'_ DbWrapper<T> {
    type Expression = Bound<Nullable<DbWrapper<T>>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<Nullable<DbWrapper<T>>> for &'_ DbWrapper<&'_ T> {
    type Expression = Bound<Nullable<DbWrapper<T>>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<DbWrapper<T>> for &'_ &'_ DbWrapper<T> {
    type Expression = Bound<DbWrapper<T>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<DbWrapper<T>> for &'_ &'_ DbWrapper<&'_ T> {
    type Expression = Bound<DbWrapper<T>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<Nullable<DbWrapper<T>>> for &'_ &'_ DbWrapper<T> {
    type Expression = Bound<Nullable<DbWrapper<T>>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: Debug> AsExpression<Nullable<DbWrapper<T>>> for &'_ &'_ DbWrapper<&'_ T> {
    type Expression = Bound<Nullable<DbWrapper<T>>, Self>;

    #[inline]
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<T: DbTo, DB: Backend> ToSql<DbWrapper<T>, DB> for DbWrapper<T> {
    #[inline]
    fn to_sql<W: Write>(&self, out: &mut Output<'_, W, DB>) -> serialize::Result {
        let v = self.0.convert_back();
        out.write_all(v)?;
        Ok(IsNull::No)
    }
}

impl<T: DbTo, DB: Backend> ToSql<DbWrapper<T>, DB> for DbWrapper<&T> {
    #[inline]
    fn to_sql<W: Write>(&self, out: &mut Output<'_, W, DB>) -> serialize::Result {
        let v = self.0.convert_back();
        out.write_all(v)?;
        Ok(IsNull::No)
    }
}

impl<T: Debug, DB> ToSql<Nullable<DbWrapper<T>>, DB> for DbWrapper<T>
where
    DB: Backend,
    Self: ToSql<DbWrapper<T>, DB>,
{
    #[inline]
    fn to_sql<W: ::std::io::Write>(
        &self,
        out: &mut Output<'_, W, DB>,
    ) -> serialize::Result {
        ToSql::<Self, DB>::to_sql(self, out)
    }
}

impl<'a, T: Debug, DB> ToSql<Nullable<DbWrapper<T>>, DB> for DbWrapper<&'a T>
where
    DB: Backend,
    Self: ToSql<DbWrapper<&'a T>, DB>,
{
    #[inline]
    fn to_sql<W: ::std::io::Write>(
        &self,
        out: &mut Output<'_, W, DB>,
    ) -> serialize::Result {
        ToSql::<Self, DB>::to_sql(self, out)
    }
}

impl<T: Debug> HasSqlType<DbWrapper<T>> for Mysql {
    #[inline]
    fn metadata(_lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
        MysqlType::String
    }
}

impl<T: DbFrom> FromSqlRow<DbWrapper<T>, Mysql> for DbWrapper<T> {
    #[inline]
    fn build_from_row<R: Row<Mysql>>(row: &mut R) -> deserialize::Result<Self> {
        FromSql::<Self, Mysql>::from_sql(row.take())
    }
}

impl<T: DbFrom> FromSql<DbWrapper<T>, Mysql> for DbWrapper<T> {
    #[inline]
    fn from_sql(raw: Option<&[u8]>) -> deserialize::Result<Self> {
        match raw {
            Some(v) => T::convert(v).map(Self).map_err(|_| {
                format!(
                    "Unrecognized enum variant: '{}'",
                    String::from_utf8_lossy(v)
                )
                .into()
            }),
            None => Err("Unexpected null for non-null column".into()),
        }
    }
}

impl<T: DbFrom> Queryable<DbWrapper<T>, Mysql> for DbWrapper<T> {
    type Row = Self;

    #[inline]
    fn build(row: Self::Row) -> Self {
        row
    }
}

impl<T: DbName> HasSqlType<DbWrapper<T>> for Pg {
    #[inline]
    fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
        lookup.lookup_type(T::db_type_name())
    }
}

impl<T: DbFrom> FromSqlRow<DbWrapper<T>, Pg> for DbWrapper<T> {
    #[inline]
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> deserialize::Result<Self> {
        FromSql::<Self, Pg>::from_sql(row.take())
    }
}

impl<T: DbFrom> FromSql<DbWrapper<T>, Pg> for DbWrapper<T> {
    #[inline]
    fn from_sql(raw: Option<&[u8]>) -> deserialize::Result<Self> {
        match raw {
            Some(v) => T::convert(v).map(Self).map_err(|_| {
                format!(
                    "Unrecognized enum variant: '{}'",
                    String::from_utf8_lossy(v)
                )
                .into()
            }),
            None => Err("Unexpected null for non-null column".into()),
        }
    }
}

impl<T: DbFrom> Queryable<DbWrapper<T>, Pg> for DbWrapper<T> {
    type Row = Self;

    #[inline]
    fn build(row: Self::Row) -> Self {
        row
    }
}

use crate::{DbFrom, DbTo};
use core_common::serde::Serialize;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, Output, ToSql},
    sql_types::Binary,
    AsExpression, FromSqlRow,
};
use std::{borrow::Cow, fmt::Debug, io, ops::Deref};

/// Wrapper for a type to be usable with Diesel Query Logic
#[derive(Debug, Clone, Hash, PartialEq, Serialize, AsExpression, FromSqlRow, Eq)]
#[sql_type = "Binary"]
pub struct BinaryWrapper<T: Debug>(pub T);

impl<T: Debug> Deref for BinaryWrapper<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DbTo, DB: Backend> ToSql<Binary, DB> for BinaryWrapper<T>
where
    [u8]: ToSql<Binary, DB>,
{
    #[inline]
    fn to_sql<W>(&self, out: &mut Output<'_, W, DB>) -> serialize::Result
    where
        W: io::Write,
    {
        let v: &[u8] = self.0.convert_back();
        <[u8]>::to_sql(v, out)
    }
}

impl<T: DbTo + Clone, DB: Backend> ToSql<Binary, DB> for BinaryWrapper<Cow<'_, T>>
where
    [u8]: ToSql<Binary, DB>,
{
    #[inline]
    fn to_sql<W>(&self, out: &mut Output<'_, W, DB>) -> serialize::Result
    where
        W: io::Write,
    {
        let v: &[u8] = self.0.convert_back();
        <[u8]>::to_sql(v, out)
    }
}

impl<T: DbTo + Clone, DB: Backend> ToSql<Binary, DB> for BinaryWrapper<&Cow<'_, T>>
where
    [u8]: ToSql<Binary, DB>,
{
    #[inline]
    fn to_sql<W>(&self, out: &mut Output<'_, W, DB>) -> serialize::Result
    where
        W: io::Write,
    {
        let v: &[u8] = self.0.convert_back();
        <[u8]>::to_sql(v, out)
    }
}

impl<T: DbFrom, DB: Backend> FromSql<Binary, DB> for BinaryWrapper<T>
where
    *const [u8]: FromSql<Binary, DB>,
{
    #[allow(unsafe_code)]
    #[inline]
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let v: &[u8] = unsafe {
            <*const [u8]>::from_sql(bytes)?
                .as_ref()
                .expect("Database reference is invalid")
        };
        T::convert(v).map(Self).map_err(|err| Box::new(err).into())
    }
}

impl<T: DbFrom + Clone, DB: Backend> FromSql<Binary, DB>
    for BinaryWrapper<Cow<'_, T>>
where
    *const [u8]: FromSql<Binary, DB>,
{
    #[allow(unsafe_code)]
    #[inline]
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let v: &[u8] = unsafe {
            <*const [u8]>::from_sql(bytes)?
                .as_ref()
                .expect("Database reference is invalid")
        };
        T::convert(v)
            .map(Cow::Owned)
            .map(Self)
            .map_err(|err| Box::new(err).into())
    }
}

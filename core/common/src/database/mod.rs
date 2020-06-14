//! This module contains various types for database interaction

mod create;
#[allow(clippy::module_inception)]
mod database;
mod database_error;
mod db_list;
mod delete;
mod delete_obj;
mod fetch_all;
mod fetch_all_for;
mod fetch_by_id;
mod fetch_by_uid;
mod fetch_first;
mod save;
#[doc(hidden)]
pub mod tests;

/// Result type for Database Communication
pub type DbResult<T, D> = Result<T, DatabaseError<D>>;

pub use self::{
    create::Create, database::Database, database_error::DatabaseError,
    db_list::DbList, delete::Delete, delete_obj::DeleteObj, fetch_all::FetchAll,
    fetch_all_for::FetchAllFor, fetch_by_id::FetchById, fetch_by_uid::FetchByUid,
    fetch_first::FetchFirst, save::Save,
};

//! Core Library providing data structures

#![deny(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    // box_pointers,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![deny(
    clippy::correctness,
    clippy::restriction,
    clippy::style,
    clippy::pedantic,
    clippy::complexity,
    clippy::perf,
    clippy::cargo,
    clippy::nursery
)]
#![allow(
    clippy::implicit_return,
    clippy::missing_docs_in_private_items,
    clippy::result_expect_used,
    clippy::shadow_reuse,
    clippy::option_expect_used,
    clippy::similar_names,
    clippy::else_if_without_else,
    clippy::multiple_crate_versions,
    clippy::module_name_repetitions
)]
#![allow(macro_use_extern_crate)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod binary_wrapper;
mod common_types;
mod database;
mod db_traits;
mod db_wrapper;
mod error;
mod event;
mod group;
mod macros;
mod migrate;
mod public_key;
mod schema;
mod server;
mod user;

pub use crate::{
    binary_wrapper::BinaryWrapper,
    database::{DieselDB, DieselPooledConnection, UniqueExtension},
    db_traits::{DbFrom, DbName, DbTo},
    db_wrapper::DbWrapper,
    error::DieselError,
};
pub use diesel::{mysql::MysqlConnection, pg::PgConnection};

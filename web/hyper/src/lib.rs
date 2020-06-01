//! Core Library implementation for the hyper framework

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

mod hyper_request;
mod hyper_server;

pub use crate::{
    hyper_request::HyperRequest,
    hyper_server::{HyperError, HyperServer},
};

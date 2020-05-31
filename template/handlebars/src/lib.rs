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
    // meta_variable_misuse,
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

mod helper;

use core_common::{
    serde::Serialize,
    web::{RenderError, TemplateEngine},
};
use handlebars::{Handlebars, RenderError as HRenderError, TemplateFileError};
use std::{
    path::PathBuf,
    sync::{RwLock, RwLockWriteGuard},
};

/// The handlebars template engine
#[derive(Debug)]
pub struct Template {
    /// Sources from which templates are loaded
    pub sources: Vec<DirectorySource>,
    /// Internal Handlebar Registry
    pub registry: RwLock<Handlebars<'static>>,
}

impl Default for Template {
    #[inline]
    fn default() -> Self {
        let mut hbs = Handlebars::new();
        for (name, func) in crate::helper::get_helpers() {
            let _ = hbs.register_helper(name, func);
        }
        Self {
            sources: Vec::new(),
            registry: RwLock::new(hbs),
        }
    }
}

impl TemplateEngine for Template {
    type TemplateError = HRenderError;

    #[inline]
    fn render<T: Serialize>(
        &self,
        name: &str,
        data: &T,
    ) -> Result<String, RenderError<Self>> {
        let hbs = self.registry.read().expect("Unable to lock registry");
        hbs.render(name, data).map_err(RenderError::Custom)
    }
}

impl Template {
    /// add a template source
    #[inline]
    pub fn add(&mut self, source: DirectorySource) {
        self.sources.push(source);
    }

    /// load template from registered sources
    ///
    /// # Errors
    /// Fails when a folder could not be reloaded
    #[inline]
    pub fn reload(&self) -> Result<(), TemplateFileError> {
        let mut hbs = self.handlebars_mut();
        hbs.clear_templates();
        for s in &self.sources {
            s.load(&mut hbs)?
        }
        Ok(())
    }

    /// access internal handlebars registry, useful to register custom helpers
    #[inline]
    pub fn handlebars_mut(&self) -> RwLockWriteGuard<'_, Handlebars<'static>> {
        self.registry.write().expect("Unable to lock registry")
    }
}

/// Import files from directory
#[derive(Debug, Clone)]
pub struct DirectorySource {
    /// Path to file
    pub prefix: PathBuf,
    /// File suffix
    pub suffix: &'static str,
}

impl DirectorySource {
    /// Create a new directory source
    #[inline]
    pub fn new<P>(prefix: P, suffix: &'static str) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            prefix: prefix.into(),
            suffix,
        }
    }

    fn load(&self, reg: &mut Handlebars<'_>) -> Result<(), TemplateFileError> {
        reg.register_templates_directory(self.suffix, &self.prefix)
    }
}

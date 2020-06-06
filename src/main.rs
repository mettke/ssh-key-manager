//! A tool for managing user and server SSH access to any number of servers.

#![deny(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
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
    clippy::wildcard_enum_match_arm,
    clippy::exit,
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::else_if_without_else,
    clippy::multiple_crate_versions,
    clippy::cargo_common_metadata
)]

mod args;
mod routes;

use crate::args::{get_arguments, AuthType, CliArguments};
use core_common::{
    database::{Create, Database, Delete, FetchAll, FetchById, FetchByUid, Save},
    objects::{Entity, PublicKey, PublicKeyFilter, User},
    sec::{Auth, OAuth2, PreAuth},
    tokio::{fs, signal},
    web::{BaseData, BaseView, Server, TemplateEngine},
};
use std::{process::exit, sync::Arc, time::SystemTime};

#[cfg(feature = "diesel")]
use database_diesel::{DieselDB, PgConnection};
#[cfg(feature = "jwt")]
use sec_token::Token;
#[cfg(not(feature = "handlebars"))]
#[cfg(not(feature = "diesel"))]
#[cfg(not(feature = "hyper"))]
#[cfg(not(feature = "jwt"))]
use std::compile_error;
#[cfg(feature = "handlebars")]
use template_handlebars::{DirectorySource, Template};
#[cfg(feature = "hyper")]
use web_hyper::HyperServer;

async fn shutdown_signal() {
    if let Err(err) = signal::ctrl_c().await {
        eprintln!("Unable to install CTRL+C signal handler: {}", err);
        exit(1);
    }
}

#[tokio::main]
#[allow(clippy::print_stdout)]
async fn main() {
    let args = get_arguments();
    if let Some(log_level) = args.log_level {
        if let Err(err) = simple_logger::init_with_level(log_level) {
            eprintln!("Unable to initialize Logger: {}", err);
            exit(1);
        }
    }
    if args.print_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }
    let database = {
        #[cfg(not(feature = "diesel_pg"))]
        {
            compile_error!("Database backend is required")
        }
        #[cfg(feature = "diesel_pg")]
        {
            let urls = args.db_host.iter().map(|host| {
                format!(
                    "postgres://{}:{}@{}/{}",
                    args.db_user, args.db_pass, host, args.db_name
                )
            });
            match DieselDB::<PgConnection>::new(urls) {
                Err(err) => {
                    eprintln!("Unable to setup database: {}", err);
                    exit(1);
                }
                Ok(db) => db,
            }
        }
    };
    if let Err(err) = database.migrate() {
        log::error!("Unable to migrate database: {}", err);
        exit(1);
    }
    let templates = {
        #[cfg(not(feature = "handlebars"))]
        {
            compile_error!("Template backend is required")
        }
        #[cfg(feature = "handlebars")]
        {
            let mut hbse = Template::default();
            hbse.add(DirectorySource::new("./templates/", ".hbs"));
            if let Err(err) = hbse.reload() {
                log::error!("Unable to find template directory: {}", err);
                exit(1);
            }
            hbse
        }
    };
    #[cfg(not(feature = "jwt"))]
    {
        compile_error!("Auth backend is required")
    }
    #[cfg(feature = "jwt")]
    {
        build_server::<Token, _, _>(&args, Arc::new(database), Arc::new(templates))
            .await;
    }
}

#[allow(single_use_lifetimes)]
async fn build_server<A, D, T>(
    args: &CliArguments,
    database: Arc<D>,
    templates: Arc<T>,
) where
    A: 'static + Auth,
    for<'a, 'b, 'c> D: 'static
        + Database
        + FetchByUid<PreAuth, User<'a>, D>
        + FetchByUid<A, User<'a>, D>
        + FetchById<'b, A, PublicKey<'a>, D>
        + FetchById<'b, A, Entity<'a>, D>
        + Create<PreAuth, User<'a>, D>
        + Create<A, PublicKey<'a>, D>
        + Delete<A, PublicKey<'a>, D>
        + Save<PreAuth, User<'a>, D>
        + FetchAll<'b, A, PublicKey<'a>, PublicKeyFilter<'c>, D>,
    T: 'static + TemplateEngine,
{
    let oauth = if let AuthType::OAuth(ref oauth) = args.auth_type {
        match OAuth2::new::<D>(
            oauth.client_id.clone(),
            Some(oauth.client_secret.clone()),
            oauth.metadata_url.clone(),
            &format!(
                "http{}://localhost:8080/auth/callback",
                args.tls.as_ref().map_or("", |_| "s")
            ),
            oauth.user_scope.clone(),
            oauth.admin_scope.clone(),
            oauth.superuser_scope.clone(),
        )
        .await
        {
            Ok(oauth) => oauth,
            Err(err) => {
                log::error!("Unable to connect to oauth provider: {}", err);
                exit(1);
            }
        }
    } else {
        log::error!("Only OAuth2 supported");
        exit(1);
    };
    let view = Arc::new(BaseView {
        title: "SSH Key Authority".into(),
        style_mtime: get_filetime("style.css").await,
        js_mtime: get_filetime("extra.js").await,
        jsh_mtime: get_filetime("header.js").await,
        version: Some(env!("CARGO_PKG_VERSION")),
    });
    let data = Arc::new(BaseData {
        app_secret: args.app_secret,
        oauth,
    });
    let server = {
        #[cfg(not(feature = "hyper"))]
        {
            compile_error!("Server backend is required")
        }
        #[cfg(feature = "hyper")]
        {
            HyperServer::<A, D, T>::new(database, templates, view, data)
        }
    };
    log::info!("Starting server on {}:{}", args.listen, args.port);
    if let Err(err) = server
        .start_server(
            &args.listen,
            args.port,
            shutdown_signal(),
            routes::handler,
            args.tls.as_ref().map(|(c, k)| (c.as_ref(), k.as_ref())),
        )
        .await
    {
        log::error!("Unable to start server: {}", err);
        exit(1);
    }
    log::info!("Gracefull stop");
}

async fn get_filetime(file: &str) -> u64 {
    get_filetime_inner(file)
        .await
        .map_err(|err| {
            log::error!("Unable to get modification time of {}: {}", file, err);
        })
        .unwrap_or(0)
}

async fn get_filetime_inner(file: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let mtime = fs::metadata(format!("./static/{}", file))
        .await?
        .modified()?;
    Ok(mtime.duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
}

use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version,
    Arg, ArgMatches,
};
use std::process::exit;

const ARGS_LISTEN: &str = "listen";
const ARGS_LISTEN_ENV: &str = "LISTEN";
const ARGS_LISTEN_DEFAULT: &str = "::";
const ARGS_PORT: &str = "port";
const ARGS_PORT_ENV: &str = "PORT";
const ARGS_PORT_DEFAULT: &str = "8080";
const ARGS_PORT_DEFAULT_U16: u16 = 8080;
const ARGS_VERBOSE: &str = "verbose";
const ARGS_SILENT: &str = "silent";
const ARGS_VERSION: &str = "version";

const ARGS_DATABASE_HOST: &str = "db-host";
const ARGS_DATABASE_HOST_ENV: &str = "DB_HOST";
const ARGS_DATABASE_NAME: &str = "db-name";
const ARGS_DATABASE_NAME_ENV: &str = "DB_NAME";
const ARGS_DATABASE_USER: &str = "db-user";
const ARGS_DATABASE_USER_ENV: &str = "DB_USER";
const ARGS_DATABASE_PASS: &str = "db-pass";
const ARGS_DATABASE_PASS_ENV: &str = "DB_PASS";

const ARGS_APP_SECRET: &str = "app-secret";
const ARGS_APP_SECRET_ENV: &str = "APP_SECRET";
const ARGS_AUTH_TYPE: &str = "auth-type";
const ARGS_AUTH_TYPE_ENV: &str = "AUTH_TYPE";
const ARGS_AUTH_TYPE_DEFAULT: &str = "password";

const ARGS_OAUTH_CLIENT_ID: &str = "oauth-client-id";
const ARGS_OAUTH_CLIENT_ID_ENV: &str = "OAUTH_CLIENT_ID";
const ARGS_OAUTH_CLIENT_SECRET: &str = "oauth-client-secret";
const ARGS_OAUTH_CLIENT_SECRET_ENV: &str = "OAUTH_CLIENT_SECRET";
const ARGS_OAUTH_CLIENT_METADATA_URL: &str = "oauth-metadata_url";
const ARGS_OAUTH_CLIENT_METADATA_URL_ENV: &str = "OAUTH_METADATA_URL";
const ARGS_OAUTH_CLIENT_USER_SCOPE: &str = "oauth-user-scope";
const ARGS_OAUTH_CLIENT_USER_SCOPE_ENV: &str = "OAUTH_USER_SCOPE";
const ARGS_OAUTH_CLIENT_ADMIN_SCOPE: &str = "oauth-admin-scope";
const ARGS_OAUTH_CLIENT_ADMIN_SCOPE_ENV: &str = "OAUTH_ADMIN_SCOPE";
const ARGS_OAUTH_CLIENT_SUPERUSER_SCOPE: &str = "oauth-superuser-scope";
const ARGS_OAUTH_CLIENT_SUPERUSER_SCOPE_ENV: &str = "OAUTH_SUPERUSER_SCOPE";

#[derive(Debug, Clone)]
pub struct CliArguments {
    pub print_version: bool,

    pub listen: String,
    pub port: u16,
    pub log_level: Option<log::Level>,
    pub auth_type: AuthType,
    pub app_secret: [u8; 32],

    pub db_host: Vec<String>,
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
}

#[derive(Debug, Clone)]
pub enum AuthType {
    Password,
    Ldap,
    OAuth(OAuth),
}

#[derive(Debug, Clone)]
pub struct OAuth {
    pub client_id: String,
    pub client_secret: String,
    pub metadata_url: String,
    pub user_scope: String,
    pub admin_scope: String,
    pub superuser_scope: String,
}

#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn get_arguments() -> CliArguments {
    let matches = get_cli_config();
    let print_version = matches.is_present(ARGS_VERSION);
    let listen = matches
        .value_of(ARGS_LISTEN)
        .unwrap_or(ARGS_LISTEN_DEFAULT)
        .into();
    let port: u16 = matches
        .value_of(ARGS_PORT)
        .and_then(|port| port.parse().ok())
        .unwrap_or(ARGS_PORT_DEFAULT_U16);
    let log_level = match (
        matches.occurrences_of(ARGS_SILENT),
        matches.occurrences_of(ARGS_VERBOSE),
    ) {
        (0, 1) => Some(log::Level::Debug),
        (0, v) if v > 1 => Some(log::Level::Trace),
        (1, 0) => Some(log::Level::Warn),
        (2, 0) => Some(log::Level::Error),
        (s, 0) if s > 2 => None,
        _ => Some(log::Level::Info),
    };

    let db_host = if let Some(v) = matches.values_of(ARGS_DATABASE_HOST) {
        v.map(|s| s.into()).collect()
    } else {
        eprintln!("Database Host is required");
        exit(1);
    };
    let db_name = if let Some(v) = matches.value_of(ARGS_DATABASE_NAME) {
        v.into()
    } else {
        eprintln!("Database Name is required");
        exit(1);
    };
    let db_user = if let Some(v) = matches.value_of(ARGS_DATABASE_USER) {
        v.into()
    } else {
        eprintln!("Database User is required");
        exit(1);
    };
    let db_pass = if let Some(v) = matches.value_of(ARGS_DATABASE_PASS) {
        v.into()
    } else {
        eprintln!("Database Password is required");
        exit(1);
    };
    let app_secret = if let Some(v) = matches
        .value_of(ARGS_APP_SECRET)
        .map(str::as_bytes)
        .and_then(|v| v.get(..32))
    {
        let mut out: [u8; 32] = Default::default();
        out.copy_from_slice(v);
        out
    } else {
        eprintln!("Application Secret missing or small then 32 bytes");
        exit(1);
    };

    let auth_type = match matches.value_of(ARGS_AUTH_TYPE) {
        Some("password") => AuthType::Password,
        Some("ldap") => AuthType::Ldap,
        Some("oauth") => {
            let client_id = if let Some(v) = matches.value_of(ARGS_OAUTH_CLIENT_ID) {
                v.into()
            } else {
                eprintln!("OAuth client id missing");
                exit(1);
            };
            let client_secret =
                if let Some(v) = matches.value_of(ARGS_OAUTH_CLIENT_SECRET) {
                    v.into()
                } else {
                    eprintln!("OAuth client secret missing");
                    exit(1);
                };
            let metadata_url =
                if let Some(v) = matches.value_of(ARGS_OAUTH_CLIENT_METADATA_URL) {
                    v.into()
                } else {
                    eprintln!("OAuth metadata url missing");
                    exit(1);
                };
            let user_scope =
                if let Some(v) = matches.value_of(ARGS_OAUTH_CLIENT_USER_SCOPE) {
                    v.into()
                } else {
                    eprintln!("OAuth user scope missing");
                    exit(1);
                };
            let admin_scope =
                if let Some(v) = matches.value_of(ARGS_OAUTH_CLIENT_ADMIN_SCOPE) {
                    v.into()
                } else {
                    eprintln!("OAuth admin scope missing");
                    exit(1);
                };
            let superuser_scope = if let Some(v) =
                matches.value_of(ARGS_OAUTH_CLIENT_SUPERUSER_SCOPE)
            {
                v.into()
            } else {
                eprintln!("OAuth super user scope missing");
                exit(1);
            };

            AuthType::OAuth(OAuth {
                client_id,
                client_secret,
                metadata_url,
                user_scope,
                admin_scope,
                superuser_scope,
            })
        }
        _ => {
            eprintln!("Authorisation Type is invalid or missing");
            exit(1);
        }
    };

    CliArguments {
        print_version,

        listen,
        port,
        log_level,
        auth_type,
        app_secret,

        db_host,
        db_name,
        db_user,
        db_pass,
    }
}

#[allow(clippy::too_many_lines)]
fn get_cli_config<'a>() -> ArgMatches<'a> {
    app_from_crate!()
        .arg(
            Arg::with_name(ARGS_VERSION)
                .long(ARGS_VERSION)
                .help("Print the version of the program")
                .takes_value(false),
        )
        .arg(
            Arg::with_name(ARGS_LISTEN)
                .short("l")
                .long(ARGS_LISTEN)
                .env(ARGS_LISTEN_ENV)
                .value_name("hostname/ip")
                .help("Set the listening ip/hostname")
                .default_value(ARGS_LISTEN_DEFAULT)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ARGS_PORT)
                .short("p")
                .long(ARGS_PORT)
                .env(ARGS_PORT_ENV)
                .value_name("port")
                .help("Set the port to bind to")
                .default_value(ARGS_PORT_DEFAULT)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ARGS_VERBOSE)
                .short("v")
                .long(ARGS_VERBOSE)
                .conflicts_with(ARGS_SILENT)
                .multiple(true)
                .help("Increase verbosity. Once for debug, twice for trace")
        )
        .arg(
            Arg::with_name(ARGS_SILENT)
                .short("s")
                .long(ARGS_SILENT)
                .conflicts_with(ARGS_VERBOSE)
                .multiple(true)
                .help("Decrease verbosity. Once for warning, twice for error, thrice for none")
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_HOST)
                .long(ARGS_DATABASE_HOST)
                .env(ARGS_DATABASE_HOST_ENV)
                .value_name("<hostname/ip>:<port>")
                .help("Database Hostname or Ip with port")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
                .required(true),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_NAME)
                .long(ARGS_DATABASE_NAME)
                .env(ARGS_DATABASE_NAME_ENV)
                .value_name("name")
                .help("Database Name")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_USER)
                .long(ARGS_DATABASE_USER)
                .env(ARGS_DATABASE_USER_ENV)
                .value_name("username")
                .help("Database Username")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_PASS)
                .long(ARGS_DATABASE_PASS)
                .env(ARGS_DATABASE_PASS_ENV)
                .value_name("password")
                .help("Database Password")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(ARGS_APP_SECRET)
                .long(ARGS_APP_SECRET)
                .env(ARGS_APP_SECRET_ENV)
                .value_name("secret")
                .help("App Secret")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(ARGS_AUTH_TYPE)
                .long(ARGS_AUTH_TYPE)
                .env(ARGS_AUTH_TYPE_ENV)
                .value_name("type")
                .help("Authentication Backend")
                .takes_value(true)
                .default_value(ARGS_AUTH_TYPE_DEFAULT)
                .possible_values(&["password", "ldap", "oauth"]),
        )
        .arg(
            Arg::with_name(ARGS_OAUTH_CLIENT_ID)
                .long(ARGS_OAUTH_CLIENT_ID)
                .env(ARGS_OAUTH_CLIENT_ID_ENV)
                .value_name("id")
                .help("OAuth Client Id")
                .takes_value(true)
                .required_if(ARGS_AUTH_TYPE, "oauth"),
        )
        .arg(
            Arg::with_name(ARGS_OAUTH_CLIENT_SECRET)
                .long(ARGS_OAUTH_CLIENT_SECRET)
                .env(ARGS_OAUTH_CLIENT_SECRET_ENV)
                .value_name("secret")
                .help("OAuth Client Secret")
                .takes_value(true)
                .required_if(ARGS_AUTH_TYPE, "oauth"),
        )
        .arg(
            Arg::with_name(ARGS_OAUTH_CLIENT_METADATA_URL)
                .long(ARGS_OAUTH_CLIENT_METADATA_URL)
                .env(ARGS_OAUTH_CLIENT_METADATA_URL_ENV)
                .value_name("url")
                .help("OAuth Metadata Url")
                .takes_value(true)
                .required_if(ARGS_AUTH_TYPE, "oauth"),
        )
        .arg(
            Arg::with_name(ARGS_OAUTH_CLIENT_USER_SCOPE)
                .long(ARGS_OAUTH_CLIENT_USER_SCOPE)
                .env(ARGS_OAUTH_CLIENT_USER_SCOPE_ENV)
                .value_name("scope")
                .help("OAuth User Scope")
                .takes_value(true)
                .required_if(ARGS_AUTH_TYPE, "oauth"),
        )
        .arg(
            Arg::with_name(ARGS_OAUTH_CLIENT_ADMIN_SCOPE)
                .long(ARGS_OAUTH_CLIENT_ADMIN_SCOPE)
                .env(ARGS_OAUTH_CLIENT_ADMIN_SCOPE_ENV)
                .value_name("scope")
                .help("OAuth Admin Scope")
                .takes_value(true)
                .required_if(ARGS_AUTH_TYPE, "oauth"),
        )
        .arg(
            Arg::with_name(ARGS_OAUTH_CLIENT_SUPERUSER_SCOPE)
                .long(ARGS_OAUTH_CLIENT_SUPERUSER_SCOPE)
                .env(ARGS_OAUTH_CLIENT_SUPERUSER_SCOPE_ENV)
                .value_name("scope")
                .help("OAuth Superuser Scope")
                .takes_value(true)
                .required_if(ARGS_AUTH_TYPE, "oauth"),
        )
        .get_matches()
}

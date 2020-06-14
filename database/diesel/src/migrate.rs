use crate::{MysqlConnection, PgConnection};
use diesel::{
    backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax},
    connection::AnsiTransactionManager,
    r2d2::{ConnectionManager, PooledConnection},
    Connection,
};

#[allow(clippy::unreachable)]
mod mysql {
    pub(crate) use embedded_migrations::run_with_output;
    embed_migrations!("migrations.mysql");
}

#[allow(clippy::unreachable)]
mod postgres {
    pub(crate) use embedded_migrations::run_with_output;
    embed_migrations!("migrations.postgres");
}

pub trait Migrate {
    fn migrate<B>(
        conn: &PooledConnection<ConnectionManager<Self>>,
    ) -> Result<(), diesel_migrations::RunMigrationsError>
    where
        B: 'static
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + SupportsDefaultKeyword,
        Self: 'static
            + Connection<Backend = B, TransactionManager = AnsiTransactionManager>;
}

impl Migrate for PgConnection {
    fn migrate<B>(
        conn: &PooledConnection<ConnectionManager<Self>>,
    ) -> Result<(), diesel_migrations::RunMigrationsError>
    where
        B: 'static
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + SupportsDefaultKeyword,
        Self: 'static
            + Connection<Backend = B, TransactionManager = AnsiTransactionManager>,
    {
        postgres::run_with_output(conn, &mut std::io::stdout())
    }
}

impl Migrate for MysqlConnection {
    fn migrate<B>(
        conn: &PooledConnection<ConnectionManager<Self>>,
    ) -> Result<(), diesel_migrations::RunMigrationsError>
    where
        B: 'static
            + Backend<RawValue = [u8]>
            + UsesAnsiSavepointSyntax
            + SupportsDefaultKeyword,
        Self: 'static
            + Connection<Backend = B, TransactionManager = AnsiTransactionManager>,
    {
        mysql::run_with_output(conn, &mut std::io::stdout())
    }
}

use core_common::database::{tests, Database};
use database_diesel::{DieselDB, PgConnection};
use std::sync::Arc;

#[tokio::test(core_threads = 2)]
async fn database_tests() {
    let url = vec![format!(
        "postgres://postgres:@localhost:5432/skm"
    )];
    let db = Arc::new(
        DieselDB::<PgConnection>::new(url.into_iter())
            .expect("Unable to connect to postgres test database"),
    );
    db.migrate().await.expect("Unable to migrate database");
    for (name, test_fn) in tests::get_tests() {
        println!("Testing: {}", name);
        test_fn(db.clone()).await;
    }
}

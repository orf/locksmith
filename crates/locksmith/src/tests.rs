use crate::TableObject;
use crate::introspection::Introspector;
use crate::locker::Locker;
use anyhow::Context;
use testcontainers_modules::postgres;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

#[must_use = "Postgres container must be used"]
pub async fn start_test_postgres() -> (ContainerAsync<Postgres>, String) {
    let postgres_tag = std::env::var("TEST_POSTGRES_TAG").unwrap_or("14-alpine".to_string());
    let container = postgres::Postgres::default()
        .with_init_sql(include_bytes!("../tests/test_schema.sql").to_vec())
        .with_user("user")
        .with_password("password")
        .with_tag(&postgres_tag)
        .start()
        .await
        .context("Starting Postgres container")
        .unwrap();
    let host_ip = container.get_host().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    (
        container,
        format!("postgresql://user:password@{host_ip}:{host_port}/postgres"),
    )
}

/// Create a locker, and lock the given tables.
pub async fn lock_tables(dsn: &str, tables: impl IntoIterator<Item = &str>) -> Locker {
    let mut locker = Locker::new(dsn).await.unwrap();
    let names: Vec<_> = tables.into_iter().map(TableObject::from).collect();
    locker.lock_tables(&names).await.unwrap();
    locker
}

/// Check that a given table exists in the test database.
pub async fn table_exists(dsn: &str, table: impl Into<TableObject>) -> bool {
    let table = table.into();
    let mut introspector = Introspector::new(dsn).await.unwrap();
    let tables = introspector.list_tables().await.unwrap();
    tables.into_iter().any(|t| t == table)
}

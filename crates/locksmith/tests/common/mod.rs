mod test_case;

use anyhow::Context;
pub use test_case::*;
use testcontainers_modules::postgres;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

#[must_use = "Postgres container must be used"]
pub async fn start_integration_test_postgres() -> (ContainerAsync<Postgres>, String) {
    let postgres_tag = std::env::var("TEST_POSTGRES_TAG").unwrap_or("14-alpine".to_string());

    let container = postgres::Postgres::default()
        .with_init_sql(include_bytes!("../test_schema.sql").to_vec())
        .with_user("user")
        .with_password("password")
        .with_tag(postgres_tag)
        .start()
        .await
        .context("Starting Postgres containe")
        .unwrap();
    let host_ip = container.get_host().await.unwrap();
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();

    (
        container,
        format!("postgresql://user:password@{host_ip}:{host_port}/postgres"),
    )
}

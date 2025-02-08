use crate::objects::DBObject;
use crate::{ColumnObject, IndexObject, TableObject};
use anyhow::Context;
use sqlx::{query, query_as, Connection, PgConnection};
use std::collections::{HashMap, HashSet};

/// An [Introspector] provides various introspection functions for a given Postgres database.
/// Namely, it provides methods to list objects via [Introspector::list_objects], and to list
/// file nodes for objects via [Introspector::list_object_file_nodes].
pub struct Introspector {
    conn: PgConnection,
}

impl Introspector {
    /// Construct a new [Introspector] with a connection to the Postgres database at `dsn`.
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        let mut conn = PgConnection::connect(dsn)
            .await
            .context("Creating connection")?;
        conn.ping().await.context("Pinging postgres")?;
        Ok(Self { conn })
    }

    /// ## List all objects in the database
    /// This returns the set of all tables, columns and indexes in the database.
    pub async fn list_objects(&mut self) -> anyhow::Result<HashSet<DBObject>> {
        let tables = self.list_tables().await?.into_iter().map(DBObject::from);
        let columns = self.list_columns().await?.into_iter().map(DBObject::from);
        let indexes = self.list_indexes().await?.into_iter().map(DBObject::from);
        Ok(tables.chain(columns).chain(indexes).collect())
    }

    /// List the file nodes of all objects in the database.
    ///
    /// A file node is a unique identifier for a table's underlying storage file, which
    /// is guaranteed to change if the table is rewritten (even if the table is empty).
    /// This uses the [pg_relation_filenode](https://pgpedia.info/p/pg_relation_filenode.html)
    /// function to get the file node for each table.
    ///
    /// **Note**: Currently this only lists table file nodes, and not indexes or other objects.
    pub async fn list_object_file_nodes(&mut self) -> anyhow::Result<HashMap<DBObject, i32>> {
        query!(
            r#"
            SELECT table_name as "table!", pg_relation_filenode(table_name::text)::int as "file_node!"
            FROM information_schema.tables
            WHERE table_schema = "current_schema"()
              AND table_catalog = current_database()
            order by table_name;"#
        )
            .fetch_all(&mut self.conn)
            .await.context("Query error while listing table file nodes")
            .map(|r| {
                r.into_iter().map(|r| {
                    let table = DBObject::Table(TableObject {
                        name: r.table,
                    });
                    (table, r.file_node)
                }).collect()
            })
    }

    /// ## List all tables in the database.
    /// This uses the [information_schema.tables](https://www.postgresql.org/docs/current/infoschema-tables.html)
    /// view to retrieve tables in the current schema.
    pub async fn list_tables(&mut self) -> anyhow::Result<Vec<TableObject>> {
        query_as!(
            TableObject,
            r#"
            SELECT table_name as "name!"
            FROM information_schema.tables
            WHERE table_schema = "current_schema"()
              AND table_catalog = current_database()
            order by table_name;"#
        )
        .fetch_all(&mut self.conn)
        .await
        .context("Query error while listing tables")
    }

    /// ## List columns in the database
    /// This uses the [information_schema.columns](https://www.postgresql.org/docs/current/infoschema-columns.html)
    /// view to retrieve columns in the current schema.
    pub async fn list_columns(&mut self) -> anyhow::Result<Vec<ColumnObject>> {
        query_as!(
            ColumnObject,
            r#"
            SELECT table_name as "table!", column_name as "name!", data_type as "data_type!"
            FROM information_schema.columns
            WHERE table_schema = "current_schema"()
              AND table_catalog = current_database()
            order by table_name, column_name;
            "#
        )
        .fetch_all(&mut self.conn)
        .await
        .context("Query error while listing columns")
    }

    /// ## List indexes in the database
    /// This uses the [pg_stat_all_indexes](https://pgpedia.info/p/pg_stat_all_indexes.html) view
    /// to retrieve indexes in the current schema.
    pub async fn list_indexes(&mut self) -> anyhow::Result<Vec<IndexObject>> {
        query_as!(
            IndexObject,
            r#"
            SELECT relname as "table!", indexrelname as "name!"
            FROM pg_stat_all_indexes
            WHERE schemaname = "current_schema"()
            order by 1, 2;"#
        )
        .fetch_all(&mut self.conn)
        .await
        .context("Query error while listing indexes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::start_test_postgres;

    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    async fn test_list_tables() {
        let (_container, dsn) = start_test_postgres().await;
        let mut target = Introspector::new(&dsn).await.unwrap();
        let tables = target.list_tables().await.unwrap();
        assert_eq!(tables, vec!["customers".into(), "orders".into()]);
    }
    #[traced_test]
    #[tokio::test]
    async fn test_list_columns() {
        let (_container, dsn) = start_test_postgres().await;
        let mut target = Introspector::new(&dsn).await.unwrap();
        let columns = target.list_columns().await.unwrap();
        assert_eq!(
            columns,
            vec![
                ColumnObject {
                    table: "customers".into(),
                    name: "id".to_string(),
                    data_type: "integer".to_string()
                },
                ColumnObject {
                    table: "customers".into(),
                    name: "name".to_string(),
                    data_type: "text".to_string()
                },
                ColumnObject {
                    table: "orders".into(),
                    name: "customer_id".to_string(),
                    data_type: "integer".to_string()
                },
                ColumnObject {
                    table: "orders".into(),
                    name: "id".to_string(),
                    data_type: "integer".to_string()
                },
                ColumnObject {
                    table: "orders".into(),
                    name: "price".to_string(),
                    data_type: "numeric".to_string()
                }
            ]
        );
    }
    #[traced_test]
    #[tokio::test]
    async fn test_list_indexes() {
        let (_container, dsn) = start_test_postgres().await;
        let mut target = Introspector::new(&dsn).await.unwrap();

        let indexes = target.list_indexes().await.unwrap();
        assert_eq!(
            indexes,
            vec![
                IndexObject {
                    table: "customers".into(),
                    name: "customers_pkey".to_string()
                },
                IndexObject {
                    table: "orders".into(),
                    name: "orders_pkey".to_string()
                },
                IndexObject {
                    table: "orders".into(),
                    name: "orders_price_idx".to_string()
                }
            ]
        )
    }
}

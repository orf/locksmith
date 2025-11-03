use crate::ConnectionID;
use crate::objects::{TableLock, TableObject};
use anyhow::Context;
use sqlx::{Connection, Executor, PgConnection, query, query_as};
use tracing::trace;

/// A [Locker] manages Postgres table locks in a transaction.
/// It provides two methods: [Locker::lock_tables] and [Locker::list_connection_locks].
pub struct Locker {
    conn: PgConnection,
}

impl Locker {
    /// Construct a new [Locker] with a connection to the Postgres database at `dsn`.
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        let mut conn = PgConnection::connect(dsn)
            .await
            .context("Creating connection")?;
        conn.ping().await.context("Pinging postgres")?;
        query!("BEGIN;")
            .execute(&mut conn)
            .await
            .context("Starting transaction")?;
        Ok(Self { conn })
    }

    /// Lock a set of tables, by name, in the database with `ACCESS EXCLUSIVE MODE`.
    pub async fn lock_tables(
        &mut self,
        tables: impl IntoIterator<Item = &TableObject>,
    ) -> anyhow::Result<()> {
        for table in tables {
            trace!(?table, "Locking table");
            let lock_query = format!("LOCK TABLE \"{}\" IN ACCESS EXCLUSIVE MODE;", table.name);
            self.conn
                .execute(lock_query.as_str())
                .await
                .with_context(|| format!("Query error while locking {table:?}"))?;
        }
        Ok(())
    }

    /// List the locks held by a given connection ID. This returns a list of [TableLock]s, which
    /// contain the table name and the lock mode.
    pub async fn list_connection_locks(
        &mut self,
        connection_id: ConnectionID,
    ) -> anyhow::Result<Vec<TableLock>> {
        query_as!(
            TableLock,
            r#"
            select relation::regclass::text as "table!", mode as "lock!"
            from pg_locks l
            join pg_class c ON l.relation = c.oid
            join pg_namespace n ON c.relnamespace = n.oid
            WHERE l.pid = $1
              AND n.nspname = current_schema()
              AND c.relkind IN ('r', 'p')
              AND l.locktype = 'relation'
              AND l.mode IS NOT NULL
              AND database = (SELECT oid FROM pg_database WHERE datname = current_database());
            "#,
            connection_id.0
        )
        .fetch_all(&mut self.conn)
        .await
        .with_context(|| {
            format!("Query error while listing connection locks for {connection_id:?}")
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::start_test_postgres;

    use crate::locker::Locker;
    use crate::{ConnectionID, Lock, TableLock};
    use sqlx::query_scalar;
    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    async fn test_list_connection_locks() {
        let (_container, dsn) = start_test_postgres().await;
        let mut locker = Locker::new(&dsn).await.unwrap();
        let connection_id = query_scalar!(r#"select pg_backend_pid() as "pid!""#)
            .fetch_one(&mut locker.conn)
            .await
            .map(ConnectionID)
            .unwrap();

        let orders_table = "orders".into();
        locker.lock_tables([&orders_table]).await.unwrap();
        let locks = locker.list_connection_locks(connection_id).await.unwrap();
        assert_eq!(
            locks,
            vec![TableLock {
                table: orders_table,
                lock: Lock::AccessExclusiveLock,
            }]
        )
    }
}

use crate::ConnectionID;
use anyhow::{bail, Context};
use std::pin::pin;
use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::{AsyncMessage, Client, Connection, NoTls, Socket};
use tracing::{debug, trace};

/// A [StatementExecutor] is a client for executing statements on a Postgres database.
/// It provides a method, [StatementExecutor::check_statement_for_locks], which executes a statement
/// and returns true if the statement was blocked by a lock.
pub struct StatementExecutor {
    client: Client,
    connection: Connection<Socket, NoTlsStream>,
    connection_id: ConnectionID,
}

impl StatementExecutor {
    /// Create a new [StatementExecutor] with a connection to the Postgres database at `dsn`.
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        // We have to use [tokio-postgres](https://crates.io/crates/tokio-postgres) for this, because
        // sqlx does not give us the ability to receive NOTICE messages from the server.
        let (client, mut connection) = tokio_postgres::connect(dsn, NoTls)
            .await
            .context("Creating connection")?;

        // There are some peculiarities when using tokio-postgres compared to sqlx, namely that the
        // client and the connection are separate and need to be driven separately.
        // We do this by using `tokio::select!` to drive them both in parallel.
        // If the connection future ever finishes before the client future then we bail out.
        let connection_id = tokio::select! {
            row = client.query_one("SELECT pg_backend_pid()", &[]) => {
                let row = row.context("Query error while retrieving connection ID")?;
                ConnectionID(row.get(0))
            },
            _ = &mut connection => {
                bail!("Connection unexpectedly finished: retrieving connection ID")
            }
        };

        // These statements are necessary to enable logging of lock waits. See
        // `detect_if_statement_blocks` for the implementation details.
        const SETUP_STATEMENTS: &str = r#"
            BEGIN;
            SET log_lock_waits=true;
            SET deadlock_timeout='1ms';
            SET client_min_messages='log';
        "#;
        tokio::select! {
            setup_result = client.batch_execute(SETUP_STATEMENTS) => {
                setup_result.context("Query error while executing setup statement")?
            },
            _ = &mut connection => bail!("Connection unexpectedly finished: executing setup statement")
        }

        Ok(Self {
            client,
            connection,
            connection_id,
        })
    }

    /// Attempt to terminate the backend connection. This is best-effort.
    pub async fn attempt_termination(&self) {
        self.client.cancel_token().cancel_query(NoTls).await.ok();
    }

    /// Get the connection ID for this [StatementExecutor].
    pub fn connection_id(&self) -> ConnectionID {
        self.connection_id
    }

    /// the statement was blocked by a lock.
    ///
    /// Postgres takes some locks when `COMMIT`ing a transaction, so this method will also attempt
    /// to commit the transaction after executing the statement.
    ///
    /// If this method returns `true`, the connection and transaction will still be open and
    /// blocked until the [StatementExecutor] is dropped. This allows the caller to inspect
    /// the locks taken by the statement via another connection.
    #[tracing::instrument(skip(self, statement))]
    pub async fn check_statement_for_locks(&mut self, statement: &str) -> anyhow::Result<bool> {
        for to_execute in [statement, "COMMIT;"] {
            let is_blocked = self.detect_if_statement_blocks(to_execute).await?;
            if is_blocked {
                return Ok(true);
            }
        }
        debug!("Statement succeeded");
        Ok(false)
    }

    /// Detect if a statement is blocked by a lock. This
    ///
    /// The implementation of this method relies on a Postgres server feature called
    /// [log_lock_waits](https://pgpedia.info/l/log_lock_waits.html).
    /// In [StatementExecutor::new] we enable this option for our transaction, which causes the
    /// server to send us a NOTICE message if the statement is blocked by a lock.
    /// Server messages are delivered asynchronously to the [Connection], so we poll both the
    /// [Connection] and the [Client] in parallel to drive the query and receive messages.
    ///
    /// If the server delivers us a NOTICE message that the statement is blocked, we stop polling
    /// and return `true`. In this state the connection is still open and the transaction is still
    /// intact.
    ///
    /// When the [StatementExecutor] is dropped the connection will be closed and the transaction
    /// will be aborted.
    #[tracing::instrument(skip(self, statement))]
    async fn detect_if_statement_blocks(&mut self, statement: &str) -> anyhow::Result<bool> {
        let mut poll_message_future = std::future::poll_fn(|cx| self.connection.poll_message(cx));
        let mut execute_future = pin!(self.client.batch_execute(statement));

        // Drive both the query future and the message future in parallel.
        loop {
            tokio::select! {
                res = &mut execute_future => {
                    res.context("Failed to execute statement")?;
                    debug!("Statement executed successfully");
                    return Ok(false)
                },
                async_message = &mut poll_message_future => {
                    match async_message {
                        None => {
                            bail!("Connection unexpectedly finished: executing statement")
                        }
                        Some(msg) => {
                            let async_message = msg.context("Reading message from server")?;
                            trace!(?async_message, "Received message");

                            if let AsyncMessage::Notice(msg) = async_message {
                                // Postgres doesn't provide a structured code for notice messages,
                                // so the best we can do is check the message text.
                                let message = msg.message();
                                if message.contains("still waiting for") {
                                    debug!("Statement blocked");
                                    return Ok(true)
                                }
                            }

                        }
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::executor::StatementExecutor;
    use crate::tests::{lock_tables, start_test_postgres, table_exists};
    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    async fn test_get_connection_id() {
        let (_container, dsn) = start_test_postgres().await;
        let executor = StatementExecutor::new(&dsn).await.unwrap();
        assert!(executor.connection_id.0 > 0)
    }
    #[traced_test]
    #[tokio::test]
    async fn test_check_statement_does_not_commit_if_blocked() {
        let (_container, dsn) = start_test_postgres().await;
        let mut executor = StatementExecutor::new(&dsn).await.unwrap();
        let _locker = lock_tables(&dsn, ["orders"]).await;
        let is_blocked = executor
            .check_statement_for_locks("drop table orders;")
            .await
            .unwrap();
        assert!(is_blocked);
        assert!(table_exists(&dsn, "orders").await);
    }

    #[traced_test]
    #[tokio::test]
    async fn test_check_statement_commits() {
        let (_container, dsn) = start_test_postgres().await;
        let mut executor = StatementExecutor::new(&dsn).await.unwrap();
        let is_blocked = executor
            .check_statement_for_locks("drop table orders;")
            .await
            .unwrap();
        assert!(!is_blocked);
        assert!(!table_exists(&dsn, "orders").await);
    }

    #[traced_test]
    #[tokio::test]
    async fn test_check_statement_with_invalid_sql() {
        let (_container, dsn) = start_test_postgres().await;
        let mut executor = StatementExecutor::new(&dsn).await.unwrap();
        assert!(executor.check_statement_for_locks("foobar").await.is_err());
    }

    #[traced_test]
    #[tokio::test]
    async fn test_is_statement_blocked() {
        let (_container, dsn) = start_test_postgres().await;

        assert!(!StatementExecutor::new(&dsn)
            .await
            .unwrap()
            .detect_if_statement_blocks("select * from customers")
            .await
            .unwrap());

        let _locker = lock_tables(&dsn, ["customers"]).await;

        assert!(StatementExecutor::new(&dsn)
            .await
            .unwrap()
            .detect_if_statement_blocks("select * from customers")
            .await
            .unwrap());
        drop(_locker)
    }
}

use crate::executor::StatementExecutor;
use crate::introspection::Introspector;
use crate::locker::Locker;
use crate::{DBObject, TableLock};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, info};

/// [InspectedStatement] is a struct that contains the side effects of inspecting a SQL statement.
/// It includes the objects that were added, removed, locked, and rewritten by the statement.
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct InspectedStatement {
    pub added_objects: HashSet<DBObject>,
    pub removed_objects: HashSet<DBObject>,
    pub locks: HashSet<TableLock>,
    pub rewrites: HashSet<DBObject>,
}

pub struct QueryOracle {
    dsn: String,
}

impl QueryOracle {
    pub fn new(dsn: impl ToString) -> Self {
        Self {
            dsn: dsn.to_string(),
        }
    }

    /// Inspect a statement and return an [InspectedStatement], containing a summary of the
    /// side effects of the statement.
    ///
    /// It is non-trivial to understand exactly what impacts a given SQL statement has on a
    /// Postgres database. This is because the actual effects of a statement depend almost entirely
    /// on the *current* state of the database (including the version), which is not present in the
    /// statement itself. The statement is completely devoid of any context, and the context is
    /// the only thing that matters.
    ///
    /// For example, an `ALTER COLUMN` statement on a large table will either:
    /// - Require a lengthy rewrite of the entire table, during which no other operations can
    ///   be performed on the table *at all*, resulting in potential service disruption
    /// - Be an instantaneous metadata-only operation that does not block any other operations
    ///   on the table and requires no data to be rewritten.
    ///
    /// Which of these two outcomes occurs depends on the current type of the column and the
    /// type we are altering:
    /// - if the column is "binary compatible" with the new type then the operation is metadata-only
    /// - otherwise it requires a full rewrite.
    ///
    /// Which types are binary compatible is often an implementation specific detail of the  Postgres
    /// version in use, or in the case of custom data types the implementation of the type itself.
    ///
    /// In order to understand the impact of a statement, we need to execute it in a controlled
    /// environment and observe the side effects. This is what this method does: it provides a
    /// "query oracle" that executes the statements and observes the side effects. This
    /// relies entirely on Postgres' introspection capabilities as well as it's ability to execute
    /// DDL statements (such as `ALTER TABLE`) in a transaction, meaning it is generic across all
    /// supported Postgres versions, data types and extensions.
    ///
    /// # Detecting locks
    ///
    /// Postgres provides no built-in way to detect which tables a statement is trying to access,
    /// or what locks are required. In order to work around this, we force a controlled lock conflict
    /// using multiple transactions in a way that allows us to view the blocking locks. This gives
    /// us information on the specific tables and lock types that the query requires.
    ///
    /// Specifically, we use the following algorithm:
    /// 1. A new "locker" connection is created, and a transaction is begun
    /// 2. The locker connection locks all tables present in the database with an ACCESS EXCLUSIVE
    ///    lock. This prevents all other transactions from performing any operations on the tables.
    /// 3. A new "executor" connection is created, and a transaction is begun
    /// 4. The executor executes the statement we are inspecting. This will cause the transaction
    ///    to be blocked by the "locker" connection.
    /// 5. When the executor detects that the statement is blocked it yields control back to the
    ///    oracle, whilst keeping the blocked transaction open in it's blocked state.
    /// 6. The "locker" connection then lists all locks that are blocking the executor's transaction,
    ///    which reveals the tables that the statement is trying to access. We now have an
    ///    initial set of tables that the statement is accessing.
    /// 7. We close the "executor" connection _without_ committing the transaction, and close the
    ///    "locker" connection after. This releases the locks on the tables.
    /// 8. We repeat steps 1 to 7, but this time only locking the tables that have *not* been
    ///    locked in the previous iterations. This allows us to detect new locks that the statement
    ///    is trying to acquire.
    /// 9. We repeat this process until the statement is no longer blocked by any locks and executes
    ///    successfully.
    ///
    /// Once this process is completed, we have observed the complete set of locks that the statement
    /// requires in order to execute.
    ///
    /// One caveat of this approach is that it is only really suitable to be run in a controlled,
    /// isolated instance of Postgres (i.e. not a production environment), and so it requires the
    /// schema to completely match the production schema in order to give accurate results.
    ///
    /// # Detecting other effects
    ///
    /// In addition to detecting locks, we also need to detect the other side effects of the
    /// statement. These include the specific objects that are added, removed, or modified, as well
    /// as any tables that are rewritten.
    ///
    /// The implementation of this is much simpler: we simply introspect relevant database objects
    /// *before* and *after* the statement is executed, and compare the two sets of objects.
    pub async fn inspect_statement(
        &mut self,
        statement: &str,
    ) -> anyhow::Result<InspectedStatement> {
        let mut all_detected_locks: HashSet<TableLock> = HashSet::new();

        // Create an inspector, and list the initial objects in the database.
        let mut introspector = Introspector::new(&self.dsn)
            .await
            .context("Creating introspector")?;
        let initial_objects = introspector
            .list_objects()
            .await
            .context("Listing initial objects")?;
        let initial_table_file_nodes = introspector
            .list_object_file_nodes()
            .await
            .context("Listing object file nodes")?;

        // Retrieve the set of initial tables
        let all_tables: HashSet<_> = initial_objects
            .iter()
            .filter_map(|obj| match obj {
                DBObject::Table(table) => Some(table),
                _ => None,
            })
            .collect();

        // This implements the main loop of the algorithm.
        // Here we repeatedly lock tables and execute the statement until it is no longer blocked.
        loop {
            // Create a set of tables to lock that we have not yet observed requiring a lock.
            let known_locked_table: HashSet<_> =
                all_detected_locks.iter().map(|t| &t.table).collect();
            let tables_to_lock = all_tables.difference(&known_locked_table);

            // Create a new "locker" connection and lock those tables
            let mut locker = Locker::new(&self.dsn).await.context("Creating locker")?;
            locker
                .lock_tables(tables_to_lock.into_iter().copied())
                .await?;

            // Create a statement executor and retrieve its connection ID
            let mut executor = StatementExecutor::new(&self.dsn)
                .await
                .context("Creating executor")?;
            let connection_id = executor.connection_id();
            debug!("Executor created with connection ID {connection_id:?}");

            // Execute the statement, returning true if the statement has been blocked by
            // a lock taken by the locker connection.
            // If the statement successfully executed without any blocking, we can break the loop
            let is_blocked = executor.check_statement_for_locks(statement).await?;
            if !is_blocked {
                info!("Statement executed successfully");
                break;
            }

            // List all locks that are taken by the executor connection and add them to our
            // set of seen locks.
            let new_locks: Vec<_> = locker
                .list_connection_locks(connection_id)
                .await
                .context("Listing connection locks")?;
            debug!(?new_locks, "Detected {} new locks", new_locks.len());
            all_detected_locks.extend(new_locks);

            // Attempt to terminate the executor connection. Not required, but prevents some
            // spurious issues with Postgres 13 and connection limits.
            executor.attempt_termination().await;
        }

        // Take a snapshot of the objects in the database after the statement has executed
        let new_objects: HashSet<_> = introspector
            .list_objects()
            .await
            .context("Listing new objects")?;

        let added_objects: HashSet<_> = new_objects.difference(&initial_objects).cloned().collect();
        let removed_objects: HashSet<_> =
            initial_objects.difference(&new_objects).cloned().collect();

        // Detect any tables that have been rewritten. A rewritten table will always have a
        // different file node than the original table.
        let new_table_file_nodes = introspector
            .list_object_file_nodes()
            .await
            .context("Listing new table file nodes")?;

        let rewrites: HashSet<_> = new_table_file_nodes
            .into_iter()
            .filter_map(|(table, node)| match initial_table_file_nodes.get(&table) {
                Some(initial_node) if initial_node != &node => Some(table),
                _ => None,
            })
            .collect();

        Ok(InspectedStatement {
            added_objects,
            removed_objects,
            locks: all_detected_locks,
            rewrites,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::start_test_postgres;

    use crate::InspectedStatement;
    use crate::oracle::QueryOracle;

    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    async fn test_simple_inspect_statement() {
        let (_container, dsn) = start_test_postgres().await;
        let mut oracle = QueryOracle::new(&dsn);
        let result = oracle.inspect_statement("select 1;").await.unwrap();
        assert_eq!(result, InspectedStatement::default())
    }
}

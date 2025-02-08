use crate::introspection::Introspector;
use crate::{DBObject, TableLock};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// [InspectedStatement] is a struct that contains the side effects of inspecting a SQL statement.
/// It includes the objects that were added, removed, locked, and rewritten by the statement.
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct InspectedStatement {
    pub added_objects: HashSet<DBObject>,
    pub removed_objects: HashSet<DBObject>,
    pub locks: HashSet<TableLock>,
    pub rewrites: HashSet<DBObject>,
}

struct Driver {
    dsn: String,
}

impl Driver {
    pub fn new(dsn: impl ToString) -> Self {
        Self {
            dsn: dsn.to_string(),
        }
    }

    /// Inspect a statement and return an [InspectedStatement], containing a summary of the
    /// side effects of the statement.
    pub async fn inspect_statement(
        &mut self,
        statement: &str,
    ) -> anyhow::Result<InspectedStatement> {
        todo!()
    }
}

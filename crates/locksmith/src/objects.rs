use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A Postgres connection ID. Connections IDs can be retrieved via the
/// [pg_backend_pid](https://pgpedia.info/p/pg_backend_pid.html) function.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConnectionID(pub i32);

/// A lock
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Lock {
    AccessShareLock,
    RowShareLock,
    RowExclusiveLock,
    ShareUpdateExclusiveLock,
    ShareLock,
    ShareRowExclusiveLock,
    ExclusiveLock,
    AccessExclusiveLock,
    /// Note: this variant is used for any lock type that is not recognized by the library
    /// This is useful for future compatibility with new lock types, but should never
    /// appear in practice.
    Unknown(String),
}

impl Display for Lock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<String> for Lock {
    fn from(value: String) -> Self {
        match value.as_str() {
            "AccessShareLock" => Self::AccessShareLock,
            "RowShareLock" => Self::RowShareLock,
            "RowExclusiveLock" => Self::RowExclusiveLock,
            "ShareUpdateExclusiveLock" => Self::ShareUpdateExclusiveLock,
            "ShareLock" => Self::ShareLock,
            "ShareRowExclusiveLock" => Self::ShareRowExclusiveLock,
            "ExclusiveLock" => Self::ExclusiveLock,
            "AccessExclusiveLock" => Self::AccessExclusiveLock,
            _ => Self::Unknown(value),
        }
    }
}

/// A database object, which can be a table, column, or index
#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, derive_more::From, Serialize, Deserialize,
)]
pub enum DBObject {
    Table(TableObject),
    Column(ColumnObject),
    Index(IndexObject),
}

impl Display for DBObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DBObject::Table(table) => write!(f, "Table {}", table.name),
            DBObject::Column(column) => write!(
                f,
                "Column {}.{} ({})",
                column.table.name, column.name, column.data_type
            ),
            DBObject::Index(index) => write!(f, "Index {}.{}", index.table.name, index.name),
        }
    }
}

/// A lock on a specific table
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TableLock {
    pub table: TableObject,
    pub lock: Lock,
}

/// A table, identified by its name
#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, derive_more::From, Serialize, Deserialize,
)]
#[from(String, &str)]
pub struct TableObject {
    pub name: String,
}

impl Display for TableObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

/// A column in a given table, with a data type
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ColumnObject {
    pub table: TableObject,
    pub name: String,
    pub data_type: String,
}

/// An index on a table, identified by its name
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct IndexObject {
    pub table: TableObject,
    pub name: String,
}

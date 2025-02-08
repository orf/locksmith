use serde::{Deserialize, Serialize};

/// A Postgres connection ID. Connections IDs can be retrieved via the
/// [pg_backend_pid](https://pgpedia.info/p/pg_backend_pid.html) function.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConnectionID(pub i32);

/// A lock
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Hash, Eq, PartialEq, derive_more::From, Serialize, Deserialize)]
pub enum DBObject {
    Table(TableObject),
    Column(ColumnObject),
    Index(IndexObject),
}

/// A lock on a specific table
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TableLock {
    pub table: TableObject,
    pub lock: Lock,
}

/// A table, identified by its name
#[derive(Clone, Debug, Hash, Eq, PartialEq, derive_more::From, Serialize, Deserialize)]
#[from(String, &str)]
pub struct TableObject {
    pub name: String,
}

/// A column in a given table, with a data type
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ColumnObject {
    pub table: TableObject,
    pub name: String,
    pub data_type: String,
}

/// An index on a table, identified by its name
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndexObject {
    pub table: TableObject,
    pub name: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConnectionID(pub i32);

#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub enum Lock {
    // to-do
}

impl From<String> for Lock {
    fn from(value: String) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum DBObject {
    Table(TableObject)
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TableLock {
    pub table: TableObject,
    pub lock: Lock,
}

#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct TableObject {
    pub name: String,
}

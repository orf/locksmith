use crate::ConnectionID;
use crate::objects::{TableLock, TableObject};

pub struct Locker {}

impl Locker {
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        todo!()
    }

    pub async fn lock_tables(&mut self, tables: impl IntoIterator<Item=&TableObject>) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn list_connection_locks(&mut self, connection_id: ConnectionID) -> anyhow::Result<Vec<TableLock>> {
        todo!()
    }
}

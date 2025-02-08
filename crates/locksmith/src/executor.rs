use crate::ConnectionID;

pub struct StatementExecutor {}

impl StatementExecutor {
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        todo!()
    }

    pub fn connection_id(&self) -> ConnectionID {
        todo!()
    }

    pub async fn check_statement_for_locks(&mut self, statement: &str) -> anyhow::Result<bool> {
        todo!()
    }
}
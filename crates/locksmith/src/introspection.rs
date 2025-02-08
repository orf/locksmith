use crate::objects::DBObject;
use std::collections::{HashMap, HashSet};

pub struct Introspector {}

impl Introspector {
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        todo!()
    }

    pub async fn list_object_file_nodes(&mut self) -> anyhow::Result<HashMap<DBObject, i32>> {
        todo!()
    }

    pub async fn list_objects(&mut self) -> anyhow::Result<HashSet<DBObject>> {
        todo!()
    }
}

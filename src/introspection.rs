use tokio_postgres::{Client, Connection, Error, Socket};
use tokio_postgres::tls::NoTlsStream;


pub async fn list_server_objects(client: &Client) -> Result<(), Error> {
    let rows = client.query(r#"
    select oid, relname, relkind, relnamespace, reltype, relfilenode from pg_class
    "#, &[]).await?;
    for row in rows {
        let obj: &str = row.get("relkind");
        eprintln!("rows: {:#?}", row);
    }

    Ok(())
}

// https://www.postgresql.org/docs/current/catalog-pg-class.html
// relkind char:
// r = ordinary table, i = index, S = sequence, t = TOAST table, v = view, m = materialized view,
// c = composite type, f = foreign table, p = partitioned table, I = partitioned index
#[derive(Debug)]
enum PgObject {
    Table,
    Index,
    Sequence,
    Toast,
    View,
    MaterializedView,
    CompositeType,
    ForeignTable,
    PartitionedTable,
    PartitionedIndex,
}

impl TryFrom<char> for PgObject {
    type Error = derive_more::FromStrError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'r' => Ok(PgObject::Table),
            'i' => Ok(PgObject::Index),
            'S' => Ok(PgObject::Sequence),
            't' => Ok(PgObject::Toast),
            'v' => Ok(PgObject::View),
            'm' => Ok(PgObject::MaterializedView),
            'c' => Ok(PgObject::CompositeType),
            'f' => Ok(PgObject::ForeignTable),
            'p' => Ok(PgObject::PartitionedTable),
            'I' => Ok(PgObject::PartitionedIndex),
            _ => Err(Self::Error::new("PgObject")),
        }
    }
}


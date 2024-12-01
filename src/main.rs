mod lock;

use tokio_postgres::{NoTls, Error, AsyncMessage};
use futures_util::{
    future, join, pin_mut, stream, try_join, Future, FutureExt, SinkExt, StreamExt, TryStreamExt,
};
use futures_channel::mpsc;

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> Result<(), Error> {
    // Connect to the database.
    let (client, mut connection) =
        tokio_postgres::connect("host=localhost user=postgres password=password", NoTls).await?;

    let (tx, rx) = mpsc::unbounded::<AsyncMessage>();

    let (tx, rx) = mpsc::unbounded();
    let stream =
        stream::poll_fn(move |cx| connection.poll_message(cx)).map_err(|e| panic!("{}", e));
    let connection = stream.forward(tx).map(|r| r.unwrap());
    tokio::spawn(connection);

    client.execute("set client_min_messages=debug5;", &[]).await?;
    client.execute("set log_min_messages=debug5;", &[]).await?;
    client.execute("set trace_locks=true;", &[]).await?;

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    client.execute("drop table if exists test;", &[]).await?;
    client.execute("create table test (id int);", &[]).await?;
    client.execute("insert into test VALUES (1);", &[]).await?;
    client.execute("drop table test;", &[]).await?;

    // And then check that we got back the same string we sent over.
    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");
    println!("value: {:?}", rows);

    drop(client);

    let notices = rx
        .filter_map(|m| match m {
            AsyncMessage::Notice(n) => future::ready(Some(n)),
            _ => future::ready(None),
        })
        .collect::<Vec<_>>()
        .await;
    for notice in notices {
        if notice.file() == Some("lock.c") && notice.message().contains("lock(") {

            eprintln!("{}", notice.message());
            // eprintln!("{:#?}", notice);
        }

    }
    Ok(())
}
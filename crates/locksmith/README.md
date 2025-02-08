# Locksmith

This crate provides the logic for detecting the impact of a given SQL statement on a Postgres database. 

It is the brains behind [locksmith-cli](https://crates.io/crates/locksmith-cli).

## Detecting the impact of a statement

```rust
use locksmith::QueryOracle;

async fn inspect_statement() {
    let mut oracle = QueryOracle::new("postgres://localhost:5432/mydb");
    let inspection = oracle.inspect_statement("alter table customers alter column id type bigint;").await.unwrap();
    println!("{:?}", inspection.locks);
}
```
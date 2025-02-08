# locksmith-cli 🔐

This is the CLI for Locksmith, a Postgres migration helper that can detect the impact of a given SQL statement on a 
Postgres database. Specifically, it can detect:

1. Per-table locks
2. Table rewrites
3. Added, removed, and modified tables, columns and indexes

# Installation

With cargo: `cargo install locksmith-cli`

With Docker:

```shell
docker run -v /var/run/docker.sock:/var/run/docker.sock -vschema.sql:/data/schema.sql \
  ghcr.io/orf/locksmith /data/test_schema.sql "drop table customers cascade;"
```

# Example:

Given this schema:

```sql
create table customers
(
    id   serial primary key,
    name text not null
);

create table orders
(
    id          serial primary key,
    customer_id integer not null references customers (id)
);
```

Running the following command:

```shell
$ locksmith-cli schema.sql 'alter table customers alter column id type bigint;'
2025-02-08T20:59:07.299156Z  INFO locksmith_cli: Starting Postgres container tag="15-alpine"
2025-02-08T20:59:08.901689Z  INFO locksmith::oracle: Statement executed successfully
2025-02-08T20:59:08.905577Z  INFO locksmith_cli: Inspected statement added=1 removed=1 locks=2 rewrites=1
```

Will output the following JSON to stdout, describing the impact of the statement:

```json
{
  "added_objects": [
    {
      "Column": {
        "table": {
          "name": "customers"
        },
        "name": "id",
        "data_type": "bigint"
      }
    }
  ],
  "removed_objects": [
    {
      "Column": {
        "table": {
          "name": "customers"
        },
        "name": "id",
        "data_type": "integer"
      }
    }
  ],
  "locks": [
    {
      "table": {
        "name": "orders"
      },
      "lock": "AccessExclusiveLock"
    },
    {
      "table": {
        "name": "customers"
      },
      "lock": "AccessExclusiveLock"
    }
  ],
  "rewrites": [
    {
      "Table": {
        "name": "customers"
      }
    }
  ]
}
```

# Full usage:

```shell
$ locksmith-cli --help
Usage: locksmith-cli [OPTIONS] <SCHEMA_FILE> <QUERY>

Arguments:
  <SCHEMA_FILE>  The path to a file containing the initial database schema for the test. This can be in a plaintext SQL format or a binary format generated by `pg_dump`
  <QUERY>        The SQL query to inspect

Options:
  -t, --tag <TAG>        The tag of the Postgres container to start [default: 15-alpine]
  -o, --output <OUTPUT>  The output file to write the inspection results to. If not provided, the results will be written to stdout [default: -]
  -f, --format <FORMAT>  The output format [default: json] [possible values: json, markdown]
  -h, --help             Print help
  -V, --version          Print version
```

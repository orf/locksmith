-- lock:    {"table": {"name": "orders"}, "lock": "AccessExclusiveLock"}
-- lock:    {"table": {"name": "customers"}, "lock": "AccessExclusiveLock"}
-- removed: {"Column": {"table": {"name": "customers"}, "name": "id", "data_type": "integer"}}
-- added:   {"Column": {"table": {"name": "customers"}, "name": "id", "data_type": "bigint"}}
-- rewrite: {"Table": {"name": "customers"}}
alter table customers alter column id type bigint;
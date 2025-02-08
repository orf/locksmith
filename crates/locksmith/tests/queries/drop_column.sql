-- lock: {"table": {"name": "orders"}, "lock": "AccessExclusiveLock"}
-- lock: {"table": {"name": "customers"}, "lock": "AccessExclusiveLock"}
-- removed: {"Column": {"table": {"name": "orders"}, "name": "customer_id", "data_type": "integer"}}
alter table orders drop column customer_id;
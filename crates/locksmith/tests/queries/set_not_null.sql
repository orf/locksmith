-- lock: {"table": {"name": "orders"}, "lock": "AccessExclusiveLock"}
alter table orders alter column customer_id set not null;
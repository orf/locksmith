-- lock: {"table": {"name": "customers"}, "lock": "AccessExclusiveLock"}
-- lock: {"table": {"name": "orders"}, "lock": "AccessExclusiveLock"}
-- removed: {"Table": {"name": "orders"}}
-- removed: {"Index": {"table": {"name": "orders"}, "name": "orders_pkey"}}
-- removed: {"Index": {"table": {"name": "orders"}, "name": "orders_price_idx"}}
-- removed: {"Column": {"table": {"name": "orders"}, "name": "id", "data_type": "integer"}}
-- removed: {"Column": {"table": {"name": "orders"}, "name": "price", "data_type": "numeric"}}
-- removed: {"Column": {"table": {"name": "orders"}, "name": "customer_id", "data_type": "integer"}}
drop table orders;
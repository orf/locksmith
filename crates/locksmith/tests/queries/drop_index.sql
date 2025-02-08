-- lock: {"table": {"name": "orders"}, "lock": "AccessExclusiveLock"}
-- removed: {"Index": {"table": {"name": "orders"}, "name": "orders_price_idx"}}
drop index orders_price_idx;
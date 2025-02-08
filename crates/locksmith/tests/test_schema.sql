create table customers
(
    id   serial primary key,
    name text not null
);

create table orders
(
    id          serial primary key,
    customer_id integer not null references customers (id),
    price       numeric not null
);

create index orders_price_idx on orders (price);

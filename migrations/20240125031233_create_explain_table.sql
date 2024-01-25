-- Add migration script here
create table explains
(
    id            text not null
        constraint explains_pk
            primary key,
    query         varchar not null,
    txn_uuid      text,
    explain_id    integer not null,
    select_type   varchar not null,
    "table"       varchar not null,
    partitions    varchar,
    _type         varchar not null,
    possible_keys varchar,
    key           varchar,
    key_len       integer,
    _ref          varchar,
    rows          integer,
    filtered      REAL,
    extra         varchar,
    record_time   int8 not null
);


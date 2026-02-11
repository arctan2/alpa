# Alpa DB
Serverless database engine made mainly for embedded devices. The core is generic and can run anywhere. You just need to implement few traits.
It's made for my esp32 microcontroller. It should theoritically run on anything though.

# Features
It's a relational-ish database. It's current state is like a key based row storage engine. But the codebase is designed in such a way that one can support full robust relational database.
Advanced querying capabilites is not implemented due to the limited ram the microcontroller has.

## Core Database Operations

✔️ Create tables

✔️ Read (SELECT)

✔️ Insert

✔️ Delete

✔️ Delete tables (drop)

⚠️ Update (does delete + insert in a single wal transaction internally, update of keys not supported)

## Schema and Data Model
⚠️ Mandatory Primary Key

⚠️ Only one primary key per table

✔️ Nullable

❌ No composite keys

❌ Foreign keys

❌ Constraints beyond primary key

❌ Secondary indexes

## Supported Data Types
✔️ Int64

✔️ Float64

✔️ Chars(255) (fixed max length, it will cut off if exceeded)

✔️ Null

## Naming Constraints
* Table name max length: 64 characters
* Column name max length: 64 characters
* Names exceeding the limit are silently truncated
* Number of columns per table is limited to 56

## Query Engine

Filtering Conditions:

✔️ Simple filtering

✔️ Top-level logical operators only (`AND` and `OR`)

❌ Nested expressions

❌ Subqueries

## Supported Operators

✔️ Eq

✔️ Gt

✔️ Lt

✔️ StartsWith

✔️ EndsWith

✔️ Contains

✔️ IsNull

| Note ⚠️: These operate on column-value comparisons only. No column-column is supported yet.

## Query Structure

✔️  Filter by primary key (fast path)

❌ Joins

❌ Cartesian products

❌ Table aliases

❌ Column projections (always full row)

## Pagination

✔️ LIMIT `[from, to)` supported

❌ ORDER BY

## Storage & Execution

✔️  Page-based storage

✔️  Deterministic execution

✔️  Embedded-first design

❌ No background threads

❌ No async execution

❌ No query planner / optimizer

## These are intentional omissions, not missing features (atleast for now)

❌ Joins

❌ Cartesian products

❌ GROUP BY

❌ Aggregations (SUM, AVG, etc.)

❌ HAVING

❌ ORDER BY

❌ Transactions

❌ Foreign keys

❌ Views

❌ Triggers

❌ Stored procedures

❌ SQL compliance

# WAL

It has a basic undo based write ahead logger. It just writes the pages it touches during insert and delete. If there is a fault during the operation then during next db open it will recover the old
db pages stored in wal file. No checkpoint system or transaction based stuff is implemented yet.

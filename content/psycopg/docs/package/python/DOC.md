---
name: package
description: "Psycopg PostgreSQL adapter guide for Python projects using Psycopg 3"
metadata:
  languages: "python"
  versions: "3.3.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "psycopg,postgresql,database,sql,async,connection-pool"
---

# Psycopg Python Package Guide

## Golden Rule

Use `psycopg` for new PostgreSQL code, not `psycopg2`. As of March 12, 2026, PyPI lists `psycopg 3.3.3` as the latest stable release, while the main docs root is already serving `3.3.4.dev1`. Treat the docs as the authoritative API guide for Psycopg 3, but double-check edge features against the installed package when the docs mention newer development behavior.

## Install

For most applications, install the binary build plus the pool extra:

```bash
python -m pip install --upgrade pip
python -m pip install "psycopg[binary,pool]==3.3.3"
```

Other supported install choices:

```bash
# Pure Python package only. Requires a local libpq runtime.
python -m pip install "psycopg==3.3.3"

# Build the C extension against the system libpq.
python -m pip install "psycopg[c,pool]==3.3.3"
```

Choose installs based on what you are shipping:

- Application/service: `psycopg[binary]` is the fastest path to a working install; `psycopg[c]` is the preferred production-style install when you want to link against the system `libpq`.
- Reusable library: depend on `psycopg`, not `psycopg[binary]`, so the final application can choose the implementation.
- Connection pooling: install `psycopg[pool]` or the separate `psycopg_pool` package. Pooling is released separately from the base package.

## Connection Setup

Psycopg accepts a PostgreSQL conninfo string or keyword arguments. A practical pattern is to store a full DSN in an environment variable.

```bash
export DATABASE_URL="postgresql://app:secret@db.example.com:5432/appdb?sslmode=require"
```

Basic sync setup:

```python
import os

import psycopg
from psycopg.rows import dict_row

dsn = os.environ["DATABASE_URL"]

with psycopg.connect(dsn, row_factory=dict_row) as conn:
    with conn.cursor() as cur:
        cur.execute("SELECT current_database() AS db, current_user AS user")
        print(cur.fetchone())
```

Equivalent keyword-style setup:

```python
import psycopg

conn = psycopg.connect(
    dbname="appdb",
    host="db.example.com",
    port=5432,
    user="app",
    password="secret",
    sslmode="require",
    application_name="context-hub-example",
)
```

If you need to inspect or merge conninfo values programmatically, use `psycopg.conninfo.make_conninfo()` and `conninfo_to_dict()`.

## Core Query Workflow

The import name is `psycopg`, not `psycopg3`.

```python
import psycopg
from psycopg.rows import dict_row

with psycopg.connect(row_factory=dict_row) as conn:
    with conn.cursor() as cur:
        cur.execute(
            """
            INSERT INTO users (email, active)
            VALUES (%s, %s)
            RETURNING id, email, active
            """,
            ("alice@example.com", True),
        )
        user = cur.fetchone()
        print(user["id"], user["email"], user["active"])
```

Useful defaults and shortcuts:

- `with psycopg.connect(...) as conn` closes the connection and commits on normal exit or rolls back on exception.
- `with conn.cursor() as cur` closes the cursor automatically.
- `executemany()` is the normal batch-write path for repeated parameter sets.
- `row_factory` can be set on the connection or per cursor. `dict_row`, `namedtuple_row`, `scalar_row`, and `class_row(...)` are the most useful built-ins.

Example using dataclass rows:

```python
from dataclasses import dataclass

import psycopg
from psycopg.rows import class_row

@dataclass
class UserRecord:
    id: int
    email: str

with psycopg.connect() as conn:
    with conn.cursor(row_factory=class_row(UserRecord)) as cur:
        cur.execute("SELECT id, email FROM users WHERE id = %s", (1,))
        print(cur.fetchone())
```

## Transactions

By default, Psycopg starts a transaction on the first database operation, including a plain `SELECT`. That surprises people coming from `psql` and is a common source of idle-in-transaction sessions.

For request/response style application code, the connection context is often enough:

```python
import psycopg

with psycopg.connect() as conn:
    with conn.cursor() as cur:
        cur.execute(
            "INSERT INTO audit_log (event_type, payload) VALUES (%s, %s)",
            ("user.created", '{"id": 1}'),
        )
```

For long-lived workers, prefer autocommit and open explicit transaction blocks only when needed:

```python
import psycopg

with psycopg.connect(autocommit=True) as conn:
    with conn.transaction():
        conn.execute(
            "INSERT INTO ledger (account_id, delta) VALUES (%s, %s)",
            (123, -50),
        )
        conn.execute(
            "INSERT INTO ledger (account_id, delta) VALUES (%s, %s)",
            (456, 50),
        )
```

Use `autocommit=True` when you need statements that cannot run inside a transaction, such as `CREATE DATABASE`, `VACUUM`, or stored procedures that use transaction control.

If one statement fails, the transaction remains aborted until you call `rollback()`.

## Safe SQL Composition

Use `%s` or `%(name)s` placeholders only for values. Do not merge user input into SQL with `%`, `+`, or f-strings.

```python
with conn.cursor() as cur:
    cur.execute(
        "INSERT INTO events (kind, payload) VALUES (%s, %s)",
        ("signup", '{"plan":"pro"}'),
    )
```

If you need dynamic table or column names, use `psycopg.sql`:

```python
from psycopg import sql

table = "users"
columns = ["id", "email"]

query = sql.SQL("SELECT {fields} FROM {table} WHERE id = %s").format(
    fields=sql.SQL(", ").join(map(sql.Identifier, columns)),
    table=sql.Identifier(table),
)

with conn.cursor() as cur:
    cur.execute(query, (42,))
    print(cur.fetchone())
```

Important server-side binding caveat: placeholders do not work for every SQL construct. Statements such as `SET`, `NOTIFY`, and many DDL commands cannot bind values server-side the same way normal DML does. For those cases, use PostgreSQL helper functions where possible or compose the statement with `psycopg.sql`.

## Async Usage

Use `AsyncConnection` and `AsyncCursor` for `asyncio` applications.

```python
import asyncio
import os

import psycopg
from psycopg.rows import dict_row

async def main() -> None:
    async with await psycopg.AsyncConnection.connect(
        os.environ["DATABASE_URL"],
        row_factory=dict_row,
    ) as conn:
        async with conn.cursor() as cur:
            await cur.execute(
                "SELECT id, email FROM users WHERE active = %s",
                (True,),
            )
            async for row in cur:
                print(row)

asyncio.run(main())
```

Async-specific notes:

- `AsyncConnection.connect()` must be awaited before it can be used as a context manager. `async with psycopg.AsyncConnection.connect(...)` is wrong; use `async with await ...`.
- On Windows, Psycopg async support is not compatible with the default `ProactorEventLoop`; use `SelectorEventLoop`.
- Since Psycopg 3.1, async connect resolves DNS without blocking. Before 3.1, avoiding DNS blocking required `hostaddr` or `resolve_hostaddr_async()`.

## Connection Pooling

Psycopg pools live in `psycopg_pool` and are also installable through the `pool` extra.

```python
from psycopg_pool import ConnectionPool

pool = ConnectionPool(
    conninfo="postgresql://app:secret@db.example.com:5432/appdb?sslmode=require",
    min_size=1,
    max_size=10,
)

with pool:
    with pool.connection() as conn:
        with conn.cursor() as cur:
            cur.execute("SELECT now()")
            print(cur.fetchone())
```

For async code:

```python
import os

from psycopg_pool import AsyncConnectionPool

async with AsyncConnectionPool(conninfo=os.environ["DATABASE_URL"]) as pool:
    async with pool.connection() as conn:
        await conn.execute("SELECT 1")
```

Pool connections behave like normal Psycopg connections: leaving the `pool.connection()` block commits or rolls back the current transaction and returns the connection to the pool.

## Concurrency Notes

- `Connection` objects are thread-safe, but query execution on one connection is serialized.
- `Cursor` objects are not thread-safe.
- Connections are not process-safe; create them after worker processes fork.
- If you need real parallel database work, use multiple connections or a pool instead of many cursors on one shared connection.

## Common Pitfalls

- Do not install `psycopg2` for new code unless you are maintaining an existing `psycopg2`-based dependency chain.
- Do not write `import psycopg3`; the module is `psycopg`.
- Do not forget that even `SELECT` starts a transaction by default. Long-lived jobs should usually use `autocommit=True` plus explicit `conn.transaction()` blocks.
- Do not pass a scalar directly as the second argument to `execute()`. A single value still needs a sequence such as `(value,)`.
- Do not quote placeholders. Use `WHERE id = %s`, not `WHERE id = '%s'`.
- Do not try to bind identifiers with `%s`; use `psycopg.sql.Identifier`.
- Do not assume async code gives parallel SQL execution on one connection. Access is still serialized per connection.
- Do not forget that pooling is separate from the base install unless you add the `pool` extra.

## Version-Sensitive Notes For Psycopg 3.3.3

- PyPI stable is `3.3.3` on February 18, 2026. The docs root currently serves `3.3.4.dev1`, so examples for very new edge behavior may reflect the development branch.
- Psycopg 3.3 supports Python `3.10` through `3.14`. Python `3.8` and `3.9` support ended before the 3.3 line.
- Psycopg 3.3 adds template string queries for Python `3.14+`. They are safer than f-strings because Psycopg processes the interpolated values itself.
- Psycopg 3.3 adds `Transaction.status` and improves cursor result-set navigation with `Cursor.results()` and `Cursor.set_result()`.
- Psycopg 3.3 cursors are iterators, so `next(cur)` and `anext(cur)` now work directly.

---
name: package
description: "psycopg2-binary package guide for Python projects using PostgreSQL via psycopg2"
metadata:
  languages: "python"
  versions: "2.9.11"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "postgresql,python,database,sql,db-api,psycopg2"
---

# psycopg2-binary Python Package Guide

## Golden Rule

Use `psycopg2-binary` when you need the Psycopg 2 driver installed quickly from wheels, but write application code against the `psycopg2` import path and Psycopg 2 APIs. For production environments, the upstream project advises using the source-built `psycopg2` package instead of the binary wheel package. For new greenfield projects, upstream also advises considering Psycopg 3 (`psycopg`), because Psycopg 2 is maintained but not expected to receive new features.

## Install

Fastest install:

```bash
python -m pip install --upgrade pip
python -m pip install "psycopg2-binary==2.9.11"
```

Common alternatives:

```bash
uv add "psycopg2-binary==2.9.11"
poetry add "psycopg2-binary==2.9.11"
```

The package name is `psycopg2-binary`, but the import remains:

```python
import psycopg2
```

## Connection Setup

`psycopg2.connect()` accepts a DSN string or keyword arguments. The common connection fields are `dbname`, `user`, `password`, `host`, `port`, and libpq-supported extras such as `sslmode` and `options`.

Example with environment variables:

```bash
export PGHOST=localhost
export PGPORT=5432
export PGDATABASE=appdb
export PGUSER=app
export PGPASSWORD=secret
```

```python
import os
import psycopg2

conn = psycopg2.connect(
    dbname=os.environ["PGDATABASE"],
    user=os.environ["PGUSER"],
    password=os.environ["PGPASSWORD"],
    host=os.environ.get("PGHOST", "localhost"),
    port=os.environ.get("PGPORT", "5432"),
    sslmode=os.environ.get("PGSSLMODE", "prefer"),
)
```

Example with a DSN string:

```python
import psycopg2

dsn = "dbname=appdb user=app password=secret host=localhost port=5432"
conn = psycopg2.connect(dsn)
```

Useful connection options:

- `sslmode="require"` or stricter in managed/cloud environments
- `connect_timeout=5` to fail fast
- `application_name="my-service"` for PostgreSQL observability
- `options="-c search_path=app,public"` when schema selection must be explicit

## Core Usage

### Basic query flow

```python
import psycopg2

conn = psycopg2.connect(
    dbname="appdb",
    user="app",
    password="secret",
    host="localhost",
    port=5432,
)

try:
    with conn.cursor() as cur:
        cur.execute(
            """
            CREATE TABLE IF NOT EXISTS users (
                id serial PRIMARY KEY,
                email text NOT NULL UNIQUE,
                active boolean NOT NULL DEFAULT true
            )
            """
        )

        cur.execute(
            "INSERT INTO users (email, active) VALUES (%s, %s) RETURNING id",
            ("ada@example.com", True),
        )
        user_id = cur.fetchone()[0]

        cur.execute(
            "SELECT id, email, active FROM users WHERE id = %s",
            (user_id,),
        )
        row = cur.fetchone()

    conn.commit()
    print(row)
except Exception:
    conn.rollback()
    raise
finally:
    conn.close()
```

### Prefer context managers for transactions

Connections and cursors support `with`. In Psycopg 2, leaving `with conn:` commits on success and rolls back on exception, but it does **not** close the connection.

```python
import psycopg2

conn = psycopg2.connect("dbname=appdb user=app password=secret host=localhost")
try:
    with conn:
        with conn.cursor() as cur:
            cur.execute(
                "INSERT INTO audit_log (event_type, payload) VALUES (%s, %s)",
                ("user.created", '{"id": 1}'),
            )
finally:
    conn.close()
```

### Use parameters, never string interpolation

Use `%s` placeholders for values, even for integers and dates. Do not use Python `%` formatting, f-strings, or string concatenation to build value expressions.

```python
cur.execute(
    "SELECT id, email FROM users WHERE email = %s AND active = %s",
    ("ada@example.com", True),
)
```

Named parameters are supported too:

```python
cur.execute(
    """
    UPDATE users
    SET email = %(email)s
    WHERE id = %(id)s
    """,
    {"id": 42, "email": "new@example.com"},
)
```

### Dynamic identifiers need `psycopg2.sql`

Values go through `execute(..., params)`. Table names, column names, and schema names must be composed with `psycopg2.sql`.

```python
from psycopg2 import sql

table_name = "users"

query = sql.SQL("SELECT id, email FROM {} WHERE active = %s").format(
    sql.Identifier(table_name)
)
cur.execute(query, (True,))
```

### Dict-like rows

For handler code and APIs, `RealDictCursor` is often more convenient than tuple rows.

```python
import psycopg2
from psycopg2.extras import RealDictCursor

with psycopg2.connect("dbname=appdb user=app password=secret host=localhost") as conn:
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute("SELECT id, email FROM users ORDER BY id LIMIT 2")
        rows = cur.fetchall()
        print(rows[0]["email"])
```

### Bulk inserts

For large insert batches, prefer `psycopg2.extras.execute_values()` over looping `execute()` calls.

```python
from psycopg2.extras import execute_values

rows = [
    ("ada@example.com", True),
    ("grace@example.com", True),
    ("linus@example.com", False),
]

execute_values(
    cur,
    "INSERT INTO users (email, active) VALUES %s",
    rows,
)
```

### Stream large result sets

Named cursors create server-side cursors so you do not pull the entire result set into memory at once.

```python
with conn.cursor(name="users_stream") as cur:
    cur.itersize = 1000
    cur.execute("SELECT id, email FROM users ORDER BY id")
    for row in cur:
        process(row)
```

## Transactions And Autocommit

Psycopg starts a transaction for normal statements. If you never commit or rollback, the session can remain idle in transaction and hold locks or prevent cleanup work.

Use default transactional behavior for normal CRUD:

```python
with conn:
    with conn.cursor() as cur:
        cur.execute("UPDATE accounts SET balance = balance - %s WHERE id = %s", (10, 1))
        cur.execute("UPDATE accounts SET balance = balance + %s WHERE id = %s", (10, 2))
```

Enable autocommit only for commands that must run outside a transaction, such as `CREATE DATABASE` or `VACUUM`:

```python
conn.autocommit = True
with conn.cursor() as cur:
    cur.execute("VACUUM")
```

## JSON, Arrays, UUID, And PostgreSQL Types

Psycopg 2 adapts many Python types automatically, including `None`, `bool`, `int`, `Decimal`, `datetime`, lists to PostgreSQL arrays, `UUID`, and JSON-capable values. For explicit JSON adaptation:

```python
from psycopg2.extras import Json

cur.execute(
    "INSERT INTO events (kind, payload) VALUES (%s, %s)",
    ("user.created", Json({"id": 1, "email": "ada@example.com"})),
)
```

## Errors And Diagnostics

Catch broad DB-API exceptions when you need a rollback boundary, and use PostgreSQL-specific subclasses or SQLSTATE codes when behavior depends on the exact error.

```python
import psycopg2
from psycopg2 import errorcodes, errors

try:
    cur.execute("INSERT INTO users (email) VALUES (%s)", ("ada@example.com",))
except errors.UniqueViolation:
    conn.rollback()
    raise
except psycopg2.Error as exc:
    conn.rollback()
    if exc.pgcode == errorcodes.DEADLOCK_DETECTED:
        raise RuntimeError("retry this transaction") from exc
    raise
```

## Connection Pooling

Opening PostgreSQL connections is relatively expensive. For threaded apps, use `ThreadedConnectionPool`. `SimpleConnectionPool` is only for single-threaded applications.

```python
from psycopg2.pool import ThreadedConnectionPool

pool = ThreadedConnectionPool(
    minconn=1,
    maxconn=10,
    dbname="appdb",
    user="app",
    password="secret",
    host="localhost",
    port=5432,
)

conn = pool.getconn()
try:
    with conn:
        with conn.cursor() as cur:
            cur.execute("SELECT 1")
finally:
    pool.putconn(conn)
```

## Common Pitfalls

- Package name and import name differ: install `psycopg2-binary`, import `psycopg2`.
- Do not interpolate query values with f-strings or `%`; always pass parameters separately.
- `%s` is the placeholder for all values. Do not switch to `%d` or quote placeholders yourself.
- Do not use bound parameters for table or column names; use `psycopg2.sql.Identifier`.
- `with conn:` manages the transaction only; it does not close the connection.
- Long-lived transactions can leave sessions idle in transaction. Commit or rollback promptly.
- Cursors are not safe to share across threads. Keep cursor scope local and use a pool or one connection per concurrent unit.
- In forked worker models, create connections after the fork instead of reusing inherited connections.
- `executemany()` is usually not the fastest bulk insert path for PostgreSQL; use `execute_values()` or `COPY` for larger loads.

## Version-Sensitive Notes

- The upstream docs and PyPI both currently show `2.9.11` as the active Psycopg 2 line as of March 12, 2026.
- The 2.9.11 release notes add Python 3.14 support and drop Python 3.8 support, so treat Python 3.9+ as the safe baseline for this package version.
- `psycopg2-binary` and `psycopg2` share the same runtime API; the difference is mainly packaging and linked client libraries.
- Upstream states that `psycopg2-binary` is convenient for development and testing, but production deployments should prefer the source-built `psycopg2` package.
- Upstream also states Psycopg 2 is still maintained but not expected to receive new features; new projects should consider Psycopg 3 (`psycopg`) if migration cost is low.
- Psycopg 2.9 changed `with connection` behavior so it starts a transaction even on autocommit connections; avoid assuming the context manager is transaction-free.

## Official Sources

- Docs root: https://www.psycopg.org/docs/
- Installation: https://www.psycopg.org/docs/install.html
- Basic usage: https://www.psycopg.org/docs/usage.html
- SQL composition: https://www.psycopg.org/docs/sql.html
- Extras: https://www.psycopg.org/docs/extras.html
- Pooling: https://www.psycopg.org/docs/pool.html
- PyPI: https://pypi.org/project/psycopg2-binary/

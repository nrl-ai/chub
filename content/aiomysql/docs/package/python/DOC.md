---
name: package
description: "aiomysql package guide for asyncio-based MySQL and MariaDB access in Python"
metadata:
  languages: "python"
  versions: "0.3.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aiomysql,mysql,mariadb,asyncio,database,sql"
---

# aiomysql Python Package Guide

## Golden Rule

Use `aiomysql` when you need direct asyncio access to MySQL or MariaDB with a PyMySQL-like API. Prefer a connection pool for application code, assume writes are not autocommitted unless you enable it, and close pooled resources with `pool.close()` plus `await pool.wait_closed()`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "aiomysql==0.3.2"
```

Common alternatives:

```bash
uv add "aiomysql==0.3.2"
poetry add "aiomysql==0.3.2"
```

Notes:

- `aiomysql` depends on `PyMySQL`; you do not install a separate MySQL C driver for the basic package.
- PyPI metadata exposes optional extras `sa` and `rsa`. Use them only when you specifically need the SQLAlchemy integration layer or RSA auth-related dependencies.

## Connect And Create A Pool

For application code, start with a pool instead of one ad hoc connection:

```python
import asyncio
import os

import aiomysql

async def main() -> None:
    pool = await aiomysql.create_pool(
        host=os.environ.get("MYSQL_HOST", "127.0.0.1"),
        port=int(os.environ.get("MYSQL_PORT", "3306")),
        user=os.environ["MYSQL_USER"],
        password=os.environ["MYSQL_PASSWORD"],
        db=os.environ["MYSQL_DATABASE"],
        minsize=1,
        maxsize=10,
        autocommit=False,
        pool_recycle=3600,
        charset="utf8mb4",
    )

    try:
        async with pool.acquire() as conn:
            async with conn.cursor(aiomysql.DictCursor) as cur:
                await cur.execute("SELECT VERSION() AS version")
                row = await cur.fetchone()
                print(row["version"])
    finally:
        pool.close()
        await pool.wait_closed()

asyncio.run(main())
```

Why these settings matter:

- `autocommit=False` is the documented default, so inserts and updates need an explicit `await conn.commit()`.
- `pool_recycle` helps avoid stale server-side connections in long-running processes.
- `charset="utf8mb4"` is the safer default for modern Unicode text than leaving the charset empty.

For short scripts or migrations, a single connection is fine:

```python
import asyncio
import aiomysql

async def main() -> None:
    conn = await aiomysql.connect(
        host="127.0.0.1",
        port=3306,
        user="app",
        password="secret",
        db="appdb",
        autocommit=True,
        charset="utf8mb4",
    )

    try:
        async with conn.cursor() as cur:
            await cur.execute("SELECT 1")
            print(await cur.fetchone())
    finally:
        await conn.ensure_closed()

asyncio.run(main())
```

## Core Usage

### Run queries

```python
async with pool.acquire() as conn:
    async with conn.cursor(aiomysql.DictCursor) as cur:
        await cur.execute(
            """
            SELECT id, email
            FROM users
            WHERE is_active = %s
            ORDER BY id
            LIMIT %s
            """,
            (True, 100),
        )
        rows = await cur.fetchall()

for row in rows:
    print(row["id"], row["email"])
```

Use parameter binding with `%s` placeholders. Do not build SQL by concatenating user input.

### Transactions

Because autocommit is off by default, explicit commit and rollback handling matters:

```python
async with pool.acquire() as conn:
    try:
        async with conn.cursor() as cur:
            await cur.execute(
                "INSERT INTO audit_log(event_type, payload) VALUES (%s, %s)",
                ("user.created", '{"user_id": 42}'),
            )
        await conn.commit()
    except Exception:
        await conn.rollback()
        raise
```

If you want each statement committed immediately, set `autocommit=True` on the connection or pool.

### Batch inserts

`executemany()` batches inserts using MySQL multi-row syntax:

```python
records = [
    ("Jane", "555-001"),
    ("Joe", "555-002"),
    ("John", "555-003"),
]

async with pool.acquire() as conn:
    async with conn.cursor() as cur:
        await cur.executemany(
            "INSERT INTO employees(name, phone) VALUES (%s, %s)",
            records,
        )
    await conn.commit()
```

### Large result sets

Use `SSCursor` or `SSDictCursor` when you need to stream a lot of rows with lower memory usage:

```python
async with pool.acquire() as conn:
    async with conn.cursor(aiomysql.SSDictCursor) as cur:
        await cur.execute("SELECT id, payload FROM events ORDER BY id")
        while True:
            rows = await cur.fetchmany(1000)
            if not rows:
                break
            for row in rows:
                process(row)
```

`SSCursor` is unbuffered. The docs note that row counts are not known up front and backward scrolling is not supported.

### Stored procedures

`callproc()` works, but the docs call out two important DB-API quirks:

- OUT and INOUT values are not returned directly; you may need to query server variables such as `@_procname_0` after consuming result sets.
- Stored procedure calls create an extra empty result set, so use `await cur.nextset()` until all result sets are consumed.

## Configuration And Authentication

Important `connect()` parameters from the official API reference:

- `host`, `port`, `db`, `user`, `password`: standard TCP connection settings
- `unix_socket`: use a local Unix domain socket instead of TCP when appropriate
- `charset`: character set for the session
- `sql_mode`: session SQL mode such as `STRICT_TRANS_TABLES`
- `connect_timeout`: timeout in seconds for the initial connection
- `init_command`: SQL command to run immediately after connection setup
- `read_default_file` and `read_default_group`: read client settings from a MySQL option file
- `cursorclass`: set a default cursor type such as `aiomysql.DictCursor`
- `ssl`: pass an `ssl.SSLContext` to require TLS
- `auth_plugin`: manually choose an auth plugin when needed, for example `mysql_clear_password` for Amazon RDS IAM authentication
- `server_public_key`: public key for SHA256 authentication flows
- `program_name`: client program name sent during handshake
- `local_infile`: enable `LOAD DATA LOCAL`; disabled by default and unsafe against untrusted servers

Example with TLS and explicit auth plugin:

```python
import asyncio
import ssl

import aiomysql

async def main() -> None:
    tls = ssl.create_default_context(cafile="/path/to/ca.pem")

    conn = await aiomysql.connect(
        host="db.example.com",
        user="app",
        password="secret",
        db="appdb",
        ssl=tls,
        auth_plugin="mysql_clear_password",
        autocommit=True,
    )

    try:
        async with conn.cursor() as cur:
            await cur.execute("SELECT CURRENT_USER()")
            print(await cur.fetchone())
    finally:
        await conn.ensure_closed()

asyncio.run(main())
```

## SQLAlchemy Integration

`aiomysql.sa` is the package's official SQLAlchemy integration layer:

```python
import asyncio
import sqlalchemy as sa
from aiomysql.sa import create_engine

metadata = sa.MetaData()
users = sa.Table(
    "users",
    metadata,
    sa.Column("id", sa.Integer, primary_key=True),
    sa.Column("email", sa.String(255)),
)

async def main() -> None:
    engine = await create_engine(
        host="127.0.0.1",
        user="app",
        password="secret",
        db="appdb",
    )

    try:
        async with engine.acquire() as conn:
            result = await conn.execute(users.select())
            async for row in result:
                print(row.id, row.email)
    finally:
        engine.close()
        await engine.wait_closed()

asyncio.run(main())
```

The official docs describe this as support for SQLAlchemy's functional SQL layer. Treat it as a separate API surface from the raw connection and cursor API.

## Common Pitfalls

- `pool.close()` and `pool.terminate()` are not coroutines. Call them directly, then `await pool.wait_closed()`.
- `conn.close()` closes immediately and is not a coroutine; `await conn.ensure_closed()` sends a quit command and then closes the socket.
- The upstream docs still show older `loop=...` examples. For new Python 3.9+ code, prefer `asyncio.run(...)` and omit `loop=` unless you are integrating with older event-loop management. This is an inference from the current docs still exposing optional `loop` parameters while also showing modern `asyncio.run()` examples in the README and `aiomysql.sa` docs.
- `autocommit=False` is the default. If inserts "work" but nothing is persisted, you probably forgot `await conn.commit()`.
- Leave `local_infile=False` unless you explicitly need `LOAD DATA LOCAL` and trust the server; the docs warn that aiomysql does not validate server-requested files.
- Use `DictCursor` or `SSDictCursor` when your calling code expects column names. The default cursor returns tuples.
- `SSCursor.fetchall()` defeats the point of streaming because it still reads the full result set.
- Stored procedures have result-set edge cases. Consume all result sets with `nextset()` before assuming the connection is safe for the next query.

## Version-Sensitive Notes For 0.3.x

- `0.3.2` is the current package release on PyPI, published on 2025-10-22.
- The `0.3.0` release dropped support for Python 3.7 and 3.8. If you find older examples pinned to `0.2.x` or earlier, check the runtime first.
- PyPI now declares `Requires-Python >=3.9`.
- The `0.3.0` release tightened `local_infile` handling: passing only a client flag is no longer enough; use the documented `local_infile` parameter explicitly.
- The `0.2.0` release bumped the minimum SQLAlchemy version for `aiomysql.sa` to `1.3` and the minimum PyMySQL version to `1.0.0`.

## Official Sources

- Docs root: `https://aiomysql.readthedocs.io/en/stable/`
- Tutorial: `https://aiomysql.readthedocs.io/en/stable/tutorial.html`
- Connection API: `https://aiomysql.readthedocs.io/en/stable/connection.html`
- Pool API: `https://aiomysql.readthedocs.io/en/stable/pool.html`
- Cursor API: `https://aiomysql.readthedocs.io/en/stable/cursors.html`
- SQLAlchemy integration: `https://aiomysql.readthedocs.io/en/stable/sa.html`
- PyPI package page: `https://pypi.org/project/aiomysql/`
- PyPI release page for `0.3.2`: `https://pypi.org/project/aiomysql/0.3.2/`
- GitHub repository and releases: `https://github.com/aio-libs/aiomysql` and `https://github.com/aio-libs/aiomysql/releases`

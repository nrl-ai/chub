---
name: package
description: "asyncpg package guide for asyncio PostgreSQL connections, pools, transactions, and type codecs"
metadata:
  languages: "python"
  versions: "0.31.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "asyncpg,postgresql,postgres,asyncio,database,sql"
---

# asyncpg Python Package Guide

## What It Is

`asyncpg` is MagicStack's native asyncio PostgreSQL driver. It is not a DB-API wrapper and it is not an ORM. It exposes PostgreSQL-oriented async primitives directly: connections, pools, transactions, prepared statements, cursors, COPY helpers, and type codecs.

## Version-Sensitive Notes For `0.31.0`

- The version used here `0.31.0`, the current docs site, and the current PyPI release all align as of March 12, 2026.
- `0.31.0` adds PostgreSQL connection service file support via the `service` and `servicefile` connection arguments.
- `0.31.0` adds Python 3.14 support, including experimental subinterpreter and free-threading support.
- `0.31.0` drops Python 3.8 support. Do not target Python 3.8 for new code.
- Pool `connect` and `reset` hooks were added in `0.30.0`; they are available in this version and useful for advanced pool customization.

## Install

Pin the exact version if your project depends on stable behavior:

```bash
python -m pip install "asyncpg==0.31.0"
```

Common alternatives:

```bash
uv add "asyncpg==0.31.0"
poetry add "asyncpg==0.31.0"
```

If you need GSSAPI or SSPI authentication support:

```bash
python -m pip install "asyncpg[gssauth]==0.31.0"
```

Upstream notes that Linux GSSAPI installs need a C compiler plus Kerberos development headers because the `gssapi` dependency does not ship Linux wheels.

## Connection Setup

You can connect with a DSN or explicit keyword arguments. `asyncpg` also honors the usual PostgreSQL environment variables for most connection defaults, including `PGHOST`, `PGPORT`, `PGUSER`, and `PGDATABASE`.

### Minimal connection

```python
import asyncio
import asyncpg

async def main() -> None:
    conn = await asyncpg.connect(
        user="app",
        password="secret",
        database="appdb",
        host="127.0.0.1",
    )
    try:
        value = await conn.fetchval("select 1")
        print(value)
    finally:
        await conn.close()

asyncio.run(main())
```

### DSN-based connection

```python
conn = await asyncpg.connect(
    "postgresql://app:secret@127.0.0.1:5432/appdb"
)
```

### Environment-driven setup

Prefer keeping connection settings out of source code:

```bash
export PGHOST=127.0.0.1
export PGPORT=5432
export PGUSER=app
export PGDATABASE=appdb
```

Then connect with only the secret at runtime, or let PostgreSQL password handling resolve it from a passfile:

```python
conn = await asyncpg.connect()
```

## Authentication And Config

### Password handling

- `password=` can be a string or a callable. If it is a callable, `asyncpg` calls it whenever a new connection is established.
- `PGPASSWORD` works, but upstream explicitly discourages it because other users or processes may be able to read it.
- Prefer `passfile=` or the default PostgreSQL password file (`~/.pgpass`, or `%APPDATA%\\postgresql\\pgpass.conf` on Windows) for local and server deployments.

### PostgreSQL service files

`0.31.0` adds support for PostgreSQL connection service files. This is useful when you want to keep host, port, database, SSL, and similar settings in `pg_service.conf` rather than in code.

```python
conn = await asyncpg.connect(
    service="app-primary",
    servicefile="/etc/postgresql-common/pg_service.conf",
)
```

### SSL

`ssl=` accepts:

- `True` or an `ssl.SSLContext`
- PostgreSQL-style modes such as `'disable'`, `'prefer'`, `'allow'`, `'require'`, `'verify-ca'`, and `'verify-full'`

The default is `'prefer'`, which will try TLS first and fall back to non-TLS if that fails. For production systems, prefer `ssl="verify-full"` or an explicit `ssl.SSLContext` that verifies the server certificate.

```python
import ssl

ssl_ctx = ssl.create_default_context(
    ssl.Purpose.SERVER_AUTH,
    cafile="/etc/ssl/certs/pg-ca.pem",
)
ssl_ctx.check_hostname = True

conn = await asyncpg.connect(
    host="db.example.com",
    user="app",
    database="appdb",
    ssl=ssl_ctx,
)
```

### Timeouts and server settings

`asyncpg.connect(...)` supports:

- `timeout=` for connection establishment
- `command_timeout=` as the default per-command timeout
- `server_settings=` for PostgreSQL runtime parameters such as `application_name`

```python
conn = await asyncpg.connect(
    host="db.example.com",
    user="app",
    database="appdb",
    timeout=10,
    command_timeout=30,
    server_settings={"application_name": "billing-worker"},
)
```

## Core Query Patterns

Use PostgreSQL positional placeholders: `$1`, `$2`, and so on.

```python
row = await conn.fetchrow(
    """
    select id, email, created_at
    from users
    where id = $1
    """,
    user_id,
)
```

Common methods:

- `fetch(...)`: many rows as `list[asyncpg.Record]`
- `fetchrow(...)`: one row or `None`
- `fetchval(...)`: first column of the first row
- `execute(...)`: command status string
- `executemany(...)`: repeated statement execution

### Inserts and reads

```python
await conn.execute(
    """
    insert into users(email, is_active)
    values($1, $2)
    """,
    "alice@example.com",
    True,
)

user_id = await conn.fetchval(
    "select id from users where email = $1",
    "alice@example.com",
)
```

### Records are mappings, not dicts

Returned rows are `asyncpg.Record` objects. Convert them when you need plain JSON-serializable data:

```python
row = await conn.fetchrow("select id, email from users where id = $1", user_id)
payload = dict(row) if row is not None else None
```

## Transactions

Outside an explicit transaction block, writes are applied immediately. Use `Connection.transaction()` with `async with` for atomic multi-statement work.

```python
async with conn.transaction():
    order_id = await conn.fetchval(
        """
        insert into orders(customer_id, total_cents)
        values($1, $2)
        returning id
        """,
        customer_id,
        total_cents,
    )

    await conn.execute(
        """
        insert into order_events(order_id, event_type)
        values($1, $2)
        """,
        order_id,
        "created",
    )
```

You can also control transaction mode:

```python
async with conn.transaction(
    isolation="serializable",
    readonly=False,
    deferrable=False,
):
    ...
```

## Connection Pools

For server-style applications, upstream recommends using `asyncpg.create_pool(...)`. The pool handles connection reuse and exposes `acquire()` for short-lived request work.

```python
import asyncio
import asyncpg

async def init_pool() -> asyncpg.Pool:
    return await asyncpg.create_pool(
        host="127.0.0.1",
        user="app",
        password="secret",
        database="appdb",
        min_size=5,
        max_size=20,
        command_timeout=30,
    )

async def get_user(pool: asyncpg.Pool, user_id: int) -> dict | None:
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            "select id, email from users where id = $1",
            user_id,
        )
        return dict(row) if row is not None else None
```

Important pool parameters in `0.31.0`:

- `min_size`, `max_size`
- `max_queries`
- `max_inactive_connection_lifetime`
- `init=` to initialize new connections once, such as registering type codecs
- `setup=` to prepare a connection before each `acquire()`
- `reset=` to customize how a connection is cleaned up before release

### Register codecs on every pooled connection

If your application depends on custom JSON or other codecs, register them in `init=` so every new pooled connection is configured consistently:

```python
import json

async def init_connection(conn: asyncpg.Connection) -> None:
    await conn.set_type_codec(
        "json",
        encoder=json.dumps,
        decoder=json.loads,
        schema="pg_catalog",
    )

pool = await asyncpg.create_pool(
    host="127.0.0.1",
    user="app",
    password="secret",
    database="appdb",
    init=init_connection,
)
```

## Type Conversion And Codecs

`asyncpg` automatically maps many PostgreSQL types to Python values:

- `timestamp` -> `datetime.datetime`
- `date` -> `datetime.date`
- `uuid` -> `uuid.UUID`
- `numeric` -> `decimal.Decimal`
- arrays -> Python `list`
- records -> `asyncpg.Record`

Important default behaviors:

- `json` and `jsonb` decode to `str` by default, not Python `dict`
- `numeric` decodes to `Decimal`
- unsupported or unknown types fall back to text

### Decode JSON automatically

```python
import json

await conn.set_type_codec(
    "json",
    encoder=json.dumps,
    decoder=json.loads,
    schema="pg_catalog",
)

doc = await conn.fetchval("select $1::json", {"ok": True})
assert doc == {"ok": True}
```

### Decode `numeric` as float when precision loss is acceptable

```python
await conn.set_type_codec(
    "numeric",
    encoder=str,
    decoder=float,
    schema="pg_catalog",
    format="text",
)
```

### Enable builtin `hstore` mapping

```python
await conn.set_builtin_type_codec(
    "hstore",
    codec_name="pg_contrib.hstore",
)
```

## COPY And Bulk Loading

If you need PostgreSQL bulk I/O, prefer the native COPY helpers instead of row-by-row inserts:

- `copy_records_to_table(...)`
- `copy_to_table(...)`
- `copy_from_table(...)`
- `copy_from_query(...)`

These are usually the right tool for high-volume ingestion or export pipelines.

## Common Pitfalls

- Use PostgreSQL placeholders like `$1`, not `%s` or `?`.
- `expression IN $1` is invalid PostgreSQL syntax. Use `expression = any($1::mytype[])`.
- `asyncpg` is not DB-API compatible. Do not assume cursor or connection semantics from synchronous drivers.
- `asyncpg.Record` is not a plain dict. Convert with `dict(row)` if the next layer expects serializable mappings.
- `json` and `jsonb` come back as strings unless you register a codec.
- Cursors created with `Connection.cursor()` cannot be used outside a transaction.
- Prepared statements do not work correctly behind PgBouncer in `transaction` or `statement` pool modes. If you must use that setup, pass `statement_cache_size=0` and avoid explicit prepared statements, or switch PgBouncer to `session` mode.
- Prepared statements, cursors, and listeners tied to a pooled connection become invalid or are removed when the connection is released back to the pool.
- The pool default size is `min_size=10` and `max_size=10`. Tune it deliberately instead of assuming it matches your service concurrency.
- For production TLS, do not rely on the default `'prefer'` mode if plaintext fallback is unacceptable.

## Recommended Workflow For Agents

1. Pin `asyncpg==0.31.0` unless the project already uses a newer reviewed version.
2. Choose one connection style early: DSN, explicit kwargs, or PostgreSQL service file.
3. Set SSL and password handling explicitly for deployed environments instead of relying on implicit local defaults.
4. Use `fetchval`, `fetchrow`, and `execute` first; reach for prepared statements or cursors only when there is a clear need.
5. For web services or workers, create one pool at startup and acquire connections per request or job.
6. If the schema uses JSON, `hstore`, PostGIS, or custom types, register codecs before writing business logic.

## Official Sources

- Installation: https://magicstack.github.io/asyncpg/current/installation.html
- Usage guide: https://magicstack.github.io/asyncpg/current/usage.html
- API reference: https://magicstack.github.io/asyncpg/current/api/index.html
- FAQ: https://magicstack.github.io/asyncpg/current/faq.html
- PyPI package page: https://pypi.org/project/asyncpg/
- `0.31.0` release notes: https://github.com/MagicStack/asyncpg/releases/tag/v0.31.0

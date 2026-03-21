---
name: package
description: "aiosqlite package guide for Python async SQLite access with asyncio"
metadata:
  languages: "python"
  versions: "0.22.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aiosqlite,sqlite,asyncio,database,python"
---

# aiosqlite Python Package Guide

## Golden Rule

Use `aiosqlite` when an asyncio application needs SQLite without blocking the event loop. Keep the connection open for a unit of work, `await` every DB operation, and close the connection explicitly or use `async with`.

`aiosqlite` mirrors most of the standard-library `sqlite3` API, but runs database work through a single worker thread per connection and exposes async methods for connection and cursor operations.

## Install

```bash
python -m pip install "aiosqlite==0.22.1"
```

Common alternatives:

```bash
uv add "aiosqlite==0.22.1"
poetry add "aiosqlite==0.22.1"
```

## Initialize A Connection

Use `async with` unless you have a strong reason to manage lifecycle manually:

```python
import asyncio
import aiosqlite

async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        await db.execute("""
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                email TEXT NOT NULL UNIQUE
            )
        """)
        await db.commit()

asyncio.run(main())
```

Manual lifecycle is valid, but you must close the connection:

```python
import aiosqlite

async def open_db() -> aiosqlite.Connection:
    db = await aiosqlite.connect("app.db")
    return db
```

## Core Usage

### Insert rows safely

Use SQL parameters instead of string formatting:

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        await db.execute(
            "INSERT INTO users (email) VALUES (?)",
            ("ada@example.com",),
        )
        await db.commit()
```

### Read rows with a cursor

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        async with db.execute(
            "SELECT id, email FROM users WHERE email LIKE ? ORDER BY id",
            ("%@example.com",),
        ) as cursor:
            rows = await cursor.fetchall()

        for row in rows:
            print(row[0], row[1])
```

### Stream rows with async iteration

This is the better default for large result sets:

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        async with db.execute("SELECT id, email FROM users ORDER BY id") as cursor:
            async for row in cursor:
                print(row)
```

### Return dictionary-like rows

`aiosqlite.Row` gives named access similar to `sqlite3.Row`:

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        db.row_factory = aiosqlite.Row

        async with db.execute("SELECT id, email FROM users LIMIT 1") as cursor:
            row = await cursor.fetchone()

        if row is not None:
            print(row["email"])
```

### Use connection helper shortcuts

`execute`, `execute_fetchall`, `execute_insert`, `executemany`, and `executescript` are exposed on the connection:

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        rows = await db.execute_fetchall(
            "SELECT id, email FROM users WHERE id > ?",
            (10,),
        )
```

## Transactions

`aiosqlite` follows SQLite transaction behavior from `sqlite3`:

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        await db.execute("UPDATE accounts SET balance = balance - 50 WHERE id = ?", (1,))
        await db.execute("UPDATE accounts SET balance = balance + 50 WHERE id = ?", (2,))
        await db.commit()
```

Rollback on error:

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        try:
            await db.execute("INSERT INTO audit_log(message) VALUES (?)", ("started",))
            await db.execute("INSERT INTO broken_table(value) VALUES (?)", ("x",))
            await db.commit()
        except Exception:
            await db.rollback()
            raise
```

If you want autocommit-style behavior, pass the same kind of connection settings you would use with `sqlite3`, for example `isolation_level=None`.

## Configuration

There is no auth layer. Configuration is mostly:

- database location: file path, `:memory:`, or SQLite URI
- `sqlite3.connect` keyword arguments forwarded through `aiosqlite.connect(...)`
- async iteration chunking through `iter_chunk_size`

Example with common SQLite options:

```python
import sqlite3
import aiosqlite

async def open_db() -> aiosqlite.Connection:
    return await aiosqlite.connect(
        "file:app.db?mode=rwc",
        uri=True,
        timeout=10,
        detect_types=sqlite3.PARSE_DECLTYPES | sqlite3.PARSE_COLNAMES,
        isolation_level=None,
        iter_chunk_size=128,
    )
```

Practical notes:

- `timeout` controls how long SQLite waits on a locked database before raising.
- `detect_types` enables adapter and converter behavior from `sqlite3`.
- `uri=True` lets you use SQLite URI filenames.
- `iter_chunk_size` controls how many rows are pulled per worker-thread chunk during async iteration.

## Useful sqlite3-Parity Features

### Register custom SQL functions

```python
import hashlib
import aiosqlite

def sha1_hex(value: str) -> str:
    return hashlib.sha1(value.encode("utf-8")).hexdigest()

async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        await db.create_function("sha1_hex", 1, sha1_hex, deterministic=True)
```

### Backup one database into another

```python
import aiosqlite

async def main() -> None:
    async with aiosqlite.connect("app.db") as source:
        async with aiosqlite.connect("backup.db") as target:
            await source.backup(target)
```

### Interrupt long-running work

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        await db.interrupt()
```

### Install progress or trace callbacks

```python
async def main() -> None:
    async with aiosqlite.connect("app.db") as db:
        await db.set_progress_handler(lambda: 0, 1000)
        await db.set_trace_callback(print)
```

`set_authorizer` is also available in `0.21.0+` if you need SQLite authorizer hooks.

## Concurrency Model

The package uses one dedicated thread per connection and executes actions through a shared request queue for that connection. This prevents overlapping SQLite calls on the same connection from blocking the event loop, but it does not make a single connection truly parallel.

Practical guidance:

- Reuse one connection for a small transactional unit of work.
- Open separate connections for independent concurrent tasks.
- Do not expect `asyncio.gather(...)` on the same connection to run queries in parallel.
- For write-heavy apps, expect SQLite locking rules to matter just as they do with `sqlite3`.

## Common Pitfalls

- Forgetting `await`: `db.execute(...)`, `cursor.fetchone()`, `db.commit()`, and `db.close()` are async.
- Forgetting `commit()`: inserts and updates are not persisted until committed unless you explicitly configure autocommit behavior.
- Leaking connections: `0.22.1` adds a `ResourceWarning` when connections are not closed before garbage collection.
- Sharing one connection for all concurrent work: operations serialize through one queue, so latency stacks up quickly.
- Using string interpolation in SQL: use placeholders and bound parameters.
- Expecting network-style auth or pooling features: this is a thin async wrapper around local SQLite, not a client/server database driver.
- Assuming the docs landing page is fully current on packaging constraints: the stable landing page still says Python 3.8+, but PyPI metadata and the changelog reflect the current support floor.

## Version-Sensitive Notes For 0.22.1

- `0.22.1` is the latest PyPI release as of March 11, 2026.
- `0.22.1` adds a `ResourceWarning` when a connection is garbage-collected without being closed.
- `0.22.0` changed `Connection` so it no longer subclasses `Thread`; if you had code relying on thread inheritance details, update it.
- `0.21.0` added `set_authorizer` support and dropped Python 3.8 support.
- PyPI currently declares `Requires-Python >=3.9`, which is the packaging constraint agents should trust when generating install or CI guidance.

## Official Sources

- Main docs: https://aiosqlite.omnilib.dev/en/stable/
- API reference: https://aiosqlite.omnilib.dev/en/stable/api.html
- Changelog: https://aiosqlite.omnilib.dev/en/stable/changelog.html
- PyPI metadata: https://pypi.org/project/aiosqlite/
- Python `sqlite3.connect`: https://docs.python.org/3/library/sqlite3.html#sqlite3.connect
- Python row factories: https://docs.python.org/3/library/sqlite3.html#how-to-create-and-use-row-factories

---
name: package
description: "databases package guide for Python async database access with SQLAlchemy Core"
metadata:
  languages: "python"
  versions: "0.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "databases,python,asyncio,sqlalchemy,postgresql,mysql,sqlite"
---

# databases Python Package Guide

## Golden Rule

Use `databases` as an async query layer on top of SQLAlchemy Core, not as an ORM. Pick and install the async driver that matches your database URL, connect and disconnect it with your app lifecycle, and keep a separate synchronous driver/toolchain for Alembic migrations or other synchronous SQLAlchemy operations.

## Install

Install the package and the async backend you actually use:

```bash
python -m pip install "databases==0.9.0"
python -m pip install "databases[asyncpg]==0.9.0"
python -m pip install "databases[aiosqlite]==0.9.0"
```

Available backend extras in the maintainer package metadata:

- PostgreSQL: `asyncpg` or alias `postgresql`
- PostgreSQL via psycopg2-compatible stack for sync tooling: install `psycopg2` separately
- MySQL: `aiomysql`, `asyncmy`, or alias `mysql`
- MySQL sync tooling: install `pymysql` separately
- SQLite: `aiosqlite` or alias `sqlite`

`databases 0.9.0` depends on `sqlalchemy>=2.0.7`, so use SQLAlchemy 2-style Core APIs and avoid old 1.3-era snippets.

## Setup And Initialization

`databases` works from a database URL and manages a connection pool behind the scenes.

SQLite:

```python
from databases import Database

database = Database("sqlite+aiosqlite:///./app.db")
```

PostgreSQL:

```python
from databases import Database

database = Database("postgresql+asyncpg://user:password@localhost:5432/appdb")
```

MySQL:

```python
from databases import Database

database = Database("mysql+asyncmy://user:password@localhost:3306/appdb")
```

Use it with an async context manager for scripts:

```python
from databases import Database

async def main() -> None:
    async with Database("sqlite+aiosqlite:///./app.db") as database:
        value = await database.fetch_val("SELECT 1")
        print(value)
```

For web apps, wire it into startup and shutdown instead of reconnecting per request:

```python
from databases import Database
from fastapi import FastAPI

DATABASE_URL = "postgresql+asyncpg://user:password@localhost:5432/appdb"

database = Database(DATABASE_URL)
app = FastAPI()

@app.on_event("startup")
async def startup() -> None:
    await database.connect()

@app.on_event("shutdown")
async def shutdown() -> None:
    await database.disconnect()
```

## Core Usage

### SQLAlchemy Core queries

Define tables with SQLAlchemy Core, then execute those expressions through `databases`:

```python
import sqlalchemy
from databases import Database

metadata = sqlalchemy.MetaData()

notes = sqlalchemy.Table(
    "notes",
    metadata,
    sqlalchemy.Column("id", sqlalchemy.Integer, primary_key=True),
    sqlalchemy.Column("text", sqlalchemy.String(length=200)),
    sqlalchemy.Column("completed", sqlalchemy.Boolean, nullable=False, server_default=sqlalchemy.false()),
)

database = Database("sqlite+aiosqlite:///./app.db")

async def list_notes() -> list[dict]:
    await database.connect()
    try:
        await database.execute(notes.insert(), {"text": "ship docs", "completed": False})
        rows = await database.fetch_all(notes.select().order_by(notes.c.id))
        return [dict(row._mapping) for row in rows]
    finally:
        await database.disconnect()
```

The result objects support attribute access and `row._mapping[...]`. When you need a plain dict, convert from `_mapping`.

### Raw SQL queries

Raw SQL uses named parameters in `:name` format:

```python
from databases import Database

database = Database("postgresql+asyncpg://user:password@localhost:5432/appdb")

async def create_note(text: str) -> int:
    query = """
    INSERT INTO notes(text, completed)
    VALUES (:text, :completed)
    RETURNING id
    """
    row = await database.fetch_one(query, {"text": text, "completed": False})
    return row.id
```

Common methods:

- `execute(...)`: run one statement
- `execute_many(...)`: bulk writes
- `fetch_one(...)`: one row
- `fetch_all(...)`: all rows in memory
- `fetch_val(...)`: a single scalar
- `iterate(...)`: async row streaming when you do not want to materialize everything

## Transactions And Connection Behavior

Transactions are async context managers:

```python
async with database.transaction():
    await database.execute(
        "INSERT INTO audit_log(message) VALUES (:message)",
        {"message": "starting work"},
    )
    await database.execute(
        "UPDATE notes SET completed = :completed WHERE id = :id",
        {"completed": True, "id": 1},
    )
```

Useful details from the maintainer docs:

- Connections are acquired per `asyncio.Task`
- Nested transactions are supported via savepoints
- You can specify an isolation level such as `isolation="serializable"` if the backend supports it
- If child tasks need to participate in the same transaction, share the same connection explicitly

For low-level control:

```python
transaction = await database.transaction()
try:
    await database.execute("DELETE FROM temp_rows")
except Exception:
    await transaction.rollback()
    raise
else:
    await transaction.commit()
```

## Configuration And Auth

`databases` does not have a separate auth layer. Authentication and connection tuning come from the database URL or keyword arguments.

Examples:

```python
Database(
    "postgresql+asyncpg://user:password@db.example.com/appdb?ssl=true",
    min_size=5,
    max_size=20,
)
```

```python
Database(
    "mysql+aiomysql://user:password@db.example.com/appdb",
    min_size=5,
    max_size=20,
)
```

Practical guidance:

- Prefer environment variables for credentials and build the URL once during app startup
- Match the URL scheme to the installed async driver: `postgresql+asyncpg`, `mysql+asyncmy`, `mysql+aiomysql`, `sqlite+aiosqlite`
- SSL, pool sizing, and similar options are backend-specific; keyword arguments are often clearer than URL query strings
- SQLite is the easiest local dev option, but it is not a drop-in substitute for PostgreSQL or MySQL behavior under concurrency

## Migrations And Schema Management

Do not expect `databases` to replace Alembic or SQLAlchemy schema tooling.

Important constraints from the official docs:

- `databases` does not use SQLAlchemy's engine internally for normal access
- `metadata.create_all()` is not part of the usual `databases` workflow
- serious projects should use Alembic for migrations
- Alembic and other synchronous SQLAlchemy tooling still need a synchronous driver

Minimal Alembic direction:

1. Install `alembic`
2. Point Alembic at your `DATABASE_URL`
3. Expose your SQLAlchemy `metadata` to `target_metadata`
4. For MySQL migrations, the docs call out `pymysql` as the synchronous dialect to use

## Testing

For strict per-test isolation, the official docs recommend force-rollback mode:

```python
from databases import Database

database = Database("postgresql+asyncpg://user:password@localhost/testdb", force_rollback=True)
```

That wraps work in a transaction that is rolled back when the database disconnects. A lower-level option is:

```python
async with database.transaction(force_rollback=True):
    ...
```

Use a dedicated test database URL rather than pointing tests at development data.

## Common Pitfalls

- Installing `databases` without an async driver. The base package is not enough for PostgreSQL, MySQL, or SQLite I/O.
- Using the wrong URL scheme for the installed driver.
- Forgetting `await database.connect()` before the first query or `await database.disconnect()` on shutdown.
- Treating it like SQLAlchemy ORM. `databases` is built around SQLAlchemy Core expressions and raw SQL.
- Calling synchronous schema helpers or Alembic commands without also installing a synchronous DB driver.
- Copying old snippets that assume SQLAlchemy 1.x. `0.9.0` added SQLAlchemy 2+ support and the package now depends on `sqlalchemy>=2.0.7`.
- Assuming rows are plain dicts. Use attribute access or `row._mapping`.
- Opening one transaction and then spawning unrelated tasks that expect to see the same transaction state without sharing the connection.

## Version-Sensitive Notes For 0.9.0

- `0.9.0` was released on March 1, 2024 and is still the latest PyPI release as of March 12, 2026.
- The `0.9.0` changelog notes three important changes: Python 3.7 support was dropped, Python 3.12 support was added, and SQLAlchemy 2+ support was added.
- PyPI still classifies the package as `Development Status :: 3 - Alpha`, so avoid assuming the maintenance posture of a more active database toolkit.
- The official GitHub repository was archived by the owner on August 19, 2025 and is now read-only. For new projects, treat `databases` as stable but effectively unmaintained unless the project is revived elsewhere.

## Official Sources

- Docs: https://www.encode.io/databases/
- Database queries: https://www.encode.io/databases/database_queries/
- Connections and transactions: https://www.encode.io/databases/connections_and_transactions/
- Tests and migrations: https://www.encode.io/databases/tests_and_migrations/
- PyPI: https://pypi.org/project/databases/
- Repository: https://github.com/encode/databases
- Changelog: https://raw.githubusercontent.com/encode/databases/master/CHANGELOG.md

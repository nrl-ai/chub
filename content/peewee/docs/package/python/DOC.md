---
name: package
description: "Peewee ORM package guide for Python projects using the official 4.0.1 docs"
metadata:
  languages: "python"
  versions: "4.0.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "peewee,python,orm,sqlite,postgresql,mysql,database"
---

# Peewee Python Package Guide

## Golden Rule

Use the official `peewee` package and write against the Peewee 4.x docs, not old 3.x blog posts. As of March 12, 2026, both the official docs and PyPI point to `peewee 4.0.1`.

## Install

Pin the version your project expects:

```bash
python -m pip install "peewee==4.0.1"
```

Driver extras from the maintainer docs:

```bash
python -m pip install "peewee[postgres]==4.0.1"
python -m pip install "peewee[psycopg3]==4.0.1"
python -m pip install "peewee[mysql]==4.0.1"
```

Async extras from the 4.x asyncio docs:

```bash
python -m pip install "peewee[aiosqlite]==4.0.1"
python -m pip install "peewee[aiopg]==4.0.1"
python -m pip install "peewee[asyncpg]==4.0.1"
python -m pip install "peewee[aiomysql]==4.0.1"
```

Common alternatives:

```bash
uv add "peewee==4.0.1"
poetry add "peewee==4.0.1"
```

## Initialize A Database

### Basic SQLite setup

```python
from peewee import (
    CharField,
    ForeignKeyField,
    Model,
    SqliteDatabase,
    TextField,
)

db = SqliteDatabase(
    "app.db",
    pragmas={
        "journal_mode": "wal",
        "foreign_keys": 1,
    },
    autoconnect=False,
)

class BaseModel(Model):
    class Meta:
        database = db

class User(BaseModel):
    username = CharField(unique=True)

class Note(BaseModel):
    user = ForeignKeyField(User, backref="notes", on_delete="CASCADE")
    body = TextField()

db.connect()
db.create_tables([User, Note])
```

Why this shape:

- `autoconnect=False` is safer for explicit connection handling.
- SQLite should usually enable `foreign_keys`.
- A shared `BaseModel` keeps `Meta.database` in one place.

### Configure from a database URL

Use `playhouse.db_url.connect()` when config comes from an environment variable:

```python
import os

from playhouse.db_url import connect

DATABASE_URL = os.environ["DATABASE_URL"]
db = connect(DATABASE_URL)
```

This supports URLs such as:

- `sqlite:///app.db`
- `postgresql://user:pass@localhost:5432/app`
- `mysql://user:pass@localhost:3306/app`

### Delay binding until runtime

If the database is chosen later, use `DatabaseProxy`:

```python
from peewee import DatabaseProxy, Model, PostgresqlDatabase, SqliteDatabase

database_proxy = DatabaseProxy()

class BaseModel(Model):
    class Meta:
        database = database_proxy

def init_db(testing: bool) -> None:
    if testing:
        database_proxy.initialize(SqliteDatabase(":memory:"))
    else:
        database_proxy.initialize(PostgresqlDatabase("app"))
```

## Define Models

```python
import datetime

from peewee import (
    AutoField,
    CharField,
    DateTimeField,
    ForeignKeyField,
    Model,
    TextField,
)

class BaseModel(Model):
    class Meta:
        database = db

class Blog(BaseModel):
    id = AutoField()
    title = CharField(max_length=200, unique=True)
    created_at = DateTimeField(default=datetime.datetime.utcnow)

class Entry(BaseModel):
    id = AutoField()
    blog = ForeignKeyField(Blog, backref="entries", on_delete="CASCADE")
    title = CharField(max_length=200)
    content = TextField()
    created_at = DateTimeField(default=datetime.datetime.utcnow)
```

Use `AutoField` for the normal integer primary key case. Use `ForeignKeyField(..., backref=...)` so reverse access is predictable.

## Core Query Patterns

### Create rows

```python
blog = Blog.create(title="Engineering")

entry = Entry.create(
    blog=blog,
    title="Peewee 4 notes",
    content="Ship the migration first.",
)
```

### Select and filter

```python
query = (
    Entry.select(Entry, Blog)
    .join(Blog)
    .where(Blog.title == "Engineering")
    .order_by(Entry.created_at.desc())
)

for entry in query:
    print(entry.title, entry.blog.title)
```

### Update and delete

```python
(
    Entry.update(content="Updated content")
    .where(Entry.id == entry.id)
    .execute()
)

(
    Entry.delete()
    .where(Entry.created_at < datetime.datetime(2026, 1, 1))
    .execute()
)
```

### Bulk inserts and upserts

```python
rows = [
    {"title": "A"},
    {"title": "B"},
]

Blog.insert_many(rows).execute()

(
    Blog.insert(title="Engineering")
    .on_conflict_ignore()
    .execute()
)
```

Use bulk queries for large writes instead of looping over `.create()`.

## Transactions

Wrap multi-step writes in `db.atomic()`:

```python
with db.atomic():
    blog = Blog.create(title="Infra")
    Entry.create(blog=blog, title="Runbook", content="...")
```

Nested `atomic()` blocks use savepoints when the backend supports them.

## Async Support In 4.x

Peewee 4 adds first-class asyncio support under `playhouse.pwasyncio`.

```python
import asyncio

from peewee import CharField
from playhouse.pwasyncio import AsyncSqliteDatabase

db = AsyncSqliteDatabase("app.db")

class User(db.Model):
    username = CharField(unique=True)

async def main() -> None:
    async with db:
        await db.create_tables([User])

        user = await User.create(username="huey")
        fetched = await User.get(User.username == "huey")
        print(user.id, fetched.username)

        rows = await db.run(User.select().where(User.username.startswith("h")))
        print([row.username for row in rows])

asyncio.run(main())
```

Use the async APIs consistently inside async code. Do not mix synchronous query execution into an async request path.

## Schema Migrations

Use `playhouse.migrate` for straightforward column and table changes:

```python
from playhouse.migrate import SqliteMigrator, migrate

migrator = SqliteMigrator(db)

with db.atomic():
    migrate(
        migrator.add_column("entry", "slug", CharField(null=True)),
    )
```

For larger production migrations, treat Peewee's migrator as a low-level tool and still plan rollout steps yourself.

## Useful Playhouse Extensions

The official `playhouse` modules cover common production needs:

- `playhouse.db_url`: parse a `DATABASE_URL` string into the right database class
- `playhouse.pool`: pooled Postgres/MySQL/SQLite database classes
- `playhouse.migrate`: schema migrations
- `playhouse.shortcuts`: helpers like `model_to_dict()` and `dict_to_model()`
- `playhouse.signals`: model lifecycle hooks when you need them

## Common Pitfalls

### 1. Manage connections explicitly

In web apps or workers, open a connection at request/job start and close it at the end. Do not rely on implicit autoconnect in long-running services.

Typical pattern:

```python
def handle_request() -> None:
    db.connect(reuse_if_open=True)
    try:
        ...
    finally:
        if not db.is_closed():
            db.close()
```

### 2. SQLite defaults are not enough by themselves

If you need enforced foreign keys in SQLite, set the pragma. Without it, deletes and cascades may not behave like Postgres.

### 3. Field defaults are usually Python-side

Peewee applies `default=` in Python. That does not create a database-level default constraint. Use explicit constraints if the database itself must own the default.

For mutable values, pass a callable such as `default=dict` or `default=list` on the relevant field type. Do not use `default={}` or `default=[]`.

### 4. Query expressions use Peewee operators, not Python boolean operators

Use:

```python
Blog.select().where((Blog.title != "") & (Blog.title != "admin"))
```

Do not use `and` / `or` inside `.where(...)`, and use `.in_(...)` instead of Python's `in`.

### 5. Avoid N+1 queries

When you know you need related rows, join or prefetch instead of lazy-loading each foreign key in a loop.

```python
from peewee import prefetch

blogs = Blog.select()
entries = Entry.select()

for blog in prefetch(blogs, entries):
    print(blog.title, [entry.title for entry in blog.entries])
```

### 6. Stream large result sets

Normal iteration caches rows. For large exports, use `.iterator()` and narrower row formats like `.tuples()` or `.dicts()` when you do not need model instances.

```python
query = Entry.select(Entry.id, Entry.title).tuples().iterator()

for entry_id, title in query:
    print(entry_id, title)
```

## Version-Sensitive Notes For 4.0.1

- Peewee 4 is the major-version line documented at `docs.peewee-orm.com/en/latest/`; do not assume 3.x blog examples still match 4.x behavior.
- The official changelog says 4.0 adds asyncio support and `psycopg3` support.
- The 4.0.1 changelog removes `SqliteExtDatabase`; use `SqliteDatabase` instead.
- If you find older snippets importing from legacy SQLite extension classes or third-party async wrappers, rewrite them for the built-in 4.x async modules before copying them into production code.

## Official Sources

- Docs root: https://docs.peewee-orm.com/en/latest/
- Quickstart: https://docs.peewee-orm.com/en/latest/peewee/quickstart.html
- Database config: https://docs.peewee-orm.com/en/latest/peewee/database.html
- Querying: https://docs.peewee-orm.com/en/latest/peewee/querying.html
- Asyncio: https://docs.peewee-orm.com/en/latest/peewee/asyncio.html
- Playhouse extensions: https://docs.peewee-orm.com/en/latest/peewee/playhouse.html
- Changelog: https://github.com/coleifer/peewee/blob/master/CHANGELOG.md
- PyPI: https://pypi.org/project/peewee/

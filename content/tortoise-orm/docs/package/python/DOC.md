---
name: package
description: "Tortoise ORM async Python ORM for SQLite, PostgreSQL, MySQL, MSSQL, and Oracle"
metadata:
  languages: "python"
  versions: "1.1.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tortoise-orm,python,orm,async,database,sqlite,postgresql,mysql"
---

# Tortoise ORM Python Package Guide

## Golden Rule

Use `tortoise-orm` for async ORM access, initialize it once per app lifecycle, and close connections explicitly on shutdown. For schema changes on `1.x`, use the built-in `tortoise` migration CLI rather than older Aerich-based guides unless you are maintaining legacy code.

## Install

Pick the extra that matches your database driver:

```bash
python -m pip install "tortoise-orm==1.1.6"
python -m pip install "tortoise-orm[asyncpg]==1.1.6"
python -m pip install "tortoise-orm[psycopg]==1.1.6"
python -m pip install "tortoise-orm[asyncmy]==1.1.6"
python -m pip install "tortoise-orm[aiomysql]==1.1.6"
python -m pip install "tortoise-orm[asyncodbc]==1.1.6"
```

Useful extras:

```bash
python -m pip install "tortoise-orm[accel]==1.1.6"
python -m pip install "tortoise-orm[ipython]==1.1.6"
```

Notes:

- No extra is needed for SQLite.
- `[accel]` installs optional performance helpers such as `orjson`, `uvloop`, and `ciso8601`.
- Supported databases are SQLite, PostgreSQL, MySQL/MariaDB, MSSQL, and Oracle, but production usage is usually SQLite for local/dev and PostgreSQL or MySQL for deployed services.

## Initialize And Configure

### Minimal script or local development setup

Use `run_async()` for simple scripts. `generate_schemas()` is for empty databases and development only.

```python
from tortoise import Tortoise, fields, run_async
from tortoise.models import Model

class User(Model):
    id = fields.IntField(primary_key=True)
    email = fields.CharField(max_length=255, unique=True)
    is_active = fields.BooleanField(default=True)

async def main() -> None:
    await Tortoise.init(
        db_url="sqlite://db.sqlite3",
        modules={"models": ["__main__"]},
    )
    await Tortoise.generate_schemas()

    user = await User.create(email="a@example.com")
    print(user.id)

run_async(main())
```

### Production-style config dict

Use dict config when you need multiple connections, per-app routing, or passwords that are awkward to encode safely into a URL.

```python
from tortoise import Tortoise

TORTOISE_ORM = {
    "connections": {
        "default": {
            "engine": "tortoise.backends.asyncpg",
            "credentials": {
                "host": "127.0.0.1",
                "port": 5432,
                "user": "app",
                "password": "secret",
                "database": "appdb",
                "minsize": 1,
                "maxsize": 10,
            },
        }
    },
    "apps": {
        "models": {
            "models": ["app.models"],
            "default_connection": "default",
            "migrations": "app.migrations",
        }
    },
}

async def init_orm() -> None:
    await Tortoise.init(config=TORTOISE_ORM)
```

### DB URL examples

```text
sqlite://db.sqlite3
sqlite://:memory:
asyncpg://user:password@127.0.0.1:5432/appdb
psycopg://user:password@127.0.0.1:5432/appdb
mysql://user:password@127.0.0.1:3306/appdb
```

If the password contains `%`, `+`, `/`, or other special characters, prefer dict config or URL-encode the password first.

## App Lifecycle And Cleanup

Tortoise keeps async DB connections open until you close them.

```python
from tortoise import Tortoise

async def startup() -> None:
    await Tortoise.init(config=TORTOISE_ORM)

async def shutdown() -> None:
    await Tortoise.close_connections()
```

If initialization happens in a different task from the code that runs queries, enable global fallback:

```python
await Tortoise.init(
    db_url="sqlite://db.sqlite3",
    modules={"models": ["app.models"]},
    _enable_global_fallback=True,
)
```

That is commonly needed in ASGI lifespan handlers, framework setup tasks, and some test harnesses. Only one global fallback context can be active at a time.

## Define Models

```python
from tortoise import fields
from tortoise.models import Model
from tortoise.expressions import RawSQL

class Team(Model):
    id = fields.IntField(primary_key=True)
    name = fields.CharField(max_length=255, unique=True)

class Tournament(Model):
    id = fields.IntField(primary_key=True)
    name = fields.CharField(max_length=255)

class Event(Model):
    id = fields.IntField(primary_key=True)
    name = fields.CharField(max_length=255)
    tournament = fields.ForeignKeyField("models.Tournament", related_name="events")
    participants = fields.ManyToManyField("models.Team", related_name="events")
    created_at = fields.DatetimeField(db_default=RawSQL("CURRENT_TIMESTAMP"))
```

Notes:

- String relations use `"app_label.ModelName"`.
- `ForeignKeyField()` and `ManyToManyField()` can also accept model classes directly on `1.x`.
- Use `db_default` for database-level defaults. Plain `default=` is Python-side only.

## Core CRUD

```python
from tortoise.exceptions import DoesNotExist

tournament = await Tournament.create(name="Spring Cup")
team = await Team.create(name="Blue Team")

event = await Event.create(name="Opening Match", tournament=tournament)
await event.participants.add(team)

same_event = await Event.get(id=event.id)
await Event.filter(id=event.id).update(name="Updated Match")

try:
    fetched = await Event.get(name="Updated Match")
except DoesNotExist:
    fetched = None

all_events = await Event.filter(tournament__name="Spring Cup").order_by("-id")
lightweight_rows = await Event.filter(id__in=[event.id]).values("id", "name")
```

Use:

- `.create()` or `.save()` to insert
- `.filter()`, `.get()`, `.get_or_none()`, `.first()` for reads
- `.update()` for bulk updates without loading objects first
- `.values()` or `.values_list()` when you want dict/tuple output and fewer ORM objects

## Relations And Query Shaping

`prefetch_related()` is the normal way to load related objects:

```python
events = await Event.all().prefetch_related("tournament", "participants")
```

For foreign keys, `select_related()` can fetch related objects in one joined query:

```python
events = await Event.all().select_related("tournament")
```

Performance notes:

- Each `prefetch_related()` depth adds more queries. `events__participants` means one extra query per relation depth.
- When you only need flattened fields, prefer `values()` or `values_list()` instead of full ORM instances.
- Many-to-many relations require both sides to be saved before `.add()` works.

## Transactions

Use `in_transaction()` when several writes must succeed or fail together.

```python
from tortoise.transactions import in_transaction

async with in_transaction() as connection:
    event = await Event.create(name="Transactional", tournament=tournament, using_db=connection)
    await Event.filter(id=event.id).using_db(connection).update(name="Committed")
```

Use `select_for_update()` inside a transaction when row locking matters:

```python
async with in_transaction():
    event = await Event.filter(id=event.id).select_for_update().get()
    event.name = "Locked update"
    await event.save()
```

## Migrations

The built-in migration system is the current recommended path.

```python
TORTOISE_ORM = {
    "connections": {"default": "sqlite://db.sqlite3"},
    "apps": {
        "models": {
            "models": ["app.models"],
            "default_connection": "default",
            "migrations": "app.migrations",
        }
    },
}
```

```bash
tortoise init
tortoise makemigrations
tortoise migrate
tortoise sqlmigrate models 0001_initial
```

The CLI resolves config from `-c/--config`, `--config-file`, or `[tool.tortoise]` in `pyproject.toml`.

Use `generate_schemas()` only for empty local databases, tests, or throwaway prototypes. Do not mix it into normal production deployments after real schema history exists.

## Pydantic Serialization

Tortoise ships a Pydantic plugin for output serialization:

```python
from tortoise.contrib.pydantic import pydantic_model_creator

EventOut = pydantic_model_creator(Event, name="EventOut")

event = await Event.get(id=1).prefetch_related("participants")
payload = await EventOut.from_tortoise_orm(event)
```

Important limitation:

- The plugin is for serialization. It does not generate deserialization/write models for you.

## Framework Usage

For FastAPI, Starlette, Sanic, AIOHTTP, and similar frameworks, treat Tortoise as app-lifecycle state:

- initialize once on startup or lifespan entry
- close connections on shutdown
- enable `_enable_global_fallback=True` when the framework runs startup work in a different task and queries later fail with `No TortoiseContext is currently active`

If you are writing plain scripts, `run_async()` already handles connection cleanup.

## Common Pitfalls

- Do not use `generate_schemas()` as your migration strategy for an app with real data.
- Do not forget `await Tortoise.close_connections()` on shutdown.
- `use_tz` defaults to `True` on `1.x`. If your project expects naive datetimes, set `use_tz=False` explicitly.
- `default=` is not the same as `db_default=`. Use `db_default` when the database itself should own the default clause.
- Older blog posts still point to Aerich. The upstream docs now recommend the built-in `tortoise` migration system.
- If queries run in a different task from initialization, you can hit `RuntimeError: No TortoiseContext is currently active`.
- If your DB password contains special URL characters, URL parsing can break. Prefer dict config in that case.
- `prefetch_related()` can turn into many queries. Use `select_related()` for foreign keys and `values()` when you only need a projection.
- Pydantic support is output-focused. You still need explicit request/write schemas in web apps.

## Version-Sensitive Notes For 1.1.6

- `1.0.0` was a breaking release: Python `3.10+` is required, `use_tz=True` became the default, and `Tortoise.init()` moved to a context-first architecture.
- `1.0.0` also introduced the native migration framework and deprecated `from tortoise import connections` in favor of `get_connection()` / `get_connections()`.
- `1.1.0` added `db_default`, which is now the correct way to emit database `DEFAULT` clauses in schemas and migrations.
- `1.1.1` clarified that `Field(default=...)` and `auto_now` / `auto_now_add` are Python-side behavior, not DB-level defaults.
- `1.1.6` fixes migration ordering for indexes and constraints, `db_default` handling in `CreateModel`, `max_length` change detection in `AlterField`, MySQL timezone handling when `use_tz=True`, and `+` in DB URL passwords.

## Official Sources

- Docs root: `https://tortoise.github.io/`
- Getting started: `https://tortoise.github.io/getting_started.html`
- Setup: `https://tortoise.github.io/setup.html`
- Databases: `https://tortoise.github.io/databases.html`
- Query API: `https://tortoise.github.io/query.html`
- Models: `https://tortoise.github.io/models.html`
- Migrations: `https://tortoise.github.io/migration.html`
- Pydantic serialization: `https://tortoise.github.io/contrib/pydantic.html`
- Changelog: `https://tortoise.github.io/CHANGELOG.html`
- PyPI: `https://pypi.org/project/tortoise-orm/`

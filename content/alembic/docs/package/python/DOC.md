---
name: package
description: "Alembic package guide for Python database schema migrations with SQLAlchemy"
metadata:
  languages: "python"
  versions: "1.18.4"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "alembic,sqlalchemy,migrations,database,schema"
---

# alembic Python Package Guide

## What Alembic Is For

Alembic is SQLAlchemy's migration tool for managing schema changes over time. Use it to:

- initialize a migration environment for a project
- generate revision files
- autogenerate candidate schema diffs from SQLAlchemy metadata
- upgrade or downgrade a database to a target revision
- work with branched migration histories and merge points
- emit SQL scripts instead of executing DDL directly

Alembic is most effective when your application already has a clear SQLAlchemy metadata model and you treat generated migrations as drafts that still need review.

## Version And Compatibility

- Package covered: `alembic==1.18.4`
- PyPI release date: `2026-02-10`
- PyPI metadata for `1.18.4` requires Python `>=3.10`
- Alembic docs still say Python `3.9+`; treat the PyPI package metadata as the safer installation requirement for this release
- Alembic works with SQLAlchemy `>=1.4`

Alembic does not follow SemVer. The docs explicitly describe the middle digit as a "Significant Minor Release", so pin by major/minor when you need stability:

```bash
pip install "alembic~=1.18.0"
```

## Installation

```bash
pip install "alembic~=1.18.0"
```

Common project-level installs:

```bash
uv add "alembic~=1.18.0"
poetry add "alembic~=1.18.0"
```

If your project uses async SQLAlchemy drivers, you still install normal `alembic`; the async behavior comes from the generated `env.py` template and SQLAlchemy async engine setup.

## Initialize A Migration Environment

Basic setup:

```bash
alembic init alembic
```

Useful templates documented upstream:

- `generic`: default single-database setup
- `pyproject`: stores Alembic source-configuration in `pyproject.toml`
- `async`: bootstrap for async DBAPI projects
- `multidb`: rudimentary multi-database setup

For a modern Python project already using `pyproject.toml`, prefer:

```bash
alembic init --template pyproject alembic
```

For async SQLAlchemy projects, prefer:

```bash
alembic init -t async alembic
```

The generated environment typically includes:

- `alembic.ini`
- `alembic/env.py`
- `alembic/script.py.mako`
- `alembic/versions/`

## Minimal Setup That Must Be Correct

### 1. Point Alembic At The Right Database

The default templates read the database URL from `sqlalchemy.url`:

```ini
[alembic]
sqlalchemy.url = postgresql+psycopg://user:pass@localhost/app
```

The tutorial is explicit that `sqlalchemy.url` is consumed by the user-maintained `env.py` script, so it is normal to replace this with environment-driven configuration.

### 2. Set `target_metadata`

Autogenerate does not work until `env.py` imports your SQLAlchemy metadata and assigns it to `target_metadata`.

```python
# alembic/env.py
from myapp.db import Base

target_metadata = Base.metadata
```

If your app has multiple metadata collections, Alembic can use a sequence:

```python
target_metadata = [Model1Base.metadata, Model2Base.metadata]
```

### 3. If You Override The URL In `env.py`, Escape `%`

When using `Config.set_main_option()` or `set_section_option()`, Alembic passes the value through `ConfigParser`, so raw percent signs must be doubled.

```python
import os

from alembic import context

config = context.config
database_url = os.environ["DATABASE_URL"]
config.set_main_option("sqlalchemy.url", database_url.replace("%", "%%"))
```

This matters for URLs that contain percent-encoded credentials.

## Core CLI Workflow

Create a blank revision:

```bash
alembic revision -m "create users table"
```

Autogenerate a candidate revision from metadata vs. live database:

```bash
alembic revision --autogenerate -m "add users table"
```

Apply all pending revisions:

```bash
alembic upgrade head
```

Step back one revision:

```bash
alembic downgrade -1
```

Inspect migration state:

```bash
alembic current
alembic history
alembic heads
```

Check whether model changes would produce a non-empty autogenerate diff:

```bash
alembic check
```

`alembic check` is useful in CI because it runs the autogenerate comparison without creating a new revision file.

## Typical Revision Contents

Generated files are normal Python migration scripts. Expect to edit them.

```python
from alembic import op
import sqlalchemy as sa

def upgrade() -> None:
    op.add_column("account", sa.Column("last_transaction_date", sa.DateTime()))

def downgrade() -> None:
    op.drop_column("account", "last_transaction_date")
```

Upstream guidance is clear: autogenerate produces candidate migrations, not finished migrations.

## Programmatic Usage

Alembic can run from Python code instead of the CLI.

```python
from alembic import command, config

cfg = config.Config("alembic.ini")
command.upgrade(cfg, "head")
```

To reuse an existing SQLAlchemy connection or transaction, pass it through `Config.attributes` and make `env.py` read it:

```python
from alembic import command, config

cfg = config.Config("alembic.ini")

with engine.begin() as connection:
    cfg.attributes["connection"] = connection
    command.upgrade(cfg, "head")
```

This is the upstream pattern for application-managed transactions.

## Async Projects

Alembic does not expose a separate async API, but it supports SQLAlchemy async engines through the generated environment.

The documented pattern is:

- bootstrap with the async template
- use `async_engine_from_config(...)` in `run_migrations_online()`
- call `await connection.run_sync(do_run_migrations)`

This lets you keep using normal Alembic commands like:

```bash
alembic upgrade head
```

with an async database URL configured in the environment.

## Existing Databases And `stamp`

If you create all tables directly from SQLAlchemy metadata for a fresh install, you can align Alembic with the current schema without replaying every old migration:

```python
from alembic import command
from alembic.config import Config

my_metadata.create_all(engine)

cfg = Config("alembic.ini")
command.stamp(cfg, "head")
```

Use this only when the live schema already matches the latest model state.

## Offline SQL Script Generation

If DDL must be reviewed or run by DBAs, generate SQL instead of executing directly:

```bash
alembic upgrade head --sql > migration.sql
```

Offline mode can also target a specific revision range:

```bash
alembic upgrade START_REV:END_REV --sql > migration.sql
```

That `start:end` syntax is specifically for offline mode.

## Branches And Merge Points

Alembic supports branched revision graphs. If you have multiple heads:

```bash
alembic heads
alembic current
```

`alembic upgrade head` becomes ambiguous when more than one head exists. In that case, either target a specific branch, use `heads`, or create a merge revision:

```bash
alembic merge -m "merge feature branches" heads
```

Use merge revisions when two independent migration lines both need to remain valid upgrade paths.

## Naming Conventions Matter

Alembic autogenerate cannot reliably reason about anonymously named constraints. If you use SQLAlchemy naming conventions, autogenerate and operations behave much more predictably.

Typical pattern:

```python
from sqlalchemy import MetaData
from sqlalchemy.orm import DeclarativeBase

class Base(DeclarativeBase):
    metadata = MetaData(
        naming_convention={
            "ix": "ix_%(column_0_label)s",
            "uq": "uq_%(table_name)s_%(column_0_name)s",
            "ck": "ck_%(table_name)s_%(constraint_name)s",
            "fk": "fk_%(table_name)s_%(column_0_name)s_%(referred_table_name)s",
            "pk": "pk_%(table_name)s",
        }
    )
```

Do this before you depend heavily on autogenerate.

## SQLite And Batch Migrations

SQLite has very limited `ALTER TABLE` support. Alembic handles this through batch migrations:

```python
with op.batch_alter_table("some_table") as batch_op:
    batch_op.add_column(sa.Column("foo", sa.Integer()))
    batch_op.drop_column("bar")
```

Important SQLite-specific behaviors:

- Alembic may recreate the table with a move-and-copy workflow
- named CHECK and foreign key constraints need extra care
- foreign key enforcement can block the required drop/rename cycle
- unnamed constraints are especially awkward during batch operations

If you are migrating SQLite schemas, review generated batch migrations more carefully than usual.

## Common Pitfalls

- Autogenerate is not exact. Always review the file it creates.
- Table renames and column renames are not detected as renames; they show up as add/drop pairs and must be hand-edited.
- Unnamed constraints are not reliably detectable. Use naming conventions.
- If your database contains tables outside your ORM metadata, autogenerate may try to drop them unless you filter with `include_name`.
- `compare_server_default=True` is optional and can produce false positives; enable it deliberately.
- For projects with many schemas, configure `include_schemas` and `include_name` intentionally.
- Data migrations are application-specific. Alembic's cookbook recommends keeping only small data changes inline and using separate scripts for more complex migrations.
- Multiple heads are normal in collaborative workflows, but you need an explicit merge strategy.

## Version-Sensitive Notes

- `1.15.0` was yanked because `alembic init` packaging was broken. Do not pin to `1.15.0`; use `1.15.1+`.
- `1.15.x+` dropped support for SQLAlchemy `<1.4`.
- `1.16.0` added `pyproject.toml` support for source-configuration and introduced `path_separator`, which is relevant if you split `version_locations` or `prepend_sys_path`.
- `1.17.1` added `alembic current --check-heads`, which is useful for tests and deployment checks.
- `1.18.0` added the plugin system and changed autogenerate reflection to use SQLAlchemy 2.0 bulk inspector methods for some dialects.
- `1.18.3` fixed an autogenerate regression from `1.18.0` affecting certain foreign key reflections when tables were filtered or reflected across remote schemas. If you depend on complex autogenerate filtering, prefer `>=1.18.3`.

## Official Sources

- Documentation root: https://alembic.sqlalchemy.org/en/latest/
- Tutorial: https://alembic.sqlalchemy.org/en/latest/tutorial.html
- Autogenerate guide: https://alembic.sqlalchemy.org/en/latest/autogenerate.html
- Branches guide: https://alembic.sqlalchemy.org/en/latest/branches.html
- Naming constraints: https://alembic.sqlalchemy.org/en/latest/naming.html
- Batch migrations: https://alembic.sqlalchemy.org/en/latest/batch.html
- Offline mode: https://alembic.sqlalchemy.org/en/latest/offline.html
- Cookbook: https://alembic.sqlalchemy.org/en/latest/cookbook.html
- Config API: https://alembic.sqlalchemy.org/en/latest/api/config.html
- Changelog: https://alembic.sqlalchemy.org/en/latest/changelog.html
- PyPI: https://pypi.org/project/alembic/

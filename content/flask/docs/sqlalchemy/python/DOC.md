---
name: sqlalchemy
description: "Flask-SQLAlchemy extension guide for integrating SQLAlchemy 2.x with Flask applications"
metadata:
  languages: "python"
  versions: "3.1.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,sqlalchemy,orm,database,python,web"
---

# Flask-SQLAlchemy Python Guide

## Install

Install the extension at the pinned version your project expects:

```bash
python -m pip install "flask-sqlalchemy==3.1.1"
```

Common alternatives:

```bash
uv add "flask-sqlalchemy==3.1.1"
poetry add "flask-sqlalchemy==3.1.1"
```

For schema migrations, add Alembic or Flask-Migrate separately. `db.create_all()` is useful for quickstarts, tests, and throwaway prototypes, but it is not a replacement for migrations in a real app.

## Recommended App Factory Setup

Configure the app before calling `db.init_app(app)`. Flask-SQLAlchemy reads engine configuration during initialization and does not re-read it later.

```python
from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column

class Base(DeclarativeBase):
    pass

db = SQLAlchemy(model_class=Base)

class User(db.Model):
    id: Mapped[int] = mapped_column(primary_key=True)
    username: Mapped[str] = mapped_column(unique=True)

def create_app() -> Flask:
    app = Flask(__name__, instance_relative_config=True)
    app.config.from_mapping(
        SQLALCHEMY_DATABASE_URI="sqlite:///app.db",
        SQLALCHEMY_ENGINE_OPTIONS={"pool_pre_ping": True},
    )

    db.init_app(app)

    with app.app_context():
        db.create_all()

    return app
```

Important setup rules:

- Define `db = SQLAlchemy(...)` once at module scope, then call `db.init_app(app)` inside the app factory.
- Import or define all models before `db.create_all()` so SQLAlchemy knows which tables exist.
- Prefer `model_class=Base` with SQLAlchemy 2.x typed mappings instead of older untyped model patterns.
- Keep the database URL in config or environment variables, not inline in reusable library code.

## Configuration

At least one of these must be configured before `init_app`:

- `SQLALCHEMY_DATABASE_URI`
- `SQLALCHEMY_BINDS`

Common config keys:

- `SQLALCHEMY_DATABASE_URI`: default bind URL
- `SQLALCHEMY_BINDS`: named extra engines for multi-database apps
- `SQLALCHEMY_ENGINE_OPTIONS`: default `create_engine()` options for the default bind
- `SQLALCHEMY_ECHO`: log SQLAlchemy statements for debugging
- `SQLALCHEMY_RECORD_QUERIES`: enable query recording for inspection helpers
- `SQLALCHEMY_TRACK_MODIFICATIONS`: enables model change signals

Example environment-driven setup:

```python
import os

app.config["SQLALCHEMY_DATABASE_URI"] = os.environ["DATABASE_URL"]
app.config["SQLALCHEMY_ENGINE_OPTIONS"] = {
    "pool_pre_ping": True,
    "pool_recycle": 1800,
}
app.config["SQLALCHEMY_ECHO"] = os.getenv("SQLALCHEMY_ECHO", "").lower() == "true"
```

SQLite note:

- `sqlite:///project.db` is relative to the Flask instance path, not necessarily the current working directory.

## Models

Use typed SQLAlchemy 2.x models. Flask-SQLAlchemy exposes `db.Model`, `db.session`, and helper metadata, but mapped columns and relationships are still standard SQLAlchemy.

```python
from __future__ import annotations

from sqlalchemy import ForeignKey, String
from sqlalchemy.orm import Mapped, mapped_column, relationship

class Author(db.Model):
    id: Mapped[int] = mapped_column(primary_key=True)
    name: Mapped[str] = mapped_column(String(120), unique=True)
    books: Mapped[list["Book"]] = relationship(back_populates="author")

class Book(db.Model):
    id: Mapped[int] = mapped_column(primary_key=True)
    title: Mapped[str] = mapped_column(String(200))
    author_id: Mapped[int] = mapped_column(ForeignKey("author.id"))
    author: Mapped[Author] = relationship(back_populates="books")
```

Use `db.create_all()` only for initial table creation in simple setups:

```python
with app.app_context():
    db.create_all()
```

It will not alter existing tables to match later model changes. Use migrations for that.

## Core Query and Session Patterns

For new code, prefer `select()` and `db.session.execute()` / `db.session.scalars()` over the legacy query interface.

Insert and commit:

```python
author = Author(name="Octavia Butler")
db.session.add(author)
db.session.commit()
```

Select rows:

```python
from sqlalchemy import select

stmt = select(Author).order_by(Author.name)
authors = db.session.scalars(stmt).all()
```

Fetch one row or fail with a 404 in a Flask view:

```python
from flask import Blueprint

bp = Blueprint("authors", __name__)

@bp.get("/authors/<int:author_id>")
def get_author(author_id: int):
    author = db.get_or_404(Author, author_id)
    return {"id": author.id, "name": author.name}
```

Run a filtered scalar query and fail with 404:

```python
from sqlalchemy import select

stmt = select(Author).where(Author.name == "Octavia Butler")
author = db.one_or_404(stmt)
```

Pagination:

```python
from sqlalchemy import select

page = db.paginate(
    select(Book).order_by(Book.title),
    page=1,
    per_page=20,
    max_per_page=100,
)

for book in page.items:
    print(book.title)
```

`db.paginate()` is meant for request-driven list views and can read `page` and `per_page` from request args when called in a Flask request context.

## Application Context Rules

Access to `db.session`, `db.engine`, and model queries requires an active Flask application context.

Typical cases that need a context:

- request handlers
- CLI commands registered with Flask
- startup or bootstrap code
- tests

Manual context usage:

```python
app = create_app()

with app.app_context():
    user_count = db.session.scalar(db.select(db.func.count()).select_from(User))
```

Without a context, Flask-SQLAlchemy raises `RuntimeError: Working outside of application context.` Keep session work inside request handlers, CLI commands, or explicit `with app.app_context():` blocks.

For tests, create a real app fixture and push a context only around the test or fixture scope that needs DB access. Do not keep one global context alive for the entire test suite unless you understand the cleanup tradeoffs.

## Multiple Databases With Binds

Use binds when a model belongs to a different database than the default bind.

```python
app.config["SQLALCHEMY_DATABASE_URI"] = "postgresql+psycopg://app:secret@localhost/main"
app.config["SQLALCHEMY_BINDS"] = {
    "meta": "sqlite:////tmp/meta.db",
    "audit": {
        "url": "postgresql+psycopg://app:secret@localhost/audit",
        "pool_recycle": 1200,
    },
}
```

Bind a model to one of those engines:

```python
class AuditLog(db.Model):
    __bind_key__ = "audit"

    id: Mapped[int] = mapped_column(primary_key=True)
    event: Mapped[str]
```

Important:

- The bind key is attached when the model or table is defined.
- Changing `__bind_key__` after definition does not move an existing table to a new engine.
- Use `db.create_all(bind_key="audit")` or `db.create_all()` inside an app context if you need Flask-SQLAlchemy to create tables for bound engines in simple setups.

## Working With Raw SQL

Drop to standard SQLAlchemy APIs when you need raw SQL or engine-level work:

```python
from sqlalchemy import text

with db.engine.begin() as conn:
    conn.execute(text("INSERT INTO user (username) VALUES (:username)"), {"username": "ada"})
```

Use `db.session` for ORM unit-of-work behavior. Use `db.engine` or `db.engines["bind_name"]` for connection-level operations that should bypass the ORM session.

## Common Pitfalls

- Do not call `db.session`, `db.engine`, or query helpers outside an app context.
- Do not set `SQLALCHEMY_DATABASE_URI`, binds, or engine options after `db.init_app(app)` and expect them to be picked up.
- Do not rely on `db.create_all()` for schema migrations after your tables already exist.
- Do not keep using `Model.query` or `session.query()` for new code just because older Flask examples still do.
- Do not forget to import model modules before `db.create_all()` or your tables may not be registered.
- Do not assume a SQLite URL is relative to the repo root; in Flask it is usually relative to the app instance path.
- Do not mutate a model's `__bind_key__` after class creation and expect routing to change.

## Version-Sensitive Notes

- `3.1.0` adds the `model_class` parameter so you can provide your own SQLAlchemy 2.x declarative base and use fully typed mappings.
- `3.1.0` also raises the minimum SQLAlchemy version to `2.0.16` and drops Python 3.7 support.
- `3.0` changed session scoping to the current app context instead of thread-local behavior and removed the old default in-memory SQLite engine. New apps must configure a database URI or binds explicitly.
- `3.1.1` deprecates the `__version__` attribute. Use `importlib.metadata.version("flask-sqlalchemy")` if code needs the installed package version.
- The stable Pallets changelog already shows a `3.1.2` entry, but PyPI and GitHub releases still show `3.1.1` as the latest published version on March 12, 2026. Do not assume `3.1.2` fixes are installed unless your environment proves it.

## Current Official Sources

- Docs root: `https://flask-sqlalchemy.palletsprojects.com/en/stable/`
- Quickstart: `https://flask-sqlalchemy.palletsprojects.com/en/stable/quickstart/`
- Config: `https://flask-sqlalchemy.palletsprojects.com/en/stable/config/`
- Models: `https://flask-sqlalchemy.palletsprojects.com/en/stable/models/`
- Queries: `https://flask-sqlalchemy.palletsprojects.com/en/stable/queries/`
- Binds: `https://flask-sqlalchemy.palletsprojects.com/en/stable/binds/`
- Contexts: `https://flask-sqlalchemy.palletsprojects.com/en/stable/contexts/`
- Changelog: `https://flask-sqlalchemy.palletsprojects.com/en/stable/changes/`
- PyPI: `https://pypi.org/project/flask-sqlalchemy/`

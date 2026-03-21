---
name: migrate
description: "Flask-Migrate package guide for Python: Alembic-powered schema migrations for Flask-SQLAlchemy applications"
metadata:
  languages: "python"
  versions: "4.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,sqlalchemy,alembic,migrations,database"
---

# Flask-Migrate Python Package Guide

## Golden Rule

Use `Flask-Migrate` as the Flask CLI integration for Alembic. `flask db migrate` only generates a revision file; review that file, then run `flask db upgrade` to apply the change.

## What It Does

`Flask-Migrate` connects Alembic to Flask's CLI so your app can:

- create a migrations repository
- autogenerate migration scripts from SQLAlchemy metadata changes
- upgrade, downgrade, stamp, inspect, and merge revisions
- support app factories and Flask-SQLAlchemy multiple binds

It is normally used with `Flask-SQLAlchemy` and an initialized `SQLAlchemy` object.

## Install

Pin the package version when you need behavior that matches this doc:

```bash
pip install Flask-Migrate==4.1.0
```

Typical stack:

```bash
pip install Flask Flask-SQLAlchemy Alembic Flask-Migrate==4.1.0
```

PyPI metadata for `4.1.0` declares `Requires: Python >=3.6`, but the practical floor in a real app can be higher because Flask, Flask-SQLAlchemy, SQLAlchemy, and your database driver may require newer Python versions.

## Minimal Setup

Application factories are the safest default for new Flask apps:

```python
from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate

db = SQLAlchemy()
migrate = Migrate()

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        SQLALCHEMY_DATABASE_URI="postgresql+psycopg://user:pass@localhost/app",
        SQLALCHEMY_TRACK_MODIFICATIONS=False,
    )

    db.init_app(app)
    migrate.init_app(app, db)

    return app

class User(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    email = db.Column(db.String(255), unique=True, nullable=False)
```

If you do not use an app factory, constructing `Migrate(app, db)` directly is also supported:

```python
migrate = Migrate(app, db)
```

Make sure the Flask CLI imports all models before migration commands run. If metadata is incomplete, autogenerate will miss tables or columns.

## First Migration Workflow

For a module that exposes `app`:

```bash
flask --app app db init
flask --app app db migrate -m "create user table"
flask --app app db upgrade
```

For an app factory:

```bash
flask --app 'myapp:create_app()' db upgrade
```

Equivalent environment-variable style from the official docs:

```bash
export FLASK_APP=app.py
flask db migrate -m "add last_login to user"
flask db upgrade
```

Commit the generated `migrations/` directory. Alembic revision history belongs in source control.

## Core Commands

| Command | Use |
| --- | --- |
| `flask db init` | Create the Alembic migrations repository. |
| `flask db migrate -m "msg"` | Autogenerate a new revision from detected model changes. |
| `flask db upgrade [revision]` | Apply migrations up to a target revision, usually `head`. |
| `flask db downgrade [-1\|revision]` | Roll back one or more revisions. |
| `flask db check` | Exit non-zero when autogenerate would produce a new migration; useful in CI. |
| `flask db current` | Show the current database revision. |
| `flask db history` | Show revision history. |
| `flask db heads` | Show current head revisions. |
| `flask db stamp <revision>` | Mark a revision without running migrations. |
| `flask db merge` | Merge divergent heads. |
| `flask db revision` | Create an empty or manually edited revision. |

`flask db migrate` uses Alembic autogenerate, so treat the output as a draft and inspect it before applying it.

## Configuration

### `Migrate` options

Pass Alembic autogenerate options to `Migrate()` or `init_app()`:

```python
migrate = Migrate(
    app,
    db,
    directory="migrations",
    compare_type=True,
    render_as_batch=True,
)
```

Important `4.x` defaults from the official docs:

- `compare_type=True` is enabled by default, so type changes are detected automatically.
- `render_as_batch=True` is enabled by default, which improves SQLite schema migrations by using batch mode.

### Custom CLI group or migration directory

```python
Migrate(app, db, directory="db-migrations", command="schema")
```

Then run:

```bash
flask schema upgrade
```

You can also override the directory with the `FLASK_DB_DIRECTORY` environment variable.

### Alembic config callbacks

Use `@migrate.configure` to modify the Alembic config object before commands run:

```python
from flask_migrate import Migrate

migrate = Migrate()

@migrate.configure
def configure_alembic(config):
    config.set_main_option("compare_type", "true")
    return config
```

This is the hook for environment-driven config changes. If multiple callbacks are registered, their execution order is not guaranteed.

## Multiple Databases And Custom Arguments

For Flask-SQLAlchemy binds, initialize the repository in multi-db mode from the start:

```bash
flask --app app db init --multidb
```

Custom Alembic arguments are passed with `-x` and can be consumed inside migration scripts:

```bash
flask --app app db upgrade -x tenant=acme
```

The official docs note that all `flask db` commands accept `-x` arguments in the current `4.x` line. Read them from Alembic's context helpers inside your revision scripts instead of assuming Flask request globals exist.

## Config And Secrets

`Flask-Migrate` has no package-specific authentication layer. Database access comes from your Flask and SQLAlchemy configuration, typically by mapping environment variables into `SQLALCHEMY_DATABASE_URI` or related engine settings:

```python
import os

app.config["SQLALCHEMY_DATABASE_URI"] = os.environ["DATABASE_URL"]
```

Treat migration commands as privileged operations because they run with full schema-changing database credentials.

## Common Pitfalls

- `flask db migrate` creates a revision file but does not change the database until `flask db upgrade` runs.
- Alembic autogenerate detects many schema changes, but it does not reliably infer table renames, column renames, or anonymously named constraints.
- Review generated scripts before applying them, especially for data migrations, server defaults, and engine-specific DDL.
- Import all models before running CLI commands or Alembic will compare incomplete metadata.
- Use `flask db merge` for multiple heads instead of manually editing revision identifiers.
- Batch mode makes SQLite development workflows smoother, but production behavior on PostgreSQL or MySQL can still differ.

## Version-Sensitive Notes For `4.1.0`

- PyPI currently lists `4.1.0` as the latest release for `flask-migrate`.
- The maintainer changelog for `4.1.0` is short: it updates support for current Python and dependency versions, without documenting a new CLI surface.
- `4.0.6` added `-x` support across all `db` commands and `--purge` support for `flask db stamp`.
- `4.0.2` added `flask db check`, which is useful as a CI guard for missing migrations.
- `4.0.0` changed defaults by enabling `compare_type=True` and `render_as_batch=True`.
- The official Read the Docs site uses a rolling `/latest/` URL, so examples there can drift ahead of older pinned releases. For version-specific behavior, check the changelog alongside the docs.

## Official Sources

- Docs index: https://flask-migrate.readthedocs.io/en/latest/
- Commands reference: https://flask-migrate.readthedocs.io/en/latest/#command-reference
- PyPI package page: https://pypi.org/project/flask-migrate/
- Maintainer changelog: https://raw.githubusercontent.com/miguelgrinberg/Flask-Migrate/main/CHANGES.md
- Repository: https://github.com/miguelgrinberg/Flask-Migrate

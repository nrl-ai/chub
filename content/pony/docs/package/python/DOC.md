---
name: package
description: "Pony ORM package guide for Python projects using declarative entities, db_session transactions, and generator-based queries"
metadata:
  languages: "python"
  versions: "0.7.19"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pony,ponyorm,orm,sql,sqlite,postgresql,mysql"
---

# Pony ORM Python Package Guide

## Golden Rule

Use `pony` for ORM work only inside a `db_session`, bind the `Database()` explicitly before touching entities, and materialize any data you need before leaving the session. Pony commits automatically on successful exit, rolls back on exceptions, clears its identity map, and returns the connection to the pool when the session ends.

## Install

Pin the package version your project expects:

```bash
python -m pip install "pony==0.7.19"
```

Common alternatives:

```bash
uv add "pony==0.7.19"
poetry add "pony==0.7.19"
```

Database drivers are separate from Pony:

- SQLite: no extra driver needed
- PostgreSQL: install a PostgreSQL DB-API driver such as `psycopg2-binary`
- MySQL: install a supported MySQL DB-API driver such as `PyMySQL`
- Oracle or CockroachDB: verify the driver and version notes from the upstream docs before wiring production code

## Initialize A Database

Create one `Database()` object, define entities off `db.Entity`, then call `bind()` and `generate_mapping()`:

```python
from datetime import date

from pony.orm import Database, PrimaryKey, Required, Set, db_session, select

db = Database()

class Customer(db.Entity):
    id = PrimaryKey(int, auto=True)
    email = Required(str, unique=True)
    orders = Set("Order")

class Order(db.Entity):
    id = PrimaryKey(int, auto=True)
    total_cents = Required(int)
    created_on = Required(date)
    customer = Required(Customer)

db.bind(provider="sqlite", filename="app.sqlite", create_db=True)
db.generate_mapping(create_tables=True)
```

Important setup rules:

- Define entities before `generate_mapping()`
- Use `create_tables=True` for new local schemas; do not assume that is the right production migration strategy
- If you omit an explicit primary key, Pony adds `id = PrimaryKey(int, auto=True)` automatically
- Relationship fields must be declared on both sides

## reviewnd Transaction Model

All ORM work should happen inside `@db_session` or `with db_session:`:

```python
from pony.orm import db_session, flush

@db_session
def create_customer(email: str) -> int:
    customer = Customer(email=email)
    flush()  # assign auto PK before the session exits
    return customer.id

@db_session
def rename_customer(customer_id: int, new_email: str) -> None:
    Customer[customer_id].email = new_email
```

Context-manager form:

```python
from pony.orm import db_session

with db_session:
    customer = Customer(email="alice@example.com")
```

What `db_session` does for you:

- rolls back if an exception escapes the block
- commits automatically if data changed and no exception occurred
- clears the session cache on exit
- returns the database connection to the pool

Practical rule: entity instances are only safe to use inside the active session. If you need to hand data to templates, APIs, or background jobs, convert entities to plain dicts, lists, or scalar values before the session ends.

## Core Usage

### Create rows

```python
from datetime import date
from pony.orm import db_session

@db_session
def seed() -> None:
    alice = Customer(email="alice@example.com")
    Order(total_cents=1999, created_on=date.today(), customer=alice)
```

Pony tracks changes automatically. You do not call `.save()` on entities.

### Look up one object

```python
from pony.orm import db_session

@db_session
def get_customer(customer_id: int) -> Customer | None:
    return Customer.get(id=customer_id)
```

Use cases:

- `Customer[123]`: fetch by primary key and raise if missing
- `Customer.get(email="alice@example.com")`: return one object or `None`

If you return data to code outside the session, prefer returning plain values instead of the entity itself.

### Query multiple rows

```python
from pony.orm import db_session, desc, select

@db_session
def top_orders(limit: int = 10) -> list[tuple[str, int]]:
    query = select(order for order in Order).order_by(lambda o: desc(o.total_cents))[:limit]
    return [(order.customer.email, order.total_cents) for order in query]
```

Useful query patterns from Pony’s docs:

- generator-based `select(...)`
- slicing for `LIMIT` / `OFFSET`
- `order_by(...)`
- `exists(...)`, `count(...)`, and aggregate expressions when you need booleans or totals

### Update and delete

```python
from pony.orm import db_session

@db_session
def update_order(order_id: int, total_cents: int) -> None:
    order = Order[order_id]
    order.total_cents = total_cents

@db_session
def delete_order(order_id: int) -> None:
    Order[order_id].delete()
```

Changes persist automatically when the session exits successfully.

## Provider Configuration

Pony does not have API keys or package-level auth. Configuration is database connection config passed to `db.bind(...)`.

SQLite:

```python
db.bind(provider="sqlite", filename="app.sqlite", create_db=True)
```

PostgreSQL:

```python
import os

db.bind(
    provider="postgres",
    user=os.environ["PGUSER"],
    password=os.environ["PGPASSWORD"],
    host=os.getenv("PGHOST", "127.0.0.1"),
    database=os.environ["PGDATABASE"],
)
```

MySQL:

```python
import os

db.bind(
    provider="mysql",
    host=os.getenv("MYSQL_HOST", "127.0.0.1"),
    user=os.environ["MYSQL_USER"],
    passwd=os.environ["MYSQL_PASSWORD"],
    db=os.environ["MYSQL_DATABASE"],
)
```

Notes:

- The getting-started page lists supported databases with the label `postgresql`, but the canonical bind examples use `provider="postgres"`. Follow the bind examples.
- Use environment variables or your secret manager for credentials instead of hard-coding them.
- `db.on_connect(...)` exists if you need per-connection session setup such as SQLite PRAGMAs or PostgreSQL session options.

## Common Pitfalls

- Forgetting `db_session`: even read-only work should run inside a session so connections are cleaned up correctly.
- Returning live entities outside the session: later attribute access can fail because the identity map and connection are gone.
- Assuming Pony needs explicit `.save()`: it does not. Changes are flushed and committed by the session machinery.
- Using `create_tables=True` blindly in production: Pony can create missing tables, but schema migration planning is still your job.
- Copying old docs snippets literally: the official docs still contain Python 2 era examples such as `print` statements without parentheses and broad `from pony.orm import *` imports.
- Missing both sides of a relationship: Pony relationships are defined bidirectionally.

## Version-Sensitive Notes For 0.7.19

- As of March 12, 2026, PyPI still lists `0.7.19` as the current package version, so the version used here is current.
- The maintainer changelog indicates `0.7.19` was a small packaging release that added a missed Python 3.12 classifier.
- The same changelog notes `0.7.18` added Python 3.12 compatibility and fixed a SQLite 3.45 JSON-related SQL issue.
- The changelog for `0.7.17` says Python 3.7 support was dropped. Treat current support as Python 3.8+ and verify against your runtime if you are pinned to older Python.
- The docs site is still valuable, but parts of it are older than the latest releases. Trust the current package metadata and changelog for Python-version support when they disagree with older tutorial prose.

## Official Sources

- Docs root: `https://docs.ponyorm.org/`
- Getting started: `https://docs.ponyorm.org/firststeps.html`
- Database binding and connection options: `https://docs.ponyorm.org/database.html`
- Entity declarations: `https://docs.ponyorm.org/entities.html`
- CRUD and flush/commit behavior: `https://docs.ponyorm.org/crud.html`
- Query syntax: `https://docs.ponyorm.org/queries.html`
- Relationships: `https://docs.ponyorm.org/working_with_relationships.html`
- PyPI package page: `https://pypi.org/project/pony/`
- Maintainer repository: `https://github.com/ponyorm/pony`
- Maintainer changelog: `https://github.com/ponyorm/pony/blob/master/CHANGELOG.md`

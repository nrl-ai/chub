---
name: package
description: "Neo4j Python driver for connecting to Neo4j, running Cypher queries, and managing transactions"
metadata:
  languages: "python"
  versions: "6.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "neo4j,graph,database,cypher,bolt,asyncio"
---

# Neo4j Python Package Guide

## Golden Rule

Use the official `neo4j` package for Python applications. Do not install `neo4j-driver`; the API docs mark `neo4j-driver` as the old package name and say it stopped receiving updates starting with 6.0.0.

## Install

Pin the package version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "neo4j==6.1.0"
```

Common alternatives:

```bash
uv add "neo4j==6.1.0"
poetry add "neo4j==6.1.0"
```

Optional high-performance drop-in replacement:

```bash
python -m pip install neo4j-rust-ext
```

The Rust extension uses the same API as `neo4j` and the Neo4j manual says it can be significantly faster.

## Initialize The Driver

The core entry point is `GraphDatabase.driver(...)`. Create one long-lived driver per process, verify connectivity once, and create short-lived sessions as needed.

```python
import os
from neo4j import GraphDatabase

URI = os.environ["NEO4J_URI"]
AUTH = (os.environ["NEO4J_USERNAME"], os.environ["NEO4J_PASSWORD"])
DATABASE = os.getenv("NEO4J_DATABASE", "neo4j")

driver = GraphDatabase.driver(URI, auth=AUTH)
driver.verify_connectivity()

try:
    records, summary, keys = driver.execute_query(
        """
        MERGE (p:Person {name: $name})
        SET p.updated_at = datetime()
        RETURN p.name AS name, p.updated_at AS updated_at
        """,
        name="Alice",
        database_=DATABASE,
    )
    print(records[0]["name"])
finally:
    driver.close()
```

Recommended environment variables:

```bash
export NEO4J_URI="neo4j+s://<instance-id>.databases.neo4j.io"
export NEO4J_USERNAME="neo4j"
export NEO4J_PASSWORD="<password>"
export NEO4J_DATABASE="neo4j"
```

For local development, the URI is often `neo4j://localhost:7687` or `bolt://localhost:7687`.

## Connection And Auth

### URI schemes

- `neo4j://`: routing-aware connection, good default for local and clustered setups
- `neo4j+s://`: TLS with CA-signed certificates; this is the normal Aura scheme
- `neo4j+ssc://`: TLS with self-signed certificates
- `bolt://`: direct connection to a specific server, without routing

Use the scheme required by the server. The driver does not infer it for you.

### Basic auth

Basic username/password auth is the common case:

```python
from neo4j import GraphDatabase

driver = GraphDatabase.driver(
    "neo4j+s://<instance-id>.databases.neo4j.io",
    auth=("neo4j", "password"),
)
```

### Bearer or rotating auth

If your Neo4j deployment uses SSO or expiring bearer tokens, use an auth manager instead of rebuilding the driver for each token refresh:

```python
import neo4j
from neo4j.auth_management import AuthManagers, ExpiringAuth

def auth_provider():
    token = get_sso_token()
    return ExpiringAuth(
        auth=neo4j.bearer_auth(token)
    ).expires_in(50)

driver = neo4j.GraphDatabase.driver(
    URI,
    auth=AuthManagers.bearer(auth_provider),
)
```

The auth manager must always return credentials for the same identity. It is not a user-switching mechanism.

## Core Query Workflow

### Use `execute_query()` for one query

`driver.execute_query()` is the simplest API for a single query. It eagerly returns `(records, summary, keys)`.

```python
records, summary, keys = driver.execute_query(
    """
    MATCH (p:Person)
    WHERE p.name STARTS WITH $prefix
    RETURN p.name AS name
    ORDER BY name
    LIMIT $limit
    """,
    prefix="Al",
    limit=10,
    database_="neo4j",
    routing_="r",
)

names = [record["name"] for record in records]
print(names)
print(summary.result_available_after)
```

Rules that matter:

- Always pass `database_="..."` explicitly. Neo4j documents this as both the recommended pattern and a performance optimization.
- Use Cypher parameters like `$prefix` and `$limit`; do not build query strings with string interpolation.
- Use `routing_="r"` for read-only queries when you want the driver to route them as reads.

### Use sessions and managed transactions for multi-step writes

Use a session plus `execute_write()` or `execute_read()` when a logical operation spans multiple queries.

```python
from neo4j import GraphDatabase

def create_friendship(tx, person_name: str, friend_name: str) -> list[str]:
    tx.run(
        "MERGE (:Person {name: $name})",
        name=person_name,
    )
    result = tx.run(
        """
        MATCH (a:Person {name: $person_name})
        MERGE (b:Person {name: $friend_name})
        MERGE (a)-[:KNOWS]->(b)
        RETURN a.name AS person, b.name AS friend
        """,
        person_name=person_name,
        friend_name=friend_name,
    )
    record = result.single(strict=True)
    return [record["person"], record["friend"]]

with GraphDatabase.driver(URI, auth=AUTH) as driver:
    with driver.session(database="neo4j") as session:
        person, friend = session.execute_write(
            create_friendship,
            "Alice",
            "Bob",
        )
        print(person, friend)
```

Important transaction behavior:

- Sessions are lightweight and not thread-safe. Create a separate session per thread or unit of work.
- Transaction callbacks should return concrete values, not the raw `Result` object.
- Managed transactions retry retryable failures for you; if you catch `Neo4jError`, check `is_retryable()` before retrying explicit transaction logic.

## Async Usage

Use `AsyncGraphDatabase` if your application already uses `asyncio` and needs concurrent database work.

```python
import asyncio
from neo4j import AsyncGraphDatabase

URI = "neo4j+s://<instance-id>.databases.neo4j.io"
AUTH = ("neo4j", "password")

async def main() -> None:
    async with AsyncGraphDatabase.driver(URI, auth=AUTH) as driver:
        await driver.verify_connectivity()
        records, summary, keys = await driver.execute_query(
            "MATCH (p:Person) RETURN p.name AS name LIMIT 5",
            database_="neo4j",
            routing_="r",
        )
        print([record["name"] for record in records])

asyncio.run(main())
```

Only use the async driver if the rest of the stack is async-aware. Do not share async sessions across tasks.

## Common Configuration

Typical application configuration:

```python
import os

NEO4J_URI = os.environ["NEO4J_URI"]
NEO4J_USERNAME = os.environ["NEO4J_USERNAME"]
NEO4J_PASSWORD = os.environ["NEO4J_PASSWORD"]
NEO4J_DATABASE = os.getenv("NEO4J_DATABASE", "neo4j")
```

Operational guidance:

- Keep one driver object per app process and inject it where needed.
- Keep secrets out of source control.
- Explicitly close drivers and sessions, or use `with` / `async with`.
- If you need to run queries as another user for a single operation, use the `auth_=` parameter on `execute_query()` instead of creating a separate driver.

## Common Pitfalls

- Installing `neo4j-driver` instead of `neo4j`. `neo4j-driver` is deprecated and frozen.
- Omitting `database_` or `database`. It works, but costs an extra round-trip and is not the recommended pattern.
- Returning a raw `Result` from a transaction callback. Consume it inside the callback and return plain Python values.
- Forgetting to close the driver. In 6.x, objects are not implicitly closed in destructors.
- Using removed APIs such as `session.read_transaction()` and `session.write_transaction()`. In 6.x, use `execute_read()` and `execute_write()`.
- Relying on legacy numeric graph ids. Prefer `element_id` over `id`.
- Mixing query parameters into strings instead of using Cypher placeholders.

## Version-Sensitive Notes For 6.1.0

- PyPI currently lists `neo4j 6.1.0`, released on January 12, 2026.
- The current 6.x manual says this driver line is compatible with Neo4j 4.4, 5.x, and the 2025/2026 server series, so driver upgrades can usually happen before server upgrades.
- Version 6 removed `read_transaction()` and `write_transaction()`, removed implicit cleanup through `__del__()`, and stopped updating the `neo4j-driver` package alias.
- The 6.x upgrade guide also calls out newer `Vector` support and GQL status objects in errors. If you are updating older code, review the upstream upgrade page before copying 5.x examples.

## Official Sources

- Python manual: https://neo4j.com/docs/python-manual/current/
- Connect to the database: https://neo4j.com/docs/python-manual/current/connect/
- Query the database: https://neo4j.com/docs/python-manual/current/query-simple/
- Run your own transactions: https://neo4j.com/docs/python-manual/current/transactions/
- Run concurrent transactions: https://neo4j.com/docs/python-manual/current/concurrency/
- Performance recommendations: https://neo4j.com/docs/python-manual/current/performance/
- Advanced connection information: https://neo4j.com/docs/python-manual/current/connect-advanced/
- API docs: https://neo4j.com/docs/api/python-driver/current/
- PyPI: https://pypi.org/project/neo4j/

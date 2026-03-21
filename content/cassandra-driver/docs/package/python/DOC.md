---
name: package
description: "DataStax Python Driver for Apache Cassandra and Astra DB applications"
metadata:
  languages: "python"
  versions: "3.29.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "cassandra,datastax,astra,cql,database,python"
---

# cassandra-driver Python Package Guide

## Golden Rule

Use the official `cassandra-driver` package for Cassandra-compatible clusters, and treat the DataStax 3.29 docs as the primary guide even though the docs site lags the current PyPI patch release. As of March 12, 2026, PyPI publishes `3.29.3`, while the maintained docs root is still the `3.29` documentation set.

## Install

Pin the package version your project expects:

```bash
python -m pip install "cassandra-driver==3.29.3"
```

Common alternatives:

```bash
uv add "cassandra-driver==3.29.3"
poetry add "cassandra-driver==3.29.3"
```

If you build from source or need better throughput, read the installation and performance docs before assuming the pure-Python fallback is good enough. The driver can use optional C extensions and a `libev` reactor for materially better performance.

## Initialize A Local Or Self-Managed Cluster Connection

The usual entry point is `Cluster`, then `connect()` to create a `Session`:

```python
from cassandra.cluster import Cluster

cluster = Cluster(["127.0.0.1"], port=9042)
session = cluster.connect()

rows = session.execute("SELECT release_version FROM system.local")
for row in rows:
    print(row.release_version)

cluster.shutdown()
```

Connect straight to a keyspace when that is the steady-state target:

```python
from cassandra.cluster import Cluster

cluster = Cluster(["db1.internal", "db2.internal"])
session = cluster.connect("app_keyspace")
```

## Username/Password Authentication

For clusters using password auth, pass `PlainTextAuthProvider` into `Cluster`:

```python
from cassandra.auth import PlainTextAuthProvider
from cassandra.cluster import Cluster

auth_provider = PlainTextAuthProvider(
    username="cassandra",
    password="cassandra",
)

cluster = Cluster(
    ["127.0.0.1"],
    auth_provider=auth_provider,
)
session = cluster.connect()
```

Keep credentials in environment variables or your secret manager, not in source files.

## TLS / SSL Configuration

Use a Python `ssl.SSLContext` and pass it as `ssl_context` when the cluster requires TLS:

```python
import os
import ssl

from cassandra.auth import PlainTextAuthProvider
from cassandra.cluster import Cluster

ssl_context = ssl.create_default_context(cafile=os.environ["CASSANDRA_CA_CERT"])
ssl_context.check_hostname = True
ssl_context.verify_mode = ssl.CERT_REQUIRED

cluster = Cluster(
    ["db1.internal"],
    ssl_context=ssl_context,
    auth_provider=PlainTextAuthProvider(
        username=os.environ["CASSANDRA_USERNAME"],
        password=os.environ["CASSANDRA_PASSWORD"],
    ),
)
session = cluster.connect("app_keyspace")
```

## Connect To Astra DB

For Astra DB, use the secure connect bundle and an auth provider:

```python
import os

from cassandra.auth import PlainTextAuthProvider
from cassandra.cluster import Cluster

cloud = {
    "secure_connect_bundle": os.environ["ASTRA_SECURE_CONNECT_BUNDLE"],
}

auth_provider = PlainTextAuthProvider(
    os.environ["ASTRA_CLIENT_ID"],
    os.environ["ASTRA_CLIENT_SECRET"],
)

cluster = Cluster(cloud=cloud, auth_provider=auth_provider)
session = cluster.connect()
```

Typical environment variables:

```bash
ASTRA_SECURE_CONNECT_BUNDLE=/absolute/path/secure-connect-database.zip
ASTRA_CLIENT_ID=...
ASTRA_CLIENT_SECRET=...
```

## Core Query Patterns

### Simple reads

Use `SimpleStatement` when you need per-query settings such as consistency level or fetch size:

```python
from cassandra import ConsistencyLevel
from cassandra.cluster import Cluster
from cassandra.query import SimpleStatement

cluster = Cluster(["127.0.0.1"])
session = cluster.connect("app_keyspace")

statement = SimpleStatement(
    "SELECT id, email FROM users",
    consistency_level=ConsistencyLevel.LOCAL_QUORUM,
    fetch_size=500,
)

for row in session.execute(statement):
    print(row.id, row.email)
```

### Prepared statements

Prefer prepared statements for repeated queries with parameters:

```python
from cassandra.cluster import Cluster

cluster = Cluster(["127.0.0.1"])
session = cluster.connect("app_keyspace")

prepared = session.prepare(
    "INSERT INTO users (id, email, created_at) VALUES (?, ?, ?)"
)

session.execute(prepared, (user_id, email, created_at))
```

Use `?` placeholders for prepared statements. Non-prepared statements passed to `session.execute(query, parameters)` use `%s` or `%(name)s` placeholders instead.

### Async execution

Use `execute_async()` when you need request concurrency without blocking the calling thread:

```python
from cassandra.cluster import Cluster

cluster = Cluster(["127.0.0.1"])
session = cluster.connect("app_keyspace")

future = session.execute_async(
    "SELECT id, email FROM users WHERE id = %s",
    (user_id,),
)

row = future.result().one()
print(row.email)
```

### Row factories

Switch the row factory when tuple-style rows are inconvenient:

```python
from cassandra.cluster import Cluster
from cassandra.query import dict_factory

cluster = Cluster(["127.0.0.1"])
session = cluster.connect("app_keyspace")
session.row_factory = dict_factory

row = session.execute("SELECT id, email FROM users").one()
print(row["email"])
```

## Execution Profiles And Tuning

Execution profiles are the main way to centralize query defaults such as consistency, timeouts, load balancing, and row factories:

```python
from cassandra import ConsistencyLevel
from cassandra.cluster import (
    Cluster,
    EXEC_PROFILE_DEFAULT,
    ExecutionProfile,
)
from cassandra.policies import TokenAwarePolicy, DCAwareRoundRobinPolicy
from cassandra.query import tuple_factory

profile = ExecutionProfile(
    load_balancing_policy=TokenAwarePolicy(DCAwareRoundRobinPolicy(local_dc="us-east-1")),
    consistency_level=ConsistencyLevel.LOCAL_QUORUM,
    request_timeout=10,
    row_factory=tuple_factory,
)

cluster = Cluster(
    ["db1.internal", "db2.internal"],
    execution_profiles={EXEC_PROFILE_DEFAULT: profile},
)
session = cluster.connect("app_keyspace")
```

Use profiles instead of scattering per-call overrides once the application has a stable read/write policy.

## Object Mapper

The package also ships the `cqlengine` object mapper under `cassandra.cqlengine`. Use it only if the project is already committed to that abstraction; otherwise start with raw session queries so schema behavior stays obvious.

## Configuration Notes

- `Cluster([...])` contact points only need to be enough for initial discovery; they do not need to list every node.
- `port=9042` is the usual native transport port for self-managed clusters.
- `connect_timeout`, `control_connection_timeout`, heartbeat settings, and retry/load-balancing policies are cluster-level tuning knobs exposed on `Cluster`.
- Shut down both `Session` and `Cluster` in long-running apps, scripts, and tests to avoid leaked connections.

## Common Pitfalls

- Do not mix placeholder styles. Prepared statements use `?`, while non-prepared `session.execute(query, parameters)` calls use `%s` or `%(name)s`.
- Do not open a new `Cluster` or `Session` per request. Reuse them for the life of the process.
- Do not assume the docs site's version selector tracks the newest PyPI patch. For `3.29.3`, use the `3.29` docs plus the changelog for patch-specific fixes.
- Pure-Python mode can become the bottleneck under load. If latency or throughput matters, verify whether the C extensions and `libev` reactor are available in your environment.
- Always set the local datacenter explicitly in multi-DC deployments when using policies such as `DCAwareRoundRobinPolicy`.
- For Astra DB, use the secure connect bundle flow. Do not copy self-managed cluster contact-point examples into Astra configuration.
- `execute_async()` gives driver-level concurrency, but it is not `asyncio`. If your app is asyncio-native, treat the driver as blocking I/O unless you deliberately isolate it behind threads or worker processes.

## Version-Sensitive Notes For 3.29.3

- The official docs root is still the `3.29` documentation set; there is no separate `3.29.3` docs tree.
- PyPI and the upstream changelog show `3.29.3` released on January 30, 2025.
- `3.29.3` adds Python 3.13 support and ships patch-level fixes on top of the `3.29` docs set.
- The docs site still reflects older Python support guidance in some pages, so prefer PyPI metadata and the changelog when the docs and package metadata disagree.

## Official Sources

- DataStax Python Driver docs root: https://docs.datastax.com/en/developer/python-driver/3.29/
- Getting started: https://docs.datastax.com/en/developer/python-driver/3.29/getting_started/
- Installation: https://docs.datastax.com/en/developer/python-driver/3.29/installation/
- Performance notes: https://docs.datastax.com/en/developer/python-driver/3.29/performance/
- Security: https://docs.datastax.com/en/developer/python-driver/3.29/security/
- Astra cloud connection: https://docs.datastax.com/en/developer/python-driver/3.29/cloud/
- Execution profiles: https://docs.datastax.com/en/developer/python-driver/3.29/execution_profiles/
- Object mapper: https://docs.datastax.com/en/developer/python-driver/3.29/object_mapper/
- PyPI package page: https://pypi.org/project/cassandra-driver/
- Upstream changelog: https://github.com/apache/cassandra-python-driver/blob/master/CHANGELOG.rst

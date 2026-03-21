---
name: package
description: "PyMySQL package guide for Python with connection setup, cursors, transactions, TLS, and 1.1.2 notes"
metadata:
  languages: "python"
  versions: "1.1.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pymysql,mysql,mariadb,sql,db-api,python"
---

# PyMySQL Python Package Guide

## What It Is

`pymysql` is a pure-Python MySQL and MariaDB client that implements the Python DB-API 2.0 interface. Use it when Python code needs direct MySQL or MariaDB access without compiling a native driver.

PyMySQL is synchronous. If you need async database access, use an async driver or an async ORM layer instead of expecting `pymysql` itself to become non-blocking.

## Install

Pin the version your project expects:

```bash
python -m pip install "PyMySQL==1.1.2"
```

Optional extras from the official package metadata:

```bash
python -m pip install "PyMySQL[rsa]==1.1.2"
python -m pip install "PyMySQL[ed25519]==1.1.2"
```

Use `rsa` when the server auth flow needs `sha256_password` or `caching_sha2_password`. Use `ed25519` for MariaDB servers configured with the `ed25519` authentication plugin.

## Golden Rules

- Set `charset="utf8mb4"` unless you have a specific server-side reason not to.
- Use parameter binding with `%s` or `%(name)s`; do not build SQL with f-strings or string concatenation.
- Remember that `autocommit` defaults to `False`. Call `commit()` after writes or enable autocommit explicitly.
- Prefer `database=` and `password=`. The older aliases `db=` and `passwd=` are deprecated.
- Set connection and I/O timeouts explicitly in production code.
- Keep `local_infile=False` unless you intentionally need `LOAD DATA LOCAL`.

## Initialize A Connection

Typical application setup uses environment variables and a `DictCursor`:

```python
import os
import pymysql
import pymysql.cursors

connection = pymysql.connect(
    host=os.environ["MYSQL_HOST"],
    port=int(os.getenv("MYSQL_PORT", "3306")),
    user=os.environ["MYSQL_USER"],
    password=os.environ["MYSQL_PASSWORD"],
    database=os.environ["MYSQL_DATABASE"],
    charset="utf8mb4",
    cursorclass=pymysql.cursors.DictCursor,
    autocommit=False,
    connect_timeout=10,
    read_timeout=30,
    write_timeout=30,
)
```

Important connection parameters from the official API reference:

- `host` and `port` for TCP connections
- `unix_socket` for local socket-based connections
- `database` for the default schema
- `cursorclass` to choose `Cursor`, `DictCursor`, `SSCursor`, or `SSDictCursor`
- `read_default_file` to load client options from a `my.cnf`
- `ssl`, `ssl_ca`, `ssl_cert`, `ssl_key`, `ssl_verify_cert`, `ssl_verify_identity` for TLS
- `read_timeout` and `write_timeout` for socket I/O
- `server_public_key` and `auth_plugin_map` for specialized auth flows

If you want to read credentials from a MySQL client config file:

```python
import pymysql

connection = pymysql.connect(
    read_default_file="~/.my.cnf",
    read_default_group="client",
    database="appdb",
    charset="utf8mb4",
)
```

## Core Usage

### Simple Query Pattern

```python
import pymysql
import pymysql.cursors

connection = pymysql.connect(
    host="localhost",
    user="app",
    password="secret",
    database="appdb",
    charset="utf8mb4",
    cursorclass=pymysql.cursors.DictCursor,
)

with connection:
    with connection.cursor() as cursor:
        cursor.execute(
            "SELECT id, email FROM users WHERE status=%s ORDER BY id DESC LIMIT %s",
            ("active", 20),
        )
        rows = cursor.fetchall()

    print(rows)
```

Parameter style reminders:

- Use `%s` for tuple or list parameters.
- Use `%(name)s` for mapping-style parameters.
- Do not quote placeholders yourself.

### Insert, Commit, And Roll Back

Because autocommit is off by default, treat writes as explicit transactions:

```python
import pymysql

connection = pymysql.connect(
    host="localhost",
    user="app",
    password="secret",
    database="appdb",
    charset="utf8mb4",
)

try:
    with connection.cursor() as cursor:
        cursor.execute(
            "INSERT INTO users (email, password_hash) VALUES (%s, %s)",
            ("alice@example.com", "hashed-password"),
        )
    connection.commit()
except Exception:
    connection.rollback()
    raise
finally:
    connection.close()
```

### Batch Inserts

Use `executemany()` for repeated `INSERT` or `REPLACE` statements:

```python
rows = [
    ("alice@example.com", "hash-1"),
    ("bob@example.com", "hash-2"),
]

with connection.cursor() as cursor:
    cursor.executemany(
        "INSERT INTO users (email, password_hash) VALUES (%s, %s)",
        rows,
    )

connection.commit()
```

`executemany()` improves multi-row insert performance, but it still builds a statement that is bounded by packet-size limits. For very large batches, chunk your input instead of sending everything in one call.

## Cursor Types

Choose the cursor class deliberately:

- `Cursor`: default tuple-based rows
- `DictCursor`: dict-like rows keyed by column name
- `SSCursor`: unbuffered streaming cursor for large result sets
- `SSDictCursor`: unbuffered streaming cursor with dict rows

Use an unbuffered cursor when result sets are large or the connection is remote and you want lower peak memory use:

```python
import pymysql.cursors

with connection.cursor(pymysql.cursors.SSDictCursor) as cursor:
    cursor.execute("SELECT id, payload FROM huge_table")
    for row in cursor:
        process(row)
```

Unbuffered cursor caveats from the upstream API docs:

- total row count is not known up front
- backwards scrolling is not supported
- you should fully consume the result before reusing the connection for another query

## Transactions And Connection State

Useful connection methods:

- `commit()` to persist changes
- `rollback()` to abandon the current transaction
- `begin()` to start a transaction explicitly
- `ping(reconnect=True)` to verify the server is alive and optionally reconnect

Do not treat `ping(reconnect=True)` as a general connection-pooling strategy. In web apps or workers, manage connection lifetime at the framework or application boundary.

## TLS And Authentication

Use TLS for remote database traffic. The connection API accepts either an `ssl.SSLContext` or TLS-related keyword arguments.

Example with an `SSLContext`:

```python
import os
import ssl
import pymysql

ssl_context = ssl.create_default_context(cafile=os.environ["MYSQL_SSL_CA"])
ssl_context.check_hostname = True
ssl_context.verify_mode = ssl.CERT_REQUIRED

connection = pymysql.connect(
    host=os.environ["MYSQL_HOST"],
    user=os.environ["MYSQL_USER"],
    password=os.environ["MYSQL_PASSWORD"],
    database=os.environ["MYSQL_DATABASE"],
    ssl=ssl_context,
)
```

Example with explicit TLS keyword arguments:

```python
connection = pymysql.connect(
    host="db.example.com",
    user="app",
    password="secret",
    database="appdb",
    ssl_ca="/etc/ssl/certs/mysql-ca.pem",
    ssl_cert="/etc/ssl/certs/client-cert.pem",
    ssl_key="/etc/ssl/private/client-key.pem",
    ssl_verify_cert=True,
    ssl_verify_identity=True,
)
```

Authentication notes:

- `PyMySQL[rsa]` is needed for some MySQL auth plugin flows such as `sha256_password` and `caching_sha2_password`.
- `server_public_key=` is available when the server requires an RSA public key for auth.
- `auth_plugin_map=` exists for specialized or custom auth plugin handling, but treat it as advanced usage.

## Error Handling

PyMySQL follows DB-API exception families under `pymysql.err`. Common ones:

- `pymysql.err.OperationalError` for network, auth, timeout, or server-state failures
- `pymysql.err.IntegrityError` for unique-key or foreign-key failures
- `pymysql.err.ProgrammingError` for malformed SQL or placeholder mistakes

Pattern:

```python
import pymysql

try:
    with connection.cursor() as cursor:
        cursor.execute("SELECT * FROM users WHERE id=%s", (user_id,))
except pymysql.err.IntegrityError:
    connection.rollback()
    raise
except pymysql.MySQLError:
    connection.rollback()
    raise
```

## Common Pitfalls

- Forgetting `commit()` after `INSERT`, `UPDATE`, `DELETE`, or DDL when `autocommit=False`
- Using string interpolation instead of DB-API placeholders
- Assuming `?` placeholders work; PyMySQL uses `%s` and `%(name)s`
- Omitting `charset="utf8mb4"` and then hitting Unicode or emoji issues
- Leaving `read_timeout` and `write_timeout` unset for long-lived production connections
- Using `SSCursor.fetchall()` for very large query results and defeating the point of streaming
- Enabling `local_infile=True` without understanding the security implications
- Relying on `db=` or `passwd=` in new code even though they are deprecated aliases

## Version-Sensitive Notes For 1.1.2

- PyPI shows `PyMySQL 1.1.2` released on August 24, 2025.
- The official docs site still uses old `0.7.2` page titles and older requirement text, so do not treat those titles as the package version.
- PyPI package metadata for `1.1.2` says `Requires-Python >=3.8`.
- The current upstream README text published on PyPI says CPython `3.9 and newer` and latest PyPy 3.x. Treat this as an upstream documentation mismatch and verify against your deployment target if you are pinned to Python 3.8.
- The GitHub `v1.1.2` release notes include Python 3.13 support work and MySQL 8.4 compatibility-related changes, so prefer `1.1.2` over older `1.1.0` or `1.1.1` examples when debugging modern runtime behavior.

## Official Sources Used For This Guide

- PyPI release page: `https://pypi.org/project/PyMySQL/1.1.2/`
- Docs root: `https://pymysql.readthedocs.io/en/latest/`
- Installation guide: `https://pymysql.readthedocs.io/en/latest/user/installation.html`
- Examples: `https://pymysql.readthedocs.io/en/latest/user/examples.html`
- Connection API reference: `https://pymysql.readthedocs.io/en/latest/modules/connections.html`
- Cursor API reference: `https://pymysql.readthedocs.io/en/latest/modules/cursors.html`
- GitHub releases: `https://github.com/PyMySQL/PyMySQL/releases`

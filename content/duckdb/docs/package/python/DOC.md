---
name: package
description: "DuckDB Python package guide for embedded SQL analytics, DataFrames, and file-backed workflows"
metadata:
  languages: "python"
  versions: "1.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "duckdb,python,sql,analytics,dataframe,parquet"
---

# duckdb Python Package Guide

## Golden Rule

Use an explicit `duckdb.connect(...)` connection for application code, persistent databases, and threaded work. Reserve top-level `duckdb.sql(...)` for short interactive queries.

## What It Is

`duckdb` embeds DuckDB inside your Python process. Common uses:

- run SQL directly over local Parquet, CSV, and JSON files
- query Pandas, Arrow, Polars, and NumPy data without moving data into a separate server
- keep an in-memory database for analysis or a `.duckdb` file for local persistence
- read from object storage such as S3 once extensions and credentials are configured

## Install

```bash
pip install duckdb==1.5.0
```

Check the installed version:

```bash
python -c "import duckdb; print(duckdb.__version__)"
```

Notes:

- The DuckDB Python client requires Python 3.9 or newer.

## Quick Start

### Ad hoc query

```python
import duckdb

result = duckdb.sql("SELECT 42 AS answer").fetchone()
print(result[0])
```

### Persistent database file

```python
import duckdb

con = duckdb.connect("app.duckdb")

con.execute("""
    CREATE TABLE IF NOT EXISTS events (
        id INTEGER,
        kind VARCHAR,
        created_at TIMESTAMP
    )
""")

con.execute(
    "INSERT INTO events VALUES (?, ?, current_timestamp)",
    [1, "signup"],
)

rows = con.execute(
    "SELECT id, kind FROM events WHERE kind = ?",
    ["signup"],
).fetchall()

print(rows)
con.close()
```

### Connection configuration

Pass engine settings at connect time:

```python
import duckdb

con = duckdb.connect(
    "app.duckdb",
    config={
        "threads": 4,
        "memory_limit": "4GB",
    },
)
```

Use `read_only=True` when another process may already have the same database file open:

```python
import duckdb

con = duckdb.connect("app.duckdb", read_only=True)
```

## Core Usage Patterns

### Query files directly

DuckDB can query files without a load step:

```python
import duckdb

con = duckdb.connect()

count = con.execute("""
    SELECT count(*)
    FROM read_parquet('data/orders/*.parquet')
    WHERE order_date >= DATE '2026-01-01'
""").fetchone()[0]

print(count)
```

The SQL shell syntax also works for some formats:

```python
import duckdb

rows = duckdb.sql("SELECT * FROM 'data/example.parquet' LIMIT 5").fetchall()
```

### Query a Pandas DataFrame without registering it

DuckDB's Python integration can discover DataFrames by variable name:

```python
import duckdb
import pandas as pd

orders = pd.DataFrame(
    [
        {"customer_id": 1, "total": 25},
        {"customer_id": 2, "total": 50},
        {"customer_id": 1, "total": 30},
    ]
)

summary = duckdb.sql("""
    SELECT customer_id, sum(total) AS total_spend
    FROM orders
    GROUP BY customer_id
    ORDER BY total_spend DESC
""").df()

print(summary)
```

If you need persistence or indexes, materialize the DataFrame into a table:

```python
import duckdb
import pandas as pd

orders_df = pd.DataFrame([{"customer_id": 1, "total": 25}])

con = duckdb.connect("analytics.duckdb")
con.execute("CREATE OR REPLACE TABLE orders AS SELECT * FROM orders_df")
```

### Fetch results in the format you need

```python
import duckdb

rel = duckdb.sql("SELECT * FROM range(5)")

rows = rel.fetchall()
df = rel.df()
arrow_table = rel.fetch_arrow_table()
numpy_cols = rel.fetchnumpy()
```

Use:

- `fetchall()` or `fetchone()` for DB-API style results
- `.df()` when the next step is Pandas
- `.fetch_arrow_table()` for Arrow pipelines
- `.fetchnumpy()` for NumPy-oriented code

### Parameter binding

Prefer parameters over string interpolation:

```python
import duckdb

con = duckdb.connect()

rows = con.execute(
    "SELECT * FROM range(?) WHERE range < ?",
    [100, 10],
).fetchall()
```

## Extensions, Remote Storage, and Credentials

Local files need no authentication. Remote object storage does.

For S3 and S3-compatible storage, DuckDB recommends secrets-based configuration:

```python
import duckdb

con = duckdb.connect()
con.execute("INSTALL httpfs")
con.execute("LOAD httpfs")

con.execute("""
    CREATE OR REPLACE SECRET s3_creds (
        TYPE s3,
        PROVIDER credential_chain,
        REGION 'us-east-1'
    )
""")

df = con.execute("""
    SELECT *
    FROM read_parquet('s3://my-bucket/path/data.parquet')
    LIMIT 10
""").df()
```

Practical notes:

- `credential_chain` uses the standard AWS credential resolution flow. Prefer it over hard-coding keys.
- Keep remote access setup inside the connection bootstrap path so queries fail early if extensions or credentials are missing.
- If a project depends on extensions, pin DuckDB and test extension loading in CI.

## Common Pitfalls

### `duckdb.sql(...)` uses a shared default connection

The module-level helpers operate on a global in-memory connection. That is convenient in notebooks, but it is the wrong default for request handlers, background jobs, and threaded code. Use a dedicated `duckdb.connect(...)` per unit of work or per thread.

### `cursor()` helps with concurrent work, but it is not a substitute for connection design

DuckDB's DB-API docs describe `cursor()` as a way to create a second connection to an existing database, which can help with parallel threads. The same section also notes that a single connection is thread-safe but locked while queries run, so you should still design concurrency explicitly and not rely on one shared connection for throughput.

### `:memory:` and named in-memory databases behave differently

- `:memory:` creates a private in-memory database per connection.
- A named in-memory database such as `:memory:analytics` shares state across connections that use the same name.

Use a file-backed database or explicitly named in-memory connection only when shared state is intentional.

### Direct DataFrame scans are convenient, but they are not persisted

Querying a Python variable by name is great for analysis, but the underlying DataFrame is still a Python object. Create a table with `CREATE TABLE ... AS SELECT * FROM df_name` when later queries must not depend on Python variable scope.

### `executemany()` is not the fast path for large bulk inserts

DuckDB's DB-API docs explicitly warn against using `executemany()` for large data loads. For bulk ingestion, prefer:

- `CREATE TABLE ... AS SELECT * FROM df_name`
- `INSERT INTO target SELECT * FROM df_name`
- `read_parquet(...)`, `read_csv(...)`, or `COPY`

### Python worker-thread imports can matter

The DuckDB known-issues page documents a NumPy import issue in worker threads. If threaded code fetches Pandas or NumPy results, import `numpy.core.multiarray` before starting threads.

### Notebook display oddities are documented upstream

The known-issues page notes that `DESCRIBE` and `SUMMARIZE` can appear empty in some Jupyter paths. The documented workaround is to wrap them in a subquery, for example `FROM (DESCRIBE tbl)`.

## Version-Sensitive Notes For 1.5.0

- This entry targets package version `1.5.0`. If a project is pinned to `1.4.x`, check the LTS docs before copying examples.
- DuckDB's 1.5 release notes call out a change in the `httpfs` extension backend from `httplib` to `curl`. Re-test proxy, TLS, and remote filesystem behavior when upgrading from earlier releases.
- The 1.5 release notes also warn that the single-arrow lambda syntax is deprecated and DuckDB 2.0 will disable it by default. Prefer `lambda x: ...` syntax in new SQL.

## Recommended Agent Workflow

1. Install the exact package version used by the project.
2. Decide early whether you need an ephemeral in-memory connection or a persistent `.duckdb` file.
3. Use parameterized SQL and explicit connections in non-interactive code.
4. For DataFrames and files, push work into DuckDB SQL instead of row-by-row Python loops.
5. For S3 or HTTP-backed reads, validate extension loading and credentials before writing query logic.

## Official Sources Used

- DuckDB Python client overview: `https://duckdb.org/docs/stable/clients/python/overview`
- DuckDB Python DB-API reference: `https://duckdb.org/docs/stable/clients/python/dbapi`
- DuckDB Python conversion and result methods: `https://duckdb.org/docs/stable/clients/python/conversion`
- DuckDB guide for SQL on Pandas: `https://duckdb.org/docs/stable/guides/python/import_pandas`
- DuckDB known Python issues: `https://duckdb.org/docs/stable/clients/python/known_issues`
- DuckDB S3 API / secrets guidance: `https://duckdb.org/docs/stable/core_extensions/httpfs/s3api`
- DuckDB 1.5.0 release notes: `https://duckdb.org/2026/03/09/announcing-duckdb-150`
- PyPI package page: `https://pypi.org/project/duckdb/`

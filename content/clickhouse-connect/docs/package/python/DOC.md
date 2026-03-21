---
name: package
description: "clickhouse-connect Python package guide for connecting to ClickHouse over HTTP/HTTPS and working with query, insert, pandas, Arrow, Polars, and async APIs"
metadata:
  languages: "python"
  versions: "0.14.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "clickhouse,python,database,analytics,http,pandas,pyarrow,polars,sqlalchemy"
---

# clickhouse-connect Python Package Guide

## Golden Rule

Use `clickhouse-connect` for Python projects that talk to ClickHouse over the HTTP/HTTPS interface. This is the ClickHouse-maintained client, and the upstream docs and repo both treat it as the primary Python driver.

Do not assume it speaks the native TCP protocol. `clickhouse-connect` is HTTP-based, so the usual defaults are port `8123` for HTTP and `8443` for HTTPS.

## Install

Pin the package version your project expects:

```bash
python -m pip install "clickhouse-connect==0.14.0"
```

Optional extras published on PyPI:

```bash
python -m pip install "clickhouse-connect[pandas]==0.14.0"
python -m pip install "clickhouse-connect[arrow]==0.14.0"
python -m pip install "clickhouse-connect[sqlalchemy]==0.14.0"
python -m pip install "clickhouse-connect[polars]==0.14.0"
```

## Create A Client

### Basic local or self-hosted HTTP client

```python
import clickhouse_connect

client = clickhouse_connect.get_client(
    host="localhost",
    port=8123,
    username="default",
    password="",
    database="default",
)

result = client.query("SELECT version()")
print(result.first_row)
```

### HTTPS client for ClickHouse Cloud or any TLS endpoint

```python
import os

import clickhouse_connect

client = clickhouse_connect.get_client(
    host="your-service.clickhouse.cloud",
    port=8443,
    secure=True,
    username="default",
    password=os.environ["CLICKHOUSE_PASSWORD"],
    database="default",
    verify=True,
    ca_cert="certifi",
    compress="lz4",
)
```

### JWT access token instead of username/password

Use `access_token` for ClickHouse Cloud JWT auth, and do not combine it with `username` or `password`.

```python
import os

import clickhouse_connect

client = clickhouse_connect.get_client(
    host="your-service.clickhouse.cloud",
    secure=True,
    access_token=os.environ["CLICKHOUSE_ACCESS_TOKEN"],
)
```

### DSN-based setup

The client also accepts a DSN and will derive host, port, credentials, and database from it:

```python
import clickhouse_connect

client = clickhouse_connect.get_client(
    dsn="https://default:password@example.clickhouse.cloud:8443/default"
)
```

## Core Query Workflow

### `query()` returns a `QueryResult`

Use `query()` for regular `SELECT` and similar result-returning SQL:

```python
result = client.query("SELECT name, engine FROM system.tables LIMIT 3")

print(result.result_rows)
print(result.first_item)
print(list(result.named_results()))
```

Useful `QueryResult` access patterns:

- `result.result_rows`: row-oriented Python data
- `result.result_set`: row-oriented by default, column-oriented if the query requested that shape
- `result.first_row` / `result.first_item`: fast access for single-row results
- `result.named_results()`: generator of dictionaries keyed by column name

### `command()` for DDL and one-off commands

Use `command()` when you do not want a full result matrix back:

```python
client.command("""
CREATE TABLE IF NOT EXISTS events
(
    id UInt32,
    name String
)
ENGINE = MergeTree
ORDER BY id
""")
```

### DataFrame and Arrow query methods

Use the higher-level query method that matches the output you actually need:

```python
df = client.query_df("SELECT number FROM numbers(5)")
arrow_table = client.query_arrow("SELECT number FROM numbers(5)")
```

For Arrow-backed DataFrame output:

```python
arrow_df = client.query_df_arrow(
    "SELECT number FROM numbers(5)",
    dataframe_library="pandas",
)
```

Use `dataframe_library="polars"` if you want a Polars DataFrame instead.

### Streaming large results

For large result sets, stream blocks instead of materializing everything at once:

```python
with client.query_df_stream("SELECT * FROM events") as stream:
    for block_df in stream:
        print(block_df.shape)
```

There are also stream variants for row blocks, numpy, Arrow, and Arrow-backed DataFrames.

## Insert Data

### Insert Python rows

```python
rows = [
    [1, "alpha"],
    [2, "beta"],
]

client.insert("events", rows, column_names=["id", "name"])
```

### Insert a pandas DataFrame

```python
import pandas as pd

df = pd.DataFrame(
    [
        {"id": 3, "name": "gamma"},
        {"id": 4, "name": "delta"},
    ]
)

client.insert_df("events", df)
```

### Reuse insert metadata with `create_insert_context()`

If you are inserting many batches into the same table, build the insert context once and reuse it:

```python
insert_context = client.create_insert_context(
    table="events",
    column_names=["id", "name"],
)

for batch in (
    [[5, "one"], [6, "two"]],
    [[7, "three"], [8, "four"]],
):
    insert_context.data = batch
    client.data_insert(insert_context)
```

This avoids re-describing the table on every batch when the target schema is stable.

### Arrow and Arrow-backed DataFrame inserts

Use Arrow-based insert methods when your data is already Arrow-native:

```python
client.insert_arrow("events", arrow_table)
client.insert_df_arrow("events", polars_df)
```

`insert_df_arrow()` is optimized for:

- Polars DataFrames
- pandas 2.x DataFrames whose columns already use PyArrow dtypes

## Async Usage

The async client is obtained with `await clickhouse_connect.get_async_client(...)`.

```python
import asyncio

import clickhouse_connect

async def main() -> None:
    client = await clickhouse_connect.get_async_client(
        host="localhost",
        port=8123,
        username="default",
        password="",
    )
    try:
        result = await client.query("SELECT 1")
        print(result.first_row)
    finally:
        await client.close()

asyncio.run(main())
```

You can also use the async context manager:

```python
async with await clickhouse_connect.get_async_client(host="localhost") as client:
    result = await client.query("SELECT 1")
```

## Configuration And Authentication Notes

Common client arguments you will actually use:

- `host`, `port`, `database`
- `username` and `password`
- `access_token` for JWT auth
- `secure=True` or `interface="https"` for TLS
- `verify` and `ca_cert` for certificate validation
- `client_cert` and `client_cert_key` for mutual TLS
- `compress` with `True`, `lz4`, `zstd`, `brotli`, or `gzip`
- `connect_timeout` and `send_receive_timeout`
- `client_name` to make queries easier to identify in `system.query_log`
- `form_encode_query_params=True` when very large query parameter payloads would exceed URL limits
- `proxy_path` when ClickHouse is exposed behind a path-based reverse proxy
- `transport_settings` on query and insert calls for per-request HTTP headers

The client defaults to:

- `localhost` when `host` is omitted
- `8123` for plain HTTP
- `8443` for HTTPS when `secure=True` or `interface="https"`

## SQLAlchemy

`clickhouse-connect` provides a lightweight SQLAlchemy dialect aimed at SQLAlchemy Core and Superset compatibility, not full ORM behavior.

Use the `clickhousedb://` scheme:

```text
clickhousedb://{username}:{password}@{host}:{port}
```

Supported SQLAlchemy features are intentionally limited upstream. Core-style `SELECT`, `JOIN`, `ARRAY JOIN`, `FINAL`, and lightweight `DELETE` are in scope; ORM-heavy patterns are not the target.

## Common Pitfalls

- `clickhouse-connect` uses the ClickHouse HTTP interface, not the native TCP client protocol. Do not point it at port `9000` and expect it to work.
- `query()` returns a `QueryResult`, not a pandas DataFrame. Use `query_df()`, `query_arrow()`, or `query_df_arrow()` when you need those shapes.
- `query_df()` and `insert_df()` require pandas to be installed. Arrow methods require `pyarrow`. Polars support depends on `polars`.
- `insert_df_arrow()` with pandas requires pandas `2.x` and PyArrow-backed dtypes for every column.
- `access_token` cannot be combined with `username` or `password`.
- `secure=True` changes the expected default port to `8443`. Plain HTTP defaults to `8123`.
- The async client wraps the sync client in an executor. Always `await client.close()` or use `async with`.
- `utc_tz_aware` is deprecated. Prefer `tz_mode="naive_utc"`, `"aware"`, or `"schema"`.
- SQLAlchemy support is Core-oriented. Do not promise full ORM parity with Postgres/MySQL dialects.

## Version-Sensitive Notes For 0.14.0

- PyPI and the maintainer GitHub repo both show `0.14.0` as the current release as of March 12, 2026.
- Python `3.9` is still accepted by package metadata, but the maintainer repo now emits a deprecation warning on import for Python `<3.10`, and the 0.11.0 release notes tell users to plan upgrades to `3.10+`.
- Since `0.13.0`, `tz_mode="schema"` is available and is the preferred modern replacement when you want timezone handling to match the ClickHouse column definition. Older examples may still use the deprecated `utc_tz_aware` argument.
- Since `0.13.0`, writing to `Variant` columns uses native type-aware dispatch instead of stringifying values first. If older code relied on implicit string coercion for `Variant`, re-test inserts.
- Since `0.9.0`, SQLAlchemy support targets `>=1.4.40` and includes SQLAlchemy `2.x` support. Old `1.3`-era examples are outdated.
- Since `0.9.0`, Arrow-backed DataFrame methods such as `query_df_arrow()` and `insert_df_arrow()` are available. Prefer them when your pipeline is already Arrow-native.
- Since `0.8.17`, `transport_settings` can attach per-query HTTP headers, and `proxy_path` can help when ClickHouse sits behind a path-based proxy.
- Since `0.8.15`, `AsyncClient.close()` is async. Older blog posts that call `client.close()` without `await` are wrong for current releases.

## Official Sources

- ClickHouse Python integration page: `https://clickhouse.com/integrations/python`
- PyPI package page: `https://pypi.org/project/clickhouse-connect/`
- Maintainer repository: `https://github.com/ClickHouse/clickhouse-connect`

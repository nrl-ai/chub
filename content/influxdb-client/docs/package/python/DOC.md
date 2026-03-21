---
name: package
description: "InfluxDB Python client for writing, querying, and managing InfluxDB 2.x from Python"
metadata:
  languages: "python"
  versions: "1.50.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "influxdb,time-series,flux,python,database,observability"
---

# influxdb-client Python Package Guide

## Golden Rule

Use `influxdb-client` for InfluxDB 2.x and Flux-based workflows. The package import is `influxdb_client`, not `influxdb`.

As of March 12, 2026, PyPI lists `influxdb-client 1.50.0`, which matches the version used here. Upstream describes this package as the Python client for InfluxDB 2.x and Flux. If you are targeting InfluxDB 3.x, use the official v3 client instead. If you are maintaining old `influxdb-python` code, do not assume the APIs are backwards compatible.

## Install

Base install:

```bash
python -m pip install "influxdb-client==1.50.0"
```

Recommended install with faster datetime parsing:

```bash
python -m pip install "influxdb-client[ciso]==1.50.0"
```

Async support:

```bash
python -m pip install "influxdb-client[async]==1.50.0"
```

If you plan to use pandas-heavy paths, install pandas separately and keep the `ciso` extra enabled when possible.

## Required Connection Settings

For most InfluxDB 2.x and Cloud setups you need:

- `url`: base URL such as `http://localhost:8086` or your Cloud URL
- `token`: API token
- `org`: default organization for writes and queries
- `bucket`: target bucket for writes

Environment variables supported by `from_env_properties()`:

```bash
export INFLUXDB_V2_URL="http://localhost:8086"
export INFLUXDB_V2_TOKEN="your-token"
export INFLUXDB_V2_ORG="your-org"
export INFLUXDB_V2_TIMEOUT="10000"
```

For local development, a `.env` file plus `python-dotenv` is enough:

```env
INFLUXDB_V2_URL=http://localhost:8086
INFLUXDB_V2_TOKEN=your-token
INFLUXDB_V2_ORG=your-org
INFLUXDB_V2_BUCKET=telemetry
```

## Initialize The Client

The safest default for scripts and request/response handlers is a context manager plus synchronous writes:

```python
import os

from dotenv import load_dotenv
from influxdb_client import InfluxDBClient
from influxdb_client.client.write_api import SYNCHRONOUS

load_dotenv()

url = os.environ["INFLUXDB_V2_URL"]
token = os.environ["INFLUXDB_V2_TOKEN"]
org = os.environ["INFLUXDB_V2_ORG"]
bucket = os.environ["INFLUXDB_V2_BUCKET"]

with InfluxDBClient(url=url, token=token, org=org) as client:
    write_api = client.write_api(write_options=SYNCHRONOUS)
    query_api = client.query_api()
```

If you already store connection settings in an INI, TOML, or JSON config file:

```python
from influxdb_client import InfluxDBClient

client = InfluxDBClient.from_config_file("influxdb.ini")
```

Or load directly from the `INFLUXDB_V2_*` environment variables:

```python
from influxdb_client import InfluxDBClient

client = InfluxDBClient.from_env_properties()
```

Minimal INI example:

```ini
[influx2]
url = http://localhost:8086
org = my-org
token = my-token
timeout = 10000
verify_ssl = true
```

## Core Usage

### Write points

Use `Point` objects or line protocol. `Point` is usually clearer and safer for agents.

```python
from influxdb_client import InfluxDBClient, Point
from influxdb_client.client.write_api import SYNCHRONOUS

with InfluxDBClient(url=url, token=token, org=org) as client:
    write_api = client.write_api(write_options=SYNCHRONOUS)

    point = (
        Point("cpu")
        .tag("host", "web-1")
        .field("usage_idle", 91.2)
    )

    write_api.write(bucket=bucket, record=point)
```

The client also accepts line protocol strings, dict-like records, dataclasses, named tuples, pandas DataFrames, and lists of those record types.

### Query with Flux

This client is built around Flux queries:

```python
from influxdb_client import InfluxDBClient

query = f'''
from(bucket: "{bucket}")
  |> range(start: -1h)
  |> filter(fn: (r) => r["_measurement"] == "cpu")
  |> filter(fn: (r) => r["_field"] == "usage_idle")
'''

with InfluxDBClient(url=url, token=token, org=org) as client:
    tables = client.query_api().query(query=query)

    for table in tables:
        for record in table.records:
            print(record.get_time(), record.values["_value"])
```

### Stream query results

For large result sets, prefer `query_stream()` so you do not materialize everything in memory:

```python
with InfluxDBClient(url=url, token=token, org=org) as client:
    records = client.query_api().query_stream(
        f'from(bucket: "{bucket}") |> range(start: -10m)'
    )

    for record in records:
        print(record["_time"], record["_value"])

    records.close()
```

### Query into pandas

If the workflow is analytical rather than service-oriented, use the DataFrame helpers:

```python
with InfluxDBClient(url=url, token=token, org=org) as client:
    df = client.query_api().query_data_frame(
        f'from(bucket: "{bucket}") |> range(start: -1d)'
    )
    print(df.head())
```

### Check connectivity and health

Use a quick readiness check before failing deeper in your code:

```python
with InfluxDBClient(url=url, token=token, org=org) as client:
    if not client.ping():
        raise RuntimeError("InfluxDB is not reachable")
```

### Delete data

Delete operations go through `delete_api()` and require an explicit time range plus predicate:

```python
from datetime import datetime, timedelta, timezone

with InfluxDBClient(url=url, token=token, org=org) as client:
    stop = datetime.now(timezone.utc)
    start = stop - timedelta(hours=1)

    client.delete_api().delete(
        start=start,
        stop=stop,
        predicate='_measurement="cpu"',
        bucket=bucket,
        org=org,
    )
```

## Async Usage

Use the async client only if the surrounding app is already async:

```python
import asyncio

from influxdb_client import Point
from influxdb_client.client.influxdb_client_async import InfluxDBClientAsync

async def main() -> None:
    async with InfluxDBClientAsync(url=url, token=token, org=org) as client:
        await client.write_api().write(
            bucket=bucket,
            record=Point("cpu").tag("host", "web-1").field("usage_idle", 91.2),
        )

        records = await client.query_api().query_stream(
            f'from(bucket: "{bucket}") |> range(start: -10m)'
        )
        async for record in records:
            print(record["_time"], record["_value"])

asyncio.run(main())
```

## Config And Auth Notes

Token auth is the preferred option for InfluxDB 2.x and Cloud.

The client also supports:

- username/password
- HTTP Basic auth

The `auth_basic` config flag is specifically for InfluxDB 1.8.x setups that are fronted by a reverse proxy using Basic auth.

Useful config keys from the upstream client config support:

- `timeout`: socket timeout in milliseconds
- `verify_ssl`: disable only for controlled local/testing cases
- `ssl_ca_cert`: custom CA bundle
- `cert_file` and `cert_key_file`: mTLS client certificates
- `connection_pool_maxsize`: useful for high-concurrency services
- `profilers`: Flux profiler configuration

If you use environment-based config, the supported names start with `INFLUXDB_V2_...`, including `INFLUXDB_V2_URL`, `INFLUXDB_V2_TOKEN`, `INFLUXDB_V2_ORG`, `INFLUXDB_V2_TIMEOUT`, and `INFLUXDB_V2_AUTH_BASIC`.

## Common Pitfalls

- The package name is `influxdb-client`, but the import is `influxdb_client`.
- `WriteApi` defaults to batching. In short-lived scripts, tests, lambdas, or request handlers, use `SYNCHRONOUS` or explicitly close the batching writer so data is flushed.
- If you do use batching writes, keep the writer as a long-lived singleton and register callbacks if you need to observe background write failures.
- This client is for InfluxDB 2.x and Flux-centric flows. Do not use it as the default choice for new InfluxDB 3.x work.
- The API is not backwards compatible with the old `influxdb-python` client. Migration usually requires changing imports, auth setup, and query shapes.
- InfluxDB 1.8 compatibility uses `/api/v2/query` and `/api/v2/write`; Flux must be enabled on the server for query compatibility.
- Python `datetime` does not preserve nanosecond precision. If you need nanosecond timestamps, use pandas timestamps and the related upstream nanosecond-precision guidance.
- Base install works, but upstream recommends `ciso8601` for much faster date parsing.
- Outside batching writes, the client does not apply a general retry policy by default. Set `retries=` on `InfluxDBClient` if your service needs explicit HTTP retry behavior.

## Version-Sensitive Notes

- Version used here: `1.50.0`
- Live PyPI version on March 12, 2026: `1.50.0`
- PyPI release date for `1.50.0`: January 23, 2026
- Python support declared on PyPI for this release: `>=3.7`
- The upstream changelog entry for `1.50.0` is a packaging/build change, not a documented application-level API shift, so current 1.4x usage patterns in the official docs still apply to 1.50.0.

## Official Sources

- PyPI: https://pypi.org/project/influxdb-client/
- README: https://github.com/influxdata/influxdb-client-python/blob/master/README.md
- Product docs: https://docs.influxdata.com/influxdb/v2/api-guide/client-libraries/python/
- API reference: https://influxdb-client.readthedocs.io/en/stable/
- Changelog: https://github.com/influxdata/influxdb-client-python/blob/master/CHANGELOG.md

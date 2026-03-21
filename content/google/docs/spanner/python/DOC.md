---
name: spanner
description: "Google Cloud Spanner Python client library for sessions, SQL reads, transactions, DML, and emulator-backed local development"
metadata:
  languages: "python"
  versions: "3.63.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,spanner,database,sql,gcp,transactions"
---

# Google Cloud Spanner Python Client

## Golden Rule

Use the official `google-cloud-spanner` package and authenticate with Application Default Credentials (ADC). Structure code around one long-lived `spanner.Client`, then derive `Instance` and `Database` handles from it.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-spanner==3.63.0"
```

Common alternatives:

```bash
uv add "google-cloud-spanner==3.63.0"
poetry add "google-cloud-spanner==3.63.0"
```

## Authentication And Environment

Enable the API once per Google Cloud project:

```bash
gcloud services enable spanner.googleapis.com
```

For local development with your user credentials:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

For deployed workloads, prefer attached service accounts. If you must use a key locally or in CI, point ADC at it:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

A simple app-level environment setup:

```bash
export GOOGLE_CLOUD_PROJECT="your-project-id"
export SPANNER_INSTANCE_ID="your-instance-id"
export SPANNER_DATABASE_ID="your-database-id"
```

## Initialize The Client

Create one reusable client, then derive instance and database handles:

```python
import os

from google.cloud import spanner

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
instance_id = os.environ["SPANNER_INSTANCE_ID"]
database_id = os.environ["SPANNER_DATABASE_ID"]

client = spanner.Client(project=project_id)
instance = client.instance(instance_id)
database = instance.database(database_id)
```

Notes:

- `Client` is gRPC-based. The official client docs explicitly note that no `_http` transport argument is accepted.
- Reuse the same `Client` instead of creating one per request.
- `Database` uses a session pool under the hood. The default is a `BurstyPool`; tune `pool=` only if you understand your concurrency and session lifecycle needs.

## Core Usage

### Read rows with a snapshot

Use `database.snapshot()` for consistent read-only work:

```python
from google.cloud import spanner

client = spanner.Client(project="your-project-id")
database = client.instance("my-instance").database("my-database")

with database.snapshot() as snapshot:
    results = snapshot.execute_sql(
        """
        SELECT SingerId, FirstName, LastName
        FROM Singers
        ORDER BY SingerId
        """
    )
    for row in results:
        print(row)
```

Key point:

- `Snapshot` is for reads. Do not try to mutate data from it.

### Insert or update rows with a batch

Use `database.batch()` for straightforward mutations:

```python
from google.cloud import spanner

client = spanner.Client(project="your-project-id")
database = client.instance("my-instance").database("my-database")

with database.batch() as batch:
    batch.insert(
        table="Singers",
        columns=("SingerId", "FirstName", "LastName"),
        values=[
            (1, "Marc", "Richards"),
            (2, "Catalina", "Smith"),
        ],
    )
```

Useful mutation methods include `insert`, `update`, `insert_or_update`, `replace`, and `delete`.

### Use a read-write transaction

Use `run_in_transaction()` when multiple reads and writes must commit atomically:

```python
from google.cloud import spanner

client = spanner.Client(project="your-project-id")
database = client.instance("my-instance").database("my-database")

def transfer_budget(transaction, singer_id, delta):
    row = transaction.execute_sql(
        """
        SELECT MarketingBudget
        FROM Albums
        WHERE SingerId = @singer_id
        """,
        params={"singer_id": singer_id},
        param_types={"singer_id": spanner.param_types.INT64},
    ).one()

    new_budget = row[0] + delta

    transaction.execute_update(
        """
        UPDATE Albums
        SET MarketingBudget = @budget
        WHERE SingerId = @singer_id
        """,
        params={"budget": new_budget, "singer_id": singer_id},
        param_types={
            "budget": spanner.param_types.INT64,
            "singer_id": spanner.param_types.INT64,
        },
    )

database.run_in_transaction(transfer_budget, singer_id=1, delta=5000)
```

Use `execute_update()` for DML inside transactions. For large standalone DML operations, check whether `execute_partitioned_dml()` is the better fit for your workload.

### Parameterized SQL

Do not format values directly into SQL strings. Use parameters and Spanner types:

```python
from google.cloud import spanner

with database.snapshot() as snapshot:
    rows = snapshot.execute_sql(
        """
        SELECT SingerId, FirstName
        FROM Singers
        WHERE LastName = @last_name
        """,
        params={"last_name": "Richards"},
        param_types={"last_name": spanner.param_types.STRING},
    )
    for row in rows:
        print(row)
```

## DB-API 2.0 Mode

This package also ships `google.cloud.spanner_dbapi`, a PEP 249 wrapper that is useful when you need a familiar DB-API cursor/connection interface.

```python
from google.cloud import spanner_dbapi

conn = spanner_dbapi.connect(
    "my-instance",
    "my-database",
    project="your-project-id",
)

with conn.cursor() as cursor:
    cursor.execute(
        "SELECT SingerId, FirstName FROM Singers WHERE SingerId = %s",
        (1,),
    )
    print(cursor.fetchall())

conn.close()
```

Use the DB-API layer when a framework expects DB-API semantics. Use `google.cloud.spanner` directly when you want native Spanner transaction and snapshot APIs.

## Emulator Setup

For local or test environments, start the emulator and point the client at it:

```bash
gcloud emulators spanner start
export SPANNER_EMULATOR_HOST="localhost:9010"
```

Then create the instance and database your tests expect:

```bash
gcloud spanner instances create test-instance \
  --config=emulator-config \
  --description="Spanner emulator" \
  --nodes=1

gcloud spanner databases create test-database --instance=test-instance
```

The library detects `SPANNER_EMULATOR_HOST`. When you configure the client manually, use anonymous credentials against the emulator:

```python
from google.auth.credentials import AnonymousCredentials
from google.cloud import spanner

client = spanner.Client(
    project="test-project",
    credentials=AnonymousCredentials(),
)
database = client.instance("test-instance").database("test-database")
```

## Common Pitfalls

- Do not use API keys. Cloud Spanner client authentication is based on ADC and IAM credentials.
- Do not create a fresh `spanner.Client()` for every query or HTTP request.
- `snapshot()` is read-only. Use `batch()` or `run_in_transaction()` for writes.
- Always parameterize SQL and provide `param_types`; agents often forget the type mapping and then produce invalid examples.
- The emulator is not production-equivalent. Validate IAM, quotas, leader routing, and latency-sensitive behavior against real Cloud Spanner before release.
- If you need custom routing or metrics behavior, check constructor flags like `route_to_leader_enabled`, `directed_read_options`, `disable_builtin_metrics`, and `resource_exhausted_behavior` in the current client reference instead of guessing names from older blog posts.
- If you use the DB-API wrapper, keep in mind that not every third-party ORM assumption maps cleanly onto Spanner semantics.

## Version-Sensitive Notes

- PyPI lists `3.63.0` as the current package version for this package.
- The Google Cloud `latest` reference root is usable for `3.63.0`, but rendered pages are not perfectly in sync: the changelog includes `3.63.0`, while some class pages under `latest` still show older minor-version badges.
- Because the docs root is `latest`, prefer the package version from PyPI when pinning dependencies and use the reference site for API shape, constructor flags, and examples.

## Official Sources

- Reference root: `https://docs.cloud.google.com/python/docs/reference/spanner/latest`
- Client usage guide: `https://docs.cloud.google.com/python/docs/reference/spanner/latest/client-usage`
- Client class reference: `https://cloud.google.com/python/docs/reference/spanner/latest/google.cloud.spanner_v1.client.Client`
- Database class reference: `https://cloud.google.com/python/docs/reference/spanner/latest/google.cloud.spanner_v1.database.Database`
- Changelog: `https://cloud.google.com/python/docs/reference/spanner/latest/changelog`
- Getting started: `https://docs.cloud.google.com/spanner/docs/getting-started/python`
- Emulator guide: `https://docs.cloud.google.com/spanner/docs/emulator`
- PyPI package page: `https://pypi.org/project/google-cloud-spanner/`
- Upstream repository: `https://github.com/googleapis/python-spanner`

---
name: bigtable
description: "Google Cloud Bigtable Python client library for ADC auth, classic admin/data access, async data workflows, and emulator-backed testing"
metadata:
  languages: "python"
  versions: "2.35.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,bigtable,google-cloud,gcp,nosql,grpc"
---

# Google Cloud Bigtable Python Client Library

## Golden Rule

Install `google-cloud-bigtable`, authenticate with Application Default Credentials (ADC), and pick one client surface before writing code:

- `google.cloud.bigtable` for the classic synchronous client and admin/table helpers
- `google.cloud.bigtable.data` for the newer async data-plane client

The official Bigtable Python docs explicitly note that the async data client is focused on data reads and writes. Table creation and other admin operations still use the regular synchronous admin client.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-bigtable==2.35.0"
```

Common alternatives:

```bash
uv add "google-cloud-bigtable==2.35.0"
poetry add "google-cloud-bigtable==2.35.0"
```

## Authentication And Setup

Before using the client library, make sure all of the following are true:

- The target Google Cloud project exists
- Billing is enabled
- The Cloud Bigtable API is enabled
- The caller has IAM permissions for the instance and tables
- The instance, table, and column family already exist unless your code is explicitly creating them

For local development, the normal path is ADC with the Google Cloud CLI:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="my-gcp-project"
```

If you must use a service account key file:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="my-gcp-project"
```

Basic classic client setup:

```python
from google.cloud import bigtable

client = bigtable.Client(project="my-gcp-project", admin=False)
instance = client.instance("my-instance")
table = instance.table("my-table")
```

Basic async data client setup:

```python
from google.cloud import bigtable

client = bigtable.data.BigtableDataClientAsync(project="my-gcp-project")
table = client.get_table("my-instance", "my-table")
```

Pass `project` explicitly unless you have already verified that project discovery is correct in the runtime. Bigtable failures are often caused by pointing at the wrong project or instance rather than by bad row logic.

## Choose The Client Surface First

### Classic synchronous client

Use `google.cloud.bigtable` when you need:

- synchronous request flow
- table creation or schema/admin helpers
- compatibility with older examples that use `Client`, `Instance`, `Table`, and `DirectRow`

### Async data client

Use `google.cloud.bigtable.data` when you need:

- an async application flow
- the newer data-plane API for row reads and writes
- bulk mutation and streaming reads in an asyncio codebase

The official docs and PyPI page both warn against dropping the async client into an otherwise synchronous codebase just to use a newer API surface.

## Core Usage

### Create a table with the classic admin client

The official hello-world sample creates tables with the regular admin client, not the async data client:

```python
from google.cloud import bigtable
from google.cloud.bigtable import column_family

project_id = "my-gcp-project"
instance_id = "my-instance"
table_id = "my-table"

client = bigtable.Client(project=project_id, admin=True)
instance = client.instance(instance_id)
table = instance.table(table_id)

column_families = {
    b"cf1": column_family.MaxVersionsGCRule(2),
}

if not table.exists():
    table.create(column_families=column_families)
```

### Write and read a row with the classic synchronous API

```python
import datetime

from google.cloud import bigtable
from google.cloud.bigtable import row_filters

project_id = "my-gcp-project"
instance_id = "my-instance"
table_id = "my-table"
column_family_id = b"cf1"
column = b"status"

client = bigtable.Client(project=project_id, admin=False)
table = client.instance(instance_id).table(table_id)

row = table.direct_row(b"device#123")
row.set_cell(
    column_family_id,
    column,
    b"ok",
    timestamp=datetime.datetime.utcnow(),
)
table.mutate_rows([row])

stored = table.read_row(
    b"device#123",
    row_filters.CellsColumnLimitFilter(1),
)

if stored is not None:
    cell = stored.cells[column_family_id.decode("utf-8")][column][0]
    print(cell.value.decode("utf-8"))
```

### Write and read rows with the async data client

```python
import asyncio

from google.cloud import bigtable

project_id = "my-gcp-project"
instance_id = "my-instance"
table_id = "my-table"
column_family_id = b"cf1"
column = b"status"

async def main() -> None:
    client = bigtable.data.BigtableDataClientAsync(project=project_id)
    table = client.get_table(instance_id, table_id)

    mutation = bigtable.data.RowMutationEntry(
        b"device#123",
        bigtable.data.SetCell(column_family_id, column, b"ok"),
    )
    await table.bulk_mutate_rows([mutation])

    row_filter = bigtable.data.row_filters.CellsColumnLimitFilter(1)
    row = await table.read_row(b"device#123", row_filter=row_filter)

    if row is not None:
        print(row.cells[0].value.decode("utf-8"))

asyncio.run(main())
```

### Scan rows with the async data client

```python
import asyncio

from google.cloud import bigtable

async def main() -> None:
    client = bigtable.data.BigtableDataClientAsync(project="my-gcp-project")
    table = client.get_table("my-instance", "my-table")
    query = bigtable.data.ReadRowsQuery(
        row_filter=bigtable.data.row_filters.CellsColumnLimitFilter(1),
    )

    async for row in await table.read_rows_stream(query):
        print(row.row_key)

asyncio.run(main())
```

## Emulator

For local integration tests, use the official Bigtable emulator:

```bash
gcloud beta emulators bigtable start --host-port=localhost:8086
export BIGTABLE_EMULATOR_HOST=localhost:8086
```

Important emulator behavior from the official docs:

- it is in-memory only and does not persist data across runs
- it does not support a secure connection
- it does not provide admin APIs for creating or managing instances and clusters
- once `BIGTABLE_EMULATOR_HOST` is set, the client library connects to the emulator automatically

Unset `BIGTABLE_EMULATOR_HOST` after tests so production code does not accidentally point at the emulator.

## Common Pitfalls

- The package name is `google-cloud-bigtable`, but the common import is `from google.cloud import bigtable`.
- Do not mix classic `Client` / `Table` examples with `bigtable.data` examples unless you deliberately adapt imports, row types, and method names.
- The async client is not a general replacement for synchronous code. Google explicitly recommends using it only in an async codebase.
- Table creation is an admin operation. The async data client does not replace the regular admin client for that workflow.
- Bigtable rows are stored in sorted row-key order. The official sample warns that sequential numeric row keys are fine for demos but can create poor production distribution and hotspotting.
- Data examples assume the instance, table, and column family already exist. Missing schema is a common cause of first-run failures.
- If `BIGTABLE_EMULATOR_HOST` is left set in a shell or CI job, the library will connect to the local emulator instead of the real service.

## Version-Sensitive Notes For 2.35.0

- As of `2026-03-12`, PyPI and the official Google Cloud Bigtable Python reference both align on `2.35.0` as the current package release.
- The official docs still call out `v2.23.0` as the introduction of `BigtableDataClientAsync` under `google.cloud.bigtable.data`. Older examples written before that release often show only the classic client surface.
- PyPI lists the package requirement as `Python >= 3.7`.
- If you are adapting older blog posts or Stack Overflow answers, verify whether they predate the newer `google.cloud.bigtable.data` API before copying method names into current code.

## Official Sources

- Bigtable Python reference root: `https://cloud.google.com/python/docs/reference/bigtable/latest`
- Bigtable Python hello-world sample: `https://cloud.google.com/bigtable/docs/samples-python-hello`
- Google Cloud ADC guide: `https://cloud.google.com/docs/authentication/provide-credentials-adc`
- Bigtable emulator guide: `https://cloud.google.com/bigtable/docs/emulator`
- PyPI package page: `https://pypi.org/project/google-cloud-bigtable/`

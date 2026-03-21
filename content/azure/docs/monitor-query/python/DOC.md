---
name: monitor-query
description: "azure-monitor-query package guide for Python 2.0.0 with LogsQueryClient setup, Entra auth, batch queries, and 2.x migration notes"
metadata:
  languages: "python"
  versions: "2.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-monitor,log-analytics,logs,observability,python"
---

# azure-monitor-query Python Package Guide

## What It Is

`azure-monitor-query` is the official Azure SDK package for querying Azure Monitor logs from Python.

For `2.0.0`, this package is focused on logs queries:

- use `LogsQueryClient` for Log Analytics workspace queries
- use `LogsBatchQuery` with `query_batch()` for multiple logs queries in one request
- use `query_resource()` when you want logs scoped to an Azure resource instead of a workspace

Important version-sensitive change:

- `azure-monitor-query` `2.0.0` no longer carries the metrics query clients from older releases
- if older code uses `MetricsQueryClient` or `MetricsClient`, move that code to the separate `azure-monitor-querymetrics` package

This package is for querying existing Azure Monitor logs. It is not the ingestion client and it does not manage alerts, dashboards, or Azure Monitor resources.

## Install

Install the version used here plus Azure Identity:

```bash
python -m pip install "azure-monitor-query==2.0.0" "azure-identity"
```

If you use the async client, add an async transport:

```bash
python -m pip install aiohttp
```

If you need metrics queries too, install the split metrics package separately:

```bash
python -m pip install "azure-monitor-querymetrics" "azure-identity"
```

## Initialize And Authenticate

The standard Azure SDK path is `DefaultAzureCredential` from `azure-identity`.

For local development, one of these usually works:

- `az login`
- Azure Developer CLI sign-in
- service principal environment variables
- workload identity or managed identity when running in Azure

At minimum, you usually need:

- a Log Analytics workspace ID for `query_workspace()`
- a full Azure resource ID for `query_resource()`

Basic setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.monitor.query import LogsQueryClient

credential = DefaultAzureCredential()
client = LogsQueryClient(credential)

workspace_id = os.environ["AZURE_MONITOR_WORKSPACE_ID"]
resource_id = os.environ["AZURE_RESOURCE_ID"]
```

For sovereign clouds, pass an explicit endpoint when creating the client:

```python
from azure.identity import DefaultAzureCredential
from azure.monitor.query import LogsQueryClient

credential = DefaultAzureCredential()
client = LogsQueryClient(
    credential=credential,
    endpoint="https://api.loganalytics.us",
)
```

## Core Usage

### Query A Log Analytics Workspace

Use `query_workspace()` when you already know the Log Analytics workspace that holds the data.

```python
from datetime import timedelta

from azure.monitor.query import LogsQueryClient, LogsQueryStatus

result = client.query_workspace(
    workspace_id=workspace_id,
    query="""
    AppRequests
    | take 5
    """,
    timespan=timedelta(hours=1),
)

if result.status == LogsQueryStatus.SUCCESS:
    tables = result.tables
elif result.status == LogsQueryStatus.PARTIAL:
    print(result.partial_error.code, result.partial_error.message)
    tables = result.partial_data
else:
    raise RuntimeError(f"Unexpected status: {result.status}")

for table in tables:
    columns = [column.name for column in table.columns]
    for row in table.rows:
        print(dict(zip(columns, row)))
```

Notes:

- pass `timespan=` explicitly; it can be a `timedelta`, `(start, duration)`, or `(start, end)`
- handle partial results instead of assuming every query returns `SUCCESS`
- `query_workspace()` supports `additional_workspaces`, `include_statistics`, `include_visualization`, and `server_timeout`

### Query Logs For An Azure Resource

Use `query_resource()` when your query should follow the Azure resource rather than a single workspace.

```python
from datetime import timedelta

from azure.monitor.query import LogsQueryStatus

result = client.query_resource(
    resource_id,
    query="""
    AzureActivity
    | take 5
    """,
    timespan=timedelta(hours=2),
)

if result.status == LogsQueryStatus.SUCCESS:
    for table in result.tables:
        print(table.name, len(table.rows))
elif result.status == LogsQueryStatus.PARTIAL:
    print(result.partial_error.message)
```

Use this when your app naturally starts from a resource ID, such as an App Service, Function App, or VM.

### Run Multiple Logs Queries In One Call

Use `LogsBatchQuery` plus `query_batch()` for fewer round trips when you need several queries.

```python
from datetime import timedelta

from azure.monitor.query import LogsBatchQuery, LogsQueryStatus

requests = [
    LogsBatchQuery(
        workspace_id=workspace_id,
        query="AppRequests | take 5",
        timespan=timedelta(hours=1),
    ),
    LogsBatchQuery(
        workspace_id=workspace_id,
        query="AzureActivity | summarize count() by Category",
        timespan=timedelta(hours=6),
    ),
]

results = client.query_batch(requests)

for item in results:
    if item.status == LogsQueryStatus.SUCCESS:
        print(item.tables[0].rows)
    elif item.status == LogsQueryStatus.PARTIAL:
        print(item.partial_error.code, item.partial_error.message)
    else:
        print(item.code, item.message)
```

Batch results can mix success, partial, and failure states. Inspect each item independently.

### Query Across Multiple Workspaces

If your main target is one workspace but the query needs data from others, pass `additional_workspaces`.

```python
from datetime import timedelta

result = client.query_workspace(
    workspace_id=workspace_id,
    query="AppRequests | summarize count() by bin(TimeGenerated, 5m)",
    timespan=timedelta(hours=1),
    additional_workspaces=[
        "00000000-0000-0000-0000-000000000000",
        "11111111-1111-1111-1111-111111111111",
    ],
)
```

The extra entries can be workspace IDs, qualified workspace names, or Azure resource IDs according to the official client docs.

### Include Query Statistics

Use `include_statistics=True` when you need query execution details for debugging or tuning.

```python
from datetime import timedelta

result = client.query_workspace(
    workspace_id=workspace_id,
    query="AppRequests | count",
    timespan=timedelta(hours=1),
    include_statistics=True,
)

print(result.statistics)
```

The service can also return visualization metadata with `include_visualization=True`.

### Async Client

The async surface lives under `azure.monitor.query.aio`.

```python
from datetime import timedelta

from azure.identity.aio import DefaultAzureCredential
from azure.monitor.query.aio import LogsQueryClient

credential = DefaultAzureCredential()
client = LogsQueryClient(credential)

try:
    result = await client.query_workspace(
        workspace_id=workspace_id,
        query="AppRequests | take 5",
        timespan=timedelta(hours=1),
    )
finally:
    await client.close()
    await credential.close()
```

Install `aiohttp` if your environment does not already provide an async transport.

## Configuration Notes

- `query_workspace()` expects a Log Analytics workspace ID.
- `query_resource()` expects a full Azure resource ID.
- The package import is `azure.monitor.query`, not `azure_monitor_query`.
- For non-public Azure clouds, set `endpoint=` on `LogsQueryClient`.
- `server_timeout` is a server-side limit for logs queries, not a client socket timeout.
- `azure-monitor-query` `2.0.0` requires a modern Python runtime; PyPI lists `Requires: Python >=3.9`.

## Common Pitfalls

### Old Metrics Examples Break On 2.0.0

If copied code imports any of these from `azure.monitor.query`, it is from the pre-split package surface:

- `MetricsQueryClient`
- `MetricsClient`
- metrics model types from the old combined package

For `2.0.0`, move metrics code to `azure-monitor-querymetrics`. Do not try to mix old metrics examples into this package.

### Package Name And Import Name Differ

Install with:

```bash
pip install azure-monitor-query
```

Import with:

```python
from azure.monitor.query import LogsQueryClient
```

### Workspace ID And Resource ID Are Not Interchangeable

This is the most common wiring mistake:

- `query_workspace()` takes a workspace ID
- `query_resource()` takes a resource ID like `/subscriptions/.../resourceGroups/.../providers/...`

Passing the wrong identifier usually produces confusing authorization or not-found failures.

### Partial Results Are Normal

Logs queries can return `LogsQueryStatus.PARTIAL`. Treat that as a handled state and inspect:

- `partial_error`
- `partial_data`

Do not assume every successful HTTP response means full query success.

### Async Cleanup Matters

Close both the async client and the async credential. If you skip cleanup, long-running apps and tests can leak connections.

## Version Notes For Agents

- Version used here: `2.0.0`
- PyPI latest at verification time: `2.0.0`
- PyPI release date for `2.0.0`: `2025-07-30`
- The Microsoft Learn docs root is a moving latest-style reference, not a version-pinned static snapshot

The `2.0.0` breaking change that matters most for code generation is the package split:

- logs queries stay in `azure-monitor-query`
- metrics queries moved to `azure-monitor-querymetrics`

If a project is upgrading from `1.x`, check imports first before changing query code.

## Official Sources Used

- Microsoft Learn overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/monitor-query-readme?view=azure-python`
- Microsoft Learn `LogsQueryClient`: `https://learn.microsoft.com/en-us/python/api/azure-monitor-query/azure.monitor.query.logsqueryclient?view=azure-python`
- Microsoft Learn `LogsBatchQuery`: `https://learn.microsoft.com/en-us/python/api/azure-monitor-query/azure.monitor.query.logsbatchquery?view=azure-python`
- PyPI package page: `https://pypi.org/project/azure-monitor-query/`

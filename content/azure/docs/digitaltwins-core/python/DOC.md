---
name: digitaltwins-core
description: "azure-digitaltwins-core for Python with Azure Identity auth, model/twin operations, relationships, queries, and concurrency-aware updates"
metadata:
  languages: "python"
  versions: "1.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,digitaltwins,iot,azure-identity,dtdl,twins"
---

# azure-digitaltwins-core Python Package Guide

## What It Is

`azure-digitaltwins-core` is the Azure SDK package for Azure Digital Twins data-plane operations in Python.

Use it when code needs to:

- create, list, decommission, and delete DTDL models
- create, fetch, patch, and delete digital twins
- create, read, patch, and delete relationships
- query twins and relationships with the ADT query language
- manage event routes and publish twin or component telemetry

Primary clients:

- sync: `azure.digitaltwins.core.DigitalTwinsClient`
- async: `azure.digitaltwins.core.aio.DigitalTwinsClient`

## Install

For the package version pinned in this session:

```bash
python -m pip install "azure-digitaltwins-core==1.3.0" azure-identity
```

PyPI metadata for `1.3.0` lists `Requires: Python >=3.7`.

If you use the async client or async Azure Identity credentials, install an async transport too:

```bash
python -m pip install aiohttp
```

## Golden Rules

- Use Azure AD credentials from `azure-identity`; `DefaultAzureCredential` is the normal starting point.
- Pass the instance URL as `endpoint`, for example `https://<instance>.api.<region>.digitaltwins.azure.net`.
- The caller needs Azure Digital Twins data-plane RBAC, commonly `Azure Digital Twins Data Owner` for writes and `Azure Digital Twins Data Reader` for reads.
- The authenticated identity must come from the same Microsoft Entra tenant as the Azure Digital Twins instance or requests can fail with `404 Sub-Domain not found`.
- Query results are paged and eventually consistent. Microsoft documents query latency of less than 10 seconds in normal cases.
- Use ETags plus `match_condition` on update or delete paths when concurrent writers are possible.

## Authentication And Setup

### Environment Variables

```bash
export AZURE_URL="https://example.api.wus2.digitaltwins.azure.net"
```

For local service-principal auth, `DefaultAzureCredential` can use:

```bash
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_ID="<client-id>"
export AZURE_CLIENT_SECRET="<client-secret>"
```

For local developer auth, `az login` is often enough.

### Sync Client

```python
import os

from azure.digitaltwins.core import DigitalTwinsClient
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential()
client = DigitalTwinsClient(
    endpoint=os.environ["AZURE_URL"],
    credential=credential,
)
```

### Async Client

```python
import os

from azure.digitaltwins.core.aio import DigitalTwinsClient
from azure.identity.aio import DefaultAzureCredential

credential = DefaultAzureCredential()
client = DigitalTwinsClient(
    endpoint=os.environ["AZURE_URL"],
    credential=credential,
)
```

Close async resources explicitly:

```python
await client.close()
await credential.close()
```

### Useful Client Options

`DigitalTwinsClient(..., **kwargs)` accepts Azure Core pipeline options. The useful ones in practice are:

- `api_version`
- `logging_enable`
- `retry_total`, `retry_connect`, `retry_read`, `retry_status`
- `retry_backoff_factor`, `retry_backoff_max`
- `connection_timeout`, `read_timeout`

Practical guidance:

- set retries and timeouts deliberately for query-heavy or patch-heavy flows
- keep `logging_enable=False` in production unless you are debugging request failures
- override `api_version` only if you know the target instance supports it

## Core Usage

### Create DTDL Models

`create_models()` takes a list of DTDL JSON interface definitions.

```python
import os

from azure.digitaltwins.core import DigitalTwinsClient
from azure.identity import DefaultAzureCredential

client = DigitalTwinsClient(os.environ["AZURE_URL"], DefaultAzureCredential())

models = [
    {
        "@id": "dtmi:example:Room;1",
        "@type": "Interface",
        "@context": "dtmi:dtdl:context;2",
        "displayName": "Room",
        "contents": [
            {"@type": "Property", "name": "Temperature", "schema": "double"},
            {"@type": "Telemetry", "name": "Humidity", "schema": "double"},
        ],
    }
]

created = client.create_models(models)
print(created[0]["id"])
```

### Upsert And Read A Twin

Set the model through `$metadata.$model` when creating or replacing a twin.

```python
twin_id = "room-101"
twin_body = {
    "$metadata": {"$model": "dtmi:example:Room;1"},
    "Temperature": 21.5,
}

client.upsert_digital_twin(twin_id, twin_body)
stored = client.get_digital_twin(twin_id)
print(stored["Temperature"])
```

### Patch A Twin

`update_digital_twin()` uses JSON Patch operations.

```python
patch = [
    {"op": "replace", "path": "/Temperature", "value": 22.0},
]

client.update_digital_twin("room-101", patch)
```

### Query Twins

`query_twins()` returns an iterator over a paged query result.

```python
query = """
SELECT twin
FROM digitaltwins twin
WHERE IS_OF_MODEL(twin, 'dtmi:example:Room;1')
"""

for item in client.query_twins(query):
    print(item["$dtId"])
```

If code needs the latest value right after a write, prefer `get_digital_twin()` for the known ID instead of assuming the query index updated immediately.

### Create And Traverse Relationships

```python
relationship = {
    "$relationshipId": "building-1-contains-room-101",
    "$sourceId": "building-1",
    "$relationshipName": "contains",
    "$targetId": "room-101",
}

client.upsert_relationship(
    "building-1",
    relationship["$relationshipId"],
    relationship,
)

for rel in client.list_relationships("building-1"):
    print(rel["$relationshipName"], rel["$targetId"])
```

For reverse traversal:

```python
for rel in client.list_incoming_relationships("room-101"):
    print(rel["relationshipName"], rel["sourceId"])
```

### Update A Component

Use component patches when the model contains a component instead of a top-level property.

```python
component_patch = [
    {"op": "replace", "path": "/Temperature", "value": 23.0},
]

client.update_component("room-101", "thermostat", component_patch)
```

### Publish Telemetry

Twin telemetry:

```python
client.publish_telemetry(
    "room-101",
    {"Humidity": 48.2},
    message_id="humidity-20260312-01",
)
```

Component telemetry:

```python
client.publish_component_telemetry(
    "room-101",
    "thermostat",
    {"Humidity": 48.2},
    message_id="component-20260312-01",
)
```

Telemetry is only useful if the instance has event routes configured to send it to Event Grid, Event Hubs, or Service Bus.

### Event Routes

The client can create and manage event routes once the destination endpoints exist.

```python
from azure.digitaltwins.core import DigitalTwinsEventRoute

route = DigitalTwinsEventRoute(
    endpoint_name="eh-endpoint",
    filter="type = 'Microsoft.DigitalTwins.Twin.Update'",
)

client.upsert_event_route("twin-updates", route)
```

## Concurrency And Conditional Writes

The `1.3.0` client exposes `etag` and `match_condition` on update, upsert, and delete operations such as:

- `upsert_digital_twin`
- `upsert_relationship`
- `update_component`
- `update_digital_twin`
- `delete_digital_twin`
- `delete_relationship`

Use them when multiple workers may touch the same twin or relationship:

```python
from azure.core import MatchConditions

current = client.get_digital_twin("room-101")
etag = current["$etag"]

client.update_digital_twin(
    "room-101",
    [{"op": "replace", "path": "/Temperature", "value": 24.0}],
    etag=etag,
    match_condition=MatchConditions.IfNotModified,
)
```

Without this, a later writer can silently overwrite an earlier update.

## Error Handling

Expect Azure Core exceptions:

- `azure.core.exceptions.ResourceNotFoundError`
- `azure.core.exceptions.ResourceExistsError`
- `azure.core.exceptions.HttpResponseError`
- auth failures raised by `azure-identity`

```python
from azure.core.exceptions import HttpResponseError, ResourceNotFoundError

try:
    twin = client.get_digital_twin("room-101")
except ResourceNotFoundError:
    print("twin not found")
except HttpResponseError as exc:
    print(f"request failed: {exc.status_code} {exc.message}")
```

## Common Pitfalls

- Import name is `azure.digitaltwins.core`, not `azure_digitaltwins_core`.
- `DefaultAzureCredential` tries several credential sources in order. A stale local login can be selected before the credential you expected.
- Azure Digital Twins uses tenant-bound auth. Cross-tenant identities commonly fail even when the URL is correct.
- Query language does not support every SQL habit. Check query restrictions before assuming aggregations or ordering are available.
- `upsert_digital_twin()` replaces the twin document you send. Do not treat it like a patch helper.
- Relationship IDs are scoped to the source twin. Reusing a relationship ID with the same source twin overwrites that relationship.
- Async code must close both the client and the async credential.

## Version-Sensitive Notes For `1.3.0`

- This guide is intentionally pinned to the version used here `1.3.0`.
- PyPI currently lists `1.3.0` as the latest release.
- Microsoft Learn package readme examples align with the `1.3.0` API surface, including telemetry publishing and conditional update parameters.
- Older blog posts for `1.0.x` often miss `etag` and `match_condition` kwargs on write operations. Prefer current Microsoft Learn signatures when writing new code.
- The package metadata still allows Python `>=3.7`, but check the rest of your Azure SDK stack before standardizing on an older interpreter.

## Official Sources Used

- Microsoft Learn package index: `https://learn.microsoft.com/en-us/python/api/azure-digitaltwins-core/`
- Microsoft Learn package readme: `https://learn.microsoft.com/en-us/python/api/overview/azure/digitaltwins-core-readme?view=azure-python`
- Microsoft Learn `DigitalTwinsClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-digitaltwins-core/azure.digitaltwins.core.digitaltwinsclient?view=azure-python`
- Microsoft Learn Azure Digital Twins API and SDK overview: `https://learn.microsoft.com/en-us/azure/digital-twins/concepts-apis-sdks`
- Microsoft Learn authentication guidance: `https://learn.microsoft.com/en-us/azure/digital-twins/how-to-authenticate-client`
- Microsoft Learn Azure Identity readme: `https://learn.microsoft.com/en-us/python/api/overview/azure/identity-readme?view=azure-python`
- Microsoft Learn query language guidance: `https://learn.microsoft.com/en-us/azure/digital-twins/concepts-query-language`
- PyPI package page: `https://pypi.org/project/azure-digitaltwins-core/`

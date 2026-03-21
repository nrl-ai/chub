---
name: maps-route
description: "Azure Maps Route client library for Python (preview) with auth, directions, route range, matrix, and migration notes"
metadata:
  languages: "python"
  versions: "1.0.0b3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-maps,route,routing,geospatial,python"
---

# azure-maps-route Python Package Guide

## Golden Rule

Use `azure-maps-route` only as a pinned preview dependency, initialize `MapsRouteClient` with the correct Azure Maps credential type, and prefer the current Microsoft Learn class reference over older PyPI README snippets when method names disagree.

## Install

Pin the version you tested:

```bash
python -m pip install "azure-maps-route==1.0.0b3"
```

If you want the newest preview build instead of an exact pin:

```bash
python -m pip install --pre azure-maps-route
```

Common companion packages:

```bash
python -m pip install azure-identity
python -m pip install azure-mgmt-maps
python -m pip install aiohttp
```

Use them for:

- `azure-identity`: Microsoft Entra ID or managed identity auth
- `azure-mgmt-maps`: generating SAS tokens programmatically
- `aiohttp`: async transport for `azure.maps.route.aio`

## Service Setup

Before writing code, make sure you have:

1. an Azure subscription
2. an Azure Maps account/resource
3. one supported auth path: subscription key, SAS token, or Microsoft Entra ID

Create a Maps account with Azure CLI:

```bash
az maps account create \
  --kind Gen2 \
  --account-name <maps-account> \
  --resource-group <resource-group> \
  --sku G2
```

## Authentication And Client Initialization

### Subscription key

```bash
export AZURE_SUBSCRIPTION_KEY="..."
```

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.route import MapsRouteClient

client = MapsRouteClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)
```

### SAS token

`1.0.0b3` adds SAS authentication support. Generate the token with `azure-mgmt-maps`, then pass it through `AzureSasCredential`.

```bash
export AZURE_MAPS_SAS_TOKEN="..."
```

```python
import os

from azure.core.credentials import AzureSasCredential
from azure.maps.route import MapsRouteClient

client = MapsRouteClient(
    credential=AzureSasCredential(os.environ["AZURE_MAPS_SAS_TOKEN"])
)
```

### Microsoft Entra ID

For token-based auth, you need both the normal Azure identity environment variables and the Azure Maps account client ID.

```bash
export AZURE_CLIENT_ID="app-client-id"
export AZURE_CLIENT_SECRET="app-client-secret"
export AZURE_TENANT_ID="tenant-id"
export MAPS_CLIENT_ID="azure-maps-account-client-id"
```

```python
import os

from azure.identity import DefaultAzureCredential
from azure.maps.route import MapsRouteClient

client = MapsRouteClient(
    credential=DefaultAzureCredential(),
    client_id=os.environ["MAPS_CLIENT_ID"],
)
```

Important notes:

- `MAPS_CLIENT_ID` is the Azure Maps account client ID, not your Entra app registration client ID.
- Microsoft Learn’s broader Azure Maps Python SDK guide uses `MAPS_CLIENT_ID`; keep that name consistent in your project even if older snippets use different names.

## Core Usage

### Get route directions

`get_route_directions()` is the main sync call for point-to-point routing. The current SDK surface takes route points as `(latitude, longitude)` tuples.

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.route import MapsRouteClient

client = MapsRouteClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)

result = client.get_route_directions(
    route_points=[
        (47.60323, -122.33028),
        (47.62050, -122.34930),
    ]
)

route = result.routes[0]
print(route.summary.length_in_meters)
print(route.summary.travel_time_in_seconds)
```

### Get route range

Use `get_route_range()` when you need an isochrone-style reachable area from a starting point.

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.route import MapsRouteClient

client = MapsRouteClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)

result = client.get_route_range(
    coordinates=(47.60323, -122.33028),
    time_budget_in_sec=1800,
)

print(result.reachable_range.center.latitude)
print(result.reachable_range.center.longitude)
```

## Matrix And Batch Operations

### Route matrix

Use matrix APIs when you need travel-time or distance summaries across many origin and destination pairs instead of full turn-by-turn routes.

Relevant methods in the current class reference:

- sync matrix: `request_route_matrix_sync(...)`
- async matrix submit: `begin_request_route_matrix(...)`
- async matrix retrieval by id: `begin_get_route_matrix(...)`

Important constraints:

- sync matrix limit: `100` cells
- async matrix limit: `700` cells
- async matrix results are retained for `14` days
- since `1.0.0b2`, matrix requests must use a `RouteMatrixQuery` object rather than a plain `dict`

### Route directions batch

Use batch directions when you need many direction queries in one request.

Relevant methods in the current class reference:

- sync batch: `request_route_directions_batch_sync(...)`
- async batch submit: `begin_request_route_directions_batch(...)`
- batch retrieval flow: `begin_get_route_directions_batch(...)`

Important constraints:

- sync batch limit: `100` queries
- async batch limit: `700` queries
- async batch results are retained for `14` days

## Async Usage

Install an async transport first, then use the `aio` client.

```python
import asyncio
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.route.aio import MapsRouteClient

async def main() -> None:
    async with MapsRouteClient(
        credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
    ) as client:
        result = await client.get_route_directions(
            route_points=[
                (47.60323, -122.33028),
                (47.62050, -122.34930),
            ]
        )
        print(result.routes[0].summary.length_in_meters)

asyncio.run(main())
```

## Errors, Logging, And Debugging

Azure Maps Route raises Azure Core exceptions such as `HttpResponseError`.

```python
import logging
import os
import sys

from azure.core.credentials import AzureKeyCredential
from azure.core.exceptions import HttpResponseError
from azure.maps.route import MapsRouteClient

logger = logging.getLogger("azure.maps.route")
logger.setLevel(logging.DEBUG)
logger.addHandler(logging.StreamHandler(sys.stdout))

client = MapsRouteClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"]),
    logging_enable=True,
)

try:
    client.get_route_directions(route_points=[(47.60323, -122.33028), (47.62050, -122.34930)])
except HttpResponseError as exc:
    print(exc)
    print(getattr(exc, "error_code", None))
```

## Common Pitfalls

- The package is still preview-only. Pin `1.0.0b3` or the exact preview you validated.
- Older PyPI README examples still use `begin_get_route_matrix_result(...)` and `begin_get_route_directions_batch_result(...)`. The current Microsoft Learn class reference documents `begin_get_route_matrix(...)` and `begin_get_route_directions_batch(...)`.
- Microsoft Entra auth needs the Azure Maps account client ID in addition to `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, and `AZURE_TENANT_ID`.
- The current SDK uses Route v1-style tuple inputs like `(latitude, longitude)`. Do not copy those tuples directly into newer Azure Maps Routing `2025-01-01` REST examples, which use GeoJSON `[longitude, latitude]`.
- Matrix code written against `1.0.0b1` will break on `1.0.0b2+` if it still passes a plain dictionary instead of `RouteMatrixQuery`.
- For async matrix or async batch flows, results expire after 14 days.

## Version-Sensitive Notes

- `1.0.0b3` adds SAS-based authentication support.
- `1.0.0b2` changes matrix inputs to `RouteMatrixQuery` and removes Python 3.6 support.
- The Python SDK docs are still published under `view=azure-python-preview`.
- Azure Maps now has an official migration guide from Route v1.0 to Routing `2025-01-01`. That newer REST surface changes request shapes and behavior, including replacing older `GET` route directions and `GET` route range patterns with `POST`-based flows. If you need features from the newer Routing service, validate against the REST migration guide instead of assuming this SDK preview already covers them.

## Official Sources

- Microsoft Learn package API root: https://learn.microsoft.com/en-us/python/api/azure-maps-route/
- Microsoft Learn `MapsRouteClient` class: https://learn.microsoft.com/en-us/python/api/azure-maps-route/azure.maps.route.mapsrouteclient?view=azure-python-preview
- Azure Maps Python SDK guide: https://learn.microsoft.com/en-us/azure/azure-maps/how-to-dev-guide-py-sdk
- Azure Maps Route v1 migration guide: https://learn.microsoft.com/en-us/azure/azure-maps/migrate-route-v1-api
- PyPI package page: https://pypi.org/project/azure-maps-route/

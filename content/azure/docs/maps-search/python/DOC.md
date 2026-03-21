---
name: maps-search
description: "Azure Maps Search SDK for Python preview package for geocoding, reverse geocoding, batch search, and polygon boundaries"
metadata:
  languages: "python"
  versions: "2.0.0b2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,maps,search,geocoding,reverse-geocoding,location,python"
---

# Azure Maps Search Python Package Guide

## Golden Rule

Use `azure-maps-search` for Azure Maps Search calls, import `MapsSearchClient` from `azure.maps.search`, and treat the current line as preview-only. Pin the beta version your project expects, use `--pre` for unpinned installs, and do not copy pre-`2.0.0b1` examples that still call removed methods like `fuzzy_search`.

## Install

Because the package is still published as a beta, prefer an explicit prerelease pin:

```bash
python -m pip install --pre "azure-maps-search==2.0.0b2"
```

Common companion packages:

```bash
python -m pip install azure-identity
python -m pip install azure-mgmt-maps
```

Use them only when needed:

- `azure-identity`: Microsoft Entra authentication via `DefaultAzureCredential`
- `azure-mgmt-maps`: generating Azure Maps SAS tokens

If you omit the version pin and just run `pip install azure-maps-search`, pip may skip the package because there is no stable final release on PyPI as of March 12, 2026.

## Authentication And Setup

`MapsSearchClient` supports three auth modes in the official package overview:

1. Azure Maps subscription key
2. Microsoft Entra ID with an Azure Maps account client ID
3. SAS token via `AzureSASCredential`

### Subscription key

This is the simplest path and the clearest starting point for scripts and services.

```bash
export AZURE_SUBSCRIPTION_KEY="your-azure-maps-key"
```

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.search import MapsSearchClient

client = MapsSearchClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"]),
)
```

### Microsoft Entra ID

For Entra auth, `DefaultAzureCredential()` is not enough on its own. The Azure Maps client also needs the Maps account client ID.

```bash
export AZURE_CLIENT_ID="your-app-client-id"
export AZURE_CLIENT_SECRET="your-app-client-secret"
export AZURE_TENANT_ID="your-tenant-id"
export MAPS_CLIENT_ID="your-azure-maps-account-client-id"
```

```python
import os

from azure.identity import DefaultAzureCredential
from azure.maps.search import MapsSearchClient

client = MapsSearchClient(
    client_id=os.environ["MAPS_CLIENT_ID"],
    credential=DefaultAzureCredential(),
)
```

Use the `MAPS_CLIENT_ID` value from the Azure Maps account authentication section in the Azure portal. If you forget it, token acquisition may succeed while service requests still fail.

### SAS token

`2.0.0b2` added SAS-based authentication support. Use this when your deployment model already issues Azure Maps SAS tokens.

```bash
export AZURE_SAS_TOKEN="your-maps-sas-token"
```

```python
import os

from azure.core.credentials import AzureSASCredential
from azure.maps.search import MapsSearchClient

client = MapsSearchClient(
    credential=AzureSASCredential(os.environ["AZURE_SAS_TOKEN"]),
)
```

Generating the SAS token is a management-plane step. The official docs use `azure-mgmt-maps` plus `AzureMapsManagementClient.accounts.list_sas(...)` for that flow.

## Core Usage

### Geocode an address

The current preview line uses `get_geocoding` and returns a GeoJSON-like result with `features`.

```python
from azure.core.credentials import AzureKeyCredential
from azure.maps.search import MapsSearchClient

client = MapsSearchClient(credential=AzureKeyCredential("..."))

result = client.get_geocoding(query="15127 NE 24th Street, Redmond, WA 98052")

if result.get("features"):
    coordinates = result["features"][0]["geometry"]["coordinates"]
    longitude, latitude = coordinates
    print(longitude, latitude)
else:
    print("No results")
```

### Batch geocoding

`get_geocoding_batch` accepts a payload with `batchItems`. The `2.0.0b1` release notes say batch geocoding supports up to 100 queries in one request.

```python
from azure.core.credentials import AzureKeyCredential
from azure.maps.search import MapsSearchClient

client = MapsSearchClient(credential=AzureKeyCredential("..."))

result = client.get_geocoding_batch(
    {
        "batchItems": [
            {"query": "400 Broad St, Seattle, WA 98109"},
            {"query": "15127 NE 24th Street, Redmond, WA 98052"},
        ]
    }
)

for item in result.get("batchItems", []):
    if item.get("features"):
        longitude, latitude = item["features"][0]["geometry"]["coordinates"]
        print(longitude, latitude)
```

### Reverse geocoding

Coordinates are `[longitude, latitude]`, not `[latitude, longitude]`.

```python
from azure.core.credentials import AzureKeyCredential
from azure.maps.search import MapsSearchClient

client = MapsSearchClient(credential=AzureKeyCredential("..."))

result = client.get_reverse_geocoding(coordinates=[-122.138679, 47.630356])

if result.get("features"):
    address = result["features"][0]["properties"]["address"]["formattedAddress"]
    print(address)
```

### Reverse geocoding in batch

```python
from azure.core.credentials import AzureKeyCredential
from azure.maps.search import MapsSearchClient

client = MapsSearchClient(credential=AzureKeyCredential("..."))

result = client.get_reverse_geocoding_batch(
    {
        "batchItems": [
            {"coordinates": [-122.138679, 47.630356]},
            {"coordinates": [-122.126, 47.64]},
        ]
    }
)

for item in result.get("batchItems", []):
    if item.get("features"):
        print(item["features"][0]["properties"]["address"]["formattedAddress"])
```

### Polygon lookup

Use `get_polygon` when you need a locality or other boundary geometry for a coordinate pair.

```python
from azure.core.credentials import AzureKeyCredential
from azure.maps.search import BoundaryResultType, MapsSearchClient, Resolution

client = MapsSearchClient(credential=AzureKeyCredential("..."))

result = client.get_polygon(
    coordinates=[-122.204141, 47.61256],
    result_type=BoundaryResultType.LOCALITY,
    resolution=Resolution.SMALL,
)

geometry = result.get("geometry")
if geometry:
    print(geometry["type"])
```

## Error Handling And Diagnostics

Azure SDK failures are raised through Azure Core exceptions such as `HttpResponseError`.

```python
from azure.core.exceptions import HttpResponseError

try:
    result = client.get_geocoding(query="invalid input")
except HttpResponseError as exc:
    print(exc.error_code)
    raise
```

For request/response diagnostics, the package supports Azure SDK logging:

```python
result = client.get_geocoding(
    query="15127 NE 24th Street, Redmond, WA 98052",
    logging_enable=True,
)
```

Be careful with debug logging in production because Azure SDK logging can include request details you may not want in logs.

## Configuration Notes

- `MapsSearchClient` does not need an explicit endpoint in the common Azure public-cloud path; auth is the main configuration input.
- For Entra auth, keep the Azure app identity values and `MAPS_CLIENT_ID` separate. They are different IDs used for different purposes.
- For local development, prefer environment variables or a managed identity-aware workflow over hard-coded secrets.
- The package overview says async support exists, but you must install an async transport such as `aiohttp` and close async clients and credentials when finished.

## Common Pitfalls

- `pip install azure-maps-search` without `--pre` can fail to find a release because the latest published versions are prereleases.
- The Azure Maps how-to guide uses `SUBSCRIPTION_KEY` in one snippet and `AZURE_SUBSCRIPTION_KEY` in others. Standardize on `AZURE_SUBSCRIPTION_KEY` in your own code to match the package overview and avoid mixed env var names.
- Old blog posts and early preview examples may call removed methods such as `fuzzy_search`, `search_address`, or `reverse_search_address`. Those are not part of the `2.x` preview line.
- Reverse geocoding and polygon calls expect `[longitude, latitude]`. Swapping the order is an easy mistake.
- Batch methods take request bodies with `batchItems`; they are not simple lists of strings or coordinate arrays.
- Entra auth also needs the Azure Maps account client ID. Missing `MAPS_CLIENT_ID` is a common setup bug.
- SAS auth exists only in `2.0.0b2+`. Do not assume older preview environments support it.

## Version-Sensitive Notes

- PyPI currently lists `2.0.0b2` as the latest release, published on December 12, 2024, with development status `Beta`.
- `2.0.0b2` added SAS authentication support.
- `2.0.0b1` introduced the current `get_geocoding`, `get_geocoding_batch`, `get_reverse_geocoding`, `get_reverse_geocoding_batch`, and `get_polygon` methods.
- `2.0.0b1` also removed the older search method family, including `fuzzy_search`, `search_point_of_interest`, `search_address`, and related batch/reverse variants.
- Use official preview docs and the current PyPI release history instead of generic Azure Maps Python search examples from before August 2024.

## Official Sources

- Microsoft Learn package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/maps-search-readme?view=azure-python-preview`
- Microsoft Learn API root: `https://learn.microsoft.com/en-us/python/api/azure-maps-search/`
- Azure Maps Python SDK how-to: `https://learn.microsoft.com/en-us/azure/azure-maps/how-to-dev-guide-py-sdk`
- PyPI project page: `https://pypi.org/project/azure-maps-search/`

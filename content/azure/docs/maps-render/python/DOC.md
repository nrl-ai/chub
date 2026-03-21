---
name: maps-render
description: "Azure Maps Render Python client library for tiles, static images, attribution, and Azure Maps authentication"
metadata:
  languages: "python"
  versions: "2.0.0b2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-maps,maps,render,geospatial,tiles,static-maps"
---

# Azure Maps Render Python Client Library

## Golden Rule

Use `azure-maps-render` with `MapsRenderClient`, pin the exact prerelease you expect, and treat Microsoft Learn's preview docs as the source of truth for method names and auth requirements. For Microsoft Entra ID auth, you need both a token credential and the Azure Maps account `client_id`.

## Install

Pin the prerelease explicitly:

```bash
python -m pip install "azure-maps-render==2.0.0b2"
```

If you want the latest prerelease in this line instead of an exact pin:

```bash
python -m pip install --pre azure-maps-render
```

Common companion packages:

```bash
python -m pip install azure-identity
python -m pip install azure-mgmt-maps
python -m pip install aiohttp
```

- `azure-identity` is for Microsoft Entra ID auth.
- `azure-mgmt-maps` is needed if you want to generate SAS tokens programmatically.
- `aiohttp` is a common async transport for `azure.maps.render.aio`.

## Azure Maps Setup

You need:

1. An Azure subscription
2. An Azure Maps account
3. A deployed Maps resource
4. One of these auth modes: subscription key, SAS token, or Microsoft Entra ID

Azure CLI example from the package overview:

```bash
az maps account create \
  --resource-group <resource-group-name> \
  --account-name <account-name> \
  --sku <sku-name>
```

## Authentication And Client Initialization

The constructor is credential-first:

```python
MapsRenderClient(credential=..., **kwargs)
```

Unlike many Azure SDK clients, the common `MapsRenderClient` examples do not take a service endpoint.

### Subscription Key

```bash
export AZURE_SUBSCRIPTION_KEY="..."
```

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.render import MapsRenderClient

client = MapsRenderClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)
```

### Microsoft Entra ID

For token auth, pass the Azure Maps account `client_id` on the client in addition to the credential:

```bash
export AZURE_CLIENT_ID="app-or-managed-identity-client-id"
export AZURE_TENANT_ID="tenant-id"
export AZURE_CLIENT_SECRET="client-secret"
export AZURE_MAPS_CLIENT_ID="azure-maps-account-client-id"
```

```python
import os

from azure.identity import DefaultAzureCredential
from azure.maps.render import MapsRenderClient

client = MapsRenderClient(
    credential=DefaultAzureCredential(),
    client_id=os.environ["AZURE_MAPS_CLIENT_ID"],
)
```

The `AZURE_MAPS_CLIENT_ID` value is the Azure Maps resource client ID from the Maps account authentication settings, not just your app registration ID.

### SAS Token

The overview doc says to generate SAS tokens with `azure-mgmt-maps`, then pass the token to the render client:

```bash
export AZURE_SAS_TOKEN="..."
```

```python
import os

from azure.core.credentials import AzureSasCredential
from azure.maps.render import MapsRenderClient

client = MapsRenderClient(
    credential=AzureSasCredential(os.environ["AZURE_SAS_TOKEN"])
)
```

## Core Usage

### Get a map tile

`get_map_tile()` returns `Iterator[bytes]`. Save or forward the joined bytes.

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.render import MapsRenderClient, TilesetID

client = MapsRenderClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)

tile_bytes = b"".join(
    client.get_map_tile(
        tileset_id=TilesetID.MICROSOFT_BASE,
        z=6,
        x=9,
        y=22,
        tile_size="512",
    )
)

with open("tile.png", "wb") as f:
    f.write(tile_bytes)
```

### Get a static image

The static image API also returns `Iterator[bytes]`. In `2.0.0b2`, the SDK parameter is still named `bounding_box_private`.

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.render import MapsRenderClient

client = MapsRenderClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)

image_bytes = b"".join(
    client.get_map_static_image(
        zoom=10,
        bounding_box_private=[13.228, 52.4559, 13.5794, 52.629],
    )
)

with open("static-map.png", "wb") as f:
    f.write(image_bytes)
```

Either `center` or `bounding_box_private` must be supplied.

### Get tileset metadata

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.render import MapsRenderClient, TilesetID

client = MapsRenderClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)

tileset = client.get_map_tileset(tileset_id=TilesetID.MICROSOFT_BASE)
print(tileset)
```

### Get attribution

Use attribution when you need display-safe copyright data for a specific map section:

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.render import MapsRenderClient, TilesetID

client = MapsRenderClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)

attribution = client.get_map_attribution(
    tileset_id=TilesetID.MICROSOFT_BASE,
    zoom=6,
    bounds=[42.982261, 24.980233, 56.526017, 1.355233],
)

print(attribution)
```

The `bounds` order is southwest longitude/latitude followed by northeast longitude/latitude.

### Get copyright information for the world

```python
import os

from azure.core.credentials import AzureKeyCredential
from azure.maps.render import MapsRenderClient

client = MapsRenderClient(
    credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
)

result = client.get_copyright_for_world()
print(result)
```

## Async Usage

Install an async transport first, then use the `aio` namespace and close the client with `async with`.

```python
import os
import asyncio

from azure.core.credentials import AzureKeyCredential
from azure.maps.render.aio import MapsRenderClient

async def main() -> None:
    async with MapsRenderClient(
        credential=AzureKeyCredential(os.environ["AZURE_SUBSCRIPTION_KEY"])
    ) as client:
        tileset = await client.get_map_tileset(tileset_id="microsoft.base")
        print(tileset)

asyncio.run(main())
```

## Configuration Notes

- `get_map_tile()` uses `z` for zoom, not `zoom`.
- `get_map_attribution()` uses `zoom`, not `z`.
- `tile_size` accepts `"256"` or `"512"`.
- `get_map_tile()` also accepts `time_stamp` for time-based weather tiles.
- `language` and `localized_map_view` are per-request options on render methods.
- `get_map_static_image()` defaults `tileset_id` to a road style when omitted, but setting it explicitly is safer in agent-generated code.

## Common Pitfalls

- Do not assume this package is stable: it is still a beta line on PyPI and Microsoft Learn serves it under `azure-python-preview`.
- Do not pass an endpoint unless you have a specific documented need; the normal constructor shape is credential plus keyword options.
- Do not forget the Azure Maps account `client_id` when using Microsoft Entra ID credentials.
- Do not confuse the official docs' inconsistent SAS class casing. The API reference uses `AzureSasCredential`, which is the class name you should import from `azure.core.credentials`.
- Do not mix older Azure Maps Render v1 REST examples into this SDK workflow. Azure product docs note that Render v1 is deprecated and retires on September 17, 2026.
- Catch `azure.core.exceptions.HttpResponseError` for request failures and inspect `error_code` when you need service-specific handling.
- `logging_enable=True` can expose request details; use it for debugging, not by default in production code.

## Version-Sensitive Notes

- PyPI and the Azure SDK package index both list `2.0.0b2` as the current published `azure-maps-render` package.
- The package overview shows common samples with subscription key, SAS token, and Microsoft Entra ID; if your code uses older Azure Maps auth guidance, re-check those flows against the current preview docs.
- Product docs now flag some coverage entries as available only in Render v2 and API version `v2024-04-01`. If you are working from older map-render examples, verify the tileset and service version assumptions before copying them.

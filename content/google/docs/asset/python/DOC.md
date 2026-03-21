---
name: asset
description: "google-cloud-asset package guide for Python with ADC setup, asset inventory search/list patterns, and export workflows"
metadata:
  languages: "python"
  versions: "4.2.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,gcp,cloud-asset-inventory,python,iam,inventory,search,export"
---

# google-cloud-asset Python Package Guide

## What It Is

`google-cloud-asset` is the official Python client for Google Cloud Asset Inventory.

Use it when code needs to:

- inventory resources under a project, folder, or organization
- search resources or IAM policies across a scope
- export snapshots to Cloud Storage or BigQuery
- analyze IAM access relationships

For Python code, import from `google.cloud.asset_v1`.

## Install

```bash
python -m pip install "google-cloud-asset==4.2.0"
```

If you manage dependencies with `uv` or Poetry, pin the same version there.

## Authentication And Setup

The client library uses Application Default Credentials by default. For local development, the standard flow is:

```bash
gcloud auth application-default login
```

Before calling the API, make sure:

- Cloud Asset API is enabled in the target project
- the caller has access to the project, folder, or organization being queried
- the scope strings you pass use full resource names such as `projects/my-project`, `folders/1234567890`, or `organizations/1234567890`

## Initialize A Client

```python
from google.cloud import asset_v1

client = asset_v1.AssetServiceClient()
```

If you need a custom endpoint or mTLS behavior, pass `client_options`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import asset_v1

client = asset_v1.AssetServiceClient(
    client_options=ClientOptions(api_endpoint="cloudasset.googleapis.com"),
)
```

The generated client also accepts explicit `credentials=` and `transport=` arguments when ADC is not the right fit.

## Core Usage

### List Assets For An Inventory Snapshot

Use `list_assets` when you want a structured inventory for a project, folder, or organization.

```python
from google.cloud import asset_v1

client = asset_v1.AssetServiceClient()

request = asset_v1.ListAssetsRequest(
    parent="projects/my-project-id",
    asset_types=["storage.googleapis.com/Bucket"],
    content_type=asset_v1.ContentType.RESOURCE,
    page_size=100,
)

pager = client.list_assets(request=request)

for asset in pager:
    print(asset.name)
    print(asset.asset_type)
```

Use `read_time` when you need a point-in-time snapshot instead of the latest view.

### Search Resources Across A Scope

Use `search_all_resources` when you know roughly what you want and need filtered lookup instead of a full inventory walk.

```python
from google.cloud import asset_v1

client = asset_v1.AssetServiceClient()

request = asset_v1.SearchAllResourcesRequest(
    scope="projects/my-project-id",
    query='displayName:prod',
    asset_types=["compute.googleapis.com/Instance"],
    page_size=50,
)

pager = client.search_all_resources(request=request)

for resource in pager:
    print(resource.name)
    print(resource.asset_type)
```

Use `search_all_iam_policies` when the task is policy-oriented rather than resource-oriented.

### Export Large Inventories

Use `export_assets` for scheduled reporting or large inventories. It is a long-running operation.

```python
from google.cloud import asset_v1

client = asset_v1.AssetServiceClient()

request = asset_v1.ExportAssetsRequest(
    parent="projects/my-project-id",
    asset_types=["storage.googleapis.com/Bucket"],
    output_config={
        "gcs_destination": {
            "uri": "gs://my-bucket/asset-export.json",
        }
    },
)

operation = client.export_assets(request=request)
result = operation.result(timeout=300)

print(result.read_time)
```

Use a BigQuery destination when downstream code will query or join asset data instead of reading raw JSON snapshots.

### Async Client

If the application is already async, use `AssetServiceAsyncClient` directly.

```python
from google.cloud import asset_v1

client = asset_v1.AssetServiceAsyncClient()

async def main() -> None:
    request = asset_v1.SearchAllResourcesRequest(
        scope="projects/my-project-id",
        asset_types=["compute.googleapis.com/Instance"],
    )
    pager = client.search_all_resources(request=request)
    async for resource in pager:
        print(resource.name)
```

## Config And Auth Notes

- Prefer ADC in local development and attached service accounts or Workload Identity in deployed environments.
- `parent` and `scope` are not plain project IDs. They must be fully qualified resource names.
- Keep the queried scope as narrow as possible. Organization-wide searches are slower and need broader IAM.
- The generated GAPIC methods accept either a `request=` object or flattened fields. If you pass `request=...`, do not also pass method-specific flattened arguments.

## Common Pitfalls

### Empty Or Partial Results Usually Mean IAM Or Scope Problems

`search_all_resources` and `list_assets` only return what the caller can see. If results look incomplete, verify:

- the resource scope string
- whether the API is enabled
- whether the caller has the relevant Cloud Asset Inventory permissions for that scope

### `list_assets` Is Not The Best Tool For Very Large Dumps

The official reference warns that `list_assets` might not meet performance requirements for large-scale exports. Use `export_assets` when you need broad snapshots.

### Rolling Reference Docs Can Lag The Exact Package Version

PyPI shows `4.2.0` as the package release, but the rolling Google Cloud Python reference pages visited on March 12, 2026 still surfaced `4.1.0` in page chrome. Treat the reference pages as the canonical API family docs, but verify installed-client behavior if you are debugging a version-pinned environment.

### Prefer Resource Search For Targeted Queries

If you only need specific resources, `search_all_resources` is usually simpler than scanning `list_assets` and filtering client-side.

## Official Source URLs

- PyPI package page: `https://pypi.org/project/google-cloud-asset/4.2.0/`
- PyPI release JSON: `https://pypi.org/pypi/google-cloud-asset/4.2.0/json`
- Package docs URL: `https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-asset`
- Google Cloud Python reference root: `https://cloud.google.com/python/docs/reference/cloudasset/latest`
- `AssetServiceClient` reference: `https://cloud.google.com/python/docs/reference/cloudasset/latest/google.cloud.asset_v1.services.asset_service.AssetServiceClient`
- `ListAssets` reference: `https://cloud.google.com/python/docs/reference/cloudasset/latest/google.cloud.asset_v1.services.asset_service.AssetServiceClient#google_cloud_asset_v1_services_asset_service_AssetServiceClient_list_assets`
- `SearchAllResources` reference: `https://cloud.google.com/python/docs/reference/cloudasset/latest/google.cloud.asset_v1.services.asset_service.AssetServiceClient#google_cloud_asset_v1_services_asset_service_AssetServiceClient_search_all_resources`
- `ExportAssets` reference: `https://cloud.google.com/python/docs/reference/cloudasset/latest/google.cloud.asset_v1.services.asset_service.AssetServiceClient#google_cloud_asset_v1_services_asset_service_AssetServiceClient_export_assets`
- ADC guide: `https://cloud.google.com/docs/authentication/provide-credentials-adc`

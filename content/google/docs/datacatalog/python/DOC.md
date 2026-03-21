---
name: datacatalog
description: "google-cloud-datacatalog package guide for Python legacy Data Catalog integrations and migration work"
metadata:
  languages: "python"
  versions: "3.29.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,google-cloud-datacatalog,data-catalog,metadata,governance"
---

# google-cloud-datacatalog Python Package Guide

## Status

`google-cloud-datacatalog` is the Google-maintained Python client for Google Cloud Data Catalog.

Use this package only for legacy maintenance or migration work. Google Cloud's generated client reference marks `DataCatalogClient` as deprecated in favor of Dataplex Catalog, and the Data Catalog release notes announced a shutdown date of January 30, 2026. As of March 12, 2026, the Python package `3.29.0` and its reference docs are still published, but new integrations should target Dataplex Catalog unless you have a specific legacy requirement.

## Install

```bash
pip install google-cloud-datacatalog==3.29.0
```

```bash
uv add google-cloud-datacatalog==3.29.0
```

```bash
poetry add google-cloud-datacatalog==3.29.0
```

## Authentication And Setup

The client uses Application Default Credentials (ADC). For local development, the usual order is:

1. `GOOGLE_APPLICATION_CREDENTIALS` pointing at a service-account key JSON file.
2. `gcloud auth application-default login` for user credentials.
3. Attached service-account credentials when running on Google Cloud.

```bash
export GOOGLE_CLOUD_PROJECT="my-project"
export GOOGLE_APPLICATION_CREDENTIALS="/abs/path/service-account.json"
```

```python
from google.cloud import datacatalog_v1

client = datacatalog_v1.DataCatalogClient()
```

Notes:

- Prefer ADC or an explicit `credentials=` object. The library changelog deprecates the older `credentials_file` client argument in recent releases.
- Keep the Data Catalog API enabled in the target project if you are maintaining an existing deployment.
- If you are migrating away from Data Catalog, validate service availability first instead of assuming the legacy API is still enabled everywhere.

## Core Client Types

```python
from google.cloud import datacatalog_v1

catalog_client = datacatalog_v1.DataCatalogClient()
catalog_async_client = datacatalog_v1.DataCatalogAsyncClient()
policy_tag_client = datacatalog_v1.PolicyTagManagerClient()
serialization_client = datacatalog_v1.PolicyTagManagerSerializationClient()
```

Use them for different surfaces:

- `DataCatalogClient`: entries, entry groups, tag templates, tags, search, lookup.
- `PolicyTagManagerClient`: taxonomies and policy tags for column-level governance.
- `PolicyTagManagerSerializationClient`: import or export entire taxonomies.

Policy tags are not managed through `DataCatalogClient`, which is a common source of agent mistakes.

## Common Usage

### Search Catalog Assets

`search_catalog` is the main discovery call. It returns a pager.

```python
from google.cloud import datacatalog_v1

project_id = "my-project"

client = datacatalog_v1.DataCatalogClient()
scope = datacatalog_v1.SearchCatalogRequest.Scope()
scope.include_project_ids.append(project_id)

request = datacatalog_v1.SearchCatalogRequest(
    scope=scope,
    query="type=entry name:orders",
    page_size=20,
)

for result in client.search_catalog(request=request):
    print(result.relative_resource_name)
    print(result.linked_resource)
    print(result.search_result_subtype)
```

Use search when you know keywords, asset type, columns, tags, or business terms but do not already have the entry name.

### Look Up A Known Asset

If you already know the underlying Google Cloud resource, `lookup_entry` is usually more reliable than search.

```python
from google.cloud import datacatalog_v1

project_id = "my-project"
dataset_id = "analytics"
table_id = "orders"

client = datacatalog_v1.DataCatalogClient()
linked_resource = (
    f"//bigquery.googleapis.com/projects/{project_id}"
    f"/datasets/{dataset_id}/tables/{table_id}"
)

entry = client.lookup_entry(
    request=datacatalog_v1.LookupEntryRequest(
        linked_resource=linked_resource,
    )
)

print(entry.name)
print(entry.type_)
print(entry.fully_qualified_name)
```

Use `lookup_entry` when you have one of the supported identifiers:

- `linked_resource` for a concrete resource URI such as BigQuery or Pub/Sub.
- `sql_resource` for SQL resources.
- `fully_qualified_name` for supported systems.

Only set one of those fields in the request.

### Read An Entry And Its Tags

Search results are intentionally lightweight. Fetch the entry when you need the full record, then list tags separately.

```python
from google.cloud import datacatalog_v1

client = datacatalog_v1.DataCatalogClient()
entry_name = "projects/my-project/locations/us/entryGroups/@bigquery/entries/orders"

entry = client.get_entry(request={"name": entry_name})
print(entry.display_name)
print(entry.schema.columns)

for tag in client.list_tags(request={"parent": entry.name}):
    print(tag.name, tag.template)
```

### Attach A Tag From An Existing Tag Template

This is the common metadata-enrichment flow after you already created the tag template in the same project and organization.

```python
from google.cloud import datacatalog_v1

project_id = "my-project"
location = "us-central1"
tag_template_id = "governance_template"
entry_name = "projects/my-project/locations/us/entryGroups/@bigquery/entries/orders"

client = datacatalog_v1.DataCatalogClient()

tag = datacatalog_v1.Tag()
tag.template = client.tag_template_path(project_id, location, tag_template_id)
tag.fields["owner"].string_value = "data-platform"
tag.fields["contains_pii"].bool_value = True

created = client.create_tag(
    request=datacatalog_v1.CreateTagRequest(
        parent=entry_name,
        tag=tag,
    )
)

print(created.name)
```

Important constraint: the entry and the tag template must belong to the same organization.

## Configuration Notes

### Credentials

If you need explicit credentials instead of ADC, pass a credentials object:

```python
from google.cloud import datacatalog_v1
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/abs/path/service-account.json"
)

client = datacatalog_v1.DataCatalogClient(credentials=credentials)
```

### Endpoints And Client Options

Most legacy integrations use the default endpoint and do not need custom transport settings.

If you must override the endpoint:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import datacatalog_v1

client = datacatalog_v1.DataCatalogClient(
    client_options=ClientOptions(api_endpoint="datacatalog.googleapis.com")
)
```

Only override `api_endpoint` when the product documentation for your exact workload requires it.

## Common Pitfalls

- The import name is `datacatalog_v1`, not `google_cloud_datacatalog`.
- Treat Data Catalog as a legacy surface. For new metadata catalog work, use Dataplex Catalog.
- `search_catalog` does not guarantee full recall. Use it for discovery, then follow up with `get_entry`, `lookup_entry`, or targeted reads.
- `lookup_entry` uses a oneof request shape. Set only one of `linked_resource`, `sql_resource`, or `fully_qualified_name`.
- The official quickstart and tag-template samples often use `us-central1`. Do not hard-code that location unless your resources or templates actually live there.
- `create_tag` requires the tag template and the target entry to be in the same organization.
- Policy tags use `PolicyTagManagerClient`, not `DataCatalogClient`.
- If you copied older client-construction examples that pass `credentials_file=...`, update them to ADC or explicit `credentials=...`.

## Version-Sensitive Notes

- PyPI lists `3.29.0` as the current package version for `google-cloud-datacatalog`.
- The published changelog for recent releases shows Python 3.14 support added in `3.28.0`.
- The same changelog deprecates the `credentials_file` argument in favor of `credentials`.
- The `3.26.0` changelog explicitly marks the Data Catalog service as deprecated.

## Official Sources

- PyPI package page: `https://pypi.org/project/google-cloud-datacatalog/`
- PyPI metadata JSON: `https://pypi.org/pypi/google-cloud-datacatalog/json`
- Python client reference root: `https://cloud.google.com/python/docs/reference/datacatalog/latest`
- `DataCatalogClient` reference: `https://cloud.google.com/python/docs/reference/datacatalog/latest/google.cloud.datacatalog_v1.services.data_catalog.DataCatalogClient`
- Library changelog: `https://cloud.google.com/python/docs/reference/datacatalog/latest/changelog`
- Data Catalog search sample: `https://cloud.google.com/data-catalog/docs/samples/data-catalog-search-assets`
- Data Catalog quickstart sample: `https://cloud.google.com/data-catalog/docs/samples/data-catalog-quickstart`
- ADC guide: `https://cloud.google.com/docs/authentication/application-default-credentials`
- Data Catalog release notes: `https://cloud.google.com/data-catalog/docs/release-notes`

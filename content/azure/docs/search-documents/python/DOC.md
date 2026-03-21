---
name: search-documents
description: "Azure AI Search Python SDK for index schema, document indexing, querying, authentication, and stable 11.6.0 usage notes"
metadata:
  languages: "python"
  versions: "11.6.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-ai-search,search,indexing,query,rbac,python"
---

# azure-search-documents Python Package Guide

## What This Package Is

`azure-search-documents` is the official Azure SDK package for Azure AI Search data-plane work in Python.

Use it when your code needs to:

- create and manage search indexes
- upload, merge, delete, and fetch indexed documents
- run keyword, filtered, semantic, vector, and hybrid searches against an existing index
- manage indexers, data sources, synonym maps, and skillsets

Primary client types:

- `SearchClient`: query and mutate documents in one existing index
- `SearchIndexClient`: create, update, list, and delete indexes and related schema objects
- `SearchIndexerClient`: manage pull-based indexing resources such as data sources, skillsets, and indexers

Package and import path differ:

- PyPI package: `azure-search-documents`
- Python namespace: `azure.search.documents`

This guide is scoped to package version `11.6.0` on PyPI.

## Golden Rules

- Use the service endpoint root, not an index URL: `https://<service-name>.search.windows.net`
- Use a query key only for read-only client-side queries; use an admin key or RBAC for writes and schema changes
- `upload_documents()` replaces the stored document when the key already exists
- `merge_documents()` only updates named fields and fails if the document does not already exist
- `merge_or_upload_documents()` is the safest default for idempotent sync jobs
- Search document keys are case-sensitive, must be unique, and must be a top-level string field in the index
- Many Learn quickstarts now show prerelease or preview-oriented examples; verify that a sample matches stable `11.6.0` before copying it

## Install

Python `3.8+` is required upstream.

```bash
python -m pip install "azure-search-documents==11.6.0"
```

For Microsoft Entra ID authentication:

```bash
python -m pip install "azure-search-documents==11.6.0" azure-identity
```

For async clients:

```bash
python -m pip install "azure-search-documents==11.6.0" aiohttp
```

## Service Setup And Environment

Typical environment variables:

```bash
export AZURE_SEARCH_SERVICE_ENDPOINT="https://<service-name>.search.windows.net"
export AZURE_SEARCH_INDEX_NAME="hotels"
export AZURE_SEARCH_API_KEY="<admin-or-query-key>"
```

For service principal authentication, `DefaultAzureCredential` also uses the standard Azure Identity variables:

```bash
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_ID="<client-id>"
export AZURE_CLIENT_SECRET="<client-secret>"
```

Docs and samples use both `SEARCH_*` and `AZURE_SEARCH_*` environment variable names. Pick one convention in your codebase and keep it consistent.

## Authentication

### API Key

Use API keys when you want the simplest bootstrap path.

- query key: read-only queries from client apps
- admin key: index creation, document writes, indexer management, and other mutations

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.search.documents import SearchClient

client = SearchClient(
    endpoint=os.environ["AZURE_SEARCH_SERVICE_ENDPOINT"],
    index_name=os.environ["AZURE_SEARCH_INDEX_NAME"],
    credential=AzureKeyCredential(os.environ["AZURE_SEARCH_API_KEY"]),
)
```

### Microsoft Entra ID

Use `DefaultAzureCredential` for local development, managed identity, or service principals.

```python
import os
from azure.identity import DefaultAzureCredential
from azure.search.documents import SearchClient

client = SearchClient(
    endpoint=os.environ["AZURE_SEARCH_SERVICE_ENDPOINT"],
    index_name=os.environ["AZURE_SEARCH_INDEX_NAME"],
    credential=DefaultAzureCredential(),
)
```

Role mapping that matters in practice:

- query only: `Search Index Data Reader`
- document uploads and updates: `Search Index Data Contributor`
- schema and index management: `Search Service Contributor`
- end-to-end quickstart flows that create, load, and query often need all three roles on the same principal

For local development, `az login` is usually enough for `DefaultAzureCredential`.

### National Clouds

For sovereign clouds, set the Azure Identity authority host and pass `audience=` to the client.

```python
import os
from azure.identity import AzureAuthorityHosts, DefaultAzureCredential
from azure.search.documents import SearchClient

credential = DefaultAzureCredential(authority=AzureAuthorityHosts.AZURE_CHINA)

client = SearchClient(
    endpoint=os.environ["AZURE_SEARCH_SERVICE_ENDPOINT"],
    index_name=os.environ["AZURE_SEARCH_INDEX_NAME"],
    credential=credential,
    audience="https://search.azure.cn",
)
```

## Core Index Setup

This is the minimum useful schema flow for agents: define fields, create an index, then reuse `SearchClient` for writes and queries.

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.search.documents.indexes import SearchIndexClient
from azure.search.documents.indexes.models import (
    SearchFieldDataType,
    SearchIndex,
    SearchableField,
    SimpleField,
)

endpoint = os.environ["AZURE_SEARCH_SERVICE_ENDPOINT"]
credential = AzureKeyCredential(os.environ["AZURE_SEARCH_API_KEY"])
index_name = "hotels"

index_client = SearchIndexClient(endpoint=endpoint, credential=credential)

fields = [
    SimpleField(name="hotelId", type=SearchFieldDataType.String, key=True),
    SearchableField(name="hotelName", type=SearchFieldDataType.String),
    SearchableField(name="description", type=SearchFieldDataType.String),
    SimpleField(name="rating", type=SearchFieldDataType.Double, filterable=True, sortable=True),
]

index = SearchIndex(name=index_name, fields=fields)
index_client.create_index(index)
```

Notes:

- the key field must be a top-level string field
- field names are part of your query contract; keep them stable once clients depend on them
- if you rerun setup against an existing index name, you need an explicit update or replacement path instead of another `create_index()` call

## Upload, Merge, And Delete Documents

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.search.documents import SearchClient

endpoint = os.environ["AZURE_SEARCH_SERVICE_ENDPOINT"]
index_name = os.environ["AZURE_SEARCH_INDEX_NAME"]

search_client = SearchClient(
    endpoint=endpoint,
    index_name=index_name,
    credential=AzureKeyCredential(os.environ["AZURE_SEARCH_API_KEY"]),
)

documents = [
    {
        "hotelId": "1",
        "hotelName": "Secret Point Motel",
        "description": "Near the airport with a restaurant",
        "rating": 4.2,
    },
    {
        "hotelId": "2",
        "hotelName": "Twin Dome Motel",
        "description": "Walkable to downtown shops",
        "rating": 3.9,
    },
]

results = search_client.upload_documents(documents=documents)
for item in results:
    print(item.key, item.succeeded)
```

Update one existing document without replacing the whole payload:

```python
update_results = search_client.merge_documents(
    documents=[
        {
            "hotelId": "1",
            "rating": 4.5,
        }
    ]
)
```

Safer upsert pattern for sync jobs:

```python
upsert_results = search_client.merge_or_upload_documents(
    documents=[
        {
            "hotelId": "3",
            "hotelName": "City Lights Hotel",
            "description": "Late check-in and strong Wi-Fi",
            "rating": 4.7,
        }
    ]
)
```

Delete is keyed by the document key. Other fields are ignored.

```python
delete_results = search_client.delete_documents(
    documents=[{"hotelId": "2"}]
)
```

Behavior to remember:

- `upload_documents()` inserts or fully replaces the document with that key
- `merge_documents()` updates only named fields, but requires the document to exist
- `merge_or_upload_documents()` acts like merge if the key exists and upload if it does not
- `delete_documents()` is idempotent and ignores non-key fields in the delete payload

## Query Documents

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.search.documents import SearchClient

endpoint = os.environ["AZURE_SEARCH_SERVICE_ENDPOINT"]
index_name = os.environ["AZURE_SEARCH_INDEX_NAME"]

client = SearchClient(
    endpoint=endpoint,
    index_name=index_name,
    credential=AzureKeyCredential(os.environ["AZURE_SEARCH_API_KEY"]),
)

results = client.search(
    search_text="restaurant",
    filter="rating ge 4",
    select=["hotelId", "hotelName", "rating"],
    top=5,
)

for doc in results:
    print(doc["hotelId"], doc["hotelName"], doc["rating"])
```

Useful search patterns:

- `search_text="*"` returns a match-all result set
- `filter=` uses OData-style filter syntax on filterable fields
- `select=` reduces payload size and latency
- `search_fields=` limits which searchable fields are queried
- `get_document(key=...)` is the direct lookup path when you already know the key
- `autocomplete()` and `suggest()` require suggester-aware index configuration

The `search()` signature in current docs includes optional parameters for semantic answers/captions and `vector_queries`. That surface exists in stable `11.6.0`, but whether a given query actually works depends on your service configuration and supported Search API features.

## Async Usage

The package ships an async API under `.aio`.

```python
import asyncio
import os
from azure.core.credentials import AzureKeyCredential
from azure.search.documents.aio import SearchClient

async def main() -> None:
    client = SearchClient(
        endpoint=os.environ["AZURE_SEARCH_SERVICE_ENDPOINT"],
        index_name=os.environ["AZURE_SEARCH_INDEX_NAME"],
        credential=AzureKeyCredential(os.environ["AZURE_SEARCH_API_KEY"]),
    )

    async with client:
        results = await client.search(search_text="spa")
        async for doc in results:
            print(doc["hotelName"])

asyncio.run(main())
```

If async imports work but requests fail immediately, the usual cause is missing `aiohttp`.

## SearchIndexerClient When You Need Pull-Based Ingestion

Use `SearchIndexerClient` when Azure AI Search should pull from a data source instead of your app pushing JSON documents directly.

This client is the right surface for:

- data source connections
- indexers
- skillsets
- synonym maps

If your task is "load app-owned JSON records into an existing index", start with `SearchClient` instead. If your task is "crawl blob storage / SQL / Cosmos and enrich content", reach for `SearchIndexerClient`.

## Version-Sensitive Notes For `11.6.0`

- `11.6.0` is the current stable PyPI release in the official registry entry.
- Microsoft Learn quickstarts can point to prerelease package pins such as `11.6.0b1` even when the current stable package is newer. Treat those tutorials as product guidance, but check whether they use preview-only SDK shapes before copying code into a project pinned to stable `11.6.0`.
- Microsoft Learn search results also expose preview namespaces such as `azure.search.documents.knowledgebases.*`. Do not assume those preview surfaces are safe defaults for stable production code.
- The SDK documentation still warns about confusion with the retired `Microsoft.Azure.Search` v10 client library. Ignore old blog posts or samples that use the retired namespace.
- Client constructors expose `api_version=` and, for Entra ID scenarios, `audience=`. Only override these when the feature you need is documented for the target service API version or cloud.
- Features like semantic ranking, vector search, and hybrid queries depend on both SDK support and service-side configuration. The method parameters being present in Python does not mean your index or service is already configured for them.

## Common Pitfalls

- Mixing package names and import paths: install `azure-search-documents`, import `azure.search.documents`
- Using a query key for uploads or index creation: it will fail with authorization errors
- Using the wrong endpoint: pass the service root, not `/indexes/<name>` or a portal URL
- Assuming `upload_documents()` is partial update: it is full replacement for an existing key
- Forgetting to check per-document indexing results in bulk operations
- Copying preview examples from Learn without checking whether they rely on prerelease packages or preview service features
- Forgetting that document keys are case-sensitive

## Official URLs

- Docs root: https://learn.microsoft.com/en-us/python/api/azure-search-documents/
- PyPI package: https://pypi.org/project/azure-search-documents/
- Azure SDK README: https://github.com/Azure/azure-sdk-for-python/tree/main/sdk/search/azure-search-documents
- Product docs: https://learn.microsoft.com/en-us/azure/search/

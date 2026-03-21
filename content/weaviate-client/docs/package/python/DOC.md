---
name: package
description: "weaviate-client for Python - official Weaviate client for connecting, defining collections, ingesting data, and running vector search"
metadata:
  languages: "python"
  versions: "4.20.4"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "weaviate,weaviate-client,vector-database,semantic-search,rag,embeddings"
---

# weaviate-client Python Package Guide

## What It Is

`weaviate-client` is the official Python client for Weaviate's v4 API. Use it to connect to a local, custom, embedded, or Weaviate Cloud instance, define collections, insert data, and run vector, keyword, hybrid, and generative queries.

For this entry, the package version is pinned to `4.20.4` from PyPI. The main Weaviate Python client docs currently describe the `4.20.x` line and, at the time of writing, the docs page header still says `v4.20.3`; the API examples below are written to match the documented `4.20.x` surface.

## Install

`4.20.4` requires Python `>=3.10` according to PyPI metadata.

```bash
pip install weaviate-client==4.20.4
```

If you use `uv`:

```bash
uv add weaviate-client==4.20.4
```

If you need the optional Agents extra published on PyPI:

```bash
pip install "weaviate-client[agents]==4.20.4"
```

## Version And Server Compatibility

- Use the v4 client, not the deprecated v3 API.
- The v4 Python client requires Weaviate `1.23.7+`.
- Weaviate's current compatibility table maps Python client `4.20.x` to Weaviate Database `1.36.x`.
- The client uses gRPC for many operations. For self-hosted deployments, expose both the HTTP port and the gRPC port.

For local Docker setups, `8080` and `50051` are the usual defaults.

## Recommended Imports

Use the top-level `weaviate` module for connection helpers, and `weaviate.classes` for typed config/query helpers.

```python
import weaviate
import weaviate.classes as wvc
from weaviate.classes.init import AdditionalConfig, Auth, Timeout
```

`import weaviate.classes as wvc` is the recommended style for helper classes. Older patterns that import many classes directly from `weaviate` are deprecated.

## Connection Setup

### Weaviate Cloud

Use API key auth for WCD. OIDC password auth is deprecated in the docs and should not be your default.

```python
import os
import weaviate
from weaviate.classes.init import AdditionalConfig, Auth, Timeout

client = weaviate.connect_to_weaviate_cloud(
    cluster_url=os.environ["WEAVIATE_URL"],
    auth_credentials=Auth.api_key(os.environ["WEAVIATE_API_KEY"]),
    headers={
        "X-OpenAI-Api-Key": os.environ["OPENAI_API_KEY"],
    },
    additional_config=AdditionalConfig(
        timeout=Timeout(init=30, query=60, insert=120)
    ),
)

try:
    if not client.is_ready():
        raise RuntimeError("Weaviate is not ready")
finally:
    client.close()
```

### Local Weaviate

```python
import os
import weaviate

client = weaviate.connect_to_local(
    host="localhost",
    port=8080,
    grpc_port=50051,
    headers={
        "X-OpenAI-Api-Key": os.getenv("OPENAI_API_KEY", ""),
    },
)

try:
    print(client.is_ready())
finally:
    client.close()
```

### Custom Endpoints

Use `connect_to_custom()` when the HTTP and gRPC hosts or ports differ from the defaults:

```python
import os
import weaviate
from weaviate.classes.init import AdditionalConfig, Auth, Timeout

client = weaviate.connect_to_custom(
    http_host="weaviate.internal.example",
    http_port=443,
    http_secure=True,
    grpc_host="weaviate.internal.example",
    grpc_port=443,
    grpc_secure=True,
    auth_credentials=Auth.api_key(os.environ["WEAVIATE_API_KEY"]),
    headers={"X-OpenAI-Api-Key": os.environ["OPENAI_API_KEY"]},
    additional_config=AdditionalConfig(
        timeout=Timeout(init=30, query=60, insert=120)
    ),
)
```

### Direct Client Instantiation

If the helper functions are too limiting, instantiate `weaviate.WeaviateClient(...)` directly. When you do that, you must call `client.connect()` yourself before using it.

## Always Close The Client

Since `v4.4b7`, you must close client connections explicitly. Two safe patterns:

```python
client = weaviate.connect_to_local()
try:
    # work
    pass
finally:
    client.close()
```

```python
with weaviate.connect_to_local() as client:
    print(client.is_ready())
```

## Defining A Collection

For production code, define properties explicitly instead of relying on auto-schema. Weaviate's collection naming conventions also matter:

- collection names should start with an upper-case letter
- property names should start with a lower-case letter

```python
import weaviate
import weaviate.classes as wvc

with weaviate.connect_to_local() as client:
    if not client.collections.exists("Article"):
        client.collections.create(
            "Article",
            vector_config=wvc.config.Configure.Vectors.text2vec_openai(),
            properties=[
                wvc.config.Property(
                    name="title",
                    data_type=wvc.config.DataType.TEXT,
                ),
                wvc.config.Property(
                    name="body",
                    data_type=wvc.config.DataType.TEXT,
                ),
                wvc.config.Property(
                    name="source",
                    data_type=wvc.config.DataType.TEXT,
                ),
            ],
        )
```

If you provide your own embeddings instead of using a server-side vectorizer, use `wvc.config.Configure.Vectors.self_provided()`.

## Insert Data

### Insert One Object

```python
import weaviate

with weaviate.connect_to_local() as client:
    articles = client.collections.use("Article")
    uuid = articles.data.insert(
        properties={
            "title": "Project notes",
            "body": "Agents need concise package docs for evolving SDKs.",
            "source": "internal",
        }
    )
    print(uuid)
```

### Insert One Object With A Known UUID

```python
articles.data.insert(
    properties={
        "title": "Deterministic object",
        "body": "Use a stable UUID when you need idempotent upserts.",
        "source": "seed",
    },
    uuid="12345678-e64f-5d94-90db-c8cfa3fc1234",
)
```

### Insert With A User-Provided Vector

```python
articles.data.insert(
    properties={
        "title": "Pre-embedded object",
        "body": "The embedding was computed outside Weaviate.",
        "source": "offline-pipeline",
    },
    vector=[0.123] * 1536,
)
```

## Batch Import

Batching is a common point of failure when copying older examples. In `4.20.4`:

- batching must use a context manager
- old manual patterns such as `.create_objects(...)` are obsolete
- use `dynamic()`, `fixed_size()`, or `rate_limit()`
- failed writes are collected on `collection.batch.failed_objects` and `failed_references`

```python
import weaviate

rows = [
    {"title": "Doc 1", "body": "Batch import example", "source": "seed"},
    {"title": "Doc 2", "body": "Second row", "source": "seed"},
]

with weaviate.connect_to_local() as client:
    articles = client.collections.use("Article")

    with articles.batch.fixed_size(batch_size=200) as batch:
        for row in rows:
            batch.add_object(properties=row)
            if batch.number_errors > 10:
                raise RuntimeError("Too many batch errors")

    if articles.batch.failed_objects:
        raise RuntimeError(articles.batch.failed_objects[0])
```

Use `rate_limit()` if your vectorizer or generative provider has strict request-per-minute limits.

The async Python client does not support batching. Use the sync client for bulk imports.

## Query Data

### Fetch Objects

```python
with weaviate.connect_to_local() as client:
    articles = client.collections.use("Article")
    response = articles.query.fetch_objects(
        return_properties=["title", "source"],
        limit=5,
    )

    for obj in response.objects:
        print(obj.uuid, obj.properties)
```

### Vector Search With `near_text`

`near_text` only works when the target collection has a vectorizer configured.

```python
import weaviate
import weaviate.classes as wvc

with weaviate.connect_to_local() as client:
    articles = client.collections.use("Article")
    response = articles.query.near_text(
        query="package documentation for coding agents",
        filters=wvc.query.Filter.by_property("source").equal("internal"),
        limit=3,
        return_metadata=wvc.query.MetadataQuery(distance=True),
    )

    for obj in response.objects:
        print(obj.properties["title"], obj.metadata.distance)
```

### Named Vectors

If the collection uses named vectors, pass `target_vector="name"` to `near_text`, `near_object`, `near_vector`, or hybrid search methods.

### Multi-Tenant Collections

If a collection is multi-tenant, scope the handle before querying or writing:

```python
tenant_articles = client.collections.use("Article").with_tenant("tenantA")
```

## Async Client

An async API is available through `WeaviateAsyncClient` from `weaviate-client` `v4.7.0+`. Use it when your application is already async, but do not expect batching support there.

## Authentication And Headers

- For Weaviate Cloud, prefer `Auth.api_key(...)`.
- If you use OpenAI, Cohere, or another provider-backed vectorizer or generative module, pass those credentials through `headers`.
- Keep secrets in environment variables; do not hard-code them into examples or checked-in source.
- If you use the lower-level `WeaviateClient(...)` constructor, the equivalent parameter is `additional_headers`.

Common header examples:

```python
headers = {
    "X-OpenAI-Api-Key": os.environ["OPENAI_API_KEY"],
    "X-Cohere-Api-Key": os.environ["COHERE_API_KEY"],
}
```

## Timeouts And Network Issues

The Python client uses gRPC for many operations and is sensitive to network latency. If connection init, queries, or inserts time out, raise the specific timeout bucket instead of blindly retrying everything.

```python
from weaviate.classes.init import AdditionalConfig, Timeout

additional_config=AdditionalConfig(
    timeout=Timeout(init=30, query=60, insert=120)
)
```

If `generate` queries are timing out, increase the query timeout first.

## Common Pitfalls

### 1. Forgetting The gRPC Port

If you can reach HTTP but not gRPC, some operations will fail or behave unexpectedly. For local/self-hosted setups, expose `50051` unless you intentionally changed it.

### 2. Copying Pre-`4.16.0` Collection Config Examples

Current `4.20.x` code should use:

- `vector_config`, not `vectorizer_config`
- `Configure.Vectors` / `Configure.MultiVectors`
- `Configure.Vectors.self_provided()`, not older `none()` helpers

### 3. Copying Pre-`4.4b7` Batch Or Filter Examples

Outdated snippets often use:

- `client.batch` without a context manager
- removed manual batch methods
- older filter syntax instead of `Filter.by_property(...)`

### 4. Leaving Auto-Schema On In Production

Auto-schema is convenient for quick experiments, but the docs recommend manually defining production schemas for predictable behavior.

### 5. Not Closing The Client

`client.close()` is required unless you use a context manager.

### 6. Threading Assumptions

The docs describe the client as fundamentally designed to be thread-safe, but they also call out limitations due to the `requests` dependency, and batching is not thread-safe. Do not share a single batch workflow across threads.

## Version-Sensitive Notes For `4.20.4`

- PyPI currently publishes `4.20.4` as the latest version, released on `2026-03-10`.
- The main Python client landing page still presents itself as `v4.20.3`, but its compatibility table already covers `4.20.x`.
- PyPI metadata for `4.20.4` requires Python `>=3.10`. Some upstream descriptive text still mentions older Python versions; prefer the package metadata when deciding whether an environment is supported.
- The `4.16.0` to `4.16.3` auto-schema/vectorizer edge case is already fixed in `4.16.4+`, so `4.20.4` is not affected.

## Official Sources

- Weaviate Python client docs: https://docs.weaviate.io/weaviate/client-libraries/python
- Notes and best practices: https://docs.weaviate.io/weaviate/client-libraries/python/notes-best-practices
- Weaviate Cloud connection guide: https://docs.weaviate.io/weaviate/connections/connect-cloud
- Collection operations: https://docs.weaviate.io/weaviate/manage-collections/collection-operations
- Object creation: https://docs.weaviate.io/weaviate/manage-data/create
- Batch import: https://docs.weaviate.io/weaviate/manage-objects/import
- Search basics: https://docs.weaviate.io/weaviate/search/basics
- Vector similarity search: https://docs.weaviate.io/weaviate/search/similarity
- PyPI package page: https://pypi.org/project/weaviate-client/

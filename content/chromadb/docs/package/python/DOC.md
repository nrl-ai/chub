---
name: package
description: "ChromaDB Python package for local, server, and cloud vector search, collections, embeddings, and retrieval"
metadata:
  languages: "python"
  versions: "1.5.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "chromadb,vector-db,retrieval,rag,embeddings,search"
---

# ChromaDB Python Package Guide

## Golden Rule

Use `chromadb` for Python code that creates or queries Chroma collections, and choose the client class based on where the database lives:

- `PersistentClient` for local embedded storage
- `EphemeralClient` or `Client()` for tests and short-lived prototypes
- `HttpClient` or `AsyncHttpClient` for a self-hosted server
- `CloudClient` for Chroma Cloud

Be explicit about tenant, database, and embedding strategy when the defaults are not obvious. In current Chroma docs, query and get remain the core collection APIs for OSS and self-hosted use.

## Install

Pin the version your project expects:

```bash
python -m pip install "chromadb==1.5.5"
```

Common alternatives:

```bash
uv add "chromadb==1.5.5"
poetry add "chromadb==1.5.5"
```

Notes:

- `chromadb` includes the Python client and the bundled `chroma` CLI.
- If you only need a smaller remote-only client, upstream also publishes `chromadb-client`, but this doc covers the full `chromadb` package.
- PyPI shows `1.5.3` as yanked, so prefer an exact known-good pin such as `1.5.5` rather than assuming any `1.5.x` build is interchangeable.

## Choose The Right Client

### Local persistent storage

Use this for embedded apps, notebooks, local development, and single-node deployments:

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma-data")
```

### In-memory development client

Use this for tests or disposable prototypes:

```python
import chromadb

client = chromadb.EphemeralClient()
```

`chromadb.Client()` is the environment-configured variant. It is useful when you want client construction to follow `Settings`, `.env`, or other environment-driven configuration:

```python
import chromadb

client = chromadb.Client()
```

### Self-hosted server client

Run a local or remote Chroma server:

```bash
chroma run --path ./chroma-data
```

Then connect over HTTP:

```python
import chromadb
from chromadb.config import DEFAULT_DATABASE, DEFAULT_TENANT, Settings

client = chromadb.HttpClient(
    host="localhost",
    port=8000,
    ssl=False,
    headers=None,
    settings=Settings(),
    tenant=DEFAULT_TENANT,
    database=DEFAULT_DATABASE,
)
```

Async HTTP is available when your app is already async:

```python
import asyncio

import chromadb

async def main() -> None:
    client = await chromadb.AsyncHttpClient(host="localhost", port=8000, ssl=False)
    print(client)

asyncio.run(main())
```

### Chroma Cloud client

Cloud uses an API key plus tenant and database selection:

```bash
export CHROMA_API_KEY="ck-..."
export CHROMA_TENANT="your-tenant-id"
export CHROMA_DATABASE="your-database-name"
```

```python
import chromadb

client = chromadb.CloudClient()
```

Or pass them explicitly:

```python
import chromadb

client = chromadb.CloudClient(
    api_key="ck-...",
    tenant="your-tenant-id",
    database="your-database-name",
)
```

## Core Collection Workflow

Create or reuse a collection, then add or upsert records:

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma-data")

collection = client.get_or_create_collection(
    name="support_articles",
    configuration={"hnsw": {"space": "cosine"}},
)

collection.upsert(
    ids=["doc-1", "doc-2"],
    documents=[
        "Reset your password from the account settings page.",
        "Contact billing@example.com for invoice issues.",
    ],
    metadatas=[
        {"source": "kb", "tags": ["auth", "account"]},
        {"source": "kb", "tags": ["billing", "account"]},
    ],
)
```

What matters here:

- If you add only `documents`, Chroma computes embeddings using the collection's embedding function.
- If you add `documents` plus explicit `embeddings`, Chroma stores both without re-embedding the documents.
- Metadata values can be strings, integers, floats, booleans, and homogeneous arrays of those scalar types.

## Query, Get, And Filters

Use `.query()` for similarity search and `.get()` for direct retrieval without ranking:

```python
result = collection.query(
    query_texts=["How do I change my password?"],
    n_results=3,
    where={"tags": {"$contains": "auth"}},
    include=["documents", "metadatas", "distances"],
)

for doc_id, document, metadata, distance in zip(
    result["ids"][0],
    result["documents"][0],
    result["metadatas"][0],
    result["distances"][0],
):
    print(doc_id, distance, metadata, document)
```

```python
records = collection.get(
    ids=["doc-1"],
    include=["documents", "metadatas"],
)

for doc_id, document, metadata in zip(
    records["ids"],
    records["documents"],
    records["metadatas"],
):
    print(doc_id, metadata, document)
```

Useful filters:

- `where={...}` for metadata predicates such as equality, ranges, `$and`, `$or`, `$in`, and array membership with `$contains` or `$not_contains`
- `where_document={...}` for document-text filters such as `$contains` and `$regex`

Example document filter:

```python
matches = collection.get(
    where_document={"$regex": "billing@example\\.com"},
    include=["documents"],
)
```

Result-shape reminder:

- `.query()` returns grouped results per input query, so Python results are nested lists
- `.get()` returns flat arrays, so corresponding elements line up by index
- `include=[...]` controls payload size; `ids` are always returned

## Embedding Functions

If you do not specify an embedding function, Chroma uses `DefaultEmbeddingFunction`, which runs locally and uses the `all-MiniLM-L6-v2` model. The first call may download model files automatically.

Default embedding function:

```python
collection = client.create_collection(name="notes")
```

Provider-backed embedding function:

```python
from chromadb.utils.embedding_functions import OpenAIEmbeddingFunction

collection = client.create_collection(
    name="openai-notes",
    embedding_function=OpenAIEmbeddingFunction(
        model_name="text-embedding-3-small",
    ),
)
```

If a collection has no embedding function attached, you must provide `query_embeddings` rather than `query_texts`.

## Auth, Tenancy, And Configuration

### Self-hosted token auth

For Chroma `1.0.x+`, use a proxy or external auth layer rather than legacy built-in auth examples. The cookbook shows Envoy-based token auth with either `Authorization` or `X-Chroma-Token`.

Python client example:

```python
import os

import chromadb
from chromadb.config import Settings

client = chromadb.HttpClient(
    host="chroma.internal.example",
    port=443,
    ssl=True,
    settings=Settings(
        chroma_client_auth_provider="chromadb.auth.token_authn.TokenAuthClientProvider",
        chroma_client_auth_credentials=os.environ["CHROMA_TOKEN"],
        chroma_auth_token_transport_header="Authorization",
    ),
)
```

### Tenants and databases

All current client constructors accept or resolve a tenant and database. Defaults are usually `default_tenant` and `default_database`, but production code should not assume that when you are using shared or cloud environments.

### Collection configuration

At collection creation time, you can tune HNSW parameters such as:

- `space`: `l2`, `cosine`, or `ip`
- `ef_construction`
- `ef_search`
- `max_neighbors`

For text embeddings, `cosine` is often the right first choice.

## Common Pitfalls

- `collection.add()` ignores rows whose IDs already exist. Use `update()` or `upsert()` when you intend to overwrite.
- `update()` recomputes embeddings if you pass `documents` without corresponding `embeddings`.
- Manual embeddings and query embeddings must match the collection's embedding dimensionality.
- Collection names are restricted: 3 to 512 characters, lowercase letter or digit at both ends, dots/dashes/underscores allowed inside, no consecutive dots, and not a valid IP address.
- `where_document` full-text and regex matching is case-sensitive.
- Query results are nested by input query. Agents often treat `result["documents"]` as a flat list and then zip the wrong level.
- Use `include` aggressively. Returning embeddings, documents, metadatas, and distances for every query can waste bandwidth and tokens.
- `PersistentClient`, `HttpClient`, and `AsyncHttpClient` accept positional parameters, but keyword arguments are safer because the signatures are dense and easy to misorder.

## Version-Sensitive Notes For 1.5.5

- PyPI lists `chromadb 1.5.5` as the latest release on March 10, 2026, with Python `>=3.9`.
- Array metadata is part of the current docs model: you can store homogeneous arrays and filter them with `$contains` and `$not_contains`.
- The v1.0.0 migration notes say Chroma no longer provides built-in authentication implementations. Prefer current proxy or token-based patterns from the cookbook instead of pre-1.0 auth examples.
- The current docs still center OSS and self-hosted retrieval on `collection.query()` and `collection.get()`. Do not assume Cloud-specific search examples replace these methods in existing Python codebases.

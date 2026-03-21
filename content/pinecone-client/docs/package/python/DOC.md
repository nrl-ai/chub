---
name: package
description: "pinecone-client Python package guide for Pinecone vector database SDK usage and migration to the renamed pinecone package"
metadata:
  languages: "python"
  versions: "6.0.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pinecone,pinecone-client,vector-database,embeddings,rag,python"
---

# pinecone-client Python Package Guide

## Golden Rule

`pinecone-client` is deprecated. For `6.0.0`, treat this package entry as a migration target to the official `pinecone` SDK package:

```bash
pip uninstall pinecone-client
pip install "pinecone==6.0.0"
```

Do not keep both `pinecone-client` and `pinecone` installed in the same environment. Pinecone explicitly warns that this can cause confusing interactions.

## When To Use It

Use Pinecone when your Python app needs a managed vector database for:

- semantic search and retrieval
- RAG pipelines
- metadata-filtered nearest-neighbor search
- hybrid workflows with Pinecone-managed embedding and reranking

For new code, pin the `pinecone` package version that matches your deployment target. This doc tracks the `6.0.0` line because that is the version used here associated with `pinecone-client`.

## Install

### Standard install

```bash
pip uninstall -y pinecone-client
pip install "pinecone==6.0.0"
```

### gRPC transport for higher-throughput data operations

```bash
pip uninstall -y pinecone-client
pip install "pinecone[grpc]==6.0.0"
```

Use gRPC when `upsert` and `query` throughput matter more than keeping dependencies minimal.

### Async support

```bash
pip uninstall -y pinecone-client
pip install "pinecone[asyncio]==6.0.0"
```

Use the asyncio extra for async web stacks such as FastAPI or `aiohttp`.

## Auth And Configuration

Set your API key before creating a client:

```bash
export PINECONE_API_KEY="your-api-key"
export PINECONE_INDEX_HOST="your-index-host"
```

`PINECONE_INDEX_HOST` is the unique DNS host for a specific index. Pinecone recommends targeting indexes by host in production instead of by index name, because using the name forces an extra `describe_index` lookup.

You can get the host from:

- the Pinecone console
- `describe_index(name="...")` during setup or provisioning

## Initialize The Client

```python
import os
from pinecone import Pinecone

pc = Pinecone(api_key=os.environ["PINECONE_API_KEY"])
index = pc.Index(host=os.environ["PINECONE_INDEX_HOST"])
```

For quick experiments, `pc.Index("index-name")` is convenient. For production code, cache the host and pass `host=...` directly.

## Create An Index

For vectors generated outside Pinecone:

```python
import os
from pinecone import Pinecone, ServerlessSpec

pc = Pinecone(api_key=os.environ["PINECONE_API_KEY"])
index_name = "docs-example"

if not pc.has_index(index_name):
    pc.create_index(
        name=index_name,
        vector_type="dense",
        dimension=1536,
        metric="cosine",
        spec=ServerlessSpec(cloud="aws", region="us-east-1"),
        deletion_protection="disabled",
    )
```

Match `dimension` and `metric` to the embedding model you actually use. Pinecone will not fix a dimension mismatch for you.

## Core Vector Workflow

### Upsert vectors

```python
index.upsert(
    vectors=[
        {
            "id": "vec1",
            "values": [0.1] * 1536,
            "metadata": {"genre": "comedy", "year": 2020},
        },
        {
            "id": "vec2",
            "values": [0.2] * 1536,
            "metadata": {"genre": "documentary", "year": 2019},
        },
    ],
    namespace="example-namespace",
)
```

### Query by vector

```python
results = index.query(
    namespace="example-namespace",
    vector=[0.15] * 1536,
    top_k=3,
    include_metadata=True,
    include_values=False,
    filter={"genre": {"$eq": "documentary"}},
)
```

### Fetch by ID

```python
records = index.fetch(
    ids=["vec1", "vec2"],
    namespace="example-namespace",
)
```

### Delete by ID or metadata filter

```python
index.delete(ids=["vec1"], namespace="example-namespace")

index.delete(
    filter={"genre": {"$eq": "documentary"}},
    namespace="example-namespace",
)
```

Pinecone is eventually consistent. After `upsert` or `delete`, allow for a short delay before assuming queries will reflect the latest state.

## Integrated Embedding Workflow

If you want Pinecone to generate embeddings from source text, create an index configured for a model and use `upsert_records` plus `search`:

```python
import os
from pinecone import Pinecone

pc = Pinecone(api_key=os.environ["PINECONE_API_KEY"])
index_name = "integrated-search"

if not pc.has_index(index_name):
    pc.create_index_for_model(
        name=index_name,
        cloud="aws",
        region="us-east-1",
        embed={
            "model": "llama-text-embed-v2",
            "field_map": {"text": "chunk_text"},
        },
    )

index = pc.Index(host=os.environ["PINECONE_INDEX_HOST"])

index.upsert_records(
    namespace="docs",
    records=[
        {
            "_id": "doc-1",
            "chunk_text": "Pinecone stores and searches vectors.",
            "category": "guide",
        }
    ],
)

results = index.search(
    namespace="docs",
    query={"inputs": {"text": "vector search setup"}, "top_k": 3},
    fields=["category", "chunk_text"],
)
```

Use `search` with text only on indexes that were created with integrated embedding. For regular vector indexes, use `query` with a vector.

## Async And gRPC Variants

### gRPC

The API shape stays the same, but you import a different client:

```python
from pinecone.grpc import PineconeGRPC as Pinecone
```

Pinecone documents gRPC as a modest performance improvement for data operations such as `upsert` and `query`. It also supports parallel writes with `async_req=True`.

### asyncio

```python
import os
import asyncio
from pinecone import PineconeAsyncio

async def main():
    async with PineconeAsyncio(api_key=os.environ["PINECONE_API_KEY"]) as pc:
        async with pc.IndexAsyncio(host=os.environ["PINECONE_INDEX_HOST"]) as idx:
            await idx.query(vector=[0.15] * 1536, top_k=3)

asyncio.run(main())
```

Close async clients with `async with` so `aiohttp` sessions are cleaned up correctly.

## Common Pitfalls

- Do not install both `pinecone-client` and `pinecone`. Remove the deprecated package first.
- Do not assume the package name matches the import name. Use `pinecone`, not `pinecone_client`, in code.
- Do not target an index by name in production hot paths. Cache the index host and use `pc.Index(host=...)`.
- Do not mix text search and vector query semantics. `search(...)` with text input is for integrated-embedding indexes; `query(...)` is for vector or record-ID search.
- Do not ignore vector dimensions. The query vector length must match the index dimension.
- Do not assume namespace APIs are symmetric across transports. For example, `delete_namespace` is documented as not supported with `pinecone[grpc]`.
- Do not assume immediate read-after-write visibility. Pinecone documents eventual consistency for record freshness.

## Version-Sensitive Notes For 6.0.0

- `6.0.0` is the first major version after Pinecone's package rename guidance. The deprecated `pinecone-client` project on PyPI exists mainly to direct users to `pinecone`.
- Pinecone maps SDK major versions to API versions. Their Python SDK overview associates `v6.x` with API version `2025-01`.
- Pinecone's upgrade notes for `6.x` say Python `3.8` support was dropped and Python `3.13` support was added. Treat Python `3.9+` as the safe floor for this version line.
- Pinecone's current latest SDK docs are ahead of this package version and now describe later major versions as requiring Python `3.10+`. Do not copy that later interpreter requirement back onto a `6.0.0` environment without checking the actual pinned SDK version.
- Pinecone's upgrade notes also call out `asyncio` support in `6.x` and the removal of some deprecated internal configuration hooks. If you are upgrading older `5.x` code, re-check any custom client initialization.

## Official Sources

- Python SDK overview: `https://docs.pinecone.io/reference/python-sdk`
- Current SDK reference root: `https://sdk.pinecone.io/python/`
- SDK upgrading notes: `https://sdk.pinecone.io/python/upgrading.html`
- Deprecated package registry page: `https://pypi.org/project/pinecone-client/`
- Canonical runtime package registry page: `https://pypi.org/project/pinecone/`

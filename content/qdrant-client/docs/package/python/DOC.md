---
name: package
description: "qdrant-client for Python - Qdrant vector database client for local, server, and cloud usage"
metadata:
  languages: "python"
  versions: "1.17.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "qdrant,vector-database,embeddings,search,retrieval,rag"
---

# qdrant-client Python Package Guide

## Golden Rule

Use `qdrant-client` for Python integrations with Qdrant, and choose one connection mode up front:

- local mode for tests and quick prototypes
- remote REST for normal server or cloud usage
- gRPC when bulk upload throughput matters

Define the collection schema before writing points. Vector size and distance metric must match the embeddings you actually send.

## Install

```bash
pip install qdrant-client==1.17.0
```

Optional local embedding extras from the upstream README:

```bash
pip install "qdrant-client[fastembed]"
pip install "qdrant-client[fastembed-gpu]"
```

`qdrant-client` `1.17.0` requires Python `>=3.10`.

## Choose a Connection Mode

### In-memory local mode

Useful for tests, notebooks, and offline prototypes.

```python
from qdrant_client import QdrantClient

client = QdrantClient(location=":memory:")
```

### Persistent local mode

Stores data on disk without running a separate Qdrant server.

```python
from qdrant_client import QdrantClient

client = QdrantClient(path="./qdrant-data")
```

### Remote server over REST

Use either a full `url=` or `host=` plus ports. Do not mix styles unless you have a specific reason.

```python
from qdrant_client import QdrantClient

client = QdrantClient(url="http://localhost:6333")
```

### Qdrant Cloud

```python
import os
from qdrant_client import QdrantClient

client = QdrantClient(
    url=os.environ["QDRANT_URL"],
    api_key=os.environ["QDRANT_API_KEY"],
)
```

### Prefer gRPC for heavier writes

The client can use gRPC for custom methods and uploads when the server exposes it.

```python
import os
from qdrant_client import QdrantClient

client = QdrantClient(
    url=os.environ["QDRANT_URL"],
    api_key=os.environ.get("QDRANT_API_KEY"),
    prefer_grpc=True,
    grpc_port=6334,
)
```

## Initialize a Collection

Create the collection once with the right vector shape and distance metric.

```python
from qdrant_client import QdrantClient, models

client = QdrantClient(location=":memory:")

client.create_collection(
    collection_name="docs",
    vectors_config=models.VectorParams(
        size=768,
        distance=models.Distance.COSINE,
    ),
)
```

Notes:

- `size` must match the embedding dimension exactly.
- The metric is part of the collection schema. Changing models later often means recreating the collection.

## Insert Points

Use `PointStruct` when you already have vectors.

```python
from qdrant_client import QdrantClient, models

client = QdrantClient(location=":memory:")

client.create_collection(
    collection_name="docs",
    vectors_config=models.VectorParams(size=4, distance=models.Distance.COSINE),
)

client.upsert(
    collection_name="docs",
    points=[
        models.PointStruct(
            id=1,
            vector=[0.05, 0.61, 0.76, 0.74],
            payload={"title": "Getting started", "lang": "en"},
        ),
        models.PointStruct(
            id=2,
            vector=[0.19, 0.81, 0.75, 0.11],
            payload={"title": "Async usage", "lang": "en"},
        ),
    ],
)
```

For large ingests, prefer `upload_collection()` or `upload_points()` instead of hand-written small loops. The upstream client batches lazily and retries automatically.

## Query Points

Use `query_points()` for current generic vector search in the Python client docs.

```python
from qdrant_client import QdrantClient, models

client = QdrantClient(location=":memory:")

results = client.query_points(
    collection_name="docs",
    query=[0.04, 0.60, 0.77, 0.70],
    limit=3,
)

for point in results.points:
    print(point.id, point.score, point.payload)
```

Add filters when you need payload-aware retrieval:

```python
from qdrant_client import models

results = client.query_points(
    collection_name="docs",
    query=[0.04, 0.60, 0.77, 0.70],
    query_filter=models.Filter(
        must=[
            models.FieldCondition(
                key="lang",
                match=models.MatchValue(value="en"),
            )
        ]
    ),
    limit=3,
)
```

## Read and Delete

```python
from qdrant_client import models

points = client.retrieve(
    collection_name="docs",
    ids=[1, 2],
)

client.delete(
    collection_name="docs",
    points_selector=models.PointIdsList(points=[2]),
)
```

## Async Usage

`AsyncQdrantClient` exposes the same API shape as the sync client and is available in the Python client from `1.6.1` onward.

```python
from qdrant_client import AsyncQdrantClient, models

client = AsyncQdrantClient(url="http://localhost:6333")

await client.create_collection(
    collection_name="docs",
    vectors_config=models.VectorParams(size=4, distance=models.Distance.COSINE),
)

await client.upsert(
    collection_name="docs",
    points=[
        models.PointStruct(id=1, vector=[0.1, 0.2, 0.3, 0.4], payload={"kind": "note"}),
    ],
)

results = await client.query_points(
    collection_name="docs",
    query=[0.1, 0.2, 0.3, 0.4],
    limit=1,
)
```

Use the async client in FastAPI, Starlette, or other asyncio apps instead of wrapping the sync client in thread pools.

## Optional Embedded Inference Flow

If you install the FastEmbed extra, the upstream README shows higher-level document APIs:

```python
from qdrant_client import QdrantClient

client = QdrantClient(":memory:")

client.add(
    collection_name="demo_collection",
    documents=[
        "Qdrant has a LangChain integration",
        "Qdrant also has a LlamaIndex integration",
    ],
    metadata=[
        {"source": "langchain-docs"},
        {"source": "llamaindex-docs"},
    ],
    ids=[1, 2],
)

hits = client.query(
    collection_name="demo_collection",
    query_text="Which integrations does Qdrant support?",
)
```

Use this only when you intentionally want the client to own embedding generation. If your application already uses a separate embedding model or pipeline, keep embeddings explicit and use `upsert()` plus `query_points()`.

## Auth and Configuration

### Common environment variables

```bash
export QDRANT_URL="https://YOUR-CLUSTER.cloud.qdrant.io:6333"
export QDRANT_API_KEY="YOUR_API_KEY"
```

### Token-based auth

The client constructor supports `auth_token_provider` for bearer token flows that must refresh at runtime.

### Compatibility checks

The constructor supports `check_compatibility=True` by default. Leave it enabled unless you are intentionally testing against a mismatched server.

### Timeout and transport settings

The constructor also supports `timeout`, `https`, `host`, `port`, `grpc_port`, and `prefer_grpc`. Pick one connection style and keep it consistent per service.

## Common Pitfalls

- Import path is `qdrant_client`, not `qdrant`.
- Do not create a collection with the wrong vector dimension. Inserts and searches will fail or behave incorrectly.
- Do not switch embedding models without checking whether the target collection schema still matches the new vectors.
- Do not assume local mode behaves like a clustered production deployment. It is mainly for development, tests, and lightweight local persistence.
- Do not mix `url` and `host` or `port` arguments casually. Prefer one style so the resolved endpoint is obvious.
- If you enable `prefer_grpc=True`, make sure the server exposes the gRPC port and that your network path allows it.
- Older examples on blogs may use `search()` heavily. In the current Python client docs, `query_points()` is the generic query entry point to prefer.
- For bulk ingestion, avoid per-point network round trips. Use `upload_points()` or `upload_collection()`.

## Version-Sensitive Notes for 1.17.0

- PyPI package version covered here is `1.17.0`.
- PyPI metadata for `1.17.0` requires Python `>=3.10`.
- The package supports both sync `QdrantClient` and async `AsyncQdrantClient`.
- The constructor surface in the current official API docs includes local mode, cloud API keys, gRPC preference, compatibility checks, and token-provider auth hooks.

## Official Sources

- Docs root: https://python-client.qdrant.tech/
- Quickstart: https://python-client.qdrant.tech/quickstart
- Sync API: https://python-client.qdrant.tech/qdrant_client.qdrant_client
- Async API: https://python-client.qdrant.tech/qdrant_client.async_qdrant_client
- PyPI package: https://pypi.org/project/qdrant-client/
- PyPI JSON metadata: https://pypi.org/pypi/qdrant-client/json
- GitHub README: https://github.com/qdrant/qdrant-client

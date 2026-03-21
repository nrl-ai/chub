---
name: vector-search
description: "Qdrant Python Client — use qdrant-client for vector database operations in Python"
metadata:
  languages: "python"
  versions: "1.15.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "qdrant,vector-search,embeddings,similarity,ai"
---

# Qdrant Python Client v1.15.1

## Golden Rule

**Always use `qdrant-client` for Qdrant vector database operations in Python.**

Install with:
```bash
pip install qdrant-client
```

For FastEmbed integration (automatic embedding generation):
```bash
pip install 'qdrant-client[fastembed]'
```

**Do NOT use:**
- Outdated or unmaintained Qdrant Python packages
- Direct REST API calls when the client library provides methods

The official `qdrant-client` provides both synchronous and asynchronous interfaces, gRPC support, and built-in FastEmbed integration.

## Installation

```bash
# Basic installation
pip install qdrant-client

# With FastEmbed support
pip install 'qdrant-client[fastembed]'

# Or using uv
uv pip install qdrant-client
```

## Initialization

### In-Memory Client

```python
from qdrant_client import QdrantClient

# In-memory instance (no persistence)
client = QdrantClient(":memory:")
```

### Local Persistent Storage

```python
from qdrant_client import QdrantClient

# Persistent local storage
client = QdrantClient(path="./qdrant_data")
```

### Remote Server

```python
from qdrant_client import QdrantClient

# Connect to Qdrant server
client = QdrantClient(host="localhost", port=6333)

# Alternative using URL
client = QdrantClient(url="http://localhost:6333")
```

### Qdrant Cloud

```python
from qdrant_client import QdrantClient

# Connect to Qdrant Cloud
client = QdrantClient(
    url="https://xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.us-east-0-1.aws.cloud.qdrant.io",
    api_key="your-api-key-here"
)
```

### With Environment Variables

```python
import os
from qdrant_client import QdrantClient

client = QdrantClient(
    url=os.getenv("QDRANT_URL", "http://localhost:6333"),
    api_key=os.getenv("QDRANT_API_KEY")
)
```

### gRPC Configuration

```python
from qdrant_client import QdrantClient

# Use gRPC for better performance
client = QdrantClient(
    host="localhost",
    grpc_port=6334,
    prefer_grpc=True
)
```

### Async Client

```python
from qdrant_client import AsyncQdrantClient

# Async client initialization
async_client = AsyncQdrantClient(
    url="http://localhost:6333"
)

# Use with async/await
async def main():
    collections = await async_client.get_collections()
    print(collections)
```

## Collections

### List Collections

```python
collections = client.get_collections()
print(collections.collections)
```

### Check Collection Exists

```python
exists = client.collection_exists("my_collection")
print(f"Collection exists: {exists}")
```

### Create Collection - Single Vector

```python
from qdrant_client.models import Distance, VectorParams

client.create_collection(
    collection_name="my_collection",
    vectors_config=VectorParams(size=384, distance=Distance.COSINE)
)
```

### Distance Metrics

```python
from qdrant_client.models import Distance

# Available distance metrics:
# Distance.COSINE - Cosine similarity
# Distance.EUCLID - Euclidean distance
# Distance.DOT - Dot product
# Distance.MANHATTAN - Manhattan distance
```

### Create Collection - Named Vectors

```python
from qdrant_client.models import Distance, VectorParams

client.create_collection(
    collection_name="multi_vector_collection",
    vectors_config={
        "image": VectorParams(size=512, distance=Distance.DOT),
        "text": VectorParams(size=384, distance=Distance.COSINE)
    }
)
```

### Create Collection - Advanced Configuration

```python
from qdrant_client.models import (
    Distance,
    VectorParams,
    OptimizersConfigDiff,
    HnswConfigDiff,
    QuantizationConfig,
    ScalarQuantization,
    ScalarType
)

client.create_collection(
    collection_name="advanced_collection",
    vectors_config=VectorParams(size=768, distance=Distance.COSINE),
    shard_number=2,
    replication_factor=1,
    write_consistency_factor=1,
    on_disk_payload=True,
    hnsw_config=HnswConfigDiff(
        m=16,
        ef_construct=100,
        full_scan_threshold=10000
    ),
    optimizers_config=OptimizersConfigDiff(
        default_segment_number=2,
        indexing_threshold=20000
    ),
    quantization_config=ScalarQuantization(
        scalar=ScalarType.INT8,
        quantile=0.99,
        always_ram=True
    )
)
```

### Create Collection - Sparse Vectors

```python
from qdrant_client.models import Distance, VectorParams, SparseVectorParams

client.create_collection(
    collection_name="sparse_collection",
    vectors_config={
        "dense": VectorParams(size=384, distance=Distance.COSINE),
        "sparse": SparseVectorParams()
    }
)
```

### Get Collection Info

```python
info = client.get_collection("my_collection")
print(f"Points count: {info.points_count}")
print(f"Vectors config: {info.config.params.vectors}")
```

### Update Collection

```python
from qdrant_client.models import OptimizersConfigDiff

client.update_collection(
    collection_name="my_collection",
    optimizers_config=OptimizersConfigDiff(
        indexing_threshold=30000
    )
)
```

### Delete Collection

```python
client.delete_collection("my_collection")
```

## Points (Vectors)

### Upsert Points - Basic

```python
from qdrant_client.models import PointStruct

operation_info = client.upsert(
    collection_name="my_collection",
    wait=True,
    points=[
        PointStruct(
            id=1,
            vector=[0.05, 0.61, 0.76, 0.74],
            payload={"city": "Berlin", "price": 1.99}
        ),
        PointStruct(
            id=2,
            vector=[0.19, 0.81, 0.75, 0.11],
            payload={"city": "London", "price": 2.49}
        )
    ]
)
```

### Upsert Points - Auto ID

```python
from qdrant_client.models import PointStruct
import uuid

points = [
    PointStruct(
        id=str(uuid.uuid4()),
        vector=[0.1, 0.2, 0.3, 0.4],
        payload={"title": "Document 1"}
    )
]

client.upsert(
    collection_name="my_collection",
    points=points
)
```

### Upsert Points - Named Vectors

```python
from qdrant_client.models import PointStruct

client.upsert(
    collection_name="multi_vector_collection",
    points=[
        PointStruct(
            id=1,
            vector={
                "image": [0.1, 0.2, 0.3, 0.4],
                "text": [0.5, 0.6, 0.7, 0.8]
            },
            payload={"title": "Product A", "category": "electronics"}
        )
    ]
)
```

### Upsert Points - Sparse Vectors

```python
from qdrant_client.models import PointStruct, SparseVector

client.upsert(
    collection_name="sparse_collection",
    points=[
        PointStruct(
            id=1,
            vector={
                "dense": [0.1, 0.2, 0.3, 0.4],
                "sparse": SparseVector(
                    indices=[1, 3, 5, 7],
                    values=[0.1, 0.2, 0.3, 0.4]
                )
            },
            payload={"text": "Sample document"}
        )
    ]
)
```

### Upsert Points - Batch

```python
from qdrant_client.models import PointStruct
import numpy as np

vectors = np.random.rand(1000, 384)
points = [
    PointStruct(
        id=idx,
        vector=vector.tolist(),
        payload={"index": idx, "category": f"cat_{idx % 10}"}
    )
    for idx, vector in enumerate(vectors)
]

client.upsert(
    collection_name="my_collection",
    points=points,
    wait=True
)
```

### Add Documents with FastEmbed

```python
documents = [
    "Qdrant has Langchain integrations",
    "Qdrant also has Llama Index integrations",
    "Qdrant is a vector database"
]

metadata = [
    {"source": "langchain-docs"},
    {"source": "llamaindex-docs"},
    {"source": "qdrant-docs"}
]

ids = [1, 2, 3]

client.add(
    collection_name="demo_collection",
    documents=documents,
    metadata=metadata,
    ids=ids
)
```

### Retrieve Points by ID

```python
points = client.retrieve(
    collection_name="my_collection",
    ids=[1, 2, 3],
    with_payload=True,
    with_vectors=False
)

for point in points:
    print(f"ID: {point.id}, Payload: {point.payload}")
```

### Retrieve Points - With Vectors

```python
points = client.retrieve(
    collection_name="my_collection",
    ids=[1, 2, 3],
    with_payload=True,
    with_vectors=True
)
```

### Delete Points by ID

```python
client.delete(
    collection_name="my_collection",
    points_selector=[1, 2, 3]
)
```

### Delete Points by Filter

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue

client.delete(
    collection_name="my_collection",
    points_selector=Filter(
        must=[
            FieldCondition(
                key="city",
                match=MatchValue(value="Berlin")
            )
        ]
    )
)
```

### Count Points

```python
count = client.count(
    collection_name="my_collection",
    exact=True
)
print(f"Total points: {count.count}")
```

### Count Points with Filter

```python
from qdrant_client.models import Filter, FieldCondition, Range

count = client.count(
    collection_name="my_collection",
    count_filter=Filter(
        must=[
            FieldCondition(
                key="price",
                range=Range(gte=2.0)
            )
        ]
    ),
    exact=True
)
```

## Scroll (Pagination)

### Basic Scroll

```python
records, next_page_offset = client.scroll(
    collection_name="my_collection",
    limit=10,
    with_payload=True,
    with_vectors=False
)

for record in records:
    print(f"ID: {record.id}, Payload: {record.payload}")
```

### Scroll with Filter

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue

records, next_page_offset = client.scroll(
    collection_name="my_collection",
    scroll_filter=Filter(
        must=[
            FieldCondition(
                key="city",
                match=MatchValue(value="Berlin")
            )
        ]
    ),
    limit=20,
    with_payload=True
)
```

### Scroll Pagination

```python
offset = None
all_points = []

while True:
    records, next_page_offset = client.scroll(
        collection_name="my_collection",
        limit=100,
        offset=offset,
        with_payload=True,
        with_vectors=False
    )

    all_points.extend(records)

    if next_page_offset is None:
        break

    offset = next_page_offset

print(f"Total points retrieved: {len(all_points)}")
```

### Scroll with Order By

```python
records, next_page_offset = client.scroll(
    collection_name="my_collection",
    limit=15,
    order_by="timestamp",
    with_payload=True
)
```

## Search

### Basic Vector Search

```python
search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.18, 0.81, 0.75, 0.12],
    limit=5
)

for result in search_result:
    print(f"ID: {result.id}, Score: {result.score}")
    print(f"Payload: {result.payload}")
```

### Search with Score Threshold

```python
search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.18, 0.81, 0.75, 0.12],
    limit=10,
    score_threshold=0.8
)
```

### Search Named Vectors

```python
search_result = client.search(
    collection_name="multi_vector_collection",
    query_vector=[0.1, 0.2, 0.3, 0.4],
    using="text",
    limit=5
)
```

### Search without Vectors

```python
search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.18, 0.81, 0.75, 0.12],
    limit=5,
    with_payload=True,
    with_vectors=False
)
```

### Text Query with FastEmbed

```python
search_result = client.query(
    collection_name="demo_collection",
    query_text="This is a query document",
    limit=5
)

for result in search_result:
    print(f"Score: {result.score}, Text: {result.payload}")
```

## Filtering

### Match Filter

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="city",
                match=MatchValue(value="London")
            )
        ]
    ),
    limit=5
)
```

### Range Filter

```python
from qdrant_client.models import Filter, FieldCondition, Range

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="price",
                range=Range(gte=1.0, lt=3.0)
            )
        ]
    ),
    limit=5
)
```

### Multiple Conditions (AND)

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue, Range

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="city",
                match=MatchValue(value="Berlin")
            ),
            FieldCondition(
                key="price",
                range=Range(gte=2.0)
            )
        ]
    ),
    limit=5
)
```

### OR Conditions

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        should=[
            FieldCondition(
                key="city",
                match=MatchValue(value="Berlin")
            ),
            FieldCondition(
                key="city",
                match=MatchValue(value="London")
            )
        ]
    ),
    limit=5
)
```

### NOT Conditions

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must_not=[
            FieldCondition(
                key="city",
                match=MatchValue(value="Berlin")
            )
        ]
    ),
    limit=5
)
```

### Complex Filter

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue, Range

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="category",
                match=MatchValue(value="electronics")
            )
        ],
        should=[
            FieldCondition(
                key="price",
                range=Range(lt=100)
            ),
            FieldCondition(
                key="discount",
                match=MatchValue(value=True)
            )
        ],
        must_not=[
            FieldCondition(
                key="out_of_stock",
                match=MatchValue(value=True)
            )
        ]
    ),
    limit=10
)
```

### Match Any (Array)

```python
from qdrant_client.models import Filter, FieldCondition, MatchAny

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="tags",
                match=MatchAny(any=["electronics", "gadgets", "tech"])
            )
        ]
    ),
    limit=5
)
```

### Geo Radius Filter

```python
from qdrant_client.models import Filter, FieldCondition, GeoRadius, GeoPoint

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="location",
                geo_radius=GeoRadius(
                    center=GeoPoint(lon=13.404954, lat=52.520008),
                    radius=5000.0
                )
            )
        ]
    ),
    limit=5
)
```

### Is Empty Filter

```python
from qdrant_client.models import Filter, IsEmptyCondition

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            IsEmptyCondition(is_empty={"key": "tags"})
        ]
    ),
    limit=5
)
```

### Is Null Filter

```python
from qdrant_client.models import Filter, IsNullCondition

search_result = client.search(
    collection_name="my_collection",
    query_vector=[0.2, 0.1, 0.9, 0.7],
    query_filter=Filter(
        must=[
            IsNullCondition(is_null={"key": "description"})
        ]
    ),
    limit=5
)
```

## Recommend

### Basic Recommend

```python
recommend_result = client.recommend(
    collection_name="my_collection",
    positive=[1, 2, 3],
    negative=[10],
    limit=5
)

for result in recommend_result:
    print(f"ID: {result.id}, Score: {result.score}")
```

### Recommend with Vectors

```python
recommend_result = client.recommend(
    collection_name="my_collection",
    positive=[1, 2, [0.1, 0.2, 0.3, 0.4]],
    negative=[[0.9, 0.8, 0.7, 0.6]],
    limit=5
)
```

### Recommend with Filter

```python
from qdrant_client.models import Filter, FieldCondition, MatchValue

recommend_result = client.recommend(
    collection_name="my_collection",
    positive=[1, 2, 3],
    negative=[10],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="category",
                match=MatchValue(value="electronics")
            )
        ]
    ),
    limit=5
)
```

### Recommend Strategy

```python
from qdrant_client.models import RecommendStrategy

recommend_result = client.recommend(
    collection_name="my_collection",
    positive=[1, 2, 3],
    negative=[10],
    strategy=RecommendStrategy.AVERAGE_VECTOR,
    limit=5
)
```

### Recommend Named Vectors

```python
recommend_result = client.recommend(
    collection_name="multi_vector_collection",
    positive=[1, 2],
    negative=[10],
    using="text",
    limit=5
)
```

## Discover

### Basic Discover

```python
from qdrant_client.models import ContextPair

discover_result = client.discover(
    collection_name="my_collection",
    target=100,
    context=[
        ContextPair(positive=200, negative=300),
        ContextPair(positive=150, negative=250)
    ],
    limit=10
)
```

### Discover with Filter

```python
from qdrant_client.models import ContextPair, Filter, FieldCondition, MatchValue

discover_result = client.discover(
    collection_name="my_collection",
    target=100,
    context=[
        ContextPair(positive=200, negative=300)
    ],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="active",
                match=MatchValue(value=True)
            )
        ]
    ),
    limit=10
)
```

## Query API (Universal)

### Vector Query

```python
from qdrant_client.models import QueryRequest, VectorQuery

query_result = client.query_points(
    collection_name="my_collection",
    query=[0.1, 0.2, 0.3, 0.4],
    limit=10
)
```

### Recommend Query

```python
from qdrant_client.models import RecommendQuery

query_result = client.query_points(
    collection_name="my_collection",
    query=RecommendQuery(
        recommend=RecommendInput(
            positive=[100, 231],
            negative=[718]
        )
    ),
    limit=10
)
```

### Discover Query

```python
from qdrant_client.models import DiscoverQuery, ContextPair

query_result = client.query_points(
    collection_name="my_collection",
    query=DiscoverQuery(
        discover=DiscoverInput(
            target=100,
            context=[
                ContextPair(positive=200, negative=300)
            ]
        )
    ),
    limit=10
)
```

### Context Query

```python
from qdrant_client.models import ContextQuery, ContextPair

query_result = client.query_points(
    collection_name="my_collection",
    query=ContextQuery(
        context=[
            ContextPair(positive=100, negative=718),
            ContextPair(positive=200, negative=300)
        ]
    ),
    limit=10
)
```

## Batch Operations

### Batch Search

```python
from qdrant_client.models import SearchRequest, Filter, FieldCondition, MatchValue

requests = [
    SearchRequest(
        vector=[0.1, 0.2, 0.3, 0.4],
        limit=5,
        filter=Filter(
            must=[
                FieldCondition(
                    key="category",
                    match=MatchValue(value="electronics")
                )
            ]
        )
    ),
    SearchRequest(
        vector=[0.5, 0.6, 0.7, 0.8],
        limit=5,
        filter=Filter(
            must=[
                FieldCondition(
                    key="category",
                    match=MatchValue(value="books")
                )
            ]
        )
    )
]

results = client.search_batch(
    collection_name="my_collection",
    requests=requests
)

for i, result in enumerate(results):
    print(f"Results for request {i}:")
    for point in result:
        print(f"  ID: {point.id}, Score: {point.score}")
```

### Batch Query

```python
from qdrant_client.models import QueryRequest

requests = [
    QueryRequest(
        query=[0.1, 0.2, 0.3, 0.4],
        limit=5
    ),
    QueryRequest(
        query=[0.5, 0.6, 0.7, 0.8],
        limit=5
    )
]

results = client.query_batch_points(
    collection_name="my_collection",
    requests=requests
)
```

### Batch Recommend

```python
from qdrant_client.models import RecommendRequest, Filter, FieldCondition, MatchValue

requests = [
    RecommendRequest(
        positive=[100, 231],
        negative=[718],
        limit=10
    ),
    RecommendRequest(
        positive=[500, 600],
        negative=[700, 800],
        limit=10,
        filter=Filter(
            must=[
                FieldCondition(
                    key="active",
                    match=MatchValue(value=True)
                )
            ]
        )
    )
]

results = client.recommend_batch(
    collection_name="my_collection",
    requests=requests
)
```

### Batch Update Points

```python
from qdrant_client.models import (
    PointStruct,
    UpsertOperation,
    DeleteOperation,
    SetPayloadOperation,
    PointIdsList,
    UpdateStatus
)

client.batch_update_points(
    collection_name="my_collection",
    update_operations=[
        UpsertOperation(
            upsert=PointsList(
                points=[
                    PointStruct(
                        id=1,
                        vector=[1.0, 2.0, 3.0, 4.0],
                        payload={"title": "Item 1"}
                    )
                ]
            )
        ),
        DeleteOperation(
            delete=PointIdsList(points=[5, 6, 7])
        ),
        SetPayloadOperation(
            set_payload=SetPayload(
                payload={"updated": True},
                points=[1, 2, 3]
            )
        )
    ]
)
```

### Mixed Batch Operations

```python
from qdrant_client.models import (
    UpsertOperation,
    UpdateVectorsOperation,
    DeletePayloadOperation,
    PointStruct,
    PointsList,
    UpdateVectors,
    PointVectors,
    DeletePayload,
    PointIdsList
)

client.batch_update_points(
    collection_name="my_collection",
    update_operations=[
        UpsertOperation(
            upsert=PointsList(
                points=[
                    PointStruct(
                        id=1,
                        vector=[1.0, 2.0, 3.0, 4.0],
                        payload={}
                    )
                ]
            )
        ),
        UpdateVectorsOperation(
            update_vectors=UpdateVectors(
                points=[
                    PointVectors(
                        id=2,
                        vector=[5.0, 6.0, 7.0, 8.0]
                    )
                ]
            )
        ),
        DeletePayloadOperation(
            delete_payload=DeletePayload(
                keys=["old_field"],
                points=PointIdsList(points=[3, 4])
            )
        )
    ]
)
```

## Payload Operations

### Set Payload

```python
from datetime import datetime

client.set_payload(
    collection_name="my_collection",
    payload={
        "updated_at": datetime.now().isoformat(),
        "verified": True
    },
    points=[1, 2, 3]
)
```

### Set Payload with Filter

```python
from qdrant_client.models import Filter, FieldCondition, Range

client.set_payload(
    collection_name="my_collection",
    payload={"promoted": True},
    points=Filter(
        must=[
            FieldCondition(
                key="price",
                range=Range(lt=50)
            )
        ]
    )
)
```

### Overwrite Payload

```python
client.overwrite_payload(
    collection_name="my_collection",
    payload={
        "title": "New Title",
        "price": 99.99
    },
    points=[1, 2]
)
```

### Delete Payload Keys

```python
client.delete_payload(
    collection_name="my_collection",
    keys=["old_field", "deprecated_field"],
    points=[1, 2, 3]
)
```

### Clear Payload

```python
client.clear_payload(
    collection_name="my_collection",
    points_selector=[1, 2, 3]
)
```

## Payload Indexing

### Create Payload Index - Keyword

```python
client.create_payload_index(
    collection_name="my_collection",
    field_name="city",
    field_schema="keyword"
)
```

### Create Payload Index - Integer

```python
client.create_payload_index(
    collection_name="my_collection",
    field_name="price",
    field_schema="integer"
)
```

### Create Payload Index - Float

```python
client.create_payload_index(
    collection_name="my_collection",
    field_name="rating",
    field_schema="float"
)
```

### Create Payload Index - Geo

```python
client.create_payload_index(
    collection_name="my_collection",
    field_name="location",
    field_schema="geo"
)
```

### Create Payload Index - Text

```python
from qdrant_client.models import TextIndexParams, TokenizerType

client.create_payload_index(
    collection_name="my_collection",
    field_name="description",
    field_schema=TextIndexParams(
        type="text",
        tokenizer=TokenizerType.WORD,
        min_token_len=2,
        max_token_len=20,
        lowercase=True
    )
)
```

### Delete Payload Index

```python
client.delete_payload_index(
    collection_name="my_collection",
    field_name="old_index_field"
)
```

## Snapshots

### Create Collection Snapshot

```python
snapshot_info = client.create_snapshot(
    collection_name="my_collection"
)
print(f"Snapshot created: {snapshot_info.name}")
```

### List Collection Snapshots

```python
snapshots = client.list_snapshots(
    collection_name="my_collection"
)

for snapshot in snapshots:
    print(f"Snapshot: {snapshot.name}, Size: {snapshot.size}")
```

### Delete Collection Snapshot

```python
client.delete_snapshot(
    collection_name="my_collection",
    snapshot_name="snapshot_2025-01-15-10-30-00"
)
```

### Create Full Storage Snapshot

```python
snapshot_info = client.create_full_snapshot()
print(f"Full snapshot created: {snapshot_info.name}")
```

### List Full Snapshots

```python
snapshots = client.list_full_snapshots()
for snapshot in snapshots:
    print(f"Full snapshot: {snapshot.name}")
```

### Download Snapshot

```python
client.download_snapshot(
    collection_name="my_collection",
    snapshot_name="snapshot_2025-01-15-10-30-00",
    snapshot_path="./backups/snapshot.snapshot"
)
```

## Cluster Operations

### Get Cluster Info

```python
cluster_info = client.cluster_status()
print(f"Peers: {cluster_info.peers}")
print(f"Raft info: {cluster_info.raft_info}")
```

## Async Operations

### Async Client Usage

```python
from qdrant_client import AsyncQdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct
import asyncio

async def main():
    client = AsyncQdrantClient(url="http://localhost:6333")

    # Create collection
    await client.create_collection(
        collection_name="async_collection",
        vectors_config=VectorParams(size=4, distance=Distance.COSINE)
    )

    # Upsert points
    await client.upsert(
        collection_name="async_collection",
        points=[
            PointStruct(
                id=1,
                vector=[0.1, 0.2, 0.3, 0.4],
                payload={"title": "Async document"}
            )
        ]
    )

    # Search
    results = await client.search(
        collection_name="async_collection",
        query_vector=[0.1, 0.2, 0.3, 0.4],
        limit=5
    )

    for result in results:
        print(f"ID: {result.id}, Score: {result.score}")

    await client.close()

asyncio.run(main())
```

### Async Context Manager

```python
from qdrant_client import AsyncQdrantClient

async def main():
    async with AsyncQdrantClient(url="http://localhost:6333") as client:
        collections = await client.get_collections()
        print(collections)

asyncio.run(main())
```

## Error Handling

### Basic Error Handling

```python
try:
    search_result = client.search(
        collection_name="my_collection",
        query_vector=[0.1, 0.2, 0.3, 0.4],
        limit=10
    )
except Exception as e:
    print(f"Search failed: {e}")
```

### Check Collection Exists Before Operations

```python
if not client.collection_exists("my_collection"):
    from qdrant_client.models import Distance, VectorParams

    client.create_collection(
        collection_name="my_collection",
        vectors_config=VectorParams(size=384, distance=Distance.COSINE)
    )
```

### Retry Logic

```python
import time
from qdrant_client.http.exceptions import UnexpectedResponse

def search_with_retry(client, collection_name, query_vector, max_retries=3):
    for attempt in range(max_retries):
        try:
            return client.search(
                collection_name=collection_name,
                query_vector=query_vector,
                limit=5
            )
        except UnexpectedResponse as e:
            if attempt == max_retries - 1:
                raise
            time.sleep(2 ** attempt)

results = search_with_retry(client, "my_collection", [0.1, 0.2, 0.3, 0.4])
```

## Complete Example

```python
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct, Filter, FieldCondition, MatchValue, Range
import numpy as np

# Initialize client
client = QdrantClient(url="http://localhost:6333")

collection_name = "products"

# Create collection
if not client.collection_exists(collection_name):
    client.create_collection(
        collection_name=collection_name,
        vectors_config=VectorParams(size=4, distance=Distance.COSINE)
    )

# Upsert points
client.upsert(
    collection_name=collection_name,
    points=[
        PointStruct(
            id=1,
            vector=[0.05, 0.61, 0.76, 0.74],
            payload={
                "name": "Product A",
                "category": "electronics",
                "price": 299.99,
                "in_stock": True
            }
        ),
        PointStruct(
            id=2,
            vector=[0.19, 0.81, 0.75, 0.11],
            payload={
                "name": "Product B",
                "category": "electronics",
                "price": 199.99,
                "in_stock": True
            }
        ),
        PointStruct(
            id=3,
            vector=[0.36, 0.55, 0.47, 0.94],
            payload={
                "name": "Product C",
                "category": "books",
                "price": 29.99,
                "in_stock": False
            }
        ),
        PointStruct(
            id=4,
            vector=[0.88, 0.22, 0.33, 0.44],
            payload={
                "name": "Product D",
                "category": "electronics",
                "price": 399.99,
                "in_stock": True
            }
        )
    ],
    wait=True
)

# Basic search
print("=== Basic Search ===")
results = client.search(
    collection_name=collection_name,
    query_vector=[0.2, 0.7, 0.8, 0.1],
    limit=3
)

for result in results:
    print(f"{result.payload['name']} - Score: {result.score:.4f}")

# Filtered search
print("\n=== Filtered Search (electronics under $300) ===")
filtered_results = client.search(
    collection_name=collection_name,
    query_vector=[0.2, 0.7, 0.8, 0.1],
    query_filter=Filter(
        must=[
            FieldCondition(
                key="category",
                match=MatchValue(value="electronics")
            ),
            FieldCondition(
                key="price",
                range=Range(lt=300)
            ),
            FieldCondition(
                key="in_stock",
                match=MatchValue(value=True)
            )
        ]
    ),
    limit=5
)

for result in filtered_results:
    print(f"{result.payload['name']} - ${result.payload['price']}")

# Count points
count = client.count(collection_name=collection_name, exact=True)
print(f"\n=== Total Points: {count.count} ===")

# Recommend
print("\n=== Recommendations based on Product A ===")
recommendations = client.recommend(
    collection_name=collection_name,
    positive=[1],
    negative=[],
    limit=3
)

for rec in recommendations:
    print(f"{rec.payload['name']} - Score: {rec.score:.4f}")

# Scroll through all points
print("\n=== All Products (via scroll) ===")
records, next_offset = client.scroll(
    collection_name=collection_name,
    limit=10,
    with_payload=True,
    with_vectors=False
)

for record in records:
    print(f"ID {record.id}: {record.payload['name']} - {record.payload['category']}")

# Cleanup
client.delete_collection(collection_name)
print(f"\n=== Collection '{collection_name}' deleted ===")
```

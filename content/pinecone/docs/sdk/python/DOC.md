---
name: sdk
description: "Vector database for AI applications with semantic search, hybrid search, reranking, and integrated embeddings"
metadata:
  languages: "python"
  versions: "7.3.0"
  updated-on: "2025-10-26"
  source: maintainer
  tags: "pinecone,sdk,vector-db,ai,search"
---
# Pinecone Python SDK Coding Guidelines

You are a Pinecone vector database coding expert. Help me with writing code using the Pinecone SDK for Python.

You can find the official SDK documentation and code samples here:
https://docs.pinecone.io/

## Golden Rule: Use the Correct and Current SDK

Always use the official Pinecone Python SDK for all Pinecone vector database interactions. Do not use legacy libraries or unofficial packages.

- **Library Name:** Pinecone Python SDK
- **PyPI Package:** `pinecone`
- **Legacy Libraries**: `pinecone-client` (deprecated) and other unofficial packages are not recommended

**Installation:**

- **Correct:** `pip install pinecone`
- **Correct (with optional dependencies):** `pip install "pinecone[asyncio,grpc]"`

**APIs and Usage:**

- **Correct:** `from pinecone import Pinecone`
- **Correct:** `pc = Pinecone(api_key='...')`
- **Correct:** `pc.create_index(...)`
- **Correct:** `pc.Index(host=...)`
- **Incorrect:** `import pinecone` (legacy pattern)
- **Incorrect:** `pinecone.init()` (deprecated)
- **Incorrect:** `pinecone.Index('name')` (legacy pattern)

## Installation

Install the Pinecone SDK using pip:

```bash
pip install pinecone
```

### With Optional Dependencies

For enhanced functionality, install with optional dependencies:

```bash
pip install "pinecone[asyncio,grpc]"
```

- `asyncio`: Enables async/await support
- `grpc`: Enables gRPC for better performance with high-concurrency scenarios

### Alternative Package Managers

**Using uv:**
```bash
uv add pinecone
```

**Using Poetry:**
```bash
poetry add pinecone
```

**Requirements:**
- Python 3.9 or higher
- Tested with CPython 3.9 to 3.13

## Initialization and API Key

The SDK requires creating a `Pinecone` instance for all API calls.

### Using Environment Variable

Set your API key as an environment variable:

```bash
export PINECONE_API_KEY="your_api_key_here"
```

Then initialize:

```python
from pinecone import Pinecone

pc = Pinecone()
```

### Using Configuration Parameter

Pass the API key directly:

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
```

### gRPC Client (Recommended for High Performance)

For better performance with high-concurrency operations:

```python
from pinecone.grpc import PineconeGRPC as Pinecone

pc = Pinecone(api_key='your_api_key_here')
```

The gRPC client offers multiplexing advantages over REST for parallel processing.

## Creating Indexes

### Serverless Index (Basic)

```python
from pinecone import Pinecone, ServerlessSpec

pc = Pinecone(api_key='your_api_key_here')

pc.create_index(
    name='my-index',
    dimension=1536,
    metric='cosine',
    spec=ServerlessSpec(
        cloud='aws',
        region='us-west-2'
    )
)
```

### Serverless Index with Deletion Protection

```python
from pinecone import Pinecone, ServerlessSpec

pc = Pinecone(api_key='your_api_key_here')

pc.create_index(
    name='protected-index',
    dimension=1536,
    metric='cosine',
    spec=ServerlessSpec(
        cloud='aws',
        region='us-east-1'
    ),
    deletion_protection='enabled'
)
```

### Pod-Based Index

```python
from pinecone import Pinecone, PodSpec

pc = Pinecone(api_key='your_api_key_here')

pc.create_index(
    name='pod-index',
    dimension=1536,
    metric='cosine',
    spec=PodSpec(
        environment='us-west-2',
        pod_type='p1.x1',
        pods=1,
        replicas=1
    )
)
```

### Hybrid Search Index (Sparse-Dense)

For hybrid search supporting both dense and sparse vectors, use `dotproduct` metric:

```python
from pinecone import Pinecone, ServerlessSpec

pc = Pinecone(api_key='your_api_key_here')

pc.create_index(
    name='hybrid-index',
    dimension=1024,
    metric='dotproduct',
    spec=ServerlessSpec(
        cloud='aws',
        region='us-east-1'
    )
)
```

**Note:** The `dotproduct` metric is the only metric that supports sparse-dense hybrid search.

### Using Enums for Type Safety

```python
from pinecone import Pinecone, ServerlessSpec, CloudProvider, AwsRegion, VectorType

pc = Pinecone(api_key='your_api_key_here')

index_config = pc.create_index(
    name='typed-index',
    dimension=1536,
    metric='cosine',
    spec=ServerlessSpec(
        cloud=CloudProvider.AWS,
        region=AwsRegion.US_EAST_1
    ),
    vector_type=VectorType.DENSE
)

print(index_config.host)
```

## Index Management

### Describe Index

Get index details including status, dimension, and host:

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

index_description = pc.describe_index('my-index')
print(index_description)
print(f"Host: {index_description.host}")
```

### List All Indexes

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

index_list = pc.list_indexes()
for index in index_list:
    print(index.name)
```

### Delete Index

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

pc.delete_index('my-index')
```

### Configure Index (Scale)

For pod-based indexes, adjust replicas and pod type:

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

pc.configure_index(
    name='pod-index',
    replicas=2,
    pod_type='p1.x2'
)
```

### Check if Index Exists

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

if pc.has_index('my-index'):
    print('Index exists')
else:
    print('Index does not exist')
```

## Connecting to an Index

### Basic Connection

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

# Get index host from description
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)
```

### Using gRPC Client

```python
from pinecone.grpc import PineconeGRPC as Pinecone

pc = Pinecone(api_key='your_api_key_here')

index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)
```

## Upserting Vectors

### Dense Vectors (Basic)

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

index.upsert(
    vectors=[
        ("vec1", [0.1, 0.2, 0.3, 0.4, 0.5]),
        ("vec2", [0.2, 0.3, 0.4, 0.5, 0.6]),
    ]
)
```

### Dense Vectors with Metadata

```python
index.upsert(
    vectors=[
        ("vec1", [0.1, 0.2, 0.3, 0.4, 0.5], {"genre": "comedy", "year": 2020}),
        ("vec2", [0.2, 0.3, 0.4, 0.5, 0.6], {"genre": "action", "year": 2021}),
    ]
)
```

### Using Dictionary Format

```python
index.upsert(
    vectors=[
        {
            "id": "vec1",
            "values": [0.1, 0.2, 0.3, 0.4, 0.5],
            "metadata": {"genre": "comedy", "year": 2020}
        },
        {
            "id": "vec2",
            "values": [0.2, 0.3, 0.4, 0.5, 0.6],
            "metadata": {"genre": "action", "year": 2021}
        }
    ]
)
```

### Namespaced Upsert

```python
index.upsert(
    vectors=[
        ("vec1", [0.1, 0.2, 0.3], {"category": "A"}),
    ],
    namespace='example-namespace'
)
```

### Batch Upsert (Large Datasets)

For upserting many vectors, batch them in groups of up to 1,000 records (max 2 MB per batch):

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

batch_size = 1000
vectors = []  # Your vector list

for i in range(0, len(vectors), batch_size):
    batch = vectors[i:i + batch_size]
    index.upsert(vectors=batch)
```

### Sparse-Dense Vectors (Hybrid Search)

```python
index.upsert(
    vectors=[
        {
            "id": "vec1",
            "values": [0.1, 0.2, 0.3],  # Dense vector
            "sparse_values": {
                "indices": [10, 45, 16],
                "values": [0.5, 0.5, 0.2]
            },
            "metadata": {"text": "original text content"}
        }
    ]
)
```

### Parallel Upsert with Threading

For better throughput, use parallel upserts:

```python
from concurrent.futures import ThreadPoolExecutor
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

def upsert_batch(batch):
    return index.upsert(vectors=batch)

batch_size = 1000
batches = [vectors[i:i + batch_size] for i in range(0, len(vectors), batch_size)]

with ThreadPoolExecutor(max_workers=10) as executor:
    results = list(executor.map(upsert_batch, batches))
```

## Querying Vectors

### Basic Query (Dense)

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

query_response = index.query(
    vector=[0.1, 0.2, 0.3, 0.4, 0.5],
    top_k=10,
    include_metadata=True,
    include_values=False
)

for match in query_response.matches:
    print(f"ID: {match.id}, Score: {match.score}, Metadata: {match.metadata}")
```

### Query with Metadata Filter

```python
query_response = index.query(
    vector=[0.1, 0.2, 0.3, 0.4, 0.5],
    top_k=5,
    filter={"genre": {"$eq": "comedy"}},
    include_metadata=True
)
```

### Complex Metadata Filters

```python
# Multiple conditions with $and
response = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=10,
    filter={
        "$and": [
            {"genre": {"$eq": "comedy"}},
            {"year": {"$gte": 2020}}
        ]
    },
    include_metadata=True
)

# Using $or operator
response2 = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=10,
    filter={
        "$or": [
            {"genre": {"$eq": "comedy"}},
            {"genre": {"$eq": "drama"}}
        ]
    },
    include_metadata=True
)

# Using $in for multiple values
response3 = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=10,
    filter={"genre": {"$in": ["comedy", "action", "drama"]}},
    include_metadata=True
)
```

### Query by ID

```python
query_response = index.query(
    id='vec1',
    top_k=10,
    include_metadata=True
)
```

### Namespaced Query

```python
query_response = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=5,
    namespace='example-namespace',
    include_metadata=True
)
```

### Hybrid Query (Sparse-Dense)

```python
query_response = index.query(
    vector=[0.1, 0.2, 0.3],  # Dense query vector
    sparse_vector={
        "indices": [10, 45, 16],
        "values": [0.5, 0.5, 0.2]
    },
    top_k=10,
    include_metadata=True
)
```

## Fetching Vectors

### Fetch by IDs

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

fetch_response = index.fetch(ids=['vec1', 'vec2', 'vec3'])
print(fetch_response.vectors)
```

### Fetch from Namespace

```python
fetch_response = index.fetch(
    ids=['vec1', 'vec2'],
    namespace='example-namespace'
)

for id, vector in fetch_response.vectors.items():
    print(f"ID: {id}, Values: {vector.values}, Metadata: {vector.metadata}")
```

## Updating Vectors

### Update Values

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

index.update(
    id='vec1',
    values=[0.9, 0.8, 0.7, 0.6, 0.5]
)
```

### Update Metadata

```python
index.update(
    id='vec1',
    set_metadata={"genre": "documentary", "year": 2023}
)
```

### Update with Namespace

```python
index.update(
    id='vec1',
    values=[0.1, 0.2, 0.3],
    set_metadata={"updated": True},
    namespace='example-namespace'
)
```

### Update Both Values and Metadata

```python
index.update(
    id='vec1',
    values=[0.9, 0.8, 0.7, 0.6, 0.5],
    set_metadata={"genre": "thriller", "updated": True}
)
```

## Deleting Vectors

### Delete by IDs

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

index.delete(ids=['vec1', 'vec2', 'vec3'])
```

### Delete with Metadata Filter

```python
index.delete(
    filter={"genre": {"$eq": "comedy"}}
)
```

### Delete All in Namespace

```python
index.delete(delete_all=True, namespace='example-namespace')
```

**Warning:** Deleting all records from a namespace will also delete the namespace itself. This operation is irreversible.

### Delete from Specific Namespace

```python
index.delete(
    ids=['vec1', 'vec2'],
    namespace='example-namespace'
)
```

## Index Statistics

### Get Index Stats

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

stats = index.describe_index_stats()
print(f"Total vectors: {stats.total_vector_count}")
print(f"Dimension: {stats.dimension}")
print(f"Namespaces: {stats.namespaces}")
```

### Get Stats with Filter

```python
stats = index.describe_index_stats(
    filter={"genre": {"$eq": "comedy"}}
)
print(f"Filtered vector count: {stats.total_vector_count}")
```

## Integrated Inference (Embeddings)

Pinecone provides hosted embedding models through the Inference API.

### Generate Embeddings

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

# Embed documents
embeddings = pc.inference.embed(
    model="multilingual-e5-large",
    inputs=[
        "Turkey is a classic meat to eat at American Thanksgiving.",
        "Many people enjoy the beautiful mosques in Turkey."
    ],
    parameters={
        "input_type": "passage",
        "truncate": "END"
    }
)

for embedding in embeddings:
    print(f"Values: {embedding['values'][:5]}...")  # Show first 5 dimensions
```

### Embed Queries

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

query_embedding = pc.inference.embed(
    model="multilingual-e5-large",
    inputs=["How should I prepare my turkey?"],
    parameters={
        "input_type": "query",
        "truncate": "END"
    }
)

# Use the embedding for querying
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

results = index.query(
    vector=query_embedding[0]['values'],
    top_k=10,
    include_metadata=True
)
```

### Using Enum for Models

```python
from pinecone import Pinecone, EmbedModel

pc = Pinecone(api_key='your_api_key_here')

embeddings = pc.inference.embed(
    model=EmbedModel.Multilingual_E5_Large,
    inputs=["Sample text"],
    parameters={"input_type": "passage"}
)
```

### Available Embedding Models

- `multilingual-e5-large`: Efficient dense embedding model trained on multilingual datasets
- `pinecone-sparse-english-v0`: Sparse embedding model for keyword or hybrid semantic/keyword search

## Sparse Embeddings for Hybrid Search

### Using Pinecone Sparse Embedding

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

# Generate sparse embeddings
sparse_embeddings = pc.inference.embed(
    model="pinecone-sparse-english-v0",
    inputs=[
        "Apple's first product was the Apple I computer.",
        "The company was founded in 1976."
    ],
    parameters={
        "input_type": "passage"
    }
)

# Generate dense embeddings
dense_embeddings = pc.inference.embed(
    model="multilingual-e5-large",
    inputs=[
        "Apple's first product was the Apple I computer.",
        "The company was founded in 1976."
    ],
    parameters={
        "input_type": "passage"
    }
)

# Upsert both
index_config = pc.describe_index('hybrid-index')
index = pc.Index(host=index_config.host)

for i, (dense, sparse) in enumerate(zip(dense_embeddings, sparse_embeddings)):
    index.upsert(
        vectors=[{
            "id": f"doc{i}",
            "values": dense['values'],
            "sparse_values": {
                "indices": sparse['indices'],
                "values": sparse['values']
            }
        }]
    )
```

## Namespaces

Namespaces partition records within an index and enable multitenancy.

### Create Namespace (Implicit)

Namespaces are created automatically when you upsert to them:

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

# Create namespace for customer 1
index.upsert(
    vectors=[("vec1", [0.1, 0.2, 0.3])],
    namespace='customer-1'
)

# Create namespace for customer 2
index.upsert(
    vectors=[("vec1", [0.4, 0.5, 0.6])],
    namespace='customer-2'
)
```

### List Namespaces

```python
stats = index.describe_index_stats()
print(list(stats.namespaces.keys()))
```

### Default Namespace

If no namespace is specified, vectors are stored in the `""` (empty string) namespace:

```python
# This uses the default namespace
index.upsert(vectors=[("vec1", [0.1, 0.2, 0.3])])

# Equivalent to
index.upsert(vectors=[("vec1", [0.1, 0.2, 0.3])], namespace='')
```

## Reranking

Use Pinecone's reranking models to reorder results by relevance.

### Rerank Results

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

reranked = pc.inference.rerank(
    model="bge-reranker-v2-m3",
    query="What is the capital of France?",
    documents=[
        {"id": "doc1", "text": "Paris is the capital of France."},
        {"id": "doc2", "text": "London is the capital of England."},
        {"id": "doc3", "text": "Berlin is the capital of Germany."}
    ],
    top_n=2,
    return_documents=True
)

for result in reranked.data:
    print(f"ID: {result['id']}, Score: {result['score']}, Text: {result.get('document', {}).get('text')}")
```

### Rerank Query Results

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

# First, query for candidates
query_response = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=50,
    include_metadata=True
)

# Prepare documents for reranking
documents = [
    {"id": match.id, "text": match.metadata.get('text', '')}
    for match in query_response.matches
]

# Rerank
reranked = pc.inference.rerank(
    model="bge-reranker-v2-m3",
    query="User's search query",
    documents=documents,
    top_n=10
)

print(reranked)
```

## Collections (Backups)

Collections are static snapshots of an index that can be used to create new indexes.

### Create Collection

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

pc.create_collection(
    name='my-collection',
    source='my-index'
)
```

### List Collections

```python
collections = pc.list_collections()
for collection in collections:
    print(collection.name)
```

### Describe Collection

```python
collection_info = pc.describe_collection('my-collection')
print(collection_info)
```

### Create Index from Collection

```python
from pinecone import Pinecone, ServerlessSpec

pc = Pinecone(api_key='your_api_key_here')

pc.create_index(
    name='new-index-from-backup',
    dimension=1536,
    metric='cosine',
    spec=ServerlessSpec(
        cloud='aws',
        region='us-west-2'
    ),
    source_collection='my-collection'
)
```

### Delete Collection

```python
pc.delete_collection('my-collection')
```

## Async Support

The SDK supports async/await for non-blocking operations.

### Installation with Async Support

```bash
pip install "pinecone[asyncio]"
```

### Async Initialization

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host, async_req=True)
```

### Async Upsert

```python
import asyncio
from pinecone import Pinecone

async def async_upsert():
    pc = Pinecone(api_key='your_api_key_here')
    index_config = pc.describe_index('my-index')
    index = pc.Index(host=index_config.host, async_req=True)

    await index.upsert(
        vectors=[("vec1", [0.1, 0.2, 0.3])]
    )

asyncio.run(async_upsert())
```

### Async Query

```python
import asyncio
from pinecone import Pinecone

async def async_query():
    pc = Pinecone(api_key='your_api_key_here')
    index_config = pc.describe_index('my-index')
    index = pc.Index(host=index_config.host, async_req=True)

    results = await index.query(
        vector=[0.1, 0.2, 0.3],
        top_k=10,
        include_metadata=True
    )
    return results

results = asyncio.run(async_query())
```

## Error Handling

### Basic Error Handling

```python
from pinecone import Pinecone
from pinecone.exceptions import PineconeException

pc = Pinecone(api_key='your_api_key_here')

try:
    index_config = pc.describe_index('my-index')
    index = pc.Index(host=index_config.host)

    results = index.query(
        vector=[0.1, 0.2, 0.3],
        top_k=10
    )
except PineconeException as e:
    print(f"Pinecone error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

### Handling Specific Errors

```python
from pinecone import Pinecone
from pinecone.exceptions import NotFoundException, UnauthorizedException

pc = Pinecone(api_key='your_api_key_here')

try:
    index_config = pc.describe_index('nonexistent-index')
except NotFoundException:
    print("Index not found")
except UnauthorizedException:
    print("Invalid API key")
```

## Performance Best Practices

### Use gRPC Client for High Throughput

```python
from pinecone.grpc import PineconeGRPC as Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)
```

The gRPC client offers better performance for high-concurrency scenarios.

### Batch Operations

Always batch upsert operations for better throughput:

```python
# Good: Batch upsert
vectors = generate_vectors(1000)
index.upsert(vectors=vectors)

# Bad: Individual upserts
for vector in vectors:
    index.upsert(vectors=[vector])  # Don't do this
```

### Parallel Requests

Use ThreadPoolExecutor for independent parallel operations:

```python
from concurrent.futures import ThreadPoolExecutor
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

def query_batch(query_vector):
    return index.query(vector=query_vector, top_k=5)

query_vectors = [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6], [0.7, 0.8, 0.9]]

with ThreadPoolExecutor(max_workers=10) as executor:
    results = list(executor.map(query_batch, query_vectors))
```

### Connection Reuse

Reuse the index connection instead of recreating it:

```python
# Good: Create once, reuse
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('my-index')
index = pc.Index(host=index_config.host)

for i in range(100):
    index.query(vector=get_vector(i), top_k=5)

# Bad: Recreate each time
for i in range(100):
    index_config = pc.describe_index('my-index')
    index = pc.Index(host=index_config.host)  # Don't do this
    index.query(vector=get_vector(i), top_k=5)
```

## Metadata Filtering Operators

Pinecone supports MongoDB-style query operators for metadata filtering:

- `$eq`: Equal to
- `$ne`: Not equal to
- `$gt`: Greater than
- `$gte`: Greater than or equal to
- `$lt`: Less than
- `$lte`: Less than or equal to
- `$in`: In array
- `$nin`: Not in array
- `$exists`: Field exists
- `$and`: Logical AND
- `$or`: Logical OR

### Example: Complex Filter

```python
results = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=10,
    filter={
        "$and": [
            {"year": {"$gte": 2020, "$lte": 2023}},
            {
                "$or": [
                    {"genre": {"$eq": "comedy"}},
                    {"genre": {"$eq": "drama"}}
                ]
            },
            {"rating": {"$exists": True}}
        ]
    },
    include_metadata=True
)
```

## Limits and Constraints

- **Max vectors per upsert**: 1,000 records (or 2 MB per batch)
- **Max vectors per upsert (with text)**: 96 records
- **Metadata size per vector**: 40 KB max
- **Record ID length**: 512 characters max
- **Dense vector dimensions**: Up to 20,000
- **Sparse vector non-zero values**: 2,048 max
- **Top-K query limit**: 10,000

## Common Patterns

### Pattern: Complete Workflow

```python
from pinecone import Pinecone, ServerlessSpec

# Initialize
pc = Pinecone(api_key='your_api_key_here')

# Create index
if not pc.has_index('example-index'):
    pc.create_index(
        name='example-index',
        dimension=1536,
        metric='cosine',
        spec=ServerlessSpec(cloud='aws', region='us-west-2')
    )

# Connect to index
index_config = pc.describe_index('example-index')
index = pc.Index(host=index_config.host)

# Upsert vectors
index.upsert(
    vectors=[
        ("doc1", [0.1] * 1536, {"title": "Document 1", "category": "A"}),
        ("doc2", [0.2] * 1536, {"title": "Document 2", "category": "B"}),
    ]
)

# Query
results = index.query(
    vector=[0.1] * 1536,
    top_k=5,
    filter={"category": {"$eq": "A"}},
    include_metadata=True
)

for match in results.matches:
    print(f"ID: {match.id}, Score: {match.score}, Metadata: {match.metadata}")
```

### Pattern: Multi-Tenant with Namespaces

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')
index_config = pc.describe_index('multi-tenant-index')
index = pc.Index(host=index_config.host)

# Tenant 1
index.upsert(
    vectors=[("vec1", [0.1, 0.2, 0.3], {"user": "user1"})],
    namespace='tenant-1'
)

# Tenant 2
index.upsert(
    vectors=[("vec1", [0.4, 0.5, 0.6], {"user": "user2"})],
    namespace='tenant-2'
)

# Query tenant-specific data
tenant1_results = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=5,
    namespace='tenant-1'
)

tenant2_results = index.query(
    vector=[0.1, 0.2, 0.3],
    top_k=5,
    namespace='tenant-2'
)
```

### Pattern: Hybrid Search with Integrated Inference

```python
from pinecone import Pinecone, ServerlessSpec

pc = Pinecone(api_key='your_api_key_here')

# Create hybrid index
if not pc.has_index('hybrid-index'):
    pc.create_index(
        name='hybrid-index',
        dimension=1024,
        metric='dotproduct',
        spec=ServerlessSpec(cloud='aws', region='us-east-1')
    )

# Connect to index
index_config = pc.describe_index('hybrid-index')
index = pc.Index(host=index_config.host)

# Prepare data
data = [
    "Turkey is a classic meat to eat at American Thanksgiving.",
    "Many people enjoy the beautiful mosques in Turkey."
]

# Generate dense embeddings
dense_embeddings = pc.inference.embed(
    model="multilingual-e5-large",
    inputs=data,
    parameters={"input_type": "passage"}
)

# Generate sparse embeddings
sparse_embeddings = pc.inference.embed(
    model="pinecone-sparse-english-v0",
    inputs=data,
    parameters={"input_type": "passage"}
)

# Upsert combined vectors
records = []
for i, (text, dense, sparse) in enumerate(zip(data, dense_embeddings, sparse_embeddings)):
    records.append({
        "id": f"doc{i}",
        "values": dense['values'],
        "sparse_values": {
            "indices": sparse['indices'],
            "values": sparse['values']
        },
        "metadata": {"text": text}
    })

index.upsert(vectors=records)

# Query with hybrid search
query = "How should I prepare my turkey?"

query_dense = pc.inference.embed(
    model="multilingual-e5-large",
    inputs=[query],
    parameters={"input_type": "query"}
)

query_sparse = pc.inference.embed(
    model="pinecone-sparse-english-v0",
    inputs=[query],
    parameters={"input_type": "query"}
)

# Apply alpha weighting (0.7 = 70% dense, 30% sparse)
alpha = 0.7

results = index.query(
    vector=[v * alpha for v in query_dense[0]['values']],
    sparse_vector={
        "indices": query_sparse[0]['indices'],
        "values": [v * (1 - alpha) for v in query_sparse[0]['values']]
    },
    top_k=10,
    include_metadata=True
)

for match in results.matches:
    print(f"Score: {match.score}, Text: {match.metadata['text']}")
```

### Pattern: RAG Pipeline

```python
from pinecone import Pinecone

pc = Pinecone(api_key='your_api_key_here')

# Embed and upsert documents
documents = [
    "Document 1 text content...",
    "Document 2 text content...",
    "Document 3 text content..."
]

embeddings = pc.inference.embed(
    model="multilingual-e5-large",
    inputs=documents,
    parameters={"input_type": "passage"}
)

index_config = pc.describe_index('rag-index')
index = pc.Index(host=index_config.host)

vectors = [
    (f"doc{i}", emb['values'], {"text": doc})
    for i, (doc, emb) in enumerate(zip(documents, embeddings))
]

index.upsert(vectors=vectors)

# Query and retrieve context
user_query = "What is the content about?"

query_embedding = pc.inference.embed(
    model="multilingual-e5-large",
    inputs=[user_query],
    parameters={"input_type": "query"}
)

results = index.query(
    vector=query_embedding[0]['values'],
    top_k=3,
    include_metadata=True
)

# Extract context for LLM
context = "\n\n".join([match.metadata['text'] for match in results.matches])
print(f"Context for LLM:\n{context}")
```

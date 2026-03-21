---
name: vector-db
description: "Weaviate Python SDK (v4) — use the official weaviate-client package for vector database operations"
metadata:
  languages: "python"
  versions: "4.17.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "weaviate,vector-db,search,embeddings,ai"
---

# Weaviate Python SDK (v4)

## Golden Rule

**ALWAYS use the official `weaviate-client` package (v4.x).**

```bash
pip install weaviate-client
```

**NEVER use:**
- `weaviate-python-client` (deprecated)
- The v3 client (deprecated as of 2024)
- Unofficial or outdated packages

The v4 client uses gRPC for 60-80% faster performance, typed classes with autocomplete, and is only compatible with Weaviate v1.23.6+.

---

## Installation

### Install the Client

```bash
pip install weaviate-client
```

### Environment Variables

Create a `.env` file:

```env
WEAVIATE_URL=https://your-instance.weaviate.network
WEAVIATE_API_KEY=your-api-key-here
OPENAI_API_KEY=your-openai-key-here
```

### Load Environment Variables

```python
import os
from dotenv import load_dotenv

load_dotenv()

WEAVIATE_URL = os.getenv("WEAVIATE_URL")
WEAVIATE_API_KEY = os.getenv("WEAVIATE_API_KEY")
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
```

---

## Initialization

### Connect to Weaviate Cloud

```python
import weaviate
from weaviate.classes.init import Auth

client = weaviate.connect_to_weaviate_cloud(
    cluster_url=WEAVIATE_URL,
    auth_credentials=Auth.api_key(WEAVIATE_API_KEY),
    headers={
        "X-OpenAI-Api-Key": OPENAI_API_KEY
    }
)

print("Connected to Weaviate")
```

### Connect to Local Instance

```python
import weaviate

client = weaviate.connect_to_local(
    host="localhost",
    port=8080,
    grpc_port=50051,
    headers={
        "X-OpenAI-Api-Key": OPENAI_API_KEY
    }
)
```

### Custom Connection

```python
import weaviate
from weaviate.classes.init import Auth, AdditionalConfig, Timeout

client = weaviate.connect_to_custom(
    http_host="weaviate.example.com",
    http_port=443,
    http_secure=True,
    grpc_host="grpc.weaviate.example.com",
    grpc_port=443,
    grpc_secure=True,
    auth_credentials=Auth.api_key(WEAVIATE_API_KEY),
    headers={
        "X-OpenAI-Api-Key": OPENAI_API_KEY
    },
    additional_config=AdditionalConfig(
        timeout=Timeout(init=30, query=60, insert=120)
    )
)
```

### Context Manager Pattern

```python
import weaviate
from weaviate.classes.init import Auth

with weaviate.connect_to_weaviate_cloud(
    cluster_url=WEAVIATE_URL,
    auth_credentials=Auth.api_key(WEAVIATE_API_KEY)
) as client:
    # Your code here
    collections = client.collections.list_all()
    print(collections)
# Connection automatically closed
```

### Close Connection

```python
client.close()
```

---

## Collections

### Create a Collection

#### Basic Collection

```python
from weaviate.classes.config import Property, DataType

client.collections.create(
    name="Article",
    properties=[
        Property(name="title", data_type=DataType.TEXT),
        Property(name="body", data_type=DataType.TEXT),
        Property(name="author", data_type=DataType.TEXT),
        Property(name="publish_date", data_type=DataType.DATE),
        Property(name="view_count", data_type=DataType.INT),
    ],
)
```

#### Collection with Vectorizer

```python
from weaviate.classes.config import Configure, Property, DataType

client.collections.create(
    name="Article",
    vectorizer_config=Configure.Vectorizer.text2vec_openai(
        model="text-embedding-3-small"
    ),
    properties=[
        Property(name="title", data_type=DataType.TEXT),
        Property(name="body", data_type=DataType.TEXT),
        Property(name="author", data_type=DataType.TEXT),
    ],
)
```

#### Collection with Multiple Named Vectors

```python
from weaviate.classes.config import Configure, Property, DataType

client.collections.create(
    name="MultiModalArticle",
    properties=[
        Property(name="title", data_type=DataType.TEXT),
        Property(name="body", data_type=DataType.TEXT),
        Property(name="image", data_type=DataType.BLOB),
    ],
    vectorizer_config=[
        Configure.NamedVectors.text2vec_openai(
            name="text_vector",
            source_properties=["title", "body"],
        ),
        Configure.NamedVectors.multi2vec_clip(
            name="image_vector",
            image_fields=["image"],
        ),
    ],
)
```

#### Collection with Generative Module

```python
from weaviate.classes.config import Configure, Property, DataType

client.collections.create(
    name="Article",
    vectorizer_config=Configure.Vectorizer.text2vec_openai(),
    generative_config=Configure.Generative.openai(
        model="gpt-4"
    ),
    properties=[
        Property(name="title", data_type=DataType.TEXT),
        Property(name="body", data_type=DataType.TEXT),
    ],
)
```

### List All Collections

```python
collections = client.collections.list_all()
for collection_name in collections:
    print(collection_name)
```

### Get a Collection Reference

```python
articles = client.collections.get("Article")
```

### Delete a Collection

```python
client.collections.delete("Article")
```

### Check if Collection Exists

```python
exists = client.collections.exists("Article")
print(f"Collection exists: {exists}")
```

---

## Data Types

### Available Data Types

```python
from weaviate.classes.config import DataType

# Text types
DataType.TEXT          # Single text value
DataType.TEXT_ARRAY    # List of text values

# Numeric types
DataType.NUMBER        # Float/double
DataType.INT           # Integer
DataType.INT_ARRAY     # List of integers
DataType.NUMBER_ARRAY  # List of numbers

# Boolean
DataType.BOOL          # True/false
DataType.BOOL_ARRAY    # List of booleans

# Date and UUID
DataType.DATE          # ISO 8601 date-time
DataType.DATE_ARRAY    # List of dates
DataType.UUID          # UUID
DataType.UUID_ARRAY    # List of UUIDs

# Binary data
DataType.BLOB          # Base64 encoded binary data
DataType.BLOB_ARRAY    # List of blobs

# Geolocation
DataType.GEO_COORDINATES # {"latitude": float, "longitude": float}

# Object reference
DataType.OBJECT        # Nested object
DataType.OBJECT_ARRAY  # List of nested objects
```

---

## Insert Data

### Insert Single Object

```python
from datetime import datetime
from weaviate.util import generate_uuid5

articles = client.collections.get("Article")

uuid = articles.data.insert(
    properties={
        "title": "Weaviate is Amazing",
        "body": "A comprehensive guide to vector databases",
        "author": "John Doe",
        "publish_date": datetime(2024, 1, 15).isoformat(),
        "view_count": 1250,
    }
)

print(f"Inserted object with UUID: {uuid}")
```

### Insert with Custom UUID

```python
from weaviate.util import generate_uuid5

my_uuid = generate_uuid5("my-unique-id")

articles.data.insert(
    properties={
        "title": "Custom UUID Article",
        "body": "Article with custom UUID",
    },
    uuid=my_uuid
)
```

### Insert with Vector

```python
uuid = articles.data.insert(
    properties={
        "title": "Manual Vector Article",
        "body": "Article with manually provided vector",
    },
    vector=[0.1, 0.2, 0.3, ...]  # Your embedding vector
)
```

### Batch Insert (Dynamic Batching)

```python
articles = client.collections.get("Article")

data_objects = [
    {
        "title": "First Article",
        "body": "Content of first article",
        "author": "Jane Smith",
    },
    {
        "title": "Second Article",
        "body": "Content of second article",
        "author": "Bob Johnson",
    },
    {
        "title": "Third Article",
        "body": "Content of third article",
        "author": "Alice Williams",
    },
]

with articles.batch.dynamic() as batch:
    for data_obj in data_objects:
        batch.add_object(properties=data_obj)

print(f"Batch insert completed")
```

### Batch Insert with Error Handling

```python
articles = client.collections.get("Article")

with articles.batch.dynamic() as batch:
    for data_obj in data_objects:
        batch.add_object(properties=data_obj)

    # Check for errors within the context
    if batch.number_errors > 0:
        print(f"Errors occurred: {batch.number_errors}")
        for failed in batch.failed_objects:
            print(f"Failed: {failed}")
```

### Batch Insert with Vectors

```python
data_with_vectors = [
    {"title": "Article 1", "body": "Content 1"},
    {"title": "Article 2", "body": "Content 2"},
]

vectors = [
    [0.1, 0.2, 0.3, ...],
    [0.4, 0.5, 0.6, ...],
]

with articles.batch.dynamic() as batch:
    for i, data_obj in enumerate(data_with_vectors):
        batch.add_object(
            properties=data_obj,
            vector=vectors[i]
        )
```

### Fixed Size Batching

```python
with articles.batch.fixed_size(batch_size=100) as batch:
    for data_obj in data_objects:
        batch.add_object(properties=data_obj)
```

### Rate Limited Batching

```python
with articles.batch.rate_limit(requests_per_minute=600) as batch:
    for data_obj in data_objects:
        batch.add_object(properties=data_obj)
```

---

## Query Data

### Fetch Objects (No Search)

```python
articles = client.collections.get("Article")

response = articles.query.fetch_objects(
    limit=10,
    offset=0
)

for obj in response.objects:
    print(obj.properties)
    print(f"UUID: {obj.uuid}")
```

### Fetch by ID

```python
obj = articles.query.fetch_object_by_id(uuid)
if obj:
    print(obj.properties)
```

### Fetch with Filters

```python
from weaviate.classes.query import Filter

response = articles.query.fetch_objects(
    filters=Filter.by_property("view_count").greater_than(1000),
    limit=5
)

for obj in response.objects:
    print(obj.properties)
```

---

## Vector Search

### near_text Search

```python
from weaviate.classes.query import MetadataQuery

articles = client.collections.get("Article")

response = articles.query.near_text(
    query="vector databases",
    limit=5,
    return_metadata=MetadataQuery(distance=True, score=True)
)

for obj in response.objects:
    print(obj.properties)
    print(f"Distance: {obj.metadata.distance}")
    print(f"Score: {obj.metadata.score}")
```

### near_text with Filters

```python
from weaviate.classes.query import Filter, MetadataQuery

response = articles.query.near_text(
    query="machine learning",
    limit=3,
    filters=Filter.by_property("author").equal("John Doe"),
    return_metadata=MetadataQuery(distance=True)
)

for obj in response.objects:
    print(obj.properties)
```

### near_vector Search

```python
from weaviate.classes.query import MetadataQuery

query_vector = [0.1, 0.2, 0.3, ...]  # Your query vector

response = articles.query.near_vector(
    near_vector=query_vector,
    limit=5,
    return_metadata=MetadataQuery(distance=True)
)

for obj in response.objects:
    print(obj.properties)
```

### near_object Search

```python
from weaviate.classes.query import MetadataQuery

response = articles.query.near_object(
    near_object=existing_uuid,
    limit=5,
    return_metadata=MetadataQuery(distance=True)
)

for obj in response.objects:
    print(obj.properties)
```

---

## Hybrid Search

Hybrid search combines keyword (BM25) and vector search.

### Basic Hybrid Search

```python
articles = client.collections.get("Article")

response = articles.query.hybrid(
    query="artificial intelligence",
    limit=10,
    alpha=0.5  # 0 = pure keyword, 1 = pure vector, 0.5 = balanced
)

for obj in response.objects:
    print(obj.properties)
```

### Hybrid Search with Filters

```python
from weaviate.classes.query import Filter, MetadataQuery
from datetime import datetime

response = articles.query.hybrid(
    query="deep learning",
    limit=5,
    alpha=0.75,
    filters=Filter.by_property("publish_date").greater_than(
        datetime(2024, 1, 1).isoformat()
    ),
    return_metadata=MetadataQuery(score=True)
)

for obj in response.objects:
    print(obj.properties)
    print(f"Score: {obj.metadata.score}")
```

### Hybrid Search with Vector

```python
response = articles.query.hybrid(
    query="machine learning",
    limit=5,
    alpha=0.5,
    vector=[0.1, 0.2, 0.3, ...]  # Optional manual vector
)
```

---

## Filters

### Basic Filters

```python
from weaviate.classes.query import Filter

articles = client.collections.get("Article")

# Equal
Filter.by_property("author").equal("John Doe")

# Not Equal
Filter.by_property("author").not_equal("Jane Smith")

# Greater Than
Filter.by_property("view_count").greater_than(1000)

# Greater Than or Equal
Filter.by_property("view_count").greater_or_equal(500)

# Less Than
Filter.by_property("view_count").less_than(2000)

# Less Than or Equal
Filter.by_property("view_count").less_or_equal(1500)

# Like (text contains)
Filter.by_property("title").like("*vector*")

# Contains Any
Filter.by_property("author").contains_any(["John Doe", "Jane Smith"])

# Is Null
Filter.by_property("author").is_null(True)
```

### Combining Filters (AND)

```python
from weaviate.classes.query import Filter

response = articles.query.fetch_objects(
    filters=(
        Filter.by_property("view_count").greater_than(1000) &
        Filter.by_property("author").equal("John Doe")
    ),
    limit=10
)
```

### Combining Filters (OR)

```python
from weaviate.classes.query import Filter

response = articles.query.fetch_objects(
    filters=(
        Filter.by_property("author").equal("John Doe") |
        Filter.by_property("author").equal("Jane Smith")
    ),
    limit=10
)
```

### Complex Filter Combinations

```python
from weaviate.classes.query import Filter
from datetime import datetime

response = articles.query.fetch_objects(
    filters=(
        (
            Filter.by_property("view_count").greater_than(500) &
            Filter.by_property("publish_date").greater_than(
                datetime(2024, 1, 1).isoformat()
            )
        ) |
        Filter.by_property("author").equal("John Doe")
    ),
    limit=10
)
```

---

## Generative Search (RAG)

Generative search pipes search results through an LLM.

### Single Result Generation

```python
articles = client.collections.get("Article")

response = articles.generate.near_text(
    query="AI trends",
    single_prompt="Summarize this article in one sentence: {title} - {body}",
    limit=5
)

for obj in response.objects:
    print("Article:", obj.properties)
    print("Generated:", obj.generated)
```

### Grouped Task Generation

```python
response = articles.generate.near_text(
    query="machine learning",
    grouped_task="Write a brief overview summarizing these articles about machine learning",
    limit=5
)

print("Generated summary:", response.generated)

for obj in response.objects:
    print("Source article:", obj.properties["title"])
```

### Both Single and Grouped

```python
response = articles.generate.near_text(
    query="neural networks",
    single_prompt="Explain this in simple terms: {body}",
    grouped_task="Create a comprehensive guide based on all these articles",
    limit=5
)

for obj in response.objects:
    print("Article:", obj.properties["title"])
    print("Individual generation:", obj.generated)

print("\nGrouped generation:", response.generated)
```

### Generative Search with Filters

```python
from weaviate.classes.query import Filter

response = articles.generate.near_text(
    query="databases",
    single_prompt="Summarize: {title}",
    limit=3,
    filters=Filter.by_property("author").equal("John Doe")
)
```

### Generate with near_vector

```python
query_vector = [0.1, 0.2, 0.3, ...]

response = articles.generate.near_vector(
    near_vector=query_vector,
    grouped_task="Create a summary of these related articles",
    limit=5
)

print(response.generated)
```

---

## Update Data

### Update Object

```python
articles = client.collections.get("Article")

articles.data.update(
    uuid=uuid,
    properties={
        "view_count": 2000,
        "title": "Updated Title",
    }
)
```

### Replace Object

Replace replaces the entire object (not a merge).

```python
from datetime import datetime

articles.data.replace(
    uuid=uuid,
    properties={
        "title": "Completely New Title",
        "body": "Completely new body",
        "author": "New Author",
        "publish_date": datetime.now().isoformat(),
        "view_count": 0,
    }
)
```

---

## Delete Data

### Delete by ID

```python
articles = client.collections.get("Article")

articles.data.delete_by_id(uuid)
print("Object deleted")
```

### Delete Many by Filter

```python
from weaviate.classes.query import Filter

response = articles.data.delete_many(
    where=Filter.by_property("view_count").less_than(100)
)

print(f"Deleted {response.successful} objects")
print(f"Failed: {response.failed}")
```

### Delete All Objects in Collection

```python
from weaviate.classes.query import Filter

articles.data.delete_many(
    where=Filter.by_property("title").like("*")
)
```

---

## Aggregations

### Count Objects

```python
articles = client.collections.get("Article")

response = articles.aggregate.over_all()
print(f"Total objects: {response.total_count}")
```

### Count with Filter

```python
from weaviate.classes.query import Filter

response = articles.aggregate.over_all(
    filters=Filter.by_property("author").equal("John Doe")
)

print(f"Objects by John Doe: {response.total_count}")
```

### Aggregate Numeric Properties

```python
from weaviate.classes.aggregate import Metrics

response = articles.aggregate.over_all(
    return_metrics=Metrics("view_count").maximum()
)

print(f"Max views: {response.properties['view_count'].maximum}")
```

### Multiple Aggregations

```python
from weaviate.classes.aggregate import Metrics

response = articles.aggregate.over_all(
    return_metrics=[
        Metrics("view_count").sum(),
        Metrics("view_count").average(),
        Metrics("view_count").minimum(),
        Metrics("view_count").maximum(),
    ]
)

view_stats = response.properties["view_count"]
print(f"Sum: {view_stats.sum}")
print(f"Average: {view_stats.average}")
print(f"Min: {view_stats.minimum}")
print(f"Max: {view_stats.maximum}")
```

---

## Multi-Tenancy

### Create Collection with Multi-Tenancy

```python
from weaviate.classes.config import Configure, Property, DataType

client.collections.create(
    name="Article",
    multi_tenancy_config=Configure.multi_tenancy(enabled=True),
    properties=[
        Property(name="title", data_type=DataType.TEXT),
        Property(name="body", data_type=DataType.TEXT),
    ],
)
```

### Add Tenants

```python
from weaviate.classes.tenants import Tenant

articles = client.collections.get("Article")

articles.tenants.create(
    tenants=[
        Tenant(name="tenant1"),
        Tenant(name="tenant2"),
        Tenant(name="tenant3"),
    ]
)
```

### Get Tenant Reference

```python
tenant1_articles = articles.with_tenant("tenant1")
```

### Insert Data for Tenant

```python
tenant1_articles.data.insert(
    properties={
        "title": "Tenant 1 Article",
        "body": "This belongs to tenant 1",
    }
)
```

### Query Tenant Data

```python
response = tenant1_articles.query.fetch_objects(limit=10)

for obj in response.objects:
    print(obj.properties)
```

### List Tenants

```python
tenants = articles.tenants.get()
for tenant in tenants:
    print(tenant.name)
```

### Remove Tenant

```python
articles.tenants.remove(["tenant3"])
```

---

## Cross-References

### Define Collection with Reference

```python
from weaviate.classes.config import Property, DataType, ReferenceProperty

# Create Author collection
client.collections.create(
    name="Author",
    properties=[
        Property(name="name", data_type=DataType.TEXT),
        Property(name="bio", data_type=DataType.TEXT),
    ],
)

# Create Article collection with reference to Author
client.collections.create(
    name="Article",
    properties=[
        Property(name="title", data_type=DataType.TEXT),
        Property(name="body", data_type=DataType.TEXT),
    ],
    references=[
        ReferenceProperty(
            name="has_author",
            target_collection="Author"
        )
    ],
)
```

### Insert with Reference

```python
authors = client.collections.get("Author")
articles = client.collections.get("Article")

# Insert author
author_uuid = authors.data.insert(
    properties={
        "name": "John Doe",
        "bio": "Expert in AI",
    }
)

# Insert article with reference
articles.data.insert(
    properties={
        "title": "AI Article",
        "body": "Content about AI",
    },
    references={
        "has_author": author_uuid
    }
)
```

### Add Reference to Existing Object

```python
articles.data.reference_add(
    from_uuid=article_uuid,
    from_property="has_author",
    to=author_uuid
)
```

### Query with References

```python
from weaviate.classes.query import QueryReference

response = articles.query.fetch_objects(
    limit=10,
    return_references=QueryReference(
        link_on="has_author",
        return_properties=["name", "bio"]
    )
)

for obj in response.objects:
    print("Article:", obj.properties)
    if obj.references and "has_author" in obj.references:
        print("Author:", obj.references["has_author"].objects[0].properties)
```

### Delete Reference

```python
articles.data.reference_delete(
    from_uuid=article_uuid,
    from_property="has_author",
    to=author_uuid
)
```

---

## Advanced Configurations

### Collection with Custom Vectorizer Settings

```python
from weaviate.classes.config import (
    Configure,
    Property,
    DataType,
    VectorDistances
)

client.collections.create(
    name="Article",
    vectorizer_config=Configure.Vectorizer.text2vec_openai(
        model="text-embedding-3-large",
        dimensions=1536,
        vectorize_collection_name=False,
    ),
    vector_index_config=Configure.VectorIndex.hnsw(
        distance_metric=VectorDistances.COSINE,
        ef_construction=128,
        max_connections=64,
    ),
    properties=[
        Property(
            name="title",
            data_type=DataType.TEXT,
            vectorize_property_name=False,
            skip_vectorization=False,
        ),
        Property(
            name="body",
            data_type=DataType.TEXT,
        ),
        Property(
            name="metadata",
            data_type=DataType.TEXT,
            skip_vectorization=True,  # Don't vectorize this property
        ),
    ],
)
```

### Collection with Inverted Index Config

```python
from weaviate.classes.config import Configure, Property, DataType, Tokenization

client.collections.create(
    name="Article",
    properties=[
        Property(
            name="title",
            data_type=DataType.TEXT,
            tokenization=Tokenization.WORD,
            index_filterable=True,
            index_searchable=True,
        ),
        Property(
            name="sku",
            data_type=DataType.TEXT,
            tokenization=Tokenization.FIELD,  # Exact match only
            index_filterable=True,
            index_searchable=False,
        ),
    ],
    inverted_index_config=Configure.inverted_index(
        index_timestamps=True,
        index_null_state=True,
        index_property_length=True,
    ),
)
```

---

## Error Handling

### Try-Except Pattern

```python
import weaviate
from weaviate.classes.init import Auth

try:
    client = weaviate.connect_to_weaviate_cloud(
        cluster_url=WEAVIATE_URL,
        auth_credentials=Auth.api_key(WEAVIATE_API_KEY)
    )

    articles = client.collections.get("Article")

    response = articles.query.near_text(
        query="AI",
        limit=5
    )

    for obj in response.objects:
        print(obj.properties)

    client.close()

except Exception as e:
    print(f"Error: {e}")
```

### Batch Insert Error Handling

```python
articles = client.collections.get("Article")

with articles.batch.dynamic() as batch:
    for data_obj in data_objects:
        batch.add_object(properties=data_obj)

    if batch.number_errors > 0:
        print(f"Number of errors: {batch.number_errors}")

        for failed_obj in batch.failed_objects:
            print(f"Failed object: {failed_obj.object_}")
            print(f"Error message: {failed_obj.message}")
```

---

## Complete Example

```python
import weaviate
from weaviate.classes.init import Auth
from weaviate.classes.config import Configure, Property, DataType
from weaviate.classes.query import Filter, MetadataQuery
from weaviate.classes.aggregate import Metrics
from datetime import datetime
import os

# Connect
client = weaviate.connect_to_weaviate_cloud(
    cluster_url=os.getenv("WEAVIATE_URL"),
    auth_credentials=Auth.api_key(os.getenv("WEAVIATE_API_KEY")),
    headers={
        "X-OpenAI-Api-Key": os.getenv("OPENAI_API_KEY")
    }
)

print("Connected to Weaviate")

# Create collection
client.collections.create(
    name="Article",
    vectorizer_config=Configure.Vectorizer.text2vec_openai(
        model="text-embedding-3-small"
    ),
    generative_config=Configure.Generative.openai(
        model="gpt-4"
    ),
    properties=[
        Property(name="title", data_type=DataType.TEXT),
        Property(name="body", data_type=DataType.TEXT),
        Property(name="author", data_type=DataType.TEXT),
        Property(name="publish_date", data_type=DataType.DATE),
        Property(name="view_count", data_type=DataType.INT),
    ],
)

print("Collection created")

# Get collection reference
articles = client.collections.get("Article")

# Insert data
data_objects = [
    {
        "title": "Vector Databases Explained",
        "body": "A comprehensive guide to understanding vector databases and their applications in modern AI systems.",
        "author": "John Doe",
        "publish_date": datetime(2024, 1, 15).isoformat(),
        "view_count": 1250,
    },
    {
        "title": "Machine Learning in Production",
        "body": "Best practices for deploying machine learning models in production environments.",
        "author": "Jane Smith",
        "publish_date": datetime(2024, 2, 20).isoformat(),
        "view_count": 2100,
    },
    {
        "title": "Introduction to RAG",
        "body": "Retrieval Augmented Generation combines search with language models for better responses.",
        "author": "John Doe",
        "publish_date": datetime(2024, 3, 10).isoformat(),
        "view_count": 890,
    },
]

with articles.batch.dynamic() as batch:
    for data_obj in data_objects:
        batch.add_object(properties=data_obj)

print(f"Batch insert completed")

# Vector search
print("\n--- Vector Search ---")
search_response = articles.query.near_text(
    query="AI and databases",
    limit=2,
    return_metadata=MetadataQuery(distance=True, score=True)
)

for obj in search_response.objects:
    print(f"\nTitle: {obj.properties['title']}")
    print(f"Author: {obj.properties['author']}")
    print(f"Score: {obj.metadata.score}")

# Filtered search
print("\n--- Filtered Search ---")
filtered_response = articles.query.near_text(
    query="machine learning",
    limit=3,
    filters=Filter.by_property("view_count").greater_than(1000)
)

for obj in filtered_response.objects:
    print(f"\nTitle: {obj.properties['title']}")
    print(f"Views: {obj.properties['view_count']}")

# Hybrid search
print("\n--- Hybrid Search ---")
hybrid_response = articles.query.hybrid(
    query="production deployment",
    limit=2,
    alpha=0.5
)

for obj in hybrid_response.objects:
    print(f"\nTitle: {obj.properties['title']}")

# Generative search (RAG)
print("\n--- Generative Search ---")
rag_response = articles.generate.near_text(
    query="vector databases",
    single_prompt="Summarize this article in one sentence: {title}",
    grouped_task="Write a brief paragraph about these articles",
    limit=2
)

print("\nGrouped generation:")
print(rag_response.generated)

for obj in rag_response.objects:
    print(f"\n{obj.properties['title']}")
    print(f"Summary: {obj.generated}")

# Aggregate
print("\n--- Aggregations ---")
agg_response = articles.aggregate.over_all(
    return_metrics=[
        Metrics("view_count").sum(),
        Metrics("view_count").average(),
        Metrics("view_count").maximum(),
    ]
)

view_stats = agg_response.properties["view_count"]
print(f"Total articles: {agg_response.total_count}")
print(f"Total views: {view_stats.sum}")
print(f"Average views: {view_stats.average}")
print(f"Max views: {view_stats.maximum}")

# Update
response = articles.query.fetch_objects(limit=1)
if response.objects:
    first_uuid = response.objects[0].uuid
    articles.data.update(
        uuid=first_uuid,
        properties={"view_count": 1500}
    )
    print("\nUpdated first article view count")

# Delete by filter
delete_response = articles.data.delete_many(
    where=Filter.by_property("view_count").less_than(900)
)
print(f"\nDeleted {delete_response.successful} low-view articles")

# Cleanup
client.collections.delete("Article")
print("\nCollection deleted")

client.close()
print("Connection closed")
```

---

## Type Hints

```python
from typing import List, Dict, Any
import weaviate
from weaviate.collections.collection import Collection
from weaviate.collections.classes.grpc import QueryReturn

# Collection reference with type hint
articles: Collection = client.collections.get("Article")

# Query result type
response: QueryReturn = articles.query.near_text(
    query="AI",
    limit=5
)

# Iterate with type hints
for obj in response.objects:
    properties: Dict[str, Any] = obj.properties
    print(properties)
```

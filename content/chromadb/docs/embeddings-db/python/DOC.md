---
name: embeddings-db
description: "ChromaDB Python SDK for vector embeddings and AI-powered search"
metadata:
  languages: "python"
  versions: "1.2.1"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "chromadb,embeddings,vector-db,ai,search"
---

# ChromaDB Python SDK - v1.2.1

## Golden Rule

**ALWAYS use the official `chromadb` package (v1.2.1 or later) for Python projects.**

```bash
pip install chromadb
```

**DO NOT use:**
- Deprecated packages like `chromadb-client`
- Old versions below 1.0
- Community wrappers that may be outdated

ChromaDB is the official AI-native open-source vector database. It handles embeddings, indexing, and vector similarity search automatically.

**Requires Python >= 3.9**

---

## Installation

### Using pip

```bash
pip install chromadb
```

### Using Poetry

```bash
poetry add chromadb
```

### Using uv

```bash
uv pip install chromadb
```

### Install with Specific Version

```bash
pip install chromadb==1.2.1
```

---

## Initialization

### Ephemeral Client (In-Memory)

```python
import chromadb

client = chromadb.EphemeralClient()
```

Use for experimentation, testing, and prototyping. Data is lost when the process ends.

### Persistent Client (Local Storage)

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma_data")
```

Stores data locally at the specified path. Creates the directory if it doesn't exist.

### Persistent Client with Default Path

```python
import chromadb

client = chromadb.PersistentClient()
```

Defaults to `./chroma` in the current working directory.

### HTTP Client (Remote Server)

```python
import chromadb

client = chromadb.HttpClient(host="localhost", port=8000)
```

Connects to a remote ChromaDB server.

### HTTP Client with Custom Configuration

```python
import chromadb
from chromadb.config import Settings

client = chromadb.HttpClient(
    host="localhost",
    port=8000,
    ssl=False,
    headers={"Authorization": "Bearer token"},
    settings=Settings(),
    tenant="default_tenant",
    database="default_database"
)
```

---

## Running ChromaDB Server

### Local Server

```bash
chroma run --path ./chroma_data
```

Starts server on `http://localhost:8000`

### Docker

```bash
docker pull chromadb/chroma
docker run -p 8000:8000 chromadb/chroma
```

---

## Collections

### Create a Collection

```python
collection = client.create_collection(name="my_collection")
```

### Create Collection with Distance Metric

```python
collection = client.create_collection(
    name="my_collection",
    metadata={"hnsw:space": "cosine"}
)
```

**Distance Metrics:**
- `cosine`: Cosine similarity (best for text, normalized vectors)
- `l2`: Euclidean/L2 distance (default, sensitive to magnitude)
- `ip`: Inner product (for recommendation systems)

### Get an Existing Collection

```python
collection = client.get_collection(name="my_collection")
```

### Get or Create Collection

```python
collection = client.get_or_create_collection(name="my_collection")
```

### List All Collections

```python
collections = client.list_collections()
for col in collections:
    print(col.name)
```

### Delete a Collection

```python
client.delete_collection(name="my_collection")
```

---

## Adding Data

### Add Documents (Auto-Embedding)

```python
collection.add(
    ids=["id1", "id2", "id3"],
    documents=[
        "This is a document about pineapples",
        "This is a document about oranges",
        "This is a document about apples"
    ]
)
```

ChromaDB automatically embeds the documents using the default embedding function.

### Add with Metadata

```python
collection.add(
    ids=["id1", "id2", "id3"],
    documents=[
        "This is a document about pineapples",
        "This is a document about oranges",
        "This is a document about apples"
    ],
    metadatas=[
        {"category": "tropical", "color": "yellow"},
        {"category": "citrus", "color": "orange"},
        {"category": "temperate", "color": "red"}
    ]
)
```

### Add with Custom Embeddings

```python
collection.add(
    ids=["id1", "id2"],
    embeddings=[
        [1.5, 2.9, 3.4, 1.2, 0.8],
        [9.8, 2.3, 2.9, 4.1, 3.3]
    ],
    documents=["Document one", "Document two"],
    metadatas=[
        {"source": "manual"},
        {"source": "manual"}
    ]
)
```

### Batch Adding (Large Datasets)

```python
batch_size = 5000
for i in range(0, len(documents), batch_size):
    batch_docs = documents[i:i + batch_size]
    batch_ids = [f"id{j}" for j in range(i, i + len(batch_docs))]

    collection.add(
        ids=batch_ids,
        documents=batch_docs
    )
```

ChromaDB supports adding up to 100k+ documents at once.

---

## Querying Data

### Query with Text (Auto-Embedding)

```python
results = collection.query(
    query_texts=["What fruits are tropical?"],
    n_results=2
)

print(results)
```

**Response Structure:**

```python
{
    'ids': [['id1', 'id2']],
    'distances': [[0.1234, 0.5678]],
    'documents': [['This is a document about pineapples', 'This is...']],
    'metadatas': [[{'category': 'tropical', 'color': 'yellow'}, {...}]],
    'embeddings': None  # Not included by default
}
```

### Query with Multiple Texts

```python
results = collection.query(
    query_texts=[
        "What fruits are tropical?",
        "What fruits are citrus?"
    ],
    n_results=2
)
```

Returns `n_results` for each query text.

### Query with Custom Embeddings

```python
results = collection.query(
    query_embeddings=[[1.5, 2.9, 3.4, 1.2, 0.8]],
    n_results=3
)
```

### Query with Metadata Filters

```python
results = collection.query(
    query_texts=["What fruits are available?"],
    n_results=5,
    where={"category": "tropical"}
)
```

### Complex Metadata Filtering

```python
# Using $or operator
results = collection.query(
    query_texts=["Find fruits"],
    n_results=5,
    where={
        "$or": [
            {"category": "tropical"},
            {"category": "citrus"}
        ]
    }
)

# Using $and operator
results = collection.query(
    query_texts=["Find fruits"],
    n_results=5,
    where={
        "$and": [
            {"category": "tropical"},
            {"color": "yellow"}
        ]
    }
)

# Using comparison operators
results = collection.query(
    query_texts=["Find items"],
    n_results=5,
    where={
        "price": {"$gt": 10}  # $gt, $gte, $lt, $lte, $ne, $eq
    }
)
```

### Query with Document Content Filters

```python
results = collection.query(
    query_texts=["Find documents"],
    n_results=5,
    where_document={"$contains": "pineapple"}
)

# Using $not_contains
results = collection.query(
    query_texts=["Find documents"],
    n_results=5,
    where_document={"$not_contains": "apple"}
)
```

### Query with Include Options

```python
results = collection.query(
    query_texts=["What fruits are tropical?"],
    n_results=2,
    include=["documents", "metadatas", "distances", "embeddings"]
)
```

**Include Options:**
- `documents`: The document text (included by default)
- `metadatas`: Metadata for each document (included by default)
- `distances`: Distance/similarity scores (included by default)
- `embeddings`: Vector embeddings (not included by default for performance)

---

## Getting Data

### Get Documents by IDs

```python
results = collection.get(
    ids=["id1", "id2"]
)

print(results)
```

### Get All Documents

```python
results = collection.get()
```

Returns all documents in the collection.

### Get with Metadata Filter

```python
results = collection.get(
    where={"category": "tropical"}
)
```

### Get with Document Filter

```python
results = collection.get(
    where_document={"$contains": "pineapple"}
)
```

### Get with Limit and Offset

```python
results = collection.get(
    limit=10,
    offset=20
)
```

### Get with Include Options

```python
results = collection.get(
    ids=["id1", "id2"],
    include=["documents", "metadatas", "embeddings"]
)
```

---

## Updating Data

### Update Documents

```python
collection.update(
    ids=["id1", "id2"],
    documents=[
        "Updated document about pineapples",
        "Updated document about oranges"
    ],
    metadatas=[
        {"category": "tropical", "color": "yellow", "updated": True},
        {"category": "citrus", "color": "orange", "updated": True}
    ]
)
```

### Update with Custom Embeddings

```python
collection.update(
    ids=["id1"],
    embeddings=[[1.1, 2.2, 3.3, 4.4, 5.5]],
    documents=["Updated document"],
    metadatas=[{"source": "updated"}]
)
```

---

## Upsert (Add or Update)

### Upsert Documents

```python
collection.upsert(
    ids=["id1", "id2", "id3"],
    documents=[
        "Document one - may be new or updated",
        "Document two - may be new or updated",
        "Document three - may be new or updated"
    ],
    metadatas=[
        {"version": 2},
        {"version": 2},
        {"version": 1}
    ]
)
```

If the ID exists, it updates the document. If not, it adds it as new.

---

## Deleting Data

### Delete by IDs

```python
collection.delete(
    ids=["id1", "id2"]
)
```

### Delete with Metadata Filter

```python
collection.delete(
    where={"category": "tropical"}
)
```

### Delete with Document Filter

```python
collection.delete(
    where_document={"$contains": "deprecated"}
)
```

### Delete All Documents (Keep Collection)

```python
collection.delete()
```

---

## Collection Utilities

### Count Documents

```python
count = collection.count()
print(f"Total documents: {count}")
```

### Peek at First Documents

```python
first_docs = collection.peek(limit=5)
print(first_docs)
```

Returns the first 5 documents in the collection.

### Modify Collection Metadata

```python
collection.modify(
    metadata={
        "description": "Collection of fruit documents",
        "version": "1.0"
    }
)
```

---

## Embedding Functions

### Using Default Embedding Function

```python
import chromadb

client = chromadb.PersistentClient()
collection = client.create_collection(name="my_collection")
```

By default, ChromaDB uses the Sentence Transformers `all-MiniLM-L6-v2` model.

### Using OpenAI Embeddings

```python
import chromadb
from chromadb.utils import embedding_functions

openai_ef = embedding_functions.OpenAIEmbeddingFunction(
    api_key="your-openai-api-key",
    model_name="text-embedding-3-small"
)

collection = client.create_collection(
    name="openai_collection",
    embedding_function=openai_ef
)
```

**Available OpenAI Models:**
- `text-embedding-3-small`
- `text-embedding-3-large`
- `text-embedding-ada-002`

### Using Cohere Embeddings

```python
import chromadb
from chromadb.utils import embedding_functions

cohere_ef = embedding_functions.CohereEmbeddingFunction(
    api_key="your-cohere-api-key",
    model_name="embed-english-v3.0"
)

collection = client.create_collection(
    name="cohere_collection",
    embedding_function=cohere_ef
)
```

**Available Cohere Models:**
- `embed-english-v3.0`
- `embed-multilingual-v3.0`
- `embed-english-light-v3.0`

### Using Hugging Face Embeddings

```python
import chromadb
from chromadb.utils import embedding_functions

huggingface_ef = embedding_functions.HuggingFaceEmbeddingFunction(
    api_key="your-hf-api-key",
    model_name="sentence-transformers/all-MiniLM-L6-v2"
)

collection = client.create_collection(
    name="hf_collection",
    embedding_function=huggingface_ef
)
```

### Using Sentence Transformers (Local)

```python
import chromadb
from chromadb.utils import embedding_functions

sentence_transformer_ef = embedding_functions.SentenceTransformerEmbeddingFunction(
    model_name="all-MiniLM-L6-v2"
)

collection = client.create_collection(
    name="local_collection",
    embedding_function=sentence_transformer_ef
)
```

### Using Ollama Embeddings (Local)

```python
import chromadb
from chromadb.utils import embedding_functions

ollama_ef = embedding_functions.OllamaEmbeddingFunction(
    url="http://localhost:11434/api/embeddings",
    model_name="llama2"
)

collection = client.create_collection(
    name="ollama_collection",
    embedding_function=ollama_ef
)
```

### Custom Embedding Function

```python
import chromadb
from chromadb import Documents, EmbeddingFunction, Embeddings

class CustomEmbeddingFunction(EmbeddingFunction):
    def __call__(self, input: Documents) -> Embeddings:
        # Your custom embedding logic here
        embeddings = []
        for doc in input:
            # Example: simple character code embedding (replace with real model)
            embedding = [ord(c) / 255.0 for c in doc[:384]]
            # Pad to fixed length
            embedding.extend([0.0] * (384 - len(embedding)))
            embeddings.append(embedding)
        return embeddings

custom_ef = CustomEmbeddingFunction()
collection = client.create_collection(
    name="custom_collection",
    embedding_function=custom_ef
)
```

---

## Advanced Client Configuration

### PersistentClient with Full Options

```python
import chromadb
from chromadb.config import Settings

client = chromadb.PersistentClient(
    path="./my_chroma_data",
    settings=Settings(
        anonymized_telemetry=False,
        allow_reset=True
    ),
    tenant="default_tenant",
    database="default_database"
)
```

### HttpClient with Authentication

```python
import chromadb
from chromadb.config import Settings

client = chromadb.HttpClient(
    host="my-chroma-server.com",
    port=8000,
    ssl=True,
    headers={"Authorization": "Bearer my-token"},
    settings=Settings(
        chroma_client_auth_provider="chromadb.auth.token_authn.TokenAuthClientProvider",
        chroma_client_auth_credentials="my-token"
    )
)
```

### Multi-Tenancy Setup

```python
import chromadb

client = chromadb.HttpClient(host="localhost", port=8000)

# Create a new tenant and database
client.create_tenant(name="acme_corp")
client.create_database(name="production", tenant="acme_corp")

# Connect to specific tenant/database
tenant_client = chromadb.HttpClient(
    host="localhost",
    port=8000,
    tenant="acme_corp",
    database="production"
)
```

---

## Complete Example: Document Search System

```python
import chromadb

def main():
    # Initialize client
    client = chromadb.PersistentClient(path="./search_db")

    # Create or get collection
    collection = client.get_or_create_collection(
        name="knowledge_base",
        metadata={"hnsw:space": "cosine"}
    )

    # Add documents
    collection.add(
        ids=["doc1", "doc2", "doc3", "doc4"],
        documents=[
            "The quick brown fox jumps over the lazy dog",
            "Machine learning is a subset of artificial intelligence",
            "Python is a popular programming language",
            "ChromaDB is a vector database for AI applications"
        ],
        metadatas=[
            {"category": "phrases", "language": "english"},
            {"category": "ai", "language": "english"},
            {"category": "programming", "language": "english"},
            {"category": "database", "language": "english"}
        ]
    )

    # Query the collection
    results = collection.query(
        query_texts=["What is AI?"],
        n_results=2,
        where={"category": "ai"}
    )

    print("Search Results:")
    print(results["documents"][0])
    print(results["metadatas"][0])
    print(results["distances"][0])

    # Get document count
    count = collection.count()
    print(f"Total documents: {count}")

    # Update a document
    collection.update(
        ids=["doc2"],
        documents=["Machine learning is a powerful subset of artificial intelligence"],
        metadatas=[{"category": "ai", "language": "english", "updated": True}]
    )

    # Delete documents
    collection.delete(ids=["doc4"])

if __name__ == "__main__":
    main()
```

---

## Complete Example: Semantic Search with OpenAI

```python
import chromadb
from chromadb.utils import embedding_functions
import os
from dotenv import load_dotenv

load_dotenv()

def semantic_search():
    client = chromadb.PersistentClient(path="./semantic_db")

    openai_ef = embedding_functions.OpenAIEmbeddingFunction(
        api_key=os.getenv("OPENAI_API_KEY"),
        model_name="text-embedding-3-small"
    )

    collection = client.get_or_create_collection(
        name="articles",
        embedding_function=openai_ef,
        metadata={"hnsw:space": "cosine"}
    )

    # Add articles
    collection.add(
        ids=["art1", "art2", "art3"],
        documents=[
            "Climate change is affecting global weather patterns",
            "New breakthrough in quantum computing announced",
            "The future of renewable energy looks promising"
        ],
        metadatas=[
            {"topic": "environment", "date": "2024-01-15"},
            {"topic": "technology", "date": "2024-01-16"},
            {"topic": "energy", "date": "2024-01-17"}
        ]
    )

    # Search for relevant articles
    results = collection.query(
        query_texts=["Tell me about environmental issues"],
        n_results=3
    )

    for idx, doc in enumerate(results["documents"][0]):
        print(f"Result {idx + 1}:")
        print(f"Document: {doc}")
        print(f"Metadata: {results['metadatas'][0][idx]}")
        print(f"Distance: {results['distances'][0][idx]}")
        print("---")

if __name__ == "__main__":
    semantic_search()
```

---

## Complete Example: RAG (Retrieval-Augmented Generation)

```python
import chromadb
from openai import OpenAI
import os
from dotenv import load_dotenv

load_dotenv()

def rag_example():
    # Initialize ChromaDB
    chroma_client = chromadb.PersistentClient(path="./rag_db")
    collection = chroma_client.get_or_create_collection(
        name="company_docs",
        metadata={"hnsw:space": "cosine"}
    )

    # Add company knowledge base
    collection.add(
        ids=["policy1", "policy2", "policy3"],
        documents=[
            "Our company offers 20 days of paid vacation per year",
            "Remote work is available 3 days per week",
            "Health insurance includes dental and vision coverage"
        ],
        metadatas=[
            {"type": "policy", "category": "time-off"},
            {"type": "policy", "category": "work-arrangement"},
            {"type": "policy", "category": "benefits"}
        ]
    )

    # User question
    question = "How many vacation days do I get?"

    # Retrieve relevant context
    search_results = collection.query(
        query_texts=[question],
        n_results=2
    )

    context = "\n".join(search_results["documents"][0])

    # Generate answer with OpenAI
    openai_client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))

    completion = openai_client.chat.completions.create(
        model="gpt-4",
        messages=[
            {
                "role": "system",
                "content": "Answer the question based on the context provided."
            },
            {
                "role": "user",
                "content": f"Context:\n{context}\n\nQuestion: {question}"
            }
        ]
    )

    print("Answer:", completion.choices[0].message.content)

if __name__ == "__main__":
    rag_example()
```

---

## Complete Example: Streaming Large Dataset

```python
import chromadb
from typing import List, Generator

def document_generator(file_path: str) -> Generator[str, None, None]:
    """Generator to stream documents from a file."""
    with open(file_path, 'r') as f:
        for line in f:
            yield line.strip()

def stream_add_documents(collection, doc_gen: Generator, batch_size: int = 1000):
    """Add documents in batches from a generator."""
    batch_ids = []
    batch_docs = []
    counter = 0

    for doc in doc_gen:
        batch_ids.append(f"doc{counter}")
        batch_docs.append(doc)
        counter += 1

        if len(batch_docs) >= batch_size:
            collection.add(
                ids=batch_ids,
                documents=batch_docs
            )
            print(f"Added {counter} documents...")
            batch_ids = []
            batch_docs = []

    # Add remaining documents
    if batch_docs:
        collection.add(
            ids=batch_ids,
            documents=batch_docs
        )
        print(f"Added final {len(batch_docs)} documents. Total: {counter}")

def main():
    client = chromadb.PersistentClient(path="./large_db")
    collection = client.get_or_create_collection(name="large_collection")

    # Stream and add documents
    doc_gen = document_generator("large_dataset.txt")
    stream_add_documents(collection, doc_gen, batch_size=5000)

    print(f"Total documents in collection: {collection.count()}")

if __name__ == "__main__":
    main()
```

---

## Environment Variables

### .env File Setup

```bash
# ChromaDB Server
CHROMA_HOST=localhost
CHROMA_PORT=8000

# Authentication (if required)
CHROMA_AUTH_TOKEN=your-auth-token

# Embedding API Keys
OPENAI_API_KEY=sk-...
COHERE_API_KEY=...
HF_API_KEY=...
```

### Using Environment Variables

```python
import chromadb
import os
from dotenv import load_dotenv

load_dotenv()

client = chromadb.HttpClient(
    host=os.getenv("CHROMA_HOST", "localhost"),
    port=int(os.getenv("CHROMA_PORT", "8000")),
    headers={"Authorization": f"Bearer {os.getenv('CHROMA_AUTH_TOKEN')}"}
    if os.getenv("CHROMA_AUTH_TOKEN") else None
)
```

---

## Type Hints

### Query Results Type

```python
from typing import TypedDict, List, Optional

class QueryResult(TypedDict):
    ids: List[List[str]]
    embeddings: Optional[List[List[List[float]]]]
    documents: List[List[Optional[str]]]
    metadatas: List[List[Optional[dict]]]
    distances: Optional[List[List[float]]]
```

### Get Results Type

```python
from typing import TypedDict, List, Optional

class GetResult(TypedDict):
    ids: List[str]
    embeddings: Optional[List[List[float]]]
    documents: List[Optional[str]]
    metadatas: List[Optional[dict]]
```

### Type-Safe Collection Operations

```python
from typing import List, Dict, Optional

def add_typed_documents(
    collection,
    ids: List[str],
    documents: List[str],
    metadatas: Optional[List[Dict[str, any]]] = None
) -> None:
    collection.add(
        ids=ids,
        documents=documents,
        metadatas=metadatas
    )
```

---

## Error Handling

### Handle Collection Not Found

```python
import chromadb

client = chromadb.PersistentClient()

try:
    collection = client.get_collection(name="nonexistent_collection")
except ValueError as e:
    if "does not exist" in str(e):
        print("Collection not found, creating...")
        collection = client.create_collection(name="nonexistent_collection")
    else:
        raise
```

### Handle Duplicate IDs

```python
try:
    collection.add(
        ids=["id1"],
        documents=["Document"]
    )

    # This will fail - ID already exists
    collection.add(
        ids=["id1"],
        documents=["Another document"]
    )
except Exception as e:
    print(f"ID already exists. Use update() or upsert() instead. Error: {e}")

    # Use upsert to add or update
    collection.upsert(
        ids=["id1"],
        documents=["Another document"]
    )
```

### Handle Connection Errors

```python
import chromadb
from requests.exceptions import ConnectionError

try:
    client = chromadb.HttpClient(host="localhost", port=8000)
    collections = client.list_collections()
except ConnectionError:
    print("Cannot connect to ChromaDB server. Make sure it's running.")
    print("Start server with: chroma run --path ./data")
except Exception as e:
    print(f"Error: {e}")
```

### Handle Empty Results

```python
results = collection.query(
    query_texts=["Very specific query"],
    n_results=5
)

if not results["ids"][0]:
    print("No results found. Try a broader query.")
else:
    print(f"Found {len(results['ids'][0])} results")
```

---

## Performance Optimization

### Batch Operations

```python
# Batch add for large datasets
chunk_size = 5000
for i in range(0, len(documents), chunk_size):
    collection.add(
        ids=ids[i:i + chunk_size],
        documents=documents[i:i + chunk_size],
        metadatas=metadatas[i:i + chunk_size]
    )
```

### Parallel Queries

```python
import concurrent.futures

queries = [
    "Query about topic A",
    "Query about topic B",
    "Query about topic C"
]

def run_query(query):
    return collection.query(
        query_texts=[query],
        n_results=5
    )

with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
    results = list(executor.map(run_query, queries))
```

### Limit Included Fields

```python
# Exclude embeddings for better performance
results = collection.query(
    query_texts=["Search query"],
    n_results=10,
    include=["documents", "metadatas", "distances"]
    # Don't include embeddings unless needed
)
```

### Reuse Embedding Function

```python
from chromadb.utils import embedding_functions

# Create once, reuse for multiple collections
openai_ef = embedding_functions.OpenAIEmbeddingFunction(
    api_key="your-api-key",
    model_name="text-embedding-3-small"
)

collection1 = client.create_collection(
    name="collection1",
    embedding_function=openai_ef
)

collection2 = client.create_collection(
    name="collection2",
    embedding_function=openai_ef
)
```

---

## Common Patterns

### Incremental Updates

```python
def add_daily_documents(collection, new_docs: List[str]):
    """Add new documents daily."""
    count = collection.count()
    new_ids = [f"doc{count + i}" for i in range(len(new_docs))]

    from datetime import datetime
    collection.add(
        ids=new_ids,
        documents=new_docs,
        metadatas=[{"added_date": datetime.now().isoformat()} for _ in new_docs]
    )
```

### Search with Fallback

```python
def search_with_fallback(collection, query: str):
    """Try specific category first, fallback to all documents."""
    # Try specific category first
    results = collection.query(
        query_texts=[query],
        n_results=5,
        where={"category": "premium"}
    )

    # If no results, search all documents
    if not results["ids"][0]:
        results = collection.query(
            query_texts=[query],
            n_results=5
        )

    return results
```

### Deduplicate Documents

```python
def add_unique(collection, doc_id: str, document: str, metadata: dict):
    """Add document only if ID doesn't exist, otherwise update."""
    try:
        existing = collection.get(ids=[doc_id])
        if existing["ids"]:
            print("Document already exists, updating...")
            collection.update(
                ids=[doc_id],
                documents=[document],
                metadatas=[metadata]
            )
        else:
            collection.add(
                ids=[doc_id],
                documents=[document],
                metadatas=[metadata]
            )
    except Exception:
        collection.add(
            ids=[doc_id],
            documents=[document],
            metadatas=[metadata]
        )
```

### Pagination

```python
def paginate_collection(collection, page_size: int = 100):
    """Iterate through all documents in pages."""
    total = collection.count()

    for offset in range(0, total, page_size):
        results = collection.get(
            limit=page_size,
            offset=offset
        )
        yield results

# Usage
for page in paginate_collection(collection, page_size=100):
    print(f"Processing {len(page['ids'])} documents...")
    # Process documents
```

---

## Metadata Filter Operators

### Comparison Operators

```python
# Equal
where = {"category": "tech"}
where = {"category": {"$eq": "tech"}}

# Not equal
where = {"category": {"$ne": "tech"}}

# Greater than
where = {"price": {"$gt": 100}}

# Greater than or equal
where = {"price": {"$gte": 100}}

# Less than
where = {"price": {"$lt": 100}}

# Less than or equal
where = {"price": {"$lte": 100}}
```

### Logical Operators

```python
# AND
where = {
    "$and": [
        {"category": "tech"},
        {"price": {"$lt": 1000}}
    ]
}

# OR
where = {
    "$or": [
        {"category": "tech"},
        {"category": "science"}
    ]
}

# NOT
where = {
    "$not": {"category": "archived"}
}
```

### Set Operators

```python
# In array
where = {
    "category": {"$in": ["tech", "science", "health"]}
}

# Not in array
where = {
    "category": {"$nin": ["archived", "deleted"]}
}
```

---

## Document Filter Operators

### Contains

```python
where_document = {"$contains": "machine learning"}
```

### Not Contains

```python
where_document = {"$not_contains": "deprecated"}
```

### Combined with Metadata

```python
results = collection.query(
    query_texts=["AI research"],
    where={"category": "research"},
    where_document={"$contains": "neural network"},
    n_results=10
)
```

---

## Working with Different Data Types

### Numeric Metadata

```python
collection.add(
    ids=["p1", "p2", "p3"],
    documents=["Product A", "Product B", "Product C"],
    metadatas=[
        {"price": 29.99, "stock": 100},
        {"price": 49.99, "stock": 50},
        {"price": 19.99, "stock": 200}
    ]
)

# Query by price range
results = collection.query(
    query_texts=["affordable products"],
    where={
        "$and": [
            {"price": {"$lt": 50}},
            {"stock": {"$gt": 75}}
        ]
    },
    n_results=5
)
```

### Boolean Metadata

```python
collection.add(
    ids=["u1", "u2", "u3"],
    documents=["User profile A", "User profile B", "User profile C"],
    metadatas=[
        {"active": True, "premium": False},
        {"active": True, "premium": True},
        {"active": False, "premium": False}
    ]
)

# Query active premium users
results = collection.query(
    query_texts=["find users"],
    where={
        "$and": [
            {"active": True},
            {"premium": True}
        ]
    },
    n_results=10
)
```

### List Metadata

```python
collection.add(
    ids=["a1", "a2"],
    documents=["Article about AI and ML", "Article about databases"],
    metadatas=[
        {"tags": ["ai", "machine-learning", "neural-networks"]},
        {"tags": ["database", "vector-search"]}
    ]
)

# Note: ChromaDB metadata values should be strings, numbers, or booleans
# For list-like metadata, use multiple metadata fields or JSON strings
```

---

## Context Manager Pattern

```python
import chromadb
from contextlib import contextmanager

@contextmanager
def get_collection(collection_name: str, path: str = "./chroma_db"):
    """Context manager for ChromaDB collection."""
    client = chromadb.PersistentClient(path=path)
    collection = client.get_or_create_collection(name=collection_name)
    try:
        yield collection
    finally:
        # Cleanup if needed
        pass

# Usage
with get_collection("my_collection") as collection:
    collection.add(
        ids=["id1"],
        documents=["Document"]
    )
    results = collection.query(
        query_texts=["Query"],
        n_results=5
    )
    print(results)
```

---

## Testing with ChromaDB

### Using EphemeralClient for Tests

```python
import unittest
import chromadb

class TestDocumentSearch(unittest.TestCase):
    def setUp(self):
        """Create ephemeral client for each test."""
        self.client = chromadb.EphemeralClient()
        self.collection = self.client.create_collection(name="test_collection")

    def test_add_and_query(self):
        """Test adding documents and querying."""
        self.collection.add(
            ids=["test1", "test2"],
            documents=["Document about cats", "Document about dogs"]
        )

        results = self.collection.query(
            query_texts=["pets"],
            n_results=2
        )

        self.assertEqual(len(results["ids"][0]), 2)

    def test_metadata_filtering(self):
        """Test metadata filtering."""
        self.collection.add(
            ids=["test1", "test2"],
            documents=["Doc 1", "Doc 2"],
            metadatas=[{"category": "A"}, {"category": "B"}]
        )

        results = self.collection.query(
            query_texts=["search"],
            where={"category": "A"},
            n_results=5
        )

        self.assertEqual(len(results["ids"][0]), 1)

if __name__ == "__main__":
    unittest.main()
```

---

## Monitoring and Debugging

### Enable Logging

```python
import logging
import chromadb

# Enable ChromaDB logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger("chromadb")
logger.setLevel(logging.DEBUG)

client = chromadb.PersistentClient(path="./debug_db")
```

### Inspect Collection Details

```python
collection = client.get_collection(name="my_collection")

print(f"Collection name: {collection.name}")
print(f"Collection metadata: {collection.metadata}")
print(f"Document count: {collection.count()}")

# Peek at first documents
first_docs = collection.peek(limit=3)
print(f"First documents: {first_docs}")
```

### Performance Profiling

```python
import time

def profile_query(collection, query_text: str, n_results: int = 10):
    """Profile query performance."""
    start = time.time()
    results = collection.query(
        query_texts=[query_text],
        n_results=n_results
    )
    elapsed = time.time() - start

    print(f"Query: {query_text}")
    print(f"Time: {elapsed:.3f}s")
    print(f"Results: {len(results['ids'][0])}")
    return results

# Usage
profile_query(collection, "machine learning", n_results=100)
```

---

## Migration and Backup

### Export Collection to JSON

```python
import json

def export_collection(collection, output_file: str):
    """Export entire collection to JSON."""
    all_data = collection.get()

    export_data = {
        "ids": all_data["ids"],
        "documents": all_data["documents"],
        "metadatas": all_data["metadatas"],
        "embeddings": all_data.get("embeddings")
    }

    with open(output_file, 'w') as f:
        json.dump(export_data, f)

    print(f"Exported {len(all_data['ids'])} documents to {output_file}")

# Usage
export_collection(collection, "backup.json")
```

### Import Collection from JSON

```python
import json

def import_collection(collection, input_file: str):
    """Import collection from JSON."""
    with open(input_file, 'r') as f:
        import_data = json.load(f)

    collection.add(
        ids=import_data["ids"],
        documents=import_data["documents"],
        metadatas=import_data["metadatas"],
        embeddings=import_data.get("embeddings")
    )

    print(f"Imported {len(import_data['ids'])} documents from {input_file}")

# Usage
import_collection(collection, "backup.json")
```

### Copy Collection

```python
def copy_collection(source_collection, dest_collection, batch_size: int = 1000):
    """Copy all data from source to destination collection."""
    total = source_collection.count()

    for offset in range(0, total, batch_size):
        data = source_collection.get(
            limit=batch_size,
            offset=offset,
            include=["documents", "metadatas", "embeddings"]
        )

        dest_collection.add(
            ids=data["ids"],
            documents=data["documents"],
            metadatas=data["metadatas"],
            embeddings=data.get("embeddings")
        )

        print(f"Copied {offset + len(data['ids'])}/{total} documents")

# Usage
source = client.get_collection(name="original")
dest = client.create_collection(name="backup")
copy_collection(source, dest)
```

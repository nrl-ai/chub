---
name: package
description: "lancedb package guide for Python with local and cloud connections, schema definition, vector search, filtering, and indexing"
metadata:
  languages: "python"
  versions: "0.29.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "lancedb,python,vector-database,vector-search,embeddings,arrow,pydantic,rag"
---

# lancedb Python Package Guide

## Golden Rule

- Use `lancedb` as an embedded or remote database handle, then work through tables.
- For local development, connect to a filesystem path first. For hosted deployments, use a `db://...` URI with an API key and region.
- Define schema up front for production use, especially vector dimensions and embedding behavior.

## Version-Sensitive Notes

- This entry is pinned to the version used here `0.29.2`.
- PyPI currently lists `0.29.2` as the latest release for `lancedb`.
- PyPI metadata for `0.29.2` requires Python `>=3.10`.
- The main docs site is version-light and currently lives under `/docs/v0/`, so use PyPI for exact package pinning and the docs site for current 0.x usage patterns.

## Install

Pin the package when you need reproducible behavior:

```bash
python -m pip install "lancedb==0.29.2"
```

If you use `uv`:

```bash
uv add "lancedb==0.29.2"
```

If you use Poetry:

```bash
poetry add "lancedb==0.29.2"
```

## Connect To A Database

### Local embedded database

Use a directory path for local development, tests, or single-node apps:

```python
import lancedb

db = lancedb.connect("data/sample-lancedb")
```

LanceDB creates and manages data under that path. Reuse the same path to reopen the database later.

### Object storage or self-managed cloud storage

The Python SDK also accepts storage URIs such as `s3://...` and supports `storage_options` when the backend needs explicit configuration:

```python
import lancedb

db = lancedb.connect(
    "s3://my-bucket/lancedb",
    storage_options={"timeout": "60s"},
)
```

In practice, also make sure the underlying cloud credentials are available through the storage backend you are using.

### LanceDB Cloud

Use a `db://` URI plus an API key and region:

```python
import lancedb

db = lancedb.connect(
    "db://my-database",
    api_key="ldb_...",
    region="us-east-1",
)
```

If you are targeting LanceDB Cloud, do not treat it like a local path. The connection parameters are different.

## Define A Table Schema

You can start with plain Python dictionaries, but schema-first definitions are safer when vectors, filters, and production queries matter.

### Pydantic model with a vector column

The official docs show `LanceModel` and `Vector(...)` for typed schemas:

```python
from lancedb.pydantic import LanceModel, Vector

class Item(LanceModel):
    id: str
    title: str
    vector: Vector(3)
```

This is the main constraint to remember: the vector dimension in the schema must match the data you insert and the query vectors you send later.

### Schema with an embedding function

If you want to search with text queries, define the embedding behavior in the schema:

```python
import lancedb
from lancedb.embeddings import get_registry
from lancedb.pydantic import LanceModel, Vector

registry = get_registry()
func = registry.get("openai").create()

class Document(LanceModel):
    text: str = func.SourceField()
    vector: Vector(func.ndims()) = func.VectorField()

db = lancedb.connect("data/sample-lancedb")
```

This pattern lets LanceDB turn text into vectors for indexing and search. Provider-specific embedding setups can still require additional credentials or model downloads.

## Create And Reopen Tables

### Create a table from rows

```python
import lancedb

db = lancedb.connect("data/sample-lancedb")

table = db.create_table(
    "docs",
    data=[
        {"id": "a", "title": "cat", "vector": [0.1, 0.2, 0.3]},
        {"id": "b", "title": "dog", "vector": [0.3, 0.2, 0.1]},
    ],
)
```

### Reopen an existing table

```python
table = db.open_table("docs")
```

Use `create_table(...)` for first-time creation and `open_table(...)` for later access. Do not assume a table already exists.

## Add Data

Append more rows with `add(...)`:

```python
table.add(
    [
        {"id": "c", "title": "bird", "vector": [0.8, 0.1, 0.1]},
    ]
)
```

Keep row shapes consistent with the table schema. Inconsistent field names or vector sizes will cause avoidable failures.

## Search

### Vector search

If your table stores explicit vectors, pass a numeric vector to `search(...)`:

```python
results = table.search([0.2, 0.2, 0.2]).limit(3).to_pandas()
print(results)
```

### Text search through an embedding function

If the schema defines an embedding function, you can search with text:

```python
results = table.search(query="puppy").limit(3).to_pandas()
print(results[["text", "_distance"]])
```

The text-query flow depends on embedding-aware schema setup. If you only created a raw vector column, send vectors instead of strings.

### Filter search results

The Python query builder supports `where(...)` filters:

```python
results = (
    table.search([0.2, 0.2, 0.2])
    .where("id != 'b'")
    .limit(3)
    .to_pandas()
)
```

Use filters to keep reranking and result handling smaller, especially when you already know tenant, status, or category constraints.

## Indexing And Performance

Search works without an index, but larger tables should add one deliberately. The Python docs show `create_index(...)` on the table:

```python
table.create_index(metric="cosine")
```

For small prototypes, skip manual indexing until you have enough data to care about latency. For production-scale tables, add indexes before benchmarking query behavior.

## Practical Setup Patterns

### Minimal local flow

```python
import lancedb

db = lancedb.connect("data/app.lancedb")

table = db.create_table(
    "items",
    data=[
        {"id": "1", "title": "apple", "vector": [0.1, 0.0, 0.0]},
        {"id": "2", "title": "orange", "vector": [0.0, 0.1, 0.0]},
    ],
)

rows = table.search([0.1, 0.0, 0.0]).limit(5).to_pandas()
print(rows)
```

### Minimal cloud flow

```python
import lancedb

db = lancedb.connect(
    "db://my-database",
    api_key="ldb_...",
    region="us-east-1",
)

table = db.open_table("documents")
rows = table.search("semantic search query").limit(5).to_pandas()
```

This cloud example assumes the table was created with an embedding-aware schema, so string search can be converted into vectors.

## Common Pitfalls

- Python version mismatch: PyPI metadata for `0.29.2` requires Python `>=3.10`.
- Wrong connection mode: local paths, object storage URIs, and `db://` cloud databases use different connection parameters.
- Missing cloud auth: LanceDB Cloud requires both `api_key` and `region`.
- Vector dimension mismatch: `Vector(n)` must match inserted vectors and query vectors.
- Text query without embeddings: `search(query="...")` is for embedding-aware tables; plain vector tables should be queried with numeric vectors.
- Benchmarking before indexing: large-table performance can look worse than expected if you never call `create_index(...)`.

## Official Sources

- Docs root: `https://lancedb.com/docs/`
- Python quickstart: `https://lancedb.com/docs/quickstart/`
- Python cloud quickstart: `https://lancedb.com/docs/cloud/quickstart/`
- Python local/cloud storage guide: `https://lancedb.com/docs/storage/integrations/localcloud-storage/`
- Python search and filtering guide: `https://lancedb.github.io/lancedb/python/python/#search-and-filtering`
- Pydantic integration guide: `https://lancedb.com/docs/integrations/frameworks/pydantic/`
- GitHub repository: `https://github.com/lancedb/lancedb`
- PyPI package: `https://pypi.org/project/lancedb/`

---
name: package
description: "Official Elasticsearch Python client for connecting to Elasticsearch, indexing documents, searching, async usage, and bulk helpers."
metadata:
  languages: "python"
  versions: "9.3.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "elasticsearch,search,elastic,python,async,bulk"
---

# Elasticsearch Python Package Guide

## Golden Rule

Use the official `elasticsearch` Python package for Elasticsearch 9.x clusters, and match the client major version to the server major version.

As of March 11, 2026:

- PyPI lists `elasticsearch 9.3.0`
- PyPI requires Python `>=3.10`
- The docs URL points to `https://elasticsearch-py.readthedocs.io/en/latest/`, which currently serves a moving API reference and can be ahead of `9.3.0`

If your cluster is still on Elasticsearch 8.x, do not install the 9.x client by default. Use the 8.x client line instead.

## Install

Pin the version your project expects:

```bash
python -m pip install "elasticsearch==9.3.0"
```

Async support:

```bash
python -m pip install "elasticsearch[async]==9.3.0"
```

If you need multiple client majors installed side by side, Elastic also publishes versioned package names such as `elasticsearch8` and `elasticsearch9`.

## Initialize A Client

### Elastic Cloud

Use `cloud_id` when connecting to Elastic Cloud. Elastic recommends this because it configures the client appropriately for cloud deployments.

```python
import os

from elasticsearch import Elasticsearch

client = Elasticsearch(
    cloud_id=os.environ["ELASTIC_CLOUD_ID"],
    api_key=os.environ["ELASTIC_API_KEY"],
)

print(client.info().body)
```

### Self-managed cluster with CA cert

```python
import os

from elasticsearch import Elasticsearch

client = Elasticsearch(
    "https://localhost:9200",
    ca_certs="/path/to/http_ca.crt",
    basic_auth=("elastic", os.environ["ELASTIC_PASSWORD"]),
)

print(client.info().body)
```

### Certificate fingerprint verification

Use this for sync clients when you have the SHA-256 fingerprint instead of the CA file:

```python
import os

from elasticsearch import Elasticsearch

client = Elasticsearch(
    "https://localhost:9200",
    ssl_assert_fingerprint=os.environ["ELASTIC_CERT_FINGERPRINT"],
    basic_auth=("elastic", os.environ["ELASTIC_PASSWORD"]),
)
```

Important:

- fingerprint verification requires Python 3.10 or later
- the Elastic docs say this method is not available with `AsyncElasticsearch` when using the `aiohttp` HTTP client
- if you do not pass `ca_certs` or `ssl_assert_fingerprint`, the client uses `certifi` for CA certificates if available

## Authentication Patterns

### Basic auth

```python
import os

from elasticsearch import Elasticsearch

client = Elasticsearch(
    "https://localhost:9200",
    ca_certs="/path/to/http_ca.crt",
    basic_auth=("username", os.environ["ELASTIC_PASSWORD"]),
)
```

### API key

```python
import os

from elasticsearch import Elasticsearch

client = Elasticsearch(
    "https://localhost:9200",
    ca_certs="/path/to/http_ca.crt",
    api_key=os.environ["ELASTIC_API_KEY"],
)
```

### Bearer token

```python
import os

from elasticsearch import Elasticsearch

client = Elasticsearch(
    "https://localhost:9200",
    bearer_auth=os.environ["ELASTIC_TOKEN"],
)
```

### Per-request overrides with `.options()`

`.options()` is the clean way to vary auth or transport settings for one request without rebuilding the client:

```python
import os

from elasticsearch import Elasticsearch

client = Elasticsearch(
    "https://localhost:9200",
    ca_certs="/path/to/http_ca.crt",
)

secure_client = client.options(api_key=os.environ["ELASTIC_API_KEY"])
response = secure_client.search(index="logs-*", query={"match_all": {}})
print(response.body)
```

## Core Usage

### Create a client once and reuse it

The client is thread-safe and uses persistent connections. Build one shared client for your process instead of constructing a new client for every request.

```python
from elasticsearch import Elasticsearch

client = Elasticsearch("http://localhost:9200")
```

### Create an index

```python
from elasticsearch import Elasticsearch

client = Elasticsearch("http://localhost:9200")

if not client.indices.exists(index="books"):
    client.indices.create(
        index="books",
        mappings={
            "properties": {
                "title": {"type": "text"},
                "year": {"type": "integer"},
                "tags": {"type": "keyword"},
            }
        },
    )
```

### Index a document

```python
from elasticsearch import Elasticsearch

client = Elasticsearch("http://localhost:9200")

resp = client.index(
    index="books",
    id="book-1",
    document={
        "title": "Elasticsearch Guide",
        "year": 2026,
        "tags": ["search", "python"],
    },
    refresh=True,
)

print(resp.body["result"])
```

### Get a document

```python
resp = client.get(index="books", id="book-1")
print(resp.body["_source"])
```

### Search documents

Use keyword arguments. The Python client mirrors Elasticsearch APIs closely, and reserved Python keywords use aliases such as `from_`.

```python
resp = client.search(
    index="books",
    from_=0,
    size=10,
    query={
        "bool": {
            "must": [{"match": {"title": "guide"}}],
            "filter": [{"term": {"tags": "python"}}],
        }
    },
)

for hit in resp.body["hits"]["hits"]:
    print(hit["_id"], hit["_source"]["title"])
```

### Update a document

```python
resp = client.update(
    index="books",
    id="book-1",
    doc={"year": 2027},
    refresh=True,
)

print(resp.body["result"])
```

### Delete a document

```python
client.delete(index="books", id="book-1", refresh=True)
```

## Bulk And Helper APIs

Use helpers when indexing or scanning large datasets.

### `helpers.bulk()`

`helpers.bulk()` consumes an iterable of actions and returns summary information. It is the usual starting point for batch indexing.

```python
from elasticsearch import Elasticsearch, helpers

client = Elasticsearch("http://localhost:9200")

actions = [
    {
        "_index": "books",
        "_id": "book-1",
        "_source": {"title": "One", "year": 2024},
    },
    {
        "_index": "books",
        "_id": "book-2",
        "_source": {"title": "Two", "year": 2025},
    },
]

successes, errors = helpers.bulk(client, actions, stats_only=False)
print(successes, errors)
```

### `helpers.streaming_bulk()`

Prefer `streaming_bulk()` when you want per-item results or need to avoid collecting large error payloads in memory.

```python
from elasticsearch import Elasticsearch, helpers

client = Elasticsearch("http://localhost:9200")

for ok, result in helpers.streaming_bulk(client, actions, chunk_size=500):
    if not ok:
        print(result)
```

### `helpers.parallel_bulk()`

Use this for higher ingest throughput when threaded bulk writes are acceptable:

```python
from elasticsearch import Elasticsearch, helpers

client = Elasticsearch("http://localhost:9200")

for ok, result in helpers.parallel_bulk(client, actions, thread_count=4, chunk_size=500):
    if not ok:
        print(result)
```

### `helpers.scan()`

Use `scan()` for large result sets. It is built on the scroll API and does not preserve sort order unless you explicitly request `preserve_order=True`.

```python
from elasticsearch import Elasticsearch
from elasticsearch.helpers import scan

client = Elasticsearch("http://localhost:9200")

for hit in scan(
    client,
    index="books",
    query={"query": {"match": {"title": "guide"}}},
):
    print(hit["_id"])
```

### `helpers.reindex()`

The helper exists, but Elastic recommends the server-side reindex API when possible. The helper only moves document data and does not transfer mappings.

## Async Usage

Install the async extra and use `AsyncElasticsearch`:

```python
import asyncio

from elasticsearch import AsyncElasticsearch

client = AsyncElasticsearch(
    "https://localhost:9200",
    api_key="your-api-key",
)

async def main() -> None:
    resp = await client.search(
        index="books",
        query={"match_all": {}},
        size=20,
    )
    print(resp.body)
    await client.close()

asyncio.run(main())
```

Async helper variants are available in `elasticsearch.helpers` with `async_` prefixes such as `async_bulk()` and `async_scan()`.

If you use `AsyncElasticsearch` in FastAPI, Starlette, Django ASGI, or another ASGI app, close the client during application shutdown to avoid unclosed `aiohttp` session warnings.

## Transport Configuration

### Request timeouts

Set transport-level timeouts on the client or per request:

```python
from elasticsearch import Elasticsearch

client = Elasticsearch(
    "http://localhost:9200",
    request_timeout=10,
)

resp = client.options(request_timeout=5).search(
    index="books",
    query={"match_all": {}},
)
```

### Retries

Configure retries for transient failures:

```python
from elasticsearch import Elasticsearch

client = Elasticsearch(
    "http://localhost:9200",
    max_retries=5,
    retry_on_timeout=True,
)
```

### Sniffing

The client supports node sniffing with settings like `sniff_on_start`, `sniff_before_requests`, and `sniff_on_node_failure`.

Do not enable sniffing if you connect through an HTTP load balancer or proxy. Elastic explicitly warns that sniffing can bypass the load balancer by switching the client to direct node IPs.

## DSL And ES|QL

The package now includes higher-level modules in the main client:

- `elasticsearch.dsl` for the Python DSL
- `elasticsearch.esql` for the ES|QL query builder

If you still depend on the old `elasticsearch-dsl` package, migrate imports from `elasticsearch_dsl` to `elasticsearch.dsl`.

## Common Pitfalls

- Do not pair the `9.x` client with Elasticsearch `8.x`. The official compatibility table marks `9.x` as incompatible with Elasticsearch 8.x.
- For major upgrades, upgrade Elasticsearch first and then upgrade the Python client.
- Do not open a new client per request in web apps, workers, or lambdas. Reuse a process-level client.
- Do not turn on sniffing behind a proxy or load balancer.
- `ssl_assert_fingerprint` is not available for the async client when using the `aiohttp` transport.
- `helpers.bulk()` can consume a lot of memory when collecting many errors; prefer `streaming_bulk()` for large ingest jobs.
- `helpers.reindex()` does not move index mappings.
- The Python client uses keyword-only aliases for reserved words, for example `from_` instead of `from`.
- The docs URL uses `en/latest`, which is a moving target. For version-sensitive work, prefer PyPI for the pinned release and the stable/versioned docs for the matching API reference.

## Version-Sensitive Notes For 9.3.0

- PyPI shows `elasticsearch 9.3.0` released on February 3, 2026.
- The Read the Docs `latest` root currently serves newer docs than `9.3.0`, so this guide treats it as a discovery URL, not a version-locked reference.
- The Read the Docs `stable` API reference currently corresponds to `9.3.0`.
- Elastic documents compatibility mode as always enabled in the Python client, which helps during transitions, but the official compatibility matrix still says the 9.x client is not for Elasticsearch 8.x.

## Official Sources

- PyPI package: https://pypi.org/project/elasticsearch/
- Docs URL: https://elasticsearch-py.readthedocs.io/en/latest/
- Stable API reference for 9.3.0: https://elasticsearch-py.readthedocs.io/en/stable/
- Elastic Python client overview: https://www.elastic.co/docs/reference/elasticsearch/clients/python
- Elastic connecting guide: https://www.elastic.co/docs/reference/elasticsearch/clients/python/connecting
- Elastic configuration guide: https://www.elastic.co/docs/reference/elasticsearch/clients/python/configuration
- Elastic client helpers guide: https://www.elastic.co/docs/reference/elasticsearch/clients/python/client-helpers
- Elastic async guide: https://www.elastic.co/docs/reference/elasticsearch/clients/python/async
- Elastic DSL migration guide: https://www.elastic.co/docs/reference/elasticsearch/clients/python/dsl_migrating

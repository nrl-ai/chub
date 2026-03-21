---
name: package
description: "opensearch-py package guide for Python clients connecting to OpenSearch clusters"
metadata:
  languages: "python"
  versions: "3.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opensearch,search,analytics,python,aws,async"
---

# opensearch-py Python Package Guide

## Golden Rule

Use `opensearch-py` for Python access to OpenSearch clusters, and write 3.x code with keyword arguments for API calls. As of March 12, 2026, PyPI still lists `3.1.0` as the current release.

## Install

```bash
python -m pip install "opensearch-py==3.1.0"
```

Async support:

```bash
python -m pip install "opensearch-py[async]==3.1.0"
```

If you need AWS SigV4 auth, install `boto3` or `botocore` in the same environment:

```bash
python -m pip install "opensearch-py==3.1.0" boto3
```

## What The Package Covers

`opensearch-py` is the main Python client for OpenSearch. It gives you:

- low-level wrappers around the OpenSearch REST APIs
- sync and async clients
- bulk helpers
- high-level DSL functionality merged from `opensearch-dsl-py` starting in `2.2.0`

For new work, prefer this package instead of the archived `opensearch-dsl-py` package.

## Initialize A Client

### Local or self-managed cluster

Use TLS and certificate verification in real environments. The insecure flags below are only acceptable for local testing.

```python
from opensearchpy import OpenSearch

client = OpenSearch(
    hosts=[{"host": "localhost", "port": 9200}],
    http_auth=("admin", "admin"),
    http_compress=True,
    use_ssl=True,
    verify_certs=False,
    ssl_assert_hostname=False,
    ssl_show_warn=False,
)

info = client.info()
print(info["version"]["number"])
```

### Verified TLS with a CA bundle

```python
from opensearchpy import OpenSearch

client = OpenSearch(
    hosts=[{"host": "search.example.com", "port": 9200}],
    http_auth=("user", "pass"),
    http_compress=True,
    use_ssl=True,
    verify_certs=True,
    ca_certs="/path/to/root-ca.pem",
)
```

Use `client_cert` and `client_key` if your cluster is configured for client certificate authentication.

### Amazon OpenSearch Service or Serverless

Match the signer to the connection class. In the package source, `AWSV4SignerAuth` is a deprecated alias of `RequestsAWSV4SignerAuth`.

```python
import boto3
from opensearchpy import OpenSearch, RequestsHttpConnection, RequestsAWSV4SignerAuth

credentials = boto3.Session().get_credentials()
auth = RequestsAWSV4SignerAuth(credentials, region="us-west-2", service="es")

client = OpenSearch(
    hosts=[{"host": "my-domain.us-west-2.es.amazonaws.com", "port": 443}],
    http_auth=auth,
    use_ssl=True,
    verify_certs=True,
    connection_class=RequestsHttpConnection,
    pool_maxsize=20,
)
```

For OpenSearch Serverless, change `service="aoss"`.

If you use the default urllib3 connection class instead of requests, use `Urllib3AWSV4SignerAuth`.

### Async client

```python
import asyncio
from opensearchpy import AsyncHttpConnection, AsyncOpenSearch

async def main() -> None:
    client = AsyncOpenSearch(
        hosts=[{"host": "localhost", "port": 9200}],
        http_auth=("admin", "admin"),
        use_ssl=True,
        verify_certs=False,
        ssl_show_warn=False,
        connection_class=AsyncHttpConnection,
    )
    try:
        info = await client.info()
        print(info["version"]["number"])
    finally:
        await client.close()

asyncio.run(main())
```

The async guide shows the same setup with `opensearch-py[async]`. Source inspection for `AsyncOpenSearch` in the official repo shows `close()` is async, so `await client.close()` is the safe pattern.

## Core Usage

### Ping and inspect cluster info

```python
if not client.ping():
    raise RuntimeError("OpenSearch is not reachable")

info = client.info()
version = info["version"]["number"]
distribution = info["version"]["distribution"]
```

### Create an index

Use keyword arguments. In `3.0.0`, generated API methods were changed to require keyword-only arguments.

```python
index_name = "movies"

client.indices.create(
    index=index_name,
    body={
        "settings": {
            "index": {
                "number_of_shards": 1,
                "number_of_replicas": 1,
            }
        }
    },
)
```

### Index and search documents

```python
client.index(
    index="movies",
    id="1",
    body={
        "title": "Moneyball",
        "director": "Bennett Miller",
        "year": 2011,
    },
    refresh=True,
)

response = client.search(
    index="movies",
    body={
        "query": {
            "multi_match": {
                "query": "miller",
                "fields": ["title^2", "director"],
            }
        }
    },
)

hits = response["hits"]["hits"]
```

### Bulk indexing with helpers

Prefer `helpers.bulk()` or `helpers.parallel_bulk()` over hand-building newline-delimited JSON.

```python
from opensearchpy import OpenSearch, helpers

docs = [
    {"_index": "movies", "_id": "1", "title": "Moneyball", "year": 2011},
    {"_index": "movies", "_id": "2", "title": "Interstellar", "year": 2014},
]

helpers.bulk(client, docs, max_retries=3)
```

For large ingests, `helpers.parallel_bulk()` lets you tune `chunk_size`, `max_chunk_bytes`, and `request_timeout`.

## Configuration And Auth Notes

### Connection classes

- `Urllib3HttpConnection` is the default sync connection class and the project recommendation unless your app is standardized on `requests`.
- `RequestsHttpConnection` is the sync choice when you specifically want the requests stack.
- `AsyncHttpConnection` uses `aiohttp` and is the async option.

### TLS and certificate handling

- Prefer `verify_certs=True` in any non-local environment.
- Pass `ca_certs` when you need a custom CA bundle.
- Only set `ssl_assert_hostname=False` when you intentionally need to bypass hostname verification, such as local testing or certificate mismatch debugging.
- The SSL guide notes that CA discovery can fall back to OpenSSL env vars, `certifi`, or backend defaults if `ca_certs` is not provided.

### Pooling and compression

- Use `http_compress=True` for request-body gzip compression when indexing larger payloads.
- Increase `pool_maxsize` for threaded or high-concurrency workloads.

## Common Pitfalls

- The import is `opensearchpy`, not `opensearch_py`.
- Old examples that pass positional parameters to generated APIs are a poor fit on `3.x`; use `index=...`, `body=...`, `id=...`, and similar keyword arguments.
- Do not copy the docs' local-test settings into production: `verify_certs=False` and `ssl_assert_hostname=False` weaken TLS.
- `refresh=True` is convenient for tests and demos but can hurt indexing throughput in real workloads.
- The async client requires the `async` extra. `pip install opensearch-py` alone is not enough for `AsyncOpenSearch`.
- For AWS-managed clusters, match the signer and connection class correctly. The repo auth guide distinguishes `RequestsAWSV4SignerAuth`, `Urllib3AWSV4SignerAuth`, and `AWSV4SignerAsyncAuth`.
- `opensearch-dsl-py` has been merged into `opensearch-py`; avoid building new code on the archived package.

## Version-Sensitive Notes For 3.1.0

- `3.1.0` is the current PyPI release as of March 12, 2026.
- `3.1.0` supports Python `3.10`, `3.11`, and `3.12`; the package metadata requires `>=3.10,<4`.
- The `3.1.0` changelog explicitly deprecates Python `3.8` and `3.9` support, so treat older runtime examples as stale.
- The compatibility matrix says `3.x.x` clients work with OpenSearch `1.0.0` through `3.x`, as long as you avoid features removed from newer server versions.
- Since `3.0.0`, generated APIs use mandatory keyword-only arguments. If you are adapting pre-3.0 snippets, update the call style before assuming the example is valid.

## Official Sources

- Docs root: https://docs.opensearch.org/latest/clients/python-low-level/
- PyPI: https://pypi.org/project/opensearch-py/
- Repository: https://github.com/opensearch-project/opensearch-py
- User guide: https://github.com/opensearch-project/opensearch-py/blob/main/USER_GUIDE.md
- Auth guide: https://github.com/opensearch-project/opensearch-py/blob/main/guides/auth.md
- Async guide: https://github.com/opensearch-project/opensearch-py/blob/main/guides/async.md
- SSL guide: https://github.com/opensearch-project/opensearch-py/blob/main/guides/ssl.md
- Bulk guide: https://github.com/opensearch-project/opensearch-py/blob/main/guides/bulk.md
- Compatibility matrix: https://github.com/opensearch-project/opensearch-py/blob/main/COMPATIBILITY.md
- Changelog: https://github.com/opensearch-project/opensearch-py/blob/main/CHANGELOG.md

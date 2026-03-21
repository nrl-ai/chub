---
name: firestore
description: "Google Cloud Firestore Python client for document reads, queries, transactions, batched writes, and emulator-backed tests"
metadata:
  languages: "python"
  versions: "2.24.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,firestore,gcp,nosql,database,python,async"
---

# google-cloud-firestore Python Package Guide

## Golden Rule

Use the official `google-cloud-firestore` client with Google Cloud authentication.

Prefer Application Default Credentials (ADC) for local development and deployed workloads. Use a service account key file only when ADC or workload identity is not available.

## Install

```bash
pip install google-cloud-firestore
```

Pin the version if you need the exact package line covered here:

```bash
pip install "google-cloud-firestore==2.24.0"
```

Other common package managers:

```bash
uv add google-cloud-firestore
poetry add google-cloud-firestore
```

## Authentication And Setup

1. Enable Firestore for the target Google Cloud project.
2. For local development, authenticate with ADC.
3. For deployed workloads, prefer workload identity or an attached service account.
4. Set the project explicitly when your environment does not already provide one.

Local ADC setup:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

Service account key fallback:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

## Initialize A Client

Synchronous client:

```python
from google.cloud import firestore

db = firestore.Client(project="your-project-id")
```

Asynchronous client:

```python
from google.cloud import firestore

adb = firestore.AsyncClient(project="your-project-id")
```

Explicit credentials:

```python
from google.cloud import firestore
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

db = firestore.Client(
    project="your-project-id",
    credentials=credentials,
    database="(default)",
)
```

Important client note: the current official `Client` reference explicitly says `_http` is not accepted because Firestore requires the gRPC transport.

## Core Usage

### Create Or Replace A Document

```python
from google.cloud import firestore

db = firestore.Client()
city_ref = db.collection("cities").document("SF")

city_ref.set(
    {
        "name": "San Francisco",
        "state": "CA",
        "country": "USA",
        "capital": False,
        "population": 860000,
    }
)
```

### Merge Into An Existing Document

```python
city_ref.set({"nickname": "San Fran"}, merge=True)
```

### Add A Document With An Auto-Generated ID

```python
doc_ref, write_result = db.collection("cities").add(
    {"name": "Tokyo", "country": "Japan"}
)

print(doc_ref.id)
print(write_result.update_time)
```

### Read One Document

```python
snapshot = city_ref.get()

if snapshot.exists:
    print(snapshot.to_dict())
```

### Query A Collection

Prefer the current `FieldFilter` style. Older positional `where("field", "==", value)` snippets still exist in blog posts and old answers, but the official current samples use `where(filter=...)`.

```python
from google.cloud import firestore
from google.cloud.firestore_v1.base_query import FieldFilter

db = firestore.Client()

query = (
    db.collection("cities")
    .where(filter=FieldFilter("state", "==", "CA"))
    .order_by("population", direction=firestore.Query.DESCENDING)
    .limit(10)
)

for snapshot in query.stream():
    print(snapshot.id, snapshot.to_dict())
```

The query reference notes that `stream()` is usually preferred over `get()`.

### Update Nested Fields And Atomic Values

```python
from google.cloud import firestore

city_ref.update(
    {
        "population": firestore.Increment(1),
        "regions": firestore.ArrayUnion(["west_coast"]),
        "last_updated": firestore.SERVER_TIMESTAMP,
        "stats.visits": firestore.Increment(1),
    }
)
```

### Batched Writes

Use a batch when you already know the write set and do not need read-before-write logic.

```python
batch = db.batch()

sf_ref = db.collection("cities").document("SF")
la_ref = db.collection("cities").document("LA")

batch.set(sf_ref, {"name": "San Francisco", "state": "CA"})
batch.update(la_ref, {"population": 3900000})
batch.commit()
```

### Transactions

Use a transaction when writes depend on currently stored values.

```python
from google.cloud import firestore

db = firestore.Client()
transaction = db.transaction()
counter_ref = db.collection("counters").document("pageviews")

@firestore.transactional
def increment_counter(transaction, ref):
    snapshot = ref.get(transaction=transaction)
    current = snapshot.get("value") or 0
    transaction.update(ref, {"value": current + 1})

increment_counter(transaction, counter_ref)
```

### BulkWriter For High-Volume Non-Transactional Writes

Use `bulk_writer()` when throughput matters more than atomic multi-document behavior.

```python
from google.cloud import firestore

db = firestore.Client()
bulk = db.bulk_writer()

sf_ref = db.collection("cities").document("SF")
nyc_ref = db.collection("cities").document("NYC")

bulk.set(sf_ref, {"name": "San Francisco"})
bulk.update(nyc_ref, {"population": firestore.Increment(1)})
bulk.flush()
bulk.close()
```

## Emulator Setup

For local tests, point the client at the Firestore emulator:

```bash
export FIRESTORE_EMULATOR_HOST="127.0.0.1:8080"
export GOOGLE_CLOUD_PROJECT="demo-project"
```

This is useful for integration tests, but it is not a full production-equivalence check. Google documents differences around transactions, index handling, and cleanup behavior.

## Configuration Notes

- Pass `project=` explicitly when your environment is ambiguous.
- Pass `database="(default)"` unless you intentionally use a named Firestore database.
- For async code, use `firestore.AsyncClient` instead of wrapping the sync client in thread pools.
- In multiprocessing code, create Firestore clients after `os.fork()`, not before.

## Common Pitfalls

- Do not invent custom auth headers or API-key flows. Firestore Python expects Google credentials.
- Do not pass `_http` into the client. The official client reference says Firestore does not accept it.
- Do not treat a batch like a transaction. If the write depends on a prior read, use a transaction.
- Do not treat `BulkWriter` like a transaction. It improves throughput, not atomicity.
- Prefer `query.stream()` for iteration-heavy reads.
- Prefer `FieldFilter`-based query examples in new code.
- Keep emulator-only settings out of production environments.
- If copied examples reference `v1beta1` or other legacy surfaces, reconcile them against the current 2.x docs before using them.

## Version-Sensitive Notes

### Current Upstream State

- Version used here for this session: `2.24.0`
- PyPI official package page observed on 2026-03-12: `2.24.0`
- PyPI release date observed on 2026-03-12: `2026-03-06`
- Firestore Python reference docs root observed on 2026-03-12: `2.23.0 (latest)`

### Relevant 2.x Migration History

The official upgrade guide for `2.0.0` is still relevant when you run into old examples:

- `v1beta1` and older legacy surfaces were removed.
- generated GAPIC methods follow the modern request-or-flattened-kwargs style
- older query snippets often predate the current `FieldFilter` examples

## Official Sources Used

- PyPI package page: https://pypi.org/project/google-cloud-firestore/
- Firestore Python reference docs root: https://cloud.google.com/python/docs/reference/firestore/latest
- Firestore `Client` reference: https://cloud.google.com/python/docs/reference/firestore/latest/google.cloud.firestore_v1.client.Client
- Firestore `Query` reference: https://cloud.google.com/python/docs/reference/firestore/latest/google.cloud.firestore_v1.query.Query
- Firestore `BulkWriter` reference: https://cloud.google.com/python/docs/reference/firestore/latest/google.cloud.firestore_v1.bulk_writer.BulkWriter
- Firestore code samples index: https://cloud.google.com/firestore/docs/samples
- Firestore query sample using `FieldFilter`: https://cloud.google.com/firestore/docs/samples/firestore-query-filter-compound-multi-eq
- Firestore batch write sample: https://cloud.google.com/firestore/docs/samples/firestore-data-batch-writes
- Firestore transaction sample: https://cloud.google.com/firestore/docs/samples/firestore-transaction-document-update-conditional
- Google Cloud ADC guide: https://cloud.google.com/docs/authentication/provide-credentials-adc
- Firestore emulator guide: https://cloud.google.com/firestore/native/docs/emulator
- Firestore Python changelog: https://cloud.google.com/python/docs/reference/firestore/latest/changelog
- Firestore Python upgrade guide: https://cloud.google.com/python/docs/reference/firestore/latest/upgrading

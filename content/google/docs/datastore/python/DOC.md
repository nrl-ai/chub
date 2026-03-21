---
name: datastore
description: "Google Cloud Datastore Python client library for entities, keys, queries, batches, and transactions"
metadata:
  languages: "python"
  versions: "2.23.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,datastore,firestore-datastore-mode,nosql,query,transaction"
---

# Google Cloud Datastore Python Client Library

## Golden Rule

Use the official `google-cloud-datastore` package and import it as:

```python
from google.cloud import datastore
```

Authenticate with Application Default Credentials (ADC), not API keys. As of 2026-03-12, PyPI lists `google-cloud-datastore 2.23.0`, but Google Cloud reference subpages under the `latest` docs path still render page versions such as `2.21.0` and `2.22.0`. The examples below stay on the current maintained client surface that is documented in those official references and published on PyPI.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-datastore==2.23.0"
```

Common alternatives:

```bash
uv add "google-cloud-datastore==2.23.0"
poetry add "google-cloud-datastore==2.23.0"
```

If this is a fresh Google Cloud project, enable billing and the Datastore API before debugging client code.

## Authentication And Project Setup

For local development, use ADC:

```bash
gcloud auth application-default login
```

For service-account-based environments, point `GOOGLE_APPLICATION_CREDENTIALS` at the JSON key file:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

The client can usually infer the current project from ADC or environment, but pass `project=` explicitly when your runtime cannot resolve it reliably.

Useful environment variables:

```bash
export GOOGLE_CLOUD_PROJECT="your-gcp-project"
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

Optional library logging can be enabled with:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

## Initialize A Client

Basic client:

```python
from google.cloud import datastore

client = datastore.Client(project="your-gcp-project")
```

With namespace and non-default database:

```python
from google.cloud import datastore

client = datastore.Client(
    project="your-gcp-project",
    namespace="tenant-a",
    database="customer-db",
)
```

With explicit credentials:

```python
from google.cloud import datastore
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = datastore.Client(
    project="your-gcp-project",
    credentials=credentials,
)
```

## Core Usage

### Create keys and entities

Use an incomplete key to let Datastore allocate an ID when you call `put()`. Use `exclude_from_indexes` for long text or fields you do not query on.

```python
from google.cloud import datastore

client = datastore.Client(project="your-gcp-project")

task_key = client.key("Task")
task = datastore.Entity(
    key=task_key,
    exclude_from_indexes=("description",),
)
task.update(
    {
        "title": "Ship docs",
        "description": "Long text that should not be indexed",
        "done": False,
        "priority": 3,
    }
)

client.put(task)
```

### Fetch, update, and delete entities

```python
from google.cloud import datastore

client = datastore.Client(project="your-gcp-project")
task_key = client.key("Task", 123456789)

task = client.get(task_key)
if task is not None:
    task["done"] = True
    client.put(task)

client.delete(task_key)
```

For bulk operations, use `get_multi()`, `put_multi()`, and `delete_multi()` instead of looping one network call at a time.

### Query entities

Use `PropertyFilter` with `Query.add_filter()` for property-based predicates.

```python
from google.cloud import datastore
from google.cloud.datastore.query import PropertyFilter

client = datastore.Client(project="your-gcp-project")
ancestor = client.key("Account", "acct-1")

query = client.query(
    kind="Task",
    ancestor=ancestor,
    order=["priority"],
)
query.add_filter(filter=PropertyFilter("done", "=", False))

for entity in query.fetch(limit=20):
    print(entity["title"], entity.get("priority"))
```

If you need key-based filtering, use `Query.key_filter()`. For composite predicates, use the query filter helpers documented in `google.cloud.datastore.query`.

### Batch writes

Use a batch when you want to send multiple mutations together without transaction semantics:

```python
from google.cloud import datastore

client = datastore.Client(project="your-gcp-project")

task_key = client.key("Task", 1)
audit_key = client.key("AuditLog", 1)

task = datastore.Entity(key=task_key)
task.update({"done": True})

audit = datastore.Entity(key=audit_key)
audit.update({"message": "Task completed"})

with client.batch() as batch:
    batch.put(task)
    batch.put(audit)
```

### Transactions

Use a transaction for read-modify-write workflows that must commit atomically:

```python
from google.cloud import datastore

client = datastore.Client(project="your-gcp-project")
counter_key = client.key("Counter", "default")

with client.transaction():
    counter = client.get(counter_key)
    if counter is None:
        counter = datastore.Entity(key=counter_key)
        counter["value"] = 0

    counter["value"] += 1
    client.put(counter)
```

Inside a transaction, reads and writes use the active transaction automatically unless you override the `transaction=` argument explicitly.

## Local Emulator

Google documents two related emulator paths:

- Firestore emulator in Datastore mode: `gcloud emulators firestore start --database-mode=datastore-mode`
- Legacy Datastore emulator: `gcloud beta emulators datastore start`

When using the Datastore emulator, export the environment variables emitted by `gcloud` before starting your app. The official docs call out these variables:

- `DATASTORE_DATASET`
- `DATASTORE_EMULATOR_HOST`
- `DATASTORE_EMULATOR_HOST_PATH`
- `DATASTORE_HOST`
- `DATASTORE_PROJECT_ID`

Once those are set, `datastore.Client()` will talk to the emulator instead of production.

## Common Pitfalls

- Do not use API keys. The client expects ADC, user credentials, or service-account credentials.
- Do not guess the import path. Use `from google.cloud import datastore`.
- `exclude_from_indexes` matters. Large or unneeded indexed fields can fail writes or increase index cost.
- `eventual=True` is only for eventually consistent reads and cannot be combined with a transaction or `read_time`.
- Ancestor queries are strongly consistent; global non-ancestor queries are not.
- `client.put()` writes the entity state you give it. Fetch first if you need a safe read-modify-write update.
- Namespace and database are part of identity. If reads unexpectedly return nothing, verify `project`, `namespace`, and `database` all match the stored entity.
- Old blog posts often use pre-2.x patterns or older filter APIs. Prefer the `PropertyFilter`-based query examples from the official reference.

## Version-Sensitive Notes

- PyPI currently publishes `2.23.0`.
- The Google Cloud Python reference URL is still the correct canonical docs root, but some Datastore reference pages under `latest` currently display page versions like `2.21.0` or `2.22.0`. Treat that as docs-site version drift, not a reason to downgrade the package automatically.
- If you are migrating old code from `google-cloud-datastore 1.x`, read the official upgrade guide first. The 2.0 migration changed several client and query patterns.
- If your project uses Firestore in Datastore mode rather than legacy Cloud Datastore, keep emulator and operational docs aligned with that mode when testing locally.

## Official URLs

- Docs root: `https://cloud.google.com/python/docs/reference/datastore/latest`
- Client reference: `https://cloud.google.com/python/docs/reference/datastore/latest/google.cloud.datastore.client.Client`
- Query reference: `https://cloud.google.com/python/docs/reference/datastore/latest/google.cloud.datastore.query.Query`
- Filter reference: `https://cloud.google.com/python/docs/reference/datastore/latest/google.cloud.datastore.query.PropertyFilter`
- Upgrade guide: `https://cloud.google.com/python/docs/reference/datastore/latest/upgrading`
- Auth setup: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- Emulator docs: `https://cloud.google.com/datastore/docs/emulator`
- PyPI: `https://pypi.org/project/google-cloud-datastore/`

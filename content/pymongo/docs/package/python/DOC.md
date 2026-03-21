---
name: package
description: "pymongo package guide for Python with MongoClient, AsyncMongoClient, Atlas/local connections, CRUD, transactions, and auth extras"
metadata:
  languages: "python"
  versions: "4.16.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "mongodb,pymongo,bson,gridfs,python,database"
---

# PyMongo Python Package Guide

## Golden Rule

- Use the official `pymongo` package for MongoDB access in Python.
- Treat MongoDB Docs as the canonical guide surface for setup and examples, and use the PyMongo ReadTheDocs site for API details and changelog entries.
- Do not install the third-party `bson` package from PyPI. PyMongo ships its own `bson` package and the PyPI `bson` package is incompatible.

## Version-Sensitive Notes

- The docs URL points to `https://pymongo.readthedocs.io/en/stable/`, which is still official, but the current guide-style docs and examples now live under MongoDB Docs at `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/`.
- MongoDB Docs `Get Started` still says Python `3.8+`, but the live PyPI metadata for `pymongo 4.16.0` requires Python `>=3.9`. Use PyPI for packaging constraints.
- PyMongo `4.16.0` requires `dnspython>=2.6.1` for `mongodb+srv://` support and removes Eventlet support.
- PyMongo `4.14` dropped support for MongoDB Server `4.0`; current 4.16 docs target MongoDB Server `4.2+`.
- Motor is deprecated in favor of the GA PyMongo Async API. Prefer `AsyncMongoClient` for new asyncio code instead of adding `motor`.

## Install

Pin the driver when you need reproducible behavior:

```bash
python -m pip install "pymongo==4.16.0"
```

If you use SRV connection strings, make sure `dnspython` is available:

```bash
python -m pip install "pymongo==4.16.0" "dnspython>=2.6.1"
```

Useful optional extras from PyPI:

```bash
python -m pip install "pymongo[aws]==4.16.0"
python -m pip install "pymongo[gssapi]==4.16.0"
python -m pip install "pymongo[ocsp]==4.16.0"
python -m pip install "pymongo[snappy]==4.16.0"
python -m pip install "pymongo[zstd]==4.16.0"
python -m pip install "pymongo[encryption]==4.16.0"
```

Use these extras only when the deployment or feature requires them:

- `aws`: `MONGODB-AWS` authentication
- `gssapi`: Kerberos / GSSAPI authentication
- `ocsp`: OCSP certificate validation support
- `snappy`, `zstd`: wire compression
- `encryption`: client-side field level encryption / queryable encryption

## Recommended Setup

Keep the connection string in an environment variable and validate connectivity with `ping` immediately after constructing the client. PyMongo client construction does not fail fast on bad credentials or an unavailable server.

```bash
export MONGODB_URI="mongodb://localhost:27017/"
```

```python
import os
from pymongo import MongoClient
from pymongo.errors import ConnectionFailure

uri = os.environ["MONGODB_URI"]
client = MongoClient(uri, serverSelectionTimeoutMS=5000)

try:
    client.admin.command("ping")
except ConnectionFailure:
    client.close()
    raise
```

### Atlas connection

MongoDB's current driver guide shows Atlas examples with Stable API enabled:

```python
import os
import pymongo
from pymongo import MongoClient

uri = os.environ["MONGODB_URI"]

client = MongoClient(
    uri,
    server_api=pymongo.server_api.ServerApi(
        version="1",
        strict=True,
        deprecation_errors=True,
    ),
    serverSelectionTimeoutMS=5000,
)

client.admin.command("ping")
```

### Local development

```python
from pymongo import MongoClient

client = MongoClient("mongodb://localhost:27017/", serverSelectionTimeoutMS=5000)
client.admin.command("ping")
```

### Async setup

Use `AsyncMongoClient` for asyncio applications:

```python
import os
import pymongo
from pymongo import AsyncMongoClient

uri = os.environ["MONGODB_URI"]

client = AsyncMongoClient(
    uri,
    server_api=pymongo.server_api.ServerApi(
        version="1",
        strict=True,
        deprecation_errors=True,
    ),
    serverSelectionTimeoutMS=5000,
)

await client.admin.command("ping")
```

## Core Usage

### Database and collection handles

```python
from pymongo import MongoClient

client = MongoClient("mongodb://localhost:27017/")
db = client["app"]
users = db["users"]
```

### Insert documents

Let PyMongo generate `_id` values unless you have a strong reason to manage them yourself.

```python
user_id = users.insert_one(
    {
        "email": "ada@example.com",
        "name": "Ada Lovelace",
        "roles": ["admin"],
    }
).inserted_id
```

Batch insert:

```python
users.insert_many(
    [
        {"email": "grace@example.com", "name": "Grace Hopper"},
        {"email": "linus@example.com", "name": "Linus Torvalds"},
    ]
)
```

### Query documents

```python
user = users.find_one({"email": "ada@example.com"})

for doc in users.find({"roles": "admin"}):
    print(doc["email"])

count = users.count_documents({})
```

Async query pattern:

```python
user = await users.find_one({"email": "ada@example.com"})

async for doc in users.find({"roles": "admin"}):
    print(doc["email"])
```

### Update documents

Use update operators such as `$set`, and use `upsert=True` only when create-if-missing behavior is intended.

```python
result = users.update_one(
    {"email": "ada@example.com"},
    {"$set": {"name": "Ada Byron"}},
)

users.update_many(
    {"team": "platform"},
    {"$set": {"active": True}},
    upsert=True,
)
```

### Create indexes explicitly

Add the indexes your query patterns actually need. PyMongo exposes standard collection indexes and Atlas Search / Vector Search index management.

```python
users.create_index("email", unique=True)
users.create_index([("created_at", -1)])
```

For Atlas Search or Vector Search indexes:

```python
from pymongo.operations import SearchIndexModel

model = SearchIndexModel(
    definition={
        "fields": [
            {
                "type": "vector",
                "numDimensions": 1536,
                "path": "embedding",
                "similarity": "cosine",
            }
        ]
    },
    name="embedding_index",
    type="vectorSearch",
)

users.create_search_index(model=model)
```

## Transactions and Sessions

Use transactions only when you need multi-document atomicity. Keep the callback idempotent because `with_transaction()` can retry the commit or the entire transaction and may invoke the callback multiple times.

```python
from pymongo import MongoClient
from pymongo.errors import ConnectionFailure, OperationFailure
from pymongo.read_concern import ReadConcern
from pymongo.write_concern import WriteConcern

client = MongoClient("<connection string>")
orders = client["shop"]["orders"]

def write_order(session):
    coll = orders.with_options(
        write_concern=WriteConcern("majority"),
        read_concern=ReadConcern("local"),
    )
    coll.insert_one({"sku": "abc123", "qty": 1}, session=session)

with client.start_session() as session:
    try:
        session.with_transaction(write_order)
    except (ConnectionFailure, OperationFailure):
        raise
```

Important limits from the official docs:

- A `ClientSession` is not thread-safe or fork-safe.
- PyMongo does not support parallel operations inside a single transaction.
- On MongoDB Server `8.0+`, `MongoClient.bulk_write()` can perform writes across multiple namespaces inside one transaction.

## Authentication and Connection Configuration

### URI encoding

Reserved characters in usernames, passwords, and Unix socket paths must be percent-encoded:

```python
from urllib.parse import quote_plus
from pymongo import MongoClient

user = quote_plus("app-user")
password = quote_plus("p@ss:w/rd")
host = "cluster0.example.mongodb.net"

client = MongoClient(f"mongodb://{user}:{password}@{host}")
```

### Supported auth mechanisms

The current MongoDB driver docs list these mechanisms:

- `SCRAM`
- `X.509`
- `AWS IAM`
- `OIDC`
- `LDAP (PLAIN)`
- `Kerberos (GSSAPI)`

For most application code, the mechanism is implied by the deployment's connection string and server configuration. Reach for explicit auth configuration only when the deployment requires it.

### Useful connection options

These options come directly from the current `MongoClient` / `AsyncMongoClient` API docs and matter often in real code:

- `serverSelectionTimeoutMS`: how long server selection can take before failing
- `timeoutMS`: overall per-operation timeout including retries
- `socketTimeoutMS`: how long to wait for a normal database response
- `connectTimeoutMS`: how long to wait while opening a new socket
- `maxPoolSize`, `minPoolSize`, `maxIdleTimeMS`, `maxConnecting`: pool tuning
- `directConnection`: force a direct connection to a single host
- `tz_aware`: decode BSON datetimes as timezone-aware Python datetimes

Example:

```python
from pymongo import MongoClient

client = MongoClient(
    "mongodb://localhost:27017/",
    serverSelectionTimeoutMS=5000,
    timeoutMS=10000,
    connectTimeoutMS=2000,
    maxPoolSize=50,
    tz_aware=True,
)
```

## Files and Binary Data

Use `gridfs` when files exceed MongoDB's 16 MB BSON document limit:

```python
import gridfs
from pymongo import MongoClient

client = MongoClient("<connection string>")
db = client["app"]
bucket = gridfs.GridFSBucket(db)

with bucket.open_upload_stream(
    "report.pdf",
    metadata={"contentType": "application/pdf"},
) as grid_in:
    grid_in.write(b"file-bytes")
```

Async variant:

```python
import gridfs
from pymongo import AsyncMongoClient

client = AsyncMongoClient("<connection string>")
db = client["app"]
bucket = gridfs.AsyncGridFSBucket(db)
```

## Common Pitfalls

- Do not `pip install bson`. Use the `bson` module that ships with `pymongo`.
- Do not assume client construction proves connectivity. Run `ping` early.
- Do not copy MongoDB docs' Python `3.8+` prerequisite blindly for packaging. PyPI for `pymongo 4.16.0` requires Python `>=3.9`.
- Do not forget `dnspython` when using `mongodb+srv://`. SRV URIs depend on DNS resolution and implicitly enable TLS.
- Do not forget to percent-encode reserved characters in usernames and passwords inside MongoDB URIs.
- Do not share one transaction across concurrent operations. Sessions are sequential.
- Do not rely on Eventlet with PyMongo `4.16+`; support was removed.
- Do not add `motor` for new asyncio work unless you are maintaining existing code. MongoDB now recommends PyMongo Async.
- In PyMongo `4.x`, `directConnection` defaults to `False`. If you truly need a direct single-host connection, set it explicitly.
- Close clients when they are no longer needed: `client.close()` or `await client.close()`.

## Practical Defaults For Agents

- Default to `MongoClient` unless the surrounding app is already asyncio-native.
- Store the connection string in `MONGODB_URI`.
- Create clients once per process, not per request.
- Add `serverSelectionTimeoutMS` on startup so connection failures surface quickly.
- Create required indexes in application setup or migrations, not lazily in hot paths.
- Keep transaction callbacks idempotent because retries can re-run them.

## Official Sources Used For This Entry

- MongoDB driver guide: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/`
- Get started: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/get-started/`
- Connect: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/connect/`
- Authentication: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/security/authentication/`
- CRUD insert/query/update: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/crud/`
- Transactions: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/crud/transactions/`
- Indexes: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/indexes/`
- GridFS: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/crud/gridfs/`
- PyMongo API docs: `https://pymongo.readthedocs.io/en/stable/`
- PyMongo changelog: `https://pymongo.readthedocs.io/en/stable/changelog.html`
- PyPI package metadata: `https://pypi.org/project/pymongo/`

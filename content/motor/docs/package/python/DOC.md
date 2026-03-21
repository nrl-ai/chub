---
name: package
description: "Motor async MongoDB driver for Python projects using asyncio or Tornado"
metadata:
  languages: "python"
  versions: "3.7.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "motor,mongodb,asyncio,tornado,pymongo,database"
---

# Motor Python Package Guide

## Golden Rule

Use `motor` `3.7.1` when you are maintaining an existing Motor codebase or you still need Tornado integration. For new asyncio-only projects, plan around the PyMongo Async API instead: upstream says Motor will be deprecated on May 14, 2026, with only critical bug fixes through May 14, 2027.

## Install

Most projects only need the base package:

```bash
python -m pip install "motor==3.7.1"
```

Common alternatives:

```bash
uv add "motor==3.7.1"
poetry add "motor==3.7.1"
```

Optional extras from the official PyPI metadata:

```bash
python -m pip install "motor[srv]"
python -m pip install "motor[gssapi]"
python -m pip install "motor[aws]"
python -m pip install "motor[ocsp]"
python -m pip install "motor[snappy]"
python -m pip install "motor[zstd]"
python -m pip install "motor[encryption]"
```

## Initialize One Client Per Process

Motor clients connect on demand, so fail fast at startup with a `ping` instead of waiting for the first request path to discover a bad URI or auth problem.

```python
import os

from motor.motor_asyncio import AsyncIOMotorClient

MONGODB_URI = os.environ["MONGODB_URI"]
MONGODB_DB = os.getenv("MONGODB_DB", "app")

client = AsyncIOMotorClient(
    MONGODB_URI,
    appname="my-service",
    serverSelectionTimeoutMS=5000,
)
db = client[MONGODB_DB]

async def startup() -> None:
    await client.admin.command("ping")

async def shutdown() -> None:
    client.close()
```

Guidelines:

- Create the client once at app startup and reuse it.
- Close it during process shutdown.
- Do not create a new client per request, task, or repository object.

## Core CRUD Pattern

`find()` returns a cursor immediately and does not perform I/O until you iterate it or call cursor helpers like `to_list()`. Most other collection operations are coroutines and must be awaited.

```python
from bson import ObjectId

async def create_user(email: str) -> ObjectId:
    result = await db.users.insert_one({"email": email, "active": True})
    return result.inserted_id

async def get_user(user_id: ObjectId) -> dict | None:
    return await db.users.find_one({"_id": user_id})

async def list_active_users(limit: int = 100) -> list[dict]:
    cursor = db.users.find({"active": True}).sort("email", 1)
    return await cursor.to_list(length=limit)

async def deactivate_user(user_id: ObjectId) -> None:
    await db.users.update_one({"_id": user_id}, {"$set": {"active": False}})

async def delete_user(user_id: ObjectId) -> None:
    await db.users.delete_one({"_id": user_id})
```

Streaming a cursor:

```python
async def iter_active_users() -> None:
    async for user in db.users.find({"active": True}):
        print(user["email"])
```

## Sessions And Transactions

Motor sessions are awaited when created, but `start_transaction()` itself is not awaited.

```python
async def transfer(from_id: ObjectId, to_id: ObjectId, amount: int) -> None:
    async with await client.start_session() as session:
        async with session.start_transaction():
            await db.accounts.update_one(
                {"_id": from_id},
                {"$inc": {"balance": -amount}},
                session=session,
            )
            await db.accounts.update_one(
                {"_id": to_id},
                {"$inc": {"balance": amount}},
                session=session,
            )
```

Pass the `session=` argument to every operation that belongs inside the transaction.

## Config And Authentication

Prefer a standard MongoDB connection URI and keep credentials in environment variables or a secret manager.

```python
import os
from urllib.parse import quote_plus

from motor.motor_asyncio import AsyncIOMotorClient

username = quote_plus(os.environ["MONGODB_USER"])
password = quote_plus(os.environ["MONGODB_PASSWORD"])

uri = (
    f"mongodb://{username}:{password}@db.example.net:27017/app"
    "?authSource=admin"
    "&replicaSet=rs0"
    "&retryWrites=true"
    "&w=majority"
)

client = AsyncIOMotorClient(uri, tls=True)
```

Practical notes:

- Percent-encode usernames and passwords before putting them in the URI.
- Set `authSource=` when the user is stored in a different database, commonly `admin`.
- Use `mongodb+srv://...` for Atlas or SRV-based deployments.
- Keep timeouts explicit for production paths: `serverSelectionTimeoutMS`, `socketTimeoutMS`, and `connectTimeoutMS` are often worth setting deliberately.
- If you use optional auth or transport features, install the matching extra before assuming the URI will work.

## Tornado-Specific Entry Point

If the codebase is still on Tornado rather than native `asyncio`, use the Tornado client class instead of the asyncio variant:

```python
import os

from motor.motor_tornado import MotorClient

client = MotorClient(os.environ["MONGODB_URI"])
db = client[os.getenv("MONGODB_DB", "app")]
```

Keep the rest of the codebase consistent with that runtime. Do not mix Motor's Tornado and asyncio client styles in the same application layer.

## Common Pitfalls

- Do not write `await db.users.find(...)`. `find()` is synchronous and returns a cursor.
- Do not forget the first real operation triggers connection and authentication. Call `await client.admin.command("ping")` during startup if you want early failure.
- Do not create clients per request. Reuse one client for the app lifetime.
- `cursor.to_list()` accepts `length=None` in Motor 3.6+, but bounded lengths are safer for memory and still match most existing examples.
- `aggregate()` returns a cursor lazily. If the pipeline uses `$out` or `$merge`, you must iterate the cursor or convert it to a list for the operation to run.
- Close the client on shutdown so Motor can clean up connection pools and monitor threads.
- The PyMongo Async API is similar but not identical. Migration changes include imports, missing `each()`, and some `to_list()` behavior differences.

## Version-Sensitive Notes

- `3.7.1` contains documentation-only changes. If your project already runs on `3.7.0`, there is no new runtime feature surface in `3.7.1`.
- `3.7.0` added support for PyMongo `4.10` and dropped Python `3.8` and MongoDB `3.6`.
- `3.6.0` added support for MongoDB `8.0`, raised the PyMongo floor to `4.9`, and made `MotorCursor.to_list()`'s `length` parameter optional.
- The `latest` docs have already moved past stable and describe `3.7.2.dev0` or newer development work. For package-specific guidance against `3.7.1`, use the `/en/stable/` docs URLs instead of `/en/latest/`.
- Upstream now treats PyMongo Async as the forward path. If you are starting greenfield asyncio code and do not need Tornado compatibility, verify whether `pymongo.AsyncMongoClient` is the better fit before adding Motor.

## Official Links

- Docs: `https://motor.readthedocs.io/en/stable/`
- Requirements and compatibility: `https://motor.readthedocs.io/en/stable/requirements.html`
- Asyncio tutorial: `https://motor.readthedocs.io/en/stable/tutorial-asyncio.html`
- Authentication examples: `https://motor.readthedocs.io/en/stable/examples/authentication.html`
- Changelog: `https://motor.readthedocs.io/en/stable/changelog.html`
- Migration guide to PyMongo Async: `https://www.mongodb.com/docs/languages/python/pymongo-driver/current/reference/migration/`
- PyPI package page: `https://pypi.org/project/motor/`

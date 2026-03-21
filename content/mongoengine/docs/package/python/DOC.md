---
name: package
description: "MongoEngine ODM for modeling, validating, and querying MongoDB documents from Python"
metadata:
  languages: "python"
  versions: "0.29.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "mongoengine,mongodb,odm,pymongo,nosql"
---

# MongoEngine Python Package Guide

## Golden Rule

Use `mongoengine` as a thin ODM on top of MongoDB and connect with a normal MongoDB URI. Be explicit about aliases, authentication database, indexes, and dereferencing behavior. The official docs site is still branded as `0.29.0`, while PyPI currently publishes `0.29.3`, so treat the guide pages as canonical for usage patterns and cross-check release-specific behavior against PyPI and the changelog.

## Install

Pin the version your project expects:

```bash
python -m pip install "mongoengine==0.29.3"
```

Common alternatives:

```bash
uv add "mongoengine==0.29.3"
poetry add "mongoengine==0.29.3"
```

For tests using an in-memory mock server:

```bash
python -m pip install "mongoengine==0.29.3" mongomock
```

## Connect And Authenticate

Use `connect()` once during app startup. Prefer a MongoDB URI so auth, replica set, TLS, retry settings, and timeouts stay in one place.

```python
from mongoengine import connect

connect(
    db="appdb",
    alias="default",
    host="mongodb://appuser:secret@localhost:27017/appdb?authSource=admin",
)
```

Notes:

- `alias="default"` is the connection name used unless a document specifies a different `db_alias`.
- If you pass both a URI and separate parameters like `db`, `username`, or `password`, MongoEngine follows the URI values.
- Be explicit about `authSource` when the user is stored outside the target database. Since `0.26.0`, `connect()` no longer silently defaults `authentication_source` to `admin`.

### Multiple databases

Register separate aliases and bind documents with `meta["db_alias"]`:

```python
from mongoengine import Document, StringField, connect

connect(
    db="appdb",
    alias="default",
    host="mongodb://localhost:27017/appdb",
)
connect(
    db="analytics",
    alias="analytics",
    host="mongodb://localhost:27017/analytics",
)

class AuditEvent(Document):
    event = StringField(required=True)

    meta = {
        "db_alias": "analytics",
        "collection": "audit_events",
    }
```

### Testing with `mongomock`

Do not use the old `mongomock://` URI form. Current MongoEngine expects an explicit client class:

```python
import mongomock
from mongoengine import connect, disconnect

disconnect(alias="default")
connect(
    "mongoenginetest",
    alias="default",
    host="mongodb://localhost",
    mongo_client_class=mongomock.MongoClient,
)
```

Call `disconnect()` before reconnecting an alias in tests, otherwise stale connections can leak across test cases.

## Define Documents

Model documents with field classes and `meta` options for collection names, indexes, ordering, and strictness:

```python
from mongoengine import (
    CASCADE,
    DateTimeField,
    Document,
    IntField,
    ListField,
    ReferenceField,
    StringField,
)

class User(Document):
    email = StringField(required=True, unique=True)

class Post(Document):
    title = StringField(required=True, max_length=200)
    status = StringField(choices=("draft", "published"), default="draft")
    tags = ListField(StringField(max_length=50))
    author = ReferenceField(User, required=True, reverse_delete_rule=CASCADE)
    published_at = DateTimeField()
    view_count = IntField(default=0)

    meta = {
        "collection": "posts",
        "indexes": [
            "status",
            "-published_at",
            {"fields": ("author", "status"), "unique": False},
        ],
        "ordering": ["-published_at"],
        "strict": True,
    }
```

Useful upstream behaviors:

- `reverse_delete_rule` matters for document relationships. Define it intentionally instead of relying on orphaned references.
- `strict=True` is the safer default. `strict=False` accepts undeclared fields, which is useful for legacy collections but also hides typos in agent-written code.
- MongoDB stores datetimes with millisecond precision, so `DateTimeField` values can lose microseconds.

### Validation hooks

Use field validation for shape constraints and `clean()` for cross-field validation or normalization that should run on `save()`:

```python
from mongoengine import Document, IntField, StringField, ValidationError

class Product(Document):
    name = StringField(required=True)
    price_cents = IntField(min_value=0, required=True)
    currency = StringField(default="USD", choices=("USD", "EUR"))

    def clean(self):
        if self.currency == "USD" and self.price_cents % 5:
            raise ValidationError("USD prices must be stored in 5-cent increments")
```

`save()` runs validation by default. Raw update operations do not.

## Core Usage

### Create and save a document

```python
from datetime import datetime, timezone

post = Post(
    title="MongoEngine notes",
    author=user,
    tags=["mongodb", "odm"],
    status="published",
    published_at=datetime.now(timezone.utc),
)
post.save()
```

### Query documents

```python
published = Post.objects(status="published").order_by("-published_at")

for post in published[:20]:
    print(post.title)
```

Compose more complex filters with `Q` objects:

```python
from mongoengine.queryset.visitor import Q

posts = Post.objects(
    Q(status="published") & (Q(tags="mongodb") | Q(title__icontains="mongo"))
)
```

Common query operators map to Django-style suffixes such as `__in`, `__ne`, `__lte`, `__icontains`, and `__exists`.

### Atomic updates

Prefer update operators when you do not need full document validation:

```python
updated = Post.objects(id=post.id).update_one(
    set__status="published",
    inc__view_count=1,
)
```

If you need the updated document back:

```python
post = Post.objects(id=post.id).modify(
    new=True,
    set__status="published",
)
```

`modify()` and `update*()` operate directly in MongoDB and bypass `clean()` and model validation.

### Reduce dereferencing overhead

`ReferenceField` dereferences related documents automatically, which is convenient but can create extra queries. For hot paths:

- use `select_related()` when you want eager dereferencing
- use `no_dereference()` when you only need raw references
- use `LazyReferenceField` instead of `ReferenceField` when you want explicit fetches

## Indexing And Collection Options

MongoEngine lets you declare indexes in `meta["indexes"]`, but index creation behavior matters:

```python
class Event(Document):
    event_type = StringField(required=True)
    created_at = DateTimeField(required=True)

    meta = {
        "indexes": [
            "event_type",
            {"fields": ("event_type", "-created_at")},
        ],
        "index_background": True,
    }
```

Operational notes:

- Since `0.26.0`, `.save()` no longer calls `ensure_indexes()` automatically.
- If your app expects indexes to exist before writes, create them at startup with `YourDocument.ensure_indexes()` or enable `meta["auto_create_index_on_save"]` deliberately.
- `compare_indexes()` is useful in deployment checks when you need to detect drift between declared indexes and the live collection.
- MongoEngine `0.29.0` added support for timeseries collection options via document metadata. Use this only when the backing MongoDB deployment supports timeseries collections.

## Common Pitfalls

- `update()`, `update_one()`, and `modify()` bypass validation and `clean()`. Use them for atomic operations, not for user-input validation paths.
- `ReferenceField` can cause hidden N+1 behavior due to automatic dereferencing. Use `LazyReferenceField`, `select_related()`, or `no_dereference()` deliberately.
- Delete rules only apply when the related models are imported and registered. If your app lazily imports model modules, cascade behavior can appear inconsistent.
- `strict=False` is not a free schema-migration tool. It prevents crashes on unknown fields, but it also makes misspelled field names harder to detect.
- Datetime precision is milliseconds, not microseconds.
- Reusing the same alias across tests without `disconnect()` often leads to confusing state leakage.

## Version-Sensitive Notes

- PyPI currently lists `0.29.3`, released on March 10, 2026. The Read the Docs site still shows `MongoEngine 0.29.0 documentation`, so expect some release lag in headings and changelog coverage.
- `0.29.1` fixed compatibility issues with newer PyMongo `4.8` and `4.9` releases. If older examples fail with recent PyMongo, upgrade before debugging query code.
- `0.29.0` added timeseries collection support and `array_filters` support in `modify()`. It also changed `ListField(EmbeddedDocumentField(...))` patterns for custom embedded document classes.
- `0.28.0` changed `no_dereference` usage so the context manager is applied from the queryset object rather than imported as a standalone helper.
- `0.27.0` removed the legacy `mongomock://` and `is_mock` connection styles. Use `mongo_client_class=mongomock.MongoClient`.
- `0.26.0` changed two behaviors that still break copied blog examples:
  - `save()` no longer auto-creates indexes
  - `connect()` no longer defaults `authentication_source` to `admin`

## Official Sources

- Docs root: `https://mongoengine-odm.readthedocs.io/`
- Connecting guide: `https://mongoengine-odm.readthedocs.io/guide/connecting.html`
- Defining documents guide: `https://mongoengine-odm.readthedocs.io/guide/defining-documents.html`
- Querying guide: `https://mongoengine-odm.readthedocs.io/guide/querying.html`
- Validation guide: `https://mongoengine-odm.readthedocs.io/guide/validation.html`
- Changelog: `https://mongoengine-odm.readthedocs.io/changelog.html`
- PyPI: `https://pypi.org/project/mongoengine/`

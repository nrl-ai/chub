---
name: package
description: "factory-boy package guide for Python test factories, ORM integration, and reproducible fixtures"
metadata:
  languages: "python"
  versions: "3.3.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "factory-boy,factory,factories,testing,fixtures,django,sqlalchemy,mongoengine"
---

# factory-boy Python Package Guide

## Golden Rule

Use `factory-boy` for test data generation and keep persistence behavior explicit. Install the PyPI package `factory-boy`, import `factory`, and use the ORM-specific base class when you expect `.create()` to hit a database.

## Install

Use the PyPI package name from your dependency file:

```bash
python -m pip install "factory-boy==3.3.3"
```

Common alternatives:

```bash
uv add "factory-boy==3.3.3"
poetry add "factory-boy==3.3.3"
```

Upstream docs also show `pip install factory_boy`. That works because pip normalizes hyphens and underscores, but the canonical PyPI project name is `factory-boy`.

## Initialize A Basic Factory

For plain Python objects, subclass `factory.Factory` and define `Meta.model` plus field declarations:

```python
from dataclasses import dataclass

import factory

@dataclass
class User:
    username: str
    email: str
    is_admin: bool = False

class UserFactory(factory.Factory):
    class Meta:
        model = User

    username = factory.Sequence(lambda n: f"user{n}")
    email = factory.LazyAttribute(lambda obj: f"{obj.username}@example.com")
    is_admin = False
```

Use explicit strategies instead of relying on the factory call shortcut:

```python
user = UserFactory.build()
users = UserFactory.build_batch(3)
stub = UserFactory.stub()
override = UserFactory.build(username="root", is_admin=True)
```

For a plain `factory.Factory`, `.create()` still builds the object, but it does not add ORM persistence by itself. Database persistence only comes from subclasses such as `DjangoModelFactory`, `SQLAlchemyModelFactory`, or `MongoEngineFactory`.

## Core Usage Patterns

### Related objects

Use `SubFactory` for forward relations and `SelfAttribute` or `LazyAttribute` when values depend on a parent factory:

```python
import factory

class CountryFactory(factory.Factory):
    class Meta:
        model = Country

    name = factory.Iterator(["France", "Italy", "Spain"])
    lang = factory.Iterator(["fr", "it", "es"])

class UserFactory(factory.Factory):
    class Meta:
        model = User

    username = factory.Sequence(lambda n: f"user{n}")
    country = factory.SubFactory(CountryFactory)
    lang = factory.SelfAttribute("country.lang")
```

### Post-generation hooks

Use `@factory.post_generation` when the relation can only be set after the base object exists:

```python
class UserFactory(factory.django.DjangoModelFactory):
    class Meta:
        model = User
        skip_postgeneration_save = True

    username = factory.Sequence(lambda n: f"user{n}")

    @factory.post_generation
    def groups(self, create, extracted, **kwargs):
        if not create or not extracted:
            return
        self.groups.add(*extracted)
```

For SQLAlchemy many-to-many collections, append instead of calling `.add()`.

### Reproducible randomness

`factory.Faker` and `factory.fuzzy` share the same random state. Seed it in test setup when you want stable failures:

```python
import factory.random

factory.random.reseed_random("tests")
UserFactory.reset_sequence()
```

If you want to replay a failing run exactly, store and restore `factory.random.get_random_state()` / `set_random_state()`.

### Convert a factory to a payload dict

When you need request payloads instead of model instances, build `dict` directly:

```python
payload = factory.build(dict, FACTORY_CLASS=UserFactory)
```

This is useful for API tests where you want factory declarations but not ORM objects.

## ORM-Specific Setup

### Django

Use `factory.django.DjangoModelFactory` for Django models:

```python
import factory
from myapp import models

class UserFactory(factory.django.DjangoModelFactory):
    class Meta:
        model = models.User
        django_get_or_create = ("username",)
        skip_postgeneration_save = True

    username = factory.Sequence(lambda n: f"user{n}")
    password = factory.django.Password("pw")
```

Key Django options:

- `django_get_or_create`: fetches an existing row instead of always creating one
- `database`: routes factory queries to a named Django database
- `skip_postgeneration_save`: avoids the transitional extra `save()` after post-generation hooks when you do not need it

If model signals interfere with `RelatedFactory` or `post_generation`, wrap the factory with `@factory.django.mute_signals(...)` or use the context manager form.

### SQLAlchemy

Use `factory.alchemy.SQLAlchemyModelFactory` and configure exactly one session source:

```python
import factory

from myapp.db import Session
from myapp.models import User

class UserFactory(factory.alchemy.SQLAlchemyModelFactory):
    class Meta:
        model = User
        sqlalchemy_session = Session
        sqlalchemy_session_persistence = "flush"

    username = factory.Sequence(lambda n: f"user{n}")
```

Key SQLAlchemy options:

- `sqlalchemy_session`: shared session or `scoped_session`
- `sqlalchemy_session_factory`: callable returning a session; new in the 3.3.x line
- `sqlalchemy_session_persistence`: `None`, `"flush"`, or `"commit"`
- `sqlalchemy_get_or_create`: fetches existing rows instead of inserting duplicates

If you use `scoped_session`, assign the `scoped_session` object itself in the factory and configure or remove sessions in test setup and teardown. Do not call `Session()` in the factory declaration.

### MongoEngine

Use `factory.mongoengine.MongoEngineFactory` for MongoEngine documents:

```python
import factory

class PersonFactory(factory.mongoengine.MongoEngineFactory):
    class Meta:
        model = Person

    name = factory.Sequence(lambda n: f"name{n}")
```

`.create()` saves `mongoengine.Document` instances. For `EmbeddedDocument`, `.create()` does not persist anything, which is usually what you want before nesting through `SubFactory`.

## Configuration And Auth

`factory-boy` has no auth layer and no required environment variables. Configuration lives in factory declarations and in the surrounding test stack:

- `Meta.model` points at the target class
- `Meta.inline_args` is useful for constructors that require positional arguments
- `Meta.exclude` can keep helper attributes out of the final constructor call
- ORM configuration lives in Django database routing or SQLAlchemy session settings

Any credentials, database URLs, or app settings come from Django, SQLAlchemy, MongoEngine, or your own app, not from `factory-boy` itself.

## Common Pitfalls

- The install name is `factory-boy`, but the import is `factory`.
- `factory.Factory` does not add persistence. Use an ORM-specific subclass when tests expect rows or documents to be saved.
- `django_get_or_create` and `sqlalchemy_get_or_create` do not update an existing object with new override values; they only fetch the existing row.
- `RelatedFactory` and `post_generation` can trigger extra Django saves and signal behavior. For 3.3.x, move toward `skip_postgeneration_save = True` when the extra save is unnecessary.
- `sqlalchemy_session` and `sqlalchemy_session_factory` are mutually exclusive.
- Random Faker values and sequences make failures hard to replay if you do not reseed randomness or reset sequences in test setup.
- The dynamic helpers `factory.build()`, `factory.create()`, and `factory.stub()` default to plain `factory.Factory`. Pass `FACTORY_CLASS=...` when you need Django or SQLAlchemy semantics.
- `factory.use_strategy()` is deprecated. Prefer explicit `.build()`, `.create()`, `.stub()` calls or a factory `Meta.strategy` setting.

## Version-Sensitive Notes For 3.3.3

- As of March 12, 2026, PyPI still lists `3.3.3` as the latest release.
- `3.3.3` publishes type annotations. If your editor or type checker behavior changed compared with older examples, that is expected.
- `3.3.2` added Python 3.13 support, and `3.3.1` added Python 3.12 support.
- `3.3.1` also fixed the requirement that previously forced `sqlalchemy_session` even when `sqlalchemy_session_factory` was provided.
- `3.3.0` added `factory.django.Password` and `sqlalchemy_session_factory`.
- The Django extra-save behavior after post-generation hooks is on a deprecation path. In 3.3.x, upstream recommends either setting `skip_postgeneration_save = True` when the extra save is unnecessary, or saving explicitly in the hook or `_after_postgeneration` when it is necessary.

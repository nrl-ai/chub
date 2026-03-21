---
name: package
description: "polyfactory package guide for Python mock-data factories across dataclasses, Pydantic, TypedDict, attrs, msgspec, and SQLAlchemy"
metadata:
  languages: "python"
  versions: "3.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "polyfactory,python,testing,factories,mock-data,pydantic,sqlalchemy"
---

# polyfactory Python Package Guide

## Golden Rule

Use `polyfactory` when you need typed test data from model definitions, not hand-written fixtures. Define one factory per model, call `.build()` for one instance or `.batch()` for many, and make version-sensitive defaults explicit when upgrading from the old `pydantic-factories` / Polyfactory v2 behavior.

## Install

Base install:

```bash
python -m pip install "polyfactory==3.3.0"
```

Optional integration extras from PyPI:

```bash
python -m pip install "polyfactory[pydantic]==3.3.0"
python -m pip install "polyfactory[attrs]==3.3.0"
python -m pip install "polyfactory[msgspec]==3.3.0"
python -m pip install "polyfactory[sqlalchemy]==3.3.0"
python -m pip install "polyfactory[beanie]==3.3.0"
python -m pip install "polyfactory[odmantic]==3.3.0"
python -m pip install "polyfactory[full]==3.3.0"
```

Common package managers:

```bash
uv add "polyfactory==3.3.0"
poetry add "polyfactory==3.3.0"
```

## Choose The Right Factory Base

- `DataclassFactory[T]`: stdlib dataclasses and compatible dataclass implementations
- `TypedDictFactory[T]`: `TypedDict`
- `ModelFactory[T]`: Pydantic models
- `AttrsFactory[T]`: attrs models
- `SQLAlchemyFactory[T]`: SQLAlchemy declarative models

Polyfactory also ships library-specific factories for `msgspec`, Beanie, and Odmantic.

## Quick Start

For simple models, subclass the right factory and call `.build()`:

```python
from dataclasses import dataclass

from polyfactory.factories import DataclassFactory

@dataclass
class User:
    id: int
    name: str
    is_active: bool

class UserFactory(DataclassFactory[User]):
    ...

user = UserFactory.build()
users = UserFactory.batch(size=3)
custom = UserFactory.build(name="Ada Lovelace")
```

As of Polyfactory `2.13.0+`, you can omit `__model__` on declarative factories if the generic parameter already identifies the model.

Polyfactory will infer nested models and many standard Python types from annotations. If you need a factory class dynamically, use `create_factory()`:

```python
pet_factory = DataclassFactory.create_factory(model=User)
pet = pet_factory.build()
```

## Core Customization

### Deterministic test data

Use the factory-scoped RNG instead of the stdlib global random state:

```python
from polyfactory.factories import DataclassFactory

class UserFactory(DataclassFactory[User]):
    __random_seed__ = 7
```

`__random_seed__` makes factory output deterministic and also affects the attached Faker instance. If you need a fully custom RNG or locale-aware faker, set `__random__` or `__faker__` explicitly.

### Control specific fields

The main helpers are:

- `Use(...)`: call a function at build time
- `Ignore()`: omit a field from generated output
- `Require()`: force callers to pass a value to `.build(...)`
- `PostGenerated(...)`: derive one field from already-generated values

Example:

```python
from dataclasses import dataclass

from polyfactory import Require, Use
from polyfactory.factories import DataclassFactory

@dataclass
class Payment:
    amount: int
    currency: str
    reference: str

class PaymentFactory(DataclassFactory[Payment]):
    amount = Require()
    currency = Use(lambda: "USD")

payment = PaymentFactory.build(amount=500)
```

Factories can also be used as fields directly, and `Use(OtherFactory.batch, size=...)` is the standard way to generate nested lists.

### Defaults, optional fields, and collection length

Useful config flags:

- `__use_defaults__ = True`: prefer model default values instead of random ones
- `__allow_none_optionals__ = False`: force `Optional[...]` fields to get actual generated values
- `__randomize_collection_length__ = True`: vary collection sizes
- `__min_collection_length__` / `__max_collection_length__`: bound randomized collection sizes

Example:

```python
class UserFactory(DataclassFactory[User]):
    __allow_none_optionals__ = False
    __randomize_collection_length__ = True
    __min_collection_length__ = 1
    __max_collection_length__ = 3
```

## Pydantic Usage

Use `ModelFactory` for Pydantic models:

```python
from pydantic import BaseModel, Field

from polyfactory.factories.pydantic_factory import ModelFactory

class Customer(BaseModel):
    email: str
    country: str = Field(examples=["US", "DE", "JP"])

class CustomerFactory(ModelFactory[Customer]):
    __use_examples__ = True

customer = CustomerFactory.build()
```

Version-sensitive Pydantic notes:

- `__use_examples__ = True` uses Pydantic v2 field `examples` when present.
- `__by_name__ = True` is the current switch for validation-alias handling with `validation_alias` / `AliasPath`.
- The changelog shows `by_name` support landed in `3.2.0`, so this is safe for `3.3.0`.

## SQLAlchemy Usage

Use `SQLAlchemyFactory` when you need ORM instances and optionally persisted rows:

```python
from sqlalchemy.orm import Session

from polyfactory.factories.sqlalchemy_factory import (
    SQLAlchemyFactory,
    SQLAlchemyPersistenceMethod,
)

class AuthorFactory(SQLAlchemyFactory[Author]):
    __set_relationships__ = True
    __session__ = Session(engine)
    __persistence_method__ = SQLAlchemyPersistenceMethod.FLUSH

author = AuthorFactory.create_sync()
```

Important behavior:

- `build()` creates ORM objects without persisting them.
- `create_sync()` uses `__session__`; `create_async()` uses `__async_session__`.
- Current docs say the default persistence behavior commits unless you switch to `SQLAlchemyPersistenceMethod.FLUSH`.
- Current docs also say `__set_relationships__` and `__set_association_proxy__` default to `True` in v3, which is a behavioral change from older defaults.

The official changelog for `3.3.0` adds SQLAlchemy computed-field support and the `__persistence_method__` configuration.

## Custom Types And Shared Base Factories

If your model includes unsupported custom types, extend the provider map or register a provider globally:

```python
from polyfactory.factories.base import BaseFactory

BaseFactory.add_provider(CustomSecret, lambda: CustomSecret("token"))
```

For broader reuse, create a custom base factory with `__is_base_factory__ = True` and override `get_provider_map()`. If that base class adds extra config knobs, extend `__config_keys__` so concrete factories inherit them correctly.

## Pytest Integration

Polyfactory can register factories as pytest fixtures:

```python
from polyfactory.pytest_plugin import register_fixture

@register_fixture
class UserFactory(DataclassFactory[User]):
    ...
```

This exposes a fixture like `user_factory` while leaving the class usable directly.

## Coverage Generation

Use `coverage()` when you want the smallest set of examples that exercises unions and literal choices instead of purely random batches:

```python
cases = list(ProfileFactory.coverage())
```

This is especially useful for parameterized tests over discriminated or union-like models.

Known limitations in the official docs:

- recursive models can raise `RecursionError`
- `__min_collection_length__` and `__max_collection_length__` are ignored by `coverage()`

## Config And Environment

Polyfactory is a local library with no auth flow or network configuration. The main setup decisions are:

- which optional extra to install
- whether tests need deterministic randomness
- whether ORM factories should persist with `flush()` or `commit()`
- whether shared base factories should centralize custom type providers

## Common Pitfalls

- In v3, `__check_model__` defaults to `True` for `Use`, `PostGenerated`, `Ignore`, and `Require` fields. Unknown helper fields now raise instead of silently passing.
- `__use_defaults__` does nothing for `TypedDictFactory`, because `TypedDict` has no field defaults.
- Use `cls.__random__` or `__random_seed__`, not global `random`, if you want deterministic tests.
- For async data needed by a field, resolve it before building and pass the value into `.build(...)`; the docs recommend not doing async fetches inside field declarations.
- attrs validators are not currently supported by `AttrsFactory`.
- If you are migrating from `pydantic-factories`, remember the package was renamed to `polyfactory` in v2.

## Version-Sensitive Notes For 3.3.0

- PyPI currently lists `3.3.0` as the latest release, dated February 22, 2026.
- `3.3.0` adds SQLAlchemy computed-field support and `__persistence_method__`.
- `3.2.0` added Pydantic `__by_name__` support for validation aliases.
- v3 changes that matter during upgrades:
  - `__check_model__` now defaults to `True`
  - `__set_relationships__` now defaults to `True`
  - SQLAlchemy association-proxy generation defaults to enabled in current v3 docs

## Official Sources

- Docs: `https://polyfactory.litestar.dev/latest/`
- Usage guide: `https://polyfactory.litestar.dev/latest/usage/declaring_factories.html`
- Configuration: `https://polyfactory.litestar.dev/latest/usage/configuration.html`
- Fields: `https://polyfactory.litestar.dev/latest/usage/fields.html`
- Custom types: `https://polyfactory.litestar.dev/latest/usage/handling_custom_types.html`
- SQLAlchemy factory: `https://polyfactory.litestar.dev/latest/usage/library_factories/sqlalchemy_factory.html`
- Coverage: `https://polyfactory.litestar.dev/latest/usage/model_coverage.html`
- Fixtures: `https://polyfactory.litestar.dev/latest/usage/fixtures.html`
- Changelog: `https://polyfactory.litestar.dev/latest/changelog.html`
- PyPI: `https://pypi.org/project/polyfactory/`

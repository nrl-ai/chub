---
name: package
description: "attrs package guide for Python declarative classes, validators, converters, and serialization helpers"
metadata:
  languages: "python"
  versions: "25.4.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "attrs,python,dataclass,validation,serialization"
---

# attrs Python Package Guide

## What It Is

`attrs` is a Python library for defining classes without hand-writing boilerplate like `__init__`, `__repr__`, equality, ordering, and optional immutability. For new code, the upstream docs recommend the modern `attrs` namespace with `attrs.define()` and `attrs.field()`.

## Installation

```bash
python -m pip install attrs==25.4.0
```

With Poetry:

```bash
poetry add attrs==25.4.0
```

With uv:

```bash
uv add attrs==25.4.0
```

## Initialize And Define Classes

Use the modern API for new code:

```python
from attrs import Factory, define, field

@define
class User:
    id: int
    email: str
    tags: list[str] = Factory(list)
    active: bool = True
```

`@define` defaults to slotted classes (`slots=True`) and generates the initializer and common dunder methods automatically.

Legacy `attr.s` / `attr.ib` code is still valid and supported, but prefer `from attrs import ...` in new code.

## Core Usage

### Validators And Converters

Converters run before validators, so normalize incoming data first and validate the normalized value second.

```python
import attrs
from attrs import define, field

@define
class Config:
    retries: int = field(
        converter=int,
        validator=attrs.validators.ge(0),
    )
    timeout_seconds: float = field(
        converter=float,
        validator=attrs.validators.gt(0),
    )
```

This accepts string inputs like `"3"` and stores them as typed values.

### Frozen Classes

Use `@frozen` or `@define(frozen=True)` when instances should be immutable and hashable-by-default behavior is appropriate for your model.

```python
from attrs import frozen

@frozen
class Point:
    x: int
    y: int
```

### Serialize Or Copy Instances

```python
import attrs
from attrs import define

@define
class User:
    id: int
    email: str

u1 = User(1, "a@example.com")
payload = attrs.asdict(u1)
u2 = attrs.evolve(u1, email="b@example.com")
```

Use `attrs.filters.include(...)` or `attrs.filters.exclude(...)` with `attrs.asdict()` when you need to omit fields such as passwords or internal IDs.

### Post-Initialization Hooks

If you need derived values after field assignment, use `__attrs_post_init__`.

```python
from attrs import define, field

@define
class Rectangle:
    width: int
    height: int
    area: int = field(init=False)

    def __attrs_post_init__(self) -> None:
        self.area = self.width * self.height
```

Use `__attrs_pre_init__` only when you specifically need to call `super().__init__()` or integrate with subclassing-based APIs.

## Type Checking

- `attrs` works well with explicit type annotations.
- The upstream docs call out dedicated `mypy` support for attrs classes.
- Pyright support is based on `dataclass_transform` / PEP 681 and covers a subset closest to standard-library dataclasses.
- If you use forward references and need runtime-resolved field types, call `attrs.resolve_types(...)`.

## Config And Auth

`attrs` has no service authentication, API keys, or network configuration.

Typical project-level configuration is limited to:

- choosing mutable vs frozen classes
- deciding whether fields are positional or keyword-only
- deciding whether validators/converters should also run on attribute assignment
- enabling your type checker configuration if the project relies on static typing

## Common Pitfalls

- Install name vs imports: install with `pip install attrs`, prefer importing from `attrs`, and expect older code to import from `attr`.
- Mixed typed and untyped field declarations: if you define one field with `field()` but omit its annotation, attrs switches into no-typing mode and will ignore merely annotated attributes that are not also defined with `field()`.
- Mutable defaults: do not write `tags: list[str] = []`; use `Factory(list)` or `field(factory=list)` instead.
- Private attribute aliases: a field like `_token: str` is exposed as `token=` in the generated `__init__` unless you override it with `field(alias="_token")`.
- Attribute ordering: required fields cannot follow fields with defaults unless the later fields are keyword-only.
- Assignment behavior: with the modern APIs, assignment-time conversion and validation are part of the default `on_setattr` behavior. If your code mutates fields after construction, expect converters and validators to run again.
- `attrs.evolve()` uses `__init__`: pass private attributes without the leading underscore, matching the generated initializer.
- Hashing options: prefer `frozen=True` or `unsafe_hash=`. The `hash=` argument is the deprecated alias.

## Version-Sensitive Notes

- `25.4.0` is the current stable release documented at `www.attrs.org` and released on `2025-10-06`.
- `25.4.0` changed class-level `kw_only=True` behavior to match `dataclasses`: it now applies only to attributes defined on that class unless a field explicitly opts out.
- `25.4.0` added support for Python `3.14`.
- `24.3.0` dropped Python `3.7`; for `25.4.0`, PyPI requires Python `>=3.9`.
- `24.1.0` changed `attrs.evolve()` so the instance must be passed positionally, not as `inst=...`.
- Since `21.3.0`, the `attrs` import namespace is available directly; older examples may still show `import attr`.

## Practical Guidance For Agents

1. Start with `@define` plus type annotations unless the codebase already uses legacy `attr.s`.
2. Use `Factory(...)` or `field(factory=...)` for mutable values.
3. Put normalization in `converter=` and invariant checks in `validator=`.
4. Reach for `@frozen` when the class should be hashable and immutable in application code.
5. Use `attrs.asdict()` for simple serialization, but prefer `cattrs` or another dedicated serialization layer when the project needs schema-driven loading/dumping.

## Official Sources

- Documentation root: `https://www.attrs.org/en/stable/`
- Examples: `https://www.attrs.org/en/stable/examples.html`
- Initialization guide: `https://www.attrs.org/en/stable/init.html`
- API reference: `https://www.attrs.org/en/stable/api.html`
- Type annotations guide: `https://www.attrs.org/en/stable/types.html`
- Core API naming guide: `https://www.attrs.org/en/stable/names.html`
- Changelog: `https://www.attrs.org/en/stable/changelog.html`
- PyPI package page: `https://pypi.org/project/attrs/`

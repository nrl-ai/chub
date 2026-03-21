---
name: package
description: "typing-extensions package guide for Python backported and experimental typing features"
metadata:
  languages: "python"
  versions: "4.15.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "typing,type-hints,backports,static-analysis,python"
---

# typing-extensions Python Package Guide

## Golden Rule

Use the PyPI package `typing-extensions`, but import it as `typing_extensions`.

Reach for this package when you need typing features that are:

- not available in your minimum supported Python version
- newer than the stdlib `typing` module on some interpreters
- still incubating outside the standard library

For cross-version library code, prefer importing the needed symbol from `typing_extensions` instead of writing Python-version branches around `typing`.

## Installation

Install the exact version you want in an application or lockfile:

```bash
pip install "typing-extensions==4.15.0"
```

Common dependency managers:

```bash
uv add "typing-extensions==4.15.0"
poetry add "typing-extensions==4.15.0"
```

For a reusable library, do not depend on patch-level compatibility. The maintainers document SemVer expectations and explicitly recommend a range based on the first feature you need:

```toml
[project]
dependencies = [
  "typing-extensions>=4.15,<5",
]
requires-python = ">=3.9"
```

If you only need an older feature, set the lower bound to that first required version instead of forcing `4.15.0`.

## Setup

`typing_extensions` has no runtime initialization, auth, service clients, or environment variables.

Typical setup is only:

1. add it to project dependencies
2. import the needed symbols from `typing_extensions`
3. run a type checker such as `mypy`, `pyright`, or `pyre`

If your codebase targets only newer Python versions and every needed symbol already exists in `typing`, you may not need this package at runtime. Many libraries still keep it for uniform imports across supported versions.

## Core Usage

### Backport newer typing features

This is the main use case: write one import path that works across supported Python versions.

```python
from typing_extensions import NotRequired, ReadOnly, Self, TypedDict, override

class UserPatch(TypedDict, total=False):
    display_name: str
    email: NotRequired[str]
    user_id: ReadOnly[int]

class Builder:
    def set_name(self, name: str) -> Self:
        self.name = name
        return self

class BaseHandler:
    def handle(self, payload: UserPatch) -> None:
        raise NotImplementedError

class Handler(BaseHandler):
    @override
    def handle(self, payload: UserPatch) -> None:
        print(payload)
```

Useful symbols commonly imported from `typing_extensions` include `TypedDict`, `Required`, `NotRequired`, `ReadOnly`, `Literal`, `Final`, `TypeAliasType`, `Self`, `override`, `Protocol`, and `runtime_checkable`.

### Runtime protocols

`Protocol` and `runtime_checkable` are useful when code needs structural typing at runtime as well as in static analysis.

```python
from typing_extensions import Protocol, runtime_checkable

@runtime_checkable
class SupportsClose(Protocol):
    def close(self) -> None: ...

def shutdown(resource: SupportsClose) -> None:
    if not isinstance(resource, SupportsClose):
        raise TypeError("resource must define close()")
    resource.close()
```

### Annotation introspection

Recent versions add helpers for inspecting annotations in a version-tolerant way.

```python
from typing_extensions import Format, get_annotations

class Node:
    next: "Node | None"

annotations = get_annotations(Node, format=Format.FORWARDREF)
print(annotations["next"])
```

Use this when you need consistent behavior across Python versions for forward references and annotation formats.

## Config And Auth

There is no config file, auth flow, or network setup.

The only configuration that usually matters is package metadata in `pyproject.toml`:

- choose a lower bound that matches the first feature you use
- keep `requires-python` aligned with the package's supported versions
- lock exact versions in applications, but use feature-based version ranges in reusable libraries

## Common Pitfalls

### Package name vs import name

Install `typing-extensions`, import `typing_extensions`.

### Mixing `typing` and `typing_extensions` carelessly

If a symbol exists in both modules, choose one import strategy per feature in shared code. This avoids brittle runtime checks and inconsistent annotation objects across Python versions.

For runtime introspection code, the upstream docs recommend checking against both stdlib and `typing_extensions` variants when necessary.

### Evaluating annotations can run code

The docs warn that `typing_extensions.get_annotations()`, `typing_extensions.evaluate_forward_ref()`, and stdlib helpers such as `typing.get_type_hints()` may execute code contained in annotations. Do not run them on untrusted code or untrusted annotation strings.

### Old `TypedDict` functional syntax

The keyword-argument form is obsolete:

```python
# Avoid
OldStyle = TypedDict("OldStyle", name=str)
```

Prefer the dictionary form:

```python
from typing_extensions import TypedDict

NewStyle = TypedDict("NewStyle", {"name": str})
```

This matters more on Python 3.13+, where the old form is no longer supported by `typing_extensions`.

### Pre-releases are not compatibility promises

The maintainers do not guarantee backward compatibility for pre-releases. Use stable releases for production code and copy examples from stable docs or tagged releases, not release candidates.

## Version-Sensitive Notes

- `4.15.0` adds `@typing_extensions.disjoint_base`, adds `typing_extensions.type_repr()`, and fixes `evaluate_forward_ref()` handling for type parameters after PEP 695.
- `4.14.0` ended Python 3.8 support. For `4.15.0`, assume Python `>=3.9`.
- `4.13.0` added `get_annotations()`, `evaluate_forward_ref()`, and the `Format` enum; use these instead of ad hoc forward-ref parsing when you need runtime annotation inspection.
- The docs root at `latest/` is the canonical starting point, but its page title may lag behind the newest release. Use the GitHub releases page for exact per-version change notes.

## Official Sources

- Documentation: `https://typing-extensions.readthedocs.io/en/latest/`
- PyPI: `https://pypi.org/project/typing-extensions/`
- Repository: `https://github.com/python/typing_extensions`
- Releases: `https://github.com/python/typing_extensions/releases`

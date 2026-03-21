---
name: package
description: "wrapt package guide for Python decorators, wrappers, monkey patching, and object proxies"
metadata:
  languages: "python"
  versions: "2.1.2"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "wrapt,python,decorators,wrappers,proxies,monkey-patching"
---

# wrapt Python Package Guide

## What It Is

`wrapt` is a utility library for writing decorators, monkey patches, and object proxies without breaking descriptor binding, signatures, or introspection.

Use it when plain Python decorators are too fragile for methods, class methods, instance-aware wrappers, or transparent proxy objects.

## Version Note

- Package version covered here: `2.1.2`
- PyPI published `2.1.2` on 2026-03-06.
- The docs URL points at `https://wrapt.readthedocs.io/en/latest/`, but the `latest` alias is currently stale. For current release notes and API details, rely on the official changelog and docs pages under the upstream docs site plus PyPI metadata.

## Install

```bash
pip install wrapt==2.1.2
```

```bash
uv add wrapt==2.1.2
```

```bash
poetry add wrapt==2.1.2
```

## Import Surface

Common imports:

```python
import wrapt
from wrapt import ObjectProxy, BaseObjectProxy, LazyObjectProxy
from wrapt import wrap_function_wrapper, patch_function_wrapper
```

Key APIs:

- `@wrapt.decorator`: safest starting point for decorators that must work on functions, instance methods, class methods, and classes.
- `wrap_function_wrapper()` / `patch_function_wrapper()`: patch an existing import target without replacing it by hand.
- `ObjectProxy`, `BaseObjectProxy`, `LazyObjectProxy`: transparent wrappers around another object.

## Core Decorator Pattern

Use `@wrapt.decorator` and keep the wrapper signature exact:

```python
import wrapt

@wrapt.decorator
def log_calls(wrapped, instance, args, kwargs):
    print(f"calling {wrapped.__name__} with {args=} {kwargs=}")
    return wrapped(*args, **kwargs)

@log_calls
def add(a: int, b: int) -> int:
    return a + b
```

Behavior of wrapper arguments:

- `wrapped`: the original callable or class being wrapped
- `instance`: bound instance for instance methods, class for class methods, otherwise `None`
- `args` / `kwargs`: arguments passed by the caller

Do not pass `instance` back into `wrapped()`. Call `wrapped(*args, **kwargs)`.

## Decorators With Arguments

Use a closure around the wrapt decorator:

```python
import time
import wrapt

def timed(label: str):
    @wrapt.decorator
    def wrapper(wrapped, instance, args, kwargs):
        start = time.perf_counter()
        try:
            return wrapped(*args, **kwargs)
        finally:
            elapsed = time.perf_counter() - start
            print(f"{label}: {elapsed:.4f}s")

    return wrapper

@timed("query")
def run_query(sql: str) -> str:
    return sql
```

## Wrapping Existing Functions

For monkey patching imported code, prefer the helper APIs instead of assigning raw wrapper functions yourself.

```python
import wrapt
import requests.sessions

def record_request(wrapped, instance, args, kwargs):
    response = wrapped(*args, **kwargs)
    print(response.status_code)
    return response

wrapt.wrap_function_wrapper(
    requests.sessions.Session,
    "request",
    record_request,
)
```

String-based patching is useful when you need to name the import target dynamically:

```python
import wrapt

def record_open(wrapped, instance, args, kwargs):
    print("opening", args[0])
    return wrapped(*args, **kwargs)

wrapt.patch_function_wrapper("pathlib", "Path.open", record_open)
```

Use these helpers after the target module is imported. If you need lazy import-time patching, `wrapt` also provides post-import hook support.

## Post-Import Hooks

Use post-import hooks when you must patch a dependency only after it is imported somewhere else:

```python
from wrapt import register_post_import_hook, wrap_function_wrapper

def apply_patch(module):
    wrap_function_wrapper(module, "dangerous_call", my_wrapper)

register_post_import_hook(apply_patch, "some_dependency")
```

This avoids importing and patching modules too early during interpreter startup.

## Object Proxies

Use proxies when you need to wrap an object while preserving most normal behavior:

```python
from wrapt import BaseObjectProxy

class TracedList(BaseObjectProxy):
    def append(self, value):
        print("append", value)
        return self.__wrapped__.append(value)

items = TracedList([])
items.append(1)
```

Use `LazyObjectProxy` when constructing the wrapped object is expensive and can be deferred:

```python
from wrapt import LazyObjectProxy

settings = LazyObjectProxy(lambda: load_settings())
```

## Config And Environment

`wrapt` does not have service auth, API keys, or network configuration.

Relevant runtime/build configuration:

- `WRAPT_DISABLE_EXTENSIONS=1`: disable use of the C extension at runtime for debugging or compatibility checks.
- `WRAPT_INSTALL_EXTENSIONS=false`: skip building C extensions at install time.

Example:

```bash
WRAPT_DISABLE_EXTENSIONS=1 pytest
```

Most projects should use the default compiled extension when wheels are available.

## Typing

`wrapt` 2.x added improved type annotations and `ParamSpec`-based decorator typing. Keep decorator factories typed explicitly when you want good inference:

```python
from collections.abc import Callable
from typing import ParamSpec, TypeVar
import wrapt

P = ParamSpec("P")
R = TypeVar("R")

def traced() -> Callable[[Callable[P, R]], Callable[P, R]]:
    @wrapt.decorator
    def wrapper(wrapped, instance, args: P.args, kwargs: P.kwargs) -> R:
        return wrapped(*args, **kwargs)

    return wrapper
```

If a type checker still loses the original callable signature through a complex decorator stack, simplify the factory return type or add a targeted cast at the outer boundary.

## Common Pitfalls

- Use `@wrapt.decorator`, not a plain nested decorator, when binding semantics matter.
- Keep the wrapper signature exactly `(wrapped, instance, args, kwargs)`.
- Call `wrapped(*args, **kwargs)`, not `wrapped(instance, *args, **kwargs)`.
- For new proxy types, prefer `BaseObjectProxy` over `ObjectProxy`. `ObjectProxy` includes `__iter__()` unconditionally for backward compatibility, which can make non-iterable proxies look iterable.
- `AutoObjectProxy` creates a dedicated derived type per instance; use it only when you need automatic special-method coverage and accept the extra memory cost.
- `LazyObjectProxy` still executes the factory once on first access; keep the factory side effects predictable.
- When combining with `@classmethod`, put the `wrapt` decorator outside `@classmethod`. The known ordering issue remains relevant because the attempted Python-level fix was later reverted.

## Version-Sensitive Notes For 2.x

- `2.0.0` changed typing support and added `BaseObjectProxy` so new proxy code has a cleaner base class.
- `2.0.0` also removed the runtime dependency on `setuptools`.
- `2.1.0` added `py.typed`, widened Python support through 3.13, and improved decorator typing for newer type checkers.
- `2.1.2` is a packaging/build maintenance release; if examples online still talk about `1.x`, verify proxy-base-class and typing guidance before copying them.

## Practical Rules

1. Start with `@wrapt.decorator` unless you specifically need a proxy class.
2. Use `wrap_function_wrapper()` or `patch_function_wrapper()` for monkey patching instead of manual reassignment.
3. Use `BaseObjectProxy` for new custom proxies.
4. Treat the `en/latest/` docs alias as potentially stale and cross-check with PyPI and the changelog when version details matter.

## Official Sources

- Docs root: https://wrapt.readthedocs.io/en/latest/
- Current official docs pages: https://wrapt.readthedocs.io/en/master/
- Changelog: https://wrapt.readthedocs.io/en/master/changes.html
- PyPI: https://pypi.org/project/wrapt/
- Repository: https://github.com/GrahamDumpleton/wrapt

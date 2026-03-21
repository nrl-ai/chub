---
name: package
description: "decorator package guide for python - signature-preserving decorators, decorator factories, context managers, and multiple dispatch"
metadata:
  languages: "python"
  versions: "5.2.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "decorator,python,decorators,dispatch,contextmanager,metaprogramming"
---

# decorator Python Package Guide

## Golden Rule

Use `decorator` when you need a custom decorator that preserves the wrapped function's signature and introspection metadata.

For simple wrappers where exact signature preservation does not matter, `functools.wraps` is usually enough. Reach for `decorator` when you need correct `inspect.signature(...)`, `__wrapped__`, coroutine detection, or decorator factories that should keep the original call shape instead of collapsing into `*args, **kwargs`.

## Installation

```bash
pip install decorator==5.2.1
```

Common alternatives:

```bash
poetry add decorator==5.2.1
uv add decorator==5.2.1
```

Official Python requirement for `5.2.1` is `>=3.8` according to PyPI.

## Initialization And Setup

There is no client object, global configuration, environment variable, or authentication step.

Setup is just import-time:

```python
from decorator import decorator, decorate, contextmanager, dispatch_on
```

If you want to avoid the `decorator` module name colliding with the `decorator(...)` helper, alias the module:

```python
import decorator as deco
```

## Core Usage

### 1. Build a signature-preserving decorator with `@decorator`

Your caller function should accept the wrapped function first, then any decorator-factory parameters, then `*args, **kwargs` for the wrapped call.

```python
from decorator import decorator
import time

@decorator
def warn_slow(func, timelimit=0.25, *args, **kwargs):
    started = time.perf_counter()
    result = func(*args, **kwargs)
    elapsed = time.perf_counter() - started
    if elapsed > timelimit:
        print(f"{func.__name__} took {elapsed:.3f}s")
    return result

@warn_slow
def add(x: int, y: int = 1) -> int:
    return x + y

assert add(2, y=3) == 5
```

What this gives you:

- The decorated function keeps the original call signature.
- `inspect.signature(add)` still reflects `add(x: int, y: int = 1)`.
- The wrapped function is available as `add.__wrapped__`.

### 2. Use `decorate(...)` for programmatic wrapping

Use `decorate(func, caller)` when you want to wrap a function programmatically instead of using `@` syntax.

```python
from decorator import decorate

def traced_call(func, *args, **kwargs):
    print("calling", func.__name__, args, kwargs)
    return func(*args, **kwargs)

def multiply(x, y=2):
    return x * y

multiply = decorate(multiply, traced_call)
```

Prefer this over older `decorator(caller, func)` examples from blog posts or older code snippets.

### 3. Build decorator factories directly

Extra parameters before `*args, **kwargs` become decorator-factory arguments.

```python
from decorator import decorator

@decorator
def retry(func, attempts=3, *args, **kwargs):
    last_error = None
    for _ in range(attempts):
        try:
            return func(*args, **kwargs)
        except Exception as exc:  # narrow this in real code
            last_error = exc
    raise last_error

@retry(attempts=5)
def flaky_operation():
    ...
```

If every extra parameter has a default, the same decorator can also be used without parentheses:

```python
@retry
def usually_ok():
    return "ok"
```

### 4. Wrap `async def` functions

`decorator` supports coroutine functions and preserves `inspect.iscoroutinefunction(...)`.

```python
from decorator import decorator
import logging
import time

@decorator
async def log_runtime(func, *args, **kwargs):
    started = time.perf_counter()
    try:
        return await func(*args, **kwargs)
    finally:
        elapsed = time.perf_counter() - started
        logging.info("%s took %.3fs", func.__name__, elapsed)

@log_runtime
async def fetch_user(user_id: str) -> dict:
    ...
```

### 5. Use `decorator.contextmanager` when the same object should work in `with` blocks and as a decorator

The stdlib `contextlib.contextmanager` can produce callable context-manager objects, but those wrappers do not preserve the wrapped function signature. `decorator.contextmanager` returns a `ContextManager` subclass whose `__call__` is signature-preserving.

```python
from decorator import contextmanager

@contextmanager
def logged(label):
    print("enter", label)
    try:
        yield
    finally:
        print("exit", label)

@logged("db")
def run_query(sql: str, limit: int = 100) -> str:
    return f"running {sql} with limit={limit}"
```

You can also use the same object in a `with` block:

```python
with logged("db"):
    ...
```

### 6. Use `dispatch_on(...)` for lightweight multiple dispatch

`dispatch_on` creates generic functions that dispatch on one or more named arguments.

```python
from decorator import dispatch_on

@dispatch_on("value")
def render(value):
    raise NotImplementedError(type(value))

@render.register(int)
def _(value):
    return f"int:{value}"

@render.register(str)
def _(value):
    return f"str:{value}"
```

Useful helpers:

- `render.register(Type)` registers an implementation.
- `render.dispatch_info(SomeType)` shows the resolution order.
- Ambiguous virtual-ancestor matches raise `RuntimeError` instead of guessing.

### 7. Set `kwsyntax=True` only when your wrapper needs original keyword spelling

By default, `decorator` normalizes arguments against the wrapped function signature before your caller sees them. That means positional-or-keyword parameters usually arrive in `args`, even if the caller used keyword syntax.

```python
from decorator import decorator

def debug_call(func, *args, **kwargs):
    print(args, kwargs)
    return func(*args, **kwargs)

debug = decorator(debug_call, kwsyntax=True)

@debug
def printsum(x=1, y=2):
    return x + y

printsum(y=2, x=1)
```

Leave `kwsyntax=False` unless your wrapper logic depends on how the caller spelled keyword arguments.

### 8. `decoratorx(...)` is niche

`decoratorx` exists for cases where you need wrapper generation via `FunctionMaker` and want to preserve more `__code__`-level details. In normal application code, prefer `@decorator` or `decorate(...)`.

## Config And Environment

- No auth.
- No environment variables.
- No global init sequence.
- No runtime config file.

The main setup choice is which helper to use:

- `@decorator` for most new decorators and decorator families
- `decorate(func, caller)` for programmatic wrapping
- `contextmanager` when the wrapper should also be usable in `with`
- `dispatch_on` for generic-function style dispatch

## Common Pitfalls

### Import-name collisions

This is easy to misread:

```python
from decorator import decorator
```

If the code also needs the module namespace, alias one of them:

```python
import decorator as deco
from decorator import decorator
```

### Caller signatures are strict

Your caller function should be shaped like one of these:

```python
def caller(func, *args, **kwargs): ...
def caller(func, option=default, *args, **kwargs): ...
```

Put decorator-factory parameters before `*args, **kwargs`, not after them.

### `kwsyntax=False` surprises people

This package is intentionally not a thin clone of `functools.wraps`.

If your wrapper prints or inspects arguments, a call like `f(x=1, y=2)` may still arrive as positional arguments after signature normalization. Set `kwsyntax=True` only if you need wraps-like behavior.

### `contextlib.contextmanager` and `decorator.contextmanager` are not interchangeable

Use `decorator.contextmanager` when you need a context manager that is also a signature-preserving decorator.

If you only need `with ...:` behavior and never decorate functions with it, the stdlib `contextlib.contextmanager` is simpler.

### `dispatch_on` is not full multimethod infrastructure

It is a compact generic-function mechanism, not a full plugin registry or validation system.

Watch for:

- ambiguous virtual-ancestor relationships, which raise `RuntimeError`
- dispatch registration based on concrete argument names
- default implementation fallback when no registered specialization matches

### Some wrapped functions are generated objects

If `inspect.getsource(wrapped_fn)` fails, inspect `wrapped_fn.__wrapped__` instead.

```python
import inspect

inspect.getsource(my_function.__wrapped__)
```

### The maintainer docs page is not version-pinned

The repository documentation linked from PyPI currently points at the `master` branch docs page, and that page is labeled `5.2.0` rather than `5.2.1`.

For version-sensitive work, check both:

- the exact PyPI release page for `5.2.1`
- the maintainer changelog for any `5.2.x` behavior changes

## Version-Sensitive Notes

- `5.2.1` is the current PyPI release listed on the official project page, published on February 24, 2025.
- The maintainer changelog currently has a `5.2.0` section but no separate `5.2.1` section. Inference: treat `5.2.1` as the current install target, but use `5.2.0` maintainer docs/changelog to understand the public API until a dedicated `5.2.1` note appears upstream.
- `5.2.0` added official support for Python `3.11`, `3.12`, and `3.13`, dropped official support claims for Python `<3.8`, and added support for decorating `functools.partial` objects.
- `5.1.1` fixed decoration of cythonized functions and repaired `decorator.contextmanager` regressions from `5.1.0`.
- `5.1.0` added `decoratorx(...)` and fixed `kwsyntax` forwarding through `decorator.decorator(...)`.

## Official Sources

- Repository: `https://github.com/micheles/decorator`
- Maintainer documentation: `https://github.com/micheles/decorator/blob/master/docs/documentation.md`
- Changelog: `https://github.com/micheles/decorator/blob/master/CHANGES.md`
- Exact PyPI release page: `https://pypi.org/project/decorator/5.2.1/`

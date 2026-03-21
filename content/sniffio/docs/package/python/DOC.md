---
name: package
description: "sniffio for Python - detect the active async library in shared async code"
metadata:
  languages: "python"
  versions: "1.3.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "sniffio,python,asyncio,trio,curio,async,contextvars"
---

# sniffio Python Package Guide

## What It Does

`sniffio` is a tiny compatibility helper for async Python libraries. Its main API, `current_async_library()`, tells you which async library is currently running so shared code can branch correctly.

Use it when you are writing reusable async helpers or adapters that need to work under more than one runtime. It does not create an event loop, run tasks, or replace `asyncio`, Trio, or Curio.

## Install

Install the package version your project expects:

```bash
python -m pip install "sniffio==1.3.1"
```

Common alternatives:

```bash
uv add "sniffio==1.3.1"
poetry add "sniffio==1.3.1"
```

PyPI metadata for `1.3.1` declares `Requires-Python >=3.7`.

## Imports

Typical application or library code only needs:

```python
from sniffio import current_async_library, AsyncLibraryNotFoundError
```

If you are writing framework integration code, `sniffio` also exports these advanced hooks:

```python
from sniffio import current_async_library_cvar, thread_local
```

## Environment And Initialization

- No environment variables are required.
- No auth, credentials, or service endpoints are involved.
- There is no client object and no startup initialization step.
- The important setup rule is simple: call `current_async_library()` from running async code, or set one of the exported override hooks in integration code.

## Basic Detection

### `asyncio`

```python
import asyncio

from sniffio import current_async_library


async def main() -> None:
    library = current_async_library()
    print(library)  # asyncio


asyncio.run(main())
```

### Trio

```python
import trio

from sniffio import current_async_library


async def main() -> None:
    library = current_async_library()
    print(library)  # trio


trio.run(main)
```

## Branch On The Current Runtime

This is the core pattern when you maintain one async API that has runtime-specific implementations underneath:

```python
from sniffio import current_async_library


async def sleep_for(seconds: float) -> None:
    library = current_async_library()

    if library == "trio":
        import trio

        await trio.sleep(seconds)
    elif library == "asyncio":
        import asyncio

        await asyncio.sleep(seconds)
    elif library == "curio":
        import curio

        await curio.sleep(seconds)
    else:
        raise RuntimeError(f"Unsupported async library: {library!r}")
```

In `1.3.1`, the documented built-in magic strings are `"trio"`, `"curio"`, and `"asyncio"`.

## Handle Synchronous Call Sites

`current_async_library()` raises `AsyncLibraryNotFoundError` when it is called outside a recognized async context.

Catch that exception if the same code path might be used from synchronous startup, tests, or CLI utilities:

```python
from sniffio import AsyncLibraryNotFoundError, current_async_library


def get_current_library() -> str | None:
    try:
        return current_async_library()
    except AsyncLibraryNotFoundError:
        return None
```

## Integration Hooks For Framework Authors

If you are adapting a custom async runner or bridging another framework into `sniffio`, the public `ContextVar` hook is the cleanest way to declare the active runtime for the current context:

```python
from sniffio import current_async_library, current_async_library_cvar


def mark_runtime(name: str) -> None:
    token = current_async_library_cvar.set(name)
    try:
        print(current_async_library())
    finally:
        current_async_library_cvar.reset(token)


mark_runtime("my-runtime")
```

`thread_local` is also a public fallback hook:

```python
from sniffio import current_async_library, thread_local


old_name, thread_local.name = thread_local.name, "my-runtime"
try:
    print(current_async_library())
finally:
    thread_local.name = old_name
```

Prefer `current_async_library_cvar` for async integrations. It follows normal `contextvars` scoping and reset semantics.

## Common Pitfalls

- `current_async_library()` returns a plain string, not an enum or runtime object.
- `sniffio` only detects the active library; it does not provide sleep, task, or event-loop APIs.
- If the runtime is unknown and you have not set `current_async_library_cvar` or `thread_local.name`, `current_async_library()` raises `AsyncLibraryNotFoundError`.
- `thread_local.name` is checked before `current_async_library_cvar`, so a thread-local override wins if both are set.
- For `trio-asyncio`, the documented return value is still `"trio"` or `"asyncio"` depending on the current mode, not a separate combined string.

## Version-Sensitive Notes

- `1.3.1` publishes `Requires-Python >=3.7` in package metadata.
- The `1.3.1` source docstring documents support for Trio `0.6+` and `trio-asyncio` `0.8.2+`.
- The bundled upstream tests in `1.3.1` skip Curio on Python `3.12+` because Curio is noted there as broken on Python `3.12`.

## Official Links

- Documentation: `https://sniffio.readthedocs.io/en/latest/`
- PyPI: `https://pypi.org/project/sniffio/`
- Repository: `https://github.com/python-trio/sniffio`
- Changelog: `https://sniffio.readthedocs.io/en/latest/history.html`

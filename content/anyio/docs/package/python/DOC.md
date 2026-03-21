---
name: package
description: "AnyIO package guide for Python structured concurrency across asyncio and Trio"
metadata:
  languages: "python"
  versions: "4.12.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "anyio,python,asyncio,trio,async,concurrency,structured-concurrency,pytest,threads"
---

# anyio Python Package Guide

## What It Is

`anyio` is a backend-neutral async concurrency library for Python. It exposes Trio-style structured concurrency primitives that run on top of either `asyncio` or Trio, plus shared APIs for cancellation, streams, synchronization, subprocesses, files, worker threads, and pytest integration.

Use it when you want code that:

- targets both `asyncio` and Trio
- prefers structured concurrency over detached background tasks
- needs a consistent API for timeouts, cancellation, streams, and blocking offload

## Installation

Install the exact version covered here:

```bash
pip install anyio==4.12.1
```

If you want Trio available as a runtime backend, install the Trio extra:

```bash
pip install "anyio[trio]==4.12.1"
```

With common Python package managers:

```bash
uv add anyio==4.12.1
poetry add anyio==4.12.1
```

PyPI lists these extras for `4.12.1`: `trio`, `test`, and `doc`.

## Setup And Backend Selection

Use `anyio.run()` from a synchronous entrypoint. The default backend is `asyncio`.

```python
import anyio

async def main() -> None:
    await anyio.sleep(0.1)
    print("hello from anyio")

anyio.run(main)
```

To choose a backend explicitly:

```python
anyio.run(main, backend="asyncio")
anyio.run(main, backend="trio")
```

Backend-specific options go through `backend_options`:

```python
anyio.run(main, backend="asyncio", backend_options={"debug": True})
```

For uvloop with AnyIO 4, prefer the documented `backend_options` path:

```python
anyio.run(main, backend="asyncio", backend_options={"use_uvloop": True})
```

If you need custom loop construction, AnyIO 4 migration guidance says to use `loop_factory`, not `event_loop_policy`:

```python
import anyio
import uvloop

anyio.run(
    main,
    backend="asyncio",
    backend_options={"loop_factory": uvloop.new_event_loop},
)
```

If you are already inside async code, do not nest `anyio.run()`. Use AnyIO primitives directly within the existing task instead.

## Core Usage

### Task Groups

Use `create_task_group()` for structured concurrency. Child tasks are scoped to the context manager, and cancellation/cleanup is coordinated automatically.

```python
import anyio

async def worker(name: str, delay: float) -> None:
    await anyio.sleep(delay)
    print(f"{name} done")

async def main() -> None:
    async with anyio.create_task_group() as tg:
        tg.start_soon(worker, "a", 0.2)
        tg.start_soon(worker, "b", 0.1)

anyio.run(main)
```

If the parent must wait until the child has started successfully, use `TaskGroup.start()` and `task_status.started()`:

```python
import anyio
from anyio.abc import TaskStatus

async def start_service(*, task_status: TaskStatus[None]) -> None:
    task_status.started()
    await anyio.sleep_forever()

async def main() -> None:
    async with anyio.create_task_group() as tg:
        await tg.start(start_service)
        print("service is ready")

anyio.run(main)
```

### Timeouts And Cancellation

AnyIO timeout helpers are built on cancel scopes:

- `move_on_after(seconds)` cancels the block and continues
- `fail_after(seconds)` cancels the block and raises `TimeoutError`

```python
import anyio

async def main() -> None:
    with anyio.move_on_after(1) as scope:
        await anyio.sleep(2)

    if scope.cancelled_caught:
        print("timed out")

    with anyio.fail_after(1):
        await anyio.sleep(0.5)

anyio.run(main)
```

When cleanup must keep running during cancellation, shield the cleanup block:

```python
import anyio

async def close_resource(resource) -> None:
    try:
        await resource.do_work()
    except anyio.get_cancelled_exc_class():
        with anyio.move_on_after(10, shield=True):
            await resource.aclose()
        raise
```

### Memory Object Streams

For in-process producer/consumer pipelines, use memory object streams:

```python
import anyio
from anyio.abc import ObjectReceiveStream, ObjectSendStream

async def producer(send: ObjectSendStream[int]) -> None:
    async with send:
        for item in range(3):
            await send.send(item)

async def consumer(receive: ObjectReceiveStream[int]) -> None:
    async with receive:
        async for item in receive:
            print(item)

async def main() -> None:
    send, receive = anyio.create_memory_object_stream[int](10)
    async with anyio.create_task_group() as tg:
        tg.start_soon(producer, send)
        tg.start_soon(consumer, receive)

anyio.run(main)
```

In AnyIO 4, the typed form is `create_memory_object_stream[T](buffer_size)`. Older examples that pass the type as a second runtime argument are pre-4.x.

### Run Blocking Code In Worker Threads

Use `to_thread.run_sync()` for blocking I/O or sync library calls that should not block the event loop:

```python
import time
import anyio

def blocking_call() -> str:
    time.sleep(1)
    return "done"

async def main() -> None:
    result = await anyio.to_thread.run_sync(blocking_call)
    print(result)

anyio.run(main)
```

The docs say the default worker thread limiter has `40` total tokens. Adjust it if your app intentionally offloads more sync work:

```python
import anyio

async def main() -> None:
    anyio.to_thread.current_default_thread_limiter().total_tokens = 60
```

## Testing

AnyIO ships with a pytest plugin.

Mark async tests explicitly:

```python
import anyio
import pytest

@pytest.mark.anyio
async def test_example() -> None:
    await anyio.sleep(0)
```

Or enable automatic handling in `pyproject.toml`:

```toml
[tool.pytest.ini_options]
anyio_mode = "auto"
```

If `pytest-asyncio` is installed too, avoid putting both plugins in conflicting auto modes. Pick one framework's auto-discovery behavior and keep the other explicit.

## Config And Environment

`anyio` does not have package-level authentication or service credentials. The main runtime configuration points are:

- backend choice: `asyncio` vs `trio`
- `backend_options` for loop/debug behavior
- worker thread limiter sizing for `to_thread.run_sync()`
- pytest plugin mode for test suites

For library code, keep backend-specific calls at the application boundary. If a function uses raw `asyncio` APIs internally, it is no longer backend-neutral and may fail under Trio.

## Common Pitfalls

- Use PyPI, not the moving `stable` docs title, to confirm the package version and Python floor for `4.12.1`.
- Do not use `asyncio.create_task()` or other raw backend APIs inside backend-neutral AnyIO library code unless you intentionally want to lock the code to that backend.
- In AnyIO 4, task group failures propagate as `ExceptionGroup`. On Python 3.11+, use `except*`. On Python 3.9-3.10, use the `exceptiongroup` backport if you need compatible exception-group handling.
- Old AnyIO examples may show `create_memory_object_stream(size, type)`; for AnyIO 4, use `create_memory_object_stream[T](size)`.
- Old asyncio-backend setup may use `event_loop_policy`; migration guidance says to use `loop_factory` instead.
- `to_thread.run_sync()` is for blocking synchronous work. It prevents event-loop stalls, but it is still thread-based and not a substitute for deliberate process-level parallelism.

## Version-Sensitive Notes For 4.12.x

- `4.12.0` added `anyio.functools`, `get_available_backends()`, and expanded `use_uvloop=True` support on Windows when `winloop` is installed.
- `4.12.1` changed the `NoEventLoopError` exception name from the internal `NoCurrentAsyncBackend`. If you were catching the old internal name, update that code.
- The migration guide remains relevant for AnyIO 4 generally: exception groups replaced custom task-group exceptions, memory object stream typing changed, and event loop policy configuration moved to `loop_factory`.

## Official Sources

- Docs root: https://anyio.readthedocs.io/en/stable/
- Basics: https://anyio.readthedocs.io/en/stable/basics.html
- Creating and managing tasks: https://anyio.readthedocs.io/en/stable/tasks.html
- Cancellation and timeouts: https://anyio.readthedocs.io/en/stable/cancellation.html
- Working with threads: https://anyio.readthedocs.io/en/stable/threads.html
- Testing: https://anyio.readthedocs.io/en/stable/testing.html
- Migrating from AnyIO 3 to AnyIO 4: https://anyio.readthedocs.io/en/stable/migration.html
- Version history: https://anyio.readthedocs.io/en/stable/versionhistory.html
- PyPI package page: https://pypi.org/project/anyio/
- PyPI version page: https://pypi.org/project/anyio/4.12.1/

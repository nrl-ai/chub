---
name: package
description: "Trio structured concurrency runtime for Python async applications"
metadata:
  languages: "python"
  versions: "0.33.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "trio,async,structured-concurrency,concurrency,testing"
---

# Trio Python Package Guide

## Golden Rule

Use Trio as a structured-concurrency runtime, not as a drop-in `asyncio` clone.

- Start programs with `trio.run(...)`.
- Group sibling tasks inside `async with trio.open_nursery() as nursery:`.
- Use Trio deadlines and cancellation scopes (`fail_after`, `move_on_after`, `CancelScope`) instead of ad hoc sleep-and-poll logic.
- Offload blocking sync work with `trio.to_thread.run_sync(...)`.

This entry targets Trio `0.33.0`. As of `2026-03-12`, the official docs are split:

- `https://trio.readthedocs.io/en/stable/` still serves Trio `0.32.0` docs.
- `https://trio.readthedocs.io/en/latest/` and the GitHub `v0.33.0` release notes cover the current `0.33.0` changes.

For package-pinned `0.33.0` work, use the stable tutorial/reference pages for the enduring APIs, then verify `0.33.0`-specific behavior against PyPI and the `v0.33.0` release notes.

## Installation

Install the pinned package version:

```bash
pip install "trio==0.33.0"
```

```bash
poetry add "trio==0.33.0"
```

```bash
uv add trio==0.33.0
```

Verify the installed version:

```bash
python -c "import trio; print(trio.__version__)"
```

If the project still runs on Python `3.9`, Trio `0.33.0` is not compatible. Either upgrade Python or keep the older Trio line intentionally.

## Initialize And Run

Every Trio program starts by handing one async entry point to `trio.run(...)`:

```python
import trio

async def main() -> None:
    print("hello from trio")
    await trio.sleep(1)

if __name__ == "__main__":
    trio.run(main)
```

Practical rules:

- `trio.run()` creates and owns the Trio runtime for that call.
- Do not call `trio.run()` from inside another Trio task.
- If you need sync setup code, do it before `trio.run(...)` or offload it into a thread once Trio is running.

## Core Usage Pattern: Nurseries

Use a nursery for every group of sibling tasks. Trio waits for child tasks before leaving the nursery and propagates failures through structured cancellation.

```python
import trio

async def worker(name: str, delay: float) -> None:
    while True:
        print(f"{name} tick")
        await trio.sleep(delay)

async def main() -> None:
    async with trio.open_nursery() as nursery:
        nursery.start_soon(worker, "fast", 0.5)
        nursery.start_soon(worker, "slow", 1.0)

        await trio.sleep(3)
        nursery.cancel_scope.cancel()

trio.run(main)
```

Use `nursery.start(...)` when the parent must wait for the child task to signal readiness:

```python
import trio

async def start_service(*, task_status=trio.TASK_STATUS_IGNORED) -> None:
    await trio.sleep(0.1)
    task_status.started("ready")
    await trio.sleep_forever()

async def main() -> None:
    async with trio.open_nursery() as nursery:
        state = await nursery.start(start_service)
        print(state)
        nursery.cancel_scope.cancel()

trio.run(main)
```

Use `start_soon(...)` for fire-and-run sibling work. Use `start(...)` for boot sequences, listeners, or background services that must confirm they started cleanly.

## Timeouts, Cancellation, And Shutdown

Trio cancellation is explicit and scoped. Prefer Trio's timeout helpers over manual deadline bookkeeping.

```python
import trio

async def fetch_with_timeout() -> bytes:
    with trio.fail_after(5):
        await trio.sleep(1)
        return b"done"

async def optional_work() -> None:
    with trio.move_on_after(2) as scope:
        await trio.sleep(10)

    if scope.cancelled_caught:
        print("timed out without raising")

async def main() -> None:
    print(await fetch_with_timeout())
    await optional_work()

trio.run(main)
```

Guidance:

- `fail_after(...)` raises `TooSlowError` when the deadline expires.
- `move_on_after(...)` cancels enclosed work but suppresses the timeout exception.
- Nursery shutdown is driven by `nursery.cancel_scope.cancel()`.
- These are synchronous context managers inside async code: use `with trio.fail_after(...):`, not `async with`.

## Channels And Task Coordination

`trio.open_memory_channel()` is Trio's in-process queue primitive.

```python
import trio

async def producer(send_channel: trio.MemorySendChannel[int]) -> None:
    async with send_channel:
        for value in range(3):
            await send_channel.send(value)

async def consumer(receive_channel: trio.MemoryReceiveChannel[int]) -> None:
    async with receive_channel:
        async for value in receive_channel:
            print(value)

async def main() -> None:
    send_channel, receive_channel = trio.open_memory_channel[int](0)

    async with trio.open_nursery() as nursery:
        nursery.start_soon(producer, send_channel.clone())
        nursery.start_soon(consumer, receive_channel)
        await send_channel.aclose()

trio.run(main)
```

Important channel semantics:

- Capacity `0` gives backpressure; positive capacities add buffering.
- Use `.clone()` when multiple tasks need the same endpoint.
- Close every clone. Receivers only see end-of-stream after every send endpoint is closed.

## Networking, Processes, And Blocking Code

Trio includes practical primitives for low-level networking, subprocesses, and thread offloading.

TCP client:

```python
import trio

async def main() -> None:
    stream = await trio.open_tcp_stream("example.com", 80)
    async with stream:
        await stream.send_all(
            b"GET / HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Connection: close\r\n\r\n"
        )
        response = await stream.receive_some(4096)
        print(response.decode("ascii", errors="replace"))

trio.run(main)
```

Run blocking sync code safely:

```python
import time
import trio

def blocking_lookup() -> str:
    time.sleep(2)
    return "ok"

async def main() -> None:
    result = await trio.to_thread.run_sync(blocking_lookup)
    print(result)

trio.run(main)
```

Run a subprocess:

```python
import trio

async def main() -> None:
    completed = await trio.run_process(
        ["python", "-c", "print('hello from child')"],
        capture_stdout=True,
    )
    print(completed.stdout.decode().strip())

trio.run(main)
```

## Configuration / Auth

Trio itself has no package-level authentication layer and no global client object.

- Configure behavior at the call site: deadlines, nursery layout, channel capacity, socket settings, subprocess options, and thread-offload boundaries.
- Network authentication belongs to the higher-level protocol library you are using on top of Trio, not to Trio itself.
- Prefer explicit arguments and task-local state over mutable globals shared across tasks.
- If you need ecosystem portability between Trio and `asyncio`, consider an adapter layer such as AnyIO instead of mixing runtime-specific APIs directly.

## Testing

Trio ships dedicated test helpers in `trio.testing`.

- Import `trio.testing` explicitly; it is not imported automatically by `import trio`.
- Use `trio.testing.MockClock` when you need deterministic time control or auto-jumping sleeps.
- Confirm the project's runner before authoring fixtures. Common choices are `pytest-trio` or AnyIO's pytest plugin.

Example:

```python
import trio
import trio.testing

async def main() -> None:
    await trio.sleep(10)

clock = trio.testing.MockClock(autojump_threshold=0)
trio.run(main, clock=clock)
```

## Common Pitfalls

- Do not create unmanaged background tasks. Trio expects task lifetimes to be scoped by a nursery.
- Do not block inside async functions with `time.sleep()`, sync DB drivers, or CPU-heavy work. Use `trio.to_thread.run_sync(...)` or a process boundary.
- Do not forget to close channel clones. Hanging receives usually mean one send endpoint is still open.
- Use `nursery.start(...)` for readiness handshakes. `start_soon(...)` does not tell you when startup finished.
- Do not assume `asyncio`-only libraries will work under Trio. Use AnyIO-compatible libraries or explicit bridges.
- Import `trio.testing` explicitly before using `MockClock`, memory stream pairs, or other test helpers.

## Version-Sensitive Notes For 0.33.0

- Trio `0.33.0` was released on `2026-02-14` and requires Python `>=3.10`.
- The official `stable` docs currently lag at Trio `0.32.0`. For `0.33.0`-specific behavior, cross-check PyPI and the GitHub `v0.33.0` release notes.
- `trio.testing.RaisesGroup` and `trio.testing.Matcher` are deprecated in `0.33.0`; prefer `pytest.RaisesGroup` and `pytest.RaisesExc` in new tests.
- Android's `sys.platform == "android"` support was added in `0.33.0`.

## Official Sources

- Stable docs root: https://trio.readthedocs.io/en/stable/
- Stable tutorial: https://trio.readthedocs.io/en/stable/tutorial.html
- Stable core reference: https://trio.readthedocs.io/en/stable/reference-core.html
- Stable I/O reference: https://trio.readthedocs.io/en/stable/reference-io.html
- Stable testing reference: https://trio.readthedocs.io/en/stable/reference-testing.html
- Latest docs root: https://trio.readthedocs.io/en/latest/
- Latest history / release notes: https://trio.readthedocs.io/en/latest/history.html
- PyPI project page: https://pypi.org/project/trio/
- PyPI release metadata: https://pypi.org/pypi/trio
- GitHub release notes: https://github.com/python-trio/trio/releases/tag/v0.33.0

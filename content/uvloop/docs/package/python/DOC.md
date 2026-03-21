---
name: package
description: "uvloop Python package guide for replacing the default asyncio event loop with the libuv-based implementation"
metadata:
  languages: "python"
  versions: "0.22.1"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "uvloop,asyncio,event-loop,async,libuv,performance"
---

# uvloop Python Package Guide

## Golden Rule

Use `uvloop` only as an event-loop implementation choice for applications you control at process startup. Prefer `uvloop.run(...)` or `asyncio.Runner(loop_factory=uvloop.new_event_loop)` over policy hacks in new code.

After setup, write standard `asyncio` code. `uvloop` does not change the coroutine API.

## What It Is

`uvloop` is a drop-in implementation of `asyncio.AbstractEventLoop` built on top of `libuv`. It is mainly useful for network-heavy async services that already use normal `asyncio` primitives and want a faster event loop without rewriting application code.

Use it when:

- the app already uses `asyncio`
- you can choose the process event loop at startup
- you run on supported POSIX or macOS targets

Skip it when:

- the runtime is Windows
- the host framework or platform already owns loop setup in a way you cannot safely replace
- code depends on old `asyncio.get_event_loop()` fallback behavior

## Installation

`uvloop` 0.22.1 requires CPython and Python `>=3.8.1`.

Install the pinned version:

```bash
pip install uvloop==0.22.1
```

With `uv`:

```bash
uv add uvloop==0.22.1
```

For projects that resolve latest allowed versions:

```bash
uv pip install uvloop
```

Practical install notes:

- Wheels are published for common CPython builds, so most macOS and Linux installs should not need a local compile toolchain.
- Package metadata lists `MacOS :: MacOS X` and `POSIX`; do not assume Windows support.
- If pip falls back to building from source, upgrade `pip` first as upstream recommends.

## Initialize At Startup

### Preferred: `uvloop.run(...)`

For top-level application entry points, use `uvloop.run(...)`.

```python
import asyncio
import uvloop

async def main() -> None:
    await asyncio.sleep(0.1)
    print("running on uvloop")

if __name__ == "__main__":
    uvloop.run(main())
```

This is the current upstream-recommended pattern.

### When You Need Runner Options

If you need `debug=True` or other `asyncio.Runner` control, pass `uvloop.new_event_loop` as the loop factory.

```python
import asyncio
import uvloop

async def main() -> None:
    await asyncio.sleep(0.1)

with asyncio.Runner(loop_factory=uvloop.new_event_loop, debug=True) as runner:
    runner.run(main())
```

### Global Policy Installation

Use policy installation only when some other part of startup still calls `asyncio.run(...)` or otherwise expects the process-wide loop policy to be set first.

```python
import asyncio
import uvloop

asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())

async def main() -> None:
    await asyncio.sleep(0.1)

asyncio.run(main())
```

`uvloop.install()` is a shorthand for setting the event loop policy:

```python
import uvloop

uvloop.install()
```

Prefer explicit startup ownership instead of scattering `install()` calls across library code.

## Core Usage

Once active, use normal `asyncio` APIs.

### Concurrent Work

```python
import asyncio
import uvloop

async def work(name: str, delay: float) -> str:
    await asyncio.sleep(delay)
    return f"{name} done"

async def main() -> None:
    results = await asyncio.gather(
        work("a", 0.05),
        work("b", 0.10),
        work("c", 0.02),
    )
    print(results)

uvloop.run(main())
```

### TCP Server

```python
import asyncio
import uvloop

async def handle_echo(
    reader: asyncio.StreamReader,
    writer: asyncio.StreamWriter,
) -> None:
    data = await reader.read(1024)
    writer.write(data)
    await writer.drain()
    writer.close()
    await writer.wait_closed()

async def main() -> None:
    server = await asyncio.start_server(handle_echo, "127.0.0.1", 9000)
    async with server:
        await server.serve_forever()

uvloop.run(main())
```

### Explicit Loop Ownership

Use `uvloop.new_event_loop()` only when code needs direct loop ownership.

```python
import asyncio
import uvloop

loop = uvloop.new_event_loop()
asyncio.set_event_loop(loop)

try:
    loop.run_until_complete(asyncio.sleep(0.1))
finally:
    loop.close()
```

## Config And Auth

`uvloop` has no auth model, credentials, service endpoints, or package-specific environment variables.

The configuration surface is mainly:

- which startup pattern you use: `uvloop.run`, `asyncio.Runner(..., loop_factory=...)`, or policy install
- whether asyncio debug mode is enabled
- whether the process should use `uvloop` globally or only for one owned runner

Recommended defaults:

- app entry point you control: `uvloop.run(main())`
- app entry point with explicit debug or runner settings: `asyncio.Runner(loop_factory=uvloop.new_event_loop, ...)`
- legacy bootstrap needing a global loop policy: `asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())`

## Common Pitfalls

- Install or select `uvloop` before creating the first long-lived loop, transport, server, or client.
- Do not call `uvloop.run()` from inside an already-running loop. In notebooks, REPLs, and some test runners, let the host manage the loop.
- Do not assume Windows support. Package metadata still advertises macOS and POSIX targets only.
- Avoid setting loop policy from reusable library code. That is an application-startup decision.
- If code still calls `asyncio.get_event_loop()` with no current loop, test carefully on `0.22.1`; policy-installed `uvloop` may raise instead of lazily creating one.
- Benchmark the real workload before forcing `uvloop` everywhere. Blocking code, heavy CPU work, or framework-specific behavior can erase the benefit.

## Version-Sensitive Notes For 0.22.1

- PyPI lists `0.22.1` as the current release, published on `2025-10-16`.
- The maintainer GitHub release for `v0.22.1` says it is identical to `0.22.0` and was re-run with CI fixes. Treat the functional changes as the `0.22.0` line.
- The `v0.22.0` release notes call out Python `3.14` fixes and free-threading support work.
- The old Read the Docs user guide is stale for current usage: it still mentions Python `3.5` and older policy-based setup. For modern code, prefer the PyPI/GitHub README guidance built around `uvloop.run(...)`.
- The maintainer issue tracker currently has an open `0.22.1` report that `asyncio.get_event_loop()` after `uvloop.install()` raises when no current loop exists. Write code against `asyncio.run(...)`, `asyncio.Runner(...)`, or explicit `new_event_loop()` instead of depending on implicit loop creation.

## Official Sources Used

- Docs landing page: https://uvloop.readthedocs.io/
- User guide: https://uvloop.readthedocs.io/user/
- PyPI package page: https://pypi.org/project/uvloop/
- PyPI JSON metadata: https://pypi.org/pypi/uvloop/json
- GitHub repository: https://github.com/MagicStack/uvloop
- GitHub README: https://github.com/MagicStack/uvloop/blob/master/README.rst
- GitHub releases: https://github.com/MagicStack/uvloop/releases
- Maintainer issue about `get_event_loop()` behavior: https://github.com/MagicStack/uvloop/issues/702

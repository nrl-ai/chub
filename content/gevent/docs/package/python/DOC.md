---
name: package
description: "gevent package guide for Python covering greenlets, monkey patching, pools, queues, DNS, and WSGI serving"
metadata:
  languages: "python"
  versions: "25.9.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "gevent,greenlet,python,concurrency,networking,event-loop"
---

# gevent Python Package Guide

## What It Is

`gevent` is a coroutine-based networking library that uses `greenlet` to provide a mostly synchronous Python API on top of `libev` or `libuv`. Use it for I/O-bound concurrency, cooperative servers, and adapting blocking-looking code into greenlet-friendly workflows.

This guide targets the PyPI release `25.9.1`. The official docs host currently renders development docs (`25.9.2.dev0`, and some pages such as `gevent.queue` show `25.8.3.dev0`), so treat `docs.gevent.org` as current upstream reference instead of a version-pinned snapshot.

## Installation

```bash
python -m pip install gevent==25.9.1
```

`25.9.1` requires Python `>=3.9`.

Useful extras published on PyPI:

- `recommended`: default choice when you want the usual optional runtime helpers
- `monitor`: monitoring/debugging dependencies
- `dnspython`: alternate DNS resolver support
- `docs` and `test`: only for documentation or gevent development workflows

```bash
python -m pip install "gevent[recommended]==25.9.1"
```

If no compatible wheel is available for your platform, `pip` falls back to a source build.

## Pick An Integration Style

### Use gevent APIs directly

This is the safest model when you control the code path and can use gevent-native modules explicitly.

```python
import gevent

def work(label, delay):
    gevent.sleep(delay)
    return f"{label} finished"

jobs = [
    gevent.spawn(work, "a", 0.2),
    gevent.spawn(work, "b", 0.1),
]

gevent.joinall(jobs, raise_error=True)
results = [job.value for job in jobs]
print(results)
```

Practical rules:

- Use `gevent.sleep()` to yield cooperatively.
- `gevent.sleep(0)` yields to runnable greenlets, but repeated zero-sleeps can still delay I/O-heavy work for a short period.
- Use `gevent.joinall(..., raise_error=True)` or inspect `greenlet.exception`; spawned greenlets keep exceptions on the greenlet object.

### Monkey patch the standard library

Use monkey patching when you need existing stdlib-style networking code to become cooperative.

```python
from gevent import monkey

monkey.patch_all()

import gevent
from urllib.request import urlopen

def fetch(url):
    with urlopen(url, timeout=5) as response:
        return response.read()

jobs = [
    gevent.spawn(fetch, "https://example.com"),
    gevent.spawn(fetch, "https://example.org"),
]

gevent.joinall(jobs, raise_error=True)
payloads = [job.value for job in jobs]
```

Upstream guidance is strict here:

- Call `monkey.patch_all()` as early as possible in the main module.
- Patch on the main thread while the process is still single-threaded.
- Late patching can leave some modules on blocking sockets or other incompatible primitives.
- Some frameworks patch for you; verify that before adding a second patch layer.

## Core Usage Patterns

### Bound concurrency with `Pool`

```python
from gevent import joinall, sleep
from gevent.pool import Pool

pool = Pool(10)

def fetch_one(item):
    sleep(0.1)
    return item * 2

jobs = [pool.spawn(fetch_one, item) for item in range(100)]
joinall(jobs, raise_error=True)
values = [job.value for job in jobs]
```

Use a pool when you need backpressure instead of spawning an unbounded number of greenlets.

### Coordinate work with `gevent.queue`

In `25.9.1`, `gevent.queue.Queue` is the joinable queue type. `JoinableQueue` remains as a backward-compatibility alias, and `SimpleQueue` is the lightweight queue without `join()` and `task_done()`.

```python
import gevent
from gevent.queue import Queue

queue = Queue()

def producer():
    for item in range(5):
        queue.put(item)
    queue.put(None)

def consumer():
    seen = []
    while True:
        item = queue.get()
        if item is None:
            queue.task_done()
            break
        seen.append(item)
        queue.task_done()
    return seen

prod = gevent.spawn(producer)
cons = gevent.spawn(consumer)

gevent.joinall([prod], raise_error=True)
queue.join()
gevent.joinall([cons], raise_error=True)
print(cons.value)
```

### Timeouts

```python
import gevent

with gevent.Timeout(2, TimeoutError("operation timed out")):
    gevent.sleep(3)
```

Wrap risky operations so one blocked greenlet does not stall the overall workflow indefinitely.

### WSGI serving with `gevent.pywsgi`

```python
from gevent.pywsgi import WSGIServer

def app(environ, start_response):
    body = b"ok\n"
    start_response(
        "200 OK",
        [
            ("Content-Type", "text/plain"),
            ("Content-Length", str(len(body))),
        ],
    )
    return [body]

WSGIServer(("127.0.0.1", 8080), app).serve_forever()
```

`gevent.pywsgi.WSGIServer` is the standard gevent-hosted WSGI entrypoint.

## Configuration And Environment

`gevent` has no auth model of its own. Configuration is about loop selection, DNS behavior, monitoring, file-object strategy, and related runtime controls.

If you set options in code, do it before importing modules that initialize the hub or start using sockets.

```python
from gevent import config

config.loop = "libuv"
config.resolver = ["thread", "dnspython"]
config.monitor_thread = True
config.max_blocking_time = 0.5
```

Useful environment variables:

```bash
export GEVENT_LOOP=libuv
export GEVENT_RESOLVER=thread
export GEVENT_MONITOR_THREAD_ENABLE=1
export GEVENT_MAX_BLOCKING_TIME=0.5
export GEVENT_MONITOR_PRINT_BLOCKING_REPORTS=1
```

Practical notes:

- On Windows, the loop default is `libuv`; on other platforms, the default preference is `libev` first.
- The default resolver preference is `thread`, then `dnspython`, then `ares`, then `block`.
- `GEVENT_RESOLVER_NAMESERVERS` accepts IP addresses only; invalid values can hang resolver backends such as `dnspython`.
- `GEVENT_MONITOR_THREAD_ENABLE=1` starts a native monitoring thread that reports blocked hubs.
- `GEVENT_MONITOR_PRINT_BLOCKING_REPORTS` was added in `25.4.1`; it defaults to `True` when monitoring is enabled.

## Common Pitfalls

- Late monkey patching: patch before importing modules that create sockets, locks, selectors, subprocess helpers, or threads.
- CPU-bound work: greenlets do not give CPU parallelism. Use processes or native extensions for CPU-heavy jobs.
- Hidden blocking calls: unpatched C extensions, file APIs, or third-party libraries can still block the hub.
- Queue internals: after the `25.4.1` queue rename, rely on documented queue APIs like `put`, `get`, `task_done`, and `join`, not internal attributes.
- Version drift: `docs.gevent.org` tracks current development state, so confirm version-sensitive behavior against the installed release.
- Interpreter mode: free-threaded CPython 3.13/3.14 builds are not supported by gevent even though the package can build there.
- Platform assumptions: Windows support is best-effort and upstream does not recommend it for production.

## Version-Sensitive Notes For `25.9.1`

- PyPI marks `25.9.1` as the latest release and shows its release date as September 17, 2025.
- `25.9.1` fixes a `TypeError` in the C extensions when putting items into a full `SimpleQueue`. If you are stuck on `25.4.1` through `25.8.x`, upstream lists `PURE_PYTHON=1` or `GEVENT_PURE_PYTHON=1` as a workaround.
- `25.4.1` changed queue behavior in ways that matter for porting code: `Queue` became the joinable queue type, `SimpleQueue` became the simple queue type, and `patch_all()` started patching stdlib `queue.Queue`, `PriorityQueue`, and `LifoQueue` by default. Upstream explicitly notes that this solved a known `urllib3` deadlock.
- `25.4.1` also added `GEVENT_MONITOR_PRINT_BLOCKING_REPORTS`, which is useful when enabling monitor-thread diagnostics in long-running services.
- `25.5.1` updated the bundled `libuv` and raised wheel baselines to newer OS versions, including Linux kernel `3.10` with `glibc 2.17`, macOS `11`, Windows `10`, and FreeBSD `12`.
- `25.8.1` says Python `3.9` support is expected to be removed soon. For new services, prefer Python `3.10+` unless you are pinned by project constraints.

## Official Sources Used

- API landing page: https://www.gevent.org/api/
- Docs root: https://docs.gevent.org/
- Common functions API: https://docs.gevent.org/api/gevent.html
- Monkey patching API: https://docs.gevent.org/api/gevent.monkey.html
- Queue API: https://docs.gevent.org/api/gevent.queue.html
- Configuration guide: https://docs.gevent.org/configuration.html
- Changelog: https://docs.gevent.org/changelog.html
- PyPI release page: https://pypi.org/project/gevent/25.9.1/

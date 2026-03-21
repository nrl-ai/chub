---
name: package
description: "Eventlet package guide for Python cooperative networking, green threads, monkey patching, and WSGI services"
metadata:
  languages: "python"
  versions: "0.40.4"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "eventlet,python,networking,concurrency,green-threads,wsgi"
---

# eventlet Python Package Guide

## Golden Rule

**Treat Eventlet as a compatibility or maintenance dependency, not a first choice for new async work.**

The upstream maintainers explicitly discourage new usage except for maintaining or migrating existing deployments. If you are working on an Eventlet-based service, decide at process start whether the app will use monkey patching, and keep blocking code away from the green-threaded path.

## Installation

```bash
pip install eventlet==0.40.4
```

```bash
uv add eventlet==0.40.4
```

```bash
poetry add eventlet==0.40.4
```

PyPI metadata for `0.40.4` requires Python `>=3.9`.

## Initialization And Setup

### Minimal green-thread usage

Use `spawn()` to run cooperative tasks and `wait()` to join them.

```python
import eventlet

def fetch(name: str, delay: float) -> str:
    print(f"starting {name}")
    eventlet.sleep(delay)
    return f"finished {name}"

gt1 = eventlet.spawn(fetch, "alpha", 0.1)
gt2 = eventlet.spawn(fetch, "beta", 0.2)

print(gt1.wait())
print(gt2.wait())
```

### Bounded concurrency with `GreenPool`

Use a `GreenPool` when you need a fixed concurrency limit instead of unbounded `spawn()` calls.

```python
import eventlet

pool = eventlet.GreenPool(size=20)

def work(item: int) -> int:
    eventlet.sleep(0.05)
    return item * 2

for result in pool.imap(work, range(5)):
    print(result)
```

### Monkey patching

If your app expects standard-library networking and threading modules to cooperate with Eventlet, patch as early as possible, before importing modules that capture blocking implementations.

```python
import eventlet

eventlet.monkey_patch()

import socket
from urllib import request
```

You can patch selectively:

```python
import eventlet

eventlet.monkey_patch(socket=True, select=True, thread=True)
```

`monkey_patch()` is safe to call multiple times, but late patching is the common failure mode. Choose either a fully patched process model or an explicit `eventlet.green.*` import model and keep that decision consistent.

### import_patched for targeted imports

If you do not want global monkey patching, use `eventlet.patcher.import_patched()` to import specific modules against green versions of standard-library dependencies.

```python
from eventlet.patcher import import_patched

urllib_request = import_patched("urllib.request")
```

## Core Usage Patterns

### Cooperative sleeps and timeouts

Use `eventlet.sleep()` instead of `time.sleep()` inside green-threaded code, and wrap latency-sensitive work with `Timeout`.

```python
import eventlet

try:
    with eventlet.Timeout(1.0):
        eventlet.sleep(5)
except eventlet.Timeout:
    print("operation timed out")
```

Passing `False` as the second `Timeout` argument suppresses the exception when you want a soft deadline instead of cancellation.

### Simple WSGI server

`eventlet.wsgi.server()` is the standard built-in way to serve a WSGI app.

```python
import eventlet
from eventlet import wsgi

def app(env, start_response):
    start_response("200 OK", [("Content-Type", "text/plain")])
    return [b"hello from eventlet\n"]

listener = eventlet.listen(("127.0.0.1", 8080))
wsgi.server(listener, app)
```

`wsgi.server()` runs in the current greenthread and handles requests cooperatively. If a handler calls blocking code that Eventlet cannot patch, the whole worker can stall.

### Explicit green networking modules

When you choose not to monkey patch globally, use `eventlet.green.*` modules explicitly.

```python
from eventlet.green import socket

client = socket.socket()
client.connect(("example.com", 80))
client.sendall(b"GET / HTTP/1.0\r\nHost: example.com\r\n\r\n")
print(client.recv(4096))
```

## Configuration And Auth

Eventlet has no authentication layer of its own. The configuration that matters is runtime behavior: hub selection, DNS behavior, and the threadpool used for blocking native work.

### Select the hub early

If the default event loop hub is not appropriate for the platform, select a different one before creating sockets, timers, or servers.

```python
import eventlet.hubs

eventlet.hubs.use_hub("poll")
```

You can also set the hub via the `EVENTLET_HUB` environment variable before process start.

### DNS behavior

If Eventlet's green DNS resolver causes issues in your environment, disable it before import:

```bash
EVENTLET_NO_GREENDNS=yes python app.py
```

### Native threadpool size

If you rely on `eventlet.tpool` for blocking native work, set the size before importing Eventlet:

```bash
EVENTLET_THREADPOOL_SIZE=32 python app.py
```

### No built-in auth or credentials model

Eventlet does not define service credentials, tokens, or connection auth. Any auth configuration belongs to the protocol or framework layered on top of Eventlet, such as WSGI middleware, WebSocket handshake logic, or outbound client libraries.

## Common Pitfalls

- Do not call `eventlet.monkey_patch()` after importing libraries like `socket`, `ssl`, database clients, or HTTP stacks that may already hold blocking objects.
- Do not use `time.sleep()` in green-threaded code. Use `eventlet.sleep()`.
- Do not assume all third-party libraries become cooperative after patching. Blocking C extensions and unpatched I/O still block the hub.
- Be careful when mixing Eventlet with `asyncio`, native threads, or process pools. Keep boundaries explicit instead of sharing sockets or synchronization primitives casually.
- Verify DNS behavior in production-like environments. `greendns` can behave differently from the platform resolver.
- For new server development, prefer an asyncio-native stack unless you are constrained by an existing Eventlet-based framework or deployment path.

## Version-Sensitive Notes

- This guide targets `eventlet==0.40.4`, but the readthedocs pages I verified still rendered as `Eventlet 0.40.0 documentation` under `/latest/`. Treat the docs as close to current, not patch-exact.
- PyPI metadata for `0.40.4` lists Python `>=3.9`. If you are reviving an older Eventlet deployment, re-check interpreter support instead of assuming historical versions still apply.
- Upstream docs now include an asyncio migration guide and the repository README warns against new Eventlet usage. If you are touching a long-lived Eventlet service, plan around migration rather than expanding Eventlet-specific surface area.

## Official Sources

- Eventlet docs root: https://eventlet.readthedocs.io/en/latest/
- Basic usage: https://eventlet.readthedocs.io/en/latest/basic_usage.html
- Patching: https://eventlet.readthedocs.io/en/latest/patching.html
- WSGI: https://eventlet.readthedocs.io/en/latest/modules/wsgi.html
- Environment variables: https://eventlet.readthedocs.io/en/latest/environment.html
- Migration guide: https://eventlet.readthedocs.io/en/latest/asyncio/migration.html
- PyPI project: https://pypi.org/project/eventlet/
- Upstream repository README: https://github.com/eventlet/eventlet

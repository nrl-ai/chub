---
name: package
description: "Twisted package guide for Python event-driven networking, Deferreds, reactors, and protocol services"
metadata:
  languages: "python"
  versions: "25.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "twisted,python,async,networking,reactor,deferred,protocols"
---

# Twisted Python Package Guide

## What It Is

`Twisted` is an event-driven networking framework for Python. Use it when you need protocol servers or clients, long-lived connections, TLS-enabled networking, or a mature callback-based async runtime that can also interoperate with `asyncio`.

- Package name on PyPI: `Twisted`
- Import namespace: `twisted.*`
- Version covered: `25.5.0`
- Python requirement on PyPI: `>=3.8`

Twisted code is usually organized around:

- one reactor per process
- `Deferred` values for async results
- protocol and transport objects for socket-level work
- endpoints for portable listener and connector setup

## Installation

Install the version your project expects:

```bash
python -m pip install "Twisted==25.5.0"
```

Common extras from the official installation docs:

```bash
python -m pip install "Twisted[tls]==25.5.0"
python -m pip install "Twisted[http2]==25.5.0"
python -m pip install "Twisted[websocket]==25.5.0"
python -m pip install "Twisted[conch]==25.5.0"
```

- `tls`: TLS client and server support
- `http2`: HTTP/2 support
- `websocket`: Twisted Web websocket support
- `conch`: SSH and Telnet features

If you need several optional stacks on a non-platform-specific deployment target, Twisted also documents `all-non-platform`.

## Initialization And Setup

### Pick The Reactor Before Importing `reactor`

If you want Twisted to run on top of `asyncio`, install the asyncio reactor before importing `twisted.internet.reactor` or any module that may import it implicitly:

```python
import asyncio

from twisted.internet import asyncioreactor

loop = asyncio.new_event_loop()
asyncio.set_event_loop(loop)
asyncioreactor.install(loop)

from twisted.internet import reactor
```

Windows note: the official `AsyncioSelectorReactor` docs say Python 3.8+ on Windows needs `asyncio.WindowsSelectorEventLoopPolicy()` before creating the event loop.

```python
import asyncio

asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
```

If you are not integrating with `asyncio`, the platform default reactor is usually fine.

### Prefer Endpoints Over Manual Socket Setup

Endpoint strings are Twisted's portable way to describe listeners and outbound connections:

```python
from twisted.internet import endpoints, protocol, reactor

class Echo(protocol.Protocol):
    def dataReceived(self, data: bytes) -> None:
        self.transport.write(data)

endpoint = endpoints.serverFromString(
    reactor,
    "tcp:1234:interface=127.0.0.1",
)
endpoint.listen(protocol.Factory.forProtocol(Echo))
reactor.run()
```

For outbound connections, use `clientFromString`:

```python
from twisted.internet import endpoints, protocol, reactor

class Client(protocol.Protocol):
    def connectionMade(self) -> None:
        self.transport.write(b"ping")

factory = protocol.ClientFactory.forProtocol(Client)
endpoint = endpoints.clientFromString(reactor, "tcp:host=127.0.0.1:port=1234")
endpoint.connect(factory)
reactor.run()
```

### Prefer `task.react()` For One-Shot Programs

The official task API recommends `task.react()` instead of calling `reactor.run()` directly for scripts and clients with a clear completion condition:

```python
from twisted.internet import task

async def main(reactor, name):
    print(f"hello {name}")

task.react(main, ("twisted",))
```

`task.react()` stops the reactor exactly once, logs failures, and exits with a non-zero code on failure.

## Core Usage

### `Deferred` Basics

Twisted's primary async primitive is `Deferred`. Attach callbacks and errbacks to handle completion:

```python
from twisted.internet import defer

def load_config():
    return defer.succeed({"workers": 4})

def show_config(config):
    print(config["workers"])
    return config

d = load_config()
d.addCallback(show_config)
d.addErrback(lambda failure: failure.printTraceback())
```

### Prefer `Deferred.fromCoroutine` When Bridging `async def`

The current Twisted docs say `Deferred.fromCoroutine` is intended to replace the more type-ambiguous `ensureDeferred` in this role:

```python
from twisted.internet.defer import Deferred
from twisted.internet import task

async def compute_total():
    return 42

def main(reactor):
    d = Deferred.fromCoroutine(compute_total())
    d.addCallback(print)
    return d

task.react(main)
```

### `inlineCallbacks` Still Works

If a codebase already uses `inlineCallbacks`, keep it consistent rather than mixing styles arbitrarily:

```python
from twisted.internet import defer, task

def read_settings():
    return defer.succeed({"debug": True})

@defer.inlineCallbacks
def main(reactor):
    settings = yield read_settings()
    print(settings["debug"])

task.react(main)
```

### HTTP Client Requests With `Agent`

`twisted.web.client.Agent` is the low-level official HTTP client. Its `request()` method takes byte strings for the method and URI:

```python
from twisted.internet import task
from twisted.web.client import Agent, readBody

def main(reactor):
    agent = Agent(reactor)
    d = agent.request(
        b"GET",
        b"https://example.com/",
    )

    def got_response(response):
        return readBody(response)

    def got_body(body):
        print(body.decode("utf-8")[:120])

    d.addCallback(got_response)
    d.addCallback(got_body)
    return d

task.react(main)
```

### Move Blocking Work Off The Reactor

Twisted's threading guide is explicit: do not call blocking code in the reactor thread. Use `threads.deferToThread()` when you need a result back as a `Deferred`:

```python
from twisted.internet import task, threads

def blocking_read():
    with open("payload.bin", "rb") as fh:
        return fh.read()

def main(reactor):
    d = threads.deferToThread(blocking_read)
    d.addCallback(lambda data: print(len(data)))
    return d

task.react(main)
```

If another thread needs to schedule work on the reactor, use `reactor.callFromThread(...)` instead of calling Twisted APIs directly from that thread.

## Configuration And Auth

Twisted does not have a package-wide config file or global auth model. In practice, configuration lives in:

- which reactor you install
- endpoint strings for listener and connector setup
- TLS context and certificate settings
- protocol-specific settings in your application
- optional extras such as `tls`, `http2`, `websocket`, or `conch`

HTTP auth is handled in your request code. For example, attach headers explicitly:

```python
from twisted.internet import reactor
from twisted.web.client import Agent
from twisted.web.http_headers import Headers

agent = Agent(reactor)
headers = Headers(
    {
        b"authorization": [b"Bearer token-value"],
        b"user-agent": [b"my-app/1.0"],
    }
)
```

For HTTPS customization, stay within `twisted.web.client` policy/context APIs instead of mixing in unrelated SSL setup code.

## Common Pitfalls

### Installing A Reactor Too Late

Import order matters. If you need `asyncioreactor`, install it before importing `reactor`.

### Assuming The Reactor Is Restartable

Twisted reactors are meant to start once per process. If you stop a reactor and try to run it again, Twisted raises `ReactorNotRestartable`.

### Blocking The Reactor Thread

Avoid `time.sleep()`, synchronous HTTP clients, direct blocking database calls, or large filesystem operations on the reactor thread.

### Calling Twisted APIs From Worker Threads

Most Twisted objects are not thread-safe. Use `reactor.callFromThread()` to get back onto the reactor thread.

### Forgetting That Many APIs Want `bytes`

Protocol payloads and APIs such as `Agent.request()` operate on bytes. Encode and decode at boundaries instead of mixing `str` and `bytes` ad hoc.

### Using Manual Socket Setup When Endpoints Already Solve It

Endpoints are the documented portability layer. Prefer `serverFromString()` and `clientFromString()` over custom socket setup code.

## Version-Sensitive Notes For 25.5.0

- The version used here is `25.5.0`, released on `2025-06-07`.
- The official stable docs matched `25.5.0` when checked on `2026-03-12`, but the stable URL is not permanently version-pinned. Re-check if you curate this package later.
- PyPI reports `Requires: Python >=3.8.0` and lists extras including `tls`, `http2`, `websocket`, `conch`, and `serial`.
- The `25.5.0` release removed `twisted.internet.defer.waitForDeferred`. Older recipes using it should be rewritten with native coroutines, `Deferred.fromCoroutine`, or existing `inlineCallbacks` patterns.
- The `25.5.0` release also removed or privatized `twisted.trial.unittest.TestCase.deferSetUp`, `deferTestMethod`, `deferTearDown`, and `deferRunCleanups`. Older Trial examples that depend on those hooks are stale.
- The websocket extra is worth checking explicitly in `25.5.0` because current Twisted Web websocket docs require `Twisted[websocket]` or a bundle that includes it.

## Official Sources

- Docs root: https://docs.twisted.org/en/stable/
- Installation: https://docs.twisted.org/en/stable/installation.html
- Deferred intro: https://docs.twisted.org/en/latest/core/howto/defer-intro.html
- Endpoints: https://docs.twisted.org/en/stable/core/howto/endpoints.html
- Threading: https://docs.twisted.org/en/stable/core/howto/threading.html
- `task.react`: https://docs.twisted.org/en/stable/api/twisted.internet.task.html
- `Agent`: https://docs.twisted.org/en/stable/api/twisted.web.client.Agent.html
- `AsyncioSelectorReactor`: https://docs.twisted.org/en/stable/api/twisted.internet.asyncioreactor.AsyncioSelectorReactor.html
- PyPI: https://pypi.org/project/Twisted/25.5.0/
- Release notes: https://github.com/twisted/twisted/releases/tag/twisted-25.5.0

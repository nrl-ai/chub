---
name: package
description: "websockets Python package guide for 16.0 asyncio and threading clients and servers"
metadata:
  languages: "python"
  versions: "16.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "websockets,websocket,asyncio,threading,realtime,networking,proxy"
---

# websockets Python Package Guide

## What It Is

`websockets` is a focused WebSocket protocol library for Python. Use it when you need:

- an asyncio WebSocket server or client
- a synchronous threading-based client or server
- Sans-I/O protocol building blocks for framework integrations

Do not treat it as a full HTTP framework. If you need routing, request parsing, dependency injection, or mixed HTTP and WebSocket application structure, use a framework built on top of it.

## Install

Pin the package version when you need the behavior described here:

```bash
python -m pip install "websockets==16.0"
```

`websockets` has no required runtime dependencies.

If you need SOCKS proxy support for client connections, install the optional proxy dependency documented upstream:

```bash
python -m pip install "python-socks[asyncio]"
```

## Choose The Right API Surface

For new async code, prefer explicit imports from the asyncio implementation:

```python
from websockets.asyncio.client import connect
from websockets.asyncio.server import serve
```

For synchronous scripts or worker threads:

```python
from websockets.sync.client import connect
from websockets.sync.server import serve
```

Why explicit imports matter:

- `14.0` switched top-level aliases such as `websockets.connect` and `websockets.serve` to the new asyncio implementation.
- Explicit imports make it clear whether code uses the new asyncio API, the sync API, or older legacy imports.

## Asyncio Quick Start

Minimal echo server:

```python
import asyncio

from websockets.asyncio.server import serve

async def echo(websocket):
    async for message in websocket:
        await websocket.send(message)

async def main() -> None:
    async with serve(echo, "127.0.0.1", 8765):
        await asyncio.get_running_loop().create_future()

asyncio.run(main())
```

Minimal client:

```python
import asyncio

from websockets.asyncio.client import connect

async def main() -> None:
    async with connect("ws://127.0.0.1:8765") as websocket:
        await websocket.send("hello")
        reply = await websocket.recv()
        print(reply)

asyncio.run(main())
```

Core behavior:

- `connect()` is an async context manager and closes the connection automatically.
- `serve()` is an async context manager and closes active connections on shutdown.
- `async for message in websocket` is the standard receive loop.
- `recv()` raises `ConnectionClosed` subclasses when the peer closes or the connection fails.

## Reconnect Loops

The asyncio client supports automatic reconnect loops with `async for`:

```python
import asyncio
import websockets
from websockets.asyncio.client import connect

async def main() -> None:
    async for websocket in connect(
        "wss://example.com/ws",
        open_timeout=10,
        ping_interval=20,
        ping_timeout=20,
    ):
        try:
            await websocket.send("ping")
            print(await websocket.recv())
        except websockets.exceptions.ConnectionClosed:
            continue

asyncio.run(main())
```

Use this pattern for durable clients that should reconnect after network failures or temporary server outages. Override `process_exception` only if the default retry classification is wrong for your application.

## Sync API Quick Start

Use the threading API when you cannot naturally run an event loop:

```python
from websockets.sync.client import connect

def main() -> None:
    with connect("ws://127.0.0.1:8765") as websocket:
        websocket.send("hello")
        print(websocket.recv())

if __name__ == "__main__":
    main()
```

The sync API is appropriate for CLI tools, background worker threads, and simple blocking integrations. It is not a drop-in replacement for the asyncio API; pick one model and stay consistent inside a component.

## TLS, Headers, Subprotocols, And Proxies

Use `wss://` for production traffic.

Client-side configuration usually starts with `connect(...)`:

```python
import asyncio
import ssl

from websockets.asyncio.client import connect

async def main() -> None:
    ssl_context = ssl.create_default_context()

    async with connect(
        "wss://example.com/ws",
        ssl=ssl_context,
        additional_headers={"Authorization": "Bearer YOUR_TOKEN"},
        subprotocols=["chat.v1"],
        origin="https://app.example.com",
        open_timeout=10,
        proxy=None,
    ) as websocket:
        await websocket.send("hello")
        print(await websocket.recv())

asyncio.run(main())
```

Key options to know:

- `additional_headers`: custom headers such as `Authorization`
- `subprotocols`: negotiate application protocols
- `origin`: send an `Origin` header when the server checks it
- `open_timeout`: bound slow handshakes
- `ping_interval` and `ping_timeout`: heartbeat tuning
- `max_size` and `max_queue`: inbound memory and backpressure controls
- `compression=None`: disable per-message deflate if you need simpler behavior
- `proxy`: override proxy behavior; use `None` to disable automatic proxy usage

Version-sensitive proxy note:

- Since `15.0`, client connections use HTTP or SOCKS proxies automatically when the operating system or environment config says to use one.
- If upgrading older code, unexpected proxy use is a real behavior change. Set `proxy=None` when direct connections are required.

## Server Setup And Handshake Hooks

Typical server configuration:

```python
import asyncio

from websockets.asyncio.server import serve

async def handler(websocket) -> None:
    async for message in websocket:
        await websocket.send(message)

async def main() -> None:
    async with serve(
        handler,
        "0.0.0.0",
        8765,
        origins=["https://app.example.com"],
        subprotocols=["chat.v1"],
        ping_interval=20,
        ping_timeout=20,
        max_size=2**20,
        max_queue=16,
        server_header=None,
    ):
        await asyncio.get_running_loop().create_future()

asyncio.run(main())
```

Useful server-side hooks:

- `origins`: defend against Cross-Site WebSocket Hijacking by allow-listing browser origins
- `process_request`: reject or customize the HTTP opening handshake before the WebSocket upgrade
- `process_response`: customize the HTTP response generated during the handshake
- `select_subprotocol`: enforce or customize subprotocol negotiation
- `server_header=None`: remove the default server banner

If you need simple path-based dispatch, inspect `websocket.request.path` inside the handler or use the routing helpers added in `15.0`.

## Authentication

For non-browser clients, the usual pattern is an authorization header:

```python
import asyncio

from websockets.asyncio.client import connect

async def main() -> None:
    async with connect(
        "wss://example.com/ws",
        additional_headers={"Authorization": "Bearer YOUR_TOKEN"},
    ) as websocket:
        await websocket.send("hello")

asyncio.run(main())
```

For server-side HTTP Basic authentication, use the built-in helper:

```python
import asyncio

from websockets.asyncio.server import basic_auth, serve

async def handler(websocket) -> None:
    await websocket.send(f"hello {websocket.username}")

async def main() -> None:
    async with serve(
        handler,
        "127.0.0.1",
        8765,
        process_request=basic_auth(
            realm="example",
            credentials=("hello", "iloveyou"),
        ),
    ):
        await asyncio.get_running_loop().create_future()

asyncio.run(main())
```

Browser authentication tradeoffs from the upstream auth guide:

- first message after connect: simplest application-level token flow when you do not need HTTP 401 responses
- query parameter: easy but leaks into logs and URLs
- cookie: workable when the cookie domain arrangement fits your deployment
- user info in the URI: only for machine-to-machine cases; browser support is poor

## Keepalive, Timeouts, And Memory Controls

The keepalive guide documents a heartbeat ping every 20 seconds by default and expects a pong within 20 seconds.

Important controls:

- `ping_interval=None`: disable keepalive entirely
- `ping_timeout=None`: keep the trickle of ping traffic but disable heartbeat timeouts
- `max_size`: maximum incoming message size
- `max_queue`: maximum buffered incoming frames or messages before backpressure applies

Version-sensitive note:

- Since `15.0`, the threading implementation also enables keepalive.
- Code written for `14.x` may assume that only asyncio connections sent heartbeat pings.

The library also supports process-wide environment variables for advanced tuning, including:

- `WEBSOCKETS_BACKOFF_*` for reconnect backoff behavior
- `WEBSOCKETS_MAX_LOG_SIZE`
- `WEBSOCKETS_MAX_BODY_SIZE`
- `WEBSOCKETS_MAX_LINE_LENGTH`
- `WEBSOCKETS_MAX_NUM_HEADERS`
- `WEBSOCKETS_MAX_REDIRECTS`
- `WEBSOCKETS_USER_AGENT`
- `WEBSOCKETS_SERVER`

Use these only when you need global policy. Prefer explicit per-connection arguments in application code.

## Common Pitfalls

- Use explicit imports from `websockets.asyncio.*` or `websockets.sync.*` when writing new code. Top-level aliases are easy to misread during migrations.
- If older examples show a handler signature like `handler(websocket, path)`, update it. In current code, inspect `websocket.request.path` instead of relying on a separate `path` argument.
- If older examples use `extra_headers=`, update them to `additional_headers=` on the new asyncio client.
- Browser clients do not let you set arbitrary WebSocket handshake headers. Use cookies, query parameters, or an application message instead of assuming `Authorization` is always available from browser JavaScript.
- Automatic proxy use in `15.x+` can change connection behavior in CI, corporate networks, or container environments. Set `proxy=None` when you need a direct socket.
- `websockets` is only the protocol layer. Do not expect built-in HTTP routing, REST helpers, sessions, or middleware patterns.

## Upgrade Notes For Older Code

If you are upgrading code from `13.x` or earlier, check these changes first:

- `14.0`: top-level asyncio aliases now point at the new asyncio implementation
- `14.0`: several deprecated aliases and legacy argument names were removed from the default import surface
- `15.0`: routing helpers were added to the asyncio server
- `15.0`: threading connections gained keepalive
- `15.0`: client proxy auto-detection became the default
- `16.0`: current package version requires Python `3.10+`

For migration-heavy work, read the official upgrade guide before copying older examples from blogs or issue threads.

## Official Sources

- Documentation root: https://websockets.readthedocs.io/en/stable/
- Quick start: https://websockets.readthedocs.io/en/stable/howto/quickstart.html
- Asyncio client reference: https://websockets.readthedocs.io/en/stable/reference/asyncio/client.html
- Asyncio server reference: https://websockets.readthedocs.io/en/stable/reference/asyncio/server.html
- Sync client reference: https://websockets.readthedocs.io/en/stable/reference/sync/client.html
- Authentication guide: https://websockets.readthedocs.io/en/stable/topics/authentication.html
- Keepalive guide: https://websockets.readthedocs.io/en/stable/topics/keepalive.html
- Environment variables: https://websockets.readthedocs.io/en/stable/reference/variables.html
- Upgrade guide: https://websockets.readthedocs.io/en/stable/howto/upgrade.html
- Changelog: https://websockets.readthedocs.io/en/stable/project/changelog.html
- PyPI package page: https://pypi.org/project/websockets/

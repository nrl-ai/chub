---
name: package
description: "aiohttp package guide for Python - async HTTP client/server, web apps, and websockets"
metadata:
  languages: "python"
  versions: "3.13.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aiohttp,python,asyncio,http,web,websocket,client,server"
---

# aiohttp Python Package Guide

## Golden Rule

Use `aiohttp` when a Python project needs async HTTP client calls, an asyncio-native web server, or both. Keep `ClientSession` and `web.Application` lifecycle-bound to the active event loop. Do not create sessions at module import time or per request.

## Install

```bash
pip install aiohttp==3.13.3
```

Optional speedups from the official docs:

```bash
pip install "aiohttp[speedups]"
```

That bundle pulls in extras such as `aiodns` for faster DNS resolution and Brotli support for compressed client responses.

## Choose The Surface

- Use `aiohttp.ClientSession` for outbound async HTTP requests.
- Use `aiohttp.web` for an asyncio-native HTTP server.
- Use `session.ws_connect()` or `web.WebSocketResponse()` for WebSockets.
- Use one long-lived session per app or per upstream service, not one per request.

## Client: Minimal Request Flow

```python
import asyncio
import aiohttp

async def main() -> None:
    async with aiohttp.ClientSession() as session:
        async with session.get("https://httpbin.org/get") as resp:
            resp.raise_for_status()
            data = await resp.json()
            print(resp.status, data["url"])

asyncio.run(main())
```

## Client: Recommended Session Setup

Use a shared session with explicit timeouts, headers, and connector limits.

```python
import asyncio
import ssl

import aiohttp

async def main() -> None:
    timeout = aiohttp.ClientTimeout(total=60, connect=10, sock_read=30)
    ssl_context = ssl.create_default_context()
    connector = aiohttp.TCPConnector(limit=100, limit_per_host=20, ssl=ssl_context)

    async with aiohttp.ClientSession(
        base_url="https://api.example.com/",
        headers={"Accept": "application/json"},
        raise_for_status=True,
        timeout=timeout,
        connector=connector,
    ) as session:
        async with session.get("v1/items", params={"limit": 50}) as resp:
            items = await resp.json()
            print(items)

asyncio.run(main())
```

Notes:

- If `base_url` includes a path, it must end with `/`.
- Relative request paths without a leading slash join onto `base_url` as expected.
- Default total timeout is 300 seconds, with a 30 second socket connect timeout.
- `raise_for_status=True` on the session is a good default for API clients.

## Client: JSON, Streaming, Uploads

JSON request/response:

```python
async with session.post("/v1/jobs", json={"name": "build"}) as resp:
    job = await resp.json()
```

Large downloads should stream instead of calling `text()`, `read()`, or `json()`:

```python
async with session.get("/large-export") as resp:
    resp.raise_for_status()
    with open("export.bin", "wb") as f:
        async for chunk in resp.content.iter_chunked(64 * 1024):
            f.write(chunk)
```

Large uploads can stream file objects or async generators:

```python
with open("report.csv", "rb") as f:
    async with session.post("/upload", data=f) as resp:
        resp.raise_for_status()
```

Pitfall: once you consume `resp.content` manually, do not call `resp.text()`, `resp.read()`, or `resp.json()` afterward.

## Client: Auth, Proxies, Cookies, SSL

Basic auth:

```python
from aiohttp import BasicAuth

async with aiohttp.ClientSession(
    auth=BasicAuth(login="user", password="pass")
) as session:
    async with session.get("https://example.com/protected") as resp:
        print(resp.status)
```

Digest auth in `3.13.3` uses client middleware:

```python
from aiohttp import ClientSession, DigestAuthMiddleware

digest_auth = DigestAuthMiddleware(login="user", password="pass")

async with ClientSession(middlewares=(digest_auth,)) as session:
    async with session.get("https://example.com/protected") as resp:
        print(await resp.text())
```

Proxy configuration:

```python
proxy_auth = aiohttp.BasicAuth("proxy-user", "proxy-pass")

async with aiohttp.ClientSession(
    proxy="http://proxy.internal:8080",
    proxy_auth=proxy_auth,
) as session:
    async with session.get("http://python.org") as resp:
        print(resp.status)
```

Environment proxy variables are ignored unless you opt in:

```python
async with aiohttp.ClientSession(trust_env=True) as session:
    ...
```

Cookie control:

```python
jar = aiohttp.DummyCookieJar()
async with aiohttp.ClientSession(cookie_jar=jar) as session:
    ...
```

SSL guidance:

- Default HTTPS verification is strict. Keep it on in production.
- Prefer a custom `ssl.SSLContext` for private CAs or client certs.
- `ssl=False` disables certificate checks and should be limited to controlled local/dev cases.

## Client WebSockets

```python
async with aiohttp.ClientSession() as session:
    async with session.ws_connect("wss://example.com/ws") as ws:
        await ws.send_json({"type": "hello"})
        async for msg in ws:
            if msg.type == aiohttp.WSMsgType.TEXT:
                print(msg.data)
            elif msg.type == aiohttp.WSMsgType.ERROR:
                break
```

Keep one reader task for a websocket. Multiple tasks can send, but reads should stay serialized through the handler task or one dedicated reader.

## Server: Minimal App

```python
from aiohttp import web

async def hello(request: web.Request) -> web.Response:
    return web.json_response({"message": "hello"})

app = web.Application()
app.add_routes([web.get("/", hello)])

if __name__ == "__main__":
    web.run_app(app)
```

Alternative route style:

```python
from aiohttp import web

routes = web.RouteTableDef()

@routes.get("/users/{user_id}", name="user-detail")
async def get_user(request: web.Request) -> web.Response:
    return web.json_response({"user_id": request.match_info["user_id"]})

app = web.Application()
app.add_routes(routes)
```

Named routes can be reversed with `request.app.router["user-detail"].url_for(...)`.

## Server: Shared State And Lifecycle

Use `AppKey` instead of string keys when storing typed app state:

```python
from aiohttp import ClientSession, web

http_client_key = web.AppKey("http_client", ClientSession)
```

Prefer `cleanup_ctx` for paired startup/teardown resources:

```python
import aiohttp
from aiohttp import web

http_client_key = web.AppKey("http_client", aiohttp.ClientSession)

async def http_client_ctx(app: web.Application):
    app[http_client_key] = aiohttp.ClientSession(
        timeout=aiohttp.ClientTimeout(total=30)
    )
    yield
    await app[http_client_key].close()

async def health(request: web.Request) -> web.Response:
    session = request.app[http_client_key]
    async with session.get("https://httpbin.org/status/200") as resp:
        return web.json_response({"upstream": resp.status})

app = web.Application()
app.cleanup_ctx.append(http_client_ctx)
app.add_routes([web.get("/health", health)])
web.run_app(app)
```

Why `cleanup_ctx` matters:

- It pairs setup and teardown in one async generator.
- Cleanup only runs for resources that actually initialized.
- It is safer than separate `on_startup` and `on_cleanup` handlers for dependent resources.

## Server: Middleware And Background Tasks

Middleware shape:

```python
from aiohttp import web

@web.middleware
async def auth_middleware(request: web.Request, handler):
    token = request.headers.get("Authorization")
    if token != "Bearer dev-token":
        raise web.HTTPUnauthorized()
    return await handler(request)

app = web.Application(middlewares=[auth_middleware])
```

The second middleware parameter should be named `handler` exactly.

For long-lived background tasks, attach them to app lifecycle:

```python
import asyncio
import contextlib
from aiohttp import web

listener_key = web.AppKey("listener", asyncio.Task)

async def background(app: web.Application):
    app[listener_key] = asyncio.create_task(asyncio.sleep(3600))
    yield
    app[listener_key].cancel()
    with contextlib.suppress(asyncio.CancelledError):
        await app[listener_key]

app = web.Application()
app.cleanup_ctx.append(background)
```

If you need asynchronous startup without blocking `web.run_app()`, use `web.AppRunner` and `web.TCPSite`.

## Server WebSockets

```python
from aiohttp import web

async def websocket_handler(request: web.Request) -> web.WebSocketResponse:
    ws = web.WebSocketResponse()
    await ws.prepare(request)

    async for msg in ws:
        if msg.type == web.WSMsgType.TEXT:
            await ws.send_str(f"echo:{msg.data}")
        elif msg.type == web.WSMsgType.ERROR:
            break

    return ws
```

Rules that matter:

- Only one task should read from a given `WebSocketResponse`.
- Other tasks may send on that websocket.
- For graceful shutdown, track open websockets in app state and close them from an `on_shutdown` handler.

## Common Pitfalls

- Creating `ClientSession()` at import time or as a class variable can bind it to the wrong event loop and cause hangs.
- Creating a new session per request defeats connection pooling and adds overhead.
- Forgetting to close sessions leaves unclosed connector warnings and leaked sockets.
- Assuming proxy env vars are honored by default. They are not unless `trust_env=True`.
- Using `ssl=False` as a general fix for TLS problems. Install the right CA or configure an `SSLContext` instead.
- Trusting `Forwarded` or `X-Forwarded-*` headers automatically on the server side. `aiohttp.web` ignores them by default for security; only apply them in trusted reverse-proxy middleware.
- Reusing the same `web.Response` object across requests is invalid. Response objects are one-request objects.
- Reading from the same websocket in parallel from multiple tasks is forbidden.

## Version-Sensitive Notes For 3.13.3

- `3.13.3` is the current stable docs version and the PyPI latest release as of `2026-03-11`.
- The `3.13.3` changelog states this release fixes several vulnerabilities and recommends upgrading promptly.
- Client middlewares on `ClientSession` are available in `3.12+`. Older examples that rely on them will not work on earlier `aiohttp` releases.
- `ClientSession(base_url=...)` gained absolute-URL override support in `3.12`.
- `ssl_shutdown_timeout` is deprecated and scheduled for removal in `aiohttp 4.0`; avoid building new code around it.

## Official Sources

- Docs root: https://docs.aiohttp.org/en/stable/
- Client quickstart: https://docs.aiohttp.org/en/stable/client_quickstart.html
- Advanced client usage: https://docs.aiohttp.org/en/stable/client_advanced.html
- Client reference: https://docs.aiohttp.org/en/stable/client_reference.html
- Web server quickstart: https://docs.aiohttp.org/en/stable/web_quickstart.html
- Web server advanced: https://docs.aiohttp.org/en/stable/web_advanced.html
- FAQ: https://docs.aiohttp.org/en/stable/faq.html
- Changelog: https://docs.aiohttp.org/en/stable/changes.html
- PyPI: https://pypi.org/project/aiohttp/

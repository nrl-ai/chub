---
name: package
description: "Tornado async web framework and networking library for Python HTTP services, clients, and WebSockets"
metadata:
  languages: "python"
  versions: "6.5.5"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tornado,python,asyncio,web,http,websocket,server"
---

# Tornado Python Package Guide

## Golden Rule

Use Tornado as an `asyncio`-based networking framework and keep request handlers non-blocking.

- Start new apps with `asyncio.run(...)`.
- Use `async def` handlers and `AsyncHTTPClient` for network I/O.
- Offload blocking work to threads or worker processes.
- Re-check release notes before copying older `IOLoop.start()` or callback-style examples.

## Installation

```bash
pip install "tornado==6.5.5"
```

```bash
poetry add "tornado==6.5.5"
```

```bash
uv add tornado==6.5.5
```

Verify the installed version:

```bash
python -c "import tornado; print(tornado.version)"
```

## Initialize And Run

Tornado 6.x runs on top of `asyncio`. A minimal app looks like this:

```python
import asyncio

import tornado.escape
import tornado.httpserver
import tornado.web

class HealthHandler(tornado.web.RequestHandler):
    async def get(self) -> None:
        self.write({"ok": True})

class EchoHandler(tornado.web.RequestHandler):
    async def post(self) -> None:
        payload = tornado.escape.json_decode(self.request.body or b"{}")
        self.set_status(201)
        self.write({"received": payload})

def make_app() -> tornado.web.Application:
    return tornado.web.Application(
        [
            (r"/healthz", HealthHandler),
            (r"/echo", EchoHandler),
        ],
        cookie_secret="replace-me",
        xsrf_cookies=True,
        debug=False,
    )

async def main() -> None:
    server = tornado.httpserver.HTTPServer(make_app())
    server.listen(8888, address="127.0.0.1")
    await asyncio.Event().wait()

if __name__ == "__main__":
    asyncio.run(main())
```

Practical rules:

- `Application.listen(...)` is fine for small apps; use `HTTPServer(...)` when you need SSL, `xheaders`, or explicit socket control.
- `debug=True` enables autoreload and extra checks, but it is a development setting.
- Keep one `Application` per process and initialize shared clients during startup, not inside every request.

## Core Request Handling

Prefer `async def` handlers whenever a request performs I/O:

```python
import tornado.escape
import tornado.web

class UserHandler(tornado.web.RequestHandler):
    async def get(self, user_id: str) -> None:
        verbose = self.get_query_argument("verbose", "0") == "1"
        self.write({"user_id": user_id, "verbose": verbose})

    async def post(self, user_id: str) -> None:
        payload = tornado.escape.json_decode(self.request.body or b"{}")
        self.set_status(201)
        self.write({"user_id": user_id, "payload": payload})

app = tornado.web.Application([
    (r"/users/([^/]+)", UserHandler),
])
```

Use the right helper for the right input source:

- `get_query_argument(...)` for query string values only
- `get_body_argument(...)` for form fields in the request body
- `get_argument(...)` only when it is acceptable to merge query and body values
- `tornado.escape.json_decode(self.request.body)` for JSON request bodies

`self.write(dict_or_list)` serializes JSON automatically and sets the content type.

## Outbound HTTP

Use `AsyncHTTPClient` inside handlers and background tasks.

```python
import tornado.escape
import tornado.httpclient
import tornado.web

class UpstreamHandler(tornado.web.RequestHandler):
    async def get(self) -> None:
        client = tornado.httpclient.AsyncHTTPClient()
        response = await client.fetch(
            "https://httpbin.org/json",
            request_timeout=10.0,
        )
        data = tornado.escape.json_decode(response.body)
        self.write({"title": data["slideshow"]["title"]})
```

Notes:

- `AsyncHTTPClient.fetch(...)` raises `HTTPClientError` on non-2xx responses unless you pass `raise_error=False`.
- `HTTPClient` is synchronous and should not run on the main event-loop thread.
- Reuse clients for repeated upstream calls instead of constructing a new one for every operation-heavy code path.

## WebSockets

Use `tornado.websocket.WebSocketHandler` for browser or service-to-service sockets:

```python
import tornado.websocket

class EchoSocket(tornado.websocket.WebSocketHandler):
    def check_origin(self, origin: str) -> bool:
        return origin == "https://app.example.com"

    def open(self) -> None:
        self.write_message("connected")

    def on_message(self, message: str) -> None:
        self.write_message(f"echo: {message}")
```

```python
app = tornado.web.Application(
    [(r"/ws", EchoSocket)],
    websocket_ping_interval=30,
    websocket_ping_timeout=30,
)
```

Guidance:

- The default `check_origin` rejects cross-origin browser connections. Override it narrowly.
- Keep ping settings explicit when connections sit behind proxies or load balancers.
- If you still have callback-style `websocket_connect(..., callback=...)` code, treat it as deprecated and migrate to `await tornado.websocket.websocket_connect(...)`.

## Auth And Security Configuration

Signed cookies plus `@authenticated` are the standard built-in pattern:

```python
import tornado.web

class BaseHandler(tornado.web.RequestHandler):
    def get_current_user(self) -> str | None:
        value = self.get_signed_cookie("user")
        return value.decode() if value else None

class LoginHandler(BaseHandler):
    async def post(self) -> None:
        username = self.get_body_argument("username")
        self.set_signed_cookie(
            "user",
            username,
            secure=True,
            httponly=True,
            samesite="Lax",
        )
        self.write({"ok": True})

class AccountHandler(BaseHandler):
    @tornado.web.authenticated
    async def get(self) -> None:
        self.write({"user": self.current_user})
```

```python
app = tornado.web.Application(
    [
        (r"/login", LoginHandler),
        (r"/account", AccountHandler),
    ],
    cookie_secret="long-random-secret",
    login_url="/login",
    xsrf_cookies=True,
    xsrf_cookie_kwargs={"secure": True, "samesite": "Strict"},
)
```

Important behavior:

- `cookie_secret` is required for signed cookies.
- `login_url` is required when using `@tornado.web.authenticated`.
- For `GET` and `HEAD`, `@authenticated` redirects to `login_url`; for other methods, it returns `403`.
- Keep `xsrf_cookies=True` for forms or browser-driven mutation endpoints.

### Multipart form limits in 6.5.5

Tornado `6.5.5` adds process-wide multipart limits to reduce risk from requests with huge numbers of files or parts. If your app legitimately accepts large multipart requests, raise the limit deliberately:

```python
from tornado.httputil import (
    ParseBodyConfig,
    ParseMultipartConfig,
    set_parse_body_config,
)

set_parse_body_config(
    ParseBodyConfig(
        multipart=ParseMultipartConfig(max_parts=500)
    )
)
```

Treat this as a global server setting and keep the limit as low as your workload allows.

## Blocking Code And Threads

Tornado request handling runs on a single-threaded event loop. Blocking work must move off that thread.

```python
import asyncio

import tornado.web

def blocking_lookup(user_id: str) -> dict[str, str]:
    return {"user_id": user_id, "source": "thread"}

class ReportHandler(tornado.web.RequestHandler):
    async def get(self, user_id: str) -> None:
        result = await asyncio.to_thread(blocking_lookup, user_id)
        self.write(result)
```

Practical rules:

- Use native async clients for network I/O whenever they exist.
- Use `asyncio.to_thread(...)` or an executor for blocking libraries.
- Do not mutate Tornado request or application objects from worker threads.
- If another thread needs to signal the IOLoop, use `IOLoop.add_callback(...)`.

## Deployment Notes

For production:

- run Tornado behind a reverse proxy such as nginx or Envoy
- enable `xheaders=True` only when all traffic comes through a trusted proxy layer
- use one process per CPU core for I/O-heavy workloads
- prefer Tornado's documented multi-process socket pattern instead of ad hoc forking

Typical multi-process pattern:

```python
import asyncio

import tornado.httpserver
import tornado.netutil
import tornado.process

sockets = tornado.netutil.bind_sockets(8888)
tornado.process.fork_processes(0)

server = tornado.httpserver.HTTPServer(make_app())
server.add_sockets(sockets)

asyncio.run(asyncio.Event().wait())
```

Notes:

- `fork_processes(...)` is not available on Windows.
- Do not create event-loop-bound objects before forking worker processes.
- If you are already using a process manager or container platform, compare this pattern with `reuse_port=True` and your platform's normal process model.

## Common Pitfalls

- Do not call `requests`, synchronous database drivers, or filesystem-heavy code directly in handlers.
- Do not parse JSON request bodies with `get_argument(...)`; decode `self.request.body`.
- Do not return `True` from `check_origin` blindly for WebSocket handlers.
- Do not trust `xheaders=True` outside a controlled proxy boundary.
- Do not assume old Tornado blog posts match 6.5.x startup, WebSocket, or multipart parsing behavior.

## Version-Sensitive Notes For 6.5.5

- `6.5.0` raised the minimum supported Python version to `3.9` and added Python `3.14` support.
- `6.5.0` deprecated `websocket_connect(..., callback=...)`; use `await` instead.
- `6.5.2` fixed `websocket_ping_interval` and `websocket_ping_timeout` behavior in some reconnection cases.
- `6.5.3` fixed a security issue around repeated HTTP headers with mixed `Content-Length` and `Transfer-Encoding`.
- `6.5.5` fixed more security issues in `set_status`, `set_cookie`, and multipart parsing, and added `ParseMultipartConfig(max_parts=...)`.

If you upgrade from `6.4.x`, review the 6.5 release notes before reusing old auth, WebSocket, or request-parsing code unchanged.

## Official Sources

- Stable docs root: https://www.tornadoweb.org/en/stable/
- User guide: https://www.tornadoweb.org/en/stable/guide.html
- Async and blocking-code guide: https://www.tornadoweb.org/en/stable/guide/async.html
- Running and deployment guide: https://www.tornadoweb.org/en/stable/guide/running.html
- Web framework reference: https://www.tornadoweb.org/en/stable/web.html
- HTTP client reference: https://www.tornadoweb.org/en/stable/httpclient.html
- WebSocket reference: https://www.tornadoweb.org/en/stable/websocket.html
- HTTP utility reference: https://www.tornadoweb.org/en/stable/httputil.html
- Release notes index: https://www.tornadoweb.org/en/stable/releases.html
- 6.5.0 release notes: https://www.tornadoweb.org/en/stable/releases/v6.5.0.html
- 6.5.2 release notes: https://www.tornadoweb.org/en/stable/releases/v6.5.2.html
- 6.5.3 release notes: https://www.tornadoweb.org/en/stable/releases/v6.5.3.html
- 6.5.5 release notes: https://www.tornadoweb.org/en/stable/releases/v6.5.5.html
- PyPI package page: https://pypi.org/project/tornado/6.5.5/

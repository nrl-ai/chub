---
name: package
description: "Starlette ASGI framework for Python with routing, middleware, websockets, auth, and testing"
metadata:
  languages: "python"
  versions: "0.52.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "starlette,python,asgi,web,async,websockets,middleware,testing"
---

# Starlette Python Package Guide

## Golden Rule

Use Starlette for ASGI application structure, but run it with an ASGI server such as `uvicorn`. `starlette` gives you the app, routing, middleware, request/response, auth, and test primitives. It is not the server process by itself.

## Installation

### Base install

```bash
pip install starlette==0.52.1
pip install "uvicorn[standard]"
```

### With common optional dependencies

```bash
pip install "starlette[full]==0.52.1"
pip install "uvicorn[standard]"
```

`starlette[full]` is the safest default when you expect forms, templates, sessions, or other commonly used extras.

## Minimal App

```python
from starlette.applications import Starlette
from starlette.responses import JSONResponse
from starlette.routing import Route

async def homepage(request):
    return JSONResponse({"ok": True, "path": request.url.path})

app = Starlette(
    debug=False,
    routes=[Route("/", homepage)],
)
```

Run it with:

```bash
uvicorn app:app --reload
```

## Recommended App Setup

Use `lifespan=` for startup and shutdown work. In current Starlette, this is the preferred style over `on_startup` and `on_shutdown`.

```python
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import TypedDict

import httpx
from starlette.applications import Starlette
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.routing import Route

class AppState(TypedDict):
    http_client: httpx.AsyncClient

@asynccontextmanager
async def lifespan(app: Starlette) -> AsyncIterator[AppState]:
    async with httpx.AsyncClient(timeout=10.0) as client:
        yield {"http_client": client}

async def healthcheck(request: Request[AppState]):
    client = request.state["http_client"]
    upstream = await client.get("https://example.com/health")
    return JSONResponse({"status": upstream.status_code})

app = Starlette(
    routes=[Route("/health", healthcheck)],
    lifespan=lifespan,
)
```

## Core Usage

### Routing

Starlette’s routing is explicit and lightweight:

```python
from starlette.responses import PlainTextResponse
from starlette.routing import Route

async def user_detail(request):
    user_id = request.path_params["user_id"]
    return PlainTextResponse(f"user={user_id}")

routes = [
    Route("/users/{user_id:int}", user_detail, methods=["GET"]),
]
```

Use named routes plus `request.url_for(...)` when you need reverse lookups instead of hard-coding paths.

### Requests and responses

`Request` exposes parsed URL parts, headers, query params, cookies, body readers, and `request.state`.

```python
from starlette.requests import Request
from starlette.responses import JSONResponse

async def create_item(request: Request):
    payload = await request.json()
    return JSONResponse(
        {
            "item": payload,
            "query": dict(request.query_params),
            "client": request.client.host if request.client else None,
        },
        status_code=201,
    )
```

Useful response classes:

- `JSONResponse` for API payloads
- `PlainTextResponse` and `HTMLResponse` for simple text or HTML
- `RedirectResponse` for redirects
- `StreamingResponse` for async generators and large streams
- `FileResponse` for file downloads and range requests

### Multipart forms and uploads

Starlette’s `request.form()` supports limits for files, fields, and part size. Keep them explicit when handling uploads.

```python
async def upload(request):
    async with request.form(
        max_files=20,
        max_fields=50,
        max_part_size=4 * 1024 * 1024,
    ) as form:
        uploaded = form["file"]
        contents = await uploaded.read()
    return JSONResponse({"bytes": len(contents)})
```

### WebSockets

```python
from starlette.applications import Starlette
from starlette.routing import WebSocketRoute
from starlette.websockets import WebSocket, WebSocketDisconnect

async def ws_endpoint(websocket: WebSocket):
    await websocket.accept()
    try:
        async for message in websocket.iter_text():
            await websocket.send_json({"echo": message})
    except WebSocketDisconnect:
        pass

app = Starlette(routes=[WebSocketRoute("/ws", ws_endpoint)])
```

Use `send_json(..., mode="binary")` and `receive_json(mode="binary")` only if you intentionally want JSON over binary frames. Text frames are the default.

## Configuration and Auth

### Environment-based config

Starlette ships a `Config` helper for environment variables and `.env` files.

```python
from starlette.applications import Starlette
from starlette.config import Config
from starlette.datastructures import CommaSeparatedStrings, Secret

config = Config(".env")

DEBUG = config("DEBUG", cast=bool, default=False)
SECRET_KEY = config("SECRET_KEY", cast=Secret)
ALLOWED_HOSTS = config("ALLOWED_HOSTS", cast=CommaSeparatedStrings, default="")

app = Starlette(debug=DEBUG)
```

Read order is: environment variable, then `.env`, then default. Keep secrets out of source control.

### Sessions, host checks, CORS

Common middleware choices:

- `SessionMiddleware(secret_key=...)` for signed cookie-backed sessions
- `TrustedHostMiddleware(allowed_hosts=[...])` to reject unexpected host headers
- `HTTPSRedirectMiddleware()` to force HTTPS
- `CORSMiddleware(...)` for browser API access

When `allow_credentials=True` in `CORSMiddleware`, do not use `["*"]` for origins, methods, or headers. Use explicit lists.

For APIs that must return CORS headers even on unhandled 500 responses, wrap the entire application:

```python
from starlette.applications import Starlette
from starlette.middleware.cors import CORSMiddleware

inner = Starlette(routes=routes, middleware=middleware, lifespan=lifespan)
app = CORSMiddleware(
    app=inner,
    allow_origins=["https://app.example.com"],
    allow_methods=["GET", "POST"],
    allow_headers=["Authorization", "Content-Type"],
    allow_credentials=True,
)
```

### Authentication

Install `AuthenticationMiddleware` with a backend. Then use `request.user`, `request.auth`, and the `@requires(...)` decorator.

```python
import base64
import binascii

from starlette.applications import Starlette
from starlette.authentication import (
    AuthCredentials,
    AuthenticationBackend,
    AuthenticationError,
    SimpleUser,
    requires,
)
from starlette.middleware import Middleware
from starlette.middleware.authentication import AuthenticationMiddleware
from starlette.responses import PlainTextResponse
from starlette.routing import Route

class BasicAuthBackend(AuthenticationBackend):
    async def authenticate(self, conn):
        if "Authorization" not in conn.headers:
            return None

        try:
            scheme, credentials = conn.headers["Authorization"].split()
            if scheme.lower() != "basic":
                return None
            decoded = base64.b64decode(credentials).decode("ascii")
        except (ValueError, UnicodeDecodeError, binascii.Error) as exc:
            raise AuthenticationError("Invalid basic auth credentials") from exc

        username, _, _password = decoded.partition(":")
        return AuthCredentials(["authenticated"]), SimpleUser(username)

@requires("authenticated")
async def dashboard(request):
    return PlainTextResponse(f"hello {request.user.display_name}")

app = Starlette(
    routes=[Route("/dashboard", dashboard)],
    middleware=[Middleware(AuthenticationMiddleware, backend=BasicAuthBackend())],
)
```

If your auth backend raises `AuthenticationError`, provide `on_error=...` to return a consistent JSON or redirect response.

## Testing

Use `TestClient` for sync-style tests and keep it in a `with` block when your app has lifespan logic.

```python
from starlette.testclient import TestClient

def test_homepage():
    with TestClient(app, raise_server_exceptions=True) as client:
        response = client.get("/")
        assert response.status_code == 200
```

Important details:

- `TestClient(app)` uses `httpx` semantics.
- Lifespan handlers only run when `TestClient` is used as a context manager.
- `backend="trio"` is supported.
- WebSocket tests must use `with client.websocket_connect("/ws") as websocket: ...`.
- `websocket_connect()` does not support `params=`; put query params directly in the URL.

## Common Pitfalls

- Starlette is not the process manager or server. Run it under `uvicorn`, `hypercorn`, or another ASGI server.
- Prefer `lifespan=` for startup and shutdown. Do not mix it with older startup/shutdown hooks in the same app.
- `BaseHTTPMiddleware` has known `ContextVar` propagation limitations. If you depend on contextvars or need lower-level control, write pure ASGI middleware instead.
- If you need CORS headers on error responses, wrap the whole app with `CORSMiddleware`, not only inner middleware lists.
- Upload parsing can become a CPU and memory sink if you leave limits too broad. Set `max_files`, `max_fields`, and `max_part_size`.
- For tests that inspect 500 responses, set `raise_server_exceptions=False`.
- `request.state` is for app/request-scoped state, not global mutable shared state across workers.

## Version-Sensitive Notes

- `0.52.1` is the current PyPI release and includes a fix to only use `typing_extensions` on older Python versions.
- `0.52.0` added dictionary-style typed access for lifespan state, so `request.state["http_client"]` is now a first-class pattern.
- `0.51.0` added `allow_private_network` to `CORSMiddleware`.
- `0.50.0` dropped Python 3.9 support. For `0.52.1`, require Python `>=3.10`.
- `0.49.1` fixed a security issue in `FileResponse` range header parsing. Do not copy older examples that pin earlier vulnerable releases.
- `0.44.0` added `max_part_size` to `Request.form()`. Older code snippets may omit it.

## Official Sources

- Docs root: https://starlette.dev/
- Applications: https://www.starlette.dev/applications/
- Routing: https://www.starlette.dev/routing/
- Requests: https://www.starlette.dev/requests/
- Responses: https://www.starlette.dev/responses/
- Middleware: https://www.starlette.dev/middleware/
- Authentication: https://www.starlette.dev/authentication/
- Lifespan: https://www.starlette.dev/lifespan/
- TestClient: https://www.starlette.dev/testclient/
- Release notes: https://www.starlette.dev/release-notes/
- PyPI package page: https://pypi.org/project/starlette/

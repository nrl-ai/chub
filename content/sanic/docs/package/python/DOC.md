---
name: package
description: "Sanic package guide for Python async web servers, APIs, and websockets"
metadata:
  languages: "python"
  versions: "25.12.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "sanic,python,web,async,asgi,http,websocket"
---

# Sanic Python Package Guide

## What It Is

Sanic is an async Python web framework with its own production server. It is a good fit for HTTP APIs, websocket endpoints, background task orchestration, and ASGI-compatible deployment.

This entry is scoped to package version `25.12.0`. Sanic `25.12` is the current LTS line, and it raises the Python floor to `3.10+`.

## Install

```bash
pip install sanic==25.12.0
```

```bash
uv add sanic==25.12.0
```

```bash
poetry add sanic==25.12.0
```

Sanic will use `uvloop` and `ujson` when available. If you need to build without them, set the install-time flags first:

```bash
export SANIC_NO_UVLOOP=true
export SANIC_NO_UJSON=true
pip install --no-binary :all: sanic==25.12.0
```

Optional companion packages:

- `pip install sanic-testing` for the official test client
- `pip install sanic-ext` if you need extension features like validation, OpenAPI, or dependency injection

## Minimal App

```python
from sanic import Sanic
from sanic.response import json

app = Sanic("myapp")

@app.get("/health")
async def health(_request):
    return json({"ok": True})
```

Run it with the Sanic CLI:

```bash
sanic server:app --host=0.0.0.0 --port=8000 --reload
```

Sanic's own server is the default deployment path the upstream docs recommend in most cases.

## App Factory

Use a factory when configuration or dependency setup depends on runtime state.

```python
from sanic import Sanic
from sanic.response import json

def create_app() -> Sanic:
    app = Sanic("myapp")

    @app.get("/health")
    async def health(_request):
        return json({"ok": True})

    return app
```

```bash
sanic server:create_app --factory --reload
```

## Core Request And Response Usage

```python
from sanic import Sanic
from sanic.response import html, json, redirect, text

app = Sanic("api")

@app.get("/items/<item_id:int>")
async def get_item(request, item_id: int):
    expand = request.args.get("expand")
    return json({"item_id": item_id, "expand": expand})

@app.post("/items")
async def create_item(request):
    payload = request.json or {}
    if "name" not in payload:
        return json({"error": "name is required"}, status=400)
    return json({"created": payload["name"]}, status=201)

@app.post("/upload")
async def upload(request):
    file = request.files.get("file")
    if not file:
        return text("missing file", status=400)
    return json({"filename": file.name, "content_type": file.type})

@app.get("/docs")
async def docs(_request):
    return redirect("https://sanic.dev/")

@app.get("/status")
async def status(_request):
    return html("<strong>ok</strong>")
```

Request APIs you will use most:

- `request.json` for parsed JSON bodies
- `request.body` for raw bytes
- `request.form` for form fields
- `request.files` for multipart file uploads
- `request.args` for query params
- `request.ctx` for per-request state

`request.args`, `request.form`, and `request.files` are multi-value containers. Use `.getlist()` when repeated keys matter.

## Shared State And Lifecycle

Use `app.ctx` for long-lived shared objects. Use `request.ctx` for request-local state.

```python
from sanic import Sanic
from sanic.response import json

app = Sanic("svc")

@app.before_server_start
async def open_clients(app):
    app.ctx.http = await make_http_client()
    app.ctx.db = await make_db_pool()

@app.after_server_stop
async def close_clients(app):
    await app.ctx.http.aclose()
    await app.ctx.db.close()

@app.get("/users/<user_id:int>")
async def get_user(request, user_id: int):
    user = await request.app.ctx.db.fetch_user(user_id)
    return json(user)
```

Keep database pools, caches, and shared HTTP clients on `app.ctx`. Avoid module-level mutable globals when running multiple workers.

## Middleware And Auth Pattern

Sanic does not ship a full authentication framework. The common pattern is:

1. read auth data from the request
2. validate it in middleware or a decorator
3. store the resolved principal on `request.ctx`

`request.token` extracts `Authorization: Bearer ...` and `Authorization: Token ...` headers.

```python
from sanic import Sanic
from sanic.response import json

app = Sanic("authapp")

@app.middleware("request")
async def load_user(request):
    token = request.token
    request.ctx.user = await lookup_user_from_token(token) if token else None
    if token and not request.ctx.user:
        return json({"error": "unauthorized"}, status=401)

@app.get("/me")
async def me(request):
    if not request.ctx.user:
        return json({"error": "unauthorized"}, status=401)
    return json({"id": request.ctx.user.id})
```

If request middleware returns an `HTTPResponse`, Sanic stops processing and the handler does not run.

## Configuration

Sanic configuration lives on `app.config`. Use uppercase keys.

```python
from sanic import Sanic

app = Sanic("cfg")
app.config.REQUEST_TIMEOUT = 30
app.config.RESPONSE_TIMEOUT = 30
app.config.KEEP_ALIVE_TIMEOUT = 120
app.config.FALLBACK_ERROR_FORMAT = "json"
```

Environment variables prefixed with `SANIC_` are loaded into config by default:

```bash
export SANIC_REQUEST_TIMEOUT=10
export SANIC_RESPONSE_TIMEOUT=30
export SANIC_KEEP_ALIVE_TIMEOUT=60
```

```python
from sanic import Sanic

app = Sanic("cfg")
print(app.config.REQUEST_TIMEOUT)
```

If you do not want implicit env loading:

```python
from sanic import Sanic

app = Sanic("cfg", load_env=False)
```

Sanic `25.12` extends config converters with `DetailedConverter`, which is useful if you need custom environment-to-config parsing beyond the default type coercion.

## Blueprints, Websockets, And Background Tasks

Use blueprints once routes start to split by feature area.

```python
from sanic import Blueprint, Sanic
from sanic.response import json

users = Blueprint("users", url_prefix="/users")

@users.get("/<user_id:int>")
async def get_user(_request, user_id: int):
    return json({"user_id": user_id})

app = Sanic("svc")
app.blueprint(users)
```

Websocket routes are first-class:

```python
from sanic import Sanic

app = Sanic("ws")

@app.websocket("/ws")
async def echo(_request, ws):
    async for message in ws:
        await ws.send(message)
```

For app-managed tasks:

```python
task = app.add_task(refresh_index())
```

In `25.12`, `app.add_task()` returns the created task object, which makes later tracking or cancellation easier.

## Running And Deployment

Preferred server modes:

```bash
sanic server:app --host=0.0.0.0 --port=8000 --workers=4
```

```bash
sanic server:app --fast
```

Important deployment notes:

- `--workers` uses Sanic's worker manager for multi-process serving
- `--fast` chooses a maximum worker count automatically
- `--single-process` disables the worker manager and some reload/process-management features
- daemon mode is available in `25.12` if you need to detach the Sanic server process

ASGI deployment is also supported:

```bash
uvicorn myapp:app
hypercorn myapp:app
```

In ASGI mode, startup and shutdown semantics follow the ASGI server lifecycle instead of Sanic's native multi-process server lifecycle. Treat that as a deployment change, not a transparent drop-in.

## Testing

Install the official test companion:

```bash
pip install sanic-testing
```

Typical usage:

```python
from sanic import Sanic
from sanic.response import json

app = Sanic("test-app")

@app.get("/ping")
async def ping(_request):
    return json({"pong": True})

def test_ping():
    _, response = app.test_client.get("/ping")
    assert response.status_code == 200
    assert response.json == {"pong": True}
```

If `app.test_client` or `app.asgi_client` is missing, you probably forgot to install `sanic-testing`.

## Common Pitfalls

- Do not use `https://sanic.readthedocs.io/en/stable/` as your primary reference for this package version. It still serves `23.12.0`.
- Sanic `25.12.0` requires Python `3.10+`. Older project templates that still allow `3.8` or `3.9` are stale.
- Do not put blocking work like `time.sleep()` or sync database clients inside handlers. Use async libraries or move work off the event loop.
- `request.args`, `request.form`, and `request.files` can hold repeated values. Use `.getlist()` when shape matters.
- Put shared resources on `app.ctx`, not `request.ctx`.
- Returning a response from request middleware short-circuits the rest of the request pipeline.
- `sanic-testing` is a separate package by design.
- If you deploy through an ASGI server, verify lifecycle and websocket behavior in that mode instead of assuming Sanic's native server semantics.

## Version-Sensitive Notes For `25.12.0`

- `25.12` is the current LTS line.
- Sanic `25.12` drops Python `3.9` and adds Python `3.14` support.
- Daemon mode was added in `25.12`.
- Configuration converters gained `DetailedConverter` in `25.12`.
- Static file text responses now add `charset=UTF-8` automatically in `25.12`.
- `app.add_task()` now returns the created task object.

## Official Sources

- User guide: `https://sanic.dev/en/guide/`
- Running guide: `https://sanic.dev/en/guide/running/running.html`
- Configuration guide: `https://sanic.dev/en/guide/running/configuration.html`
- Request guide: `https://sanic.dev/en/guide/basics/request.html`
- App guide: `https://sanic.dev/en/guide/basics/app.html`
- Middleware guide: `https://sanic.dev/en/guide/basics/middleware.html`
- Blueprints guide: `https://sanic.dev/en/guide/best-practices/blueprints.html`
- Authentication guide: `https://sanic.dev/en/guide/how-to/authentication.html`
- Testing guide: `https://sanic.dev/en/plugins/sanic-testing/clients.html`
- `25.12` release notes: `https://sanic.dev/en/release-notes/2025/v25.12.html`
- Changelog: `https://sanic.dev/en/release-notes/changelog.html`
- PyPI release page: `https://pypi.org/project/sanic/25.12.0/`

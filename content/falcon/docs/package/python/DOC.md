---
name: package
description: "Falcon web framework for building high-performance WSGI and ASGI APIs in Python"
metadata:
  languages: "python"
  versions: "4.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "falcon,python,web,api,framework,asgi,wsgi"
---

# Falcon Python Package Guide

Falcon is a low-level Python framework for HTTP APIs and microservices. Use it when you want explicit routing, request/response objects, middleware hooks, and a choice between WSGI and ASGI without a batteries-included stack.

## Golden Rule

- Pick the runtime model first:
  - `falcon.App()` for WSGI
  - `falcon.asgi.App()` for ASGI
- Falcon does not include a production server. Pair WSGI apps with Gunicorn, uWSGI, or Waitress, and ASGI apps with Uvicorn, Daphne, or Hypercorn.
- Falcon `4.2.0` supports CPython `3.9+` and PyPy `3.9+`.
- Falcon has no required runtime dependencies.
- Use `req.get_media()` / `await req.get_media()` for request bodies and `resp.media` for JSON responses.

## Install

```bash
pip install "falcon==4.2.0"
```

If you also need a server locally:

```bash
pip install "falcon==4.2.0" gunicorn
pip install "falcon==4.2.0" uvicorn
```

Falcon ships many prebuilt wheels. If you need a source build that cythonizes locally:

```bash
pip install --no-binary :all: "falcon==4.2.0"
```

## Choose WSGI vs ASGI First

### WSGI app

Use WSGI when your app is synchronous and will run behind a WSGI server.

```python
import falcon

class ThingsResource:
    def on_get(self, req, resp):
        resp.media = {"status": "ok"}

app = falcon.App()
app.add_route("/things", ThingsResource())
```

Typical run command:

```bash
gunicorn example:app
```

### ASGI app

Use ASGI when you need `async` responders, WebSockets, SSE, or ASGI lifespan support.

```python
import falcon
import falcon.asgi

class ThingsResource:
    async def on_get(self, req, resp):
        resp.media = {"status": "ok"}

app = falcon.asgi.App()
app.add_route("/things", ThingsResource())
```

Typical run command:

```bash
uvicorn example:app
```

## Core Usage Pattern

Falcon is resource-oriented. You register a long-lived resource instance with `app.add_route()` and implement responders such as `on_get()` or `on_post()`.

```python
import falcon

class ItemResource:
    def on_get(self, req, resp, item_id):
        resp.media = {
            "item_id": item_id,
            "limit": req.get_param_as_int("limit") or 20,
        }

    def on_post(self, req, resp, item_id):
        payload = req.get_media()
        resp.media = {
            "item_id": item_id,
            "received": payload,
        }
        resp.status = falcon.HTTP_201

app = falcon.App()
app.add_route("/items/{item_id}", ItemResource())
```

ASGI body parsing is the same idea, but `get_media()` must be awaited:

```python
import falcon
import falcon.asgi

class ItemResource:
    async def on_post(self, req, resp):
        payload = await req.get_media()
        resp.media = payload

app = falcon.asgi.App()
app.add_route("/items", ItemResource())
```

Practical request/response rules:

- Route params are passed as responder arguments.
- Query params come from helpers like `req.get_param()` and `req.get_param_as_int()`.
- `resp.media` uses Falcon's media handlers and defaults to JSON.
- Calling `req.get_media()` consumes the request body stream once; Falcon caches the parsed object for later access.

## Config and App Setup

Falcon stays unopinionated about application config. The usual pattern is:

- Load settings from env vars or your own config object.
- Pass shared services into resource constructors.
- Use app kwargs and middleware for framework-level behavior.

Common app-level options:

- `middleware=[...]` for request/response cross-cutting logic
- `cors_enable=True` for a simple global CORS policy
- `request_type=` / `response_type=` when you need custom request or response classes
- `independent_middleware=False` only if you intentionally want response middleware skipped after request-stage exceptions

If you need custom JSON serialization or alternate media types, update both request and response media handlers:

```python
import json
from functools import partial

import falcon
from falcon import media

json_handler = media.JSONHandler(
    dumps=partial(json.dumps, separators=(",", ":")),
)

app = falcon.App()
app.req_options.media_handlers.update({"application/json": json_handler})
app.resp_options.media_handlers.update({"application/json": json_handler})
```

## Auth and Middleware

Auth is typically implemented in middleware or hooks. Middleware runs in the order you register it.

```python
import falcon

class AuthMiddleware:
    def process_request(self, req, resp):
        token = req.get_header("Authorization")
        if token != "Bearer secret-token":
            raise falcon.HTTPUnauthorized(
                title="Authentication required",
                description="Missing or invalid bearer token",
            )

        req.context.user_id = "demo-user"

class ProfileResource:
    def on_get(self, req, resp):
        resp.media = {"user_id": req.context.user_id}

app = falcon.App(middleware=[AuthMiddleware()])
app.add_route("/profile", ProfileResource())
```

Use middleware when you need:

- authentication and authorization
- request IDs or trace context
- tenant resolution
- request/response logging
- cross-cutting validation before routing or before the responder runs

## Testing

Use Falcon's test helpers instead of spinning up a real server in unit tests.

```python
import falcon
from falcon import testing

class HealthResource:
    def on_get(self, req, resp):
        resp.media = {"ok": True}

app = falcon.App()
app.add_route("/health", HealthResource())

client = testing.TestClient(app)
result = client.simulate_get("/health")

assert result.status_code == 200
assert result.json == {"ok": True}
```

For ASGI apps, Falcon also provides `testing.ASGIConductor` when you need to test streams, SSE, or WebSocket flows.

## Common Pitfalls

- Do not run `falcon.App()` behind an ASGI server or `falcon.asgi.App()` behind a WSGI server.
- Do not assume Falcon includes an ORM, schema validation layer, auth system, dependency injection container, or production server.
- Do not put per-request mutable state on a resource instance; resources are long-lived.
- Prefer `req.get_media()` over reading raw request streams unless you have a specific streaming need.
- For ASGI apps, remember to `await req.get_media()` and keep middleware/responders async-aware.
- Middleware order matters. Response middleware is executed as a stack, and by default Falcon still runs `process_response()` even when request-stage code raises.
- `cors_enable=True` is only a simple CORS policy. Use `CORSMiddleware` if you need finer control.
- The quickstart uses `wsgiref` to demonstrate a minimal WSGI app, but it is not a production deployment choice.

## Version-Sensitive Notes for 4.2.0

- Falcon `4.2.0` was released on `2025-11-10` on PyPI.
- `4.2.0` primarily adds typing improvements and performance optimizations.
- The release makes WSGI and ASGI `App` types generic over request and response types, which matters if you use custom `request_type` / `response_type` classes and type check your code.
- `4.2.0` also introduced precompiled wheels for free-threaded CPython `3.14` on selected Linux x86 targets.
- The docs URL points at `https://falcon.readthedocs.io/en/stable/api/`. As of `2026-03-12`, that `stable` docs root serves Falcon `4.2.0`, but it will drift in the future. Prefer pairing it with the pinned PyPI version and release notes when reproducibility matters.

## Official Sources

- Docs root: https://falcon.readthedocs.io/en/stable/
- API landing page: https://falcon.readthedocs.io/en/stable/api/
- Installation: https://falcon.readthedocs.io/en/stable/user/install.html
- Quickstart: https://falcon.readthedocs.io/en/stable/user/quickstart.html
- App class reference: https://falcon.readthedocs.io/en/stable/api/app.html
- Media handling: https://falcon.readthedocs.io/en/stable/api/media.html
- Middleware reference: https://falcon.readthedocs.io/en/stable/api/middleware.html
- Testing helpers: https://falcon.readthedocs.io/en/stable/api/testing.html
- PyPI package page: https://pypi.org/project/falcon/
- PyPI release page: https://pypi.org/project/falcon/4.2.0/
- GitHub release notes: https://github.com/falconry/falcon/releases/tag/4.2.0
- Repository: https://github.com/falconry/falcon

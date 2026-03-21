---
name: cors
description: "aiohttp-cors package guide for adding CORS policies to aiohttp.web apps"
metadata:
  languages: "python"
  versions: "0.8.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aiohttp-cors,aiohttp,cors,http,asyncio,python,web"
---

# aiohttp-cors Python Package Guide

## Golden Rule

Use `aiohttp-cors` only to express browser CORS policy for `aiohttp.web` routes and resources. It does not replace your app's authentication, authorization, CSRF protection, or normal request validation.

## Install

```bash
pip install aiohttp-cors==0.8.1
```

Base imports:

```python
from aiohttp import web
import aiohttp_cors
```

## Setup Pattern

Create the `aiohttp.web.Application`, initialize CORS once with `aiohttp_cors.setup(...)`, then wrap the routes or resources that should emit CORS headers.

```python
from aiohttp import web
import aiohttp_cors

async def health(request: web.Request) -> web.Response:
    return web.json_response({"ok": True})

app = web.Application()

cors = aiohttp_cors.setup(
    app,
    defaults={
        "https://app.example.com": aiohttp_cors.ResourceOptions(
            allow_credentials=True,
            expose_headers=("X-Request-Id",),
            allow_headers=("Authorization", "Content-Type"),
            max_age=3600,
        )
    },
)

route = app.router.add_get("/health", health)
cors.add(route)

web.run_app(app)
```

Important behavior:

- `setup()` alone does not change any route behavior.
- A route must be passed to `cors.add(route)` or attached via a resource/view pattern.
- `defaults=` is only applied to wrapped routes and resources.

## Core Usage

### Add CORS To A Route

Use route-level wrapping when one endpoint has its own policy.

```python
async def create_item(request: web.Request) -> web.Response:
    return web.json_response({"created": True}, status=201)

route = app.router.add_post("/items", create_item)

cors.add(
    route,
    {
        "https://console.example.com": aiohttp_cors.ResourceOptions(
            allow_credentials=True,
            allow_headers=("Authorization", "Content-Type"),
            expose_headers=("Location",),
            allow_methods=("POST",),
        )
    },
)
```

### Add CORS To A Resource

When multiple methods share one path, resource-level wrapping is usually cleaner.

```python
resource = app.router.add_resource("/items")
resource.add_route("GET", list_items)
resource.add_route("POST", create_item)

cors.add(
    resource,
    {
        "https://app.example.com": aiohttp_cors.ResourceOptions(
            allow_credentials=True,
            allow_headers=("Authorization", "Content-Type"),
            allow_methods=("GET", "POST"),
        )
    },
)
```

### Apply One Policy To Many Existing Routes

The maintainer README shows iterating over registered routes after setup:

```python
for route in list(app.router.routes()):
    cors.add(route)
```

This is useful when you want one default policy on most of the app and only a few explicit overrides.

## Class-Based Views And Custom Preflight

The maintainer README includes a `web.View` pattern for wildcard-style class-based views. Use `aiohttp_cors.CorsViewMixin` and register the route with `webview=True`.

```python
from aiohttp import hdrs, web
import aiohttp_cors

class UserView(web.View, aiohttp_cors.CorsViewMixin):
    async def get(self) -> web.StreamResponse:
        return web.json_response({"method": "GET"})

    @aiohttp_cors.custom_cors(
        hdrs.METH_OPTIONS,
        allow_headers=("Content-Type", "Authorization"),
        expose_headers=("X-Request-Id",),
        allow_credentials=True,
        max_age=600,
    )
    async def options(self) -> web.StreamResponse:
        return web.Response(status=200)

app.router.add_route("*", "/users", UserView, name="users", webview=True)
```

Use this only when you specifically need a class-based view with custom `OPTIONS` handling. For normal handlers, explicit route or resource registration is simpler.

## Config And Auth Notes

`aiohttp_cors.ResourceOptions(...)` is the main place where browser-facing policy is defined.

Common fields used in maintainer examples:

- `allow_credentials=True` lets browsers send cookies or auth headers cross-origin when the frontend and backend policy allows it.
- `allow_headers=(...)` lists request headers the browser may send.
- `expose_headers=(...)` lists response headers browser JavaScript may read.
- `allow_methods=(...)` narrows the allowed HTTP methods when you want stricter policy than the route definitions alone.
- `max_age=...` controls how long the preflight response may be cached.

Practical guidance:

- Prefer explicit origins like `https://app.example.com` in production.
- Treat `"*"` as a deliberate relaxation for controlled cases, not a default production policy.
- Keep your actual auth and permission checks in middleware, handlers, or upstream infrastructure.

## Common Pitfalls

### Forgetting To Wrap The Route

This does not enable CORS by itself:

```python
app = web.Application()
cors = aiohttp_cors.setup(app)
app.router.add_get("/items", list_items)
```

You still need:

```python
route = app.router.add_get("/items", list_items)
cors.add(route)
```

### Assuming Wildcard Handlers Work Automatically

The maintainer README calls out an `aiohttp` limitation around wildcard handlers such as `app.router.add_route("*", ...)`. If you need that shape, use the documented `CorsViewMixin` plus `webview=True` pattern instead of assuming normal route wrapping will work.

### Copying Historical Coroutine Syntax Verbatim

The upstream README still includes older `@asyncio.coroutine` examples. In modern Python code, translate those examples to `async def` handlers and `await` syntax.

### Treating CORS As Security By Itself

CORS controls what browsers may do with cross-origin requests. It does not make an endpoint safe to expose.

## Version-Sensitive Notes For 0.8.1

- PyPI marks `0.8.1` as released on `2025-03-31`.
- PyPI release history marks `0.8.0` as yanked. The yank reason says it "installed on Python 3.8, but wasn't compatible with it."
- The maintainer changelog says `0.8.0` added compatibility with `aiohttp 3.9+` and `python 3.9+`.
- The maintainer changelog says `0.8.1` fixed the packaging error so the project no longer installs on Python `3.8`.
- Inference: use `aiohttp-cors 0.8.1` only on Python `3.9+` projects running a supported `aiohttp 3.9+` line.

## Official Sources

- Repository: https://github.com/aio-libs/aiohttp-cors
- README: https://github.com/aio-libs/aiohttp-cors/blob/master/README.rst
- Versioned changelog entry: https://github.com/aio-libs/aiohttp-cors/blob/v0.8.1/CHANGES.rst
- PyPI project page: https://pypi.org/project/aiohttp-cors/
- PyPI release page for `0.8.1`: https://pypi.org/project/aiohttp-cors/0.8.1/

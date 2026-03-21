---
name: package
description: "Werkzeug package guide for Python - WSGI request/response, routing, middleware, and testing"
metadata:
  languages: "python"
  versions: "3.1.6"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "werkzeug,wsgi,http,routing,middleware,testing,python"
---

# Werkzeug Python Package Guide

## When To Use It

Use `werkzeug` when you need low-level WSGI building blocks without committing to a full framework. It provides:

- request and response wrappers
- routing via `Map` and `Rule`
- a local development server
- middleware such as `ProxyFix`
- a test client for WSGI apps
- security helpers such as password hashing and safe path joining

If you are working inside Flask, remember Flask already uses Werkzeug. Reach for direct Werkzeug APIs when you are writing raw WSGI apps, middleware, request/response utilities, routing logic, or tests around those layers.

## Installation

```bash
pip install Werkzeug==3.1.6
```

With the faster file-watching reloader for local development:

```bash
pip install "Werkzeug[watchdog]==3.1.6"
```

Optional upstream notes:

- `watchdog` improves the dev reloader.
- `colorama` enables colored request logs on Windows.
- `greenlet>=1.0` is required if you run with `gevent` or `eventlet`.

## Minimal WSGI App

```python
from werkzeug.wrappers import Request, Response
from werkzeug.serving import run_simple

@Request.application
def app(request: Request) -> Response:
    name = request.args.get("name", "world")
    return Response(f"Hello, {name}!", mimetype="text/plain")

if __name__ == "__main__":
    run_simple("127.0.0.1", 5000, app, use_reloader=True, use_debugger=True)
```

Notes:

- `@Request.application` lets you write a function that receives a `Request` and returns a `Response`.
- `run_simple()` is for local development only.
- Do not enable the debugger or dev server in production.

## Core Request And Response Usage

```python
from werkzeug.wrappers import Request, Response

@Request.application
def app(request: Request) -> Response:
    if request.method == "POST":
        username = request.form["username"]
        return Response(f"created {username}", status=201)

    page = request.args.get("page", default=1, type=int)
    return Response(f"page={page}", mimetype="text/plain")
```

Useful request attributes:

- `request.args` for query parameters
- `request.form` for URL-encoded or multipart form fields
- `request.files` for uploaded files
- `request.headers` for request headers
- `request.cookies` for cookies
- `request.get_data()` for raw body bytes when you intentionally need the full body in memory

Useful response patterns:

```python
from werkzeug.wrappers import Response

response = Response("ok", status=200, mimetype="text/plain")
response.headers["X-App-Version"] = "1"
response.set_cookie("session", "abc123", httponly=True, samesite="Lax")
```

## Routing With `Map` And `Rule`

Werkzeug routing is explicit and independent of any framework.

```python
from werkzeug.exceptions import HTTPException
from werkzeug.routing import Map, Rule
from werkzeug.wrappers import Request, Response

url_map = Map(
    [
        Rule("/", endpoint="index"),
        Rule("/users/<int:user_id>", endpoint="user-detail"),
    ]
)

@Request.application
def app(request: Request) -> Response:
    adapter = url_map.bind_to_environ(request.environ)

    try:
        endpoint, values = adapter.match()
    except HTTPException as exc:
        return exc

    if endpoint == "index":
        return Response("home")

    if endpoint == "user-detail":
        return Response(f"user={values['user_id']}")

    return Response("not found", status=404)
```

Routing behaviors to remember:

- `strict_slashes=True` by default, so branch URLs redirect to the trailing-slash form.
- `merge_slashes=True` by default, so repeated slashes may normalize and redirect.
- `HEAD` is added automatically when a rule allows `GET`.
- WebSocket routing exists, but Werkzeug does not provide a full WebSocket stack beyond matching.

## Testing Without Running A Server

```python
from werkzeug.test import Client
from werkzeug.wrappers import Response

client = Client(app, Response)

response = client.get("/users/42")
assert response.status_code == 200
assert response.get_data(as_text=True) == "user=42"
```

For forms and uploads:

```python
import io

response = client.post(
    "/upload",
    data={"title": "report", "file": (io.BytesIO(b"hello"), "report.txt")},
)
```

The test client keeps cookies across requests by default, which is useful for login and session flows.

## Configuration And Security-Relevant Setup

Werkzeug does not impose an application config system. You configure behavior in your own app, request subclass, or middleware stack.

### Reverse Proxy Setup

If the app is behind Nginx, a load balancer, or a platform proxy, use `ProxyFix` only when you know exactly how many trusted proxies are in front of the app.

```python
from werkzeug.middleware.proxy_fix import ProxyFix

app = ProxyFix(app, x_for=1, x_proto=1, x_host=1, x_port=1, x_prefix=1)
```

Do not blindly copy the counts. Incoming `X-Forwarded-*` headers can be faked, so the wrong `ProxyFix` configuration is a security bug.

### Request Size Limits

For upload-heavy or public endpoints, set request limits explicitly.

```python
from werkzeug.wrappers import Request, Response

class AppRequest(Request):
    max_content_length = 16 * 1024 * 1024  # 16 MiB
    max_form_memory_size = 500_000
    max_form_parts = 1_000

@AppRequest.application
def app(request: AppRequest) -> Response:
    return Response("ok")
```

Use server-level limits too. Werkzeug's request limits are only one protection layer.

### Password Hashing And Basic Auth Helpers

Werkzeug is not an auth framework, but it includes useful primitives:

```python
from werkzeug.security import check_password_hash, generate_password_hash

stored_hash = generate_password_hash("correct horse battery staple")
assert check_password_hash(stored_hash, "correct horse battery staple")
```

In `3.1`, the default PBKDF2 work factor increased. Do not hard-code assumptions about hash parameters. Store the full hash string and let Werkzeug verify it.

For HTTP auth headers, prefer `request.authorization` and other request/header helpers instead of manually splitting the `Authorization` header string.

## File Handling Helpers

For user-uploaded filenames and user-controlled download paths:

```python
from werkzeug.security import safe_join
from werkzeug.utils import secure_filename

filename = secure_filename(user_filename)
path = safe_join("/srv/uploads", filename)
if path is None:
    raise ValueError("invalid path")
```

Prefer `send_from_directory()` for serving user-selected files. Do not pass raw user paths to `send_file()`.

## Production Deployment

Do not use these in production:

- `run_simple()`
- `use_debugger=True`
- `DebuggedApplication`

Use a dedicated WSGI server or hosting platform instead. Werkzeug's built-in server, reloader, and debugger are development tools, not production infrastructure.

## Common Pitfalls

- Reading from `wsgi.input` directly can hang or interfere with parsing. Prefer the `Request` object or `parse_form_data()`, not both.
- Accessing `request.form`, `request.files`, and raw input stream operations in the wrong order can consume the body in surprising ways.
- `request.get_data()` loads the full body into memory. Set `max_content_length` first for untrusted requests.
- `ProxyFix` with the wrong proxy counts will trust spoofed headers.
- `send_file()` assumes the path is trusted. Use `send_from_directory()` for user-controlled names.
- The interactive debugger can execute arbitrary code. Never expose it publicly.
- The dev server may appear to work in staging-like environments, but upstream explicitly says it is not secure, stable, or efficient for production.

## Version-Sensitive Notes For `3.1.x`

- `3.1.6` was released on 2026-02-19 and tightens `safe_join()` handling for Windows special device names in multi-segment paths.
- `3.1.5` and `3.1.4` also contain Windows path safety fixes around `safe_join()` and `send_from_directory()`.
- `3.1.0` dropped Python `3.8`; `3.1.x` requires Python `3.9+`.
- `3.1.0` changed `Request.max_form_memory_size` from unlimited to a default of `500 kB`.
- `3.1.0` increased the default PBKDF2 work factor to `1,000,000`.
- Older blog posts may still reference removed or deprecated APIs from `2.x` or early `3.0`. Check the upstream change log before copying imports or wrapper patterns.

## Official Sources

- Stable docs: https://werkzeug.palletsprojects.com/en/stable/
- Installation: https://werkzeug.palletsprojects.com/en/stable/installation/
- Quickstart: https://werkzeug.palletsprojects.com/en/stable/quickstart/
- Routing: https://werkzeug.palletsprojects.com/en/stable/routing/
- Testing: https://werkzeug.palletsprojects.com/en/stable/test/
- ProxyFix middleware: https://werkzeug.palletsprojects.com/en/stable/middleware/proxy_fix/
- Request data limits: https://werkzeug.palletsprojects.com/en/stable/request_data/
- Utilities and security helpers: https://werkzeug.palletsprojects.com/en/stable/utils/
- Serving and deployment: https://werkzeug.palletsprojects.com/en/stable/serving/ and https://werkzeug.palletsprojects.com/en/stable/deployment/
- Change log: https://werkzeug.palletsprojects.com/en/stable/changes/
- PyPI package: https://pypi.org/project/Werkzeug/

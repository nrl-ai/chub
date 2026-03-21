---
name: package
description: "Bottle Python package guide for small WSGI apps, APIs, and templated services"
metadata:
  languages: "python"
  versions: "0.13.4"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "bottle,python,web,wsgi,microframework,api"
---

# bottle Python Package Guide

## What It Is

`bottle` is a small, single-file WSGI web framework for Python. Use it for small services, internal tools, admin panels, webhooks, or JSON APIs when you want minimal framework overhead and direct control over request handling.

## Version Scope

- This entry covers PyPI package version `0.13.4`.
- The version used here and the current stable upstream version both point to `0.13.4`.
- Prefer the version-pinned docs at `https://bottlepy.org/docs/0.13/` when you need line-accurate upstream behavior for this package version.

## Install

```bash
pip install "bottle==0.13.4"
```

Common alternatives:

```bash
uv add "bottle==0.13.4"
poetry add "bottle==0.13.4"
```

Bottle is pure Python and ships as a single package. The import path is also `bottle`.

## Create An App

Prefer an explicit `Bottle()` instance for non-trivial projects. Bottle also supports module-level decorators and a default app, but an explicit app is easier to test, configure, and mount under a real WSGI server.

```python
from bottle import Bottle, request, response

app = Bottle()
app.config.update(
    DEBUG=True,
    API_TITLE="example-service",
)

@app.get("/health")
def health() -> dict[str, object]:
    response.status = 200
    return {
        "ok": True,
        "service": app.config["API_TITLE"],
        "client_ip": request.remote_addr,
    }

if __name__ == "__main__":
    app.run(host="127.0.0.1", port=8080, debug=True, reloader=True)
```

Notes:

- Returning a `dict` produces JSON by default in `0.13` because `Bottle(autojson=True)` is the default.
- `debug=True` and `reloader=True` are development-only settings.
- The auto-reloader imports your module twice. Keep module-level side effects idempotent.

## Routing

Bottle routes are usually declared with HTTP method decorators.

```python
from bottle import Bottle, abort

app = Bottle()

@app.get("/users/<user_id:int>")
def get_user(user_id: int):
    if user_id <= 0:
        abort(400, "user_id must be positive")
    return {"user_id": user_id}

@app.post("/users/<user_id:int>/activate")
def activate_user(user_id: int):
    return {"user_id": user_id, "active": True}

@app.get("/files/<path:path>")
def get_file(path: str):
    return {"path": path}
```

Useful patterns:

- `/<name>`: plain wildcard
- `/<id:int>`: integer conversion
- `/<path:path>`: slash-containing tail segment

Avoid older blog posts that still use pre-`0.13` route syntax or rely on the global default app everywhere.

## Read Request Data

Bottle exposes request state through the global `request` object during a request.

### Query Parameters And Forms

```python
from bottle import Bottle, request

app = Bottle()

@app.get("/search")
def search():
    query = request.query.get("q", "").strip()
    page = int(request.query.get("page", "1"))
    tags = request.query.getall("tag")
    return {"q": query, "page": page, "tags": tags}

@app.post("/login")
def login():
    username = request.forms.get("username", "")
    password = request.forms.get("password", "")
    return {"accepted": bool(username and password)}
```

`request.query` and `request.forms` are `FormsDict` objects. Use `getall()` when a key can appear multiple times.

### JSON Bodies

```python
from bottle import Bottle, request, abort

app = Bottle()

@app.post("/tasks")
def create_task():
    payload = request.json
    if not payload or "title" not in payload:
        abort(400, "JSON body with 'title' is required")
    return {"task": {"title": payload["title"]}}
```

`request.json` is the common path for JSON APIs. Treat it as optional input and validate required fields yourself.

### File Uploads

```python
from pathlib import Path
from bottle import Bottle, request, abort

app = Bottle()
UPLOAD_DIR = Path("./uploads")
UPLOAD_DIR.mkdir(exist_ok=True)

@app.post("/upload")
def upload():
    upload = request.files.get("file")
    if upload is None:
        abort(400, "missing file field")

    filename = Path(upload.filename).name
    target = UPLOAD_DIR / filename
    upload.save(str(target), overwrite=False)
    return {"saved_to": str(target)}
```

Do not trust the client filename or path. Normalize the name and keep the destination directory fixed on the server side.

## Build Responses

Bottle handlers can return strings, bytes, dicts, generators, or raise HTTP helpers like `abort()`.

```python
from bottle import Bottle, response, redirect, abort

app = Bottle()

@app.get("/plain")
def plain():
    response.content_type = "text/plain; charset=UTF-8"
    return "ok"

@app.get("/created")
def created():
    response.status = 201
    response.set_header("X-Service", "bottle")
    return {"created": True}

@app.get("/old-path")
def old_path():
    redirect("/plain")

@app.get("/boom")
def boom():
    abort(503, "temporarily unavailable")
```

If you need to control headers or status codes, set them on `response` before returning the body.

## Templates And Static Files

Bottle includes the SimpleTemplate engine and static-file helpers.

```python
from pathlib import Path
from bottle import Bottle, template, static_file

app = Bottle()
STATIC_ROOT = Path(__file__).parent / "static"

@app.get("/hello/<name>")
def hello(name: str):
    return template("Hello {{name}}!", name=name)

@app.get("/assets/<filename:path>")
def assets(filename: str):
    return static_file(filename, root=str(STATIC_ROOT))
```

Use `static_file()` instead of manually opening files. It handles range requests, content types, and keeps the root directory explicit.

## Config, Cookies, And Auth

Bottle does not ship a full authentication framework. The common pattern is:

- store secrets and environment-specific settings in `app.config`
- authenticate from headers or signed cookies
- centralize auth checks in a hook or plugin

### App Config

```python
from bottle import Bottle
import os

app = Bottle()
app.config.load_dict(
    session_cookie="session",
    session_secret=os.environ["SESSION_SECRET"],
    api_token=os.environ.get("API_TOKEN", ""),
)
```

### Signed Cookies

```python
from bottle import Bottle, request, response, abort

app = Bottle()
COOKIE_NAME = "session"
COOKIE_SECRET = "replace-me"

@app.post("/session")
def create_session():
    response.set_cookie(
        COOKIE_NAME,
        "user-123",
        secret=COOKIE_SECRET,
        httponly=True,
        secure=True,
        samesite="Lax",
        path="/",
    )
    return {"created": True}

@app.get("/session")
def read_session():
    user_id = request.get_cookie(COOKIE_NAME, secret=COOKIE_SECRET)
    if user_id is None:
        abort(401, "missing or invalid session")
    return {"user_id": user_id}
```

Signed cookies are integrity-protected, not encrypted. Do not put raw secrets or private data in the cookie value.

### Header-Based Auth With A Hook

```python
from bottle import Bottle, request, abort

app = Bottle()
app.config["api_token"] = "replace-me"

@app.hook("before_request")
def require_api_token():
    if request.path == "/health":
        return
    if request.get_header("X-API-Key") != app.config["api_token"]:
        abort(401, "invalid API token")
```

Hooks are the simplest way to enforce cross-cutting policy on small apps. For reusable behavior across multiple apps, package the logic as a plugin and install it with `app.install(...)`.

## Production Deployment

Bottle apps are WSGI callables. The built-in development server is not for production.

```python
# myservice.py
from bottle import Bottle

app = Bottle()

@app.get("/")
def index():
    return {"ok": True}
```

Run it under a real WSGI server:

```bash
gunicorn myservice:app
```

Deployment rules:

- Keep `app.run(...)` for local development only.
- In production, use a WSGI server such as Gunicorn, Cheroot, Waitress, or another supported server adapter.
- Bottle is a WSGI framework, not an ASGI framework. Do not paste `async def` or ASGI middleware patterns from FastAPI or Starlette examples into Bottle code.

## Common Pitfalls

- `debug=True` enables verbose errors and template reloading. Do not enable it in production.
- The reloader restarts the process and imports the module twice.
- Returning a `dict` auto-serializes to JSON in `0.13`; older examples may show manual `json.dumps(...)` for the common case.
- `request` and `response` are request-local globals. Do not cache them across requests.
- `static_file()` still needs a trusted root directory. Never join untrusted paths directly to an arbitrary filesystem location.
- Signed cookies protect against tampering, not disclosure.
- Bottle supports a default app and module-level decorators, but explicit `Bottle()` instances are safer for tests, mounting, and larger codebases.

## Version-Sensitive Notes For 0.13.x

- `0.13` changed several compatibility assumptions. Upstream documents Python `3.8+` support and keeps deprecated Python `2.7.3+` compatibility for the `0.13` line.
- `Bottle.autojson` defaults to `True` in `0.13`, so plain `dict` return values are JSON responses unless you override that behavior.
- The CLI script name is `bottle`, not `bottle.py`.
- `Bottle.app()` now returns a configured default app, so old examples that depend on implicit global state can behave differently than older `0.12` tutorials suggest.
- If you are copying old `0.12` tutorials, re-check route declarations, JSON response handling, and deployment snippets against the `0.13` docs before using them.

## Official Links

- Version-pinned docs: https://bottlepy.org/docs/0.13/
- Stable docs root: https://bottlepy.org/docs/stable/
- Tutorial: https://bottlepy.org/docs/0.13/tutorial.html
- API reference: https://bottlepy.org/docs/0.13/api.html
- Deployment guide: https://bottlepy.org/docs/0.13/deployment.html
- Plugin guide: https://bottlepy.org/docs/0.13/plugins/index.html
- Changelog: https://bottlepy.org/docs/0.13/changelog.html
- PyPI package: https://pypi.org/project/bottle/

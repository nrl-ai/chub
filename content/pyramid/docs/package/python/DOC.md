---
name: package
description: "Pyramid package guide for Python WSGI web applications and services"
metadata:
  languages: "python"
  versions: "2.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pyramid,web,framework,wsgi,routing,python"
---

# Pyramid Python Package Guide

## What It Is

`pyramid` is a Python web framework for WSGI applications. It favors explicit configuration: define routes, attach views, build a WSGI app, and add only the pieces you need for templates, sessions, security, and persistence.

- Package name: `pyramid`
- Import name: `pyramid`
- Version covered: `2.1`
- PyPI release date for `2.1`: `2026-03-11`
- PyPI requirement for `2.1`: `Python >=3.10`

## Installation

Install the framework version your project expects:

```bash
pip install pyramid==2.1
```

Common companion tools from the official tutorial and deployment flow are:

```bash
pip install "pyramid==2.1" waitress cookiecutter
```

- `waitress`: production-ready WSGI server commonly used with Pyramid
- `cookiecutter`: generates the official starter project now that `pcreate` is gone in `2.1`

Basic imports:

```python
from pyramid.config import Configurator
from pyramid.response import Response
from pyramid.view import view_config
```

## Initialization And Setup

### Fastest Start: Official Starter

Pyramid `2.1` removed `pcreate`. Use the official Cookiecutter starter instead:

```bash
cookiecutter gh:Pylons/pyramid-cookiecutter-starter
cd <your-project>
python -m pip install -e ".[testing]"
pserve development.ini --reload
```

This gives you a working application factory, `development.ini` and `production.ini`, and a layout that matches the upstream tutorial and deployment flow.

### Minimal App Without A Scaffold

If you just need a tiny app or test fixture, wire Pyramid directly:

```python
from wsgiref.simple_server import make_server

from pyramid.config import Configurator
from pyramid.response import Response

def hello_world(request):
    return Response("Hello, Pyramid!")

if __name__ == "__main__":
    config = Configurator()
    config.add_route("hello", "/")
    config.add_view(hello_world, route_name="hello")
    app = config.make_wsgi_app()

    server = make_server("127.0.0.1", 6543, app)
    server.serve_forever()
```

Important setup points:

- `Configurator` is where routes, views, renderers, security policy, session factory, and settings are registered.
- `config.make_wsgi_app()` produces the WSGI app you serve with `waitress`, `gunicorn`, or another WSGI server.
- If you use decorator-based configuration such as `@view_config`, call `config.scan()` before `make_wsgi_app()`.

## Core Usage

### Route A URL To A View

Imperative configuration is the most direct Pyramid style:

```python
from pyramid.config import Configurator
from pyramid.response import Response

def healthcheck(request):
    return Response("ok")

config = Configurator()
config.add_route("healthcheck", "/healthz")
config.add_view(healthcheck, route_name="healthcheck")
app = config.make_wsgi_app()
```

### Use Decorator-Based Views

If you prefer to colocate routes and handlers, combine `@view_config` with `config.scan()`:

```python
from pyramid.config import Configurator
from pyramid.view import view_config

@view_config(route_name="api_status", renderer="json")
def api_status(request):
    return {"status": "ok", "path": request.path}

config = Configurator()
config.add_route("api_status", "/api/status")
config.scan()
app = config.make_wsgi_app()
```

### Read Settings

Application settings from your `.ini` file or explicit `settings={...}` are available via the registry:

```python
from pyramid.view import view_config

@view_config(route_name="info", renderer="json")
def info(request):
    settings = request.registry.settings
    return {"app_name": settings.get("app.name", "example")}
```

### Return JSON Or HTTP Responses

- Return a `Response` when you want to control status, headers, or body directly.
- Use `renderer="json"` for ordinary JSON APIs and return plain Python objects.

```python
from pyramid.response import Response
from pyramid.view import view_config

@view_config(route_name="created")
def created(request):
    return Response("created\n", status=201)
```

## Sessions, Security, And CSRF

### Sessions Are Opt-In

```python
import os

from pyramid.config import Configurator
from pyramid.session import SignedCookieSessionFactory

session_secret = os.environ["PYRAMID_SESSION_SECRET"]

config = Configurator()
config.set_session_factory(
    SignedCookieSessionFactory(
        session_secret,
        secure=True,
        httponly=True,
        samesite="Lax",
    )
)
```

Practical guidance:

- Use a long random secret from environment or secret management, not a checked-in constant.
- Keep `secure=True` in real deployments so session cookies only travel over HTTPS.
- Set `httponly=True` and an explicit `samesite` value unless you have a specific cross-site requirement.
- In `2.1`, `SignedCookieSessionFactory` now defaults to `hashalg="sha512"`. If you are upgrading from older defaults, existing signed cookies can stop validating unless you set the same algorithm on both sides of the rollout.

### Prefer Pyramid 2.x Security APIs

For Pyramid `2.1`, write view logic against the request-facing security API:

- `request.identity`
- `request.is_authenticated`
- `request.has_permission(...)`

Example guard in a view:

```python
from pyramid.httpexceptions import HTTPForbidden
from pyramid.view import view_config

@view_config(route_name="dashboard", renderer="json")
def dashboard(request):
    if not request.is_authenticated:
        raise HTTPForbidden()

    return {"authenticated": True}
```

Important notes:

- Your application still needs a security policy implementation that supplies identities and permission checks.
- New `2.x` code should not be based on the old split `authentication_policy` and `authorization_policy` style.

Do not start new code from older examples that rely on:

- `request.authenticated_userid`
- `request.effective_principals`
- separate `authentication_policy` and `authorization_policy` configuration
- the `Unauthenticated` principal

Those are part of the pre-2.0 security model and are deprecated or removed in the 2.x line.

### Enable CSRF For Unsafe Methods

If your app accepts browser form posts or other cookie-authenticated write requests, enable CSRF protection:

```python
from pyramid.config import Configurator
from pyramid.csrf import check_csrf_token
from pyramid.view import view_config

config = Configurator()
config.set_default_csrf_options(require_csrf=True)

@view_config(route_name="submit", request_method="POST", renderer="json")
def submit(request):
    check_csrf_token(request)
    return {"saved": True}
```

If you are building a JSON API that does not use browser cookies, do not cargo-cult CSRF checks into token-based machine-to-machine endpoints.

## Running And Deployment

For scaffolded apps, the standard development entry point is:

```bash
pserve development.ini --reload
```

For a direct WSGI app object, `waitress` is a common deployment choice:

```python
from waitress import serve

serve(app, listen="127.0.0.1:6543")
```

Pyramid is a WSGI framework. If the project expects native ASGI patterns, do not assume Pyramid code or middleware is interchangeable with FastAPI or Starlette examples.

## Common Pitfalls

### `pyramid` Is A Framework Core, Not A Full Stack Bundle

The base package gives you routing, views, requests, responses, configuration, sessions, and security hooks. Database integration, template engines, and authentication backends are typically added through separate packages or your own application code.

### Decorated Views Need `config.scan()`

If you register views with `@view_config` and forget `config.scan()`, Pyramid will not find them.

### Sessions And Authentication Do Nothing Until Configured

### Old Blog Posts Often Show Pre-2.0 Security Code

For `2.1`, prefer the unified security-policy model and the request properties listed above.

### `pcreate` Was Removed In 2.1

If you find old setup instructions that start with `pcreate`, they are stale for current `2.1` work. Use the Cookiecutter starter instead.

### The Stable Docs URL Is Official But Not Version-Pinned

`https://docs.pylonsproject.org/projects/pyramid/en/stable/` is a valid upstream landing page, but it will move as Pyramid releases advance. For version-sensitive behavior in this guide, use the `2.1-branch` pages listed below.

### Session Cookies Can Break Across A 2.1 Upgrade

The default signed-cookie hash changed to `sha512` in `2.1`. If an existing deployment depends on the older implicit default, make the hash algorithm explicit during the upgrade rather than assuming old cookies remain valid.

## Version-Sensitive Notes For 2.1

- PyPI lists `Requires-Python: >=3.10` for `2.1`.
- Pyramid `2.1` removed `pcreate`; scaffold new apps with the official Cookiecutter starter.
- The default `hashalg` for `SignedCookieSessionFactory` is now `sha512`.
- Pyramid `2.x` code should use `request.identity`, `request.is_authenticated`, and `request.has_permission(...)` instead of pre-2.0 request attributes and policy wiring patterns.

## Official Sources

- Stable docs landing page: https://docs.pylonsproject.org/projects/pyramid/en/stable/
- Version-pinned docs root used for 2.1 behavior: https://docs.pylonsproject.org/projects/pyramid/en/2.1-branch/
- Quick tutorial: https://docs.pylonsproject.org/projects/pyramid/en/2.1-branch/quick_tutorial.html
- Sessions: https://docs.pylonsproject.org/projects/pyramid/en/2.1-branch/narr/sessions.html
- Security: https://docs.pylonsproject.org/projects/pyramid/en/2.1-branch/narr/security.html
- What changed in 2.1: https://docs.pylonsproject.org/projects/pyramid/en/2.1-branch/whatsnew-2.1.html
- Official Cookiecutter starter: https://github.com/Pylons/pyramid-cookiecutter-starter
- PyPI package page: https://pypi.org/project/pyramid/

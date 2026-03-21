---
name: limiter
description: "Flask-Limiter package guide for Python Flask rate limiting, storage backends, CLI inspection, and 429 handling"
metadata:
  languages: "python"
  versions: "4.1.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask-limiter,flask,python,rate-limiting,redis,limits"
---

# Flask-Limiter Python Package Guide

## What It Is

`Flask-Limiter` adds request rate limiting to Flask apps. It supports:

- default limits applied to many routes
- route-specific limits
- shared buckets across multiple routes
- application-wide limits
- per-method limits
- meta limits for repeated breaches
- pluggable storage backends via `limits`

This entry is for `Flask-Limiter==4.1.1`, the current stable release on PyPI and the version tracked by the stable docs.

## Installation

Base install:

```bash
pip install Flask-Limiter==4.1.1
```

Common extras:

```bash
pip install "Flask-Limiter[redis]==4.1.1"
pip install "Flask-Limiter[valkey]==4.1.1"
pip install "Flask-Limiter[cli]==4.1.1"
```

Notes:

- Use a shared backend such as Redis or Valkey in production.
- `memory://` is fine for local development and tests, not for multi-process or multi-instance deployments.
- The `cli` extra is separate in `4.1.x`; it is no longer bundled into every install.

## Minimal Setup

Constructor-first initialization:

```python
from flask import Flask
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address

app = Flask(__name__)

limiter = Limiter(
    key_func=get_remote_address,
    app=app,
    default_limits=["200 per day", "50 per hour"],
    storage_uri="memory://",
)

@app.get("/slow")
@limiter.limit("1 per day")
def slow() -> str:
    return "slow"

@app.get("/medium")
@limiter.limit("1/second", override_defaults=False)
def medium() -> str:
    return "medium"

@app.get("/health")
@limiter.exempt
def health() -> str:
    return "ok"
```

Behavior:

- Route decorators override defaults unless `override_defaults=False`.
- `@limiter.exempt` skips default and application limits for that endpoint.
- `get_remote_address()` uses the request remote address, so proxy configuration matters.

## App Factory Setup

Use `init_app()` in app-factory projects:

```python
from flask import Flask
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address

limiter = Limiter(key_func=get_remote_address, default_limits=["100/minute"])

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["RATELIMIT_STORAGE_URI"] = "redis://localhost:6379/0"
    app.config["RATELIMIT_HEADERS_ENABLED"] = True
    limiter.init_app(app)
    return app
```

Constructor arguments win over equivalent Flask config values, so avoid setting the same option in both places unless that precedence is intentional.

## Production Storage Setup

Use a shared backend for real deployments:

```python
from flask import Flask
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address

app = Flask(__name__)
app.config.update(
    RATELIMIT_STORAGE_URI="redis://localhost:6379/0",
    RATELIMIT_STRATEGY="fixed-window",
    RATELIMIT_HEADERS_ENABLED=True,
)

limiter = Limiter(
    key_func=get_remote_address,
    app=app,
    default_limits=["100/minute"],
    application_limits=["1000/hour"],
    meta_limits=["20/hour"],
)
```

Practical choices:

- `fixed-window`: simplest and lowest overhead.
- `moving-window`: stricter behavior, more backend work.
- `sliding-window-counter`: approximation with lower cost than a full moving window.

## Core Usage Patterns

Per-route limit:

```python
@app.post("/login")
@limiter.limit("5/minute")
def login():
    ...
```

Keep defaults and add a tighter route limit:

```python
@app.post("/checkout")
@limiter.limit("10/minute", override_defaults=False)
def checkout():
    ...
```

Per-method limits:

```python
@app.route("/items", methods=["GET", "POST"])
@limiter.limit("20/minute", methods=["POST"], per_method=True)
def items():
    ...
```

Shared bucket across routes:

```python
write_limit = limiter.shared_limit("100/hour", scope="writes")

@app.post("/posts")
@write_limit
def create_post():
    ...

@app.post("/comments")
@write_limit
def create_comment():
    ...
```

Weighted requests:

```python
from flask import request

@app.post("/batch")
@limiter.limit("100/hour", cost=lambda: len(request.json.get("items", [])) or 1)
def batch():
    ...
```

Use `cost` only when one request can represent materially different backend load.

## Identity, Auth, And Proxies

`Flask-Limiter` does not authenticate users. Your app decides identity, then `key_func` decides the bucket key.

Authenticated-user buckets:

```python
from flask_login import current_user
from flask_limiter.util import get_remote_address

def rate_limit_key() -> str:
    if current_user.is_authenticated:
        return f"user:{current_user.get_id()}"
    return get_remote_address()
```

Proxy-safe setup:

```python
from werkzeug.middleware.proxy_fix import ProxyFix

app.wsgi_app = ProxyFix(app.wsgi_app, x_for=1, x_proto=1, x_host=1)
```

Important:

- If you are behind a load balancer or reverse proxy, configure proxy handling before relying on `get_remote_address()`.
- Without proxy fixups, all clients may collapse into the proxy IP and share one rate-limit bucket.
- If you have multi-tenant auth, key on tenant or account identifiers instead of raw IPs.

## 429 Handling

Custom JSON response:

```python
from flask import jsonify
from flask_limiter.errors import RateLimitExceeded

@app.errorhandler(429)
def ratelimit_handler(error: RateLimitExceeded):
    response = error.get_response()
    if response is not None:
        return response
    return jsonify(error="rate_limit_exceeded", detail=error.description), 429
```

If you use `on_breach`, keep the `429` error handler compatible with the embedded response by checking `error.get_response()` first.

## Useful Configuration Keys

High-signal Flask config keys for most projects:

- `RATELIMIT_ENABLED`: global on/off switch.
- `RATELIMIT_STORAGE_URI`: backend connection string such as `redis://...`.
- `RATELIMIT_STRATEGY`: `fixed-window`, `moving-window`, or `sliding-window-counter`.
- `RATELIMIT_DEFAULT`: default route limits.
- `RATELIMIT_APPLICATION`: app-wide shared limits.
- `RATELIMIT_META`: meta limits for repeated breaches.
- `RATELIMIT_HEADERS_ENABLED`: emits rate-limit response headers.
- `RATELIMIT_FAIL_ON_FIRST_BREACH`: stop evaluating additional limits after the first breach.
- `RATELIMIT_KEY_PREFIX`: namespace keys when multiple apps share one backend.
- `RATELIMIT_REQUEST_IDENTIFIER`: customize how requests are identified for some strategies and logging.

Use either constructor kwargs or config-first initialization consistently. Mixing both is valid, but it becomes harder to reason about precedence.

## CLI And Debugging

Install the CLI extra if you want the Flask CLI integration:

```bash
pip install "Flask-Limiter[cli]==4.1.1"
```

Useful commands:

```bash
flask limiter config
flask limiter limits
```

Use them to verify:

- the active backend and strategy
- default, application, and route limits actually registered
- whether your app factory or environment-specific config loaded the values you expected

## Common Pitfalls

- Using `memory://` in production. Counters stay local to one process.
- Relying on IP-based keys behind a proxy without `ProxyFix` or equivalent trusted-proxy setup.
- Forgetting that route decorators override defaults by default.
- Treating `Flask-Limiter` as an auth system. It only consumes request identity; it does not establish it.
- Swallowing backend failures without understanding the effect. If the backend is unavailable and you enable fallback or error swallowing, you may silently weaken protection.
- Copying older blog posts that still assume Python `3.8` or bundled CLI dependencies.

## Version-Sensitive Notes For 4.x

- `4.1.1` is the current stable release and matches the stable docs.
- `4.1.0` split extras so `cli` is installed separately; if you upgraded from older guides, add `[cli]` explicitly.
- `4.0.0` dropped support for Python `<3.10`, Flask `<2`, and Werkzeug `<2`.
- `4.0.0` also renamed the old `RATELIMIT_ENABLED` config key from the earlier `RATELIMIT_ENABLE` spelling.
- The stable docs are appropriate for `4.1.1`, but older `3.x` setups may differ in Python support, extras, and some config details.

## Official Sources

- Stable docs: https://flask-limiter.readthedocs.io/en/stable/
- Quick start and initialization: https://flask-limiter.readthedocs.io/en/stable/
- Configuration: https://flask-limiter.readthedocs.io/en/stable/configuration.html
- Recipes and proxy guidance: https://flask-limiter.readthedocs.io/en/stable/recipes.html
- CLI: https://flask-limiter.readthedocs.io/en/stable/cli.html
- Changelog: https://flask-limiter.readthedocs.io/en/stable/changelog.html
- PyPI package page: https://pypi.org/project/flask-limiter/
- PyPI version metadata JSON: https://pypi.org/pypi/Flask-Limiter/4.1.1/json

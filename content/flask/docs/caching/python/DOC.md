---
name: caching
description: "Flask-Caching package guide for Python: setup, backends, decorators, invalidation, and cache-key pitfalls"
metadata:
  languages: "python"
  versions: "2.3.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,caching,cachelib,redis,memcached,python,pypi"
---

# Flask-Caching Python Package Guide

`Flask-Caching` adds backend-agnostic caching to Flask applications through `cachelib`. Use it to cache view responses, memoize expensive helpers, cache Jinja fragments, and work with Redis, Memcached, filesystem, or in-process caches through one API.

## Install

Pin the package version you expect in application code, then add the backend driver you actually use.

```bash
pip install "Flask-Caching==2.3.1"
```

Optional backend clients:

```bash
pip install "Flask-Caching==2.3.1" redis
pip install "Flask-Caching==2.3.1" pylibmc
pip install "Flask-Caching==2.3.1" python-memcached
```

Notes:

- `Flask-Caching` wraps backends through `cachelib`.
- Redis and Memcached client libraries are separate dependencies.
- Pin the backend client too in production if cache behavior must stay stable across deploys.

## Initialize In Flask

### Direct initialization

```python
from flask import Flask
from flask_caching import Cache

app = Flask(__name__)
app.config.from_mapping(
    CACHE_TYPE="SimpleCache",
    CACHE_DEFAULT_TIMEOUT=300,
)

cache = Cache(app)
```

### App factory pattern

Prefer `init_app()` for larger applications.

```python
from flask import Flask
from flask_caching import Cache

cache = Cache()

def create_app() -> Flask:
    app = Flask(__name__)
    app.config.from_mapping(
        CACHE_TYPE="RedisCache",
        CACHE_DEFAULT_TIMEOUT=300,
        CACHE_REDIS_URL="redis://localhost:6379/0",
        CACHE_KEY_PREFIX="myapp:",
    )

    cache.init_app(app)
    return app
```

You can also pass a config mapping directly to `Cache(...)` or `cache.init_app(...)` when different cache instances need different backends.

## Backend Configuration And Credentials

Set `CACHE_TYPE` to the backend class name or a fully qualified import path.

Common built-in backends:

- `NullCache`: disable caching without removing decorators
- `SimpleCache`: in-process development cache
- `FileSystemCache`: local disk-backed cache
- `RedisCache`: Redis server or URL
- `RedisSentinelCache`: Redis Sentinel deployment
- `RedisClusterCache`: Redis Cluster deployment
- `MemcachedCache`, `SASLMemcachedCache`, `SpreadSASLMemcachedCache`

Use class-style backend names in 2.x code. Legacy lowercase aliases such as `simple` were removed in the 2.0 line.

### `SimpleCache` for local development

```python
app.config.from_mapping(
    CACHE_TYPE="SimpleCache",
    CACHE_DEFAULT_TIMEOUT=60,
    CACHE_THRESHOLD=500,
)
```

Use this only for single-process development or low-risk local workflows. It is process-local and the upstream docs describe it as not really thread safe.

### Filesystem cache

```python
app.config.from_mapping(
    CACHE_TYPE="FileSystemCache",
    CACHE_DIR="/tmp/flask-cache",
    CACHE_DEFAULT_TIMEOUT=300,
    CACHE_THRESHOLD=1000,
    CACHE_OPTIONS={"mode": 0o700},
)
```

`CACHE_OPTIONS["mode"]` controls file permissions for cache entries.

### Redis cache with URL-based config

```python
import os

app.config.from_mapping(
    CACHE_TYPE="RedisCache",
    CACHE_DEFAULT_TIMEOUT=300,
    CACHE_KEY_PREFIX="myapp:",
    CACHE_REDIS_URL=os.environ["CACHE_REDIS_URL"],
    CACHE_OPTIONS={
        "socket_timeout": 2,
        "socket_connect_timeout": 2,
        "retry_on_timeout": True,
    },
)
```

`CACHE_REDIS_URL` supports:

- `redis://`
- `rediss://` for TLS
- `unix://`

Prefer `CACHE_REDIS_URL` over spreading host, port, password, and database across multiple keys unless your deployment tooling requires that shape.

### Redis Sentinel

```python
app.config.from_mapping(
    CACHE_TYPE="RedisSentinelCache",
    CACHE_KEY_PREFIX="myapp:",
    CACHE_REDIS_SENTINELS=[
        ("redis-sentinel-1", 26379),
        ("redis-sentinel-2", 26379),
    ],
    CACHE_REDIS_SENTINEL_MASTER="mymaster",
    CACHE_REDIS_PASSWORD="secret",
    CACHE_REDIS_DB=0,
    CACHE_OPTIONS={"socket_timeout": 2},
)
```

### SASL Memcached

```python
app.config.from_mapping(
    CACHE_TYPE="SASLMemcachedCache",
    CACHE_MEMCACHED_SERVERS=["memcached-1:11211", "memcached-2:11211"],
    CACHE_MEMCACHED_USERNAME="cache-user",
    CACHE_MEMCACHED_PASSWORD="cache-pass",
    CACHE_KEY_PREFIX="myapp:",
)
```

If you need client-specific Memcached tuning, the docs note that plain `MemcachedCache` does not automatically pass `CACHE_OPTIONS` through to the client. Set those options on the underlying client after initialization if required.

## Core Usage

### Use the cache object directly

```python
from flask_caching import Cache

cache = Cache()

def get_profile(user_id: int) -> dict:
    key = f"user-profile:{user_id}"
    cached = cache.get(key)
    if cached is not None:
        return cached

    profile = load_profile_from_db(user_id)
    cache.set(key, profile, timeout=300)
    return profile
```

Useful methods include `get`, `set`, `add`, `delete`, `delete_many`, `has`, and `clear`.

### Cache view responses with `@cache.cached`

```python
from flask import Flask, jsonify, request
from flask_caching import Cache

app = Flask(__name__)
app.config["CACHE_TYPE"] = "SimpleCache"
cache = Cache(app)

@app.get("/items")
@cache.cached(timeout=60, query_string=True, source_check=True)
def list_items():
    return jsonify(fetch_items(request.args))
```

Key points:

- Default cache keys are based on `request.path`.
- Set `query_string=True` when query params change the response.
- `source_check=True` makes the decorated function's source part of the cache key, which helps when code changes but the URL does not.
- Put `@cache.cached(...)` below Flask's route decorator and directly above the function.

### Build a custom cache key for POST or body-sensitive routes

```python
import hashlib
import json

from flask import request

def make_post_cache_key() -> str:
    payload = request.get_json(silent=True) or {}
    raw = json.dumps(payload, sort_keys=True)
    digest = hashlib.sha256(raw.encode("utf-8")).hexdigest()
    return f"search:{digest}"

@app.post("/search")
@cache.cached(timeout=30, make_cache_key=make_post_cache_key)
def search():
    return run_search()
```

### Memoize helper functions with `@cache.memoize`

Use `memoize()` when function arguments should be part of the cache key.

```python
@cache.memoize(timeout=300)
def permission_check(user_id: int, role_id: str) -> bool:
    return expensive_permission_lookup(user_id, role_id)
```

### Ignore non-semantic arguments

If a function receives objects that should not affect the cache key, ignore them explicitly.

```python
@cache.memoize(timeout=300, args_to_ignore=["db_session"])
def load_report(report_id: str, db_session) -> dict:
    return build_report(report_id, db_session)
```

### Avoid caching bad responses

Use `response_filter` when you do not want to keep failures in cache.

```python
from flask import Response

def should_cache(response: Response) -> bool:
    return response.status_code < 500

@app.get("/dashboard")
@cache.cached(timeout=120, response_filter=should_cache)
def dashboard():
    return render_dashboard()
```

### Force refreshes on demand

Use `forced_update` when a call should refresh the cached value even if it has not expired yet.

```python
def refresh_requested() -> bool:
    return False

@cache.memoize(timeout=900, forced_update=refresh_requested)
def load_catalog() -> dict:
    return fetch_catalog()
```

## Invalidation And Template Fragments

### Delete memoized entries

```python
@cache.memoize(timeout=300)
def user_has_role(user_id: int, role: str) -> bool:
    return query_role(user_id, role)

cache.delete_memoized(user_has_role)
cache.delete_memoized(user_has_role, 42, "admin")
```

Rules that matter:

- Passing only the function clears all memoized variants.
- Passing arguments clears only the matching cache entry.
- For class methods, include the class as the first argument when deleting memoized values.

### Cache Jinja fragments

```jinja
{% cache 300, "sidebar", current_user.id %}
  {{ render_sidebar() }}
{% endcache %}
```

To invalidate that fragment from Python:

```python
from flask_caching import make_template_fragment_key

key = make_template_fragment_key("sidebar", vary_on=[current_user.id])
cache.delete(key)
```

## Common Pitfalls

### `SimpleCache` is process-local

It does not share state across Gunicorn workers, uWSGI workers, or multiple containers. Use Redis or Memcached for shared caches.

### Default view keys ignore query parameters

`@cache.cached` uses `request.path` by default. If `/items?page=1` and `/items?page=2` return different data, set `query_string=True` or build a custom key.

### `cached()` outside a request context still needs a stable `key_prefix`

If you decorate a helper with `cached()` and keep the default key behavior, Flask-Caching still expects request-path-based keys.

```python
@cache.cached(timeout=60, key_prefix="expensive:all-comments")
def get_all_comments():
    return load_comments()
```

### Decorator order matters

Use:

```python
@app.get("/")
@cache.cached(timeout=60)
def index():
    ...
```

Do not reverse the decorators.

### `cache_none=True` is usually the wrong tradeoff

The API docs warn that enabling it can return incorrect `None` results under concurrency. Prefer explicit sentinel values or a different cache design.

### Mutable objects in memoized arguments are risky

Memoization keys depend on argument values and representations. Mutable objects or unstable `repr()` output make cache hits and invalidation unpredictable.

### `cache.clear()` can be broader than expected

The docs warn that some backends do not support a clean scoped clear, and Redis-backed clears without a prefix can flush the whole configured database. Always set `CACHE_KEY_PREFIX` for shared backends.

## Version-Sensitive Notes For `2.3.1`

- The version used here `2.3.1` matches the current PyPI project page and GitHub release history.
- PyPI metadata for `2.3.1` lists `Python >=3.8`.
- The Read the Docs usage pages describe the current 2.x API surface, but the published changelog page currently stops at `2.1.0`. For 2.2.x and 2.3.x provenance, verify PyPI and GitHub releases instead of relying on the changelog page alone.
- `2.0.0` removed support for the old lowercase backend aliases such as `simple`, and aligned `FileSystemCache` behavior with `cachelib.FileSystemCache`. Old on-disk cache files from pre-2.0 deployments should not be assumed compatible.
- `2.1.0` added official Flask 3 support in the published changelog, so `2.3.1` is part of the maintained Flask 3-compatible 2.x line.
- If you are migrating very old code or `Flask-Cache` snippets, re-check backend names, config keys, and invalidation behavior instead of copying legacy examples directly.

## Recommended Agent Workflow

1. Configure the cache backend explicitly in Flask config, including a safe `CACHE_KEY_PREFIX` for shared backends.
2. Use `memoize()` for argument-sensitive helpers and `cached()` for route responses.
3. Decide cache keys deliberately for query params, POST bodies, tenant IDs, and auth-sensitive output.
4. Add invalidation where source data changes by using `delete`, `delete_many`, `delete_memoized`, or fragment-key deletion.
5. Avoid `SimpleCache` in multi-worker production deployments.

## Official Sources

- Documentation root: https://flask-caching.readthedocs.io/en/latest/
- Changelog: https://flask-caching.readthedocs.io/en/latest/changelog.html
- PyPI project: https://pypi.org/project/Flask-Caching/
- Source repository: https://github.com/pallets-eco/flask-caching
- Releases: https://github.com/pallets-eco/flask-caching/releases

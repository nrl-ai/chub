---
name: redis
description: "django-redis package guide for Python: Django cache backend, sessions, raw Redis access, Sentinel, and operational pitfalls"
metadata:
  languages: "python"
  versions: "6.0.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,redis,cache,sessions,sentinel,python,pypi"
---

# django-redis Python Package Guide

`django-redis` is the maintained Redis cache backend for Django. Use it when a Django project needs Redis-backed caching, cache-based sessions, or direct `redis-py` access from Django code.

## Version Baseline For 6.0.0

The 6.0.0 release notes and package metadata align on these minimums:

- Python: `>=3.9`
- Django: `>=4.2`
- `redis-py`: `>=4.0.2`
- Redis server: `>=2.8`

If your project is still on Django 4.1 or Python 3.8, do not copy this config verbatim. Upgrade planning is part of the migration.

## Install

```bash
pip install django-redis==6.0.0
```

Optional parser acceleration:

```bash
pip install hiredis
```

You also need a reachable Redis deployment:

- standalone Redis
- Redis over TLS
- Redis via Unix socket
- Redis Sentinel for failover

## Minimal Django Setup

Use `django_redis.cache.RedisCache` as the backend and point `LOCATION` at a Redis URL:

```python
# settings.py
CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": "redis://127.0.0.1:6379/1",
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.DefaultClient",
        },
    }
}
```

Supported connection styles from upstream docs:

- `redis://host:port/db` for plain TCP
- `rediss://host:port/db` for TLS
- `unix:///path/to/redis.sock?db=1` for a Unix socket

Only caches configured with `django_redis.cache.RedisCache` support `django-redis` helpers such as `ttl()`, `delete_pattern()`, and `get_redis_connection()`.

## Auth And Connection Configuration

You can put credentials directly in the URL or pass them in `OPTIONS`.

### URL-based auth

```python
CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": "redis://django:secret@redis.internal:6379/1",
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.DefaultClient",
        },
    }
}
```

### Username and password in `OPTIONS`

```python
CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": "redis://redis.internal:6379/1",
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.DefaultClient",
            "USERNAME": "django",
            "PASSWORD": "secret",
        },
    }
}
```

Practical options that show up often in real deployments:

```python
CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": "rediss://redis.internal:6379/1",
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.DefaultClient",
            "SOCKET_CONNECT_TIMEOUT": 5,
            "SOCKET_TIMEOUT": 5,
            "CONNECTION_POOL_KWARGS": {
                "max_connections": 100,
                "retry_on_timeout": True,
            },
            "SSL_CERT_REQS": None,
        },
    }
}
```

Notes:

- `SOCKET_CONNECT_TIMEOUT` covers initial connection setup.
- `SOCKET_TIMEOUT` covers read and write operations after the connection is open.
- `CONNECTION_POOL_KWARGS` is forwarded to the underlying `redis-py` connection pool.
- `SSL_CERT_REQS=None` disables certificate verification. Keep that limited to local development and controlled test environments.
- If the Redis URL already contains credentials, the URL wins over `OPTIONS["PASSWORD"]`.

## Core Cache Usage

Use Django's cache API for normal cache reads and writes:

```python
from django.core.cache import cache

cache.set("user:42", {"name": "Ada"}, timeout=300)
user = cache.get("user:42")

profile = cache.get_or_set(
    "profile:42",
    lambda: {"projects": 7},
    timeout=600,
)

created = cache.set("job:nightly", "queued", timeout=60, nx=True)
cache.incr("api:quota:used")
cache.delete("user:42")
```

Timeout behavior matters:

- `timeout=None` stores without expiration
- `timeout=0` expires immediately
- `cache.expire(key, seconds)` sets a TTL on an existing key
- `cache.persist(key)` removes an existing expiration

Redis-only helpers exposed by `django-redis`:

```python
from django.core.cache import cache

cache.set("counter", 1, timeout=60)
cache.incr("counter")

ttl = cache.ttl("counter")
pttl = cache.pttl("counter")
cache.expire("counter", 120)

for key in cache.iter_keys("tenant:42:*"):
    print(key)

cache.delete_pattern("tenant:42:*")
```

These methods are backend-specific. Do not assume they exist if a project swaps to Memcached or Django's local-memory cache.

## Raw Redis Access

When the Django cache API is too limited, use the underlying `redis-py` client:

```python
from django_redis import get_redis_connection

redis_client = get_redis_connection("default")

redis_client.hset("feature-flags", "new_dashboard", "on")
redis_client.sadd("active-tenants", "tenant-42")

flag = redis_client.hget("feature-flags", "new_dashboard")
tenants = redis_client.smembers("active-tenants")
```

This returns a `redis-py` client. Pipelines, Lua scripts, pub/sub, locks, and Redis data-structure commands all follow `redis-py` semantics, not Django cache semantics.

`get_redis_connection()` is only reliable with clients that expose raw-client access. If a project swaps in a heavily customized pluggable client, verify that helper still works.

## Locks

Use Redis locks for cross-process coordination:

```python
from django.core.cache import cache

with cache.lock("locks:nightly-report", timeout=30, blocking_timeout=5):
    run_report()
```

Always set a finite `timeout` so a crashed worker does not hold the lock forever.

## Sessions

For cache-backed sessions:

```python
# settings.py
SESSION_ENGINE = "django.contrib.sessions.backends.cache"
SESSION_CACHE_ALIAS = "default"
```

This is fast, but it inherits cache behavior:

- expired keys remove sessions
- Redis eviction removes sessions
- `FLUSHDB` or equivalent wipes sessions immediately

If login state must survive cache churn, use Django's database-backed session engine instead.

## Serialization And Compression

Upstream supports swapping serializers and compressors:

```python
CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": "redis://127.0.0.1:6379/1",
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.DefaultClient",
            "SERIALIZER": "django_redis.serializers.json.JSONSerializer",
            "COMPRESSOR": "django_redis.compressors.gzip.GzipCompressor",
        },
    }
}
```

Use JSON serialization only for JSON-safe values. If you cache arbitrary Python objects, keep the default pickle serializer or refactor the cached data shape.

Compression helps when cached values are large. It adds CPU cost, so measure before enabling it globally.

## Sentinel Setup

For Redis Sentinel, use the Sentinel client and connection factory:

```python
# settings.py
DJANGO_REDIS_CONNECTION_FACTORY = "django_redis.pool.SentinelConnectionFactory"

CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": "redis://service-name/1",
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.SentinelClient",
            "SENTINELS": [
                ("sentinel-1.internal", 26379),
                ("sentinel-2.internal", 26379),
                ("sentinel-3.internal", 26379),
            ],
            "CONNECTION_POOL_CLASS": "redis.sentinel.SentinelConnectionPool",
        },
    }
}
```

Treat Sentinel as failover infrastructure, not generic client-side load balancing. Keep write traffic on the primary unless you have explicitly designed and tested replica reads.

## Operational Flags

To emulate Django's memcached-style behavior where cache failures degrade to cache misses:

```python
CACHES = {
    "default": {
        "BACKEND": "django_redis.cache.RedisCache",
        "LOCATION": "redis://127.0.0.1:6379/1",
        "OPTIONS": {
            "CLIENT_CLASS": "django_redis.client.DefaultClient",
            "IGNORE_EXCEPTIONS": True,
        },
    }
}
```

Use this only intentionally. It hides Redis outages and can make production failures look like ordinary cache misses.

Useful companion settings:

- `DJANGO_REDIS_LOG_IGNORED_EXCEPTIONS = True`
- `DJANGO_REDIS_LOGGER = "your.project.logger"`
- `DJANGO_REDIS_CLOSE_CONNECTION = True` if commands, workers, or tests need explicit connection teardown

You can also set `CLOSE_CONNECTION: True` per cache alias.

## Common Pitfalls

- Do not call `get_redis_connection()` before Django settings are loaded.
- Do not assume Redis-only helpers such as `ttl()`, `lock()`, `iter_keys()`, or `delete_pattern()` work on non-Redis backends.
- Do not use `cache.keys("*")` on large datasets. Prefer `iter_keys()` or `delete_pattern()` and tune `DJANGO_REDIS_SCAN_ITERSIZE` if needed.
- Do not use cache-backed sessions if Redis eviction, flushes, or cache-only retention policies are unacceptable for auth state.
- Do not set `IGNORE_EXCEPTIONS=True` unless silent cache degradation is an explicit product decision.
- Do not disable TLS certificate verification outside controlled development environments.
- Do not assume old blog posts match current support requirements. `6.0.0` raises the supported Python and Django floor.

## Version-Sensitive Notes For 6.0.0

This entry targets `django-redis==6.0.0`, released on `2025-06-17`.

Compared with the 5.x line, the upstream 6.0.0 changelog calls out these relevant changes:

- support added for Python `3.13`
- support added for Django `5.2`
- support dropped for Python `3.8`
- support dropped for Django `4.1`
- support added for hash map and set operations
- `gzip` compression support added
- `cache.lock()` gained a `blocking` parameter

That means older deployment examples targeting Python 3.8 or Django 4.1 are stale for this version. If you are upgrading from `5.4.0`, re-check your runtime matrix, lock behavior, and serializer/compressor choices before rolling the config forward.

## Official Sources Used

- GitHub repository: https://github.com/jazzband/django-redis
- Upstream README: https://raw.githubusercontent.com/jazzband/django-redis/master/README.rst
- Upstream changelog: https://raw.githubusercontent.com/jazzband/django-redis/master/CHANGELOG.rst
- PyPI package page for the version covered here: https://pypi.org/project/django-redis/6.0.0/

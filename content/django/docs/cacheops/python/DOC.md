---
name: cacheops
description: "django-cacheops package guide for Python - Redis-backed Django ORM caching with automatic invalidation"
metadata:
  languages: "python"
  versions: "7.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,cacheops,redis,caching,orm,queryset"
---

# django-cacheops Python Package Guide

## What It Is

`django-cacheops` adds Redis-backed caching for Django ORM queries plus helper decorators for functions, views, and template fragments. Its main feature is granular invalidation: cache entries are tied to the models and query conditions that produced them, so writes usually invalidate the affected cached data automatically.

Use it when you want cached queryset results without hand-managing cache keys for every read path.

## Version Scope

- Ecosystem: `pypi`
- Package: `django-cacheops`
- Version covered here: `7.2`
- Registry URL: https://pypi.org/project/django-cacheops/
- Upstream docs and source: https://github.com/Suor/django-cacheops

Version-sensitive notes for `7.2`:

- PyPI `7.2` was released on April 20, 2025.
- The project description for `7.2` documents practical support as Python `3.8+`, Django `3.2+`, and Redis `4.0+`.
- PyPI metadata still reports `Requires-Python >=3.7`, but the published classifiers for `7.2` are Python `3.8` through `3.13`. Treat Python `3.8+` as the supported baseline when writing code for this version.

## Installation

```bash
pip install django-cacheops==7.2
```

```bash
poetry add django-cacheops@7.2
```

```bash
uv add django-cacheops==7.2
```

Imports come from `cacheops`, not `django_cacheops`:

```python
from cacheops import cached_as, cached_view_as, invalidate_obj
```

## Minimal Django Setup

Add the app:

```python
INSTALLED_APPS = [
    # ...
    "cacheops",
]
```

Configure Redis plus model-level cache profiles in `settings.py`:

```python
CACHEOPS_REDIS = {
    "host": "127.0.0.1",
    "port": 6379,
    "db": 1,
    "socket_timeout": 3,
}

CACHEOPS_DEFAULTS = {
    "timeout": 60 * 60,
}

CACHEOPS = {
    "auth.user": {"ops": "get", "timeout": 15 * 60},
    "blog.post": {"ops": {"fetch", "get"}, "timeout": 10 * 60},
    "blog.category": {"ops": "all", "timeout": 60 * 60},
    "*.*": {},
}
```

Practical setup rules:

- Keep `*.*` present but empty unless you intentionally want global automatic caching.
- Start with a few read-heavy models and narrow `ops`; expand only after measuring hit rate and invalidation behavior.
- Use a dedicated Redis DB or instance if you can. Cacheops stores extra invalidation data, not just plain values.

## Redis Configuration

`CACHEOPS_REDIS` accepts either a dict or a URL:

```python
CACHEOPS_REDIS = "redis://localhost:6379/1"
```

Password in the URL:

```python
CACHEOPS_REDIS = "redis://:password@localhost:6379/1"
```

Unix socket:

```python
CACHEOPS_REDIS = "unix:///var/run/redis/redis.sock?db=1"
```

Redis Sentinel:

```python
CACHEOPS_SENTINEL = {
    "locations": [("127.0.0.1", 26379)],
    "service_name": "mymaster",
    "socket_timeout": 0.1,
    "db": 0,
}
```

Custom Redis client class:

```python
CACHEOPS_CLIENT_CLASS = "path.to.CustomRedis"
```

If Redis failures should not take down the app:

```python
CACHEOPS_DEGRADE_ON_FAILURE = True
```

## Cache Profile Semantics

Supported ORM operation groups are:

- `get`
- `fetch`
- `count`
- `aggregate`
- `exists`

`"all"` enables all five.

Useful model options:

- `timeout`: cache TTL in seconds
- `ops`: which ORM operations are cached
- `local_get`: keep simple `.get()` results in local process memory
- `cache_on_save`: write a saved object directly into the cache for selected lookups
  Example values from upstream docs are `True` or a field name such as `"slug"`.
  This is useful when most reads are immediate lookups right after writes.
- `db_agnostic`: whether cache keys should ignore the current DB alias

Examples:

```python
CACHEOPS = {
    "blog.post": {"ops": {"fetch", "get", "count"}, "timeout": 900},
    "blog.category": {"ops": "all", "timeout": 3600},
    "blog.article": {"ops": "get", "local_get": True, "timeout": 300},
    "blog.tag": {"ops": "get", "cache_on_save": "slug", "timeout": 600},
}
```

Use `local_get` only for models that almost never change. It bypasses Redis for simple gets within the same process, so stale reads can last until process restart.

## Core Usage

### Automatic queryset caching

Once a model is configured in `CACHEOPS`, supported queryset operations are cached automatically:

```python
posts = Post.objects.filter(published=True)
total = posts.count()
first_post = Post.objects.get(pk=1)
```

### Manual queryset caching

Use `.cache()` when you want to opt in on a specific queryset or override defaults:

```python
posts = Post.objects.filter(published=True).cache()
```

Cache only specific operations:

```python
qs = Post.objects.filter(category_id=category_id).cache(ops=["count"])
```

Override timeout:

```python
qs = Post.objects.filter(category_id=category_id).cache(timeout=300)
```

Disable cache for one queryset:

```python
qs = Post.objects.filter(published=True).nocache()
```

### Cache derived data with model-aware invalidation

Use `@cached_as()` when the result depends on one or more models:

```python
from django.db.models import Count
from cacheops import cached_as

@cached_as(Post, timeout=120)
def post_stats():
    return list(
        Post.objects.values("category_id").annotate(total=Count("id"))
    )
```

If invalidation should depend on a filtered queryset, pass that queryset and any extra key inputs:

```python
from cacheops import cached_as

def latest_posts(category_id: int, limit: int = 10):
    base_qs = Post.objects.filter(category_id=category_id)

    @cached_as(base_qs, extra=limit, timeout=300)
    def _load():
        return list(base_qs.order_by("-published_at")[:limit])

    return _load()
```

Materialize lazy querysets inside the cached function with `list()`, `tuple()`, or a concrete value. Do not return an unevaluated queryset object.

### View caching

Use `@cached_view_as()` when a Django view should be invalidated by model changes:

```python
from cacheops import cached_view_as
from django.shortcuts import render

@cached_view_as(Post, extra=lambda request: request.user.is_staff, timeout=300)
def post_index(request):
    posts = Post.objects.filter(published=True)
    return render(request, "posts/index.html", {"posts": posts})
```

### Time-based caching without ORM invalidation

Use `@cached()` or the low-level cache object for non-ORM values:

```python
from cacheops import CacheMiss, cache, cached

@cached(timeout=300)
def expensive_value(key: str):
    return compute_value(key)

def get_cached_value(key: str):
    try:
        return cache.get(key)
    except CacheMiss:
        value = compute_value(key)
        cache.set(key, value, timeout=300)
        return value
```

## Invalidation

Cacheops automatically invalidates cached querysets when tracked model data changes, but manual helpers are still useful:

```python
from cacheops import invalidate_all, invalidate_model, invalidate_obj

invalidate_obj(post)
invalidate_model(Post)
invalidate_all()
```

When bulk updates would bypass normal model save hooks, use cacheops-aware helpers:

```python
Post.objects.filter(published=False).invalidated_update(published=True)
```

If you intentionally want to suppress invalidation during a batch operation, use `no_invalidation` and then invalidate explicitly:

```python
from cacheops import invalidate_model, no_invalidation

with no_invalidation:
    Post.objects.filter(status="draft").update(status="published")

invalidate_model(Post)
```

## Template Fragments

Enable the template tag library:

```python
INSTALLED_APPS = [
    # ...
    "cacheops",
]
```

Example:

```django
{% load cacheops %}

{% cached_as sidebar_queryset 600 "sidebar" request.user.pk %}
  {% include "includes/sidebar.html" %}
{% endcached_as %}
```

Use this when the rendered fragment depends on queryset results and should invalidate with the underlying model data.

## Transactions, Locking, and Consistency

Cacheops disables caching once a transaction becomes dirty and keeps it disabled until the outer transaction commits. This avoids reading cache entries that no longer match in-flight writes.

For heavy concurrent recomputation, upstream documents a lock-based strategy to reduce dog-pile effects. The general rule is:

- Keep cached functions deterministic.
- Avoid expensive recomputation in hot paths unless you set reasonable TTLs.
- Expect some stale reads during races; cacheops optimizes for practical correctness, not cross-request serializability.

## Operational Settings

Useful global settings from upstream:

```python
CACHEOPS_PREFIX = lambda query: "tenant:%s" % get_current_tenant_id()

CACHEOPS_ENABLED = not DEBUG

CACHEOPS_INSIDEOUT = True
```

Notes:

- `CACHEOPS_PREFIX` is the main namespacing hook for multi-tenant deployments or shared Redis instances.
- `CACHEOPS_ENABLED = False` is the simplest way to disable cacheops in tests or local debugging.
- `CACHEOPS_INSIDEOUT = True` is recommended by upstream when Redis eviction policy is `volatile-*`; it stores checksum data separately so invalidation survives better under eviction pressure.

## Common Pitfalls

- Import path mismatch: install `django-cacheops`, import from `cacheops`.
- Returning lazy querysets from cached helpers leads to incorrect cache contents. Materialize results before returning.
- Plain `QuerySet.update()` can bypass the invalidation semantics you expect. Prefer `invalidated_update()` when cached data depends on those rows.
- `select_related()` and `prefetch_related()` do not automatically make invalidation understand every downstream relation shape. Cache the exact queryset you read, and test invalidation around relation-heavy screens.
- `local_get=True` trades correctness for speed. Do not use it on frequently updated models.
- Shared Redis with aggressive eviction can break assumptions unless you configure memory policy and, for newer setups, consider `CACHEOPS_INSIDEOUT`.
- During tests, cache state can hide bugs. Disable cacheops or flush Redis between tests that assert query counts or invalidation behavior.

## Version-Sensitive Notes For Agents

- The target version for this session is `7.2`, and PyPI latest is also `7.2` as of March 12, 2026.
- Older material for `7.1` can mention Python `>=3.7`; for `7.2`, the project description and classifiers move practical support to Python `3.8+`.
- The upstream repo README tracks the current default branch. For release-sensitive compatibility questions, prefer the PyPI release page first, then use the repo README for API shape and examples.
- `django-cacheops` is configuration-heavy. Before writing production code, inspect the project’s actual `CACHEOPS`, Redis topology, and test strategy instead of assuming the defaults above are safe.

## Official Sources

- PyPI package page: https://pypi.org/project/django-cacheops/
- PyPI `7.2` release page: https://pypi.org/project/django-cacheops/7.2/
- Upstream repository: https://github.com/Suor/django-cacheops

---
name: package
description: "cachetools package guide for in-memory caching and memoization in Python projects"
metadata:
  languages: "python"
  versions: "7.0.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "cachetools,python,caching,memoization,lru,ttl"
---

# cachetools Python Package Guide

## Golden Rule

Use `cachetools` for in-process caching and memoization, but treat its cache objects as local mutable state, not shared infrastructure. Cache classes are not thread-safe by default, TTL-based entries expire lazily, and memoization keys must match your function's real argument semantics.

## Install

Pin the version your project expects:

```bash
python -m pip install "cachetools==7.0.5"
```

Common alternatives:

```bash
uv add "cachetools==7.0.5"
poetry add "cachetools==7.0.5"
```

Typing note:

```bash
python -m pip install "types-cachetools"
```

`types-cachetools` is a separate stub package published on PyPI. Use it only if your type checker needs stubs that are not already bundled in your environment.

## Setup And Configuration

`cachetools` is an in-memory library. There is no auth layer, no service endpoint, and no required environment variables.

The important configuration choices are:

- Which eviction policy fits the workload: `LRUCache`, `LFUCache`, `FIFOCache`, `RRCache`, `TTLCache`, or `TLRUCache`
- What `maxsize` means for your data
- Whether you need a custom `getsizeof` function
- Whether access crosses threads and therefore needs a `lock`
- Whether memoized calls need a custom key function

All cache classes derive from `Cache`. `maxsize` must be positive. If you want "unbounded", use `math.inf`.

## Core Usage

### Use a concrete cache directly

```python
from cachetools import LRUCache

cache = LRUCache(maxsize=1024)

cache["user:1"] = {"id": 1, "name": "Ada"}
user = cache.get("user:1")

if user is None:
    user = {"id": 1, "name": "Ada"}
    cache["user:1"] = user
```

Choose the cache class by eviction behavior:

- `LRUCache`: evict least recently used items
- `LFUCache`: evict least frequently used items
- `FIFOCache`: evict oldest inserted items
- `RRCache`: evict a random item
- `TTLCache`: `LRUCache` plus per-item expiration
- `TLRUCache`: time-aware eviction with a custom time-to-use function

### Memoize a function with `@cached`

```python
from cachetools import LRUCache, cached

cache = LRUCache(maxsize=512)

@cached(cache)
def load_user(user_id: int) -> dict:
    print("cache miss")
    return {"id": user_id}

load_user(1)
load_user(1)  # second call hits cache
```

Use `info=True` when you want cache statistics:

```python
from cachetools import LRUCache, cached

@cached(cache=LRUCache(maxsize=128), info=True)
def square(value: int) -> int:
    return value * value

square(2)
square(2)
print(square.cache_info())
```

### Add TTL-based expiration

```python
from cachetools import TTLCache, cached

cache = TTLCache(maxsize=256, ttl=300)

@cached(cache)
def fetch_settings(account_id: str) -> dict:
    return {"account_id": account_id, "feature_enabled": True}
```

`TTLCache` removes expired entries only on the next mutating operation or when you call `expire()`. If memory pressure matters, explicitly clear expired items:

```python
cache.expire()
```

### Memoize instance methods with `@cachedmethod`

```python
from cachetools import TTLCache, cachedmethod

class UserService:
    def __init__(self) -> None:
        self.cache = TTLCache(maxsize=256, ttl=60)

    @cachedmethod(lambda self: self.cache)
    def get_profile(self, user_id: int) -> dict:
        print("cache miss")
        return {"id": user_id, "name": f"user-{user_id}"}
```

This pattern is usually better than a module-global cache because each instance controls its own cache size and lifecycle.

### Use stable keys for shared or complex memoization

By default, positional and keyword arguments must be hashable, and different argument shapes may produce different cache keys. Use explicit key functions when that matters:

```python
from cachetools import LRUCache, cached
from cachetools.keys import hashkey, typedkey

cache = LRUCache(maxsize=128)

@cached(cache=cache, key=lambda query, limit=20: hashkey(query.casefold(), limit))
def search(query: str, limit: int = 20) -> list[str]:
    return [query] * limit

typed_cache = LRUCache(maxsize=128)

@cached(cache=typed_cache, key=typedkey)
def normalize(value):
    return str(value)
```

`typedkey` treats `3` and `3.0` as different keys. Use it when type distinctions matter.

### Share one cache across functions safely

If multiple functions use the same cache object, give each function a distinct key prefix:

```python
from cachetools import LRUCache, cached
from cachetools.keys import hashkey

shared_cache = LRUCache(maxsize=256)

@cached(shared_cache, key=lambda user_id: hashkey("user", user_id))
def get_user(user_id: int) -> dict:
    return {"id": user_id}

@cached(shared_cache, key=lambda team_id: hashkey("team", team_id))
def get_team(team_id: int) -> dict:
    return {"id": team_id}
```

Without a prefix, unrelated functions can collide if they receive the same arguments.

## Threading And Stampede Control

Cache classes are not thread-safe. If the cache is shared across threads, wrap access with a lock through the memoizing decorators:

```python
from threading import RLock

from cachetools import LRUCache, cached

cache = LRUCache(maxsize=256)
lock = RLock()

@cached(cache=cache, lock=lock)
def fetch_product(product_id: int) -> dict:
    return {"id": product_id}
```

`lock` only protects cache access. The wrapped function itself runs outside the lock.

If you need cache stampede protection for concurrent callers of the same function, use the decorator's `condition` support:

```python
from threading import Condition

from cachetools import TTLCache, cached

cache = TTLCache(maxsize=128, ttl=30)
condition = Condition()

@cached(cache=cache, condition=condition)
def fetch_config(name: str) -> dict:
    return {"name": name}
```

This makes identical concurrent cache misses wait instead of recomputing the same value repeatedly.

## Size Accounting With `getsizeof`

By default, cache size counts items, not payload size. If cache entries vary widely in size, pass `getsizeof`:

```python
from cachetools import LRUCache

cache = LRUCache(
    maxsize=10_000_000,
    getsizeof=lambda value: len(value["payload"]),
)
```

`getsizeof` runs only when an item is inserted. If you mutate cached values later, the recorded size does not automatically change.

## Common Pitfalls

- Cache instances are not thread-safe. Use decorator `lock` support or your own synchronization when caches cross threads.
- `TTLCache` expiration is lazy. Expired items may still occupy memory until the next mutation or an explicit `expire()`.
- Function arguments used in memoization keys must be hashable unless you normalize them yourself.
- Keyword ordering and default argument handling can change the key shape. If semantic equivalence matters, define an explicit `key=` function.
- Sharing one cache object across multiple functions without namespaced keys can create collisions.
- `getsizeof` is evaluated on insertion only. Mutating cached values can invalidate your size accounting.
- `math.inf` is supported for unbounded growth, but an unbounded in-process cache can still exhaust memory.

## Version-Sensitive Notes For 7.0.x

- `cachetools 7.0.5` on PyPI requires Python `>=3.10`.
- `7.0.5` includes minor `@cachedmethod` performance improvements, and `7.0.4` fixed and documented `@cachedmethod.cache_key` behavior.
- In `7.0.0`, `cachedmethod()` changed in ways that break some older examples: returning `None` from `cache(self)` is no longer supported, cache-related decorator attributes became properties, and using `cachedmethod()` together with `classmethod()` is deprecated.
- If you copy examples written for `cachetools 5.x` or `6.x`, re-check method-level caching and decorator attribute access against the `7.x` docs before reusing them unchanged.

## Official Sources Used

- Docs: https://cachetools.readthedocs.io/en/latest/
- API docs: https://cachetools.readthedocs.io/en/latest/#cachetools
- PyPI: https://pypi.org/project/cachetools/
- Changelog: https://github.com/tkem/cachetools/blob/master/CHANGELOG.rst

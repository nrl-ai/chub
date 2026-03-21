---
name: package
description: "arq Python package guide for async Redis-backed job queues and workers"
metadata:
  languages: "python"
  versions: "0.27.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "arq,python,asyncio,redis,queue,workers,background-jobs"
---

# arq Python Package Guide

## Golden Rule

Use `arq` for async Redis-backed background jobs only if your application is already asyncio-native and you are comfortable with Redis as the queue backend. Define worker functions as `async def`, run them through a `WorkerSettings` class, and make every job idempotent because `arq` intentionally prefers rerunning a job over dropping it during shutdown or cancellation.

`arq` is in maintenance-only mode as of 2026, so avoid building new code around old blog posts or pre-`0.16` examples without checking the current docs first.

## Install

Pin the package version your project expects:

```bash
python -m pip install "arq==0.27.0"
```

Common alternatives:

```bash
uv add "arq==0.27.0"
poetry add "arq==0.27.0"
```

If you want worker auto-reload in development:

```bash
python -m pip install "arq[watch]==0.27.0"
```

That installs the optional `watchfiles` dependency used by `arq ... --watch`.

## Core Model

`arq` has two sides:

- An async producer that connects to Redis and enqueues jobs with `create_pool(...)`
- One or more workers that load a `WorkerSettings` class and execute registered functions

The producer and worker must agree on:

- function names
- queue name
- serializer/deserializer
- Redis connection settings

## Initialize And Run

### Minimal setup

```python
import asyncio

from arq import create_pool
from arq.connections import RedisSettings

REDIS_SETTINGS = RedisSettings(host="localhost", port=6379, database=0)

async def send_email(ctx, user_id: int) -> str:
    return f"sent email to user {user_id}"

class WorkerSettings:
    functions = [send_email]
    redis_settings = REDIS_SETTINGS

async def main() -> None:
    redis = await create_pool(REDIS_SETTINGS)
    await redis.enqueue_job("send_email", 123)

if __name__ == "__main__":
    asyncio.run(main())
```

Enqueue jobs:

```bash
python demo.py
```

Run the worker:

```bash
arq demo.WorkerSettings
```

Useful worker modes:

```bash
arq demo.WorkerSettings --burst
arq demo.WorkerSettings --watch path/to/src
```

Use `--burst` for one-shot workers in scripts, CI, or short-lived containers.

### Shared startup and shutdown state

Use `on_startup` and `on_shutdown` to create expensive shared objects once per worker process:

```python
from httpx import AsyncClient

async def startup(ctx) -> None:
    ctx["http"] = AsyncClient(timeout=10.0)

async def shutdown(ctx) -> None:
    await ctx["http"].aclose()

async def fetch_url(ctx, url: str) -> int:
    response = await ctx["http"].get(url)
    response.raise_for_status()
    return len(response.text)

class WorkerSettings:
    functions = [fetch_url]
    on_startup = startup
    on_shutdown = shutdown
    redis_settings = REDIS_SETTINGS
```

Use `ctx` for shared clients, connection pools, executors, and feature flags. Do not recreate them inside every job unless that is intentional.

## Redis Configuration And Auth

`RedisSettings` is the main configuration object for both producers and workers.

Common fields:

- `host`, `port`, `database`
- `username`, `password`
- `ssl` and related TLS certificate fields
- `max_connections`
- `sentinel` and `sentinel_master`
- retry controls such as `retry_on_timeout`, `retry_on_error`, and `retry`

Environment-driven example:

```python
import os

from arq.connections import RedisSettings

REDIS_SETTINGS = RedisSettings(
    host=os.getenv("REDIS_HOST", "localhost"),
    port=int(os.getenv("REDIS_PORT", "6379")),
    database=int(os.getenv("REDIS_DB", "0")),
    username=os.getenv("REDIS_USERNAME") or None,
    password=os.getenv("REDIS_PASSWORD") or None,
    ssl=os.getenv("REDIS_SSL", "").lower() == "true",
)
```

If you already have a Redis DSN, use `RedisSettings.from_dsn(...)`:

```python
from arq.connections import RedisSettings

REDIS_SETTINGS = RedisSettings.from_dsn(
    "redis://username:password@redis.example.com:6379/0"
)
```

## Core Usage Patterns

### Enqueue a job

```python
redis = await create_pool(REDIS_SETTINGS)
job = await redis.enqueue_job("send_email", 123)
```

`enqueue_job()` returns a `Job` object, or `None` if a job with the same `_job_id` already exists.

### Enforce uniqueness with `_job_id`

Use a stable `_job_id` when duplicate work would be harmful:

```python
job = await redis.enqueue_job(
    "send_email",
    123,
    _job_id="send-email:user:123",
)
```

This is the cleanest way to deduplicate retries triggered by upstream callers or repeated button clicks.

### Defer execution

```python
from datetime import datetime, timedelta, timezone

await redis.enqueue_job("send_email", 123, _defer_by=timedelta(minutes=5))

await redis.enqueue_job(
    "send_email",
    123,
    _defer_until=datetime.now(timezone.utc) + timedelta(hours=1),
)
```

### Get status and results

```python
job = await redis.enqueue_job("send_email", 123)

status = await job.status()
info = await job.info()
result = await job.result(timeout=30)
```

Relevant statuses include `deferred`, `queued`, `in_progress`, `complete`, and `not_found`.

### Retry with backoff

Raise `arq.Retry` from a job to reschedule it:

```python
from arq import Retry

async def fetch_invoice(ctx, invoice_id: str) -> dict:
    response = await ctx["http"].get(f"https://api.example.com/invoices/{invoice_id}")
    if response.status_code == 503:
        raise Retry(defer=ctx["job_try"] * 5)
    response.raise_for_status()
    return response.json()
```

Tune retry limits per function or worker:

```python
from arq.worker import func

class WorkerSettings:
    functions = [
        func(fetch_invoice, max_tries=10, timeout=60, keep_result=300),
    ]
```

### Schedule cron jobs

Use `cron(...)` in worker settings for recurring jobs:

```python
from arq.cron import cron

async def refresh_cache(ctx) -> None:
    ...

class WorkerSettings:
    functions = []
    cron_jobs = [
        cron(refresh_cache, minute={0, 15, 30, 45}, run_at_startup=True),
    ]
```

Cron jobs are unique by default, which is important when multiple workers are running.

### Run blocking work safely

Worker functions must be coroutine functions. If the real work is blocking, offload it to an executor:

```python
import asyncio
from concurrent.futures import ProcessPoolExecutor
from functools import partial

def resize_image_sync(path: str) -> str:
    ...

async def startup(ctx) -> None:
    ctx["pool"] = ProcessPoolExecutor()

async def resize_image(ctx, path: str) -> str:
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(ctx["pool"], partial(resize_image_sync, path))
```

## Worker Tuning

Useful `WorkerSettings` options to reach for first:

- `queue_name`: separate workloads into different queues
- `max_jobs`: concurrent jobs per worker, default `10`
- `job_timeout`: default job runtime limit, default `300` seconds
- `max_tries`: default retry limit, default `5`
- `keep_result`: how long to retain results, default `3600` seconds
- `keep_result_forever`: keep results indefinitely when needed
- `poll_delay`: queue polling interval
- `allow_abort_jobs`: required for `job.abort()` to work
- `health_check_interval` and `health_check_key`: useful for liveness monitoring
- `burst`: stop after the queue drains

If you change `queue_name`, the producer must enqueue to the same queue via `_queue_name` or the worker setting.

## Serialization

By default, `arq` uses `pickle`. If you switch to a custom serializer such as MsgPack, configure it on both sides:

```python
import msgpack

from arq import create_pool

redis = await create_pool(
    REDIS_SETTINGS,
    job_serializer=msgpack.packb,
    job_deserializer=lambda b: msgpack.unpackb(b, raw=False),
)

class WorkerSettings:
    functions = [send_email]
    redis_settings = REDIS_SETTINGS
    job_serializer = msgpack.packb
    job_deserializer = lambda b: msgpack.unpackb(b, raw=False)
```

If the worker and producer serializers differ, queued jobs will not deserialize correctly.

## Common Pitfalls

- Jobs are not exactly-once. Design them to tolerate reruns after worker shutdown, cancellation, or retries.
- Old `arq <= 0.15` examples are not compatible with `0.27.0`. The project was fully rewritten in `0.16`.
- Worker functions must be `async def`. Wrap blocking CPU or file work in an executor instead of calling it directly in the event loop.
- `enqueue_job()` deduplicates only when you provide `_job_id`. Without it, repeated enqueue calls create separate jobs.
- `job.abort()` does nothing unless the worker has `allow_abort_jobs = True`.
- Producers and workers must use the same serializer/deserializer pair and compatible payload shapes.
- If you defer jobs or keep results for a long time, Redis memory usage becomes part of your queue design.
- `arq` only processes jobs while a worker is actually running. Enqueuing alone is not enough.

## Version-Sensitive Notes For `0.27.0`

- `0.27.0` supports Python `3.9` through `3.13` and no longer supports Python `3.8`.
- The `v0.16` rewrite is still the main compatibility boundary. Pre-`0.16` docs use a different actor-based API and should be treated as historical only.
- The project README now marks `arq` as maintenance-only mode. Expect bug fixes and compatibility updates, but be cautious about depending on future feature work.

## Official Sources

- Docs: https://arq-docs.helpmanual.io/
- PyPI: https://pypi.org/project/arq/
- Source repository: https://github.com/python-arq/arq
- Release history: https://github.com/python-arq/arq/releases

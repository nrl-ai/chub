---
name: package
description: "Huey Python task queue guide for background jobs, scheduling, retries, and Django integration"
metadata:
  languages: "python"
  versions: "2.6.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "huey,python,task-queue,background-jobs,scheduling,redis,django"
---

# Huey Python Package Guide

## Golden Rule

Use one shared `Huey` instance per queue, make sure the consumer imports every decorated task before it starts working, and run a real consumer process outside tests. Most production deployments should use a Redis-backed queue; the main "configuration" surface in Huey is backend and consumer setup, not application-level auth.

## Install

Core install:

```bash
python -m pip install "huey==2.6.0"
```

If you are using Redis, install the backend extra so the Redis client dependency is present:

```bash
python -m pip install "huey[backends]==2.6.0"
```

Common alternatives:

```bash
uv add "huey==2.6.0"
poetry add "huey==2.6.0"
```

Notes:

- `RedisHuey` is the default recommendation for real async workloads.
- `SqliteHuey` is simpler operationally and can be good for smaller single-host deployments.
- `FileHuey` and `MemoryHuey` are mainly useful for local development and tests.

## Initialize A Queue

Minimal Redis-backed setup:

```python
import os

from huey import RedisHuey, crontab

huey = RedisHuey(
    "my-app",
    url=os.getenv("REDIS_URL", "redis://localhost:6379/0"),
    results=True,
    store_none=False,
    utc=True,
)

@huey.task(retries=3, retry_delay=60)
def send_email(recipient: str, subject: str) -> None:
    print(f"Sending {subject} to {recipient}")

@huey.periodic_task(crontab(minute="0", hour="3"))
def run_nightly_cleanup() -> None:
    print("nightly cleanup")
```

Queue layout matters. A common pattern is:

```python
# myapp/queue.py
import os

from huey import RedisHuey

huey = RedisHuey("my-app", url=os.getenv("REDIS_URL", "redis://localhost:6379/0"))

# Import task modules so the consumer sees every decorated task.
from myapp import tasks  # noqa: F401
```

If the consumer starts without importing a task module, the task can be enqueued but the worker may not know how to execute it.

## Run The Consumer

Point the consumer at the import path for your `Huey` instance:

```bash
huey_consumer.py myapp.queue.huey
```

Useful options:

```bash
huey_consumer.py myapp.queue.huey -w 4 -k thread
huey_consumer.py myapp.queue.huey -w 4 -k process
huey_consumer.py myapp.queue.huey -w 100 -k greenlet
huey_consumer.py myapp.queue.huey --no-periodic
```

Choose worker type by workload:

- `thread`: good default for mixed I/O-bound tasks
- `process`: use for CPU-bound work, but not on Windows
- `greenlet`: only for I/O-heavy tasks after `gevent.monkey.patch_all()`

Periodic task scheduling happens inside the consumer. If you run multiple consumers for the same queue, typically only one should handle periodic scheduling and the rest should start with `--no-periodic`.

## Core Usage

### Enqueue a task and wait for the result

Calling a decorated function enqueues it and returns a `Result` handle:

```python
result = send_email("ops@example.com", "daily report")

# Block for up to 10 seconds waiting for the worker.
result.get(blocking=True, timeout=10)
```

If the task raises an exception, `.get()` raises a `TaskException`.

### Schedule work for later

```python
from datetime import datetime, timedelta

eta = datetime.utcnow() + timedelta(minutes=10)
result = send_email.schedule(("ops@example.com", "later"), eta=eta)
```

Use `delay` when relative timing is easier than an absolute ETA:

```python
result = send_email.schedule(("ops@example.com", "retry soon"), delay=60)
```

### Retry from inside a task

Use decorator-level retries for common cases:

```python
@huey.task(retries=5, retry_delay=30)
def fetch_webhook(url: str) -> dict:
    ...
```

Raise `RetryTask` when the retry decision depends on runtime state:

```python
from huey import RetryTask

@huey.task()
def sync_remote_order(order_id: str) -> None:
    if not upstream_is_ready(order_id):
        raise RetryTask(delay=60)
```

### Prevent overlapping jobs

Use a lock for singleton-style work:

```python
from huey import lock_task

@huey.periodic_task(crontab(minute="*/5"))
@lock_task("rebuild-search-index")
def rebuild_search_index() -> None:
    ...
```

### Signals

Huey exposes signals such as task completion, errors, retries, locks, and enqueue events. Use them for instrumentation or cleanup hooks:

```python
from huey import SIGNAL_ERROR

@huey.signal(SIGNAL_ERROR)
def on_task_error(signal, task, exc=None):
    print("task failed", task, exc)
```

## Backend Configuration And Connection Details

Huey does not have a package-level auth system. The main operational config is how your queue backend is connected and how the consumer is tuned.

### Redis-backed queues

Redis is the primary production backend. A Redis URL can embed auth and database selection:

```python
import os

from huey import RedisHuey

huey = RedisHuey(
    "my-app",
    url=os.environ["REDIS_URL"],  # e.g. redis://:password@redis.internal:6379/0
    results=True,
)
```

Useful variants:

- `RedisHuey`: standard Redis queue
- `PriorityRedisHuey`: priority queue support; requires Redis 5.0+
- `RedisExpireHuey`: expiring result store
- `PriorityRedisExpireHuey`: priorities plus expiring results

Use an expiring result store when many callers never read task results and you do not want result keys to accumulate indefinitely.

### SQLite-backed queues

For a single machine or smaller workloads:

```python
from huey import SqliteHuey

huey = SqliteHuey("my-app", filename="huey.db")
```

This avoids Redis as an operational dependency, but it is not the right choice for horizontally scaled workers on multiple hosts.

### Immediate mode

Immediate mode executes tasks synchronously in the caller instead of queueing them:

```python
huey.immediate = True
```

This is useful in tests and sometimes in local development. Do not confuse it with running a real background queue.

## Django Integration

Use `huey.contrib.djhuey` when you are in a Django project. Configure Huey in `settings.py`:

```python
import os

HUEY = {
    "huey_class": "huey.RedisHuey",
    "name": "my-django-app",
    "results": True,
    "store_none": False,
    "url": os.getenv("REDIS_URL", "redis://localhost:6379/0"),
    "immediate": False,
    "consumer": {
        "workers": 4,
        "worker_type": "thread",
    },
}
```

Decorators from `djhuey` handle connection management around task execution:

```python
from huey.contrib.djhuey import db_task, on_commit_task

@db_task()
def recalculate_user_stats(user_id: int) -> None:
    ...

@on_commit_task()
def send_order_confirmation(order_id: int) -> None:
    ...
```

Why these matter:

- `db_task()` closes Django database connections around task execution
- `on_commit_task()` waits until the outer transaction commits before enqueueing

That second point is important whenever a task depends on rows written in the request that queued it.

Run the Django consumer with:

```bash
python manage.py run_huey
```

Common Django surprise:

- When Django `DEBUG` is on, many teams intentionally run Huey in immediate mode during development. That is convenient, but it hides real async behavior, retries, and worker-import issues.

## Testing

For unit tests, immediate mode or local execution is usually easier than spinning up workers:

```python
def test_send_email_runs_inline():
    huey.immediate = True
    send_email("ops@example.com", "subject")
```

For pure function tests, `call_local()` runs the task body without enqueueing:

```python
send_email.call_local("ops@example.com", "subject")
```

Reset queue state between tests when sharing a backend.

## Common Pitfalls

- The consumer must import the modules that define your tasks. Missing imports look like "task enqueued but never handled".
- `results=False` means no persistent task results. Also, return values of `None` are not stored unless `store_none=True`.
- Reading a result normally consumes it. Use `preserve=True` if you need to read it more than once.
- Periodic tasks do not preserve return values the same way normal tasks do. Push important outputs to your own datastore instead of relying on task return values.
- Use aware or consistently interpreted times. Huey stores timestamps in UTC internally; mismatched local time assumptions can shift scheduled jobs.
- `greenlet` workers need proper monkey-patching and are not a drop-in replacement for CPU-bound work.
- `SIGTERM` can interrupt in-flight work abruptly. Graceful shutdown handling and signal hooks matter if you need to requeue or audit interrupted tasks.
- If you upgrade from very old Huey versions, do not mix workers with old serialized tasks still sitting in the queue until you confirm serialization compatibility.

## Version-Sensitive Notes For 2.6.0

- PyPI currently lists `2.6.0` as the latest `huey` release as of March 12, 2026, so the version used here is still current.
- The 2.6.0 release itself is mostly packaging and release-process maintenance. The operational features most agents care about were added earlier in the 2.5.x line and are present in 2.6.0.
- `on_commit_task()` landed in the 2.5 series. If older internal docs do not mention it, prefer it over enqueueing tasks inside open Django transactions.
- `SIGNAL_ENQUEUED` also arrived in the 2.5 series. Use it if you need enqueue-side instrumentation.
- If you are upgrading from pre-2.0 code, watch for old names like `always_eager` and `result_store`; Huey 2.x uses `immediate` and `results`, and 2.0 changed task serialization in a way that can matter for live queues.

## Official Sources

- Docs root: `https://huey.readthedocs.io/en/stable/`
- API reference: `https://huey.readthedocs.io/en/latest/api.html`
- Django integration: `https://huey.readthedocs.io/en/latest/django.html`
- Consumer guide: `https://huey.readthedocs.io/en/latest/consumer.html`
- Signals guide: `https://huey.readthedocs.io/en/latest/signals.html`
- Registry: `https://pypi.org/project/huey/`
- Repository: `https://github.com/coleifer/huey`

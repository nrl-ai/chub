---
name: package
description: "Celery distributed task queue for Python workers, retries, scheduling, and broker-backed background jobs"
metadata:
  languages: "python"
  versions: "5.6.2"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "celery,python,task-queue,workers,async,redis,rabbitmq,django"
---

# Celery Python Package Guide

Use `celery` when Python code needs broker-backed background jobs, scheduled tasks, retries, and horizontally scalable worker processes.

## Golden Rule

- Define one importable Celery app module and point both producers and workers at it.
- Use RabbitMQ or Redis unless you have a specific reason to pick another transport.
- Treat task payloads as JSON by default and keep tasks idempotent.
- Only configure a result backend if you actually need task states or return values.

## Version Covered

- Package: `celery`
- Ecosystem: `pypi`
- Version: `5.6.2`
- Release date: `2026-01-04`
- Python requirement on PyPI: `>=3.9`

## Install

Base install:

```bash
pip install celery==5.6.2
```

Common extras from PyPI:

```bash
# Redis broker and/or result backend support
pip install "celery[redis]==5.6.2"

# Task-side Pydantic validation helpers
pip install "celery[pydantic]==5.6.2"

# pytest plugin support
pip install "celery[pytest]==5.6.2"
```

## Core Model

Celery has four moving parts:

1. A Python app object created with `Celery(...)`.
2. A broker that carries task messages.
3. One or more worker processes that execute tasks.
4. An optional result backend for task state and return values.

Recurring schedules are handled by a separate `beat` process.

## Minimal Setup

Create an importable module such as `tasks.py`:

```python
import os

from celery import Celery

app = Celery(
    "tasks",
    broker=os.environ.get("CELERY_BROKER_URL", "pyamqp://guest@localhost//"),
    backend=os.environ.get("CELERY_RESULT_BACKEND"),
)

app.conf.update(
    task_serializer="json",
    accept_content=["json"],
    result_serializer="json",
    timezone="UTC",
    enable_utc=True,
)

@app.task(
    bind=True,
    autoretry_for=(ConnectionError, TimeoutError),
    retry_backoff=True,
    retry_jitter=True,
    max_retries=5,
)
def fetch_remote(self, url: str) -> str:
    # Put explicit I/O timeouts in the code you call from tasks.
    return f"fetched {url}"
```

Start a worker:

```bash
celery -A tasks worker --loglevel=INFO
```

Queue a task:

```python
from tasks import fetch_remote

result = fetch_remote.delay("https://example.com/data.json")
print(result.id)
```

## Broker and Backend Choices

### RabbitMQ

- Best default production broker.
- Stable, durable, and feature-complete in Celery docs.
- Handles larger messages better than Redis.

Example:

```env
CELERY_BROKER_URL=pyamqp://guest:guest@localhost//
```

### Redis

- Stable as both broker and result backend.
- Good for fast, smaller messages and common local setups.
- More susceptible to data loss on abrupt termination than RabbitMQ.
- Large messages can congest Redis when used as a broker.

Example:

```env
CELERY_BROKER_URL=redis://localhost:6379/0
CELERY_RESULT_BACKEND=redis://localhost:6379/1
```

For Redis result backends, install the Redis extra:

```bash
pip install "celery[redis]==5.6.2"
```

TLS example for Redis backend:

```env
CELERY_RESULT_BACKEND=rediss://username:password@redis.example.com:6379/0?ssl_cert_reqs=required
```

### SQS and Other Brokers

- Amazon SQS is marked stable, but Celery docs note it does not support monitoring or remote control commands.
- Kafka, Zookeeper, and some other brokers are still marked experimental.

## Calling Tasks

Use `delay()` for the common case and `apply_async()` when you need execution options:

```python
from tasks import fetch_remote

fetch_remote.delay("https://example.com/a")

fetch_remote.apply_async(
    args=("https://example.com/b",),
    countdown=10,
    expires=60,
)
```

Useful `apply_async()` options:

- `countdown` or `eta` for delayed execution
- `expires` to drop stale work
- `link` and `link_error` for callbacks and errbacks

Do not use `countdown` or `eta` for large volumes of far-future jobs. Celery keeps those tasks in worker memory until execution time, and Redis brokers can redeliver them when the delay exceeds the broker visibility timeout.

## Results and State

Results are off by default. Configure `result_backend` only if you need:

- `AsyncResult.get()`
- task state inspection
- chords or workflows that depend on stored results

Example:

```python
from tasks import fetch_remote

result = fetch_remote.delay("https://example.com/c")
value = result.get(timeout=10)
result.forget()
```

If you do not need return values, set `ignore_result=True` on the task or use global task result settings to reduce backend load.

## Production-Oriented Configuration

Common settings worth knowing:

```python
app.conf.update(
    broker_url="pyamqp://user:pass@rabbitmq.example.com/vhost",
    result_backend="redis://redis.example.com/0",
    accept_content=["json"],
    task_serializer="json",
    result_serializer="json",
    timezone="UTC",
    enable_utc=True,
    worker_prefetch_multiplier=1,
    task_acks_late=True,
    task_reject_on_worker_lost=False,
)
```

Notes:

- `accept_content=['json']` keeps workers from accepting untrusted pickle or yaml payloads.
- `worker_prefetch_multiplier=1` helps fairness when tasks are long-running.
- `task_acks_late=True` only makes sense for idempotent tasks.
- Turning on `task_reject_on_worker_lost=True` can requeue work after a worker process dies, but it can also create message loops if you do not understand the failure mode.

Celery also supports:

- separate `broker_read_url` and `broker_write_url`
- multiple broker URLs for failover
- app config modules via `app.config_from_object("celeryconfig")`

## Task Design Patterns

### Idempotent retryable task

```python
from celery import shared_task

@shared_task(
    autoretry_for=(ConnectionError, TimeoutError),
    retry_backoff=True,
    retry_backoff_max=600,
    retry_jitter=True,
    max_retries=7,
)
def sync_invoice(invoice_id: str) -> None:
    # Safe to run more than once.
    ...
```

### Task with no stored result

```python
from celery import shared_task

@shared_task(ignore_result=True)
def send_webhook(payload: dict) -> None:
    ...
```

### Pydantic task-side validation

Celery 5.5 added task-side Pydantic argument and return-value conversion:

```python
from celery import Celery
from pydantic import BaseModel

app = Celery("tasks")

class JobIn(BaseModel):
    url: str

class JobOut(BaseModel):
    status: str

@app.task(pydantic=True)
def run_job(job: JobIn) -> JobOut:
    return JobOut(status=f"queued:{job.url}")
```

Important: `pydantic=True` validates on the task side. You still need to serialize task arguments correctly when calling `delay()` or `apply_async()`.

## Workers

Basic worker:

```bash
celery -A tasks worker -l INFO
```

Named workers with explicit concurrency:

```bash
celery -A proj worker --loglevel=INFO --concurrency=10 -n worker1@%h
celery -A proj worker --loglevel=INFO --concurrency=10 -n worker2@%h
```

Operational guidance:

- Set explicit timeouts in the I/O your task performs.
- Use Celery time limits for tasks that can wedge a worker.
- Prefer multiple smaller workers over one giant process when isolating queues or workloads.

## Periodic Tasks

Run the scheduler separately:

```bash
celery -A tasks beat -l INFO
```

Inline schedule example:

```python
app.conf.beat_schedule = {
    "refresh-every-30-seconds": {
        "task": "tasks.refresh_cache",
        "schedule": 30.0,
    },
}
app.conf.timezone = "UTC"
```

## Django Integration

Celery works with Django directly; a separate integration package is no longer required.

Typical `proj/celery.py`:

```python
import os

from celery import Celery

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "proj.settings")

app = Celery("proj")
app.config_from_object("django.conf:settings", namespace="CELERY")
app.autodiscover_tasks()
```

Important Django notes:

- Put Celery settings in Django settings as `CELERY_BROKER_URL`, `CELERY_RESULT_BACKEND`, and similar.
- Use `@shared_task` inside reusable apps.
- If a task depends on committed DB state, prefer `delay_on_commit()` over `delay()` in Django transaction flows.
- `delay_on_commit()` was added in Celery 5.4 and does not return a task id because sending is deferred until commit.

## Testing

For unit tests, prefer mocking task behavior or the code inside tasks.

Important testing caveats:

- `task_always_eager=True` is not a faithful unit-test substitute for a real worker.
- If you need eager execution plus stored results, also set `task_store_eager_result=True`.
- `celery.contrib.pytest` and `pytest-celery` are different and not compatible with each other.

## Common Pitfalls

- No result backend configured: `AsyncResult.get()` and state inspection will not behave the way you expect.
- Non-idempotent tasks with `acks_late=True`: worker crashes can duplicate side effects.
- No I/O timeouts inside tasks: a stuck request can block worker capacity indefinitely.
- Accepting `pickle` or `yaml` content from an untrusted broker: this widens your attack surface.
- Large or distant `countdown` loads: workers hold those messages in memory.
- Long-running tasks with default prefetch: one worker can reserve too much work early.
- Triggering Django tasks before transaction commit: the worker may not see the saved rows yet.

## Version-Sensitive Notes for 5.6.2

- Stable docs and PyPI both identify the current covered release as `5.6.2`.
- PyPI project metadata requires Python `>=3.9`.
- Task-side Pydantic validation via `pydantic=True` is available in Celery `5.5+`.
- `delay_on_commit()` for Django is available in Celery `5.4+`.
- The PyPI long description still contains some stale `5.5.x` references, so prefer the stable docs root and PyPI metadata over older prose when checking current support statements.
- The project still says Microsoft Windows is unsupported, even though it may work in some environments.

## Official Sources

- Docs root: https://docs.celeryq.dev/en/stable/
- First steps: https://docs.celeryq.dev/en/stable/getting-started/first-steps-with-celery.html
- Brokers and backends: https://docs.celeryq.dev/en/stable/getting-started/backends-and-brokers/index.html
- Configuration: https://docs.celeryq.dev/en/stable/userguide/configuration.html
- Tasks: https://docs.celeryq.dev/en/stable/userguide/tasks.html
- Calling tasks: https://docs.celeryq.dev/en/stable/userguide/calling.html
- Workers: https://docs.celeryq.dev/en/stable/userguide/workers.html
- Periodic tasks: https://docs.celeryq.dev/en/stable/userguide/periodic-tasks.html
- Testing: https://docs.celeryq.dev/en/stable/userguide/testing.html
- Django: https://docs.celeryq.dev/en/stable/django/first-steps-with-django.html
- Changelog: https://docs.celeryq.dev/en/stable/changelog.html
- PyPI: https://pypi.org/project/celery/

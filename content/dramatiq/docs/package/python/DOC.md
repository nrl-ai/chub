---
name: package
description: "Dramatiq Python package guide for background jobs with RabbitMQ or Redis brokers"
metadata:
  languages: "python"
  versions: "2.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dramatiq,python,background-jobs,task-queue,rabbitmq,redis,workers"
---

# Dramatiq Python Package Guide

## Golden Rule

Use `dramatiq` for Python background jobs, configure the broker explicitly with `dramatiq.set_broker(...)` early in process startup, and keep actor inputs small and JSON-encodable. Dramatiq assumes tasks are retriable and may run more than once, so actors should be idempotent.

## Install

Pick the extra that matches your broker:

```bash
python -m pip install "dramatiq[rabbitmq]==2.1.0"
python -m pip install "dramatiq[redis]==2.1.0"
```

Common development installs:

```bash
python -m pip install "dramatiq[rabbitmq,watch]==2.1.0"
python -m pip install "dramatiq[redis,watch]==2.1.0"
python -m pip install "dramatiq[prometheus]==2.1.0"
python -m pip install "dramatiq[all]==2.1.0"
```

Notes:

- RabbitMQ is the upstream-recommended broker.
- If you use Redis, upstream recommends adding `hiredis` for better throughput on CPython.
- Available extras on PyPI for `2.1.0` include `rabbitmq`, `redis`, `watch`, `prometheus`, `gevent`, `memcached`, and `all`.

## Initialize The Broker

Dramatiq will fall back to a default global broker if you do not set one, but upstream recommends setting the broker yourself as early as possible so worker and producer processes use the same configuration.

### RabbitMQ

```python
import os

import dramatiq
from dramatiq.brokers.rabbitmq import RabbitmqBroker

broker = RabbitmqBroker(
    url=os.getenv("DRAMATIQ_BROKER_URL", "amqp://guest:guest@localhost:5672/"),
    confirm_delivery=True,
)
dramatiq.set_broker(broker)
```

Useful RabbitMQ options:

- `url="amqp://user:pass@host:5672/vhost"` for connection config
- `confirm_delivery=True` if you need publisher confirms
- `max_priority=<n>` if you want queue-level priority with `broker_priority`

### Redis

```python
import os

import dramatiq
from dramatiq.brokers.redis import RedisBroker

broker = RedisBroker(
    url=os.getenv("DRAMATIQ_BROKER_URL", "redis://localhost:6379/0"),
    namespace=os.getenv("DRAMATIQ_NAMESPACE", "dramatiq"),
)
dramatiq.set_broker(broker)
```

Useful Redis options:

- `url="redis://:password@host:6379/0"` for connection config
- `namespace="myapp"` to isolate multiple logical apps on the same Redis instance
- Individual connection parameters are also forwarded to `redis.Redis(...)`

## Define And Send Actors

Basic actor:

```python
import dramatiq

@dramatiq.actor
def send_email(user_id: int) -> None:
    print(f"Sending email to user {user_id}")

send_email.send(42)
```

Important behavior:

- Calling `send_email(42)` runs synchronously in-process.
- Calling `send_email.send(42)` enqueues a message for a worker.
- Actor arguments must be JSON-encodable.
- Keep payloads small. Send IDs or paths, not ORM objects or large blobs.

### Run workers

If your actors live in `myapp.tasks`, start workers with:

```bash
dramatiq myapp.tasks
```

For local development with automatic reload:

```bash
dramatiq myapp.tasks --watch .
```

The worker CLI starts one process per CPU core with 8 worker threads per process by default.

## Core Usage Patterns

### Retries and idempotency

Dramatiq retries failures automatically with exponential backoff. The default retries middleware uses `max_retries=20`, and the user guide notes retries can span roughly 30 days.

```python
import dramatiq

@dramatiq.actor(max_retries=3, min_backoff=1_000, max_backoff=30_000)
def sync_invoice(invoice_id: str) -> None:
    ...
```

Use `throws=` for expected non-retriable errors:

```python
import dramatiq

class ValidationError(Exception):
    pass

@dramatiq.actor(throws=(ValidationError,), max_retries=0)
def process_upload(upload_id: str) -> None:
    ...
```

### Delayed messages

```python
import dramatiq

@dramatiq.actor
def remind_user(user_id: str) -> None:
    ...
```

```python
remind_user.send_with_options(args=("user-123",), delay=60_000)
```

Use scheduled jobs sparingly. Upstream explicitly warns that the broker is not a database.

### Priorities

Actor priority controls which already-consumed messages run first in a worker. Lower numbers are higher priority.

```python
import dramatiq

@dramatiq.actor(priority=0)
def user_facing_job(job_id: str) -> None:
    ...

@dramatiq.actor(priority=100)
def batch_job(job_id: str) -> None:
    ...
```

If you use RabbitMQ queue priorities, configure `max_priority` on the broker and enqueue with `broker_priority`:

```python
user_facing_job.send_with_options(args=("job-1",), broker_priority=5)
```

### Time and age limits

These are middleware-driven safety controls:

```python
import dramatiq

@dramatiq.actor(max_age=3_600_000, time_limit=30_000)
def rebuild_cache(key: str) -> None:
    ...
```

- `max_age` drops messages that have sat in the queue too long.
- `time_limit` interrupts long-running work.
- The time-limit middleware cannot cancel blocking system calls immediately; it only interrupts when the worker thread next acquires the GIL.

## Async Actors

Async actors need the optional `AsyncIO` middleware:

```python
import dramatiq
from dramatiq.brokers.redis import RedisBroker
from dramatiq.middleware import AsyncIO

broker = RedisBroker()
broker.add_middleware(AsyncIO())
dramatiq.set_broker(broker)

@dramatiq.actor
async def refresh_remote_cache(key: str) -> None:
    ...
```

Without `AsyncIO`, `async def` actors are not configured correctly for worker execution.

## Results And Pipelines

Result storage is optional and not enabled by default. In `2.x`, the `Results` middleware requires an explicit backend.

```python
import dramatiq
from dramatiq.brokers.redis import RedisBroker
from dramatiq.results import Results
from dramatiq.results.backends import RedisBackend

broker = RedisBroker(url="redis://localhost:6379/0")
result_backend = RedisBackend(url="redis://localhost:6379/1")
broker.add_middleware(Results(backend=result_backend))
dramatiq.set_broker(broker)

@dramatiq.actor(store_results=True)
def add(x: int, y: int) -> int:
    return x + y

message = add.send(1, 2)
result = message.get_result(backend=result_backend)
```

Pipelines and groups are available, but pipeline completion tracking depends on results being enabled for the actors you care about.

## Testing

Use `StubBroker` for unit and integration-style tests that should not require RabbitMQ or Redis:

```python
import dramatiq
from dramatiq import Worker
from dramatiq.brokers.stub import StubBroker

broker = StubBroker()
broker.emit_after("process_boot")
dramatiq.set_broker(broker)

@dramatiq.actor(max_retries=0)
def add(x: int, y: int) -> None:
    assert x > 0

worker = Worker(broker, worker_timeout=100)
worker.start()

add.send(1, 2)
broker.join(add.queue_name)
worker.join()
worker.stop()
```

Testing notes:

- In `2.x`, `StubBroker.join()` fails fast by default and re-raises dead-lettered actor exceptions.
- Retries still apply in tests. Set `max_retries=0` or configure `Retries(...)` for fast feedback.
- You can also test actor logic synchronously by calling the actor directly.

## Configuration And Operational Notes

- Authentication is broker-specific. Use broker URLs or connection kwargs; Dramatiq itself does not have a separate auth layer.
- For RabbitMQ multi-tenancy, use virtual hosts.
- For Redis multi-tenancy, use a dedicated DB and/or `namespace`.
- If you customize the middleware list manually, you replace the defaults. Add back anything you still rely on, such as `Retries`, `Callbacks`, or `Pipelines`.
- `Prometheus` middleware is optional in `2.x`; install the `prometheus` extra and add the middleware explicitly if you need metrics.

## Common Pitfalls

- Forgetting to call `dramatiq.set_broker(...)` early, then producing with one broker config and consuming with another.
- Sending non-JSON-encodable arguments or large objects through messages.
- Treating actors as exactly-once. Worker failures can cause the same message to run more than once.
- Expecting actor `priority` to reorder messages still waiting in the broker queue; that needs RabbitMQ `broker_priority`.
- Enabling `Results()` without a backend in older examples. In `2.x`, `backend=` is required.
- Expecting `time_limit` to interrupt blocking I/O immediately.
- Running enqueue code in a pre-fork server that shares broker connections across forks. The troubleshooting guide calls out `FileNotFoundError` issues in that setup; load app code after fork or use the server-specific workaround.
- Waiting forever in tests because retries keep rescheduling failures. Disable or shrink retries in tests.

## Version-Sensitive Notes For 2.1.0

- `2.1.0` is the current PyPI release as of 2026-03-12.
- `2.1.0` raises the supported maximum `redis` client version to `7.x`.
- `dramatiq.errors.ConnectionError` was renamed to `BrokerConnectionError`; the old import still works for backward compatibility and is slated for removal in `3.0.0`.
- `2.0.0` changed several behaviors that older blog posts may miss:
  - `StubBroker.join(...)/fail_fast` now defaults to failing fast.
  - `Prometheus` is no longer part of the default middleware list.
  - `Results` now requires an explicit `backend=...`.
  - `URLRabbitmqBroker` was removed; use `RabbitmqBroker(url=...)`.

## Official Sources

- Docs root: `https://dramatiq.io/`
- Installation: `https://dramatiq.io/installation.html`
- User guide: `https://dramatiq.io/guide.html`
- API reference: `https://dramatiq.io/reference.html`
- Advanced topics: `https://dramatiq.io/advanced.html`
- Troubleshooting: `https://dramatiq.io/troubleshooting.html`
- Changelog: `https://dramatiq.io/changelog.html`
- PyPI: `https://pypi.org/project/dramatiq/`

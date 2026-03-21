---
name: package
description: "rq package guide for Python background jobs with Redis or Valkey queues, workers, retries, scheduling, and 2.7.0 notes"
metadata:
  languages: "python"
  versions: "2.7.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "rq,redis,valkey,queue,background-jobs,worker,scheduler"
---

# RQ Python Package Guide

## Golden Rule

Use `rq` when you want a small, explicit Redis- or Valkey-backed background job system in pure Python. Pass Redis connections explicitly to `Queue` and `Worker`, keep job functions importable by workers, and enable the scheduler whenever you rely on delayed jobs, repeated jobs, or retry intervals.

## When To Use RQ

Reach for `rq` when:

- your app is already comfortable depending on Redis or Valkey
- jobs are plain Python functions and you do not need a large task orchestration layer
- you want a separate worker process model instead of `asyncio` background tasks inside the web process
- you want straightforward CLI and code APIs for enqueueing, retries, results, and simple scheduling

RQ is usually a better fit than a heavier queue system when you want "push a Python function onto a queue and let workers run it" with minimal framework machinery.

## Install

Install the package version your project expects:

```bash
python -m pip install "rq==2.7.0"
```

Common alternatives:

```bash
uv add "rq==2.7.0"
poetry add "rq==2.7.0"
```

Runtime prerequisites:

- a running Redis or Valkey server
- network access from producers and workers to that backend
- the same application code available to the process that enqueues jobs and the process that runs workers

## Basic Setup

RQ does not handle application auth itself. The important configuration is the Redis or Valkey connection that backs your queues and workers.

Use one explicit connection object and pass it everywhere:

```python
import os

from redis import Redis
from rq import Queue

redis_url = os.getenv("RQ_REDIS_URL", "redis://localhost:6379/0")
redis_conn = Redis.from_url(redis_url)

queue = Queue("default", connection=redis_conn)
```

Minimal task module:

```python
# tasks.py
import requests

def fetch_status(url: str) -> int:
    response = requests.get(url, timeout=10)
    response.raise_for_status()
    return response.status_code
```

Producer code:

```python
from redis import Redis
from rq import Queue

from tasks import fetch_status

redis_conn = Redis.from_url("redis://localhost:6379/0")
queue = Queue("default", connection=redis_conn)

job = queue.enqueue(fetch_status, "https://python-rq.org")
print(job.id)
```

Start a worker from the project directory so imports resolve the same way they do for your app:

```bash
rq worker default --url redis://localhost:6379/0
```

Programmatic worker setup:

```python
from redis import Redis
from rq import Queue, Worker

redis_conn = Redis.from_url("redis://localhost:6379/0")
queue = Queue("default", connection=redis_conn)
worker = Worker([queue], connection=redis_conn)
worker.work()
```

## Core Usage

### Enqueue jobs

The basic API is `queue.enqueue(callable, *args, **kwargs)`:

```python
job = queue.enqueue(fetch_status, "https://python-rq.org")
print(job.id)
```

Useful enqueue options from the official docs:

- `job_timeout`: max runtime before the worker marks the job failed
- `result_ttl`: how long successful job results stay in Redis, default `500`
- `ttl`: max time a queued job may sit before it is discarded
- `failure_ttl`: how long failed jobs stay in Redis, default `1 year`
- `depends_on`: job or jobs that must complete first
- `job_id`: stable custom job id
- `at_front`: prioritize within a queue
- `on_success`, `on_failure`, `on_stopped`: callbacks

Example:

```python
job = queue.enqueue(
    fetch_status,
    "https://python-rq.org",
    job_timeout="30s",
    result_ttl=3600,
    failure_ttl=86400,
    job_id="fetch-python-rq-homepage",
)
```

### Use multiple queues for priority

RQ treats queue order as priority order for a worker:

```python
high = Queue("high", connection=redis_conn)
default = Queue("default", connection=redis_conn)
low = Queue("low", connection=redis_conn)

high.enqueue(fetch_status, "https://python-rq.org/docs/")
low.enqueue(fetch_status, "https://example.com")
```

```bash
rq worker high default low --url redis://localhost:6379/0
```

If you want fair queue selection instead of strict priority, RQ also ships `RoundRobinWorker` and `RandomWorker`.

### Fetch results and job state

In modern RQ, prefer `job.return_value()` and `job.latest_result()`:

```python
from rq.job import Job

job = Job.fetch(job.id, connection=redis_conn)
result = job.latest_result()

if result and result.type == result.Type.SUCCESSFUL:
    print(result.return_value)
elif result and result.type == result.Type.FAILED:
    print(result.exc_string)
```

Block for a result when needed:

```python
result = job.latest_result(timeout=60)
if result is not None:
    print(result.type, result.return_value)
```

Use `job.results()` when a job may execute multiple times. RQ keeps up to the 10 latest execution results.

### Retries

Retry exceptions with `Retry(...)`:

```python
from rq import Retry

queue.enqueue(
    fetch_status,
    "https://python-rq.org",
    retry=Retry(max=3, interval=[10, 30, 60]),
)
```

Important behavior:

- `Retry(max=3)` retries immediately after failures
- `Retry(max=3, interval=60)` or an interval list requires workers running with `--with-scheduler`

RQ also supports returning `Retry(...)` from job code for application-level retry decisions:

```python
import requests
from rq import Retry

def fetch_with_soft_retry(url: str, max_attempts: int = 3):
    try:
        response = requests.get(url, timeout=10)
        response.raise_for_status()
        return response.text
    except requests.exceptions.ConnectionError:
        return Retry(max=max_attempts, interval=60)
```

### Schedule delayed and repeated jobs

Delayed jobs:

```python
from datetime import datetime, timedelta

queue.enqueue_in(timedelta(minutes=5), fetch_status, "https://python-rq.org/docs/")
queue.enqueue_at(datetime(2026, 3, 12, 18, 0), fetch_status, "https://python-rq.org/")
```

Repeated jobs:

```python
from rq import Repeat

queue.enqueue(
    fetch_status,
    "https://python-rq.org/docs/",
    repeat=Repeat(times=3, interval=[5, 10, 30]),
)
```

Important behavior:

- scheduled jobs live in `ScheduledJobRegistry` until they are due
- repeated jobs only repeat after successful completion
- scheduling and repeating require workers started with `--with-scheduler`

Scheduler-enabled worker:

```bash
rq worker default --with-scheduler --url redis://localhost:6379/0
```

Programmatic equivalent:

```python
if __name__ == "__main__":
    worker = Worker([queue], connection=redis_conn)
    worker.work(with_scheduler=True)
```

That `if __name__ == "__main__"` guard matters when scheduler support is enabled because the scheduler uses a separate process.

### Cron-style recurring enqueueing

For recurring jobs that should be registered outside normal queue producers, use RQ's built-in cron support:

```python
# cron_config.py
from rq import cron

from tasks import fetch_status

cron.register(
    fetch_status,
    queue_name="maintenance",
    args=("https://python-rq.org/health",),
    cron="*/15 * * * *",
    job_timeout=30,
    result_ttl=3600,
)
```

Run it:

```bash
rq cron cron_config.py --url redis://localhost:6379/0
```

Use cron support when you want a dedicated scheduler process that enqueues recurring jobs; use `Repeat` when the next run should be chained off a job's own successful completion.

## Worker Selection

Default production worker:

- `Worker`: normal process-isolated execution

Useful alternatives:

- `SpawnWorker`: uses `os.spawn()` instead of `fork()`, useful on Windows or newer macOS environments where `fork()` is a problem
- `SimpleWorker`: runs jobs in-process, useful for tests and debugging, but not recommended for normal production workloads because it has less isolation and no periodic heartbeat during job execution
- `WorkerPool`: convenient CLI wrapper to launch multiple workers in one command

Examples:

```bash
rq worker default --url redis://localhost:6379/0
rq worker -w rq.worker.SpawnWorker default --url redis://localhost:6379/0
rq worker-pool high default low -n 3 --url redis://localhost:6379/0
```

## Configuration Notes

### Prefer explicit connections

The current docs recommend passing `connection=` explicitly to queues, workers, and jobs:

```python
redis_conn = Redis.from_url("redis://localhost:6379/0")
queue = Queue("default", connection=redis_conn)
worker = Worker([queue], connection=redis_conn)
```

Do not build new code around the old `Connection` context manager. The connections docs mark it as deprecated.

### Redis client options that matter

Common connection details belong in the Redis client, not in RQ:

```python
from redis import Redis

redis_conn = Redis.from_url(
    "rediss://user:password@example.cache.local:6379/0",
    socket_timeout=500,
)
```

Practical notes from the official docs:

- if you set `socket_timeout` manually, keep it higher than the worker's dequeue timeout or workers can trip `TimeoutError`
- `decode_responses=True` is not supported by RQ
- Sentinel setups are supported, but the configuration lives at the Redis connection layer and RQ config layer rather than a separate RQ auth system

### Serializer and custom classes

RQ defaults to pickle serialization. If you switch to a custom serializer or custom `Job` / `Queue` classes, use the same serializer and classes on both the enqueueing side and the worker side.

## Testing Patterns

### Fast synchronous tests

Use `is_async=False` when you want queue semantics without running a worker:

```python
from redis import Redis
from rq import Queue

test_conn = Redis.from_url("redis://localhost:6379/15")
queue = Queue(is_async=False, connection=test_conn)
job = queue.enqueue(fetch_status, "https://python-rq.org")
assert job.is_finished
```

Use a disposable Redis database or test instance so one test run does not leak state into another.

### Worker-based tests

Use `SimpleWorker` inside tests when the default worker's `fork()` behavior conflicts with the test environment:

```python
from redis import Redis
from rq import Queue, SimpleWorker

conn = Redis.from_url("redis://localhost:6379/15")
queue = Queue(connection=conn)
queue.enqueue(fetch_status, "https://python-rq.org")

worker = SimpleWorker([queue], connection=conn)
worker.work(burst=True)
```

On Windows test environments, the docs recommend subclassing `SimpleWorker` to use `TimerDeathPenalty`.

## Common Pitfalls

- Enqueued callables must be importable by workers. Do not enqueue functions defined only in `__main__`, and do not rely on request-local or process-local state.
- Workers and producers must run the same code version. If you deploy new producer code before workers are updated, imports or argument shapes can drift.
- Delayed jobs, repeated jobs, and retry intervals do not work correctly unless the scheduler component is running.
- `SimpleWorker` is for testing, debugging, and niche cases. Use `Worker` or `SpawnWorker` for normal production isolation.
- `job.result` appears in older examples, but the modern docs point to `job.return_value()` and `job.latest_result()`.
- Result data expires by default after `500` seconds. If you need longer-lived inspection, set `result_ttl` deliberately.
- Failed jobs stay around for a long time by default (`failure_ttl` defaults to one year). Set a lower value if that retention is too expensive for your Redis footprint.
- `decode_responses=True` on the Redis client is not supported.
- The old `Connection` context manager is deprecated; prefer explicit `connection=` wiring.
- `CronScheduler` is still marked beta in the official docs. Treat it as useful but newer than the core queue and worker APIs.

## Version-Sensitive Notes For `2.7.0`

- `2.7.0` formally supports Python 3.14.
- `2.7.0` improves `CronScheduler` monitoring so each cron job exposes better latest and next scheduled enqueue visibility.
- `2.7.0` fixes `job.get_status()` inside `on_success` and `on_failure` callbacks so callback code sees the correct final state.
- `2.5.0` added cron-string support to `CronScheduler`; older 2.4 examples may only show interval-based scheduling.
- `2.4.0` introduced the `rq cron` CLI command and raised the package baseline to Python `>=3.9`.
- `2.3.0` added official Valkey support and repeat jobs.
- `2.2.0` added `SpawnWorker`, which matters if you need Windows-compatible worker execution.

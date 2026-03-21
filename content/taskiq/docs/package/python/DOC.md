---
name: package
description: "Taskiq Python package guide for async background jobs, workers, scheduling, and testing"
metadata:
  languages: "python"
  versions: "0.12.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "taskiq,python,async,background-jobs,workers,scheduler,queues"
---

# Taskiq Python Package Guide

## Golden Rule

`taskiq` is the core task framework, not a complete production queue stack by itself. Use `taskiq` for task definitions, workers, scheduling, dependency injection, and testing, then pair it with an actual broker package and usually a result backend package for real deployments. `InMemoryBroker` is for local development and tests, not for multi-process or multi-host production work.

## Install

Pin the package version your project expects:

```bash
python -m pip install "taskiq==0.12.1"
```

Common alternatives:

```bash
uv add "taskiq==0.12.1"
poetry add "taskiq==0.12.1"
```

Useful extras from the maintainer docs and PyPI metadata:

```bash
python -m pip install "taskiq[reload]==0.12.1"
python -m pip install "taskiq[zmq]==0.12.1"
python -m pip install "taskiq[opentelemetry]==0.12.1"
```

Production deployments usually also need integration packages, for example:

```bash
python -m pip install "taskiq-aio-pika" "taskiq-redis"
```

That combination gives you a RabbitMQ broker plus a Redis result backend. The exact transport and backend packages depend on the infrastructure your app already uses.

## Core Model

Taskiq has a few concepts agents need to keep straight:

- `broker`: where tasks are published and where workers consume them from
- `result backend`: where task return values and errors are stored if you need them later
- `worker`: long-running process that executes tasks
- `scheduler`: separate process that enqueues due tasks; it does not execute them
- `TaskiqDepends`: lightweight dependency injection for task arguments
- labels: task metadata used for schedules, timeouts, retries, and routing-related behavior

If you only install `taskiq`, you still need to choose how tasks move between processes. The in-memory broker works only inside the current process.

## Initialize A Broker

### Minimal local setup

This is the fastest way to define and run a task in one process:

```python
import asyncio

from taskiq import InMemoryBroker

broker = InMemoryBroker()

@broker.task
async def add(a: int, b: int) -> int:
    return a + b

async def main() -> None:
    await broker.startup()
    try:
        task = await add.kiq(2, 3)
        result = await task.wait_result(timeout=5)
        print(result.return_value)
    finally:
        await broker.shutdown()

asyncio.run(main())
```

Use this for tests, local experiments, and examples. Do not treat it as a distributed queue.

### Typical production shape

Taskiq’s official docs expect you to expose a broker object from an importable module and point the worker CLI at it:

```python
# worker.py
from taskiq_aio_pika import AioPikaBroker
from taskiq_redis import RedisAsyncResultBackend

broker = AioPikaBroker("amqp://guest:guest@localhost:5672/")
broker = broker.with_result_backend(
    RedisAsyncResultBackend("redis://localhost:6379/0"),
)

@broker.task
async def add(a: int, b: int) -> int:
    return a + b
```

Run the worker:

```bash
taskiq worker worker:broker
```

If your project uses filesystem discovery instead of importing every module directly:

```bash
taskiq worker worker:broker --fs-discover
```

## Sending And Awaiting Tasks

### Enqueue a task

```python
task = await add.kiq(40, 2)
```

`kiq()` sends the task to the configured broker and returns an async task handle.

### Read the result

```python
result = await task.wait_result(timeout=10)

if result.is_err:
    raise result.error

print(result.return_value)
```

Result retrieval only works when the broker is configured with a real result backend. Without one, Taskiq uses a dummy backend and you should not expect persisted return values.

### Add labels

Labels attach metadata that middlewares and schedulers can use:

```python
task = await add.kicker().with_labels(timeout=30).kiq(1, 2)
```

Use labels when the integration you picked documents behavior tied to them. Task schedules also come from task labels.

## Dependencies And Context

Taskiq has its own DI mechanism. Use `TaskiqDepends`, not FastAPI’s `Depends`.

```python
from taskiq import Context, TaskiqDepends

def get_prefix() -> str:
    return "processed"

@broker.task
async def format_name(
    name: str,
    prefix: str = TaskiqDepends(get_prefix),
    context: Context = TaskiqDepends(),
) -> str:
    return f"{prefix}:{name}:{context.message.task_id}"
```

Important behavior from the official docs:

- The dependency graph is prepared during broker startup, so startup hooks matter.
- Event handlers can add objects into broker state for later injection.
- Framework integrations such as `taskiq-fastapi` bridge request-app state into tasks, but Taskiq’s dependency wrapper is still the task-side API.

## Scheduling

Run the scheduler as a separate process:

```python
from taskiq import TaskiqScheduler
from taskiq.schedule_sources import LabelScheduleSource

scheduler = TaskiqScheduler(broker, [LabelScheduleSource(broker)])
```

Then start it:

```bash
taskiq scheduler worker:scheduler
```

Use label-based schedules for simple cron or interval jobs declared on tasks. Keep these constraints in mind:

- You still need workers running; the scheduler only publishes tasks.
- Run only one scheduler instance unless your chosen schedule source explicitly supports coordination.
- Taskiq defaults scheduler time handling to UTC in the official docs.
- If you persist schedules outside task labels, use the appropriate schedule-source package and its storage settings.

## Testing

The maintainer docs strongly recommend `InMemoryBroker` for tests.

```python
import pytest

from taskiq import InMemoryBroker

broker = InMemoryBroker()

@broker.task
async def add(a: int, b: int) -> int:
    return a + b

@pytest.mark.anyio
async def test_add() -> None:
    await broker.startup()
    try:
        task = await add.kiq(2, 5)
        result = await task.wait_result(timeout=5)
        assert result.return_value == 7
    finally:
        await broker.shutdown()
```

Useful testing patterns from the official testing guide:

- use `InMemoryBroker(await_inplace=True)` when you want tasks to execute immediately in-process
- use `broker.wait_all()` when you need to wait until queued tasks finish
- populate dependency context explicitly in tests when the code depends on injected state

## Configuration And Auth

`taskiq` itself does not define one universal auth scheme. Connection details and credentials live in the broker and result-backend integrations you choose.

Common configuration points:

- AMQP / Redis / NATS / Kafka DSNs or connection kwargs belong to the transport package, not Taskiq core
- result backend URLs and serialization settings belong to the backend package
- hot reload support requires the `reload` extra
- ZeroMQ support requires the `zmq` extra
- OpenTelemetry integration requires the `opentelemetry` extra and compatible middleware/instrumentation setup

A practical environment-variable pattern looks like this:

```python
import os

from taskiq_aio_pika import AioPikaBroker
from taskiq_redis import RedisAsyncResultBackend

broker = AioPikaBroker(os.environ["TASKIQ_BROKER_URL"])
broker = broker.with_result_backend(
    RedisAsyncResultBackend(os.environ["TASKIQ_RESULT_BACKEND_URL"]),
)
```

## Common Pitfalls

- Forgetting `await broker.startup()` before publishing or testing tasks. Dependency graphs and broker internals are initialized there.
- Expecting task return values without configuring a real result backend.
- Running only `taskiq scheduler ...` and forgetting that a separate worker process must actually execute the task.
- Running more than one scheduler instance and accidentally publishing duplicate scheduled jobs.
- Using `InMemoryBroker` for production or cross-process communication.
- Using framework-specific dependency helpers inside tasks. Taskiq tasks should use `TaskiqDepends`.
- Assuming sync task time limits can always kill underlying work immediately. Enforced timeout behavior depends on how the worker executes the callable.
- If you use the built-in ZeroMQ broker, keep the worker count at `1`; the maintainer docs call out duplicate execution problems with higher counts.

## Version-Sensitive Notes For 0.12.x

- `0.12.0` dropped Python 3.9 support. If a project still targets 3.9, this package line is not a fit.
- `0.12.0` added interval-based scheduling and OpenTelemetry support, so examples written for `0.11.x` may miss features now available in `0.12.x`.
- `0.12.1` adds Python 3.14 support and includes fixes around sync decorators and ZeroMQ behavior. If you use sync tasks or the built-in ZMQ broker, prefer `0.12.1` over earlier `0.12.0` examples.

## Official Sources Used

- Documentation root: `https://taskiq-python.github.io/`
- Getting started: `https://taskiq-python.github.io/guide/getting-started.html`
- State and dependencies: `https://taskiq-python.github.io/guide/state-and-deps.html`
- Scheduling tasks: `https://taskiq-python.github.io/guide/scheduling-tasks.html`
- Testing guide: `https://taskiq-python.github.io/guide/testing-taskiq.html`
- Available brokers: `https://taskiq-python.github.io/available-components/brokers.html`
- PyPI package page: `https://pypi.org/project/taskiq/`
- GitHub releases: `https://github.com/taskiq-python/taskiq/releases`

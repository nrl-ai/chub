---
name: package
description: "APScheduler Python package guide for in-process job scheduling with cron, interval, date, and persistent job stores"
metadata:
  languages: "python"
  versions: "3.11.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "apscheduler,scheduling,cron,asyncio,jobs,background-tasks"
---

# APScheduler Python Package Guide

## Golden Rule

Use APScheduler as an in-process scheduler inside your app, choose the scheduler class that matches your runtime (`BlockingScheduler`, `BackgroundScheduler`, `AsyncIOScheduler`, and so on), and register durable jobs with explicit IDs plus `replace_existing=True` when your app starts. This doc is for APScheduler `3.11.2` and the 3.x API, not the 4.0 alpha API.

## Install

Pin the version your project expects:

```bash
python -m pip install "apscheduler==3.11.2"
```

Common alternatives:

```bash
uv add "apscheduler==3.11.2"
poetry add "apscheduler==3.11.2"
```

Install extras only when you use those integrations:

```bash
python -m pip install "apscheduler[sqlalchemy]==3.11.2"
python -m pip install "apscheduler[redis]==3.11.2"
python -m pip install "apscheduler[mongodb]==3.11.2"
```

## Choose The Right Scheduler

- `BlockingScheduler`: foreground process whose main job is scheduling
- `BackgroundScheduler`: normal synchronous app that should keep running while the scheduler works in threads
- `AsyncIOScheduler`: `asyncio` applications
- `GeventScheduler`, `TornadoScheduler`, `TwistedScheduler`, `QtScheduler`: framework-specific event loops

If you are unsure, start with `BackgroundScheduler` for sync apps and `AsyncIOScheduler` for `asyncio`.

## Core Usage

### Background scheduler in a synchronous app

```python
from time import sleep
from zoneinfo import ZoneInfo

from apscheduler.schedulers.background import BackgroundScheduler

def refresh_cache() -> None:
    print("refreshing cache")

scheduler = BackgroundScheduler(timezone=ZoneInfo("UTC"))

scheduler.add_job(
    refresh_cache,
    trigger="interval",
    minutes=5,
    id="refresh_cache",
    replace_existing=True,
    max_instances=1,
    coalesce=True,
    misfire_grace_time=60,
)

scheduler.add_job(
    refresh_cache,
    trigger="cron",
    hour=3,
    minute=0,
    id="nightly_refresh",
    replace_existing=True,
)

scheduler.start()

try:
    while True:
        sleep(60)
finally:
    scheduler.shutdown(wait=True)
```

Why these options matter:

- `id` and `replace_existing=True` prevent duplicate persistent jobs on every app restart
- `max_instances=1` avoids overlapping executions of the same job
- `coalesce=True` rolls multiple missed runs into one execution after downtime
- `misfire_grace_time` defines how late a missed run may still execute

### AsyncIO scheduler

```python
import asyncio
from zoneinfo import ZoneInfo

from apscheduler.schedulers.asyncio import AsyncIOScheduler

async def poll_queue() -> None:
    print("polling queue")

async def main() -> None:
    scheduler = AsyncIOScheduler(timezone=ZoneInfo("UTC"))
    scheduler.add_job(
        poll_queue,
        trigger="interval",
        seconds=30,
        id="poll_queue",
        replace_existing=True,
        max_instances=1,
        coalesce=True,
    )
    scheduler.start()

    try:
        await asyncio.Event().wait()
    finally:
        scheduler.shutdown(wait=True)

asyncio.run(main())
```

Use the scheduler that matches the event loop you already run. Do not bolt `BackgroundScheduler` into an `asyncio` app unless you have a deliberate threading reason.

### Cron expressions

For crontab-style schedules, `CronTrigger.from_crontab()` is often clearer than passing each field separately:

```python
from zoneinfo import ZoneInfo

from apscheduler.schedulers.background import BackgroundScheduler
from apscheduler.triggers.cron import CronTrigger

def generate_report() -> None:
    print("report generated")

scheduler = BackgroundScheduler(timezone=ZoneInfo("UTC"))
scheduler.add_job(
    generate_report,
    trigger=CronTrigger.from_crontab("*/15 * * * 1-5", timezone=ZoneInfo("UTC")),
    id="weekday_report",
    replace_existing=True,
)
```

## Persistent Job Stores And Scheduler Configuration

APScheduler separates:

- job stores: where jobs are persisted
- executors: where jobs run
- triggers: how schedules are calculated
- schedulers: the top-level coordinator

Typical persistent configuration:

```python
from zoneinfo import ZoneInfo

from apscheduler.executors.pool import ProcessPoolExecutor, ThreadPoolExecutor
from apscheduler.jobstores.sqlalchemy import SQLAlchemyJobStore
from apscheduler.schedulers.background import BackgroundScheduler

jobstores = {
    "default": SQLAlchemyJobStore(url="sqlite:///jobs.sqlite"),
}

executors = {
    "default": ThreadPoolExecutor(10),
    "processpool": ProcessPoolExecutor(2),
}

job_defaults = {
    "coalesce": True,
    "max_instances": 1,
    "misfire_grace_time": 300,
}

scheduler = BackgroundScheduler(
    jobstores=jobstores,
    executors=executors,
    job_defaults=job_defaults,
    timezone=ZoneInfo("UTC"),
)
```

Notes:

- The user guide recommends `SQLAlchemyJobStore` on PostgreSQL if you need a persistent store with stronger integrity guarantees.
- `MemoryJobStore` is the default and loses all jobs on process restart.
- Job stores must not be shared between multiple scheduler instances.
- Jobs sent to persistent job stores or `ProcessPoolExecutor` must be serializable. Use globally importable callables and serializable arguments.

## Lifecycle, Monitoring, And Recovery

Start paused if you want to prune or inspect persistent jobs before the scheduler begins processing:

```python
scheduler.start(paused=True)
# scheduler.remove_job("stale-job")
scheduler.resume()
```

Add listeners for basic monitoring:

```python
from apscheduler.events import EVENT_JOB_ERROR, EVENT_JOB_EXECUTED

def on_job_event(event) -> None:
    if event.exception:
        print(f"job failed: {event.job_id}")
    else:
        print(f"job succeeded: {event.job_id}")

scheduler.add_listener(on_job_event, EVENT_JOB_EXECUTED | EVENT_JOB_ERROR)
```

Useful runtime operations:

```python
scheduler.pause()
scheduler.resume()
print(scheduler.get_jobs())
scheduler.print_jobs()
```

## Configuration And Credentials

APScheduler itself does not have package-level authentication. Credentials only matter for the backends you configure around it:

- SQLAlchemy job stores use database connection URLs or engines
- Redis and MongoDB job stores use backend client credentials
- Application jobs themselves may need API keys, tokens, or cloud credentials

Keep backend URLs and secrets outside source control:

```python
import os

database_url = os.environ["SCHEDULER_DATABASE_URL"]
```

Operational defaults that are worth setting explicitly:

- `timezone`: prefer `ZoneInfo("UTC")` unless the business rule truly depends on local time
- `job_defaults.coalesce`
- `job_defaults.max_instances`
- `job_defaults.misfire_grace_time`

## Common Pitfalls

- Registering startup jobs without `id` and `replace_existing=True` will duplicate them in persistent stores on every restart.
- Persistent stores and process pools serialize jobs. Nested functions, lambdas, closures, and non-serializable arguments are common failure points.
- Long-running jobs can skip later runs or pile up. Tune `max_instances`, `coalesce`, and `misfire_grace_time` deliberately.
- Local time plus DST can produce confusing schedules. UTC is safer unless local-time semantics are required.
- `BlockingScheduler` will take over the main thread. Use `BackgroundScheduler` instead for web apps, CLIs with other work, or services that already own their main loop.
- If you have multiple app processes, each process gets its own scheduler. APScheduler 3.x is not a distributed scheduler coordinator.

## Version-Sensitive Notes For 3.11.2

- This doc targets the APScheduler 3.x API. If you see examples using `Scheduler`, `AsyncScheduler`, `add_schedule()`, or `start_in_background()`, those are from the separate 4.0 alpha line and should not be copied into a 3.11.2 project.
- APScheduler 3.11.0 added support for `ZoneInfo` time zones and deprecated `pytz`. Prefer `zoneinfo` in new code.
- APScheduler 3.11.0 dropped Python 3.6 and 3.7 support. Use Python 3.8 or newer.
- APScheduler 3.11.2 includes fixes around DST handling in `CronTrigger`. Even with the fix, ambiguous local times near DST transitions are still worth testing explicitly.

## Official Sources

- APScheduler docs root: `https://apscheduler.readthedocs.io/en/latest/`
- User guide: `https://apscheduler.readthedocs.io/en/latest/userguide.html`
- Version history: `https://apscheduler.readthedocs.io/en/latest/versionhistory.html`
- Migration notes: `https://apscheduler.readthedocs.io/en/latest/migration.html`
- PyPI package page: `https://pypi.org/project/APScheduler/`

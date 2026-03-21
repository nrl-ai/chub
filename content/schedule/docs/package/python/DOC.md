---
name: package
description: "schedule Python job scheduler for lightweight in-process recurring tasks"
metadata:
  languages: "python"
  versions: "1.2.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "schedule,scheduler,jobs,cron,tasks,python"
---

# schedule Python Package Guide

## Golden Rule

Use `schedule` for lightweight in-process recurring jobs that can run inside a single Python process. It does not provide persistence, distributed coordination, job retries, or true cron semantics, so use it for app-local automation loops, not as a replacement for Celery, APScheduler-backed stores, or system cron.

## Install

Pin the package version your project expects:

```bash
python -m pip install "schedule==1.2.2"
```

Common alternatives:

```bash
uv add "schedule==1.2.2"
poetry add "schedule==1.2.2"
```

Timezone-aware `.at(..., timezone)` usage depends on `pytz`:

```bash
python -m pip install "schedule==1.2.2" pytz
```

## Initialize And Run Jobs

Minimal recurring loop:

```python
import time

import schedule

def job() -> None:
    print("running task")

schedule.every(10).seconds.do(job)
schedule.every().minute.do(job)
schedule.every().hour.do(job)
schedule.every().day.at("10:30").do(job)

while True:
    schedule.run_pending()
    time.sleep(1)
```

Important behavior:

- Jobs run only when your process calls `run_pending()`
- Missed jobs are not replayed later; `run_pending()` intentionally runs each due job once
- By default jobs run serially on the same thread

If you want the loop to sleep exactly until the next job, use `idle_seconds()`:

```python
import time

import schedule

def job() -> None:
    print("tick")

schedule.every(5).minutes.do(job)

while True:
    schedule.run_pending()
    idle_for = schedule.idle_seconds()
    time.sleep(idle_for if idle_for is not None else 1)
```

## Core Usage

### Pass arguments to jobs

```python
import schedule

def greet(name: str, punctuation: str = "!") -> None:
    print(f"hello {name}{punctuation}")

schedule.every().day.at("09:00").do(greet, "team", punctuation=".")
```

### Use ranges and stop conditions

Randomized intervals:

```python
import schedule

def poll() -> None:
    print("polling")

schedule.every(5).to(10).seconds.do(poll)
```

Stop scheduling after a deadline:

```python
import schedule

def reminder() -> None:
    print("still active")

schedule.every().hour.until("18:30").do(reminder)
schedule.every().day.until("2030-01-01 00:00").do(reminder)
```

### Cancel jobs explicitly or from inside a job

```python
import schedule

def run_once():
    print("only once")
    return schedule.CancelJob

job = schedule.every().minute.do(run_once)

# Later:
schedule.cancel_job(job)
```

### Tag and clear jobs

```python
import schedule

def sync() -> None:
    print("sync")

schedule.every().minute.do(sync).tag("sync", "network")
schedule.every().hour.do(sync).tag("hourly")

schedule.clear("sync")
```

### Keep separate schedulers

Use `Scheduler()` when you need isolated job sets instead of the module-global default scheduler:

```python
import time

from schedule import Scheduler

fast = Scheduler()
slow = Scheduler()

fast.every(5).seconds.do(lambda: print("fast"))
slow.every().hour.do(lambda: print("slow"))

while True:
    fast.run_pending()
    slow.run_pending()
    time.sleep(1)
```

### Use the decorator helper

`@repeat(...)` is a convenience wrapper for no-argument callables:

```python
import time

import schedule
from schedule import repeat, every

@repeat(every(10).seconds)
def heartbeat() -> None:
    print("alive")

while True:
    schedule.run_pending()
    time.sleep(1)
```

It does not work on non-static class methods.

## Time And Timezone Configuration

Basic `.at()` examples:

```python
import schedule

schedule.every().day.at("12:42").do(lambda: print("lunch"))
schedule.every().hour.at(":17").do(lambda: print("minute 17"))
schedule.every().minute.at(":23").do(lambda: print("second 23"))
```

Timezone-aware scheduling:

```python
import schedule

schedule.every().day.at("12:42", "Europe/Amsterdam").do(lambda: print("ams"))
```

Notes:

- Timezone support in `.at()` requires `pytz`
- Internally, `schedule` stores naive datetimes in the process's local timezone
- Relative schedules such as `every(4).hours` are not DST-aware; only `.at()` handles timezone conversions

## Background Threads And Parallel Work

If your main thread is doing other work, run the scheduler loop in a background thread:

```python
import threading
import time

import schedule

def job() -> None:
    print("background")

def run_scheduler() -> None:
    while not stop_event.is_set():
        schedule.run_pending()
        time.sleep(1)

schedule.every(30).seconds.do(job)

stop_event = threading.Event()
thread = threading.Thread(target=run_scheduler, daemon=True)
thread.start()
```

For parallel execution, the docs recommend moving each job into its own thread or queueing work yourself. Long-running jobs block later jobs when you use the default scheduler loop.

## Error Handling And Logging

`schedule` does not catch exceptions raised by your jobs. If a job can fail, wrap it yourself so one exception does not terminate the scheduling loop:

```python
import functools
import logging
import time

import schedule

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def catch_exceptions(cancel_on_failure: bool = False):
    def decorator(job_func):
        @functools.wraps(job_func)
        def wrapper(*args, **kwargs):
            try:
                return job_func(*args, **kwargs)
            except Exception:
                logger.exception("scheduled job failed")
                if cancel_on_failure:
                    return schedule.CancelJob

        return wrapper

    return decorator

@catch_exceptions(cancel_on_failure=False)
def flaky_job() -> None:
    raise RuntimeError("boom")

schedule.every(10).seconds.do(flaky_job)

while True:
    schedule.run_pending()
    time.sleep(1)
```

## Common Pitfalls

- `schedule` is an in-process scheduler. Jobs disappear when the process exits.
- `run_pending()` does not "catch up" for missed intervals during downtime or long sleeps.
- Jobs run serially unless you explicitly add threads or a work queue.
- A blocking or slow job delays every later job on the same scheduler.
- `.at(..., timezone)` needs `pytz`; without it you will hit import or timezone-resolution errors.
- `@repeat` is convenient but limited; use `.do(...)` directly when you need arguments, methods, or more control.
- Time-sensitive production workflows often need monitoring, persistence, and retries that `schedule` does not provide.

## Version-Sensitive Notes For 1.2.2

- PyPI currently lists `schedule 1.2.2`, while the stable ReadTheDocs site is still branded `1.2.0`. Use the docs for API behavior, but treat PyPI as the current package-version source.
- The official history for `1.2.2` notes Python 3.12 support and timezone-related fixes; those changes are newer than the version banner shown on ReadTheDocs.
- The docs say the project is tested on Python 3.7 through 3.11, but PyPI classifiers include Python 3.12 for the current package release.

## Official Sources

- Docs: `https://schedule.readthedocs.io/en/stable/`
- Examples: `https://schedule.readthedocs.io/en/stable/examples.html`
- Reference: `https://schedule.readthedocs.io/en/stable/reference.html`
- Timezones: `https://schedule.readthedocs.io/en/stable/timezones.html`
- Background execution: `https://schedule.readthedocs.io/en/stable/background-execution.html`
- Parallel execution: `https://schedule.readthedocs.io/en/stable/parallel-execution.html`
- Exception handling: `https://schedule.readthedocs.io/en/stable/exception-handling.html`
- PyPI: `https://pypi.org/project/schedule/`
- Repository and history: `https://github.com/dbader/schedule`

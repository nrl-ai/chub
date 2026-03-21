---
name: package
description: "freezegun package guide for Python tests that freeze, tick, and move time deterministically"
metadata:
  languages: "python"
  versions: "1.5.5"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "freezegun,python,testing,time,mocking,datetime,pytest,unittest"
---

# freezegun Python Package Guide

## Golden Rule

Use `freezegun` to freeze time inside tests, not in production code. Prefer freezing the smallest scope that makes the assertion deterministic, and use explicit timestamps instead of relying on whatever the current clock happens to be.

As of March 11, 2026, the version used here `1.5.5` still matches the current PyPI release.

## Install

```bash
python -m pip install "freezegun==1.5.5"
```

Common alternatives:

```bash
uv add "freezegun==1.5.5"
poetry add "freezegun==1.5.5"
```

`freezegun` uses `python-dateutil` under the hood for string parsing, so readable date strings work, but ISO-style strings are safer in test code.

## What It Freezes

Once `freeze_time(...)` is active, Freezegun freezes these common clock reads:

- `datetime.datetime.now()`
- `datetime.datetime.utcnow()`
- `datetime.date.today()`
- `time.time()`
- `time.localtime()`
- `time.gmtime()`
- `time.strftime()`
- `time.monotonic()`
- `time.perf_counter()`

`time.monotonic()` and `time.perf_counter()` are frozen for relative changes, not meaningful absolute values.

## Core Usage

### Decorator

```python
from datetime import datetime

from freezegun import freeze_time

@freeze_time("2025-08-09 12:00:00")
def test_invoice_timestamp() -> None:
    assert datetime.now() == datetime(2025, 8, 9, 12, 0, 0)
```

### Context manager

```python
from datetime import datetime

from freezegun import freeze_time

def test_context_manager() -> None:
    with freeze_time("2025-08-09"):
        assert datetime.now() == datetime(2025, 8, 9, 0, 0, 0)
```

### Raw start/stop for setup-heavy tests

```python
from datetime import datetime

from freezegun import freeze_time

def test_manual_lifecycle() -> None:
    freezer = freeze_time("2025-08-09 15:30:00")
    freezer.start()
    try:
        assert datetime.now() == datetime(2025, 8, 9, 15, 30, 0)
    finally:
        freezer.stop()
```

## Common Test Patterns

### `unittest.TestCase`

Class decoration freezes time for test methods and the test case setup and teardown path:

```python
import datetime
import unittest

from freezegun import freeze_time

@freeze_time("1999-12-31 23:59:59")
class BillingTests(unittest.TestCase):
    def test_cutoff(self) -> None:
        self.assertEqual(
            datetime.datetime.now(),
            datetime.datetime(1999, 12, 31, 23, 59, 59),
        )
```

### Passing the freezer object into the test

Use `as_arg=True` or `as_kwarg=...` when the test needs to move time after entering the frozen scope:

```python
from datetime import datetime

from freezegun import freeze_time

@freeze_time("2025-08-09", as_arg=True)
def test_time_travel(frozen_time) -> None:
    assert datetime.now() == datetime(2025, 8, 9)
    frozen_time.move_to("2025-08-10")
    assert datetime.now() == datetime(2025, 8, 10)
```

### Supplying the target time dynamically

`freeze_time(...)` accepts strings, `date`, `datetime`, callables, and generators that yield datetimes:

```python
from datetime import datetime

from freezegun import freeze_time

def dynamic_now() -> datetime:
    return datetime(2025, 8, 9, 18, 45, 0)

def test_dynamic_source() -> None:
    with freeze_time(dynamic_now):
        assert datetime.now() == datetime(2025, 8, 9, 18, 45, 0)
```

## Time Control Options

### `tick=True`

Use this when time should start from a known instant but continue advancing:

```python
from datetime import datetime
import time

from freezegun import freeze_time

@freeze_time("2025-08-09 12:00:00", tick=True)
def test_tick_mode() -> None:
    start = datetime.now()
    time.sleep(0.01)
    assert datetime.now() > start
```

### `auto_tick_seconds=...`

Use this when every clock read should move forward by a fixed amount:

```python
from datetime import datetime, timedelta

from freezegun import freeze_time

@freeze_time("2025-08-09 12:00:00", auto_tick_seconds=15)
def test_auto_increment() -> None:
    first = datetime.now()
    second = datetime.now()
    assert second == first + timedelta(seconds=15)
```

### Manual `.tick(...)`

Use the freezer object when the test needs precise control:

```python
from datetime import datetime, timedelta

from freezegun import freeze_time

def test_manual_tick() -> None:
    with freeze_time("2025-08-09 12:00:00") as frozen_time:
        frozen_time.tick(delta=timedelta(minutes=5))
        assert datetime.now() == datetime(2025, 8, 9, 12, 5, 0)
```

### `.move_to(...)`

Jump directly to another date or timestamp:

```python
from datetime import datetime

from freezegun import freeze_time

def test_move_to() -> None:
    with freeze_time("2025-08-09 12:00:00") as frozen_time:
        frozen_time.move_to("2025-08-10 09:30:00")
        assert datetime.now() == datetime(2025, 8, 10, 9, 30, 0)
```

### `tz_offset=...`

Use `tz_offset` when the test cares about local time relative to UTC:

```python
from datetime import date, datetime, timedelta

from freezegun import freeze_time

@freeze_time("2025-08-09 03:00:00", tz_offset=-4)
def test_local_and_utc() -> None:
    assert datetime.utcnow() == datetime(2025, 8, 9, 3, 0, 0)
    assert datetime.now() == datetime(2025, 8, 8, 23, 0, 0)
    assert date.today() == date(2025, 8, 8)

@freeze_time("2025-08-09 03:00:00", tz_offset=-timedelta(hours=3, minutes=30))
def test_timedelta_offset() -> None:
    assert datetime.now() == datetime(2025, 8, 8, 23, 30, 0)
```

## Asyncio

`freezegun` also freezes `time.monotonic()`, which can break `asyncio.sleep()` and related scheduling if you do nothing else. For async tests, pass `real_asyncio=True` so the event loop still sees real monotonic time.

```python
import asyncio
from datetime import datetime

from freezegun import freeze_time

@freeze_time("2025-08-09 12:00:00", real_asyncio=True)
async def test_async_sleep() -> None:
    await asyncio.sleep(0.01)
    assert datetime.now() == datetime(2025, 8, 9, 12, 0, 0)
```

## Ignore Lists And Global Configuration

Ignore a package for one frozen scope:

```python
from freezegun import freeze_time

with freeze_time("2025-08-09", ignore=["threading"]):
    pass
```

Set a new default ignore list globally:

```python
import freezegun

freezegun.configure(default_ignore_list=["threading", "tensorflow"])
```

Extend the built-in ignore list instead of replacing it:

```python
import freezegun

freezegun.configure(extend_ignore_list=["tensorflow"])
```

The built-in defaults shown in the maintainer docs include modules such as `threading`, `selenium`, `_pytest.terminal.`, and `_pytest.runner.`.

## Common Pitfalls

- Default argument values are evaluated before Freezegun starts. If a function uses `default=date.today()` or similar, freezing time later will not change that captured value.
- `auto_tick_seconds` overrides `tick`. If both are set, Freezegun ignores `tick`.
- Generator inputs are consumable. Reusing the same generator across multiple `freeze_time(...)` calls can end in `StopIteration`.
- The arbitrary-class decorator form may not work in every case. It is most reliable on test functions and `unittest.TestCase` classes.
- If your test or framework depends on the event loop clock, use `real_asyncio=True`.
- Overriding `default_ignore_list` replaces the defaults. Use `extend_ignore_list` if you only want to add entries.
- Prefer explicit timestamps in assertions. Natural-language date strings are convenient, but ISO timestamps are less ambiguous in tests and agent-written code.

## Version-Sensitive Notes

- The version used here `1.5.5` matches the current PyPI release on March 11, 2026, so no version drift was found during review.
- Freezegun's maintained documentation is README-based. The GitHub README and the PyPI long description are effectively the canonical docs, so check those before trusting older blog posts.
- The current API documented by the maintainer includes `tick`, `auto_tick_seconds`, `move_to`, `as_arg`, `as_kwarg`, and `real_asyncio`. Older examples often omit the async guidance.
- PyPI shows the `1.5.5` release was published on August 9, 2025. If you need behavior identical to this guide, pin `freezegun==1.5.5`.

## Official Sources

- GitHub README: `https://github.com/spulec/freezegun`
- PyPI project page: `https://pypi.org/project/freezegun/`
- PyPI JSON metadata: `https://pypi.org/pypi/freezegun/json`

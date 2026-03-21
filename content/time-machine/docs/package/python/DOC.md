---
name: package
description: "time-machine package guide for Python tests that need deterministic time travel and pytest integration"
metadata:
  languages: "python"
  versions: "3.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "time-machine,python,testing,datetime,time,pytest,mocking"
---

# time-machine Python Package Guide

## Golden Rule

Use `time-machine` only in test code or tightly scoped development helpers. It patches interpreter-level time functions, so treat every travel block as global process state and keep the scope as small as possible.

## Install

Pin the package version your test environment expects:

```bash
python -m pip install "time-machine==3.2.0"
```

Optional extras published by the maintainer:

```bash
python -m pip install "time-machine[dateutil]==3.2.0"
python -m pip install "time-machine[cli]==3.2.0"
```

Common alternatives:

```bash
uv add --dev "time-machine==3.2.0"
poetry add --group test "time-machine==3.2.0"
```

## Core Usage

`time_machine.travel()` is the main API. You can use it as a context manager, decorator, or manual controller.

### Freeze time in a context manager

```python
from datetime import datetime, timezone

import time_machine

with time_machine.travel(datetime(2024, 5, 1, 12, 0, tzinfo=timezone.utc), tick=False):
    assert datetime.now(timezone.utc).isoformat() == "2024-05-01T12:00:00+00:00"
```

Use `tick=False` when your assertions need a fixed instant. The default is `tick=True`, which means time advances from the destination after travel starts.

### Use as a decorator

```python
import datetime as dt

import time_machine

@time_machine.travel(dt.datetime(2024, 7, 4, 9, 30, tzinfo=dt.timezone.utc), tick=False)
def test_report_timestamp() -> None:
    assert dt.date.today().isoformat() == "2024-07-04"
```

### Manual start/stop when setup spans multiple assertions

```python
import datetime as dt

import time_machine

traveller = time_machine.travel(dt.datetime(2025, 1, 1, tzinfo=dt.timezone.utc), tick=False)
traveller.start()
try:
    # multiple calls share the same travelled time
    ...
finally:
    traveller.stop()
```

## Moving During A Test

The started traveller object exposes `move_to()` and `shift()` so a single test can advance through multiple moments.

```python
from datetime import datetime, timedelta, timezone

import time_machine

traveller = time_machine.travel(datetime(2024, 1, 1, tzinfo=timezone.utc), tick=False)
traveller.start()
try:
    traveller.shift(timedelta(hours=2))
    traveller.move_to(datetime(2024, 1, 2, tzinfo=timezone.utc))
finally:
    traveller.stop()
```

This is usually cleaner than nesting multiple travel blocks.

## Accepted Destination Types

The maintainer docs allow several destination forms:

- A timezone-aware `datetime`
- A `date`
- A Unix timestamp (`int` or `float`)
- A `timedelta` offset from the current time
- A string parsed by `dateutil`, if you install the `dateutil` extra
- A callable or generator that yields destinations for repeated test setup

Prefer timezone-aware `datetime` values for deterministic tests. They make the intended zone explicit and avoid local-time surprises.

## Pytest Integration

The package ships a pytest plugin with:

- a `time_machine` fixture that returns a started traveller
- a `@pytest.mark.time_machine(...)` marker for per-test travel

Example fixture usage:

```python
from datetime import datetime, timezone

def test_expiry(time_machine) -> None:
    time_machine.move_to(datetime(2024, 9, 1, tzinfo=timezone.utc), tick=False)
    ...
```

Example marker usage:

```python
from datetime import datetime, timezone

import pytest

@pytest.mark.time_machine(datetime(2024, 9, 1, tzinfo=timezone.utc), tick=False)
def test_marker_example() -> None:
    ...
```

If your tests already rely on fixture setup, the fixture is usually easier to compose than decorators.

## Configuration And Environment

There is no auth or external service setup. The important configuration choices are behavioral:

- `tick`: defaults to `True`; set `False` when you need a frozen clock
- destination timezone: prefer aware UTC datetimes unless local time is the behavior under test
- `naive_mode`: controls how naive datetimes are interpreted; use the default only if you understand your project’s local-time assumptions

On Unix, time travel also updates the process timezone via `time.tzset()` when needed. That matters if your code reads local time or timezone-dependent formatting.

## Escape Hatch

The package exposes `time_machine.escape_hatch` for code that must read the real clock while travel is active.

```python
import time_machine

real_now = time_machine.escape_hatch.datetime.datetime.now()
travel_active = time_machine.escape_hatch.is_travelling()
```

Use this sparingly. If production code needs real time while tests are travelling, that is often a design smell worth isolating behind a clock abstraction.

## Common Pitfalls

- Travel state is global within the current process. Threads and concurrent tasks see the travelled time too.
- Child processes, external services, databases, and other machines do not inherit the travelled clock.
- Default `tick=True` can make equality assertions flaky. Freeze time with `tick=False` unless advancing time is intentional.
- Naive datetimes are easy to misread. Use timezone-aware datetimes when possible.
- String destinations require `python-dateutil`; without the extra installed, parsing helpers are unavailable.
- Class decorators are primarily for `unittest.TestCase` classes. For pytest, prefer the fixture or marker.

## Version-Sensitive Notes

- PyPI currently lists `3.2.0` and requires Python `>=3.10`.
- The project is CPython-only because it patches C-level time functions for speed and broader coverage.
- If you are migrating from `freezegun`, check the maintainer migration guide before doing a drop-in replacement. `time-machine` defaults and behavior differ in a few places, especially around ticking and parser-backed string destinations.

## Official Sources

- PyPI: https://pypi.org/project/time-machine/
- Docs: https://time-machine.readthedocs.io/en/latest/
- Installation: https://time-machine.readthedocs.io/en/latest/installation.html
- Usage: https://time-machine.readthedocs.io/en/latest/usage.html
- Pytest plugin: https://time-machine.readthedocs.io/en/latest/pytest_plugin.html
- Migration guide: https://time-machine.readthedocs.io/en/latest/migration.html
- Comparison: https://time-machine.readthedocs.io/en/latest/comparison.html
- Repository: https://github.com/adamchainz/time-machine

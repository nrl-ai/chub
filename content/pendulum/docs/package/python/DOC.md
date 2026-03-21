---
name: package
description: "Pendulum Python datetime library for timezone-aware datetimes, parsing, formatting, durations, intervals, and test time travel"
metadata:
  languages: "python"
  versions: "3.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pendulum,python,datetime,timezones,duration,interval,testing"
---

# Pendulum Python Package Guide

## Golden Rule

Use `pendulum` when you need timezone-aware datetime handling, humanized diffs, or interval and duration helpers in Python. Keep timezone behavior explicit with `tz=...`, prefer IANA timezone names, and treat the official docs site as a generic 3.x reference rather than a patch-specific `3.2.0` manual.

## Install

Pin the version your project expects:

```bash
python -m pip install "pendulum==3.2.0"
```

Common alternatives:

```bash
uv add "pendulum==3.2.0"
poetry add "pendulum==3.2.0"
```

Install the optional test helpers when you need time travel in tests:

```bash
python -m pip install "pendulum[test]==3.2.0"
poetry add "pendulum[test]==3.2.0"
```

## Initialization And Basic Setup

Pendulum has no auth or service configuration layer. The main setup decisions are:

- which timezone you want to use by default in your code paths
- whether you need localized formatting or `diff_for_humans()`
- whether your test environment needs the `test` extra

Basic creation patterns:

```python
import pendulum

utc_now = pendulum.now("UTC")
local_now = pendulum.now()

meeting = pendulum.datetime(2026, 3, 12, 9, 30, tz="America/Los_Angeles")
local_meeting = pendulum.local(2026, 3, 12, 9, 30)

print(utc_now)
print(meeting.timezone_name)
print(local_meeting.timezone_name)
```

Important defaults:

- `pendulum.datetime(...)` defaults to `UTC` unless you pass `tz=...`
- `pendulum.now()` uses the current local timezone unless you pass a timezone
- the `tz` argument is keyword-only
- timezone strings should be IANA names like `UTC`, `Europe/Paris`, or `America/New_York`

## Core Usage

### Create, convert, and normalize datetimes

```python
import pendulum

start = pendulum.datetime(2026, 3, 12, 9, 30, tz="America/Los_Angeles")
paris = start.in_timezone("Europe/Paris")

# set(tz=...) replaces timezone info without converting the instant
retagged = start.set(tz="UTC")

print(start.to_iso8601_string())
print(paris.to_iso8601_string())
print(retagged.to_iso8601_string())
```

Use `in_timezone()` or `in_tz()` when you want a real conversion. Use `set(tz=...)` only when you intentionally want to retag the same wall-clock value.

Pendulum normalizes DST-transition timestamps for the given timezone:

```python
import pendulum

dt = pendulum.datetime(2013, 3, 31, 2, 30, tz="Europe/Paris")
print(dt)  # normalized to the actual valid time in that zone
```

### Parse strings and timestamps

```python
import pendulum

dt1 = pendulum.parse("2026-03-12T17:45:00Z")
dt2 = pendulum.parse("2026-03-12 17:45:00", tz="America/New_York")
dt3 = pendulum.from_format("2026-03-12 17", "YYYY-MM-DD HH", tz="UTC")
dt4 = pendulum.from_timestamp(1_773_343_500, tz="UTC")

print(dt1, dt2, dt3, dt4)
```

Use `from_format()` when the input is not a clean ISO 8601 or RFC 3339 string. `parse()` is strict by default:

```python
import pendulum

parsed = pendulum.parse("31-01-01", strict=False)
date_only = pendulum.parse("2026-03-12", exact=True)
time_only = pendulum.parse("17:45:00", exact=True)

print(parsed)
print(type(date_only).__name__)
print(type(time_only).__name__)
```

`strict=False` allows a fallback parser for ambiguous non-standard strings. `exact=True` returns `Date` or `Time` when the input represents only that type.

### Format and localize output

```python
import pendulum

dt = pendulum.datetime(2026, 3, 12, 17, 45, tz="UTC")

print(dt.to_datetime_string())
print(dt.format("dddd DD MMMM YYYY HH:mm:ss", locale="fr"))
print(dt.diff_for_humans(locale="de"))
```

If you need consistent locale output across a process, you can set a global locale:

```python
import pendulum

pendulum.set_locale("en")
```

Prefer per-call `locale=...` when writing library code or test suites to avoid global state surprises.

### Arithmetic, durations, and intervals

```python
import pendulum

start = pendulum.datetime(2026, 3, 12, 9, 30, tz="UTC")
end = start.add(days=3, hours=6)

interval = end - start
duration = pendulum.duration(days=2, hours=5, minutes=30)

print(start.add(weeks=1))
print(start.start_of("day"))
print(end.diff_for_humans(start, absolute=True))
print(interval.in_hours())
print(duration.in_words())
```

When you subtract two `DateTime` values or call `diff()`, Pendulum returns an `Interval`, which tracks the start and end datetimes. Many arithmetic operations on an `Interval` return a `Duration` instead.

Iterating over an interval:

```python
import pendulum

start = pendulum.datetime(2026, 3, 1, tz="UTC")
end = pendulum.datetime(2026, 3, 5, tz="UTC")
interval = pendulum.interval(start, end)

for dt in interval.range("days"):
    print(dt.to_date_string())
```

### Interop with standard `datetime`

```python
from datetime import datetime
import pendulum

native = datetime(2026, 3, 12, 17, 45)
dt = pendulum.instance(native)

print(dt)
print(isinstance(dt, datetime))
```

Pendulum objects are subclasses of the standard library date/time types, but some integrations still do strict type checks.

## Configuration Notes

- There is no API key, network auth, or credentials setup.
- Timezone identifiers come from the IANA timezone database.
- `pendulum.datetime()` defaults to `UTC`; `pendulum.local()` and `pendulum.now()` are the local-timezone entry points.
- The `test` extra is required for `travel()`, `travel_to()`, and `travel_back()`.
- Prefer per-call `locale=...` over global `pendulum.set_locale()` unless your process really wants one locale everywhere.

## Testing With Time Travel

Pendulum 3.x test helpers live behind the optional `test` extra:

```python
import pendulum

with pendulum.travel(minutes=5, freeze=True):
    print(pendulum.now())

target = pendulum.datetime(2026, 3, 12, 9, 30, tz="UTC")
pendulum.travel_to(target, freeze=True)
print(pendulum.now())
pendulum.travel_back()
```

Without `freeze=True`, the clock keeps ticking from the traveled point. Use the context-manager form when possible so cleanup is automatic.

## Common Pitfalls

- `tz` is keyword-only. `pendulum.datetime(2026, 3, 12, "UTC")` is wrong; use `tz="UTC"`.
- `set(tz=...)` is not the same as `in_timezone(...)`. `set()` retags the timezone, while `in_timezone()` converts the instant.
- `parse()` is strict for non-standard inputs. For custom formats, prefer `from_format()`, or use `strict=False` only when fallback parsing is acceptable.
- `pendulum.datetime(...)` defaults to UTC, not local time. Agents often assume local time and silently shift schedules.
- Locale can be global state. `pendulum.set_locale()` affects later humanized output in the same process.
- Some libraries and database adapters use `type()` checks instead of `isinstance()`. The upstream docs call out `sqlite3`, `mysqlclient`, `PyMySQL`, and Django as cases where adapters or native conversion may be needed.
- Interval arithmetic can change type. If you need start/end-aware behavior after math, confirm you still have an `Interval` and not a plain `Duration`.

## Version-Sensitive Notes For 3.2.0

- The official docs site currently presents itself as "Version 3.0" and a generic `3.x` reference, but PyPI publishes `pendulum 3.2.0`. Treat the site as the maintained 3.x manual, not a patch-level release mirror.
- The published package metadata and the repository `pyproject.toml` declare `Python >=3.9`. The current repository README says "Supports Python 3.10 and newer", so prefer the published package metadata when checking install compatibility for `3.2.0`.
- Pendulum 3.x removed the old Pendulum 2 testing helpers `set_test_now()` and `test()`. Use `travel()`, `travel_to()`, and `travel_back()` instead.
- The 3.x line uses `zoneinfo.ZoneInfo` internally rather than the old `pytz`-style timezone model. Avoid porting older blog examples that assume pre-3.x internals.
- The upstream changelog currently documents the 3.0.0 and 3.1.0 changes, but it does not yet describe a dedicated `3.2.0` section. For `3.2.0`, rely on the docs, PyPI metadata, and the published package itself instead of assuming a richer patch-note page exists.

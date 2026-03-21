---
name: package
description: "Arrow Python package guide for creating, parsing, shifting, formatting, and localizing datetimes"
metadata:
  languages: "python"
  versions: "1.4.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "arrow,datetime,date,time,timezone,humanize,localization"
---

# Arrow Python Package Guide

## Golden Rule

Use `arrow` when you want a friendlier datetime API, but still treat timezone handling as explicit application logic: create or normalize values in UTC, convert to local zones at the boundaries, and prefer full IANA timezone names such as `America/Los_Angeles` over ambiguous abbreviations such as `PST` or `MST`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "arrow==1.4.0"
```

Common alternatives:

```bash
uv add "arrow==1.4.0"
poetry add "arrow==1.4.0"
```

PyPI also publishes `doc` and `test` extras, but normal application code usually just needs the base package.

## Initialize And Create Values

Arrow is a library, not a network client, so there is no service authentication step. Setup is just installation plus choosing consistent timezone rules in your app.

Common creation patterns:

```python
from datetime import datetime

import arrow

now_utc = arrow.utcnow()
now_local = arrow.now()
pacific_now = arrow.now("America/Los_Angeles")

from_timestamp = arrow.get(1710000000)
from_iso = arrow.get("2026-03-12T18:30:00+00:00")
from_formatted = arrow.get("2026-03-12 18:30:00", "YYYY-MM-DD HH:mm:ss")
from_datetime = arrow.get(datetime(2026, 3, 12, 18, 30, 0))
```

Important behavior:

- `arrow.utcnow()` returns an aware UTC value.
- `arrow.now()` returns an aware value in the local timezone unless you pass a timezone.
- `arrow.get(naive_datetime)` treats a naive `datetime` as UTC.

## Core Usage

### Parse, normalize, and format

```python
import arrow

event = arrow.get("2026-03-12 10:15:00", "YYYY-MM-DD HH:mm:ss")
event_utc = event.to("UTC")

print(event_utc.isoformat())
print(event_utc.format("YYYY-MM-DD HH:mm:ss ZZ"))
print(event_utc.format(arrow.FORMAT_RFC3339))
```

Arrow format strings are token-based (`YYYY`, `MM`, `DD`, `HH`, `ZZ`), not Python `strftime` directives.

### Shift or replace components

Use `shift()` for relative arithmetic and `replace()` for absolute component changes:

```python
import arrow

base = arrow.get("2026-03-12T18:30:00+00:00")

next_week = base.shift(weeks=1)
two_hours_ago = base.shift(hours=-2)
same_day_at_noon = base.replace(hour=12, minute=0, second=0, microsecond=0)
```

### Convert timezones

```python
import arrow

utc_value = arrow.utcnow()
la_value = utc_value.to("America/Los_Angeles")
berlin_value = utc_value.to("Europe/Berlin")
```

Prefer full IANA names. The upstream docs explicitly warn that abbreviations such as `MST`, `PDT`, and `BRST` are ambiguous and may fail to parse.

### Humanize and dehumanize

```python
import arrow

present = arrow.utcnow()
future = present.shift(minutes=66)

print(future.humanize(present))
print(future.humanize(present, only_distance=True, granularity=["hour", "minute"]))

earlier = present.dehumanize("2 days ago")
later = present.dehumanize("in a month")
```

This is useful for UI text, but do not treat `humanize()` output as a stable machine-readable format.

### Generate spans and ranges

```python
from datetime import datetime

import arrow

start_of_hour, end_of_hour = arrow.utcnow().span("hour")
start_of_week = arrow.utcnow().floor("week", week_start=7)

for point in arrow.Arrow.range(
    "hour",
    datetime(2026, 3, 12, 9, 0),
    datetime(2026, 3, 12, 12, 0),
):
    print(point)
```

Use `span()` when you need a closed interval for a unit, `floor()` and `ceil()` when you only need one bound, and `Arrow.range()` or `Arrow.span_range()` for iteration.

## Practical Pattern

This is a good default workflow for application code:

```python
import arrow

raw_created_at = "2026-03-12T18:30:00-04:00"

created_at = arrow.get(raw_created_at).to("UTC")
expires_at = created_at.shift(days=30)
display_value = expires_at.to("America/Los_Angeles").format("YYYY-MM-DD HH:mm:ss ZZ")

print(created_at)
print(expires_at)
print(display_value)
```

Parse once, normalize to UTC for storage and logic, then convert only for display or user-facing output.

## Configuration Notes

- There is no package-level auth or API configuration.
- Timezone strings are the main configuration surface. Prefer canonical IANA names such as `UTC`, `Europe/London`, or `America/Chicago`.
- On Python 3.9+, `zoneinfo.ZoneInfo` is a good explicit timezone object choice. On Python 3.8, passing timezone names as strings is the most portable option.
- Locale matters for `humanize()` and `dehumanize()`. Set `locale=...` explicitly when your app output is not English.
- If parsing messy input, `arrow.get(..., normalize_whitespace=True)` can help with tabs, newlines, or inconsistent spacing.

## Common Pitfalls

- `arrow.now()` is local time, while `arrow.utcnow()` is UTC. Mixing them without conversion produces subtle bugs.
- `arrow.get(naive_datetime)` assumes UTC. If a naive `datetime` actually represents local wall time, attach or replace the timezone before converting.
- Arrow formatting tokens are not `strftime` tokens. `YYYY-MM-DD` works in Arrow; `%Y-%m-%d` does not.
- Timezone abbreviations are not reliable parse inputs. Use `America/New_York`, not `EST`.
- `humanize()` is for presentation. Do not parse it later or store it as a durable value.
- `Arrow.range()` and `Arrow.span_range()` return iterators. Materialize them with `list(...)` only when you actually need all values in memory.

## Version-Sensitive Notes For 1.4.0

- PyPI and the official docs both identify `1.4.0` as the current version, released on `2025-10-18`.
- The `1.4.0` changelog adds a `week_start` parameter to `floor()` and `ceil()`, which matters when your business logic treats Sunday instead of Monday as the first day of the week.
- `1.4.0` also adds `arrow.FORMAT_RFC3339_STRICT`, which is useful if you need an RFC 3339 format with a `T` separator.
- The `1.4.0` changelog says Arrow migrated to `ZoneInfo` for timezones instead of `pytz`.
- Inference from that change: if older code depended on `pytz`-specific behavior around DST transitions or timezone object identity, retest those paths when upgrading to `1.4.0`.
- The same release notes mention a fix for the `datetime.utcnow` deprecation warning, so codebases moving to newer Python versions should prefer `1.4.0` over older Arrow releases.

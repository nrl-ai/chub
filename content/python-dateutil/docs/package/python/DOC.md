---
name: package
description: "python-dateutil for Python: datetime parsing, calendar-aware arithmetic, recurrence rules, and timezone helpers"
metadata:
  languages: "python"
  versions: "2.9.0.post0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python-dateutil,dateutil,python,datetime,timezone,parsing,rrule,relativedelta"
---

# python-dateutil Python Package Guide

## Golden Rules

- Install `python-dateutil`, but import from `dateutil`.
- Prefer `parser.isoparse()` for strict ISO-8601 input and use `parser.parse()` when the input is loose or human-written.
- Use `relativedelta` for calendar-aware arithmetic. `timedelta(days=30)` is not a substitute for "next month".
- Use `rrule` for recurring schedules and `dateutil.tz` helpers for timezone-aware and DST-aware datetime handling.

## Installation

```bash
pip install python-dateutil
```

```bash
pip install "python-dateutil==2.9.0.post0"
```

```bash
uv add python-dateutil
```

```bash
poetry add python-dateutil
```

## Initialization and Imports

There is no client object, authentication flow, or service setup. Import only the submodules you need.

```python
from datetime import datetime

from dateutil import tz
from dateutil.parser import isoparse, parse
from dateutil.relativedelta import MO, relativedelta
from dateutil.rrule import DAILY, MONTHLY, WE, rrule
```

On Python 3.7+, the `2.9.x` line lazily imports submodules, so this also works:

```python
import dateutil

pacific = dateutil.tz.gettz("America/Los_Angeles")
```

## Core Usage

### Parse loose date and time strings

Use `parse()` when the input format is inconsistent or user-provided.

```python
from dateutil.parser import parse

dt = parse(
    "2026-03-11 09:30 PST",
    dayfirst=False,
    yearfirst=True,
)
```

Useful parser controls:

- `default=` fills missing components from a fallback `datetime`
- `ignoretz=True` ignores timezone information and returns a naive `datetime`
- `tzinfos=` maps custom abbreviations to offsets or `tzinfo` objects
- `fuzzy=True` ignores extra words
- `fuzzy_with_tokens=True` returns both the parsed `datetime` and the ignored text

Example with custom timezone aliases:

```python
from dateutil.parser import parse
from dateutil.tz import gettz

tzinfos = {
    "PT": gettz("America/Los_Angeles"),
    "ET": gettz("America/New_York"),
}

dt = parse("2026-03-11 09:30 PT", tzinfos=tzinfos)
```

### Parse strict ISO-8601 input

Use `isoparse()` for API payloads, JSON timestamps, and other machine-generated input.

```python
from dateutil.parser import isoparse

created_at = isoparse("2026-03-11T09:30:00Z")
week_date = isoparse("2026-W11-3T09:30:00+00:00")
midnight_next_day = isoparse("2026-03-11T24:00")
```

`isoparse()` is stricter than `parse()` and handles ISO week dates and fractional seconds with either `.` or `,`.

### Do calendar-aware arithmetic with `relativedelta`

`relativedelta` handles months, years, weekdays, and month-end rollover.

```python
from datetime import datetime
from dateutil.relativedelta import MO, relativedelta

dt = datetime(2026, 1, 31, 10, 0)

same_clock_next_month = dt + relativedelta(months=+1)
first_monday_next_month = dt + relativedelta(months=+1, day=1, weekday=MO(1))
```

Important rule:

- Singular arguments like `year=`, `month=`, `day=` are absolute replacements.
- Plural arguments like `years=`, `months=`, `days=` are arithmetic deltas.

```python
from datetime import datetime
from dateutil.relativedelta import relativedelta

dt = datetime(2026, 3, 11, 9, 30)

replaced = dt + relativedelta(day=1)      # 2026-03-01 09:30
shifted = dt + relativedelta(days=+1)     # 2026-03-12 09:30
```

### Generate recurring schedules with `rrule`

Use `rrule` for iCalendar-style recurrence sets.

```python
from datetime import datetime
from dateutil.rrule import MONTHLY, WE, rrule

meetings = rrule(
    MONTHLY,
    count=4,
    byweekday=WE(2),
    dtstart=datetime(2026, 1, 1, 9, 0),
)

for dt in meetings:
    print(dt)
```

The frequency constants are `YEARLY`, `MONTHLY`, `WEEKLY`, `DAILY`, `HOURLY`, `MINUTELY`, and `SECONDLY`.

If you will iterate the same recurrence many times, enable caching:

```python
from datetime import datetime
from dateutil.rrule import DAILY, rrule

events = rrule(DAILY, count=365, cache=True, dtstart=datetime(2026, 1, 1))
```

### Work with time zones and DST transitions

Use `gettz()` for named zones and `UTC` for a canonical UTC tzinfo.

```python
from datetime import datetime
from dateutil.tz import UTC, datetime_ambiguous, datetime_exists, gettz, resolve_imaginary

nyc = gettz("America/New_York")
dt = datetime(2026, 3, 8, 2, 30, tzinfo=nyc)

if not datetime_exists(dt):
    dt = resolve_imaginary(dt)

is_fold = datetime_ambiguous(dt)
utc_dt = dt.astimezone(UTC)
```

Repeated calls to `gettz()` with the same zone string return the same object, which helps preserve "same zone" semantics in comparisons and conversions.

## Configuration and Environment

There is no auth or remote service configuration layer.

Common project-level configuration is limited to:

- parser behavior: `dayfirst`, `yearfirst`, `default`, `tzinfos`, `fuzzy`
- timezone identifiers passed to `tz.gettz()`
- recurrence parameters passed to `rrule()`

If your code depends on local-machine timezone resolution, remember that `gettz()` with no arguments reads the host environment. Container, CI, and developer-laptop defaults can differ.

## Common Pitfalls

### Install name and import name differ

Install `python-dateutil`, but write imports against `dateutil`.

### `parse()` is intentionally forgiving

`parse()` will accept many ambiguous or partially specified inputs. For stable machine input, prefer `isoparse()` or `datetime.fromisoformat()` when the format is fixed.

### `default=` can silently change missing fields

When parsing a partial date, missing components come from `default=`. If that produces an invalid day, `parse()` falls back to the end of the month. Test partial-date behavior explicitly.

### Ambiguous dates need an explicit policy

Inputs like `01/02/03` are ambiguous. Set `dayfirst` and `yearfirst` explicitly instead of relying on defaults.

### `ignoretz=True` discards timezone information

If the source string includes an offset or named timezone, `ignoretz=True` returns a naive `datetime`. That is lossy.

### `relativedelta` singular vs plural changes behavior

`month=1` means "set month to January". `months=+1` means "add one calendar month". Mixing them is valid, but easy to misread.

### `rrule` skips invalid instances

Per RFC 5545, invalid dates and nonexistent local times are ignored rather than coerced. End-of-month schedules and DST transitions can produce fewer occurrences than you expect.

### Do not mix `count` and `until`

The docs deprecate using `count` and `until` together in the same recurrence rule to stay aligned with RFC 5545 semantics.

### `gettz()` can return `None`

Unknown timezone strings are not resolved automatically. Check the return value before attaching it to a `datetime`.

## Version-Sensitive Notes

- This doc is pinned to `2.9.0.post0`, the version used here and the current PyPI release for `python-dateutil`.
- `2.9.0.post0` is a post release. The maintainer release notes describe it as a packaging-only fix for `setuptools_scm` compatibility, so the runtime guidance here is effectively the same as `2.9.0`.
- The `2.9.0` changelog notes three practical behavior changes relevant to agents:
  - submodules are lazily imported on Python 3.7+
  - deprecated `datetime.utcfromtimestamp` usage was removed for Python 3.12 compatibility
  - the `relativedelta` docs were clarified around month-end behavior
- The stable Read the Docs site is the right API reference, but its page title/version text is not a reliable package-version signal for this release line. Use PyPI and maintainer release tags when pinning versions.

## Official Sources

- Docs root: https://dateutil.readthedocs.io/en/stable/
- Parser reference: https://dateutil.readthedocs.io/en/stable/parser.html
- `relativedelta` reference: https://dateutil.readthedocs.io/en/stable/relativedelta.html
- `rrule` reference: https://dateutil.readthedocs.io/en/stable/rrule.html
- Time zone reference: https://dateutil.readthedocs.io/en/stable/tz.html
- Changelog: https://dateutil.readthedocs.io/en/stable/changelog.html
- PyPI package page: https://pypi.org/project/python-dateutil/
- Maintainer release notes: https://github.com/dateutil/dateutil/releases/tag/2.9.0.post0

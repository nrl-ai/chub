---
name: package
description: "pytz guide for Python projects: installation, timezone conversion, DST-safe localization, and maintenance-mode migration notes"
metadata:
  languages: "python"
  versions: "2026.1.post1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,pytz,datetime,timezone,dst,iana,zoneinfo"
---

# pytz Python Package Guide

## Golden Rule

Use `pytz` when you need compatibility with code that already depends on `pytz` timezone objects. For new Python 3.9+ code, the maintainer explicitly recommends the standard-library `zoneinfo` stack instead; `pytz` mainly remains useful for backwards compatibility.

The two `pytz` rules that matter most are:

1. Localize naive datetimes with `tz.localize(...)`.
2. Normalize localized datetimes after arithmetic that may cross a DST boundary.

## Install

```bash
pip install pytz
```

Pin the package when you need reproducible timezone data:

```bash
pip install "pytz==2026.1.post1"
```

## Initialization

There is no service configuration, API key, or account setup. Initialization is just choosing the timezone object you want to work with.

```python
import pytz

utc = pytz.UTC
eastern = pytz.timezone("America/New_York")
tokyo = pytz.timezone("Asia/Tokyo")
```

For UTC-only workflows, `pytz.UTC` is the safest default timezone object.

## Core Usage

### Localize a naive datetime

```python
from datetime import datetime
import pytz

eastern = pytz.timezone("America/New_York")
naive = datetime(2026, 1, 15, 9, 30)
local_dt = eastern.localize(naive)

print(local_dt.isoformat())
```

Do not build most localized datetimes with `datetime(..., tzinfo=tz)`. Upstream documents that this produces incorrect historical or DST-sensitive offsets for many zones.

### Convert between timezones

```python
from datetime import datetime
import pytz

utc_dt = datetime(2026, 1, 15, 14, 0, tzinfo=pytz.UTC)
tokyo = pytz.timezone("Asia/Tokyo")
tokyo_dt = utc_dt.astimezone(tokyo)

print(tokyo_dt.isoformat())
```

Use `.astimezone(...)` for aware datetimes. `localize(...)` is only for naive datetimes.

### Normalize after arithmetic across DST boundaries

```python
from datetime import datetime, timedelta
import pytz

eastern = pytz.timezone("America/New_York")
start = eastern.localize(datetime(2026, 11, 1, 0, 30))
later = start + timedelta(hours=3)
fixed = eastern.normalize(later)

print(start.isoformat())
print(later.isoformat())  # may keep the old offset
print(fixed.isoformat())  # corrected for the DST transition
```

This is the other major `pytz` gotcha. Arithmetic happens on the `datetime`, so crossing a timezone transition can leave the offset stale until you call `normalize(...)`.

### Prefer UTC for storage and internal logic

```python
from datetime import datetime
import pytz

created_at = datetime.now(tz=pytz.UTC)
display_tz = pytz.timezone("Europe/Amsterdam")

print(created_at.astimezone(display_tz).isoformat())
```

The upstream docs recommend keeping internal timestamps in UTC and converting to local time only at input and display boundaries.

## DST and Ambiguous Times

### Refuse to guess during ambiguous local times

```python
from datetime import datetime
import pytz

eastern = pytz.timezone("America/New_York")
naive = datetime(2026, 11, 1, 1, 30)

try:
    dt = eastern.localize(naive, is_dst=None)
except pytz.AmbiguousTimeError:
    # Choose a policy for your app instead of guessing silently.
    dt = eastern.localize(naive, is_dst=False)
```

`is_dst=None` is the safest choice when you want `pytz` to raise instead of silently choosing one of the two possible local times.

### Handle nonexistent local times during spring-forward

```python
from datetime import datetime
import pytz

pacific = pytz.timezone("America/Los_Angeles")

try:
    dt = pacific.localize(datetime(2026, 3, 8, 2, 30), is_dst=None)
except pytz.NonExistentTimeError:
    # This wall-clock time was skipped by the DST transition.
    dt = pacific.localize(datetime(2026, 3, 8, 3, 0), is_dst=True)
```

If your application accepts local wall-clock input, plan an explicit policy for ambiguous and nonexistent times instead of relying on the library default.

## Common Setup Patterns

### Validate a timezone name from user input

```python
import pytz

def get_timezone(name: str):
    if name not in pytz.common_timezones_set:
        raise ValueError(f"Unsupported timezone: {name}")
    return pytz.timezone(name)
```

Useful built-ins:

- `pytz.timezone(name)` returns a tzinfo object for an IANA timezone name.
- `pytz.UTC` and `pytz.utc` are the canonical UTC timezone objects.
- `pytz.common_timezones` and `pytz.common_timezones_set` are better user-facing choices than the full exhaustive timezone list.

## Config and Auth

- No authentication is required.
- No network setup is required at runtime.
- The package ships timezone data and Python helpers; configuration is mainly about dependency pinning and choosing a consistent timezone policy.
- If you integrate with frameworks or older libraries, confirm whether they still expect `pytz` objects or now use `zoneinfo`. Mixing both models in one code path is a common source of subtle bugs.

## Common Pitfalls

- Do not create local times with `datetime(..., tzinfo=pytz.timezone("America/New_York"))`.
- Do not call `localize(...)` on a datetime that already has `tzinfo`.
- Do not skip `normalize(...)` after arithmetic across DST or other timezone transitions.
- Do not assume abbreviations like `EST` or `CST` are unique; use IANA names such as `America/New_York`.
- Do not silently guess through ambiguous or nonexistent wall-clock times if correctness matters.
- Do not choose `pytz` for a brand-new Python 3.9+ project unless you need compatibility with an existing `pytz`-based stack.

## Version-Sensitive Notes

- This doc is pinned to `pytz` package version `2026.1.post1`, which PyPI lists as the latest release on March 3, 2026.
- The official GitHub releases page shows both `release_2026.1` and a later same-day `release_2026.1.post1`; if you pin exact timezone data behavior, use the full `.post1` version instead of assuming `2026.1` is equivalent.
- Upstream labels `release_2026.1.post1` as an `IANA 2026a` release.
- The package description still documents legacy compatibility across very old Python versions, but the maintainer now explicitly says Python 3.9+ projects should generally use standard-library timezone support plus packages such as `tzdata`.
- The docs URL, `https://pythonhosted.org/pytz/`, is official but stale as a version marker. It is still useful for `localize(...)`, `normalize(...)`, and DST exception behavior, but it is labeled `2014.10` and should not be treated as the current release docs homepage.

## Official Sources

- PyPI project page: https://pypi.org/project/pytz/
- PyPI JSON API: https://pypi.org/pypi/pytz/json
- Official repository releases: https://github.com/stub42/pytz/releases
- Legacy upstream docs used for behavior examples: https://pythonhosted.org/pytz/

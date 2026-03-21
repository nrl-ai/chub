---
name: package
description: "humanize Python package for turning numbers, sizes, dates, times, and lists into human-readable strings"
metadata:
  languages: "python"
  versions: "4.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "humanize,python,formatting,i18n,time,filesize,numbers"
---

# humanize Python Package Guide

## Golden Rule

Use `humanize` only for presentation. It turns values into user-facing strings like `1.2 million`, `an hour ago`, or `976.6 KiB`; it is not a parsing library and its output is locale-sensitive once translations are activated.

## Install

Pin the version your project expects:

```bash
python -m pip install "humanize==4.15.0"
```

Common alternatives:

```bash
uv add "humanize==4.15.0"
poetry add "humanize==4.15.0"
```

The package has no runtime service credentials or API setup. PyPI lists only a `tests` extra.

## Initialize

Most projects only need a plain import:

```python
import humanize
```

For deterministic examples in tests or generated content, pass explicit values rather than calling `datetime.now()` inside the formatter code.

## Core Usage

### Numbers

Use the top-level helpers for commas, short words, AP-style small numbers, and SI prefixes:

```python
import humanize

print(humanize.intcomma(1_234_567.25))     # 1,234,567.25
print(humanize.intword(1_200_000_000))     # 1.2 billion
print(humanize.apnumber(7))                # seven
print(humanize.metric(1500, "V"))          # 1.50 kV
print(humanize.scientific(500))            # scientific notation string
```

Notes:

- `intcomma()` and `intword()` accept `int`, `float`, and numeric strings.
- If the value is not parseable, these helpers return `str(value)` instead of raising, so validate inputs before using the result in program logic.
- In `4.15.0`, `intword()` gained locale-aware decimal separator support.

### Dates and relative time

`humanize` is most useful for display labels, activity feeds, and summaries:

```python
from datetime import datetime, timedelta, timezone
import humanize

now = datetime.now(timezone.utc)

print(humanize.naturalday(now))                            # today
print(humanize.naturaldate(now - timedelta(days=400)))     # includes year when far away
print(humanize.naturaldelta(timedelta(seconds=1001)))      # 16 minutes
print(humanize.naturaltime(now - timedelta(hours=2)))      # 2 hours ago
print(humanize.precisedelta(timedelta(days=2, seconds=33)))
```

Use `when=` when you need stable output relative to a fixed reference time:

```python
from datetime import datetime, timezone
import humanize

reference = datetime(2026, 3, 12, 12, 0, tzinfo=timezone.utc)
target = datetime(2026, 3, 12, 14, 30, tzinfo=timezone.utc)

print(humanize.naturaltime(target, when=reference))  # 2 hours from now
```

Important behavior:

- `naturaltime()` accepts a `datetime`, `timedelta`, or a numeric seconds value.
- The `future=` flag is ignored for `datetime` and `timedelta`; tense is derived from the value relative to `when` or the current local time.
- `months=True` uses 30.5-day month fuzziness between years.
- `minimum_unit` is useful when sub-second output matters. `naturaldelta()` and `naturaltime()` support `seconds`, `milliseconds`, and `microseconds`.

### Filesizes

Use `naturalsize()` for decimal, binary, or GNU-style units:

```python
import humanize

print(humanize.naturalsize(1_000_000))                 # 1.0 MB
print(humanize.naturalsize(1_000_000, binary=True))    # 976.6 KiB
print(humanize.naturalsize(1_000_000, gnu=True))       # 976.6K
```

Default output uses decimal suffixes (`kB`, `MB`). Set `binary=True` for IEC units (`KiB`, `MiB`) when you need powers-of-two semantics.

### Lists

`natural_list()` is useful for UI text and error messages:

```python
from humanize import natural_list

print(natural_list(["one", "two", "three"]))  # one, two and three
```

### Localization

Translations are mutable library state. Activate a locale before formatting, then deactivate when you are done if the rest of the process should stay in English:

```python
import humanize

humanize.i18n.activate("fr_FR")

print(humanize.intcomma(12345))
print(humanize.intword(1234000))

humanize.i18n.deactivate()
```

Useful helpers:

```python
import humanize

humanize.i18n.activate("fr_FR")
print(humanize.i18n.decimal_separator())
print(humanize.i18n.thousands_separator())
humanize.i18n.deactivate()
```

If `activate()` cannot find translation files, it raises `FileNotFoundError`. You can pass `path=` to load translations from a custom locale directory.

## Configuration And Auth

There is no auth model, service endpoint, or environment-variable configuration. The main configuration surface is localization behavior:

- Default behavior is effectively English.
- `humanize.i18n.activate(locale)` switches the active translation for the current process context used by `humanize`.
- `activate(None)` and locales starting with `en` behave like deactivation.
- Custom translation bundles can be loaded with `humanize.i18n.activate(locale, path="/path/to/locales")`.

## Common Pitfalls

- Do not store or compare `humanize` output as canonical data. Keep raw numbers, datetimes, and byte counts separately.
- `naturaltime()` output depends on the current clock unless you pass `when=`.
- Timezone handling is yours. Pass timezone-aware datetimes if your application mixes zones.
- `intcomma()` and `intword()` may return the original value as a string for invalid inputs instead of failing loudly.
- `precisedelta()` can round or promote units; review output before using it in strict UX copy.
- Locale activation is shared state. In long-running apps, avoid flipping locales globally in ways that can leak across requests or concurrent tasks.

## Version-Sensitive Notes

- `4.15.0` adds locale support for the decimal separator in `intword()`.
- `4.15.0` fixes `naturaldelta()` rounding to the nearest sensible unit and fixes `intword()` plural handling.
- `4.14.0` dropped Python 3.9 support, which is why current PyPI metadata requires Python `>=3.10`.
- The docs URL `https://python-humanize.readthedocs.io/en/latest/` still resolves, but PyPI and the repository point to `https://humanize.readthedocs.io/en/latest/` as the canonical docs root.

## Official Links

- Documentation: `https://humanize.readthedocs.io/en/latest/`
- PyPI: `https://pypi.org/project/humanize/`
- Repository: `https://github.com/python-humanize/humanize`
- Releases: `https://github.com/python-humanize/humanize/releases`

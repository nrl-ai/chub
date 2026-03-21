---
name: package
description: "tomli package guide for parsing TOML in Python projects"
metadata:
  languages: "python"
  versions: "2.4.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tomli,toml,configuration,parser,python,tomllib"
---

# tomli Python Package Guide

## Golden Rule

Use `tomli` when you need a TOML parser dependency that works on Python 3.8-3.10, or when you want one parser dependency across mixed Python versions.

For Python 3.11 and newer, the standard library `tomllib` module is the built-in equivalent API for the common `load()` and `loads()` workflow. `tomli` remains useful as the PyPI backport and may also be chosen explicitly for consistency or wheel performance.

`tomli` is read-only. It parses TOML into Python objects, but it does not write TOML back out.

## Install

Install a pinned version:

```bash
pip install tomli==2.4.0
```

With `uv`:

```bash
uv add tomli
```

With Poetry:

```bash
poetry add tomli
```

For libraries that should only depend on `tomli` when `tomllib` is unavailable:

```toml
[project]
dependencies = [
  'tomli>=1.1.0; python_version < "3.11"',
]
```

Equivalent `requirements.txt` entry:

```text
tomli>=1.1.0; python_version < "3.11"
```

## Initialization And Setup

### Standard compatibility import

Use the stdlib on Python 3.11+ and fall back to `tomli` elsewhere:

```python
import sys

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib
```

This lets the rest of your code call `tomllib.load()` and `tomllib.loads()` on every supported version.

### Reading TOML files

`load()` requires a binary file object:

```python
from pathlib import Path
import sys

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib

with Path("pyproject.toml").open("rb") as f:
    data = tomllib.load(f)
```

If you already have TOML text in memory, use `loads()` instead:

```python
import tomli

config = tomli.loads("""
[server]
host = "127.0.0.1"
port = 8080
""")
```

## Core Usage

### Parse a TOML string

```python
import tomli

toml_text = """
[[players]]
name = "Lehtinen"
number = 26

[[players]]
name = "Numminen"
number = 27
"""

data = tomli.loads(toml_text)
assert data["players"][0]["name"] == "Lehtinen"
```

### Parse a TOML file

```python
import tomli

with open("settings.toml", "rb") as f:
    settings = tomli.load(f)

debug = settings["app"]["debug"]
```

### Handle invalid TOML

```python
import tomli

try:
    tomli.loads("]] this is invalid TOML [[")
except tomli.TOMLDecodeError as exc:
    print(f"Invalid TOML: {exc}")
```

Treat `TOMLDecodeError` messages as human-readable diagnostics, not stable API output.

### Parse floats as `Decimal`

```python
from decimal import Decimal
import tomli

data = tomli.loads("price = 19.99", parse_float=Decimal)
assert data["price"] == Decimal("19.99")
```

`parse_float` must return a scalar numeric-like value. Returning `dict` or `list` raises `ValueError`.

### Read `pyproject.toml`

```python
from pathlib import Path
import sys

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib

with Path("pyproject.toml").open("rb") as f:
    pyproject = tomllib.load(f)

project = pyproject.get("project", {})
name = project.get("name")
dependencies = project.get("dependencies", [])
```

## Config And Environment Notes

- `tomli` has no network layer, credentials, auth flow, or service configuration.
- The main setup choice is whether your project should import `tomllib`, `tomli`, or a compatibility alias.
- `load()` is for binary file objects. `loads()` is for already-decoded strings.
- If you need to write TOML, pair `tomli` with a separate writer such as `tomli-w`.

## Common Pitfalls

### Opening files in text mode

This is wrong:

```python
import tomli

with open("settings.toml", "r", encoding="utf-8") as f:
    data = tomli.load(f)
```

Use binary mode instead:

```python
import tomli

with open("settings.toml", "rb") as f:
    data = tomli.load(f)
```

### Expecting write support

`tomli` does not provide `dump()`, `dumps()`, or file-writing helpers. Use `tomli-w` or another TOML writer when you need round-trip output.

### Building logic around error message text

Do not assert exact `TOMLDecodeError` message strings in tests or production logic. Upstream treats them as informational and they may change across releases.

### Mixing version assumptions

`tomli 2.4.0` and later are compatible with TOML `v1.1.0`. Older `tomli` releases were only TOML `v1.0.0` compatible. If you copy older examples or review older bug reports, make sure the syntax matches the installed parser version.

## Version-Sensitive Notes

- `2.4.0` is the current version specified for this doc and is published on PyPI as of January 11, 2026.
- PyPI metadata for `2.4.0` requires Python `>=3.8`.
- Python `3.11+` includes `tomllib` in the standard library, but `tomli` still has the same familiar parsing API and publishes compiled wheels for common platforms.
- If your codebase only targets Python `3.11+` and does not need an external dependency, `tomllib` is usually the simpler default.
- If your codebase spans Python `3.8+`, the compatibility import pattern above avoids branching the rest of your TOML parsing code.

## Official Sources

- Repository: https://github.com/hukkin/tomli
- README: https://github.com/hukkin/tomli/blob/master/README.md
- Changelog: https://github.com/hukkin/tomli/blob/master/CHANGELOG.md
- PyPI: https://pypi.org/project/tomli/

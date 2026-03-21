---
name: package
description: "ujson Python package guide for fast JSON serialization and parsing with the UltraJSON C extension"
metadata:
  languages: "python"
  versions: "5.11.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "ujson,json,serialization,parsing,python,c-extension"
---

# ujson Python Package Guide

## Golden Rule

Use `ujson` only when a project explicitly wants the UltraJSON C extension for fast local JSON encode/decode and can tolerate behavior differences from the standard library `json` module. For new dependencies, the maintainers explicitly describe `ujson` as maintenance-only and recommend `orjson` instead.

## What It Is For

`ujson` is a C extension that exposes fast JSON serialization and parsing for Python:

- `ujson.dumps()` and `ujson.dump()` for encoding Python objects to JSON
- `ujson.loads()` and `ujson.load()` for decoding JSON text from strings or file-like objects
- `ujson.JSONDecodeError` for parse failures in supported releases

This package is local-only. There is no service client, no network transport, and no authentication layer.

## Install

Pin the version your project expects:

```bash
python -m pip install "ujson==5.11.0"
```

Common alternatives:

```bash
uv add "ujson==5.11.0"
poetry add "ujson==5.11.0"
```

Quick verification:

```bash
python - <<'PY'
from importlib.metadata import version
import ujson

print(version("ujson"))
print(ujson.__version__)
PY
```

## Core Usage

### Serialize a Python object

```python
import ujson

payload = {
    "name": "Ada",
    "active": True,
    "scores": [3, 7, 11],
}

body = ujson.dumps(payload)
print(body)
```

By default, `ujson` emits compact JSON without spaces.

### Parse JSON text

```python
import ujson

raw = '{"name":"Ada","active":true,"scores":[3,7,11]}'
data = ujson.loads(raw)

print(data["name"])
print(data["scores"][0])
```

`loads()` accepts `str`, `bytes`, and `bytearray`.

### Read and write files

```python
from pathlib import Path
import ujson

payload = {"ok": True, "items": [1, 2, 3]}
path = Path("data.json")

with path.open("w", encoding="utf-8") as f:
    ujson.dump(payload, f, ensure_ascii=False, indent=2)

with path.open("r", encoding="utf-8") as f:
    loaded = ujson.load(f)

print(loaded)
```

### Work with custom types

Use `default=` when an object is not JSON-serializable by default:

```python
from dataclasses import asdict, dataclass
import ujson

@dataclass
class User:
    id: int
    email: str

def to_jsonable(obj):
    if isinstance(obj, User):
        return asdict(obj)
    raise TypeError(f"Unsupported type: {type(obj)!r}")

body = ujson.dumps(User(id=1, email="a@example.com"), default=to_jsonable)
print(body)
```

## Output Controls

The upstream README documents these encoding behaviors directly:

- `ensure_ascii=True` by default, so non-ASCII characters are escaped unless you pass `ensure_ascii=False`
- `escape_forward_slashes=True` by default, so `/` becomes `\/` unless you disable it
- `encode_html_chars=True` escapes characters such as `<`, `>`, and `&`
- `indent` must be an integer and controls pretty-print output

Example:

```python
import ujson

body = ujson.dumps(
    {"url": "https://example.com", "title": "café"},
    ensure_ascii=False,
    escape_forward_slashes=False,
    indent=2,
    sort_keys=True,
)

print(body)
```

The official type stubs for current upstream also expose these `dumps()` and `dump()` options:

- `sort_keys`
- `allow_nan`
- `reject_bytes`
- `default`
- `separators`

## Standard Library Compatibility Notes

`ujson` is close enough for many simple `json.dumps()` and `json.loads()` call sites, but it is not a full `json` drop-in:

- The maintainers say it is a drop-in replacement for most other JSON parsers, not all usage.
- The official stub signature for `loads()` is just `loads(s: str | bytes | bytearray) -> Any`; stdlib decoder hooks such as `object_hook`, `parse_float`, and `parse_int` are not part of the exposed `ujson.loads()` API.
- The official stub signature for `dumps()` does not expose stdlib custom encoder class parameters like `cls`.
- `indent` is typed as `int`; do not pass the string-style indentation patterns that stdlib `json.dumps()` accepts.

If a codebase depends on exact stdlib JSON semantics, validate behavior before swapping imports globally.

## Error Handling

```python
import ujson

try:
    ujson.loads('{"broken": }')
except ujson.JSONDecodeError as exc:
    print(f"Invalid JSON: {exc}")
```

For `5.11.0`, the release notes explicitly call out a fix where nested invalid JSON now raises `JSONDecodeError` instead of `SystemError`.

## Common Pitfalls

- Do not treat `ujson` as the safest default new dependency. Upstream marks it as maintenance-only and recommends `orjson` for new work.
- `ensure_ascii=True` is the default. If you expect human-readable UTF-8 output, pass `ensure_ascii=False`.
- `escape_forward_slashes=True` is also the default. This surprises agents comparing output against stdlib `json`.
- `ujson` is fast for simple encode/decode paths, but the official benchmark table on the project page is old and compares against `ujson 5.7.1.dev26` on CPython `3.11.3`. Do not treat those numbers as current `5.11.0` production guidance.
- Do not assume every stdlib `json` keyword is supported. Check the exposed `ujson` signature before forwarding kwargs from shared helpers.
- If you need schema validation, streaming JSON parsing, or custom decoder hooks, `ujson` is usually the wrong layer.

## Version-Sensitive Notes

- `5.11.0` adds inline type stubs, which makes editor and type-checker support better than older `ujson` releases.
- `5.11.0` supports Python `3.9+` and drops Python `3.8`.
- `5.11.0` adds Python `3.14`, PyPy 3.11, and Windows ARM64 support in the release notes.
- As of March 12, 2026, the current PyPI project page has moved to `5.12.0` and raises the Python floor to `>=3.10`. If your project follows latest instead of pinning `5.11.0`, verify interpreter compatibility before upgrading.

## When To Reach For Something Else

- Use stdlib `json` when dependency minimization and broad compatibility matter more than throughput.
- Use `orjson` when you are choosing a fresh high-performance JSON dependency and can adopt its API and bytes-returning behavior where relevant.
- Stay on `ujson` when a codebase is already pinned to it and its behavior is part of the existing contract.

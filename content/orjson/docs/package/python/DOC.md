---
name: package
description: "orjson package guide for fast Python JSON serialization and deserialization with dataclass, datetime, UUID, and numpy support"
metadata:
  languages: "python"
  versions: "3.11.7"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "orjson,python,json,serialization,deserialization,dataclass,datetime,numpy"
---

# orjson Python Package Guide

## Golden Rule

Use `orjson` when a Python project needs very fast JSON encoding and decoding on CPython, and treat it as a bytes-first library. `orjson.dumps()` returns UTF-8 `bytes`, not `str`, and `orjson.loads()` is strict about valid UTF-8 and valid RFC 8259 JSON.

## Install

Pin the version your project expects:

```bash
python -m pip install "orjson==3.11.7"
```

Common alternatives:

```bash
uv add "orjson==3.11.7"
poetry add "orjson==3.11.7"
```

Notes:

- PyPI publishes wheels for common Linux, macOS, and Windows targets. Normal installs should not need a local toolchain.
- If `pip` falls back to building from source, upstream says packaging requires Rust `1.89+`, a C compiler, and `maturin`.
- `orjson` does not support PyPy.

## Core API Surface

The library is intentionally small:

- `orjson.dumps(obj, *, default=..., option=...) -> bytes`
- `orjson.loads(data) -> Any`

Practical implications:

- `dumps()` returns `bytes`, so decode only at boundaries that require `str`, such as text files or framework APIs that do not accept bytes.
- `loads()` accepts `bytes`, `bytearray`, `memoryview`, and `str`. Prefer passing bytes-like input directly instead of decoding first.
- The library does not handle file I/O, NDJSON/JSONL framing, or schema-aware object reconstruction.

## Basic Usage

### Serialize and deserialize

```python
import orjson

payload = {
    "ok": True,
    "count": 3,
    "tags": ["a", "b"],
}

blob = orjson.dumps(payload)
assert isinstance(blob, bytes)

data = orjson.loads(blob)
assert data == payload
```

If an API needs a string body:

```python
body = orjson.dumps(payload).decode("utf-8")
```

### Pretty-print or append newlines

```python
import orjson

blob = orjson.dumps(
    {"a": 1, "nested": {"b": True}},
    option=orjson.OPT_INDENT_2 | orjson.OPT_APPEND_NEWLINE,
)
```

Use pretty printing for logs, snapshots, or fixtures. Skip it on hot paths.

## Common Native Types

`orjson` natively serializes many types agents often reach for `default=` to handle:

- `dataclasses.dataclass`
- `datetime.datetime`, `datetime.date`, `datetime.time`
- `uuid.UUID`
- `typing.TypedDict`
- enums whose values are supported types

Example:

```python
from dataclasses import dataclass
from datetime import UTC, datetime
from uuid import UUID

import orjson

@dataclass
class Event:
    id: UUID
    created_at: datetime
    name: str

event = Event(
    id=UUID("12345678-1234-5678-1234-567812345678"),
    created_at=datetime(2026, 3, 12, 10, 30, tzinfo=UTC),
    name="build.finished",
)

blob = orjson.dumps(event)
```

## Custom Serialization

Use `default=` only for unsupported types or when you explicitly want to override native behavior.

```python
from decimal import Decimal

import orjson

def default(obj):
    if isinstance(obj, Decimal):
        return str(obj)
    raise TypeError

blob = orjson.dumps({"price": Decimal("12.50")}, default=default)
```

Important behavior:

- Your `default` function must raise `TypeError` for unhandled objects.
- If `default` forgets to raise, Python returns `None` implicitly and `orjson` will serialize that as `null`.
- `orjson` raises `JSONEncodeError` on unsupported objects, circular references, invalid UTF-8 strings, and integers beyond its supported range.

### Override native dataclass, datetime, or subclass handling

Version 3 serializes dataclasses and UUIDs by default. If you need custom behavior, combine `default=` with passthrough options:

```python
import dataclasses
from datetime import datetime

import orjson

@dataclasses.dataclass
class User:
    id: str
    password: str

def default(obj):
    if isinstance(obj, User):
        return {"id": obj.id}
    if isinstance(obj, datetime):
        return obj.strftime("%a, %d %b %Y %H:%M:%S GMT")
    raise TypeError

blob = orjson.dumps(
    {
        "user": User("u_123", "secret"),
        "created_at": datetime(2026, 3, 12, 10, 30),
    },
    default=default,
    option=orjson.OPT_PASSTHROUGH_DATACLASS | orjson.OPT_PASSTHROUGH_DATETIME,
)
```

Use `OPT_PASSTHROUGH_SUBCLASS` the same way when subclasses of builtin types should not serialize through their base type behavior.

## Datetime and Key Options

`orjson` serializes datetimes as RFC 3339 strings.

```python
from datetime import UTC, datetime

import orjson

dt = datetime(2026, 3, 12, 10, 30, tzinfo=UTC)

print(orjson.dumps(dt))
print(orjson.dumps(dt, option=orjson.OPT_UTC_Z))
print(orjson.dumps(datetime(2026, 3, 12, 10, 30), option=orjson.OPT_NAIVE_UTC))
```

Use these options when the receiving system has explicit formatting requirements:

- `OPT_UTC_Z`: render UTC as `Z` instead of `+00:00`
- `OPT_NAIVE_UTC`: treat naive `datetime` values as UTC during serialization
- `OPT_NON_STR_KEYS`: allow keys like `int`, `UUID`, and `datetime`

Be careful with `OPT_NON_STR_KEYS`: different Python keys can serialize to the same JSON string key, which can silently collapse data.

## Numpy

Numpy serialization is opt-in:

```python
import numpy as np
import orjson

arr = np.array([[1, 2], [3, 4]], dtype=np.int64)
blob = orjson.dumps(arr, option=orjson.OPT_SERIALIZE_NUMPY)
```

Notes:

- Upstream states `orjson` is compatible with numpy v1 and v2.
- Arrays must be contiguous C arrays and use supported dtypes.
- Unsupported arrays fall through to `default=`.
- Native numpy serialization can produce different float rounding than `ndarray.tolist()` because it avoids converting through Python `double` first.

## Strictness And Performance Notes

- `loads()` rejects invalid JSON values such as `NaN`, `Infinity`, and `-Infinity`, even though Python's standard `json` module accepts them by default.
- `dumps()` serializes `NaN`, `Infinity`, and `-Infinity` as `null`.
- The GIL is held for the duration of `dumps()` and `loads()`.
- `OPT_SORT_KEYS` is available for deterministic output, but upstream documents a substantial performance penalty.
- `OPT_STRICT_INTEGER` enforces the 53-bit integer range for ecosystems that cannot safely consume wider integers, such as JavaScript clients.

## Fragment For Embedding Cached JSON

`orjson.Fragment` lets you embed already-serialized JSON without parsing it first:

```python
import orjson

cached = b'{"roles":["admin","editor"]}'
blob = orjson.dumps({"user_id": 7, "permissions": orjson.Fragment(cached)})
```

Use this when you already trust the JSON source, such as a cached payload or JSONB column. `Fragment` does not reformat or validate the embedded JSON beyond UTF-8 checks for `str` input.

## Config And Auth

`orjson` has no service auth, network setup, credentials, or environment-variable configuration.

The main configuration surface is the `option=` bitmask on `dumps()`. The options most likely to matter in application code are:

- `OPT_INDENT_2`
- `OPT_APPEND_NEWLINE`
- `OPT_NAIVE_UTC`
- `OPT_UTC_Z`
- `OPT_NON_STR_KEYS`
- `OPT_SERIALIZE_NUMPY`
- `OPT_SORT_KEYS`
- `OPT_STRICT_INTEGER`
- `OPT_PASSTHROUGH_DATACLASS`
- `OPT_PASSTHROUGH_DATETIME`
- `OPT_PASSTHROUGH_SUBCLASS`

## Common Pitfalls

- Expecting `str` output. `orjson.dumps()` returns `bytes`.
- Decoding bytes before `loads()`. If you already have bytes, pass them directly.
- Assuming it behaves exactly like `json`. `loads()` is stricter, and `dumps()` has different defaults around bytes output and native type support.
- Using `default=` to customize dataclasses or datetimes without enabling the relevant passthrough option.
- Turning on `OPT_NON_STR_KEYS` without considering duplicate-key collisions after string conversion.
- Turning on `OPT_SORT_KEYS` in hot paths. Upstream calls out a real performance cost.
- Forgetting `OPT_SERIALIZE_NUMPY` for ndarray payloads.
- Assuming it supports PyPy, subinterpreters, NDJSON/JSONL, object reconstruction, or file helpers. It intentionally does not.

## Version-Sensitive Notes For 3.11.7

- PyPI lists `3.11.7` as the latest release on `2026-02-02`.
- The `3.11.7` changelog notes a faster float serializer and a byte-level output change for positive exponents, for example `1.2e+30` instead of `1.2e30`. Both are JSON-spec compliant, but exact-string regression tests may need updates.
- The prior `3.11.6` release dropped Python `3.9` support, so `3.11.7` should be treated as Python `3.10+` only.
- The changelog for `3.11.6` also raised the source-build requirement to Rust `1.89+`.
- Version 3 behavior that still trips older examples: dataclasses and UUIDs serialize by default, and subclasses of `str`, `int`, `dict`, and `list` also serialize by default unless you opt into passthrough behavior.

## Official Sources

- Documentation and README: `https://github.com/ijl/orjson`
- PyPI package page: `https://pypi.org/project/orjson/`
- PyPI JSON metadata: `https://pypi.org/pypi/orjson/json`
- Changelog: `https://github.com/ijl/orjson/blob/master/CHANGELOG.md`

---
name: package
description: "msgpack package guide for Python with pack/unpack, streaming, custom types, security limits, and compatibility flags"
metadata:
  languages: "python"
  versions: "1.1.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "msgpack,messagepack,python,serialization,binary,streaming"
---

# msgpack Python Package Guide

## Golden Rule

Use `msgpack` when you need compact binary serialization between Python and other MessagePack clients. For new Python code, serialize with `msgpack.packb(..., use_bin_type=True)` and deserialize with `msgpack.unpackb(..., raw=False, strict_map_key=True)` unless you are intentionally targeting older raw-byte behavior.

## Version-Sensitive Notes

- As of March 12, 2026, PyPI lists `msgpack 1.1.2` released on October 8, 2025.
- The Read the Docs site still renders page chrome that says `msgpack 1.0 documentation`, so use PyPI for exact version pinning and the API docs for behavior.
- The upstream module code published on Read the Docs reports `__version__ = (1, 1, 2)`, which matches PyPI.
- The `1.1.0` changelog added `buf_size` to `msgpack.Packer`, made the pure-Python fallback require keyword arguments for `Packer` and `Unpacker`, and improved `Timestamp` precision and `datetime=True` handling.
- The `1.1.2` changelog notes source code was unchanged from `1.1.1`; the release refreshed wheels and packaging.

## Install

Pin the version when your project depends on exact wire compatibility or reproducible builds:

```bash
python -m pip install "msgpack==1.1.2"
```

Common alternatives:

```bash
uv add "msgpack==1.1.2"
poetry add "msgpack==1.1.2"
```

## Initialize And Choose The API Surface

`msgpack` exposes two common patterns:

- `packb()` / `unpackb()` for one-shot in-memory bytes
- `Packer` / `Unpacker` for streams, sockets, or concatenated messages

Minimal import:

```python
import msgpack
```

## Core Usage

### Pack And Unpack Bytes

```python
import msgpack

payload = {
    "id": 123,
    "name": "example",
    "enabled": True,
    "tags": ["a", "b"],
}

wire = msgpack.packb(payload, use_bin_type=True)
restored = msgpack.unpackb(wire, raw=False, strict_map_key=True)

print(type(wire))      # <class 'bytes'>
print(restored["name"])  # example
```

Key flags:

- `use_bin_type=True` writes Python `bytes` using MessagePack bin types instead of old raw string encoding.
- `raw=False` decodes MessagePack string types to Python `str` and keeps binary data as `bytes`.
- `strict_map_key=True` rejects non-string and non-bytes map keys, which is safer for untrusted data.

### Write To And Read From File-Like Objects

```python
import io
import msgpack

buffer = io.BytesIO()
msgpack.pack({"event": "created", "ok": True}, buffer, use_bin_type=True)

buffer.seek(0)
event = msgpack.unpack(buffer, raw=False, strict_map_key=True)
print(event)
```

Use `pack()` and `unpack()` when you already have a file, socket wrapper, or `BytesIO`.

### Stream Multiple Messages

Use `Unpacker` for concatenated frames or incremental reads from a socket:

```python
import msgpack

chunks = [
    msgpack.packb({"seq": 1}, use_bin_type=True),
    msgpack.packb({"seq": 2}, use_bin_type=True),
]

unpacker = msgpack.Unpacker(raw=False, strict_map_key=True)

for chunk in chunks:
    unpacker.feed(chunk)
    for item in unpacker:
        print(item)
```

This is the right surface when the stream may contain zero, one, or many complete MessagePack objects per read.

## Custom Types

Use `default=` during packing and `ext_hook=` during unpacking for types MessagePack does not natively understand.

Example with `uuid.UUID`:

```python
import uuid

import msgpack

UUID_EXT_CODE = 1

def encode_custom(obj):
    if isinstance(obj, uuid.UUID):
        return msgpack.ExtType(UUID_EXT_CODE, obj.bytes)
    raise TypeError(f"Unsupported type: {type(obj)!r}")

def decode_custom(code, data):
    if code == UUID_EXT_CODE:
        return uuid.UUID(bytes=data)
    return msgpack.ExtType(code, data)

value = {"user_id": uuid.uuid4()}
wire = msgpack.packb(value, default=encode_custom, use_bin_type=True)
restored = msgpack.unpackb(wire, ext_hook=decode_custom, raw=False, strict_map_key=True)
```

If you only need tuples or subclasses handled more strictly, look at `strict_types=True` on the pack side.

## Timestamp And Datetime Handling

`msgpack` has native `Timestamp` support. For Python `datetime`, use timezone-aware UTC values and opt in explicitly.

```python
from datetime import datetime, timezone

import msgpack

value = {"created_at": datetime.now(timezone.utc)}
wire = msgpack.packb(value, datetime=True, use_bin_type=True)

restored = msgpack.unpackb(
    wire,
    raw=False,
    strict_map_key=True,
    timestamp=3,
)

print(restored["created_at"])
```

Practical rules:

- `datetime=True` only packs aware datetimes with `tzinfo`.
- `timestamp=3` unpacks timestamps as UTC `datetime.datetime` objects.
- If you need full manual control, pack and unpack `msgpack.Timestamp` values directly.

## Configuration And Compatibility

### String Vs Bytes Compatibility

For modern Python-only or cross-language code:

```python
wire = msgpack.packb(value, use_bin_type=True)
restored = msgpack.unpackb(wire, raw=False)
```

Only use the older raw-string compatibility mode when you must interoperate with legacy peers that still encode strings as raw bytes:

```python
wire = msgpack.packb(value, use_bin_type=False)
restored = msgpack.unpackb(wire, raw=True)
```

### Lists Vs Tuples

By default arrays unpack to Python `list`. If you need tuples:

```python
item = msgpack.unpackb(wire, raw=False, use_list=False)
```

### Reusing A `Packer`

`Packer` can reduce allocation overhead in tight loops:

```python
import msgpack

packer = msgpack.Packer(use_bin_type=True, autoreset=False, buf_size=1024 * 1024)

for item in [{"n": 1}, {"n": 2}]:
    packer.pack(item)
    wire = packer.bytes()
    print(len(wire))
    packer.reset()
```

Notes:

- `autoreset=False` means you must call `bytes()` and then `reset()` yourself.
- `buf_size` was added in `1.1.0` and helps when packing many similarly sized messages.

### Pure Python Fallback

CPython normally uses the C extension for speed. The upstream module supports forcing the pure-Python implementation with:

```bash
export MSGPACK_PUREPYTHON=1
```

This is mainly useful for debugging, compatibility testing, or environments where the extension cannot load. It is slower than the default extension-backed path.

## Security And Limits

When unpacking untrusted data, keep limits explicit:

```python
import msgpack

safe_value = msgpack.unpackb(
    wire,
    raw=False,
    strict_map_key=True,
    max_buffer_size=16 * 1024 * 1024,
)
```

Important guardrails from the upstream docs and README:

- Keep `strict_map_key=True` for untrusted input.
- Set `max_buffer_size` deliberately for large or attacker-controlled payloads.
- Avoid `unicode_errors=` unless you are intentionally recovering malformed input; the docs warn against casual use.
- Expect `ExtraData` when multiple packed objects are present but you call `unpackb()` on the whole byte string.

## Common Pitfalls

- `packb()` returns `bytes`, not text. Do not decode it to UTF-8 just to store or send it.
- `unpackb()` expects exactly one object. For concatenated objects, use `Unpacker`.
- If a peer expects old raw-string behavior, `raw=False` plus `use_bin_type=True` may not round-trip with that peer.
- Naive `datetime` values are not automatically serialized when `datetime=True`; use timezone-aware values.
- `default=` must raise `TypeError` for unsupported objects so `msgpack` can fail predictably.
- The pure-Python fallback in `1.1.x` expects keyword arguments for `Packer` and `Unpacker`; do not depend on positional-only initialization patterns there.

## Official Sources

- Docs root: `https://msgpack-python.readthedocs.io/en/latest/`
- API reference: `https://msgpack-python.readthedocs.io/en/latest/api.html`
- Advanced usage: `https://msgpack-python.readthedocs.io/en/latest/advanced.html`
- Published module code: `https://msgpack-python.readthedocs.io/en/latest/_modules/msgpack.html`
- PyPI project page: `https://pypi.org/project/msgpack/`
- Upstream changelog: `https://github.com/msgpack/msgpack-python/blob/main/ChangeLog.rst`

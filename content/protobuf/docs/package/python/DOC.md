---
name: package
description: "protobuf package guide for Python message generation, serialization, and JSON interop"
metadata:
  languages: "python"
  versions: "7.34.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "protobuf,protocol-buffers,serialization,codegen,json"
---

# protobuf Python Package Guide

## What It Is

`protobuf` is the Python runtime for Protocol Buffers. In Python code you install `protobuf`, but the runtime APIs live under `google.protobuf`. Typical work is:

1. define messages in `.proto` files
2. generate Python modules with `protoc`
3. construct message objects, then serialize, parse, or convert them to JSON

## Install

Install the runtime:

```bash
pip install protobuf==7.34.0
```

With `uv`:

```bash
uv add protobuf==7.34.0
```

With Poetry:

```bash
poetry add protobuf@7.34.0
```

## Setup

### Runtime only

If your repository already commits generated `*_pb2.py` files, the runtime package is enough.

### Generate Python code from `.proto`

The Python package does not replace the Protocol Buffer compiler. Install `protoc` separately, then generate code:

```bash
protoc --proto_path=. --python_out=. example.proto
```

If you also want type stubs:

```bash
protoc --proto_path=. --python_out=. --pyi_out=. example.proto
```

The protobuf install docs warn that package-manager `protoc` builds can be old. Check the compiler version explicitly:

```bash
protoc --version
```

## Core Usage

### 1. Define a schema

```proto
syntax = "proto3";

message User {
  string id = 1;
  string email = 2;
  repeated string roles = 3;
}
```

### 2. Generate Python code

```bash
protoc --proto_path=. --python_out=. user.proto
```

This creates `user_pb2.py`.

### 3. Create, serialize, and parse messages

```python
import user_pb2

user = user_pb2.User(
    id="u_123",
    email="dev@example.com",
    roles=["admin", "billing"],
)

payload = user.SerializeToString()

parsed = user_pb2.User()
parsed.ParseFromString(payload)

assert parsed.id == "u_123"
assert parsed.roles == ["admin", "billing"]
```

### 4. Convert to and from JSON

Use `google.protobuf.json_format` for API boundaries that expect JSON rather than protobuf wire format.

```python
import user_pb2
from google.protobuf.json_format import MessageToDict, ParseDict

user = user_pb2.User(id="u_123", email="dev@example.com")

data = MessageToDict(user, preserving_proto_field_name=True)

round_tripped = user_pb2.User()
ParseDict(data, round_tripped)
```

### 5. Common message operations

```python
msg = user_pb2.User()

msg.id = "u_123"
msg.roles.append("reader")

copy = user_pb2.User()
copy.CopyFrom(msg)

assert copy == msg
assert copy.IsInitialized()
```

## Generated-Code Rules That Matter

- Python output modules are named `*_pb2.py`.
- `*_pb2.py` files are generated code. Do not hand-edit them.
- The `.proto` `package` declaration does **not** control the Python import path. Python imports follow where generated files are written.
- Generated message classes are not intended to be subclassed. Wrap them in your own classes if you need higher-level behavior.

## Config And Environment

There is no authentication layer in `protobuf`. The relevant configuration is build/runtime configuration:

- Pin `protobuf` in your Python environment if generated code is committed to the repo.
- Keep `protoc` reasonably aligned with the Python runtime series you use.
- If you need Python-only behavior when interoperating with C++ extensions, protobuf documents `PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION=python` for that fallback path.

## Common Pitfalls

### Install name vs import name

You install `protobuf`, but import from `google.protobuf` or from generated `*_pb2` modules.

### Generated code path surprises

If `protoc` writes files into `generated/`, import paths will usually come from that filesystem layout, not from the `.proto` package name.

### Presence rules differ by field type

Use `HasField()` only for fields with presence. Repeated fields do not support scalar presence checks the same way singular message or optional fields do.

### Repeated message fields use protobuf helpers

For repeated message fields, use protobuf’s collection helpers such as `.add()` instead of assigning plain dicts or arbitrary objects.

### Keyword collisions are real

If a field name conflicts with a Python keyword, protobuf documents using `getattr()` and related dynamic access patterns.

### JSON conversion is explicit

`SerializeToString()` produces protobuf wire bytes, not JSON. Use `json_format` helpers for JSON APIs.

## Version-Sensitive Notes For 7.34.0

- `7.34.0` is the Python major-version bump announced by protobuf on September 19, 2025.
- Upstream states that `6.33` is the final `6.x` minor release and that `7.34.0` does not change Python gencode, so older generated files should not trip new poison-pill checks in `7.34.x`.
- The 7.34.0 change note also calls out stricter runtime behavior:
  - incorrect type conversion to `Timestamp` or `Duration` now raises `TypeError`
  - assigning `bool` to enum or integer fields is rejected instead of being silently coerced

## Agent Guidance

- If a repo contains `.proto` files but no generated `*_pb2.py`, assume you need `protoc` before writing runtime code.
- If a repo contains generated files from an older protobuf line, prefer upgrading the runtime and regenerating with the project’s chosen compiler version together.
- For web APIs, prefer `MessageToDict` or `MessageToJson` with `preserving_proto_field_name=True` when snake_case field names need to stay stable.
- When debugging equality or merge behavior, use message methods such as `CopyFrom`, `MergeFrom`, `ClearField`, and `HasField` instead of manipulating internals.

## Official Sources Used

- Python reference root: https://protobuf.dev/reference/python/
- Python generated code guide: https://protobuf.dev/reference/python/python-generated/
- Python/C++ comparison notes: https://protobuf.dev/reference/python/python-comparison/
- Python tutorial: https://protobuf.dev/getting-started/pythontutorial/
- Compiler installation: https://protobuf.dev/installation/
- Version support policy: https://protobuf.dev/support/version-support/
- 7.34.0 Python change note: https://protobuf.dev/news/2025-09-19/
- PyPI package page: https://pypi.org/project/protobuf/

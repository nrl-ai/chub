---
name: package
description: "googleapis-common-protos package guide for Python generated Google API protobuf modules"
metadata:
  languages: "python"
  versions: "1.73.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,protobuf,grpc,googleapis,proto"
---

# googleapis-common-protos Python Package Guide

## What This Package Is

`googleapis-common-protos` installs generated Python protobuf modules for common Google API definitions. In practice, it gives you imports such as:

- `google.api.*_pb2`
- `google.rpc.*_pb2`
- `google.type.*_pb2`
- `google.longrunning.operations_pb2`

Use it when your code, generated stubs, or another dependency needs those message definitions at runtime.

This package is not a service client. It does not make HTTP requests, open gRPC channels, or authenticate with Google services by itself.

## Install

Pin to the version this entry covers:

```bash
pip install googleapis-common-protos==1.73.0
```

If your dependency graph expects the optional gRPC extra exposed on PyPI:

```bash
pip install "googleapis-common-protos[grpc]==1.73.0"
```

Alternative package managers:

```bash
poetry add googleapis-common-protos==1.73.0
uv add googleapis-common-protos==1.73.0
```

## Python Support

The `1.73.0` PyPI release requires Python `>=3.7`.

## Initialize And Import

There is no package-specific initialization step. Import the generated modules directly and instantiate protobuf messages as needed.

```python
from google.rpc import status_pb2
from google.type import date_pb2

status = status_pb2.Status(code=3, message="invalid request")
birthday = date_pb2.Date(year=2026, month=3, day=12)
```

## Core Usage

### Build structured error details

`google.rpc.status_pb2` and `google.rpc.error_details_pb2` are common when you need gRPC-style or Google API error payloads.

```python
from google.protobuf import any_pb2
from google.rpc import error_details_pb2, status_pb2

bad_request = error_details_pb2.BadRequest()
violation = bad_request.field_violations.add()
violation.field = "user.email"
violation.description = "must be a valid email address"

detail = any_pb2.Any()
detail.Pack(bad_request)

status = status_pb2.Status(
    code=3,  # INVALID_ARGUMENT
    message="request validation failed",
)
status.details.append(detail)
```

### Work with long-running operation messages

Some Google APIs expose long-running operations through `google.longrunning.operations_pb2`.

```python
from google.longrunning import operations_pb2

operation = operations_pb2.Operation(
    name="operations/1234567890",
    done=False,
)
```

### Use common Google API message types

The package also provides generated modules under namespaces such as `google.api` and `google.type`.

```python
from google.api import httpbody_pb2
from google.type import decimal_pb2

body = httpbody_pb2.HttpBody(
    content_type="application/json",
    data=b'{"ok": true}',
)

price = decimal_pb2.Decimal(value="19.99")
```

## Config And Auth

### Authentication

You do not need credentials to import these modules or construct protobuf messages.

Authentication belongs to the higher-level code that uses these messages, for example:

- a `google-cloud-*` client library
- a gRPC client or server
- a REST layer that serializes these messages

If you are only creating or parsing protobuf messages from this package, Google Cloud project setup, billing, and Application Default Credentials are not part of the package setup.

### Logging

PyPI documents the standard Google Python SDK logging hook. If you need Google library logging in the same process, set `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` to a valid logger scope such as `google`, or configure Python logging manually.

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

## Common Pitfalls

### Install name is not the import name

Install:

```bash
pip install googleapis-common-protos
```

Import from the generated namespaces:

```python
from google.rpc import status_pb2
from google.type import date_pb2
```

Do not write:

```python
import googleapis_common_protos
```

### `protobuf` and `googleapis-common-protos` solve different import errors

`protobuf` provides the core runtime and well-known types under `google.protobuf.*`.

`googleapis-common-protos` provides generated Google API modules such as `google.rpc.*`, `google.type.*`, and `google.api.*`.

If the failing import is `google.protobuf.timestamp_pb2`, fix `protobuf`.
If the failing import is `google.rpc.status_pb2` or `google.type.date_pb2`, fix `googleapis-common-protos`.

### Installing the wheel does not replace local `.proto` sources for `protoc`

The official `api-common-protos` guidance is explicit about this: if you compile your own protos and import files like `google/type/color.proto`, you still need local access to those `.proto` files during code generation.

Use `googleapis-common-protos` so you do not have to ship your own generated Python classes for these common definitions. Do not assume it removes the need for proto include paths when running `protoc`.

### Do not treat this like a Google client-library quickstart

The PyPI page includes standard Google client-library guidance around credentials and logging, but this package itself is only generated proto modules. For message construction and parsing alone, there is no package-specific network setup.

## Version-Sensitive Notes

- The version used here `1.73.0` matches the current PyPI latest release as of March 12, 2026.
- PyPI lists `Requires: Python >=3.7` for `1.73.0`.
- The docs URL, `https://github.com/googleapis/python-api-common-protos`, is still official but the repository is archived and points maintainers to the `google-cloud-python/packages/googleapis-common-protos` location.
- The older `googleapis/api-common-protos` repository is also archived and warns that its protos can be outdated. For canonical proto definitions, prefer `googleapis/googleapis`.

## Official Sources

- PyPI project page: `https://pypi.org/project/googleapis-common-protos/`
- PyPI version page: `https://pypi.org/project/googleapis-common-protos/1.73.0/`
- Archived repository URL: `https://github.com/googleapis/python-api-common-protos`
- Archived proto packaging guidance: `https://github.com/googleapis/api-common-protos`
- Canonical Google API proto source tree: `https://github.com/googleapis/googleapis`

---
name: mypy-boto3-iot
description: "mypy-boto3-iot type stubs for boto3 AWS IoT clients, paginators, literals, and generated type definitions"
metadata:
  languages: "python"
  versions: "1.42.14"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,iot,python,mypy,typing,stubs"
---

# mypy-boto3-iot Python Package Guide

## What It Is

`mypy-boto3-iot` is a stub-only PEP 561 package for the AWS IoT service client in `boto3`.

Use it when your code already talks to AWS IoT through `boto3`, but you want static typing for:

- the `IoTClient` interface
- generated paginator types
- generated string literal unions
- generated request and response `TypedDict` shapes

It does not make AWS calls by itself. Runtime behavior, credentials, retries, endpoints, and exceptions still come from `boto3` and `botocore`.

## Install

Install the runtime SDK and the matching stub package together:

```bash
python -m pip install "boto3==1.42.14" "mypy-boto3-iot==1.42.14"
```

If you want typed `Session.client("iot")` overloads from the broader maintainer package, use:

```bash
python -m pip install "boto3-stubs[iot]==1.42.14"
```

If your IDE struggles with the larger `boto3-stubs` package, the maintainer docs recommend either the service-specific package you are reading about now or the lighter extra:

```bash
python -m pip install "boto3-stubs-lite[iot]==1.42.14"
```

Practical rule: keep `boto3`, `botocore`, and `mypy-boto3-iot` on the same patch version when possible.

## Setup

Annotate the client explicitly and let `boto3` handle the real runtime client creation.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_iot import IoTClient

session = boto3.Session(profile_name="dev", region_name="us-west-2")
iot: IoTClient = session.client("iot")

thing = iot.describe_thing(thingName="sensor-123")
print(thing["thingName"])
```

This pattern is the safest default because:

- the annotation is available to mypy and pyright
- `boto3` still creates the real client at runtime
- `from __future__ import annotations` prevents the type annotation from being evaluated at runtime

## Auth And Configuration

`mypy-boto3-iot` has no separate auth or config system. Use the normal `boto3` credential and region chain:

1. explicit arguments on `boto3.Session(...)` or `session.client(...)`
2. environment variables such as `AWS_PROFILE` and `AWS_DEFAULT_REGION`
3. shared AWS config in `~/.aws/config` and `~/.aws/credentials`
4. runtime providers such as container or instance roles

Local development example:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Then:

```python
import boto3

session = boto3.Session()
iot = session.client("iot")
```

If the underlying boto3 client is misconfigured, the stub package will not fix it. Treat auth, retry mode, endpoint selection, and region selection as boto3 concerns.

## Core Usage

### Typed Client

Use `IoTClient` when you want operation names, parameter names, and response keys checked by your type checker.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_iot import IoTClient

iot: IoTClient = boto3.Session(region_name="us-east-1").client("iot")
certificate = iot.describe_certificate(certificateId="abcd1234")
print(certificate["certificateDescription"]["status"])
```

### Typed Paginators

The generated docs for this package include paginator support. Keep the paginator name as a string literal so the overload stays precise.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_iot import IoTClient

iot: IoTClient = boto3.Session(region_name="us-east-1").client("iot")
paginator = iot.get_paginator("list_things")

for page in paginator.paginate():
    for thing in page.get("things", []):
        print(thing["thingName"])
```

### Literals And Type Definitions

The generated docs root exposes these modules:

- `mypy_boto3_iot.client`
- `mypy_boto3_iot.literals`
- `mypy_boto3_iot.paginator`
- `mypy_boto3_iot.type_defs`

Use `literals` when an API parameter only accepts a fixed set of string values, and `type_defs` when you want generated request or response shapes in helper functions. The exact names are generated from the AWS service model, so check the docs site before inventing a `Literal` alias or `TypedDict` name.

## When To Use `boto3-stubs[iot]` Instead

Choose `mypy-boto3-iot` when you want the smallest service-specific typing package.

Choose `boto3-stubs[iot]` when you want:

- typed `Session.client("iot")` overloads with less manual annotation
- one install pattern used across multiple AWS services
- the broader maintainer setup shown in the upstream package README

Choose `boto3-stubs-lite[iot]` when you want those service extras but your IDE struggles with the full `boto3-stubs` package.

## Common Pitfalls

- This is not a runtime SDK. You still need `boto3`.
- The boto3 service name is `iot`, not `mypy-boto3-iot`.
- Prefer `TYPE_CHECKING` imports or postponed annotations. Direct runtime imports from stub-only packages are easy to get wrong.
- Keep `boto3`, `botocore`, and `mypy-boto3-iot` aligned. If the versions drift, you can type-check calls that fail at runtime or miss newly added parameters.
- `mypy-boto3-iot` alone does not configure credentials, retries, or regions. That is still boto3 setup.
- The generated docs list client, literals, paginators, and type definitions for this package. I did not find service-resource or waiter modules for `mypy-boto3-iot` in the published 1.42.14 docs.

## Version-Sensitive Notes

- The version used here `1.42.14` matches the official PyPI package page and the generated docs site as of 2026-03-12.
- PyPI shows `mypy-boto3-iot 1.42.14` was released on 2025-12-19.
- This package requires Python `>=3.9`.
- The package is generated from the maintainer's broader `boto3-stubs` project, so patch releases closely track boto3 and AWS service-model changes.
- If you copy examples from older posts, confirm the current generated symbol names on the docs site before using them in annotations.

## Official Sources

- PyPI package page: https://pypi.org/project/mypy-boto3-iot/
- PyPI JSON metadata: https://pypi.org/pypi/mypy-boto3-iot/json
- Generated docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_iot/
- Generated client docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_iot/client/

---
name: mypy-boto3-ivs
description: "Typed boto3 IVS stubs for Python with install options, typed clients and paginators, literals, and runtime-safe typing patterns"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,ivs,boto3,mypy,typing,stubs,python,streaming"
---

# mypy-boto3-ivs Python Package Guide

## Golden Rule

Use `boto3` for real AWS IVS calls and use `mypy-boto3-ivs` only for typing. If you want `Session.client("ivs")` to infer automatically, install `boto3-stubs[ivs]`; if you install only the standalone or lite package, annotate `IVSClient` explicitly.

## What This Package Gives You

The published maintainer docs for `mypy-boto3-ivs` expose typed surfaces for:

- `IVSClient`
- paginators such as `ListChannelsPaginator` and `ListStreamsPaginator`
- literal aliases such as `ChannelLatencyModeType`
- generated request and response shapes in `mypy_boto3_ivs.type_defs`

The published IVS stubs docs do not surface a `service_resource` or `waiter` section, so plan around typed client usage rather than `Session.resource("ivs")` or waiter APIs.

## Install

Choose one installation pattern based on how much boto3 typing support you want.

### Best inference: full boto3 stubs

Use this when you want `Session.client("ivs")` overload inference with the least annotation noise:

```bash
python -m pip install "boto3-stubs[ivs]"
```

### Service-specific package only

Use this when you only need IVS typings and are willing to annotate the client explicitly:

```bash
python -m pip install "boto3" "mypy-boto3-ivs==1.42.3"
```

### Lite aggregate package

Use this when the full stubs package is too heavy for your IDE or environment:

```bash
python -m pip install "boto3-stubs-lite[ivs]"
```

Practical rule:

- `boto3-stubs[ivs]`: best editor inference
- `boto3-stubs-lite[ivs]`: lighter, but explicit annotations matter more
- `mypy-boto3-ivs`: standalone IVS typings only

## Setup, Auth, And Region

`mypy-boto3-ivs` adds no auth or config layer. Credentials, region selection, retries, profiles, and endpoints still come from normal `boto3` and `botocore` configuration.

Common local setup:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_ivs.client import IVSClient

session = boto3.Session(profile_name="dev", region_name="us-west-2")
ivs: "IVSClient" = session.client("ivs")
```

IVS is regional, so set `region_name` or `AWS_DEFAULT_REGION` explicitly when your environment is not already pinned to the correct region.

## Core Usage

### Typed client for normal IVS API calls

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_ivs.client import IVSClient

ivs: "IVSClient" = boto3.Session(region_name="us-west-2").client("ivs")

response = ivs.list_channels(maxResults=10)

for channel in response.get("channels", []):
    print(channel["arn"], channel.get("name"))
```

### Typed paginator for scans

Use paginator types instead of hand-rolled next-token loops:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_ivs.client import IVSClient
    from mypy_boto3_ivs.paginator import ListStreamsPaginator

ivs: "IVSClient" = boto3.Session(region_name="us-west-2").client("ivs")

paginator: "ListStreamsPaginator" = ivs.get_paginator("list_streams")

for page in paginator.paginate():
    for stream in page.get("streams", []):
        print(stream["channelArn"], stream.get("health"))
```

### Literals for constrained string parameters

Use generated literal aliases when helper code builds request parameters:

```python
from typing import TYPE_CHECKING

import boto3
from mypy_boto3_ivs.literals import ChannelLatencyModeType

if TYPE_CHECKING:
    from mypy_boto3_ivs.client import IVSClient

ivs: "IVSClient" = boto3.Session(region_name="us-west-2").client("ivs")

latency_mode: ChannelLatencyModeType = "LOW"

ivs.create_channel(
    name="live-app",
    latencyMode=latency_mode,
)
```

### Generated `type_defs` for helper boundaries

The `type_defs` module is useful when you build IVS request dictionaries outside the call site or want typed response-shape helpers. The published docs surface generated definitions such as `AudioConfigurationTypeDef` alongside many request and response shapes.

## Runtime-Safe Typing Pattern

If production environments do not install stub packages, keep stub imports behind `TYPE_CHECKING` and use forward references:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_ivs.client import IVSClient

def make_client() -> "IVSClient":
    return boto3.Session(region_name="us-west-2").client("ivs")
```

This lets type checkers see the annotations without making `mypy-boto3-ivs` a required runtime dependency.

## Common Pitfalls

- The PyPI package name uses hyphens, but Python imports use underscores: `mypy_boto3_ivs`.
- This package is typing-only. Real requests still come from `boto3`.
- If you install only `mypy-boto3-ivs` or `boto3-stubs-lite[ivs]`, do not expect all `Session.client("ivs")` calls to infer automatically.
- The boto3 service name is `"ivs"`, not `"mypy-boto3-ivs"`.
- IVS is regional. Missing or wrong region configuration is a boto3 setup problem, not a stub-package problem.
- Typed responses are still ordinary Python dictionaries at runtime; the stubs help static analysis, not runtime validation.
- Only some IVS list operations are paginated. Use `get_paginator(...)` only for operations that boto3 exposes as paginators, such as `list_channels` and `list_streams`.

## Version-Sensitive Notes

- The version used here for this session was `1.42.3`, and the live PyPI project page also showed `1.42.3` on March 12, 2026.
- The maintainer package page says `mypy-boto3-ivs` uses the same version as the related `boto3` release, so keep your stub package close to the boto3 line you actually run.
- The docs root is a stable package URL, not a release-pinned docs URL. Use PyPI as the source of truth when you need an exact lockfile pin.
- If your runtime `boto3` moves ahead of the installed stubs, newly added IVS operations or shape fields may be missing from editor and type-checker output.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_ivs/`
- Versioning and install guidance: `https://youtype.github.io/boto3_stubs_docs/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-ivs/`
- Boto3 IVS service reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/ivs.html`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`

---
name: boto3-stubs
description: "boto3-stubs package guide for Python with service extras, typed boto3 clients/resources, and AWS config notes for mypy and pyright"
metadata:
  languages: "python"
  versions: "1.42.66"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,boto3-stubs,typing,mypy,pyright,python"
---

# boto3-stubs Python Package Guide

## What It Is

`boto3-stubs` adds generated type information and editor completion for `boto3`. It covers service clients, resources, paginators, waiters, and typed dictionaries, but it is not the runtime AWS SDK. Your code still runs through normal `boto3` sessions, clients, and resources.

The official docs position it for:

- `mypy` and `pyright` type checking
- IDE completion for `boto3` services
- explicit imports like `mypy_boto3_s3.client.S3Client`
- generated paginator, waiter, and `type_defs` modules

## Golden Rules

- Keep `boto3` installed for runtime calls. `boto3-stubs` only improves static typing.
- Install the service extras you actually use, or use `essential` / `full` when you want convenience over a small environment.
- Add explicit annotations for `session.client("...")` and `session.resource("...")` when inference is ambiguous.
- Configure credentials, profiles, regions, retries, and endpoints through `boto3` and AWS config, not through `boto3-stubs`.
- If the stubs are development-only, guard imports with `TYPE_CHECKING` so production environments do not need them.

## Install

For most projects, pin the stub version explicitly and keep it aligned with the `boto3` version you are using.

```bash
python -m pip install "boto3-stubs==1.42.66"
```

Common install patterns from the official docs:

```bash
# Install matching boto3 with the stubs
python -m pip install "boto3-stubs[boto3]==1.42.66"

# Common AWS services
python -m pip install "boto3-stubs[essential]==1.42.66"

# Only the services you need
python -m pip install "boto3-stubs[s3,sts,dynamodb]==1.42.66"

# All generated service stubs
python -m pip install "boto3-stubs[full]==1.42.66"
```

Practical install rule:

- use `[boto3]` when you want the runtime SDK and the stubs to stay aligned by default
- use service extras when you care about smaller environments and faster installs
- use `[full]` only when you genuinely need broad service coverage

## Initialize Type Checking

Install a type checker in the same environment as `boto3` and `boto3-stubs`.

```bash
python -m pip install mypy
python -m mypy app.py
```

`pyright` also works with the generated stubs. If you only need editor completion and CI type checks, keep `boto3-stubs` as a development dependency.

## Core Usage

### Typed Session Client

Use normal `boto3` runtime objects and import the matching generated types.

```python
from boto3.session import Session
from mypy_boto3_s3.client import S3Client
from mypy_boto3_s3.paginator import ListObjectsV2Paginator

session = Session(profile_name="dev", region_name="us-west-2")

s3: S3Client = session.client("s3")
paginator: ListObjectsV2Paginator = s3.get_paginator("list_objects_v2")

for page in paginator.paginate(Bucket="my-bucket", Prefix="logs/"):
    for item in page.get("Contents", []):
        print(item["Key"])
```

This explicit annotation pattern is the safest default for coding agents because the client type is obvious to the checker and to the reader.

### Typed Resource

```python
from boto3.session import Session
from mypy_boto3_s3.service_resource import S3ServiceResource

session = Session(region_name="us-east-1")
s3: S3ServiceResource = session.resource("s3")

for bucket in s3.buckets.all():
    print(bucket.name)
```

### `TYPE_CHECKING` For Dev-Only Stub Imports

If your production runtime does not install the stubs, keep type imports behind `TYPE_CHECKING`.

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_sts.client import STSClient

session = Session(profile_name="dev", region_name="us-east-1")
sts: "STSClient" = session.client("sts")

identity = sts.get_caller_identity()
print(identity["Arn"])
```

### Service Extras And Import Mapping

The package exposes service-specific modules such as:

- `mypy_boto3_s3.client`
- `mypy_boto3_s3.service_resource`
- `mypy_boto3_s3.paginator`
- `mypy_boto3_s3.waiter`
- `mypy_boto3_s3.type_defs`

Those imports only exist when the matching service stubs are installed through:

- `boto3-stubs[s3]`
- `boto3-stubs[essential]`
- `boto3-stubs[full]`
- or the standalone service package such as `mypy-boto3-s3`

If `from mypy_boto3_s3.client import S3Client` fails, the usual problem is a missing service extra, not a broken AWS client.

## Config And Authentication

`boto3-stubs` does not add its own authentication layer. Use the normal `boto3` and AWS credential/config flow.

Typical inputs:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `~/.aws/config`
- `~/.aws/credentials`

Typical local setup:

```python
from boto3.session import Session
from mypy_boto3_sts.client import STSClient

session = Session(profile_name="dev", region_name="us-west-2")
sts: STSClient = session.client("sts")

print(sts.get_caller_identity()["Account"])
```

Important constraint for agents: a typed client can still fail at runtime if the AWS profile, region, endpoint, or IAM permissions are wrong.

## Common Pitfalls

- Installing `boto3-stubs` and forgetting to install `boto3` for runtime AWS calls.
- Importing `mypy_boto3_<service>` modules without installing the matching service extra.
- Expecting type inference to always understand `session.client("service")` without an explicit annotation.
- Assuming successful type checking means credentials, region, or permissions are correct.
- Using `boto3-stubs-lite` and expecting the same `session.client(...)` and `session.resource(...)` overload coverage. The official docs call out that the lite variant does not provide those overloads.
- Mixing `boto3-stubs` imports with `types-boto3` imports in the same code path without planning the migration.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.42.66`.
- The official docs describe `boto3-stubs` versions as following the related `boto3` version, so pinning matching versions is the safest default.
- The documentation site is generated and unversioned. Search snippets and older examples can show stale patch numbers, so prefer your lockfile and the exact PyPI release page when pinning dependencies.
- The maintainer repository documents `types-boto3` as the successor project. This entry remains intentionally focused on the published `boto3-stubs` package and its `mypy_boto3_<service>` import surface.

## Official Sources

- Docs root: https://youtype.github.io/boto3_stubs_docs/
- Package page: https://pypi.org/project/boto3-stubs/
- Exact release page: https://pypi.org/project/boto3-stubs/1.42.66/
- Current repository: https://github.com/youtype/types-boto3
- Boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- Boto3 configuration guide: https://docs.aws.amazon.com/boto3/latest/guide/configuration.html

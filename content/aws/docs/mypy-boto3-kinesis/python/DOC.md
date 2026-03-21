---
name: mypy-boto3-kinesis
description: "mypy-boto3-kinesis type stubs for typed boto3 Kinesis clients, paginators, waiters, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.41"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,kinesis,boto3,python,typing,stubs,mypy,pyright"
---

# mypy-boto3-kinesis Python Package Guide

## What It Is

`mypy-boto3-kinesis` is the maintainer-generated type stub package for the Amazon Kinesis Data Streams part of `boto3`.

Use it when you want static typing and editor completion for:

- `KinesisClient`
- generated paginator types such as `ListStreamsPaginator`
- generated waiter types such as `StreamExistsWaiter`
- generated `Literal` unions for Kinesis enum-like values
- generated `TypedDict` request and response shapes under `type_defs`

This package does not replace `boto3` or change AWS runtime behavior. It only adds typing information for normal boto3 Kinesis code.

## Golden Rules

- Keep `boto3` installed. This package is only the typing layer.
- Prefer `boto3-stubs[kinesis]` when you want `Session.client("kinesis")` overload inference with minimal annotation noise.
- Use `mypy-boto3-kinesis` directly when you only need Kinesis typings and are willing to annotate clients explicitly.
- Configure credentials, region, retries, and endpoints through normal AWS and boto3 configuration, not through this package.
- Treat service semantics such as shard iterators, partial `PutRecords` failures, and stream state transitions as AWS concerns, not stub-package concerns.

## Install

### Recommended for most projects

```bash
python -m pip install boto3 "boto3-stubs[kinesis]"
```

This is the cleanest option when you want `boto3` plus the generated overloads that let editors and checkers infer `Session.client("kinesis")` automatically.

### Standalone Kinesis stubs

```bash
python -m pip install boto3 "mypy-boto3-kinesis==1.42.41"
```

Use this when you only want the Kinesis stub package. In this mode, explicit client annotations are usually the clearest approach.

### Lower-memory fallback

```bash
python -m pip install boto3 "boto3-stubs-lite[kinesis]"
```

The lite package is more memory-friendly, but the maintainer docs note that it omits the `Session.client(...)` and `Session.resource(...)` overloads. Expect to annotate clients explicitly.

### Generate against an exact boto3 line

If your project pins a different boto3 release family and you need the stubs to match it closely, generate locally:

```bash
uvx --with "boto3==1.42.41" mypy-boto3-builder
```

## Initialize And Setup

Create the real client with `boto3`, then add the Kinesis type from the stubs package:

```python
from boto3.session import Session
from mypy_boto3_kinesis import KinesisClient

session = Session(profile_name="dev", region_name="us-east-1")
client: KinesisClient = session.client("kinesis")
```

Normal AWS credential and region resolution still applies. Practical local options are:

- `profile_name` plus `~/.aws/config` and `~/.aws/credentials`
- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_DEFAULT_REGION`
- role-based credentials in ECS, EKS, Lambda, or EC2

Useful environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_MAX_ATTEMPTS`
- `AWS_RETRY_MODE`

## Core Usage

### Typed Kinesis client

```python
from boto3.session import Session
from mypy_boto3_kinesis import KinesisClient

client: KinesisClient = Session(region_name="us-east-1").client("kinesis")

response = client.describe_stream_summary(StreamName="orders")
summary = response["StreamDescriptionSummary"]

print(summary["StreamARN"])
print(summary["OpenShardCount"])
print(summary["StreamStatus"])
```

AWS recommends `describe_stream_summary` plus `list_shards` for most stream inspection work instead of building new code around the older `describe_stream` API.

### Typed producer call

`Data` is bytes, not plain text. Encode payloads explicitly before sending them.

```python
import json
from boto3.session import Session
from mypy_boto3_kinesis import KinesisClient

client: KinesisClient = Session(region_name="us-east-1").client("kinesis")

record = {"order_id": "ord-123", "status": "created"}

result = client.put_record(
    StreamName="orders",
    PartitionKey=record["order_id"],
    Data=json.dumps(record).encode("utf-8"),
)

print(result["SequenceNumber"])
```

### `TYPE_CHECKING` pattern

Use this when the stub package is installed only in development or CI:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_kinesis import KinesisClient
else:
    KinesisClient = object

client: KinesisClient = boto3.client("kinesis", region_name="us-east-1")
```

This also avoids common `pylint` false positives around typing-only imports.

### Typed paginators

The AWS Kinesis boto3 client exposes these paginators: `DescribeStream`, `ListShards`, `ListStreamConsumers`, and `ListStreams`.

```python
from boto3.session import Session
from mypy_boto3_kinesis import KinesisClient
from mypy_boto3_kinesis.paginator import ListStreamsPaginator

client: KinesisClient = Session(region_name="us-east-1").client("kinesis")
paginator: ListStreamsPaginator = client.get_paginator("list_streams")

for page in paginator.paginate(Limit=25):
    for stream_name in page.get("StreamNames", []):
        print(stream_name)
```

### Typed waiters

The AWS Kinesis client exposes `stream_exists` and `stream_not_exists` waiters.

```python
from boto3.session import Session
from mypy_boto3_kinesis import KinesisClient
from mypy_boto3_kinesis.waiter import StreamExistsWaiter

client: KinesisClient = Session(region_name="us-east-1").client("kinesis")
waiter: StreamExistsWaiter = client.get_waiter("stream_exists")

waiter.wait(StreamName="orders")
```

### Literals and `TypedDict` shapes

Use generated literal types for enum-like values and `type_defs` when you want stronger typing around helper functions.

```python
from typing import TYPE_CHECKING

from mypy_boto3_kinesis.literals import StreamStatusType

if TYPE_CHECKING:
    from mypy_boto3_kinesis.type_defs import PutRecordOutputTypeDef

status: StreamStatusType = "ACTIVE"
output: "PutRecordOutputTypeDef"
```

This is useful when validating configuration helpers or normalizing AWS responses into internal models.

## Configuration And Authentication

`mypy-boto3-kinesis` has no package-specific configuration surface. If authentication, region selection, retries, or endpoints are wrong, fix the underlying boto3 client configuration.

Typical setup patterns:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
client = session.client("kinesis")
```

For local emulators such as LocalStack, set `endpoint_url=...` on the boto3 client. The stubs package does not change endpoint behavior.

## Common Pitfalls

- Installing only the stubs and expecting real AWS calls to work without `boto3`.
- Importing `mypy-boto3-kinesis` with hyphens in Python code. The import root is `mypy_boto3_kinesis`.
- Expecting automatic `Session.client("kinesis")` inference when you installed only `mypy-boto3-kinesis` or `boto3-stubs-lite[kinesis]`. Use explicit annotations in those setups.
- Treating successful type checking as proof that IAM permissions, credentials, region, endpoint selection, or stream state are correct.
- Sending plain strings to `PutRecord` or `PutRecords` instead of bytes.
- Ignoring partial failures from `put_records`; Kinesis can return a successful HTTP response with failed individual records.
- Building new logic around `describe_stream` without noticing the AWS docs recommend `describe_stream_summary` plus `list_shards` for most cases.
- Letting `boto3`, `botocore`, and the stubs drift too far apart. Generated types can become stale even if some runtime calls still succeed.

## Version-Sensitive Notes

- This entry is pinned to `1.42.41`, which matched the initial package metadata and the public PyPI release checked on 2026-03-12.
- The PyPI package page for `1.42.41` says it is compatible with the Kinesis portion of `boto3 1.42.41`.
- The PyPI package page also reports that this release was generated with `mypy-boto3-builder 8.12.0`.
- Maintainer docs and PyPI both present `boto3-stubs[kinesis]`, `boto3-stubs-lite[kinesis]`, and the standalone `mypy-boto3-kinesis` package as valid install paths.
- If your project is pinned to a different boto3 release family, prefer matching stub versions or generate locally for the exact boto3 version you ship.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_kinesis/`
- PyPI project: `https://pypi.org/project/mypy-boto3-kinesis/`
- PyPI release page: `https://pypi.org/project/mypy-boto3-kinesis/1.42.41/`
- Boto3 Kinesis service reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/kinesis.html`
- Boto3 Kinesis `describe_stream` reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/kinesis/client/describe_stream.html`
- Boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- Boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
- Repository family: `https://github.com/youtype/types-boto3`

---
name: mypy-boto3-firehose
description: "mypy-boto3-firehose package guide for typed boto3 Firehose clients in Python"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,firehose,boto3,type-stubs,mypy,pyright"
---

# mypy-boto3-firehose Python Package Guide

## Golden Rule

Use `boto3` for the runtime Firehose client and use `mypy-boto3-firehose` only to add static types for editors, mypy, and pyright.

If you want `Session.client("firehose")` to infer the service type automatically, install `boto3-stubs[firehose]`. If you prefer the smaller service-only package, install `mypy-boto3-firehose` and annotate the client explicitly.

## Install

Service-only stubs:

```bash
python -m pip install "mypy-boto3-firehose==1.42.3" "boto3==1.42.3"
```

Bundled boto3 stubs with Firehose extras:

```bash
python -m pip install "boto3-stubs[firehose]==1.42.3"
```

Lite variant if you do not want the heavy `boto3-stubs` session overloads:

```bash
python -m pip install "boto3-stubs-lite[firehose]==1.42.3"
```

Other package managers:

```bash
uv add "mypy-boto3-firehose==1.42.3" "boto3==1.42.3"
poetry add "mypy-boto3-firehose==1.42.3" "boto3==1.42.3"
```

## Setup And Authentication

This package does not configure AWS credentials by itself. Authentication, region selection, retries, custom endpoints, and profiles all come from the normal `boto3` and `botocore` setup.

Common credential sources for `boto3`:

1. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
2. Shared AWS config in `~/.aws/config` and `~/.aws/credentials`
3. IAM roles or workload credentials when running on AWS

Basic setup:

```bash
aws configure
```

Typed client with an explicit annotation:

```python
import boto3
from mypy_boto3_firehose import FirehoseClient

session = boto3.session.Session(profile_name="dev", region_name="us-east-1")
firehose: FirehoseClient = session.client("firehose")
```

If you need a non-AWS endpoint, keep using normal boto3 client arguments:

```python
import boto3
from mypy_boto3_firehose import FirehoseClient

session = boto3.session.Session(region_name="us-east-1")
firehose: FirehoseClient = session.client(
    "firehose",
    endpoint_url="http://localhost:4566",
)
```

## Core Usage

### Annotate the client without changing runtime behavior

The stubs package is useful even if you keep runtime imports limited to `boto3`:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_firehose import FirehoseClient

session = boto3.session.Session(region_name="us-east-1")
firehose: "FirehoseClient" = session.client("firehose")
```

This pattern keeps the import type-only while still giving completions and static checking.

### Type request payloads with generated `TypedDict` definitions

The package publishes service-specific `TypedDict` names for Firehose request and response shapes. `RecordTypeDef` is useful for producer code that batches records:

```python
import json

import boto3
from mypy_boto3_firehose import FirehoseClient
from mypy_boto3_firehose.type_defs import RecordTypeDef

session = boto3.session.Session(region_name="us-east-1")
firehose: FirehoseClient = session.client("firehose")

record: RecordTypeDef = {
    "Data": (json.dumps({"event": "signup", "user_id": 123}) + "\n").encode("utf-8"),
}

response = firehose.put_record_batch(
    DeliveryStreamName="events-stream",
    Records=[record],
)

if response["FailedPutCount"]:
    raise RuntimeError(response["RequestResponses"])
```

Other useful generated Firehose shapes documented upstream include:

- `CreateDeliveryStreamInputTypeDef`
- `DeliveryStreamDescriptionTypeDef`
- `TagDeliveryStreamInputRequestTypeDef`

### Use literal types when AWS expects specific string enums

The stubs package also exposes literal aliases for enum-like string values:

```python
from mypy_boto3_firehose.literals import AmazonOpenSearchServerlessS3BackupModeType

backup_mode: AmazonOpenSearchServerlessS3BackupModeType = "AllDocuments"
```

This is most useful when you build larger typed config dictionaries for `create_delivery_stream(...)`.

### Get automatic client inference with `boto3-stubs[firehose]`

With the full `boto3-stubs` extra, `Session.client("firehose")` is overloaded so manual annotations are often unnecessary:

```python
import boto3

session = boto3.session.Session(region_name="us-east-1")
firehose = session.client("firehose")
```

With the standalone `mypy-boto3-firehose` package or the lite variant, prefer explicit annotations.

## Configuration Notes

- Keep `boto3`, `botocore`, and the stubs package on matching release lines when possible. The package version tracks the boto3 model version it was generated from.
- Region matters for Firehose stream operations. Set `region_name` explicitly when the environment could be ambiguous.
- Custom retry config, STS assume-role flows, and endpoint overrides belong on the boto3 session or client, not in the stubs package.
- Firehose streams are service resources, not local objects. Creating or updating a delivery stream can be asynchronous on the AWS side even though the typed method call returns immediately.

## Common Pitfalls

- Do not treat `mypy-boto3-firehose` as a runtime SDK. It adds types; the actual API calls still come from `boto3`.
- Do not assume the standalone package gives you overloaded `Session.client("firehose")`. That convenience comes from `boto3-stubs`, while `boto3-stubs-lite` explicitly omits those overloads.
- Do not let your stubs drift far from your boto3 version. Missing methods or stale `TypedDict` fields usually mean the model versions no longer match.
- Keep type-only imports behind `TYPE_CHECKING` if you do not want the stubs package imported at runtime.
- Firehose `put_record` and `put_record_batch` accept bytes payloads. Encode structured data yourself, typically JSON Lines for downstream S3-based destinations.
- For stream creation flows, validate the destination-specific config against the live AWS Firehose API docs. The stubs help with field names and literals, but they do not replace service-side validation rules.

## Version-Sensitive Notes For `1.42.3`

- PyPI currently lists `1.42.3` as the package version and marks it compatible with `boto3 1.42.3`.
- PyPI classifiers for `1.42.3` include Python `3.9` through `3.14`, with `Requires: Python >=3.9`.
- The package page shows `mypy-boto3-builder 8.12.0` as the generator used for this release.
- The published docs for this package live under the `youtype.github.io` generated docs tree and match the `1.42.3` release line on March 12, 2026.

## Official Sources

- PyPI package page: https://pypi.org/project/mypy-boto3-firehose/
- Package docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_firehose/
- Firehose typed definitions: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_firehose/type_defs/
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- Boto3 Firehose client reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/firehose.html

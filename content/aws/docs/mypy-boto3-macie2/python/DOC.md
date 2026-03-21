---
name: mypy-boto3-macie2
description: "mypy-boto3-macie2 package guide for typed boto3 Macie2 clients, paginators, waiters, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.40.16"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,macie2,boto3,typing,stubs,mypy,pyright,python"
---

# mypy-boto3-macie2 Python Package Guide

## What It Is

`mypy-boto3-macie2` is a stubs-only package for the Macie2 part of `boto3`. Keep using `boto3` for runtime AWS calls, and install this package or `boto3-stubs[macie2]` only to improve static typing, editor completion, and request or response shape checking.

Macie2 is client-oriented in boto3. The maintainer docs expose typed clients, paginators, waiters, literals, and `type_defs`, but not a service-resource workflow.

Use it when you want:

- a typed `Macie2Client` for `Session.client("macie2")`
- typed paginator and waiter objects
- literal unions for string-valued fields such as finding status enums
- generated `TypedDict` request and response shapes for stricter Macie calls

## Install

### Recommended: full `boto3-stubs` extra

Use this when you want the smoothest typing for `Session().client("macie2")` without adding explicit casts everywhere.

```bash
python -m pip install "boto3==1.40.16" "boto3-stubs[macie2]==1.40.16"
```

### Standalone service stub package

Use this when you only want the Macie2 stubs and do not mind explicit type annotations.

```bash
python -m pip install "boto3==1.40.16" "mypy-boto3-macie2==1.40.16"
```

### Lite variant

Use this when IDE memory or indexing cost matters more than automatic overload inference.

```bash
python -m pip install "boto3==1.40.16" "boto3-stubs-lite[macie2]==1.40.16"
```

Install notes:

- `mypy-boto3-macie2` does not include the runtime SDK. Without `boto3`, nothing works at runtime.
- `boto3-stubs[macie2]` is usually the best development experience because it types `boto3.client(...)` and `Session.client(...)`.
- `boto3-stubs-lite[macie2]` reduces overload coverage, so explicit imported types are often needed.
- The maintainer docs root is a rolling latest page. Use PyPI for the exact release pin and the docs root for current package structure.

## AWS Authentication And Region Setup

This package does not change AWS authentication. Credential lookup, region resolution, retries, and endpoint configuration still come from `boto3` and `botocore`.

Typical local setup:

```bash
aws configure --profile security
export AWS_PROFILE=security
export AWS_DEFAULT_REGION=us-east-1
```

Important boto3 rules:

1. Credentials passed directly to `Session(...)` or `client(...)` win first.
2. Then boto3 checks environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`.
3. Then it checks shared AWS config and credentials files, followed by role-based providers such as IAM Identity Center, container credentials, or EC2 instance metadata.
4. Region selection can come from the session, the client call, or `AWS_DEFAULT_REGION`.

If you need retry or timeout tuning, pass a normal botocore `Config`:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_macie2.client import Macie2Client

session = Session(profile_name="security", region_name="us-east-1")
config = Config(
    retries={"mode": "standard", "max_attempts": 10},
)

macie: Macie2Client = session.client("macie2", config=config)
```

## Core Usage

### Typed Macie2 client

```python
from boto3.session import Session
from mypy_boto3_macie2.client import Macie2Client

session = Session(profile_name="security", region_name="us-east-1")
macie: Macie2Client = session.client("macie2")

response = macie.get_macie_session()
print(response["status"])
```

This is the main pattern when you install `mypy-boto3-macie2` directly instead of the umbrella extras package.

### Typed paginators

The maintainer docs expose paginator types such as `DescribeBucketsPaginator` and `ListFindingsPaginator`.

```python
from boto3.session import Session
from mypy_boto3_macie2.client import Macie2Client
from mypy_boto3_macie2.paginator import DescribeBucketsPaginator

macie: Macie2Client = Session(region_name="us-east-1").client("macie2")
paginator: DescribeBucketsPaginator = macie.get_paginator("describe_buckets")

for page in paginator.paginate():
    for item in page.get("buckets", []):
        print(item["bucketName"])
```

### Typed waiters

Macie2 also exposes a waiter type for delayed finding visibility.

```python
from boto3.session import Session
from mypy_boto3_macie2.client import Macie2Client
from mypy_boto3_macie2.waiter import FindingRevealedWaiter

macie: Macie2Client = Session(region_name="us-east-1").client("macie2")
waiter: FindingRevealedWaiter = macie.get_waiter("finding_revealed")

waiter.wait(id="example-finding-id")
```

### Literals And TypedDict helpers

Use the generated `literals` and `type_defs` modules when you want stronger typing than `dict[str, Any]`.

```python
from mypy_boto3_macie2.literals import FindingStatusType
from mypy_boto3_macie2.type_defs import BucketCriteriaForJobTypeDef

status: FindingStatusType = "NEW"

criteria: BucketCriteriaForJobTypeDef = {
    "excludes": {
        "and": [
            {
                "simpleCriterion": {
                    "comparator": "EQ",
                    "key": "S3_BUCKET_EFFECTIVE_PERMISSION",
                    "values": ["PUBLIC"],
                }
            }
        ]
    }
}
```

### `TYPE_CHECKING` pattern

If production images omit dev-only stub packages, keep the imports behind `TYPE_CHECKING` and annotate with a string type:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_macie2.client import Macie2Client
else:
    Macie2Client = object

def make_client() -> "Macie2Client":
    return Session(region_name="us-east-1").client("macie2")
```

## Configuration Notes

- The runtime service name is still `"macie2"`, not `"mypy-boto3-macie2"`.
- Use `Session(region_name=...)` or `client(..., region_name=...)` explicitly when the environment is not unambiguous.
- Retry, proxy, TLS, and timeout settings still come from `botocore.config.Config`.
- Generated types help with request parameters and response shapes, but they do not guarantee that your IAM permissions, Macie account status, or region setup are correct.

## Common Pitfalls

- Installing only `mypy-boto3-macie2` and expecting a working SDK at runtime. You still need `boto3`.
- Using the package name instead of the boto3 service name. The client call is `session.client("macie2")`.
- Expecting `boto3-stubs-lite[macie2]` to infer `Session().client("macie2")` automatically.
- Copying the rolling docs root without checking the installed PyPI version when exact generated symbol names matter.
- Treating the stub package as an auth or config layer. Credentials, profiles, regions, retries, and endpoints are still boto3 concerns.
- Assuming every boto3 service has a typed resource layer. Macie2 is documented here as a typed client, paginator, waiter, literal, and `type_defs` surface.

## Version-Sensitive Notes

- The maintainer docs site is a rolling latest view, while PyPI is the safer source for exact published version pins. This guide is pinned to `1.40.16`, the package version visible on PyPI on `2026-03-12`.
- The maintainer versioning docs say stub package versions track the related `boto3` version line, so keep `boto3` and the stub package aligned when exact API coverage matters.
- PyPI marks the package as `Typing :: Stubs Only`; this is a development-time dependency, not a runtime AWS client.
- The maintainer docs root is not patch-pinned. Prefer PyPI for exact installation pins and use the docs site to inspect the latest module layout and generated symbols.

## Official Sources Used

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_macie2/`
- Maintainer versioning docs: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI project page: `https://pypi.org/project/mypy-boto3-macie2/`
- AWS Macie2 boto3 reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/macie2.html`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`

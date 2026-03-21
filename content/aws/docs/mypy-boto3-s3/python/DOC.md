---
name: mypy-boto3-s3
description: "mypy-boto3-s3 type stubs for boto3 S3 clients, resources, paginators, waiters, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.37"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,s3,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-s3 Python Package Guide

## Golden Rule

`mypy-boto3-s3` is a typing package for `boto3` S3 code. It does not replace `boto3`, it does not make AWS calls on its own, and it does not manage credentials or regions for you.

Use it in one of these modes:

- Install `boto3-stubs[s3]` for the best editor and type-checker experience with automatic `Session.client("s3")` and `Session.resource("s3")` overloads.
- Install `mypy-boto3-s3` when you want only the standalone S3 stubs and are willing to add explicit type annotations.
- Install `boto3-stubs-lite[s3]` when IDE memory use matters more than automatic overload inference.

## Install

### Recommended for most projects

```bash
python -m pip install boto3 'boto3-stubs[s3]'
```

This is the maintainer-recommended path when you want autocomplete plus type inference without annotating every S3 variable.

### Standalone S3 stubs

```bash
python -m pip install boto3 mypy-boto3-s3
```

Use this when you want only the S3 stubs package. In this mode, explicit annotations are usually necessary.

### Lower-memory IDE fallback

```bash
python -m pip install boto3 'boto3-stubs-lite[s3]'
```

The lite package is more memory-friendly, especially for PyCharm, but it does not provide `session.client()` and `session.resource()` overloads. Add explicit annotations if you use it.

## Runtime Setup And Auth

`mypy-boto3-s3` has no package-specific initialization. All runtime behavior still comes from `boto3`.

AWS documents that Boto3 requests need both AWS credentials and an AWS Region. The usual setup is either:

```bash
aws configure
```

or environment variables:

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_DEFAULT_REGION=us-west-2
```

For explicit account, profile, or region control, create a session:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-west-2")
```

## Core Usage

### Typed client and resource

```python
from boto3.session import Session
from mypy_boto3_s3.client import S3Client
from mypy_boto3_s3.service_resource import S3ServiceResource

session = Session(profile_name="dev", region_name="us-west-2")

s3_client: S3Client = session.client("s3")
s3_resource: S3ServiceResource = session.resource("s3")
```

Use the client for full API coverage. AWS says Boto3 clients are the low-level interface and map closely to service APIs. AWS also says the resources interface is not getting new features, so newer S3 capabilities may only appear on the client side.

### Typed paginator and waiter

```python
from boto3.session import Session
from mypy_boto3_s3.client import S3Client
from mypy_boto3_s3.paginator import ListObjectsV2Paginator
from mypy_boto3_s3.waiter import ObjectExistsWaiter

client: S3Client = Session(region_name="us-west-2").client("s3")

paginator: ListObjectsV2Paginator = client.get_paginator("list_objects_v2")
waiter: ObjectExistsWaiter = client.get_waiter("object_exists")

for page in paginator.paginate(Bucket="my-bucket", Prefix="incoming/"):
    for item in page.get("Contents", []):
        print(item["Key"])

waiter.wait(Bucket="my-bucket", Key="incoming/report.csv")
```

### Typed resources and collections

```python
from boto3.session import Session
from mypy_boto3_s3.service_resource import Bucket, ObjectSummary, S3ServiceResource

resource: S3ServiceResource = Session(region_name="us-west-2").resource("s3")

bucket: Bucket = resource.Bucket("my-bucket")

for obj in bucket.objects.filter(Prefix="incoming/"):
    typed_obj: ObjectSummary = obj
    print(typed_obj.key)
```

### TypedDict request and response shapes

```python
from boto3.session import Session
from mypy_boto3_s3.client import S3Client
from mypy_boto3_s3.literals import BucketLocationConstraintType
from mypy_boto3_s3.type_defs import GetObjectOutputTypeDef

client: S3Client = Session(region_name="us-west-2").client("s3")

location: BucketLocationConstraintType = "us-west-2"
client.create_bucket(
    Bucket="my-bucket",
    CreateBucketConfiguration={"LocationConstraint": location},
)

response: GetObjectOutputTypeDef = client.get_object(
    Bucket="my-bucket",
    Key="data.json",
)
body = response["Body"].read()
```

Use `literals` when an API field accepts a constrained string set, and use `type_defs` when you need `TypedDict` coverage for request or response shapes.

## Tooling Patterns

### Explicit annotations for standalone stubs

The standalone `mypy-boto3-s3` package is most useful when you annotate clients, resources, paginators, waiters, and `TypedDict` values directly. That keeps type checking deterministic across mypy, pyright, and IDEs.

### Keep the stubs as a dev-only dependency

If production images do not install stub packages, gate the imports with `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_s3.client import S3Client

def make_client() -> "S3Client":
    return Session(region_name="us-west-2").client("s3")
```

This avoids a runtime import dependency while preserving type information.

## Common Pitfalls

- Installing only `mypy-boto3-s3` and expecting unannotated `session.client("s3")` calls to become typed automatically. That behavior comes from `boto3-stubs[s3]`.
- Forgetting to install `boto3`. These packages are type stubs, not the runtime SDK.
- Treating the stubs package as AWS auth or config middleware. Credentials, region, retries, and endpoints still come from normal `boto3` and botocore setup.
- Defaulting to the resource API for every new S3 feature. AWS documents that new features land on clients, not resources.
- Using `boto3-stubs-lite[s3]` and then expecting overload-based inference from `session.client("s3")`. Lite mode needs more explicit annotations.
- Copying examples from other service stub packages. Imports and class names are service-specific; use `mypy_boto3_s3`, not generic or EC2 examples.

## Version-Sensitive Notes

- The version used here for this package is `1.42.37`, and the PyPI project page currently lists `1.42.37` as the published release for `mypy-boto3-s3`.
- The maintainer states that `mypy-boto3-s3` versions match the related `boto3` version. Pin them together when exact shape coverage matters.
- The maintainer also documents a local generation path with `mypy-boto3-builder`. If you need exact stubs for a pinned `boto3` version or a private fork, generate them locally instead of assuming the hosted docs are version-perfect for your environment.
- Practical rule: pin `boto3==1.42.37` with `mypy-boto3-s3==1.42.37` when you want the closest alignment to this entry.

## Official Sources

- Docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_s3/
- PyPI: https://pypi.org/project/mypy-boto3-s3/
- Homepage: https://youtype.github.io/boto3_stubs_docs/
- Source repository: https://github.com/youtype/mypy_boto3_builder
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- Boto3 clients guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/clients.html
- Boto3 resources guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/resources.html

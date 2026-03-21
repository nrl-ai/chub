---
name: mypy-boto3-ecr
description: "Type stubs for boto3 Amazon ECR clients, paginators, waiters, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.57"
  revision: 3
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,ecr,boto3,mypy,pyright,typing,type-stubs"
---

# mypy-boto3-ecr Python Package Guide

## Golden Rule

`mypy-boto3-ecr` adds static types for the Amazon ECR client in `boto3`.

It helps with:

- autocompletion for ECR client methods
- typed paginator and waiter names
- `TypedDict` request and response shapes
- literal string unions used by the ECR API

It does not replace `boto3` at runtime. AWS credentials, regions, retries, endpoints, and API behavior still come from `boto3` and your AWS configuration.

## Install

### Standalone ECR stubs

Install the runtime SDK plus the ECR service stubs:

```bash
python -m pip install boto3 "mypy-boto3-ecr==1.42.57"
```

Use this when you only want ECR typing support.

### Full boto3 stubs bundle

If you want typed overloads across multiple AWS services, use the maintained extras package:

```bash
python -m pip install boto3 "boto3-stubs[ecr]"
```

This is usually the easiest option when your project uses more than one AWS service.

### Lower-memory fallback

If IDE memory use matters more than automatic `Session.client(...)` inference:

```bash
python -m pip install boto3 "boto3-stubs-lite[ecr]"
```

With the lite package, explicit client annotations are more important because the generated overloads for `Session.client(...)` are reduced.

## Runtime Setup And Auth

There is no package-specific initialization. Create a normal boto3 session and let AWS resolve credentials through the usual boto3 credential chain.

Typical local environment setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Or configure explicit credentials in environment variables:

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_SESSION_TOKEN=...
export AWS_DEFAULT_REGION=us-west-2
```

Then create a typed ECR client:

```python
import boto3
from mypy_boto3_ecr.client import ECRClient

session = boto3.Session(profile_name="dev", region_name="us-west-2")
ecr: ECRClient = session.client("ecr")
```

For local development, named profiles are usually easier to rotate and audit than hard-coded keys. In AWS-hosted environments, prefer IAM roles.

## Core Usage

### Typed Client Calls

Use normal boto3 runtime calls and add the service client annotation explicitly:

```python
import boto3
from mypy_boto3_ecr.client import ECRClient

ecr: ECRClient = boto3.Session(region_name="us-west-2").client("ecr")

response = ecr.describe_repositories(maxResults=20)

for repository in response.get("repositories", []):
    print(repository["repositoryName"], repository["repositoryUri"])
```

This pattern works well with the standalone package, the full `boto3-stubs` package, and the lite package.

### Create A Repository

The stubs also make write operations easier to discover and shape correctly:

```python
import boto3
from mypy_boto3_ecr.client import ECRClient

ecr: ECRClient = boto3.Session(region_name="us-west-2").client("ecr")

created = ecr.create_repository(
    repositoryName="example-repo",
    imageScanningConfiguration={"scanOnPush": True},
)

print(created["repository"]["repositoryArn"])
```

### Typed Paginators

Use the generated paginator class when you need to iterate through repository pages:

```python
import boto3
from mypy_boto3_ecr.client import ECRClient
from mypy_boto3_ecr.paginator import DescribeRepositoriesPaginator

ecr: ECRClient = boto3.Session(region_name="us-west-2").client("ecr")
paginator: DescribeRepositoriesPaginator = ecr.get_paginator(
    "describe_repositories"
)

for page in paginator.paginate(PaginationConfig={"PageSize": 100}):
    for repository in page.get("repositories", []):
        print(repository["repositoryArn"])
```

### Typed Waiters

ECR exposes generated waiter classes through the stubs package. For example, wait for an image scan to finish:

```python
import boto3
from mypy_boto3_ecr.client import ECRClient
from mypy_boto3_ecr.waiter import ImageScanCompleteWaiter

ecr: ECRClient = boto3.Session(region_name="us-west-2").client("ecr")
waiter: ImageScanCompleteWaiter = ecr.get_waiter("image_scan_complete")

waiter.wait(
    repositoryName="example-repo",
    imageId={"imageTag": "latest"},
    WaiterConfig={"Delay": 5, "MaxAttempts": 60},
)
```

The generated docs also list `LifecyclePolicyPreviewCompleteWaiter`.

### Typed Request Payloads And Literals

When you build request payloads in helpers, the generated `TypedDict` and literal modules make it easier to catch mistakes before runtime:

```python
import boto3
from mypy_boto3_ecr.client import ECRClient
from mypy_boto3_ecr.literals import TagStatusType
from mypy_boto3_ecr.type_defs import DescribeImagesRequestRequestTypeDef

tag_status: TagStatusType = "TAGGED"

request: DescribeImagesRequestRequestTypeDef = {
    "repositoryName": "example-repo",
    "filter": {"tagStatus": tag_status},
}

ecr: ECRClient = boto3.Session(region_name="us-west-2").client("ecr")
images = ecr.describe_images(**request)

for detail in images.get("imageDetails", []):
    print(detail.get("imageDigest"))
```

## Dev-Only Type Imports

If production images install `boto3` but not the stub package, keep type imports behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_ecr.client import ECRClient

session = boto3.Session(region_name="us-west-2")
ecr: "ECRClient" = session.client("ecr")

print(ecr.describe_repositories(maxResults=5)["repositories"])
```

This keeps runtime imports simple while preserving type checking in development and CI.

## Import Surface

The service-specific package exposes the modules most ECR projects need:

- `mypy_boto3_ecr.client`
- `mypy_boto3_ecr.paginator`
- `mypy_boto3_ecr.waiter`
- `mypy_boto3_ecr.type_defs`
- `mypy_boto3_ecr.literals`

For ECR, the generated package is client-oriented. Do not assume there is a typed boto3 resource layer for this service.

## Common Pitfalls

### The Stub Package Does Not Replace boto3

You still need `boto3` installed for runtime AWS calls.

### Package Name And Import Name Differ

Use:

- package name: `mypy-boto3-ecr`
- import root: `mypy_boto3_ecr`

### Type Safety Does Not Fix Auth Or Region Problems

If calls fail with credential, authorization, or region errors, debug them as normal boto3 and AWS configuration issues.

### Keep boto3 And Stub Versions Close

These stubs are generated from boto3 service models. If your installed `boto3` and `mypy-boto3-ecr` versions drift too far apart, type information can become stale even when runtime code still works.

### Lite Installs Need Explicit Annotations

If you use `boto3-stubs-lite[ecr]`, annotate the client explicitly:

```python
from boto3.session import Session
from mypy_boto3_ecr.client import ECRClient

ecr: ECRClient = Session(region_name="us-west-2").client("ecr")
```

## Version Notes

This guide targets `mypy-boto3-ecr` version `1.42.57`.

If you pin dependency versions, keep the ECR stubs aligned with the boto3 release line you use in the same project.

---
name: mypy-boto3-ecs
description: "mypy-boto3-ecs typed boto3 stubs for Amazon ECS clients, resources, literals, paginators, waiters, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.58"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,ecs,boto3,python,typing,mypy,pyright,stubs"
---

# mypy-boto3-ecs Python Package Guide

## Golden Rule

`mypy-boto3-ecs` is a typing package for ECS code written with `boto3`. It improves autocomplete and static checking, but it is not the runtime SDK. Keep `boto3` installed for real AWS calls, credentials, regions, retries, and endpoints.

Choose the install mode deliberately:

- Use `boto3-stubs[ecs]` if you want the best overloads for `Session.client("ecs")` and `Session.resource("ecs")`.
- Use `mypy-boto3-ecs` if you only need ECS-specific types and are willing to annotate ECS clients and resources explicitly.
- Use `boto3-stubs-lite[ecs]` if install size matters and explicit annotations are acceptable.

## Install

### Minimal service-specific setup

```bash
python -m pip install "boto3" "mypy-boto3-ecs==1.42.58"
```

This is the narrowest install if you only need ECS typing.

### Bundled upstream install path

```bash
python -m pip install "boto3-stubs[ecs]==1.42.58"
```

Use this when you want better typing inference around `boto3.session.Session`, `client(...)`, and `resource(...)`.

### Lower-memory fallback

```bash
python -m pip install "boto3-stubs-lite[ecs]==1.42.58"
```

The lite package omits `session.client(...)` and `session.resource(...)` overloads. Add explicit annotations or `cast(...)` when you use it.

## Authentication And Configuration

`mypy-boto3-ecs` does not change how AWS auth works. Use normal `boto3` configuration:

1. Explicit `Session(...)` or `client(...)` arguments
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, `AWS_DEFAULT_REGION`, and `AWS_PROFILE`
3. Shared AWS config and credentials files
4. IAM roles and other runtime credential providers in AWS environments

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Then construct a typed session client:

```python
from boto3.session import Session
from mypy_boto3_ecs import ECSClient

session = Session(profile_name="dev", region_name="us-west-2")
ecs: ECSClient = session.client("ecs")
```

If your project spans multiple accounts or regions, keep the `Session(...)` creation explicit instead of relying on ambient defaults.

## Core Usage

### Typed ECS client

```python
from boto3.session import Session
from mypy_boto3_ecs import ECSClient

ecs: ECSClient = Session(region_name="us-east-1").client("ecs")

response = ecs.describe_clusters(clusters=["default"])

for cluster in response.get("clusters", []):
    print(cluster["clusterArn"], cluster["status"])
```

### Typed ECS resource

The generated stubs also expose the resource surface:

```python
from boto3.session import Session
from mypy_boto3_ecs.service_resource import ECSServiceResource

ecs_resource: ECSServiceResource = Session(region_name="us-east-1").resource("ecs")
```

Prefer the client API for most new code. Use the resource interface only when you already depend on its object model.

### Typed request data for Fargate networking

Use generated `TypedDict` shapes from `mypy_boto3_ecs.type_defs` when you pass ECS-shaped dictionaries between helpers:

```python
from mypy_boto3_ecs.type_defs import AwsVpcConfigurationTypeDef

network_config: AwsVpcConfigurationTypeDef = {
    "subnets": ["subnet-12345678", "subnet-abcdef12"],
    "securityGroups": ["sg-12345678"],
    "assignPublicIp": "DISABLED",
}
```

### Literals for enum-like ECS fields

```python
from mypy_boto3_ecs.literals import LaunchTypeType

launch_type: LaunchTypeType = "FARGATE"
```

This catches typos such as `"fargate"` during type checking instead of at runtime.

### Typed paginator pattern

The generated package includes paginator classes under `mypy_boto3_ecs.paginator`. Even if you do not annotate the paginator class directly, the stubs still improve completion on paginated response pages.

```python
from boto3.session import Session
from mypy_boto3_ecs import ECSClient

ecs: ECSClient = Session(region_name="us-east-1").client("ecs")
paginator = ecs.get_paginator("list_clusters")

for page in paginator.paginate():
    for cluster_arn in page.get("clusterArns", []):
        print(cluster_arn)
```

### Dev-only typing dependencies

If the stub package is installed only in development or CI, guard type-only imports:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_ecs import ECSClient

def get_ecs_client() -> "ECSClient":
    return Session(region_name="us-west-2").client("ecs")
```

## ECS Runtime Notes That Types Do Not Solve

- `RunTask`, `StartTask`, and several describe operations can return `failures` inside an otherwise successful response. Check them explicitly.
- `list_tasks` returns task ARNs, not full task objects. Follow it with `describe_tasks`.
- Fargate tasks need a task definition compatible with `awsvpc` networking plus a valid VPC network configuration.
- IAM, cluster state, capacity providers, service deployment state, and regional availability are runtime concerns. The stub package only validates shapes and names.

## Common Pitfalls

- Installing `mypy-boto3-ecs` and assuming it replaces `boto3`. It does not.
- Importing `mypy-boto3-ecs` with hyphens. Install with hyphens, import with underscores: `mypy_boto3_ecs`.
- Expecting automatic `Session.client("ecs")` inference from the standalone package in every toolchain. Use `boto3-stubs[ecs]` if you want the bundled overloads.
- Choosing `boto3-stubs-lite[ecs]` and forgetting that explicit annotations are required more often.
- Letting `boto3`, `botocore`, and `mypy-boto3-ecs` drift too far apart. New ECS fields can appear in runtime models before matching stubs land.
- Treating better typing as proof that the call will succeed. Region, credentials, IAM, cluster names, task definitions, and service state still need real validation.

## Version-Sensitive Notes

- On `2026-03-12`, the version used here `1.42.58` matches the current PyPI package page for `mypy-boto3-ecs`.
- On `2026-03-12`, the maintainer docs root also points to the ECS stub package for the `1.42.58` line.
- The maintainer project documents these stub packages as tracking the related `boto3` version family. For exact ECS model coverage, pin the stub package close to the `boto3` version you actually deploy.
- Repository branding has moved to `types-boto3`, but the published package name and Python import root remain `mypy-boto3-ecs` and `mypy_boto3_ecs`.

## Official Source URLs

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_ecs/`
- Upstream versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-ecs/`
- PyPI JSON API: `https://pypi.org/pypi/mypy-boto3-ecs/json`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- Boto3 session guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/session.html`
- Boto3 ECS reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/ecs.html`
- Repository: `https://github.com/youtype/types-boto3`

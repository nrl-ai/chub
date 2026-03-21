---
name: mypy-boto3-elbv2
description: "Typed boto3 ELBv2 stubs for Python with install choices, typed clients, paginators, waiters, and request TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,elbv2,alb,nlb,boto3,python,typing,stubs,mypy,pyright"
---

# mypy-boto3-elbv2 Python Package Guide

## Golden Rule

Use `mypy-boto3-elbv2` only as the typing layer for Elastic Load Balancing v2 code written with `boto3`. Keep `boto3` installed for real AWS calls, credentials, retries, and endpoints, and annotate `boto3.client("elbv2")` explicitly unless you install the full `boto3-stubs[elbv2]` extra.

## What This Package Gives You

`mypy-boto3-elbv2` is the generated type-stub package for the ALB and NLB portion of boto3. It gives you typed definitions for:

- `ElasticLoadBalancingv2Client`
- paginator classes such as `DescribeLoadBalancersPaginator`
- waiter classes such as `LoadBalancerAvailableWaiter`
- `TypedDict` request and response shapes under `mypy_boto3_elbv2.type_defs`
- generated literal unions for enum-like ELBv2 values

It does not make requests by itself and it does not replace boto3. The AWS ELBv2 boto3 reference exposes client, paginator, and waiter surfaces; there is no separate high-level boto3 resource API for `elbv2`.

## Install

Pick the install mode based on how much type inference you want.

### Standalone service-specific stubs

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-elbv2==1.42.3"
```

Use this when you only need ELBv2 typing and are willing to annotate the client explicitly.

### Full boto3 stubs package

```bash
python -m pip install "boto3-stubs[elbv2]"
```

Use this when you want the bundled `Session.client("elbv2")` overloads with less annotation noise.

### Lower-memory fallback

```bash
python -m pip install "boto3-stubs-lite[elbv2]"
```

The lite package omits the session `client(...)` and `resource(...)` overloads documented in the maintainer versioning guide, so explicit annotations are more important there.

## Authentication And Configuration

`mypy-boto3-elbv2` does not change AWS auth. boto3 still resolves credentials and region from the normal provider chain:

1. Explicit credentials, profile, or region passed to `Session(...)` or `client(...)`
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, `AWS_DEFAULT_REGION`, and `AWS_PROFILE`
3. Shared AWS config and credentials files
4. Assume-role, IAM Identity Center, ECS task roles, Lambda execution roles, EC2 instance profiles, and other AWS runtime providers

Typical local setup:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Then create the normal boto3 client and add the type annotation:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_elbv2.client import ElasticLoadBalancingv2Client

session = Session(profile_name="dev", region_name="us-west-2")
elbv2: ElasticLoadBalancingv2Client = session.client(
    "elbv2",
    config=Config(retries={"mode": "standard", "max_attempts": 10}),
)
```

If your application spans multiple accounts or regions, keep `Session(...)` creation explicit instead of relying on the global default session.

## Core Usage

### Typed ELBv2 client

```python
from boto3.session import Session
from mypy_boto3_elbv2.client import ElasticLoadBalancingv2Client

elbv2: ElasticLoadBalancingv2Client = Session(region_name="us-west-2").client("elbv2")

response = elbv2.describe_load_balancers(Names=["internal-api"])

for lb in response.get("LoadBalancers", []):
    print(lb["LoadBalancerArn"], lb["Type"], lb["State"]["Code"])
```

Use the service name `elbv2`. Classic Elastic Load Balancing uses the different boto3 service name `elb`.

### Typed paginator

Use paginators for list-style operations instead of hand-rolling marker loops:

```python
from boto3.session import Session
from mypy_boto3_elbv2.client import ElasticLoadBalancingv2Client
from mypy_boto3_elbv2.paginator import DescribeLoadBalancersPaginator

elbv2: ElasticLoadBalancingv2Client = Session(region_name="us-west-2").client("elbv2")
paginator: DescribeLoadBalancersPaginator = elbv2.get_paginator("describe_load_balancers")

for page in paginator.paginate(PaginationConfig={"PageSize": 20}):
    for lb in page.get("LoadBalancers", []):
        print(lb["LoadBalancerName"])
```

### Typed waiter

The generated waiters help you keep the waiter name and parameters correct:

```python
from mypy_boto3_elbv2.client import ElasticLoadBalancingv2Client
from mypy_boto3_elbv2.waiter import LoadBalancerAvailableWaiter

def wait_for_load_balancer(elbv2: ElasticLoadBalancingv2Client, lb_arn: str) -> None:
    waiter: LoadBalancerAvailableWaiter = elbv2.get_waiter("load_balancer_available")
    waiter.wait(
        LoadBalancerArns=[lb_arn],
        WaiterConfig={"Delay": 15, "MaxAttempts": 40},
    )
```

### Typed request dictionaries

Use generated request shapes when you build ELBv2 payloads outside the call site:

```python
from mypy_boto3_elbv2.client import ElasticLoadBalancingv2Client
from mypy_boto3_elbv2.type_defs import (
    MatcherTypeDef,
    RegisterTargetsInputTypeDef,
    TargetDescriptionTypeDef,
)

matcher: MatcherTypeDef = {"HttpCode": "200-399"}
targets: list[TargetDescriptionTypeDef] = [
    {"Id": "i-0123456789abcdef0", "Port": 8080},
]
request: RegisterTargetsInputTypeDef = {
    "TargetGroupArn": "arn:aws:elasticloadbalancing:us-west-2:123456789012:targetgroup/app-tg/abc123",
    "Targets": targets,
}

def register_targets(elbv2: ElasticLoadBalancingv2Client) -> None:
    elbv2.register_targets(**request)
    print(matcher["HttpCode"])
```

This is useful when request dictionaries move through helpers, background jobs, or config builders before the final boto3 call.

### `TYPE_CHECKING` pattern for dev-only stubs

If the stub package is installed only in development or CI, keep the import behind `TYPE_CHECKING` so production environments do not require it:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_elbv2.client import ElasticLoadBalancingv2Client

def get_elbv2_client() -> "ElasticLoadBalancingv2Client":
    return boto3.client("elbv2", region_name="us-west-2")
```

## Runtime Notes That Types Do Not Solve

- `describe_target_health` and similar calls still require the target group ARN, region, and IAM permissions to be correct at runtime.
- Load balancer creation and target registration are eventually consistent. A clean type check does not mean the load balancer or target group is ready yet.
- ALB, NLB, and Gateway Load Balancer behavior differs in protocol support, health checks, and listener actions. The stubs validate shapes, not service semantics.
- Retry mode, timeouts, endpoint selection, dual-stack, and FIPS settings still come from boto3 and botocore config, not from the stub package.

## Common Pitfalls

- Installing `mypy-boto3-elbv2` without `boto3`. The stubs do not make AWS calls.
- Importing the package name with hyphens. Install with hyphens, import with underscores: `mypy_boto3_elbv2`.
- Using the wrong boto3 service name. ALB and NLB APIs use `client("elbv2")`, not `client("elb")`.
- Expecting a high-level boto3 resource API for ELBv2. This service is effectively client-first in boto3.
- Using `boto3-stubs-lite[elbv2]` and expecting the same `Session.client("elbv2")` inference as the full bundle.
- Importing stub modules at runtime when they are installed only as dev dependencies. Use the `TYPE_CHECKING` pattern if production images omit the stubs.
- Reusing a custom `boto3.session.Session` across threads or processes. The boto3 session guide recommends a separate `Session` object per thread or process when you need explicit sessions.
- Assuming type safety proves runtime correctness. Region mismatches, IAM permissions, missing target groups, and target health state still fail at runtime.

## Version-Sensitive Notes

- On `2026-03-12`, the version used here `1.42.3` matched the live PyPI project page for `mypy-boto3-elbv2`.
- The live PyPI project page describes `1.42.3` as compatible with `boto3 1.42.3`, marks the package as `Typing :: Stubs Only`, and reports generation with `mypy-boto3-builder 8.12.0`.
- The maintainer docs site is rolling and can surface newer family-wide version strings than the pinned package release. Treat your installed package version and the PyPI release page as the source of truth for exact version matching.
- Repository branding has moved from `boto3-stubs` to `types-boto3`, but the published package name and Python import root remain `mypy-boto3-elbv2` and `mypy_boto3_elbv2`.

## Official Source URLs

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_elbv2/`
- Maintainer paginators reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_elbv2/paginators/`
- Maintainer type definitions reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_elbv2/type_defs/`
- Maintainer versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-elbv2/`
- AWS boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- AWS boto3 session guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/session.html`
- AWS ELBv2 boto3 reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/elbv2.html`
- Upstream repository: `https://github.com/youtype/types-boto3`

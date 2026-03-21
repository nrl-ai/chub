---
name: mypy-boto3-cloudfront
description: "mypy-boto3-cloudfront type stubs for boto3 CloudFront clients, paginators, waiters, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.40"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,cloudfront,mypy,pyright,type-stubs,python"
---

# mypy-boto3-cloudfront Python Package Guide

## What It Is

`mypy-boto3-cloudfront` is the generated type-stubs package for the CloudFront part of `boto3`.

Use it when you want:

- typed `Session.client("cloudfront")`
- typed paginator and waiter objects
- generated `Literal` unions for CloudFront string enums
- generated `TypedDict` request and response shapes

It does not make runtime AWS calls by itself. Install and use `boto3` for actual CloudFront operations.

## Install

### Recommended for most projects

```bash
python -m pip install boto3 'boto3-stubs[cloudfront]'
```

This is the upstream default when you want type inference for normal `boto3` code without adding explicit annotations everywhere.

### Lower-memory IDE fallback

```bash
python -m pip install boto3 'boto3-stubs-lite[cloudfront]'
```

Use the lite package if your IDE struggles with the full stubs. The official package notes that the lite variant is more RAM-friendly, but it does not provide `session.client()` or `session.resource()` overloads, so explicit annotations become important.

### Standalone CloudFront stubs

```bash
python -m pip install boto3 mypy-boto3-cloudfront
```

Use this when you want only the CloudFront stubs package and are comfortable importing concrete types from `mypy_boto3_cloudfront`.

### Local generation for exact boto3 parity

If your project pins a specific boto3 patch and exact type parity matters, upstream recommends local generation:

```bash
uvx --with 'boto3==1.42.40' mypy-boto3-builder
```

Then select `boto3-stubs` and the `CloudFront` service.

## Authentication And Setup

`mypy-boto3-cloudfront` has no package-specific auth or config. Credentials, region selection, retries, and endpoints all come from normal `boto3` configuration.

AWS says boto3 searches for credentials in a standard order that includes explicit client or session parameters, environment variables, shared credential files, config files, and runtime providers such as container or EC2 metadata.

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Explicit session setup is the clearest pattern for agents and reviewers:

```python
from boto3.session import Session
from mypy_boto3_cloudfront.client import CloudFrontClient

session = Session(profile_name="dev", region_name="us-east-1")
cloudfront: CloudFrontClient = session.client("cloudfront")
```

CloudFront is globally scoped, but keeping `region_name="us-east-1"` explicit avoids ambiguous environment-dependent behavior in local tooling and CI.

Do not hard-code access keys in source. Use profiles, environment variables, or workload credentials.

## Core Usage

### Typed client annotation

```python
from boto3.session import Session
from mypy_boto3_cloudfront.client import CloudFrontClient

cloudfront: CloudFrontClient = Session(region_name="us-east-1").client(
    "cloudfront"
)

response = cloudfront.get_distribution(Id="EDFDVBD6EXAMPLE")
status = response["Distribution"]["Status"]
print(status)
```

AWS documents clients as the low-level interface whose methods map closely to service APIs and support all service operations. For CloudFront code, prefer typed clients over generic `BaseClient`.

### Typed paginator usage

```python
from boto3.session import Session
from mypy_boto3_cloudfront.client import CloudFrontClient
from mypy_boto3_cloudfront.paginator import ListDistributionsPaginator

client: CloudFrontClient = Session(region_name="us-east-1").client("cloudfront")
paginator: ListDistributionsPaginator = client.get_paginator("list_distributions")

for page in paginator.paginate():
    for item in page.get("DistributionList", {}).get("Items", []):
        print(item["Id"], item["DomainName"])
```

The package also publishes paginator types for newer CloudFront operations such as connection functions, trust stores, and distribution tenants. Use the generated paginator class that matches the operation name instead of hand-rolled marker loops.

### Typed waiter usage

```python
from boto3.session import Session
from mypy_boto3_cloudfront.client import CloudFrontClient
from mypy_boto3_cloudfront.waiter import InvalidationCompletedWaiter

client: CloudFrontClient = Session(region_name="us-east-1").client("cloudfront")
waiter: InvalidationCompletedWaiter = client.get_waiter("invalidation_completed")

waiter.wait(
    DistributionId="EDFDVBD6EXAMPLE",
    Id="I1JLWSDAP8FU89",
)
```

The published waiters include `distribution_deployed`, `invalidation_completed`, `invalidation_for_distribution_tenant_completed`, and `streaming_distribution_deployed`.

### Literals and TypedDicts

```python
from mypy_boto3_cloudfront.literals import CachePolicyCookieBehaviorType
from mypy_boto3_cloudfront.type_defs import AliasICPRecordalTypeDef

cookie_behavior: CachePolicyCookieBehaviorType = "allExcept"

recordal: AliasICPRecordalTypeDef = {
    "CNAME": "cdn.example.com",
}
```

Use `literals` when an argument accepts a fixed string set, and use `type_defs` when you want stronger checking for request or response shapes.

## Type-Checking Patterns

### `TYPE_CHECKING` imports

If stubs are available only in development or CI, keep them out of runtime imports:

```python
from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_cloudfront.client import CloudFrontClient
else:
    CloudFrontClient = object

def make_client() -> "CloudFrontClient":
    return Session(region_name="us-east-1").client("cloudfront")
```

This is also the upstream workaround for `pylint` complaints about typing-only imports.

### Prefer clients over resources

AWS documents that resources are a higher-level interface, but the boto3 team does not plan to add new features to the resources interface. If you need newer CloudFront operations, type the client surface first.

## Common Pitfalls

- Installing only `mypy-boto3-cloudfront` and forgetting `boto3`. The stubs package is `Typing :: Stubs Only`.
- Expecting `boto3-stubs-lite[cloudfront]` to infer `Session().client("cloudfront")` automatically. Add explicit `CloudFrontClient` annotations in lite mode.
- Passing around a generic botocore client type and losing CloudFront-specific method signatures, paginator types, and waiter types.
- Treating `TypedDict` coverage as runtime validation. Boto3 responses are still normal dictionaries, and missing keys are still possible.
- Hard-coding credentials in code examples or app code. AWS explicitly recommends against that pattern.
- Assuming older blog posts cover the current CloudFront API surface. The generated stubs for `1.42.40` include recent paginator and waiter names that older examples may not mention.

## Version-Sensitive Notes

- This entry is pinned to version used here `1.42.40`.
- On 2026-03-12, the official PyPI page, release history, and maintainer docs all aligned on `mypy-boto3-cloudfront 1.42.40`, released on `2026-02-02`.
- The package description says its version matches the related `boto3` version. If your application is already on a later boto3 patch than `1.42.40`, verify the published stub version before pinning.
- The official boto3 credentials, clients, and resources guides currently resolve to newer boto3 docs (`1.42.66` on 2026-03-12). Use those pages for runtime behavior guidance, but keep the stub package version pinned separately.

## Official Sources

- Docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_cloudfront/
- Client docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_cloudfront/client/
- PyPI package page: https://pypi.org/project/mypy-boto3-cloudfront/
- Upstream repository: https://github.com/youtype/mypy_boto3_builder
- Boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- Boto3 clients guide: https://docs.aws.amazon.com/boto3/latest/guide/clients.html
- Boto3 resources guide: https://docs.aws.amazon.com/boto3/latest/guide/resources.html

---
name: mypy-boto3-iam
description: "Type stubs for boto3 IAM in Python, including typed clients, resources, paginators, waiters, literals, and TypedDict request/response shapes"
metadata:
  languages: "python"
  versions: "1.42.64"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,iam,boto3,python,typing,type-stubs,mypy,pyright"
---

# `mypy-boto3-iam` Python Package Guide

## Golden Rule

`mypy-boto3-iam` is a typing package, not a runtime AWS SDK. Keep using `boto3` to talk to AWS IAM, and add these stubs so `mypy`, `pyright`, and IDEs understand IAM clients, resources, paginators, waiters, literals, and TypedDict shapes.

If you install the lightweight or standalone package variants, annotate your clients and resources explicitly. If you want automatic type discovery for `Session.client("iam")` and `Session.resource("iam")`, install `boto3-stubs[iam]`.

## Install

Recommended for the best editor experience:

```bash
python -m pip install "boto3==1.42.64" "boto3-stubs[iam]==1.42.64"
```

Lower-memory option:

```bash
python -m pip install "boto3==1.42.64" "boto3-stubs-lite[iam]==1.42.64"
```

Service-specific standalone stubs:

```bash
python -m pip install "boto3==1.42.64" "mypy-boto3-iam==1.42.64"
```

Notes:

- `boto3-stubs[iam]` provides overloads so type checkers can infer `Session.client("iam")`, `Session.resource("iam")`, `client.get_paginator(...)`, and `client.get_waiter(...)` automatically.
- `boto3-stubs-lite[iam]` is more RAM-friendly, but upstream notes that it does not provide `session.client/resource` overloads, so explicit annotations are the safer default.
- `mypy-boto3-iam` gives you the IAM-specific type surface without installing the full `boto3-stubs` package. Keep `boto3` installed because the stubs do not make AWS calls themselves.

## Initialize And Authenticate

Authentication and configuration still come from `boto3` and the normal AWS credential chain. The stubs do not change request behavior, retries, endpoints, or permissions.

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_REGION=us-east-1
```

```python
from boto3.session import Session
from mypy_boto3_iam import IAMClient

session = Session(profile_name="dev", region_name="us-east-1")
iam: IAMClient = session.client("iam")
```

Use the same credential sources you would use for normal `boto3` code:

- shared config and credentials files
- environment variables
- SSO or assume-role configuration
- IAM roles on AWS compute

## Core Usage

### Typed IAM client

```python
from boto3.session import Session
from mypy_boto3_iam import IAMClient

iam: IAMClient = Session().client("iam")

response = iam.get_role(RoleName="MyApplicationRole")
print(response["Role"]["Arn"])
```

### Typed IAM resource

```python
from boto3.session import Session
from mypy_boto3_iam import IAMServiceResource

iam_resource: IAMServiceResource = Session().resource("iam")
user = iam_resource.User("alice")

print(user.arn)
```

### Typed paginators

The docs site uses a `/paginators/` URL, but the Python import module is singular: `mypy_boto3_iam.paginator`.

```python
from boto3.session import Session
from mypy_boto3_iam import IAMClient
from mypy_boto3_iam.paginator import ListUsersPaginator

iam: IAMClient = Session().client("iam")
paginator: ListUsersPaginator = iam.get_paginator("list_users")

for page in paginator.paginate():
    for user in page.get("Users", []):
        print(user["UserName"])
```

### Typed waiters

Likewise, the docs URL is `/waiters/`, but the import module is singular: `mypy_boto3_iam.waiter`.

```python
from boto3.session import Session
from mypy_boto3_iam import IAMClient
from mypy_boto3_iam.waiter import RoleExistsWaiter

iam: IAMClient = Session().client("iam")
waiter: RoleExistsWaiter = iam.get_waiter("role_exists")
waiter.wait(RoleName="MyApplicationRole")
```

### Typed request and response shapes

Use `type_defs` when you want explicit `TypedDict` shapes for request payloads or structured values:

```python
from mypy_boto3_iam.type_defs import AcceptDelegationRequestRequestTypeDef

def build_request(request_id: str) -> AcceptDelegationRequestRequestTypeDef:
    return {
        "DelegationRequestId": request_id,
    }
```

### Literals for constrained string values

```python
from mypy_boto3_iam.literals import AccessAdvisorUsageGranularityTypeType

def normalize_granularity(value: AccessAdvisorUsageGranularityTypeType) -> str:
    return value
```

### Keep stubs out of production-only dependency sets

If your deployment image should not carry stub packages, use `TYPE_CHECKING` imports and keep the stubs in a dev or type-checking environment:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_iam import IAMClient
else:
    IAMClient = object

iam = Session().client("iam")
typed_iam: "IAMClient" = iam
```

This pattern is especially useful with `pylint`, where upstream notes that undefined-name complaints can appear without a non-`TYPE_CHECKING` fallback.

## Common Pitfalls

- `mypy-boto3-iam` is not `boto3`. Install `boto3` for runtime use.
- The import package uses underscores: `mypy_boto3_iam`, not `mypy-boto3-iam`.
- Align versions. Upstream states that `mypy-boto3-iam` follows the related `boto3` version, so pin both packages to the same release family.
- Do not confuse docs URLs with import module names. The site uses `/paginators/` and `/waiters/`, but the modules are `paginator` and `waiter`.
- If `Session().client("iam")` is inferred as `BaseClient` instead of `IAMClient`, you probably installed the standalone or lite package and need explicit annotations.
- PyCharm can be slow with large `Literal` overload sets. Upstream recommends `boto3-stubs-lite` if IDE performance becomes a problem.
- These stubs describe the generated boto3 IAM surface. They do not validate that your AWS credentials, IAM permissions, or account state are correct at runtime.

## Version-Sensitive Notes

- PyPI lists `mypy-boto3-iam 1.42.64` as the latest release on March 12, 2026, released March 9, 2026.
- This release was generated with `mypy-boto3-builder 8.12.0`.
- The package is marked `Stubs Only` on PyPI and supports Python `3.9` through `3.14`.
- Upstream documents typed `Client`, `ServiceResource`, resource collections, `Paginator`, `Waiter`, `Literal`, and `TypeDef` surfaces for this service.
- If your project upgrades `boto3`, upgrade the IAM stubs at the same time to avoid drift between generated signatures and the installed runtime SDK.

## Official Sources

- PyPI package page: `https://pypi.org/project/mypy-boto3-iam/`
- Service docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_iam/`
- Client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_iam/client/`
- Paginators reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_iam/paginators/`
- Maintainer repository: `https://github.com/vemel/mypy_boto3_builder`

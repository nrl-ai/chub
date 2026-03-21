---
name: mypy-boto3-organizations
description: "mypy-boto3-organizations type stubs for boto3 Organizations clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.41"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,organizations,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-organizations Python Package Guide

## What It Is

`mypy-boto3-organizations` is a PEP 561 stub-only package for the AWS Organizations client in `boto3`.

Use it when you already call AWS Organizations through `boto3`, but want static typing for:

- `OrganizationsClient`
- generated paginator classes
- generated `TypedDict` request and response shapes
- generated string `Literal` values

It does not replace `boto3`, does not make AWS API calls by itself, and does not manage credentials, regions, retries, or endpoints. Runtime behavior still comes from `boto3` and `botocore`.

## Version Covered

- Ecosystem: `pypi`
- Package: `mypy-boto3-organizations`
- Import family: `mypy_boto3_organizations`
- Service name for boto3: `"organizations"`
- Version used here for this entry: `1.42.41`
- Python requirement on the official PyPI page checked on 2026-03-12: `>=3.9`
- Registry URL: `https://pypi.org/project/mypy-boto3-organizations/`
- Docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_organizations/`

## Install

Install the runtime SDK and the service stubs together:

```bash
python -m pip install "boto3==1.42.41" "mypy-boto3-organizations==1.42.41"
```

If you want the broader boto3 overloads upstream documents for `session.client(...)`, use the extra-based package instead:

```bash
python -m pip install "boto3-stubs[organizations]==1.42.41"
```

If editor or memory usage is a concern, upstream also documents a lite variant:

```bash
python -m pip install "boto3-stubs-lite[organizations]==1.42.41"
```

Practical rule: keep `boto3`, `botocore`, and `mypy-boto3-organizations` on the same patch line when possible.

## Auth And Configuration

`mypy-boto3-organizations` has no package-specific configuration. Use the normal boto3 credential and configuration chain.

Common setup for local development:

```bash
aws configure --profile org-admin
export AWS_PROFILE=org-admin
export AWS_DEFAULT_REGION=us-east-1
```

Then create the client with normal boto3 session handling:

```python
from boto3.session import Session

session = Session(profile_name="org-admin", region_name="us-east-1")
org = session.client("organizations")
```

If credentials, region, retry mode, or endpoint selection are wrong, the stub package will not fix runtime failures. Treat those as standard boto3 concerns.

## Core Setup

Annotate the client explicitly and let `boto3` keep creating the real runtime client.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_organizations.client import OrganizationsClient

session = Session(profile_name="org-admin", region_name="us-east-1")
org: OrganizationsClient = session.client("organizations")

response = org.describe_organization()
print(response["Organization"]["Id"])
```

This pattern is usually the most reliable with the standalone `mypy-boto3-organizations` package because it does not rely on inferred `session.client(...)` overloads.

## Core Typing Patterns

### Typed client

Use `OrganizationsClient` when you want operation names, request keys, and response keys checked by your type checker.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_organizations.client import OrganizationsClient

org: OrganizationsClient = Session(region_name="us-east-1").client("organizations")

organization = org.describe_organization()["Organization"]
print(organization["Arn"])
```

### Typed paginators

AWS Organizations exposes paginator operations in boto3, and the stub package includes generated paginator classes for them.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_organizations.client import OrganizationsClient
    from mypy_boto3_organizations.paginator import ListAccountsPaginator

org: OrganizationsClient = Session(region_name="us-east-1").client("organizations")
paginator: ListAccountsPaginator = org.get_paginator("list_accounts")

for page in paginator.paginate():
    for account in page.get("Accounts", []):
        print(account["Id"], account["Email"])
```

The official boto3 Organizations paginator list currently includes operations such as `list_accounts`, `list_accounts_for_parent`, `list_children`, `list_organizational_units_for_parent`, `list_parents`, `list_policies`, and `list_roots`.

### Generated literals and TypedDict shapes

Use `literals` when a field or parameter is constrained to a fixed string set, and `type_defs` when you want `TypedDict` coverage for helper boundaries.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_organizations.client import OrganizationsClient
    from mypy_boto3_organizations.literals import PolicyTypeType
    from mypy_boto3_organizations.type_defs import DescribePolicyResponseTypeDef

policy_type: PolicyTypeType = "SERVICE_CONTROL_POLICY"
org: OrganizationsClient = Session(region_name="us-east-1").client("organizations")
policy: DescribePolicyResponseTypeDef = org.describe_policy(PolicyId="p-12345678")

print(policy_type, policy["Policy"]["PolicySummary"]["Id"])
```

The generated module families documented upstream are:

- `mypy_boto3_organizations.client`
- `mypy_boto3_organizations.paginator`
- `mypy_boto3_organizations.type_defs`
- `mypy_boto3_organizations.literals`

## Tooling Patterns

### `TYPE_CHECKING` imports

If the stub package is only installed in dev or CI, keep stub-only imports behind `TYPE_CHECKING`:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_organizations.client import OrganizationsClient

def make_client() -> "OrganizationsClient":
    return Session(region_name="us-east-1").client("organizations")
```

This avoids turning a typing helper into a hard runtime dependency.

### When to use `boto3-stubs[organizations]` instead

Choose `mypy-boto3-organizations` when you want the smallest service-specific stub package and you are comfortable with explicit annotations.

Choose `boto3-stubs[organizations]` when you want:

- better automatic typing for `boto3.client("organizations")` and `Session.client("organizations")`
- a single dependency pattern across multiple AWS services
- the broader setup path shown in the upstream README

## Common Pitfalls

- Installing only `mypy-boto3-organizations` and expecting it to work as a runtime AWS SDK. You still need `boto3`.
- Using the package name as the boto3 service name. The service is `organizations`, not `mypy-boto3-organizations`.
- Expecting automatic `session.client(...)` overload inference from the standalone package. That is what `boto3-stubs[organizations]` is for.
- Forgetting that import names use underscores. Install `mypy-boto3-organizations`, but import from `mypy_boto3_organizations`.
- Assuming every AWS service has resources and waiters. The official boto3 Organizations reference is client-and-paginator focused; do not invent `OrganizationsServiceResource` or waiter imports.
- Letting `boto3` and the stub package drift too far apart. Newer stubs can type-check calls your installed runtime does not support, and older stubs can miss newer parameters.

## Version-Sensitive Notes

- The version used here for this session is `1.42.41`.
- On 2026-03-12, the official PyPI project page showed `1.42.42` as the latest published release for `mypy-boto3-organizations`.
- On 2026-03-12, the official generated docs search result for `mypy_boto3_organizations` surfaced documentation pages on the `1.42.64` line. Treat that as upstream docs drift relative to the pinned package version.
- Upstream states that the package version follows the related `boto3` version. Pin `boto3` and `mypy-boto3-organizations` together when exact model parity matters.
- If you need exact stubs for a different boto3 patch line than the published package provides, prefer checking the generated docs and the broader `boto3-stubs` toolchain before copying older examples.

## Official Sources

- PyPI package page: https://pypi.org/project/mypy-boto3-organizations/
- Generated docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_organizations/
- AWS Organizations boto3 client reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/organizations.html
- AWS Organizations paginators reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/organizations.html#paginators
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- Boto3 configuration guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/configuration.html

---
name: mypy-boto3-appstream
description: "Type annotations for boto3 AppStream in Python, covering typed clients, paginators, waiters, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.54"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,appstream,boto3,type-stubs,mypy,pyright,autocomplete"
---

# mypy-boto3-appstream Python Package Guide

## Golden Rule

`mypy-boto3-appstream` is a stubs-only package for static typing. It does not make AWS calls by itself and it does not replace `boto3`.

Use one of these workflows:

- Install `boto3-stubs[appstream]` when you want normal `Session().client("appstream")` code to infer types automatically.
- Install `mypy-boto3-appstream` when you want only the AppStream typing package and are willing to annotate the client, paginators, waiters, literals, and `TypedDict` shapes explicitly.
- Install `boto3-stubs-lite[appstream]` if full overloads are too heavy for PyCharm or low-memory environments. The lite package drops automatic `session.client()` overloads, so explicit annotations matter more.

The maintainer docs list client, paginator, waiter, literal, and `type_defs` modules, but no service-resource surface, so treat AppStream typing here as client-only.

## Install

Recommended when you want typed boto3 code with automatic client inference:

```bash
python -m pip install "boto3==1.42.54" "boto3-stubs[appstream]==1.42.54"
```

Standalone service-specific stubs:

```bash
python -m pip install "boto3==1.42.54" "mypy-boto3-appstream==1.42.54"
```

Lower-memory alternative:

```bash
python -m pip install "boto3==1.42.54" "boto3-stubs-lite[appstream]==1.42.54"
```

Common alternatives:

```bash
uv add "boto3==1.42.54" "boto3-stubs[appstream]==1.42.54"
poetry add "boto3==1.42.54" "boto3-stubs[appstream]==1.42.54"
```

Notes:

- Keep the stub version aligned with the boto3 version you pin. The maintainer docs say package versions match the related boto3 version.
- `mypy-boto3-appstream` does not ship the runtime client. If you install only the stubs package, type checking can pass while runtime imports fail.

## Runtime Setup And Authentication

Authentication and region handling still come from boto3. AWS documents that boto3 checks explicit client or session parameters, environment variables, shared credentials files, config files, container credentials, and instance metadata as part of its credential provider chain.

Typical local setup:

```bash
aws configure
export AWS_DEFAULT_REGION="us-west-2"
```

Or use environment variables directly:

```bash
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_SESSION_TOKEN="..."  # only for temporary credentials
export AWS_DEFAULT_REGION="us-west-2"
```

Minimal typed setup:

```python
from boto3.session import Session
from mypy_boto3_appstream import AppStreamClient

session = Session(region_name="us-west-2")
client: AppStreamClient = session.client("appstream")
```

AWS now brands the service docs as WorkSpaces Applications in parts of the boto3 reference, but the Python service identifier is still `appstream` and the stub package name is still `mypy-boto3-appstream`.

## Core Usage

### Zero-annotation workflow with `boto3-stubs[appstream]`

With the full extra installed, ordinary boto3 client code should infer types:

```python
from boto3.session import Session

session = Session(region_name="us-west-2")
client = session.client("appstream")

response = client.describe_stacks()
for stack in response["Stacks"]:
    print(stack["Name"])
```

### Explicit client annotations

Use this pattern when you installed `mypy-boto3-appstream` or `boto3-stubs-lite[appstream]`:

```python
from boto3.session import Session
from mypy_boto3_appstream import AppStreamClient

def get_client() -> AppStreamClient:
    return Session(region_name="us-west-2").client("appstream")

client = get_client()
result = client.describe_fleets()
for fleet in result["Fleets"]:
    print(fleet["Name"], fleet["State"])
```

### Typed paginator

```python
from boto3.session import Session
from mypy_boto3_appstream import AppStreamClient
from mypy_boto3_appstream.paginator import DescribeStacksPaginator

client: AppStreamClient = Session(region_name="us-west-2").client("appstream")
paginator: DescribeStacksPaginator = client.get_paginator("describe_stacks")

for page in paginator.paginate():
    for stack in page["Stacks"]:
        print(stack["Name"])
```

### Typed waiter

```python
from boto3.session import Session
from mypy_boto3_appstream import AppStreamClient
from mypy_boto3_appstream.waiter import FleetStartedWaiter

client: AppStreamClient = Session(region_name="us-west-2").client("appstream")
waiter: FleetStartedWaiter = client.get_waiter("fleet_started")
waiter.wait(Name="example-fleet")
```

### Literals and `TypedDict` request and response shapes

Use literals for constrained enum-like values and `type_defs` for helper boundaries:

```python
from boto3.session import Session
from mypy_boto3_appstream import AppStreamClient
from mypy_boto3_appstream.literals import AuthenticationTypeType
from mypy_boto3_appstream.type_defs import (
    DescribeUsersRequestTypeDef,
    DescribeUsersResultTypeDef,
)

client: AppStreamClient = Session(region_name="us-west-2").client("appstream")

auth_type: AuthenticationTypeType = "USERPOOL"
request: DescribeUsersRequestTypeDef = {"AuthenticationType": auth_type}
result: DescribeUsersResultTypeDef = client.describe_users(**request)

print(len(result["Users"]))
```

This pattern is useful when you pass request objects through helper functions and want mypy or pyright to catch wrong keys or wrong literal values before runtime.

### `TYPE_CHECKING` guard for dev-only stubs

If you keep stubs out of production images, import them only for type checking:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_appstream import AppStreamClient
else:
    AppStreamClient = object

client: "AppStreamClient" = Session(region_name="us-west-2").client("appstream")
```

The maintainer docs explicitly call this pattern safe and note that the fallback also avoids a common pylint undefined-name complaint.

## Common Pitfalls

- Do not treat `mypy-boto3-appstream` as the runtime SDK. Real AWS calls still require `boto3`.
- Do not expect a boto3 resource surface here. The published AppStream stubs expose a typed client, paginators, waiters, literals, and `type_defs`, but not a `resource("appstream")` service-resource layer.
- Do not mix unrelated boto3 and stubs versions casually. Generated names and availability track the boto3 service model version closely.
- Do not rely on automatic `session.client("appstream")` inference when you installed the standalone package or the lite package. Add explicit `AppStreamClient` annotations in those setups.
- Do not hardcode AWS credentials in application code. Keep auth in profiles, environment variables, IAM Identity Center, or runtime IAM roles.
- Do not forget the region. Even typed code still fails at runtime if credentials or region resolution are wrong.
- AppStream user-management APIs are stricter than many examples imply. AWS documents that user email addresses for operations like `enable_user` and `disable_user` are case-sensitive.
- The service branding is inconsistent across sources. AWS may say WorkSpaces Applications while boto3, the API namespace, and this package still use `appstream`.

## Version-Sensitive Notes For `1.42.54`

- PyPI lists `1.42.54` as the current `mypy-boto3-appstream` release on March 12, 2026, published on February 20, 2026.
- The package description says it provides type annotations for `boto3 AppStream 1.42.54`, and the maintainer docs say package versions are the same as the related boto3 version.
- The rolling AWS boto3 reference for AppStream is already on a newer patch line than this standalone stub release. When exact generated names matter, prefer the version you pinned from PyPI over whatever patch number appears in the live boto3 docs tree.
- If you upgrade `boto3`, upgrade the AppStream stubs in the same change. That keeps client methods, paginator names, waiter names, literals, and `TypedDict` shapes aligned.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_appstream/
- PyPI package page: https://pypi.org/project/mypy-boto3-appstream/
- AWS boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- AWS boto3 AppStream reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/appstream.html

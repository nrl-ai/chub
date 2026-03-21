---
name: mypy-boto3-workspaces
description: "Type stubs for boto3 WorkSpaces client, paginators, literals, and TypedDicts in Python"
metadata:
  languages: "python"
  versions: "1.42.66"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,workspaces,type-stubs,mypy,pyright,python"
---

# mypy-boto3-workspaces Python Package Guide

## Golden Rule

`mypy-boto3-workspaces` is a stubs-only package for static typing. It does not replace `boto3`, it does not handle AWS auth for you, and it should usually track the same version as your installed `boto3`. Use it to type `boto3.client("workspaces")`, paginator calls, literals, and request/response TypedDict shapes.

## Install

For direct use with this package:

```bash
python -m pip install "boto3==1.42.66" "mypy-boto3-workspaces==1.42.66"
```

If you want the broader boto3 stubs distribution instead of the standalone service package:

```bash
python -m pip install "boto3-stubs[workspaces]==1.42.66"
```

If you need a lower-memory option and can tolerate less inference:

```bash
python -m pip install "boto3-stubs-lite[workspaces]==1.42.66"
```

Important install behavior:

- `boto3-stubs[workspaces]` gives you boto3 typing plus the WorkSpaces service extras.
- `boto3-stubs-lite[workspaces]` does not provide `Session().client("workspaces")` overload inference, so explicit annotations become more important.
- The standalone package is useful when you want only the WorkSpaces stubs in a typing or dev dependency set.

## Authentication And Setup

This package uses normal boto3 configuration. Configure AWS credentials and region exactly as you would for any boto3 client.

Common boto3 credential sources:

1. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
2. Shared credentials and config files under `~/.aws/credentials` and `~/.aws/config`
3. A named profile via `AWS_PROFILE` or `boto3.Session(profile_name=...)`
4. IAM role or workload credentials on AWS infrastructure

Typed client setup:

```python
from boto3.session import Session
from mypy_boto3_workspaces.client import WorkSpacesClient

session = Session(profile_name="dev", region_name="us-west-2")
client: WorkSpacesClient = session.client("workspaces")
```

If you prefer the top-level import:

```python
from boto3.session import Session
from mypy_boto3_workspaces import WorkSpacesClient

client: WorkSpacesClient = Session(region_name="us-west-2").client("workspaces")
```

## Core Usage

### Typed client calls

Use request TypedDicts when you want stricter validation of request shapes before calling boto3:

```python
from boto3.session import Session
from mypy_boto3_workspaces.client import WorkSpacesClient
from mypy_boto3_workspaces.type_defs import DescribeWorkspacesRequestTypeDef

session = Session(profile_name="dev", region_name="us-west-2")
client: WorkSpacesClient = session.client("workspaces")

request: DescribeWorkspacesRequestTypeDef = {
    "DirectoryId": "d-90670a1b1d",
    "UserName": "alice@example.com",
}

response = client.describe_workspaces(**request)

for workspace in response.get("Workspaces", []):
    print(workspace["WorkspaceId"], workspace.get("State"))
```

`describe_workspaces` is a common starting point, but its filters are constrained by the AWS API:

- `WorkspaceIds` cannot be combined with other filters.
- `DirectoryId` cannot be combined with other filters except its paired `UserName`.
- `BundleId` cannot be combined with other filters.

### Typed paginators

The generated docs expose paginator types for WorkSpaces operations such as `describe_workspaces` and `describe_workspace_directories`.

```python
from boto3.session import Session
from mypy_boto3_workspaces.client import WorkSpacesClient
from mypy_boto3_workspaces.paginator import DescribeWorkspacesPaginator

client: WorkSpacesClient = Session(region_name="us-west-2").client("workspaces")

paginator: DescribeWorkspacesPaginator = client.get_paginator("describe_workspaces")

for page in paginator.paginate(
    DirectoryId="d-90670a1b1d",
    PaginationConfig={"PageSize": 25},
):
    for workspace in page.get("Workspaces", []):
        print(workspace["WorkspaceId"])
```

The maintainer docs list these WorkSpaces paginator classes in `1.42.66`:

- `DescribeAccountModificationsPaginator`
- `DescribeIpGroupsPaginator`
- `DescribeWorkspaceBundlesPaginator`
- `DescribeWorkspaceDirectoriesPaginator`
- `DescribeWorkspaceImagesPaginator`
- `DescribeWorkspacesConnectionStatusPaginator`
- `DescribeWorkspacesPaginator`
- `ListAccountLinksPaginator`
- `ListAvailableManagementCidrRangesPaginator`

### Literals and TypedDicts

Use generated literals to avoid stringly typed state and enum values:

```python
from mypy_boto3_workspaces.literals import WorkspaceStateType

def is_terminal(state: WorkspaceStateType) -> bool:
    return state in {"STOPPED", "TERMINATED", "ERROR"}
```

Use generated TypedDicts to make request builders and helpers safer:

```python
from mypy_boto3_workspaces.type_defs import RebootRequestTypeDef

payload: RebootRequestTypeDef = {
    "RebootWorkspaceRequests": [
        {"WorkspaceId": "ws-1234567890"},
    ],
}
```

## Tooling Patterns

### `TYPE_CHECKING` guard for dev-only stub installs

If stubs are installed only in dev or CI environments, guard imports so production runtime does not require the package:

```python
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from mypy_boto3_workspaces.client import WorkSpacesClient
else:
    WorkSpacesClient = object
```

Then annotate normally:

```python
from boto3.session import Session

client: WorkSpacesClient = Session(region_name="us-west-2").client("workspaces")
```

This pattern is especially useful when `pylint` or runtime-only images do not include stub packages.

## Common Pitfalls

- Do not treat `mypy-boto3-workspaces` as the runtime SDK. You still need `boto3` installed and configured.
- Keep the stub package version aligned with `boto3`. The project explicitly versions service stubs to match the corresponding boto3 release.
- `boto3-stubs-lite[workspaces]` does not provide `session.client/resource` overload inference. If you use lite, annotate the client explicitly.
- The current generated WorkSpaces docs expose a typed client, paginators, literals, and TypedDicts, but no documented `WorkSpacesServiceResource` or waiter module. Do not assume `Session().resource("workspaces")` or `client.get_waiter(...)` has matching generated types for this service.
- `describe_workspaces` filters are mutually exclusive in the ways documented by boto3. Agents often incorrectly send `WorkspaceIds` together with `DirectoryId` or `BundleId`.
- Credentials and region issues come from boto3 configuration, not this stub package. If client creation or requests fail, debug AWS auth and region selection first.

## Version-Sensitive Notes

- `1.42.66` was released on March 11, 2026 and the PyPI page says it was generated with `mypy-boto3-builder 8.12.0`.
- PyPI marks this package as `Stubs Only` and requires Python `>=3.9`.
- PyPI project links for this package point to `youtype/mypy_boto3_builder`. Separately, the older `youtype/boto3-stubs` repository page now points to `youtype/types-boto3`. That only matters if you are migrating package families; this package is still actively published and documented for the `mypy_boto3_workspaces` import path.
- If you generate stubs locally, the maintainer docs recommend `uvx --with 'boto3==1.42.66' mypy-boto3-builder`, then selecting the `WorkSpaces` service.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_workspaces/`
- PyPI project page: `https://pypi.org/project/mypy-boto3-workspaces/`
- Boto3 WorkSpaces reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/workspaces.html`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`

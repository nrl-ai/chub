---
name: mypy-boto3-connect
description: "Type stubs for boto3 Amazon Connect clients, paginators, literals, and TypedDict shapes in Python"
metadata:
  languages: "python"
  versions: "1.42.63"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,connect,boto3,python,typing,mypy,pyright,stubs"
---

# mypy-boto3-connect Python Package Guide

## Golden Rule

`mypy-boto3-connect` is a typing package for `boto3` Amazon Connect code, not a runtime SDK. Keep `boto3` installed, authenticate exactly as you would for normal AWS code, and use the stub package to type your Connect client and paginator usage so `mypy` or `pyright` can catch operation-name and shape mistakes before runtime.

Use one of these install patterns:

- `boto3-stubs[connect]` for the best automatic typing on `Session().client("connect")`
- `mypy-boto3-connect` when you want the service-specific package and explicit annotations
- `boto3-stubs-lite[connect]` when IDE performance matters more than automatic overloads

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.42.63`, which matches the current PyPI release page on March 12, 2026.
- The maintainer docs are a rolling latest site. Use PyPI for exact release pinning and use the docs site for package structure, import names, and install-mode guidance.
- The maintainer versioning guide says stub package versions follow the related `boto3` version, so pin `boto3`, `botocore`, and the stub package to the same release line when exact generated signatures matter.
- `boto3-stubs-lite` does not provide the overloaded `Session.client()` helper types. In lite mode, annotate the Connect client explicitly after creation.

## Install

### Recommended: full boto3 overloads

This is the simplest setup when your code constructs Connect clients directly from `boto3.Session`.

```bash
python -m pip install "boto3==1.42.63" "boto3-stubs[connect]==1.42.63"
```

### Service-specific stub package

Use this when you want the Connect-only package and do not mind explicit annotations.

```bash
python -m pip install "boto3==1.42.63" "mypy-boto3-connect==1.42.63"
```

### Lite variant

Use this when the full overload set is too heavy for your editor or type checker.

```bash
python -m pip install "boto3==1.42.63" "boto3-stubs-lite[connect]==1.42.63"
```

Common alternatives:

```bash
uv add "boto3==1.42.63" "mypy-boto3-connect==1.42.63"
poetry add "boto3==1.42.63" "mypy-boto3-connect==1.42.63"
```

## Authentication And Setup

This package does not change how AWS authentication or client configuration works. Use the normal boto3 credential chain:

1. Shared AWS config and credentials files plus `AWS_PROFILE` for local development
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
3. IAM roles, IAM Identity Center, or other runtime AWS credential providers in deployed environments

Typical local setup:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Then create a normal typed client:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_connect.client import ConnectClient

session = Session(profile_name="dev", region_name="us-east-1")

config = Config(
    retries={"mode": "standard", "max_attempts": 10},
)

client: ConnectClient = session.client("connect", config=config)
```

Amazon Connect resources are regional and many operations require an `InstanceId`, so keep the client region aligned with the Connect instance region you are targeting.

## Core Usage

### Type the Connect client explicitly

This is the most reliable pattern when you install the service-specific stub package:

```python
from boto3.session import Session
from mypy_boto3_connect.client import ConnectClient

session = Session(region_name="us-east-1")
client: ConnectClient = session.client("connect")

response = client.describe_user(
    InstanceId="your-connect-instance-id",
    UserId="your-user-id",
)

print(response["User"]["Username"])
```

### Type paginators for list operations

The generated package includes paginator classes for Connect list APIs such as `list_users`:

```python
from boto3.session import Session
from mypy_boto3_connect.client import ConnectClient
from mypy_boto3_connect.paginator import ListUsersPaginator

client: ConnectClient = Session(region_name="us-east-1").client("connect")
paginator: ListUsersPaginator = client.get_paginator("list_users")

for page in paginator.paginate(InstanceId="your-connect-instance-id"):
    for user in page.get("UserSummaryList", []):
        print(user["Username"])
```

### Use `TYPE_CHECKING` when stubs are dev-only

If production environments omit typing dependencies, keep the stub import out of runtime execution:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_connect.client import ConnectClient

def get_connect_client() -> "ConnectClient":
    return boto3.client("connect", region_name="us-east-1")
```

### Use generated `type_defs` and literals in helper layers

The package also exposes generated `TypedDict` request and response shapes plus `Literal` aliases. Use them when you wrap Connect calls behind helper functions, build request dictionaries incrementally, or want enum-like string parameters checked statically instead of treated as plain `str`.

## Configuration Notes

- Set `region_name` deliberately. Connect instances and many related resources are regional, and a mismatched region often looks like a not-found or access issue.
- Prefer `boto3.Session(...)` when you need predictable profile or region selection in local tooling.
- Use `botocore.config.Config` for retries, timeouts, proxies, and endpoint options. This stub package does not replace `botocore` runtime configuration.
- Keep `boto3`, `botocore`, and the stub package on the same version line in your lockfile to avoid stale method signatures or missing generated types.
- Treat the generated docs as current-package structure, but validate exact runtime behavior against AWS Connect API docs when request/response details matter.

## Common Pitfalls

- Installing `mypy-boto3-connect` without `boto3`. The stub package does not make AWS API calls on its own.
- Using `"mypy-boto3-connect"` as the boto3 service name. The runtime service name is still `"connect"`.
- Expecting `boto3-stubs-lite[connect]` to infer `Session.client("connect")` overloads automatically.
- Importing stub modules at runtime in environments where only production dependencies are installed. Use `TYPE_CHECKING` or install the package wherever those imports execute.
- Forgetting the required `InstanceId` parameter on many Connect operations.
- Pointing the client at the wrong AWS region for the target Connect instance.
- Copying examples from generic boto3 guides without checking the generated Connect stub names for the version you actually installed.

## Official Source URLs Used

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_connect/`
- Maintainer versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI release: `https://pypi.org/project/mypy-boto3-connect/1.42.63/`
- PyPI project page: `https://pypi.org/project/mypy-boto3-connect/`
- Umbrella package release: `https://pypi.org/project/boto3-stubs/1.42.63/`
- Boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- Boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
- Boto3 Connect client reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/connect.html`
- Boto3 Connect paginator reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/connect/paginator/ListUsers.html`

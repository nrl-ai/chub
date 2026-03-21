---
name: mypy-boto3-cloudformation
description: "mypy-boto3-cloudformation typed stubs for boto3 CloudFormation clients, resources, paginators, waiters, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,boto3,cloudformation,mypy,pyright,pylance,types,stubs"
---

# mypy-boto3-cloudformation Python Package Guide

## What It Is

`mypy-boto3-cloudformation` provides generated type stubs for the CloudFormation parts of `boto3`.

Use it when you want:

- typed `Session.client("cloudformation")` or explicit `CloudFormationClient` annotations
- typed `Session.resource("cloudformation")` resource access
- typed paginator and waiter objects
- CloudFormation literal unions and `TypedDict` request/response shapes
- better autocomplete and static checking in mypy, pyright, Pylance, and PyCharm

It does not make AWS calls by itself. Runtime behavior still comes from `boto3`.

## Install

Preferred install modes:

```bash
python -m pip install "boto3-stubs[cloudformation]"
```

Use this when you want the best editor and type-checker experience. The upstream docs show this as the standard install path for typed service support.

If the full stubs package is too heavy for your IDE, use the lite variant:

```bash
python -m pip install "boto3-stubs-lite[cloudformation]"
```

If you want only the standalone CloudFormation stubs package:

```bash
python -m pip install mypy-boto3-cloudformation
```

You still need the runtime SDK:

```bash
python -m pip install boto3
```

Practical rule:

- use `boto3-stubs[cloudformation]` for automatic `Session.client(...)` and `Session.resource(...)` typing
- use `boto3-stubs-lite[cloudformation]` if IDE performance matters more than overload convenience
- use `mypy-boto3-cloudformation` when you want explicit CloudFormation type imports without the full bundle

## Setup And AWS Auth

This package does not handle credentials. `boto3` still uses the normal AWS credential chain.

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Or create the session explicitly:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
```

Credential resolution still comes from:

- explicit credentials passed to `Session(...)`
- environment variables
- `~/.aws/credentials` and `~/.aws/config`
- IAM roles or workload identity in AWS environments

## Core Usage

### Typed CloudFormation Client

Use the standalone type when you want an explicit annotation:

```python
from boto3.session import Session
from mypy_boto3_cloudformation import CloudFormationClient

session = Session(region_name="us-east-1")
cloudformation: CloudFormationClient = session.client("cloudformation")

response = cloudformation.describe_stacks(StackName="example-stack")
for stack in response.get("Stacks", []):
    print(stack["StackName"], stack["StackStatus"])
```

### Typed CloudFormation Resource

```python
from boto3.session import Session
from mypy_boto3_cloudformation import CloudFormationServiceResource

session = Session(region_name="us-east-1")
cf: CloudFormationServiceResource = session.resource("cloudformation")

for stack in cf.stacks.all():
    print(stack.stack_name)
```

### Typed Paginators And Waiters

```python
from boto3.session import Session
from mypy_boto3_cloudformation.paginator import ListStacksPaginator
from mypy_boto3_cloudformation.waiter import StackCreateCompleteWaiter

session = Session(region_name="us-east-1")
cloudformation = session.client("cloudformation")

paginator: ListStacksPaginator = cloudformation.get_paginator("list_stacks")
for page in paginator.paginate(
    StackStatusFilter=["CREATE_COMPLETE", "UPDATE_COMPLETE"],
):
    for summary in page.get("StackSummaries", []):
        print(summary["StackName"])

waiter: StackCreateCompleteWaiter = cloudformation.get_waiter(
    "stack_create_complete"
)
waiter.wait(StackName="example-stack")
```

### Literals And TypedDicts

Use the generated literal aliases and type definitions in helper code:

```python
from mypy_boto3_cloudformation.literals import AccountFilterTypeType
from mypy_boto3_cloudformation.type_defs import AccountGateResultTypeDef

def summarize_gate(
    filter_type: AccountFilterTypeType,
    result: AccountGateResultTypeDef,
) -> str:
    return f"{filter_type}: {result.get('Status')}"
```

### `TYPE_CHECKING` Pattern

This keeps explicit stub imports out of runtime-only paths when you prefer string annotations:

```python
from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_cloudformation import CloudFormationClient

session = Session(region_name="us-east-1")
cloudformation: "CloudFormationClient" = session.client("cloudformation")
```

## Common Pitfalls

- These are typing stubs, not the AWS SDK itself. Install and use `boto3` for runtime calls.
- The service name must be `"cloudformation"`. Typos here break both runtime creation and type inference.
- `boto3-stubs-lite[cloudformation]` does not provide the same `Session.client(...)` and `Session.resource(...)` overload convenience as the full `boto3-stubs` extra. Use explicit annotations with the lite package.
- Responses are still plain Python dictionaries at runtime. The stubs improve editor and checker accuracy, but they do not validate live AWS responses.
- CloudFormation unions and `TypedDict` shapes can be large. If your IDE slows down, switch from the full bundle to the lite package or annotate only the surfaces you actually use.
- Pylint can complain about generated imports or aliases. The PyPI page documents a `TYPE_CHECKING` pattern specifically for this case.

## Version-Sensitive Notes

- This entry is pinned to the version used here and current PyPI release: `1.42.3`.
- The PyPI page states that the package version should match the related `boto3` version.
- The package docs site is a generated reference site and may show examples for newer builder inputs than the published standalone wheel. As of March 12, 2026, the docs site includes a local-generation example against `boto3==1.42.66`.
- Treat the docs root as the authoritative feature map for available client/resource/paginator/waiter/type exports, but verify the exact published package version on PyPI before pinning a lockfile or CI constraint.

## Official Sources

- Docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_cloudformation/
- PyPI package page: https://pypi.org/project/mypy-boto3-cloudformation/
- Upstream repository: https://github.com/youtype/mypy_boto3_builder

---
name: mypy-boto3-ssm
description: "mypy-boto3-ssm package guide for typed boto3 SSM clients, paginators, waiters, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.54"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,ssm,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-ssm Python Package Guide

## Golden Rule

Use `mypy-boto3-ssm` only for static typing around the `boto3` SSM client.

- Install `boto3` for runtime AWS calls.
- Install `mypy-boto3-ssm` when you want only the SSM stub package and are willing to annotate clients explicitly.
- Install `boto3-stubs[ssm]` when you want the upstream-recommended path with typed `boto3.client("ssm")` and `Session().client("ssm")` overloads.
- Keep auth, region, retries, and endpoints in normal `boto3` / botocore configuration. This package does not add a runtime wrapper.

## Install

### Recommended for most projects

```bash
python -m pip install "boto3-stubs[ssm]==1.42.54"
```

This is the easiest way to get typed `boto3.client("ssm")`, paginators, waiters, literals, and `TypedDict` request/response shapes.

### Standalone SSM stubs

```bash
python -m pip install "boto3==1.42.54" "mypy-boto3-ssm==1.42.54"
```

Use this when you only want the SSM service stubs. In this mode, explicit annotations are the safest default.

### Lower-memory / PyCharm-oriented install

```bash
python -m pip install "boto3-stubs-lite[ssm]==1.42.54"
```

The upstream docs note that the lite variant is more memory-friendly, but it does not provide `session.client()` overloads. Add explicit client annotations when you use it.

## Setup And AWS Auth

Common setup sources:

- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
- named profiles via `AWS_PROFILE` or `Session(profile_name="...")`
- shared config in `~/.aws/config` and `~/.aws/credentials`
- IAM roles in AWS runtimes

Typical setup:

```python
from boto3.session import Session

session = Session(
    profile_name="dev",
    region_name="us-west-2",
)
```

If your `boto3` session is wrong, the stubs will still type-check but the AWS call will fail at runtime.

## Core Usage

### Typed SSM client

```python
from boto3.session import Session
from mypy_boto3_ssm.client import SSMClient

session = Session(profile_name="dev", region_name="us-west-2")
ssm: SSMClient = session.client("ssm")

parameter = ssm.get_parameter(
    Name="/app/db/password",
    WithDecryption=True,
)

print(parameter["Parameter"]["Value"])
```

### Typed response dictionaries

```python
from mypy_boto3_ssm.client import SSMClient
from mypy_boto3_ssm.type_defs import GetParameterResultTypeDef

def read_parameter(client: SSMClient, name: str) -> str:
    response: GetParameterResultTypeDef = client.get_parameter(
        Name=name,
        WithDecryption=True,
    )
    return response["Parameter"]["Value"]
```

Use `type_defs` when you want `TypedDict` coverage for nested response or request shapes instead of plain `dict[str, Any]`.

### Typed paginator

```python
from mypy_boto3_ssm.client import SSMClient
from mypy_boto3_ssm.paginator import DescribeParametersPaginator
from mypy_boto3_ssm.type_defs import ParameterStringFilterTypeDef

def list_secure_parameters(client: SSMClient) -> list[str]:
    filters: list[ParameterStringFilterTypeDef] = [
        {"Key": "Type", "Option": "Equals", "Values": ["SecureString"]}
    ]
    paginator: DescribeParametersPaginator = client.get_paginator("describe_parameters")

    names: list[str] = []
    for page in paginator.paginate(ParameterFilters=filters):
        for parameter in page.get("Parameters", []):
            names.append(parameter["Name"])
    return names
```

### Typed waiter

The boto3 SSM client exposes the `command_executed` waiter, and the stubs provide `CommandExecutedWaiter`.

```python
from mypy_boto3_ssm.client import SSMClient
from mypy_boto3_ssm.waiter import CommandExecutedWaiter

def run_shell_command(client: SSMClient, instance_id: str) -> str:
    command = client.send_command(
        InstanceIds=[instance_id],
        DocumentName="AWS-RunShellScript",
        Parameters={"commands": ["uname -a"]},
    )

    command_id = command["Command"]["CommandId"]
    waiter: CommandExecutedWaiter = client.get_waiter("command_executed")
    waiter.wait(CommandId=command_id, InstanceId=instance_id)
    return command_id
```

## Tooling Patterns

### Explicit annotations are the safest default

With the standalone package or the lite package, annotate clients, paginators, and waiters directly instead of relying on inference.

```python
from boto3.session import Session
from mypy_boto3_ssm.client import SSMClient

ssm: SSMClient = Session(region_name="us-east-1").client("ssm")
```

### Keep stubs out of production-only environments

If the stubs exist only in development or CI, gate imports behind `TYPE_CHECKING`.

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_ssm.client import SSMClient
else:
    SSMClient = object

def make_client() -> "SSMClient":
    return Session(region_name="us-west-2").client("ssm")
```

This pattern is also the upstream workaround for tools like `pylint` when the stubs are not importable at runtime.

## Common Pitfalls

- Installing only `mypy-boto3-ssm` and expecting it to replace `boto3`. It is a stubs package, not the runtime SDK.
- Importing the wrong module path. Install `mypy-boto3-ssm`, but import from `mypy_boto3_ssm`.
- Expecting zero-annotation `session.client("ssm")` inference from `mypy-boto3-ssm` alone. The upstream docs position `boto3-stubs[ssm]` as the automatic-overload path.
- Using `boto3-stubs-lite[ssm]` and then expecting `session.client("ssm")` overloads to exist. Add explicit annotations in lite mode.
- Letting `boto3`, `botocore`, and the stub package drift too far apart. These stubs are generated from boto3 service models, so version skew can show up as missing or incorrect types.
- Treating the docs site as the source of truth for exact installable versions. Use PyPI for pinning, then use the generated docs for module layout and available typed surfaces.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.42.54`.
- PyPI currently publishes `mypy-boto3-ssm 1.42.54`, and the package metadata says the stubs version follows the related boto3 version.
- On `2026-03-11`, the maintainer docs page for `mypy-boto3-ssm` showed locally generated examples using `boto3==1.42.57`. Treat that as a docs-generation input, not proof that `mypy-boto3-ssm==1.42.57` is on PyPI.
- When exact alignment matters, pin `boto3`, `botocore`, and `mypy-boto3-ssm` together and verify the package version on PyPI before copying examples from the generated docs.
- The official boto3 SSM reference currently lists paginators and a single waiter, `CommandExecuted`; the stub package mirrors that boto3 client surface for typing.

## Official Sources

- Docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_ssm/
- PyPI project: https://pypi.org/project/mypy-boto3-ssm/
- PyPI JSON for exact release metadata: https://pypi.org/pypi/mypy-boto3-ssm/json
- Upstream repository: https://github.com/youtype/boto3_stubs
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- Boto3 SSM reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/ssm.html

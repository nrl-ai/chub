---
name: mypy-boto3-lambda
description: "Python type stubs for boto3 Lambda clients, paginators, waiters, literals, and typed dicts"
metadata:
  languages: "python"
  versions: "1.42.37"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,lambda,boto3,typing,stubs,mypy,pyright"
---

# mypy-boto3-lambda Python Package Guide

## What This Package Is For

`mypy-boto3-lambda` is a stubs-only package for the Lambda service in `boto3`. It adds static typing for `Session().client("lambda")`, Lambda paginators, waiters, literals, and typed dict request or response shapes.

Use it when your code already uses `boto3` for AWS Lambda and you want better editor completion plus stricter checking from `mypy`, `pyright`, Pylance, or similar tools.

It does not replace the runtime SDK. Your application still uses `boto3` to talk to AWS.

## Golden Rule

- Install `boto3` for runtime behavior and `mypy-boto3-lambda` or `boto3-stubs[lambda]` for typing.
- Keep the `boto3` and stub package versions on the same `1.42.x` line when possible.
- Treat AWS credentials, profiles, and regions as `boto3` setup concerns, not stub-package concerns.
- If you use the lite package, expect to add explicit annotations because `session.client("lambda")` overloads are not provided.

## Install

Recommended explicit pin for runtime plus service-specific stubs:

```bash
python -m pip install "boto3==1.42.37" "mypy-boto3-lambda==1.42.37"
```

If you want automatic type discovery for `boto3.client("lambda")` and `Session().client("lambda")`, install the full extras package instead:

```bash
python -m pip install "boto3==1.42.37" "boto3-stubs[lambda]==1.42.37"
```

RAM-friendlier option:

```bash
python -m pip install "boto3==1.42.37" "boto3-stubs-lite[lambda]==1.42.37"
```

Notes:

- `boto3-stubs-lite[lambda]` does not provide `session.client/resource` overloads, so you usually need explicit imported types.
- The standalone `mypy-boto3-lambda` package is useful when you only want the Lambda service stubs instead of the broader extras package.
- The maintainer docs root is rolling and can show newer generator examples than `1.42.37`; pin to the PyPI release your project actually installs.

## Authentication And Region Setup

The stubs package does not load credentials or choose a region. `boto3` still follows the normal AWS credential chain and needs a region before requests succeed.

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Useful points from the boto3 docs:

- `boto3` checks explicit client or session parameters first, then environment variables, then shared AWS config and credentials files, and then runtime providers such as container credentials or EC2 instance metadata.
- `Session(region_name=...)` sets the default region for clients created from that session.
- `Session.client(...)` can still override `region_name`, `endpoint_url`, and credential parameters per client.

## Core Usage

### Typed Lambda client

This is the core pattern when you want explicit typing without relying on IDE auto-discovery:

```python
from boto3.session import Session
from mypy_boto3_lambda.client import LambdaClient

session = Session(profile_name="dev", region_name="us-east-1")
lambda_client: LambdaClient = session.client("lambda")
```

You can now call normal boto3 Lambda operations with editor and type-checker help:

```python
from boto3.session import Session
from mypy_boto3_lambda.client import LambdaClient

session = Session(region_name="us-east-1")
lambda_client: LambdaClient = session.client("lambda")

response = lambda_client.invoke(
    FunctionName="process-order",
    InvocationType="RequestResponse",
    Payload=b'{"order_id":"1234"}',
)

payload = response["Payload"].read()
print(payload)
```

### Paginators

The package exposes paginator classes under `mypy_boto3_lambda.paginator`.

```python
from boto3.session import Session
from mypy_boto3_lambda.client import LambdaClient
from mypy_boto3_lambda.paginator import ListFunctionsPaginator

client: LambdaClient = Session(region_name="us-east-1").client("lambda")
pages: ListFunctionsPaginator = client.get_paginator("list_functions")

for page in pages.paginate():
    for fn in page.get("Functions", []):
        print(fn["FunctionName"])
```

Useful paginator names include `list_functions`, `list_aliases`, `list_event_source_mappings`, and `list_versions_by_function`.

### Waiters

Lambda-specific waiters are typed under `mypy_boto3_lambda.waiter`.

```python
from boto3.session import Session
from mypy_boto3_lambda.client import LambdaClient
from mypy_boto3_lambda.waiter import FunctionActiveWaiter

client: LambdaClient = Session(region_name="us-east-1").client("lambda")
waiter: FunctionActiveWaiter = client.get_waiter("function_active")

waiter.wait(FunctionName="process-order")
```

Useful waiter names include `function_active`, `function_updated`, `function_exists`, and `published_version_active`.

### Literals And Type Definitions

The generated `literals` and `type_defs` modules are useful when you want stronger typing for inputs and response shapes.

```python
from mypy_boto3_lambda.literals import ApplicationLogLevelType, InvocationTypeType
from mypy_boto3_lambda.type_defs import FunctionConfigurationTypeDef

def summarize_config(
    config: FunctionConfigurationTypeDef,
    invocation_type: InvocationTypeType,
    app_log_level: ApplicationLogLevelType,
) -> tuple[str | None, str, str]:
    return (
        config.get("FunctionName"),
        invocation_type,
        app_log_level,
    )
```

Use these modules when agents need to validate enum-like values or document-shaped dictionaries instead of leaving everything as plain `dict[str, Any]`.

## TYPE_CHECKING Pattern

If you want stubs available only during development and CI type checking, gate the imports behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_lambda.client import LambdaClient
else:
    LambdaClient = object

session = Session(region_name="us-east-1")
lambda_client: "LambdaClient" = session.client("lambda")
```

This avoids requiring the stub package at runtime, though some linters and IDEs behave better when the stubs are installed in the active environment.

## Tooling Notes

- `boto3-stubs[lambda]` is the easiest path for VS Code, Pylance, mypy, and pyright because no explicit client annotation is usually required.
- The maintainer docs recommend `boto3-stubs-lite` for PyCharm when literal-heavy overloads make the IDE slow.
- The package docs also note that `TYPE_CHECKING` is safe, but some `pylint` setups may need `object` fallbacks for imported stub types.

## Common Pitfalls

- This package is `Stubs Only`. Installing it without `boto3` does not give you a working Lambda client at runtime.
- A typed client does not guarantee valid AWS credentials, region, IAM permissions, or Lambda payload semantics.
- If you use `boto3-stubs-lite[lambda]`, `Session().client("lambda")` will not infer to `LambdaClient` automatically.
- The live docs site is generated from the latest builder output and may show commands for a newer `boto3` patch than the PyPI release you pinned.
- Lambda invoke payloads are still raw bytes or streams at runtime; the stubs improve types, but they do not JSON-encode or decode payloads for you.
- Some Lambda APIs shown in the generated paginator and type-def lists are newer service surface areas. If your environment is pinned to an older `boto3` or botocore build, verify that the operation exists before copying generated examples.

## Version-Sensitive Notes

- PyPI lists `mypy-boto3-lambda 1.42.37` as the latest release on March 12, 2026, released on January 28, 2026.
- PyPI describes the package as type annotations for `boto3 Lambda 1.42.37` generated with `mypy-boto3-builder 8.12.0`.
- The maintainer docs root is not patch-pinned. During this session it showed install-generation examples for a newer `boto3 1.42.66`, so prefer PyPI for exact package pinning and the docs root for current symbol coverage.

## Official Sources Used

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_lambda/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-lambda/`
- boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- boto3 session reference: `https://docs.aws.amazon.com/boto3/latest/reference/core/session.html`

---
name: mypy-boto3-apprunner
description: "mypy-boto3-apprunner package guide for typed boto3 App Runner clients, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,apprunner,boto3,python,typing,mypy,pyright,stubs"
---

# mypy-boto3-apprunner Python Package Guide

## Golden Rule

Use `boto3` for real AWS App Runner calls and use `mypy-boto3-apprunner` only for typing.

If you want `Session.client("apprunner")` to infer automatically, install `boto3-stubs[apprunner]`. If you install only `mypy-boto3-apprunner` or `boto3-stubs-lite[apprunner]`, expect to add explicit client annotations.

## Version-Sensitive Notes

- On `2026-03-12`, PyPI lists `mypy-boto3-apprunner 1.42.3`, which matches the version used here for this session.
- The PyPI package description says this release provides type annotations for `boto3.AppRunner 1.42.3`, so keep the stub package close to your installed `boto3` line.
- The maintainer docs site is rolling rather than patch-pinned. The same docs tree already shows generic local-generation examples for newer boto3 patch lines, so treat PyPI as the exact version source for frontmatter and install pins.
- The package index currently centers on typed `client`, `literals`, `type_defs`, and usage examples. Typed client code is the safest default workflow for App Runner.
- If the published stub package lags the boto3 version you must use, the maintainer project recommends generating fresh stubs locally with `mypy-boto3-builder`.

## Install

Choose one install mode based on how much automatic inference you want.

### Recommended for normal boto3 code

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[apprunner]==1.42.3"
```

Use this when you want editors and type checkers to infer the right type from ordinary `Session.client("apprunner")` code.

### Lower-memory option

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[apprunner]==1.42.3"
```

The maintainer docs note that the lite variant does not provide `session.client()` and `session.resource()` overloads, so explicit annotations become the normal pattern.

### Standalone service package

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-apprunner==1.42.3"
```

Use this when you want only the App Runner typing package or when you keep typing imports behind `TYPE_CHECKING`.

## Setup And Authentication

`mypy-boto3-apprunner` does not handle AWS authentication or runtime configuration. Credentials, region resolution, retries, timeouts, and endpoints still come from `boto3` and `botocore`.

Common credential sources from the boto3 credentials guide:

- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
- shared config in `~/.aws/credentials` and `~/.aws/config`
- `AWS_PROFILE` for named profiles
- assume-role, IAM Identity Center, container credentials, or EC2/ECS runtime roles

App Runner is regional, so keep the region explicit when it is not already obvious from the environment:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

```python
import boto3
from botocore.config import Config

session = boto3.Session(profile_name="dev", region_name="us-west-2")

apprunner = session.client(
    "apprunner",
    config=Config(
        retries={
            "mode": "standard",
            "max_attempts": 10,
        }
    ),
)
```

## Core Usage

### Zero-annotation workflow with `boto3-stubs[apprunner]`

With the full service extra installed, normal boto3 code should infer correctly:

```python
from boto3.session import Session

session = Session(region_name="us-west-2")
client = session.client("apprunner")

response = client.list_services()

for service in response.get("ServiceSummaryList", []):
    print(service["ServiceName"], service["Status"])
```

### Explicit client annotations

Use explicit types when you installed the standalone package or the lite package:

```python
from boto3.session import Session
from mypy_boto3_apprunner.client import AppRunnerClient

session = Session(region_name="us-west-2")
client: AppRunnerClient = session.client("apprunner")

service = client.describe_service(
    ServiceArn="arn:aws:apprunner:us-west-2:123456789012:service/my-service/0123456789abcdef"
)

print(service["Service"]["ServiceUrl"])
```

### Typed request dictionaries from `type_defs`

Use generated `TypedDict` shapes when building helper functions around nested App Runner request payloads:

```python
from boto3.session import Session
from mypy_boto3_apprunner.client import AppRunnerClient
from mypy_boto3_apprunner.type_defs import (
    DescribeServiceRequestTypeDef,
    StartDeploymentRequestTypeDef,
)

session = Session(region_name="us-west-2")
client: AppRunnerClient = session.client("apprunner")

describe_request: DescribeServiceRequestTypeDef = {
    "ServiceArn": "arn:aws:apprunner:us-west-2:123456789012:service/my-service/0123456789abcdef",
}

service = client.describe_service(**describe_request)

deploy_request: StartDeploymentRequestTypeDef = {
    "ServiceArn": describe_request["ServiceArn"],
}

client.start_deployment(**deploy_request)
print(service["Service"]["ServiceName"])
```

The published type definitions also cover many nested shapes used by operations such as `create_service`, `update_service`, `pause_service`, `resume_service`, `list_operations`, and custom-domain management.

### Literal types for constrained strings

Use literal aliases when a helper accepts only App Runner enum-like values:

```python
from mypy_boto3_apprunner.literals import RuntimeType

def normalize_runtime(value: RuntimeType) -> str:
    return value
```

This is useful when building wrappers that map project settings onto App Runner runtime or status fields.

## Runtime-Safe Typing Pattern

If production images install only `boto3`, keep stub imports behind `TYPE_CHECKING`:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_apprunner.client import AppRunnerClient
else:
    AppRunnerClient = object

client: "AppRunnerClient" = boto3.Session(region_name="us-west-2").client("apprunner")
```

This keeps the stub package out of runtime environments while preserving static analysis. The `else` branch also avoids the `pylint` undefined-name issue called out in the maintainer docs.

## App Runner-Specific Notes

- The boto3 service name is `apprunner`, not `mypy-boto3-apprunner`.
- App Runner ARNs are region-specific. Create the session in the same region as the service you are describing, updating, pausing, or resuming.
- `create_service` and `update_service` use large nested request shapes. `type_defs` helps catch missing fields or wrong nesting before you run the code.
- Starting a deployment or updating a service is still subject to IAM permissions, source connection state, image availability, VPC connector setup, and service runtime health. The stubs validate shapes, not AWS-side behavior.
- Keep retry, proxy, endpoint, and timeout configuration on the boto3 client or `botocore.config.Config`, not in your typing helpers.

## Common Pitfalls

- The PyPI package name uses hyphens, but the Python import root uses underscores: `mypy_boto3_apprunner`.
- This package is stub-only. It does not provide the runtime App Runner client by itself.
- `boto3-stubs[apprunner]`, `boto3-stubs-lite[apprunner]`, and `mypy-boto3-apprunner` are not ergonomically equivalent. Full `boto3-stubs` gives the best automatic inference.
- If your stub package version drifts away from the installed boto3 line, editor hints can become stale even when runtime code still works.
- The maintainer docs site is rolling. Install snippets there may reference newer boto3 patch versions than the exact PyPI release you pinned.
- Typed request payloads do not protect you from runtime issues like missing IAM permissions, invalid ARNs, unsupported regional resources, or bad source repository configuration.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_apprunner/`
- AppRunner client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_apprunner/client/`
- AppRunner type definitions: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_apprunner/type_defs/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-apprunner/`
- PyPI JSON metadata: `https://pypi.org/pypi/mypy-boto3-apprunner/json`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS App Runner boto3 reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/apprunner.html`

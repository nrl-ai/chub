---
name: mypy-boto3-amplify
description: "mypy-boto3-amplify package guide for typed boto3 Amplify clients, paginators, literals, and TypedDict API shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,aws,boto3,amplify,type-stubs,mypy,pyright"
---

# mypy-boto3-amplify Python Package Guide

## What This Package Is

`mypy-boto3-amplify` provides generated type annotations for the AWS Amplify boto3 client.

Use it when you want:

- typed `boto3` Amplify clients
- paginator types such as `ListAppsPaginator`
- generated literal enums from the Amplify API model
- request and response `TypedDict` shapes for static analysis

This package does not replace `boto3` at runtime and it does not configure AWS credentials or regions.

## Install

Choose one install mode based on how much boto3 typing you want.

### Recommended: full boto3 stubs for Amplify

```bash
python -m pip install "boto3-stubs[amplify]==1.42.3"
```

This is the best option when you want the most editor help around `Session().client("amplify")`.

### Service-only package

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-amplify==1.42.3"
```

Use this when you only need Amplify-specific stub modules and are willing to annotate the client explicitly.

### Lite variant for lower memory usage

```bash
python -m pip install "boto3-stubs-lite[amplify]==1.42.3"
```

The maintainer docs recommend the lite variant when the full package is too heavy for the IDE, but it does not provide the same boto3 session overloads. Plan to annotate client variables explicitly.

## Setup And Initialization

`mypy-boto3-amplify` has no package-specific configuration. Authentication, retry settings, endpoints, and region selection all come from normal `boto3` setup.

AWS's standard pattern is:

```bash
aws configure
```

or environment variables such as:

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_DEFAULT_REGION=us-east-1
```

Recommended typed setup:

```python
from boto3.session import Session
from mypy_boto3_amplify import AmplifyClient

session = Session(profile_name="dev", region_name="us-east-1")
amplify: AmplifyClient = session.client("amplify")
```

If you need a custom endpoint or botocore config, pass it to `session.client("amplify", ...)` exactly as you would without stubs.

## Core Usage

### Typed Amplify client

The stub package gives you the `AmplifyClient` type, but the runtime client still comes from `boto3`.

```python
from boto3.session import Session
from mypy_boto3_amplify import AmplifyClient

def make_amplify_client() -> AmplifyClient:
    session = Session(profile_name="dev", region_name="us-east-1")
    return session.client("amplify")
```

### Typed paginator

Amplify exposes paginator types through the package. Use the paginator class when listing resources such as apps.

```python
from boto3.session import Session
from mypy_boto3_amplify import AmplifyClient
from mypy_boto3_amplify.paginator import ListAppsPaginator

client: AmplifyClient = Session(region_name="us-east-1").client("amplify")
paginator: ListAppsPaginator = client.get_paginator("list_apps")

for page in paginator.paginate():
    for app in page.get("apps", []):
        print(app["appId"], app["name"])
```

### Typed response shapes

Use generated `type_defs` when you want the type checker to validate the response shape you are consuming.

```python
from boto3.session import Session
from mypy_boto3_amplify import AmplifyClient
from mypy_boto3_amplify.type_defs import ListAppsResultTypeDef

client: AmplifyClient = Session(region_name="us-east-1").client("amplify")
result: ListAppsResultTypeDef = client.list_apps(maxResults=25)

for app in result.get("apps", []):
    print(app["repository"])
```

### Literal enums for safer config

The generated literals catch typos in enum-like values before runtime.

```python
from mypy_boto3_amplify.literals import PlatformType

platform: PlatformType = "WEB"
```

### `TYPE_CHECKING` pattern

The maintainer docs explicitly recommend guarding stub-only imports when you do not want them to become hard runtime dependencies.

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_amplify import AmplifyClient
else:
    AmplifyClient = object

session = boto3.Session(region_name="us-east-1")
client: AmplifyClient = session.client("amplify")
```

## Type-Checking Workflow

Typical local setup:

```bash
python -m pip install boto3 mypy-boto3-amplify mypy
mypy your_module.py
```

`pyright` also works with these generated stubs. The maintainer docs also mention this package can help with IDE autocompletion and Pylint false-positive suppression.

## Config And Auth Notes

- The package does not define any Amplify-specific environment variables.
- Region handling still matters because Amplify is an AWS service client created through `boto3`.
- If credentials or region resolution fail, debug that as a `boto3` problem, not a stub-package problem.
- For multi-account or profile-based usage, prefer explicit `boto3.Session(profile_name=..., region_name=...)`.
- For custom retry or timeout behavior, use `botocore.config.Config` when constructing the client.

Example with explicit botocore config:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_amplify import AmplifyClient

config = Config(retries={"max_attempts": 5, "mode": "standard"})
session = Session(region_name="us-east-1")
client: AmplifyClient = session.client("amplify", config=config)
```

## Common Pitfalls

### Installing only the stubs

`mypy-boto3-amplify` is not the runtime SDK. Your application still needs `boto3`.

### Assuming every boto3 call becomes typed automatically

The maintainer docs call out current limitations around generated boto3 session overloads. In standalone or lite setups, explicitly annotate:

```python
amplify: AmplifyClient = session.client("amplify")
```

### Confusing the package name with the import path

- PyPI package: `mypy-boto3-amplify`
- Python import package: `mypy_boto3_amplify`

### Treating stubs as runtime validation

`TypedDict` and literal types help static analysis only. AWS still validates the request at runtime.

### Forgetting that version alignment matters

These stubs are generated from a particular boto3 and botocore model snapshot. Keep `boto3` and the stub package aligned when you depend on newer Amplify fields or enum values.

### Expecting resources or waiters

The official Amplify stubs surface client, paginator, literals, and `type_defs` usage. Do not assume there is a higher-level service resource layer or waiter support just because other AWS services have them.

## Version-Sensitive Notes

- This guide is pinned to version `1.42.3`.
- The official maintainer docs for `mypy-boto3-amplify` were generated against `boto3==1.42.3`.
- On 2026-03-12, the official PyPI project page showed a newer package release, `1.42.37`.
- The docs root and the current PyPI page are therefore not describing the same publication snapshot.
- If exact model coverage matters, pin both `boto3` and `mypy-boto3-amplify` deliberately and verify the version actually available in your package index.

## Official Sources

- Maintainer docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_amplify/
- Maintainer examples page: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_amplify/usage_examples/
- PyPI package: https://pypi.org/project/mypy-boto3-amplify/
- boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- boto3 session reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/core/session.html
- boto3 Amplify service reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/amplify.html

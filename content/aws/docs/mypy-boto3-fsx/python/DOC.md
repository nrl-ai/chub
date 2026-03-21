---
name: mypy-boto3-fsx
description: "Typed boto3 stubs for Amazon FSx clients, paginators, literals, and TypedDict request and response shapes in Python"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,fsx,boto3,typing,mypy,pyright,stubs,python"
---

# mypy-boto3-fsx Python Package Guide

## Golden Rule

`mypy-boto3-fsx` adds static types for `boto3.client("fsx")`. It is not a runtime SDK.

- Keep `boto3` installed for actual AWS requests, credentials, retries, and endpoints.
- Use `boto3-stubs[fsx]` if you want automatic typing for `Session.client("fsx")`.
- Use `mypy-boto3-fsx` or `boto3-stubs-lite[fsx]` when you want a narrower install and are willing to annotate the client explicitly.

## Install

### Full boto3 stubs

Use this when you want overloaded `Session.client("fsx")` inference in editors and type checkers:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[fsx]==1.42.3"
```

### Standalone FSx stubs

Use this when you only need FSx typing and are fine with explicit annotations:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-fsx==1.42.3"
```

### Lite stubs

Use this when install size or IDE performance matters more than overload-based inference:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[fsx]==1.42.3"
```

The maintainer docs note that the lite package does not provide `session.client/resource` overloads and is more RAM-friendly. The PyPI package page also recommends the lite variant for PyCharm when `Literal` overload performance becomes a problem.

Other package managers:

```bash
uv add "boto3==1.42.3" "mypy-boto3-fsx==1.42.3"
poetry add "boto3==1.42.3" "mypy-boto3-fsx==1.42.3"
```

## Authentication And Configuration

`mypy-boto3-fsx` does not add package-specific configuration. FSx auth and region handling come from normal boto3 behavior.

AWS documents this credential search order for boto3:

1. Credentials passed directly to `boto3.client(...)`
2. Credentials passed when creating a `Session(...)`
3. Environment variables
4. Assume-role providers
5. Shared AWS config and credential files and later providers in the standard chain

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Explicit session setup is safer when code can run in multiple accounts or regions:

```python
from boto3.session import Session
from mypy_boto3_fsx import FSxClient

session = Session(profile_name="dev", region_name="us-west-2")
fsx: FSxClient = session.client("fsx")
```

If you need client-specific retries, proxies, or a forced region, keep using normal boto3 and botocore config objects:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_fsx import FSxClient

config = Config(region_name="us-west-2", retries={"mode": "standard", "max_attempts": 10})
fsx: FSxClient = Session().client("fsx", config=config)
```

## Core Usage

### Typed FSx client

```python
from boto3.session import Session
from mypy_boto3_fsx import FSxClient

fsx: FSxClient = Session(region_name="us-west-2").client("fsx")

response = fsx.describe_file_systems()
for file_system in response.get("FileSystems", []):
    print(file_system["FileSystemId"], file_system["Lifecycle"])
```

### Typed paginator

The generated package includes paginator types such as `DescribeFileSystemsPaginator` and `DescribeVolumesPaginator`:

```python
from boto3.session import Session
from mypy_boto3_fsx import FSxClient
from mypy_boto3_fsx.paginator import DescribeFileSystemsPaginator

fsx: FSxClient = Session(region_name="us-west-2").client("fsx")
paginator: DescribeFileSystemsPaginator = fsx.get_paginator("describe_file_systems")

for page in paginator.paginate():
    for file_system in page.get("FileSystems", []):
        print(file_system["FileSystemId"])
```

### Literals and TypedDict request and response shapes

Use literal aliases for enum-like strings and `type_defs` for request or response dictionaries passed between helpers:

```python
from boto3.session import Session
from mypy_boto3_fsx import FSxClient
from mypy_boto3_fsx.literals import FileSystemTypeType
from mypy_boto3_fsx.type_defs import DescribeFileSystemsResponseTypeDef

expected_type: FileSystemTypeType = "LUSTRE"
fsx: FSxClient = Session(region_name="us-west-2").client("fsx")
response: DescribeFileSystemsResponseTypeDef = fsx.describe_file_systems()

for item in response.get("FileSystems", []):
    if item["FileSystemType"] == expected_type:
        print(item["FileSystemId"])
```

### Dev-only typing imports

If the stubs are installed only in development or CI, keep imports type-only:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_fsx import FSxClient
else:
    FSxClient = object

def get_fsx_client() -> "FSxClient":
    return Session(region_name="us-west-2").client("fsx")
```

The `object` fallback matches the maintainer's Pylint compatibility guidance for type-only imports.

## Common Pitfalls

- Treating `mypy-boto3-fsx` as a replacement for `boto3`. It only adds types.
- Installing the standalone or lite package and expecting `Session.client("fsx")` inference without an explicit `FSxClient` annotation.
- Importing with hyphens instead of underscores. Install `mypy-boto3-fsx`, import `mypy_boto3_fsx`.
- Letting `boto3` and the stubs drift apart. New FSx operations or fields can appear in runtime models before matching generated types arrive.
- Assuming type checking validates credentials, IAM permissions, region choice, or endpoint reachability. Those remain boto3 and AWS runtime concerns.
- Treating better typing as proof the FSx call is semantically correct. Service-specific constraints still come from the live FSx API.

## Version-Sensitive Notes For `1.42.3`

- On 2026-03-12, the version used here `1.42.3` matches the current maintainer docs and PyPI package page for `mypy-boto3-fsx`.
- PyPI lists this package as `Stubs Only`, requires Python `>=3.9`, and includes `1.42.3` in the published release history.
- The maintainer docs state that `mypy-boto3-fsx` versions follow the related boto3 version. If exact service-model alignment matters, pin `boto3` and the stubs to the same release line or generate stubs locally with `uvx --with 'boto3==1.42.3' mypy-boto3-builder`.

## Official Source URLs

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_fsx/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-fsx/`
- Boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- Boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
- Boto3 FSx reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/fsx.html`

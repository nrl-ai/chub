---
name: mypy-boto3-efs
description: "Type annotations for boto3 EFS in Python, covering typed clients, paginators, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,efs,boto3,type-stubs,mypy,pyright,autocomplete"
---

# mypy-boto3-efs Python Package Guide

## Golden Rule

`mypy-boto3-efs` is a stubs-only package for static typing. Use `boto3` for real AWS calls, and use either `boto3-stubs[efs]` or `mypy-boto3-efs` to improve type checking and editor completion.

- Use `boto3-stubs[efs]` when you want `Session().client("efs")` to infer types automatically.
- Use `mypy-boto3-efs` when you want only the EFS service stubs and you are willing to add explicit annotations.
- Use `boto3-stubs-lite[efs]` if full overloads are too heavy for your editor; expect to annotate types explicitly.

## Install

### Recommended for normal boto3 code

Install the runtime SDK and the full service stubs together:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[efs]==1.42.3"
```

This is the best default when you want mypy, pyright, and editor tooling to infer the return type of `Session().client("efs")` without extra annotations.

### Lower-memory option

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[efs]==1.42.3"
```

Use this when full overloads make PyCharm or other tools slow. The maintainer docs explicitly note that the lite package does not provide `session.client/resource` overloads.

### Standalone service package

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-efs==1.42.3"
```

Use this when you want only EFS stubs installed or you plan to guard imports with `TYPE_CHECKING`.

Common alternatives:

```bash
uv add "boto3==1.42.3" "boto3-stubs[efs]==1.42.3"
poetry add "boto3==1.42.3" "boto3-stubs[efs]==1.42.3"
```

Notes:

- Keep the stub version aligned with the `boto3` version you pin. The maintainer states that `mypy-boto3-efs` uses the related boto3 version number.
- `mypy-boto3-efs` does not replace `boto3`. If you install only the stub package, code may type-check but runtime imports and AWS calls still depend on `boto3`.

## Authentication And Runtime Setup

This package adds typing only. Credentials, region selection, retry behavior, endpoints, and actual network calls still come from `boto3`.

AWS documents that Boto3 searches for credentials in order, starting with explicit client parameters, explicit `Session(...)` parameters, environment variables, assume-role providers, and shared config files. In practice:

1. Prefer `aws configure` or an AWS profile for local development.
2. Set `AWS_PROFILE` and `AWS_DEFAULT_REGION` when the environment is not obvious.
3. Pass `region_name` explicitly in generated code when region ambiguity could break requests.

Profile-based setup:

```bash
export AWS_PROFILE="dev"
export AWS_DEFAULT_REGION="us-east-1"
```

Minimal typed client setup:

```python
from boto3.session import Session
from mypy_boto3_efs import EFSClient

session = Session(region_name="us-east-1")
client: EFSClient = session.client("efs")
```

Use the client interface as the main surface. The current AWS boto3 EFS reference exposes a low-level client and paginators, and the EFS stub docs I validated are centered on `EFSClient`, literals, paginators, and `type_defs`.

## Core Usage

### Zero-annotation workflow with `boto3-stubs[efs]`

With the full `boto3-stubs` extra installed, ordinary boto3 code should type-check without explicit annotations:

```python
from boto3.session import Session

session = Session(region_name="us-east-1")
client = session.client("efs")

response = client.describe_file_systems()
for file_system in response.get("FileSystems", []):
    print(file_system["FileSystemId"], file_system["LifeCycleState"])
```

### Explicit client annotations

Use explicit annotations when you installed `mypy-boto3-efs` or `boto3-stubs-lite[efs]`:

```python
from boto3.session import Session
from mypy_boto3_efs import EFSClient

session = Session(region_name="us-east-1")
client: EFSClient = session.client("efs")

response = client.describe_access_points(FileSystemId="fs-12345678")
for access_point in response.get("AccessPoints", []):
    print(access_point["AccessPointId"])
```

### Typed paginators

AWS currently documents five EFS paginators: `describe_access_points`, `describe_file_systems`, `describe_mount_targets`, `describe_replication_configurations`, and `describe_tags`.

```python
from boto3.session import Session
from mypy_boto3_efs import EFSClient
from mypy_boto3_efs.paginator import DescribeFileSystemsPaginator

session = Session(region_name="us-east-1")
client: EFSClient = session.client("efs")
pages: DescribeFileSystemsPaginator = client.get_paginator("describe_file_systems")

for page in pages.paginate():
    for file_system in page.get("FileSystems", []):
        print(file_system["FileSystemId"])
```

### Literals

Use literals for constrained string values that appear in helpers and config plumbing:

```python
from mypy_boto3_efs.literals import DeletionModeType

def normalize_deletion_mode(value: DeletionModeType) -> DeletionModeType:
    return value
```

The maintainer docs show `DeletionModeType` as one of the generated EFS literals.

### `type_defs` for request and response shapes

Use `type_defs` when you want request fragments or response objects to stay typed across helper functions:

```python
from mypy_boto3_efs.type_defs import PosixUserOutputTypeDef

def get_posix_user(uid: int, gid: int) -> PosixUserOutputTypeDef:
    return {
        "Uid": uid,
        "Gid": gid,
    }
```

The generated docs also list service-specific request and response `TypedDict`s such as `CreateAccessPointRequestTypeDef`, `DescribeFileSystemsResponseTypeDef`, and `MountTargetDescriptionTypeDef`.

## `TYPE_CHECKING` Pattern

If you do not want to ship the stub package in production, import the types only when type checkers are running:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_efs import EFSClient
else:
    EFSClient = object

session = Session(region_name="us-east-1")
client: "EFSClient" = session.client("efs")
```

This matches the maintainer guidance for keeping stub dependencies out of production environments while avoiding pylint issues with missing symbols.

## Common Pitfalls

- `mypy-boto3-efs` is stub-only. It does not create runtime clients and it does not include `boto3`.
- `boto3-stubs[efs]` and `mypy-boto3-efs` are not equivalent ergonomically. The full `boto3-stubs` extra gives better type inference for normal boto3 code.
- `boto3-stubs-lite[efs]` is more memory-friendly, but you should expect to add explicit annotations.
- Keep `boto3` and the EFS stubs on matching versions when you want predictable method signatures and generated shape names.
- The current EFS pages I validated do not expose a typed service-resource section or waiter section for EFS. Generate code around the client and paginators first.
- Type correctness does not validate your AWS setup. Bad credentials, wrong region, missing IAM permissions, or wrong file system IDs will still fail at runtime.

## Version-Sensitive Notes For `1.42.3`

- PyPI lists `mypy-boto3-efs 1.42.3` as the latest release on March 12, 2026, published on December 4, 2025.
- The package description says it provides type annotations for `boto3 EFS 1.42.3` and that package versioning follows the related `boto3` version.
- The hosted maintainer docs are a rolling generated site. The current docs root already shows a local-generation example against `boto3==1.42.44`, so do not assume every example on the docs site is pinned to `1.42.3`.
- The AWS boto3 documentation is also on a rolling latest patch line, so use PyPI as the source of truth for the exact stub package version you pin in code.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_efs/
- PyPI package page: https://pypi.org/project/mypy-boto3-efs/
- AWS boto3 EFS reference: https://docs.aws.amazon.com/boto3/latest/reference/services/efs.html
- AWS boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html

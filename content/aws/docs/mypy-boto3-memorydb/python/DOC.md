---
name: mypy-boto3-memorydb
description: "mypy-boto3-memorydb package guide for typed boto3 MemoryDB clients, paginators, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,memorydb,boto3,mypy-boto3-memorydb,boto3-stubs,typing,type-checking"
---

# mypy-boto3-memorydb Python Package Guide

## Golden Rule

Use `boto3` for real AWS calls and use `mypy-boto3-memorydb` only for static typing.

If you want type inference with normal `Session().client("memorydb")` code, install `boto3-stubs[memorydb]`. If you install `mypy-boto3-memorydb` or `boto3-stubs-lite[memorydb]`, expect to add explicit `MemoryDBClient` annotations.

MemoryDB is client-oriented in boto3. AWS publishes a MemoryDB client reference and paginators, but not a boto3 resource surface for this service, so do not plan around `Session().resource("memorydb")`.

## Install

### Recommended for ordinary boto3 code

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[memorydb]==1.42.3"
```

This is the easiest option when you want IDEs and type checkers to infer `Session().client("memorydb")` automatically.

### Lower-memory option

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[memorydb]==1.42.3"
```

Use this when PyCharm or another editor struggles with literal-heavy overloads. The lite package is more RAM-friendly, but the maintainer docs say it does not provide `session.client()` and `session.resource()` overloads.

### Standalone package

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-memorydb==1.42.3"
```

Use the standalone package when you only want the MemoryDB stubs package installed or when you prefer to keep stub imports behind `TYPE_CHECKING`.

Common alternatives:

```bash
uv add "boto3==1.42.3" "boto3-stubs[memorydb]==1.42.3"
poetry add "boto3==1.42.3" "boto3-stubs[memorydb]==1.42.3"
```

## Setup And Authentication

`mypy-boto3-memorydb` does not change AWS authentication, region resolution, retries, or endpoints. Those still come from `boto3` and `botocore`.

AWS's boto3 credentials guide says you need both credentials and a region configured, and that boto3 searches in a fixed order including:

1. Explicit credentials passed to `boto3.client(...)`
2. Explicit credentials passed to `boto3.Session(...)`
3. Environment variables
4. Assume-role and web-identity providers
5. AWS IAM Identity Center credentials
6. Shared credentials and config files
7. Container or EC2 instance metadata credentials

Practical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Minimal typed client setup:

```python
from boto3.session import Session
from mypy_boto3_memorydb import MemoryDBClient

session = Session(profile_name="dev", region_name="us-east-1")
client: MemoryDBClient = session.client("memorydb")
```

If stubs are installed only in development, guard the import:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_memorydb import MemoryDBClient
else:
    MemoryDBClient = object

client: "MemoryDBClient" = Session(region_name="us-east-1").client("memorydb")
```

The `object` fallback matches the maintainer guidance for avoiding pylint complaints in non-`TYPE_CHECKING` mode.

## Core Usage

### Explicit client annotations

```python
from boto3.session import Session
from mypy_boto3_memorydb import MemoryDBClient

client: MemoryDBClient = Session(region_name="us-east-1").client("memorydb")

response = client.describe_clusters(ShowShardDetails=False)
for cluster in response.get("Clusters", []):
    print(cluster["Name"], cluster["Status"])
```

### Typed paginator annotations

The maintainer docs publish typed paginators for MemoryDB listing calls including:

- `describe_acls`
- `describe_clusters`
- `describe_engine_versions`
- `describe_events`
- `describe_multi_region_clusters`
- `describe_parameter_groups`
- `describe_parameters`
- `describe_reserved_nodes_offerings`
- `describe_reserved_nodes`
- `describe_service_updates`
- `describe_snapshots`
- `describe_subnet_groups`
- `describe_users`

Example:

```python
from boto3.session import Session
from mypy_boto3_memorydb import MemoryDBClient
from mypy_boto3_memorydb.paginator import DescribeClustersPaginator

client: MemoryDBClient = Session(region_name="us-east-1").client("memorydb")
paginator: DescribeClustersPaginator = client.get_paginator("describe_clusters")

for page in paginator.paginate(ShowShardDetails=False):
    for cluster in page.get("Clusters", []):
        print(cluster["Name"])
```

### Typed request and response fragments

Use `type_defs` when your helpers accept or return AWS-shaped dictionaries:

```python
from mypy_boto3_memorydb.type_defs import ACLPendingChangesTypeDef

def read_pending_changes(value: ACLPendingChangesTypeDef) -> list[str]:
    return value.get("UserNamesToRemove", [])
```

### Literal value types

Use literals when you want the checker to reject impossible string values:

```python
from mypy_boto3_memorydb.literals import AZStatusType

def accepts_status(value: AZStatusType) -> str:
    return value
```

## Configuration Notes

- Keep runtime configuration on the boto3 session or client, not in the stubs package.
- Use `botocore.config.Config(...)` for retries, timeouts, or proxy settings.
- Keep the service name exact: `session.client("memorydb")`.
- Treat the stubs package as a development dependency unless you intentionally want it in production images.

Example client config:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_memorydb import MemoryDBClient

config = Config(retries={"mode": "standard", "max_attempts": 10})
client: MemoryDBClient = Session(region_name="us-east-1").client("memorydb", config=config)
```

## Common Pitfalls

- Install with `mypy-boto3-memorydb`, but import from `mypy_boto3_memorydb`.
- `mypy-boto3-memorydb` is stub-only. It does not replace `boto3`, and it does not create a working runtime client by itself.
- Do not expect a boto3 MemoryDB resource surface. AWS documents MemoryDB as a client-plus-paginators service.
- Do not assume the lite package will infer `Session().client("memorydb")` automatically; add explicit `MemoryDBClient` annotations.
- Keep `boto3`, `botocore`, and the stubs package close in version. These stubs are generated from boto3 service models, so version drift can surface stale or missing types.
- Typed code can still fail at runtime because of IAM permissions, region mismatches, service access, or request semantics. The stubs only improve static checking.

## Version-Sensitive Notes For `1.42.3`

- PyPI lists `mypy-boto3-memorydb 1.42.3` as the latest release on March 12, 2026, published on December 4, 2025.
- The live PyPI project page says the package version matches the related `boto3` version and follows Python packaging version specifiers.
- The release history jumps from `1.41.0` to `1.42.3`; do not assume every boto3 patch number gets a matching standalone MemoryDB stub release.
- The PyPI project links currently point to the `youtype/mypy_boto3_builder` repository for homepage, source, tracker, and releases. Use that live project metadata instead of older secondary references to other repo names.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_memorydb/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-memorydb/`
- AWS MemoryDB boto3 reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/memorydb.html`
- AWS boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- Maintainer repository from the live PyPI project links: `https://github.com/youtype/mypy_boto3_builder`

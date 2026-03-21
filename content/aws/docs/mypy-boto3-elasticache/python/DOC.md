---
name: mypy-boto3-elasticache
description: "Type stubs for boto3 ElastiCache in Python projects, including typed clients, paginators, waiters, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,elasticache,type-stubs,mypy,pyright,python"
---

# mypy-boto3-elasticache Python Package Guide

## Golden Rule

Use `mypy-boto3-elasticache` for static typing only, and keep normal `boto3` as the runtime SDK. Create real clients with `boto3` or `boto3.session.Session`, then annotate them with `ElastiCacheClient` or install `boto3-stubs[elasticache]` if you want broader boto3 overload support in editors and type checkers.

## What This Package Gives You

`mypy-boto3-elasticache` adds type information for the boto3 ElastiCache client surface:

- typed client annotations for `Session.client("elasticache")`
- typed paginator and waiter classes
- generated `TypedDict` request and response shapes
- literal unions for enum-like string parameters

It does not replace boto3 at runtime and does not add new AWS behavior by itself.

## Install

Pin the stubs to the boto3 version your project expects. This package follows boto3 versioning, so version drift between the runtime SDK and the stubs is the main source of missing or mismatched symbols.

Standalone service stubs:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-elasticache==1.42.3"
```

If you want the broader `boto3-stubs` experience with service extras:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[elasticache]==1.42.3"
```

Lower-memory alternative:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[elasticache]==1.42.3"
```

Common alternatives:

```bash
uv add "boto3==1.42.3" "mypy-boto3-elasticache==1.42.3"
poetry add "boto3==1.42.3" "mypy-boto3-elasticache==1.42.3"
```

If the stubs are only needed for type checking, keeping them in a development dependency group is usually the right choice.

## Authentication And Setup

ElastiCache uses normal boto3 credential and region configuration. Boto3 checks explicit client parameters, explicit `Session(...)` parameters, environment variables, shared AWS config and credentials files, and then runtime providers such as container or instance metadata.

Prefer one of these setup paths:

1. Local development with a named AWS profile in `~/.aws/config` and `~/.aws/credentials`
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_DEFAULT_REGION`
3. IAM roles or workload identity when running in AWS

Typed client setup with an explicit region:

```python
from boto3.session import Session
from mypy_boto3_elasticache import ElastiCacheClient

session = Session(profile_name="dev", region_name="us-east-1")
client: ElastiCacheClient = session.client("elasticache")
```

If the stubs are not installed in production, keep the type import behind `TYPE_CHECKING`:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_elasticache import ElastiCacheClient

session = Session(region_name="us-east-1")
client: ElastiCacheClient = session.client("elasticache")
```

## Core Usage

### Type a normal ElastiCache client

```python
from boto3.session import Session
from mypy_boto3_elasticache import ElastiCacheClient

session = Session(region_name="us-east-1")
client: ElastiCacheClient = session.client("elasticache")

response = client.describe_cache_clusters(ShowCacheNodeInfo=True)

for cluster in response.get("CacheClusters", []):
    print(cluster["CacheClusterId"], cluster["Engine"])
```

### Type paginators explicitly

```python
from mypy_boto3_elasticache import ElastiCacheClient
from mypy_boto3_elasticache.paginator import DescribeCacheClustersPaginator

client: ElastiCacheClient = session.client("elasticache")
paginator: DescribeCacheClustersPaginator = client.get_paginator(
    "describe_cache_clusters"
)

for page in paginator.paginate(ShowCacheNodeInfo=True):
    for cluster in page.get("CacheClusters", []):
        print(cluster["CacheClusterId"])
```

The ElastiCache boto3 service reference also exposes paginator names such as `DescribeCacheClusters`, `DescribeEngineDefaultParameters`, `DescribeReplicationGroups`, `DescribeReservedCacheNodes`, `DescribeReservedCacheNodesOfferings`, `DescribeServerlessCaches`, `DescribeSnapshots`, `DescribeUpdateActions`, and `DescribeUsers`.

### Type waiters explicitly

```python
from mypy_boto3_elasticache import ElastiCacheClient
from mypy_boto3_elasticache.waiter import CacheClusterAvailableWaiter

client: ElastiCacheClient = session.client("elasticache")
waiter: CacheClusterAvailableWaiter = client.get_waiter("cache_cluster_available")

waiter.wait(
    CacheClusterId="my-cache-cluster",
    WaiterConfig={"Delay": 30, "MaxAttempts": 20},
)
```

For this package version, the generated stubs include waiter classes for cache clusters, cache parameter groups, replication groups, snapshots, and global replication groups.

### Use generated literals and TypedDicts

```python
from mypy_boto3_elasticache.literals import AZModeType
from mypy_boto3_elasticache.type_defs import TagTypeDef

def normalize_inputs(mode: AZModeType, tags: list[TagTypeDef]) -> tuple[AZModeType, list[TagTypeDef]]:
    return mode, tags
```

Use these imports when you want static validation of request payloads or helper functions that wrap boto3 calls.

## Configuration Notes

- Set `region_name` explicitly when the runtime environment is ambiguous. ElastiCache is regional, and the wrong default region leads to confusing "resource not found" failures.
- Prefer `Session(profile_name=..., region_name=...)` for local tools or multi-account code instead of scattering credentials through `client(...)` calls.
- Keep runtime boto3 retries and botocore config in the real client setup. The stubs package does not change retry behavior, timeouts, endpoints, or auth resolution.
- Use the AWS service reference for operation names and runtime semantics; use the stubs package for typing and editor support.

## Common Pitfalls

- Installing `mypy-boto3-elasticache` without `boto3` does not give you a working AWS client. The stubs package is typing-only.
- Do not assume the latest online boto3 reference exactly matches your pinned stubs. On March 12, 2026, the package is pinned to `1.42.3`, while the live boto3 docs may already describe newer service updates.
- `boto3-stubs-lite[elasticache]` is lighter, but the maintainer docs note that it does not provide `Session.client()` or `Session.resource()` overloads. Use explicit annotations when you choose the lite variant.
- Keep stub imports out of production-only environments unless you intentionally ship them there. `TYPE_CHECKING` imports are the safest pattern when the dependency is dev-only.
- TypedDicts describe request and response shapes, but runtime validation still happens in boto3 and AWS. Static types do not replace service-side constraints.

## Version-Sensitive Notes

- The version used here `1.42.3` matched the current PyPI release observed on March 12, 2026.
- The maintainer states that the stubs package version is intended to stay aligned with the related boto3 version. Pin both together when possible.
- The canonical package docs are generated from the `boto3-stubs` project. When AWS adds new ElastiCache operations or waiters in later boto3 releases, your pinned `1.42.3` stubs may lag those newer symbols even if the AWS docs site already shows them.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_elasticache/`
- Maintainer client docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_elasticache/client/`
- Maintainer paginators docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_elasticache/paginators/`
- Maintainer waiters docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_elasticache/waiters/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-elasticache/`
- AWS boto3 credential guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- AWS ElastiCache service reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/elasticache.html`

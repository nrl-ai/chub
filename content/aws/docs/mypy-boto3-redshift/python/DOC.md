---
name: mypy-boto3-redshift
description: "Type stubs for boto3 Redshift in Python projects, including typed clients, paginators, waiters, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.42"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,redshift,boto3,python,typing,stubs,mypy,pyright"
---

# mypy-boto3-redshift Python Package Guide

## Golden Rule

Use `mypy-boto3-redshift` for static typing only, and keep `boto3` as the runtime AWS SDK.

Create the real client with `boto3` or `boto3.session.Session`, then:

- install `boto3-stubs[redshift]` if you want editor and type-checker auto-discovery with minimal annotation noise
- install `mypy-boto3-redshift` directly if you want the Redshift-specific stubs and are fine importing the types explicitly

This package does not change AWS runtime behavior, credentials, retries, endpoints, or service semantics.

## What This Package Gives You

`mypy-boto3-redshift` adds type information for the boto3 Redshift control-plane client surface:

- typed client annotations via `RedshiftClient`
- generated paginator classes in `mypy_boto3_redshift.paginator`
- generated waiter classes in `mypy_boto3_redshift.waiter`
- generated literal unions in `mypy_boto3_redshift.literals`
- generated `TypedDict` request and response shapes in `mypy_boto3_redshift.type_defs`

Use this package for cluster-management code such as describing clusters, snapshots, subnet groups, scheduled actions, and tags.

Do not use it as a runtime library for SQL execution. For SQL statements, use the Redshift Data API client or a PostgreSQL-compatible driver against the cluster endpoint.

## Install

Pin `boto3` and the stubs to the same release line when possible.

Recommended for most projects:

```bash
python -m pip install "boto3==1.42.42" "boto3-stubs[redshift]==1.42.42"
```

Standalone Redshift stubs:

```bash
python -m pip install "boto3==1.42.42" "mypy-boto3-redshift==1.42.42"
```

Lower-memory alternative:

```bash
python -m pip install "boto3==1.42.42" "boto3-stubs-lite[redshift]==1.42.42"
```

Common alternatives:

```bash
uv add "boto3==1.42.42" "mypy-boto3-redshift==1.42.42"
poetry add "boto3==1.42.42" "mypy-boto3-redshift==1.42.42"
```

If you need stubs generated against your exact pinned boto3 version, the maintainer recommends local generation:

```bash
uvx --with "boto3==1.42.42" mypy-boto3-builder
```

## Authentication And Setup

This package has no auth configuration of its own. Redshift typing follows normal boto3 setup.

The boto3 credentials guide says boto3 checks credentials in this order: explicit client parameters, explicit `Session(...)` parameters, environment variables, assume-role providers, AWS IAM Identity Center, shared credential/config files, then runtime providers such as containers or EC2 instance metadata.

Practical defaults:

1. Local development: `Session(profile_name=..., region_name=...)`
2. CI or containers: environment variables or workload credentials
3. AWS runtime: attached IAM role or equivalent runtime identity

Typed client setup with an explicit region:

```python
from boto3.session import Session
from mypy_boto3_redshift import RedshiftClient

session = Session(profile_name="dev", region_name="us-east-1")
client: RedshiftClient = session.client("redshift")
```

Useful environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_MAX_ATTEMPTS`
- `AWS_RETRY_MODE`

Redshift is regional. If the environment is ambiguous, set `region_name` explicitly or you will get confusing not-found and access errors against the wrong region.

## Core Usage

### Type a normal Redshift client

```python
from boto3.session import Session
from mypy_boto3_redshift import RedshiftClient

client: RedshiftClient = Session(region_name="us-east-1").client("redshift")

response = client.describe_clusters(ClusterIdentifier="analytics-prod")
cluster = response["Clusters"][0]

print(cluster["ClusterIdentifier"])
print(cluster["ClusterStatus"])
print(cluster["NodeType"])
print(cluster.get("Endpoint", {}).get("Address"))
```

### Type paginators explicitly

AWS documents paginator support for Redshift list and describe operations such as `DescribeClusters`, `DescribeClusterSnapshots`, `DescribeEvents`, `DescribeTags`, and `ListRecommendations`.

```python
from boto3.session import Session
from mypy_boto3_redshift import RedshiftClient
from mypy_boto3_redshift.paginator import DescribeClustersPaginator

client: RedshiftClient = Session(region_name="us-east-1").client("redshift")
paginator: DescribeClustersPaginator = client.get_paginator("describe_clusters")

for page in paginator.paginate(MaxRecords=100):
    for cluster in page.get("Clusters", []):
        print(cluster["ClusterIdentifier"], cluster["ClusterStatus"])
```

### Type waiters explicitly

AWS documents four Redshift waiters: `cluster_available`, `cluster_deleted`, `cluster_restored`, and `snapshot_available`.

```python
from boto3.session import Session
from mypy_boto3_redshift import RedshiftClient
from mypy_boto3_redshift.waiter import ClusterAvailableWaiter

client: RedshiftClient = Session(region_name="us-east-1").client("redshift")
waiter: ClusterAvailableWaiter = client.get_waiter("cluster_available")

waiter.wait(
    ClusterIdentifier="analytics-prod",
    WaiterConfig={"Delay": 30, "MaxAttempts": 40},
)
```

Use waiters after create, restore, resize, or resume operations instead of hand-written sleep loops.

### Use generated literals and TypedDicts

```python
from mypy_boto3_redshift.literals import ActionTypeType
from mypy_boto3_redshift.type_defs import AcceptReservedNodeExchangeInputMessageTypeDef

def build_exchange_request(action_type: ActionTypeType) -> AcceptReservedNodeExchangeInputMessageTypeDef:
    return {
        "ActionType": action_type,
        "ReservedNodeId": "reserved-node-id",
        "TargetReservedNodeOfferingId": "offering-id",
    }
```

This is useful when you wrap boto3 calls in helper functions and want stronger static validation for request payloads.

### `TYPE_CHECKING` pattern for dev-only stubs

```python
from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_redshift import RedshiftClient
else:
    RedshiftClient = object

client: RedshiftClient = boto3.client("redshift", region_name="us-east-1")
```

This avoids importing the stubs package at runtime when it is installed only in development or CI. It also matches the maintainer's documented workaround for `pylint`.

## Configuration Notes

- Keep auth, region, retries, and timeouts on the real boto3 client. The stubs package does not alter runtime configuration.
- Prefer `Session(profile_name=..., region_name=...)` for local tools and multi-account automation.
- The Redshift client in boto3 is the control-plane API. It manages clusters, snapshots, parameter groups, and related resources.
- For SQL execution, use `redshift-data` or a PostgreSQL-compatible driver instead of the Redshift control-plane client.
- Use waiter and paginator types only after confirming the underlying boto3 client actually exposes the corresponding waiter or paginator in your pinned version.

## Common Pitfalls

- Installing `mypy-boto3-redshift` without `boto3` and expecting a working AWS client.
- Importing the package as `mypy-boto3-redshift` in Python code. The import root is `mypy_boto3_redshift`.
- Treating the Redshift control-plane client as a SQL client. `describe_clusters` and `create_cluster` are not the same surface as query execution.
- Assuming type checking proves IAM permissions, network reachability, cluster availability, or region selection are correct.
- Forgetting that `boto3-stubs-lite[redshift]` is lighter specifically because it drops `session.client()` and `session.resource()` overload support.
- Letting `boto3`, `botocore`, and the stubs drift too far apart. Generated symbols can lag or differ if versions are not aligned.
- Copying examples from generic Redshift Data API docs into this package. `mypy-boto3-redshift` targets the `redshift` service client, not `redshift-data` or `redshift-serverless`.

## Version-Sensitive Notes

- This entry is pinned to `1.42.42`, which matches the PyPI project page as of March 12, 2026.
- PyPI lists this release as published on February 4, 2026.
- PyPI says `mypy-boto3-redshift 1.42.42` was generated with `mypy-boto3-builder 8.12.0`.
- The maintainer documents that the package version tracks the related boto3 version. Keep `boto3` and `mypy-boto3-redshift` on the same version when practical.
- The AWS boto3 Redshift reference is a rolling `latest` docs tree and may show a nearby patch version in the page header. Use PyPI for exact package pinning and AWS docs for runtime behavior.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_redshift/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-redshift/`
- AWS boto3 credential guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 Redshift reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/redshift.html`
- Builder repository linked from PyPI for support and bug reports: `https://github.com/youtype/mypy_boto3_builder`

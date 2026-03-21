---
name: mypy-boto3-emr
description: "mypy-boto3-emr type stubs for boto3 Amazon EMR clients, paginators, waiters, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,emr,type-stubs,mypy,pyright,python"
---

# mypy-boto3-emr Python Package Guide

## Golden Rule

`mypy-boto3-emr` is a typing package for `boto3` EMR code. It does not replace `boto3`, it does not create clusters by itself, and it does not manage AWS credentials, regions, or IAM roles.

Use it in one of these modes:

- Install `boto3-stubs[emr]` if you want the best editor and type-checker experience with automatic `Session.client("emr")` overloads.
- Install `mypy-boto3-emr` if you want only the EMR stub package and are willing to add explicit annotations.
- Install `boto3-stubs-lite[emr]` if IDE memory use matters more than overload-based inference.

## Version-Sensitive Notes

- PyPI currently lists `mypy-boto3-emr 1.42.3`, released on `2025-08-25`, and the package description says it provides type annotations for `boto3 EMR 1.42.3`.
- The maintainer docs say the standalone service packages use the related `boto3` version number, so pin `boto3==1.42.3` when you want predictable signature parity.
- PyPI also shows this package was generated with `mypy-boto3-builder 8.12.0`.
- Inference from the official sources on March 12, 2026: the version used here `1.42.3` is still the current published version for this service package.

## Install

### Recommended for most projects

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[emr]==1.42.3"
```

This is the best default if you want autocomplete plus typed `Session.client("emr")` without annotating every client variable.

### Standalone EMR stubs

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-emr==1.42.3"
```

Use this when you want only the EMR service stubs. In this mode, explicit annotations are usually necessary.

### Lower-memory IDE fallback

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[emr]==1.42.3"
```

The lite package is more memory-friendly, especially for PyCharm, but it does not provide `Session.client("emr")` overloads. Add explicit annotations if you use it.

Common alternatives:

```bash
uv add "boto3==1.42.3" "boto3-stubs[emr]==1.42.3"
poetry add "boto3==1.42.3" "boto3-stubs[emr]==1.42.3"
```

## Runtime Setup And Auth

`mypy-boto3-emr` adds no runtime setup of its own. All real behavior still comes from `boto3`.

AWS documents that boto3 resolves credentials from a standard provider chain that includes:

- credentials passed directly to `boto3.client(...)`
- credentials passed to `boto3.Session(...)`
- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, `AWS_PROFILE`, and `AWS_DEFAULT_REGION`
- shared config in `~/.aws/credentials` and `~/.aws/config`
- assume-role, IAM Identity Center, container credentials, and EC2 instance metadata

Safe local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Explicit session setup:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
emr = session.client("emr")
```

For `run_job_flow`, remember that typing only validates request shapes. Runtime success still depends on EMR prerequisites such as subnet/networking choices, instance profiles, service roles, release labels, and region-specific availability.

## Core Usage

### Typed client

```python
from boto3.session import Session
from mypy_boto3_emr.client import EMRClient

session = Session(profile_name="dev", region_name="us-east-1")
emr: EMRClient = session.client("emr")

response = emr.list_clusters(
    ClusterStates=["STARTING", "BOOTSTRAPPING", "RUNNING", "WAITING"],
)

for cluster in response.get("Clusters", []):
    print(cluster["Id"], cluster["Name"], cluster["Status"]["State"])
```

AWS documents boto3 clients as the low-level interface that maps closely to the full service API. That is the right mental model for EMR here: this package types the EMR control-plane API surface, not Spark code, bootstrap scripts, or data-processing logic inside a cluster.

### Keep stubs dev-only with `TYPE_CHECKING`

If production images do not install stub packages, gate the imports and cast the client:

```python
from typing import TYPE_CHECKING, cast

import boto3

if TYPE_CHECKING:
    from mypy_boto3_emr.client import EMRClient

emr = cast("EMRClient", boto3.Session(region_name="us-east-1").client("emr"))
```

### Typed paginators

The generated docs expose paginator types for common list operations such as:

- `list_bootstrap_actions`
- `list_clusters`
- `list_instance_fleets`
- `list_instance_groups`
- `list_instances`
- `list_steps`

Example:

```python
from boto3.session import Session
from mypy_boto3_emr.client import EMRClient
from mypy_boto3_emr.paginator import ListClustersPaginator

client: EMRClient = Session(region_name="us-east-1").client("emr")
paginator: ListClustersPaginator = client.get_paginator("list_clusters")

for page in paginator.paginate(ClusterStates=["RUNNING", "WAITING"]):
    for cluster in page.get("Clusters", []):
        print(cluster["Id"], cluster["Name"])
```

### Typed waiters

The generated docs expose waiter types including:

- `cluster_running`
- `cluster_terminated`
- `step_complete`

Example:

```python
from boto3.session import Session
from mypy_boto3_emr.client import EMRClient
from mypy_boto3_emr.waiter import ClusterRunningWaiter

client: EMRClient = Session(region_name="us-east-1").client("emr")
waiter: ClusterRunningWaiter = client.get_waiter("cluster_running")
waiter.wait(ClusterId="j-ABCDEFGHIJKL")
```

### Literals and TypedDicts

Use `literals` when an EMR field is a constrained string union, and use `type_defs` when helper functions build request payloads or normalize responses.

```python
from mypy_boto3_emr.literals import ClusterStateTypeType

def is_active_state(state: ClusterStateTypeType) -> bool:
    return state in {"STARTING", "BOOTSTRAPPING", "RUNNING", "WAITING"}
```

This is useful when code branches on cluster state, step state, action-on-failure settings, or instance-group configuration values.

## Configuration Notes

- `mypy-boto3-emr` focuses on the EMR client surface. Do not expect a rich boto3 resource layer like `service_resource` modules here.
- The stub package helps type-check `client.get_paginator(...)`, `client.get_waiter(...)`, literal unions, and request or response `TypedDict` shapes exposed under `mypy_boto3_emr.type_defs`.
- Keep the `boto3` runtime version aligned with the EMR stub version. Mismatched versions can still run, but the method signatures and enum values your editor sees may drift.
- EMR examples often fail for runtime reasons that typing cannot catch: invalid `ReleaseLabel`, missing `JobFlowRole` or `ServiceRole`, subnet constraints, unavailable instance types, or region mismatches.

## Common Pitfalls

- Installing only `mypy-boto3-emr` and expecting unannotated `Session.client("emr")` calls to become typed automatically. That behavior comes from `boto3-stubs[emr]`.
- Forgetting to install `boto3`. These packages are type stubs, not the runtime AWS SDK.
- Treating `mypy-boto3-emr` as if it validates EMR job semantics. It checks Python shapes, not whether a cluster config is valid for your account or region.
- Importing stub modules at runtime in production environments that do not install them. Use `TYPE_CHECKING` if stubs are dev-only.
- Using `boto3-stubs-lite[emr]` and expecting overload-based inference from `Session.client("emr")`. Lite mode needs more explicit annotations.
- Guessing waiter or paginator names from older blog posts. Pull them from the generated docs for the exact service package version you installed.
- Assuming typed examples cover EMR Serverless or the on-cluster Spark API surface. This package only types the classic boto3 EMR service client.

## Official Sources

- Docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_emr/`
- PyPI: `https://pypi.org/project/mypy-boto3-emr/`
- Source repository: `https://github.com/youtype/types-boto3`
- Boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- Boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`
- Boto3 clients guide: `https://docs.aws.amazon.com/boto3/latest/guide/clients.html`
- Boto3 EMR service reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/emr.html`

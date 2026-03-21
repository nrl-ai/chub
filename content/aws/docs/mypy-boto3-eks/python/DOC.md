---
name: mypy-boto3-eks
description: "mypy-boto3-eks package guide for typed boto3 Amazon EKS clients, paginators, waiters, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.66"
  revision: 3
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,eks,mypy,python,stubs,types"
---

# mypy-boto3-eks Python Package Guide

## What It Is

`mypy-boto3-eks` provides type annotations for the Amazon EKS part of `boto3`.

Use it when you want static checking and editor completion for:

- `EKSClient`
- paginator classes such as `ListClustersPaginator`
- waiter classes such as `ClusterActiveWaiter`
- generated `TypedDict` request and response shapes under `type_defs`
- generated literal unions under `literals`

It does not replace `boto3` at runtime. Real AWS calls still come from `boto3.Session(...).client("eks")`.

## Prerequisites

- Python `>=3.9`
- `boto3` for runtime AWS calls
- AWS credentials and a region configured through the normal boto3 credential chain

Common local environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

## Install

### Recommended for most projects

```bash
python -m pip install "boto3-stubs[eks]==1.42.66"
```

This is the maintainer-recommended path when you want EKS typing with `boto3-stubs` service extras.

### Install runtime boto3 through the umbrella package too

```bash
python -m pip install "boto3-stubs[boto3,eks]==1.42.66"
```

### Standalone service stubs

```bash
python -m pip install "boto3==1.42.66" "mypy-boto3-eks==1.42.66"
```

Use this when you want only the EKS stub package and are willing to annotate the client explicitly.

### Lower-memory variant

```bash
python -m pip install "boto3==1.42.66" "boto3-stubs-lite[eks]==1.42.66"
```

The lite package is useful when IDE memory use matters more than automatic `Session.client("eks")` overloads.

### Generate local stubs for an exact boto3 pin

If your project is pinned tightly to a specific `boto3` build and exact generated type parity matters, the maintainer docs point to local generation:

```bash
uvx --with "boto3==1.42.66" mypy-boto3-builder
```

## Initialize A Typed EKS Client

`mypy-boto3-eks` does not add its own auth or config layer. Create sessions and clients exactly as you would in normal boto3 code.

```python
from boto3.session import Session
from mypy_boto3_eks.client import EKSClient

session = Session(profile_name="dev", region_name="us-west-2")
eks: EKSClient = session.client("eks")

response = eks.describe_cluster(name="example-cluster")
print(response["cluster"]["arn"])
```

If you need retries, proxies, or timeouts, keep using botocore configuration:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_eks.client import EKSClient

config = Config(retries={"max_attempts": 10, "mode": "standard"})
session = Session(region_name="us-west-2")
eks: EKSClient = session.client("eks", config=config)
```

## Core Usage

### Typed Client Calls

Use the typed client for EKS control-plane operations such as cluster lookup, nodegroup inspection, access-entry management, and updates.

```python
from boto3.session import Session
from mypy_boto3_eks.client import EKSClient

eks: EKSClient = Session(region_name="us-west-2").client("eks")

cluster = eks.describe_cluster(name="example-cluster")["cluster"]
print(cluster["name"], cluster["status"])
```

EKS does not have a typed resource layer here comparable to S3 resources. Prefer the client interface.

### Typed Paginators

The generated docs expose paginator types for these operations:

- `describe_addon_versions`
- `describe_cluster_versions`
- `list_access_entries`
- `list_access_policies`
- `list_associated_access_policies`
- `list_capabilities`
- `list_clusters`
- `list_eks_anywhere_subscriptions`
- `list_fargate_profiles`
- `list_identity_provider_configs`
- `list_insights`
- `list_nodegroups`
- `list_pod_identity_associations`
- `list_updates`

Example:

```python
from boto3.session import Session
from mypy_boto3_eks.client import EKSClient
from mypy_boto3_eks.paginator import ListClustersPaginator

eks: EKSClient = Session(region_name="us-west-2").client("eks")
paginator: ListClustersPaginator = eks.get_paginator("list_clusters")

for page in paginator.paginate():
    for cluster_name in page.get("clusters", []):
        print(cluster_name)
```

### Typed Waiters

The generated docs expose waiter types for:

- `addon_active`
- `addon_deleted`
- `cluster_active`
- `cluster_deleted`
- `fargate_profile_active`
- `fargate_profile_deleted`
- `nodegroup_active`
- `nodegroup_deleted`

Example:

```python
from boto3.session import Session
from mypy_boto3_eks.client import EKSClient
from mypy_boto3_eks.waiter import ClusterActiveWaiter

eks: EKSClient = Session(region_name="us-west-2").client("eks")
waiter: ClusterActiveWaiter = eks.get_waiter("cluster_active")
waiter.wait(name="example-cluster")
```

### TypedDict Request And Response Shapes

Use `type_defs` when request payloads or responses move through helpers before the boto3 call site.

```python
from boto3.session import Session
from mypy_boto3_eks.client import EKSClient
from mypy_boto3_eks.type_defs import (
    AssociateAccessPolicyRequestTypeDef,
    DescribeClusterResponseTypeDef,
)

request: AssociateAccessPolicyRequestTypeDef = {
    "clusterName": "example-cluster",
    "principalArn": "arn:aws:iam::123456789012:role/EKSAdmin",
    "policyArn": "arn:aws:eks::aws:cluster-access-policy/AmazonEKSClusterAdminPolicy",
    "accessScope": {"type": "cluster"},
}

eks: EKSClient = Session(region_name="us-west-2").client("eks")
response: DescribeClusterResponseTypeDef = eks.describe_cluster(name="example-cluster")

print(request["policyArn"])
print(response["cluster"]["name"])
```

This is especially useful for access-entry, pod-identity, and other nested EKS request shapes where untyped dictionaries become noisy quickly.

### Literal Types

Use the generated literal unions when helpers should only accept the allowed EKS string values.

```python
from mypy_boto3_eks.literals import AMITypesType


def normalize_ami_type(value: AMITypesType) -> str:
    return value
```

### `TYPE_CHECKING` Pattern For Dev-Only Stubs

If the stub package is installed only in development or CI, keep typing imports behind `TYPE_CHECKING`.

```python
from typing import TYPE_CHECKING, cast

import boto3

if TYPE_CHECKING:
    from mypy_boto3_eks.client import EKSClient

eks = cast("EKSClient", boto3.Session(region_name="us-west-2").client("eks"))

for cluster_name in eks.list_clusters().get("clusters", []):
    print(cluster_name)
```

## What This Package Does Not Do

- It does not create AWS credentials.
- It does not ship a runtime EKS client.
- It does not manage Kubernetes objects inside a running cluster.
- It does not validate IAM permissions, cluster state, or network reachability at runtime.

For in-cluster resources such as Pods, Deployments, or Services, use kubeconfig or token-based auth plus the Kubernetes Python client after the EKS control-plane calls.

## Common Pitfalls

### Hyphenated Package Name vs Import Name

Install with `mypy-boto3-eks`, but import from `mypy_boto3_eks`.

### Treating The Stubs As The SDK

If an EKS call fails, debug it as normal boto3 code: credentials, region, IAM permissions, request shape, or service state. The stubs only affect static analysis.

### Expecting A Typed Resource Interface

EKS is a client-focused service in this package. Do not expect `Session.resource("eks")` typing similar to S3.

### Importing Stub Modules At Runtime When They Are Dev Dependencies

If production images do not install the stub package, direct runtime imports from `mypy_boto3_eks` will fail. Use `TYPE_CHECKING`, string annotations, or `cast(...)`.

### Confusing EKS Control-Plane Calls With Kubernetes API Calls

`describe_cluster`, `list_nodegroups`, `associate_access_policy`, and related methods operate on the AWS EKS control plane. They do not replace the Kubernetes API for workloads running inside the cluster.

### Assuming The Hosted Docs Are Release-Pinned

The maintainer docs site is a generated latest view. Use the PyPI release page for installation pinning, and use the hosted docs for the current typed surface and symbol names.

## Version-Sensitive Notes

- This entry is pinned to `1.42.66`.
- The package family tracks the related `boto3` version line, so keep `boto3` and `mypy-boto3-eks` close when exact API coverage matters.
- The hosted docs root is a moving latest page and can lag or lead the currently published PyPI package on patch labels.
- The maintainer docs currently show local generation text against `boto3==1.42.65`, so treat that page as the current API-shape reference rather than strict release-pinning evidence.
- If exact generated names matter more than using the published wheel, generate local stubs for your pinned `boto3` version.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_eks/`
- Upstream versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-eks/`
- PyPI JSON API: `https://pypi.org/pypi/mypy-boto3-eks/json`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- Boto3 clients guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/clients.html`
- Boto3 EKS reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/eks.html`
- Repository: `https://github.com/youtype/types-boto3`

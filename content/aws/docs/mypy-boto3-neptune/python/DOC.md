---
name: mypy-boto3-neptune
description: "mypy-boto3-neptune package guide for typed boto3 Neptune clients, paginators, waiters, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.57"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,neptune,boto3,mypy-boto3-neptune,boto3-stubs,typing,type-checking,python"
---

# mypy-boto3-neptune Python Package Guide

## Golden Rule

Use `boto3` for runtime AWS calls and use `mypy-boto3-neptune` only for typing.

If you want `Session().client("neptune")` to infer its type automatically, install `boto3-stubs[neptune]`. If you install `mypy-boto3-neptune` or `boto3-stubs-lite[neptune]`, plan on adding explicit type annotations for the client, paginators, and waiters.

## Install

### Recommended when you want overload inference

Install the runtime SDK and the full service extra together:

```bash
python -m pip install "boto3==1.42.57" "boto3-stubs[neptune]==1.42.57"
```

This is the easiest setup when you want editors and type checkers to understand `Session().client("neptune")` without explicit annotations.

### Lower-memory option

```bash
python -m pip install "boto3==1.42.57" "boto3-stubs-lite[neptune]==1.42.57"
```

The maintainer docs say the lite package is more RAM-friendly, but it does not provide `session.client()` and `session.resource()` overloads, so explicit annotations become the normal workflow.

### Standalone service package

```bash
python -m pip install "boto3==1.42.57" "mypy-boto3-neptune==1.42.57"
```

Use the standalone package when you only want Neptune typings or when you keep stub imports behind `TYPE_CHECKING`.

If you use other package managers:

```bash
uv add "boto3==1.42.57" "mypy-boto3-neptune==1.42.57"
poetry add "boto3==1.42.57" "mypy-boto3-neptune==1.42.57"
```

## Initialize Type Checking

### Zero-annotation workflow with `boto3-stubs[neptune]`

```python
from boto3.session import Session

session = Session(region_name="us-east-1")
client = session.client("neptune")

response = client.describe_db_clusters(MaxRecords=20)
for cluster in response.get("DBClusters", []):
    print(cluster["DBClusterIdentifier"])
```

### Explicit client annotations

Use this pattern with the standalone package or the lite package:

```python
from boto3.session import Session
from mypy_boto3_neptune.client import NeptuneClient

session = Session(region_name="us-east-1")
neptune: NeptuneClient = session.client("neptune")

response = neptune.describe_db_clusters(MaxRecords=20)
```

## Auth And Configuration

`mypy-boto3-neptune` does not handle AWS credentials, profiles, regions, retries, or endpoints. Those still come from normal `boto3` and `botocore` behavior.

The boto3 credentials guide says Boto3 searches credentials in this order: explicit client parameters, explicit `Session(...)` parameters, environment variables, assume-role providers, AWS IAM Identity Center, shared credentials/config files, container credentials, and EC2 instance metadata.

Typical local setup:

```bash
export AWS_PROFILE=graph-dev
export AWS_DEFAULT_REGION=us-east-1
```

Typed client setup with an explicit session:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_neptune.client import NeptuneClient

session = Session(profile_name="graph-dev", region_name="us-east-1")
neptune: NeptuneClient = session.client(
    "neptune",
    config=Config(retries={"mode": "standard", "max_attempts": 5}),
)
```

The boto3 session guide also says `Session` objects are not thread safe. Create one session per thread or worker when concurrency matters.

## Core Usage

### Typed Neptune client

The AWS boto3 reference for `describe_db_clusters` says the operation supports pagination and returns a `DBClusters` list:

```python
from boto3.session import Session
from mypy_boto3_neptune.client import NeptuneClient

neptune: NeptuneClient = Session(region_name="us-east-1").client("neptune")

response = neptune.describe_db_clusters(
    Filters=[
        {
            "Name": "engine",
            "Values": ["neptune"],
        }
    ],
    MaxRecords=20,
)

for cluster in response.get("DBClusters", []):
    print(cluster["DBClusterIdentifier"], cluster.get("Status"))
```

### Typed paginator

```python
from boto3.session import Session
from mypy_boto3_neptune.client import NeptuneClient
from mypy_boto3_neptune.paginator import DescribeDBClustersPaginator

neptune: NeptuneClient = Session(region_name="us-east-1").client("neptune")
paginator: DescribeDBClustersPaginator = neptune.get_paginator("describe_db_clusters")

for page in paginator.paginate(MaxRecords=20):
    for cluster in page.get("DBClusters", []):
        print(cluster["DBClusterIdentifier"])
```

### Typed waiter

The maintainer docs list `DBInstanceAvailableWaiter` and `DBInstanceDeletedWaiter` for this package:

```python
from boto3.session import Session
from mypy_boto3_neptune.client import NeptuneClient
from mypy_boto3_neptune.waiter import DBInstanceAvailableWaiter

neptune: NeptuneClient = Session(region_name="us-east-1").client("neptune")
waiter: DBInstanceAvailableWaiter = neptune.get_waiter("db_instance_available")

waiter.wait(
    DBInstanceIdentifier="graph-instance-1",
    WaiterConfig={"Delay": 30, "MaxAttempts": 40},
)
```

### Typed request and response helpers

Use `type_defs` and `literals` when helper functions need AWS-shaped dictionaries or constrained string values:

```python
from mypy_boto3_neptune.literals import ApplyMethodType
from mypy_boto3_neptune.type_defs import AddRoleToDBClusterMessageTypeDef

apply_method: ApplyMethodType = "immediate"

request: AddRoleToDBClusterMessageTypeDef = {
    "DBClusterIdentifier": "graph-cluster-1",
    "RoleArn": "arn:aws:iam::123456789012:role/NeptuneS3ExportRole",
}
```

### `TYPE_CHECKING` pattern

If production images do not install the stubs, keep imports type-only:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_neptune.client import NeptuneClient

session = Session(region_name="us-east-1")
neptune: "NeptuneClient" = session.client("neptune")
```

## Common Pitfalls

- `mypy-boto3-neptune` is stub-only. It does not replace `boto3`, and it does not create a runtime Neptune client by itself.
- `boto3-stubs[neptune]` and `mypy-boto3-neptune` are not equivalent ergonomically. The full `boto3-stubs` extra gives overload inference; the standalone and lite installs usually need explicit annotations.
- Keep `boto3`, `botocore`, and the stub package on the same release line when exact request and response shapes matter.
- The package name uses hyphens, but imports use underscores: `mypy-boto3-neptune` vs `mypy_boto3_neptune`.
- Good typing does not guarantee valid AWS credentials, region selection, IAM permissions, or Neptune cluster identifiers. Runtime failures still come from normal boto3 and botocore behavior.
- The current Neptune stub docs focus on the client, paginators, waiters, literals, and `type_defs`. Write new code against the client API unless you have a separate boto3 runtime reason to do otherwise.

## Version-Sensitive Notes For `1.42.57`

- PyPI listed `1.42.57` as the latest `mypy-boto3-neptune` release on March 12, 2026, and the project page reports it was released on February 25, 2026.
- The maintainer docs say the version of `boto3-stubs` follows the version of `boto3`, so keep the stub package aligned with the boto3 line you actually run.
- The standalone package docs and the umbrella `boto3-stubs` docs both note that lite packages do not provide `session.client()` overloads.
- AWS boto3 runtime docs are rolling reference docs, not exact package lockfiles. Use PyPI for the exact stub version you pin, and use the AWS boto3 docs for credential-chain behavior and operation semantics.

## Official Sources

- Maintainer docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_neptune/
- Maintainer versioning docs: https://youtype.github.io/boto3_stubs_docs/#versioning
- PyPI project page: https://pypi.org/project/mypy-boto3-neptune/
- AWS boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- AWS boto3 session guide: https://docs.aws.amazon.com/boto3/latest/guide/session.html
- AWS boto3 Neptune `describe_db_clusters`: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/neptune/client/describe_db_clusters.html

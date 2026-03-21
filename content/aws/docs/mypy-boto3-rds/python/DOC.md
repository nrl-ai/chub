---
name: mypy-boto3-rds
description: "mypy-boto3-rds package guide for Python with typed boto3 RDS clients, paginators, waiters, literals, and type defs"
metadata:
  languages: "python"
  versions: "1.42.51"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,boto3,rds,typing,stubs,mypy,pyright"
---

# mypy-boto3-rds Python Package Guide

## What It Is

`mypy-boto3-rds` is the generated type-stubs package for `boto3` Amazon RDS clients.

Use it when you want:

- a typed `RDSClient`
- typed paginators and waiters from `client.get_paginator(...)` and `client.get_waiter(...)`
- generated `Literal` aliases and `TypedDict` request and response shapes
- better autocomplete and static analysis in `mypy`, Pyright, Pylance, VS Code, or PyCharm

Important boundary:

- this package does not make AWS calls by itself
- runtime behavior still comes from `boto3` and `botocore`
- AWS credentials, region selection, retries, and endpoint settings are still standard boto3 configuration

## Version Covered

- Ecosystem: `pypi`
- Package: `mypy-boto3-rds`
- Import root: `mypy_boto3_rds`
- Runtime SDK: `boto3`
- Official latest observed on PyPI on `2026-03-11`: `1.42.51`
- PyPI release date: `2026-02-17`
- Python requirement on PyPI: `>=3.9`
- Docs root used for this entry: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_rds/`
- Registry URL: `https://pypi.org/project/mypy-boto3-rds/`
- Repository: `https://github.com/youtype/types-boto3`

## Install

Install the stubs package alongside `boto3`:

```bash
python -m pip install boto3 mypy-boto3-rds
```

If you want aligned versions in a lockfile, pin both packages to the same observed line:

```bash
python -m pip install "boto3==1.42.51" "mypy-boto3-rds==1.42.51"
```

The upstream docs also publish two broader alternatives:

```bash
python -m pip install "boto3-stubs[rds]"
python -m pip install "boto3-stubs-lite[rds]"
```

- `mypy-boto3-rds`: service-only RDS stubs
- `boto3-stubs[rds]`: includes session overloads so IDEs can infer `Session().client("rds")` without extra annotations
- `boto3-stubs-lite[rds]`: smaller install, but upstream notes that it does not provide `session.client/resource` overloads

Other package managers:

```bash
uv add boto3 mypy-boto3-rds
poetry add boto3 mypy-boto3-rds
conda install mypy-boto3-rds -c conda-forge
```

## Setup

This package is usually a development dependency. Install it anywhere type checking or editor indexing runs.

If your deployed runtime omits stubs, keep imports behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_rds import RDSClient

session = Session(profile_name="dev", region_name="us-west-2")
rds: "RDSClient" = session.client("rds")
```

If the runtime environment includes the stubs package, regular imports are fine:

```python
from boto3.session import Session
from mypy_boto3_rds import RDSClient

rds: RDSClient = Session(region_name="us-west-2").client("rds")
```

## Authentication And Configuration

`mypy-boto3-rds` has no separate auth system. Use normal AWS SDK configuration:

- `AWS_PROFILE` for local profile selection
- `AWS_DEFAULT_REGION` or `region_name=...`
- standard AWS credential resolution from environment variables, shared config files, SSO, assume-role, or runtime IAM roles
- `botocore.config.Config(...)` for retries, timeouts, and other client settings

Practical setup pattern:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_rds import RDSClient

session = Session(profile_name="dev", region_name="us-east-1")
config = Config(
    retries={"mode": "standard", "max_attempts": 10},
    connect_timeout=5,
    read_timeout=60,
)

rds: RDSClient = session.client("rds", config=config)
```

## Core Usage

### Typed RDS Client

```python
from boto3.session import Session
from mypy_boto3_rds import RDSClient

rds: RDSClient = Session(region_name="us-east-1").client("rds")

response = rds.describe_db_instances(
    MaxRecords=20,
)

for db in response.get("DBInstances", []):
    print(db["DBInstanceIdentifier"], db.get("Engine"))
```

### Typed Paginator

Use paginator types when iterating over multi-page list operations:

```python
from boto3.session import Session
from mypy_boto3_rds import RDSClient
from mypy_boto3_rds.paginator import DescribeDBInstancesPaginator

rds: RDSClient = Session(region_name="us-east-1").client("rds")
paginator: DescribeDBInstancesPaginator = rds.get_paginator("describe_db_instances")

for page in paginator.paginate(DBInstanceIdentifier="app-db"):
    for db in page.get("DBInstances", []):
        print(db["DBInstanceIdentifier"])
```

### Typed Waiter

```python
from boto3.session import Session
from mypy_boto3_rds import RDSClient
from mypy_boto3_rds.waiter import DBInstanceAvailableWaiter

rds: RDSClient = Session(region_name="us-east-1").client("rds")
waiter: DBInstanceAvailableWaiter = rds.get_waiter("db_instance_available")

waiter.wait(
    DBInstanceIdentifier="app-db",
    WaiterConfig={"Delay": 30, "MaxAttempts": 40},
)
```

### Literal Types

The generated package exposes `Literal` aliases for fixed-value parameters and waiter or paginator names:

```python
from mypy_boto3_rds.literals import ActivityStreamModeType, DBInstanceAvailableWaiterName

mode: ActivityStreamModeType = "async"
waiter_name: DBInstanceAvailableWaiterName = "db_instance_available"
```

### Type Definitions

Use generated `TypedDict` shapes to keep helper payloads typed:

```python
from mypy_boto3_rds.type_defs import FilterTypeDef

engine_filter: FilterTypeDef = {
    "Name": "engine",
    "Values": ["postgres"],
}
```

These shapes are useful for request builders, wrapper utilities, and test fixtures.

## Editor And Type Checker Notes

- `mypy`, Pyright, Pylance, and other static analyzers consume the stubs automatically once they are installed.
- Upstream notes that `boto3-stubs[rds]` gives the best zero-annotation IDE experience in VS Code and similar editors.
- Upstream also warns that PyCharm can be slow on `Literal` overloads; if that happens, prefer `boto3-stubs-lite` or rely on `mypy` or `pyright` for checking.
- With standalone `mypy-boto3-rds`, explicit client annotations remain the safest way to keep `Session().client("rds")` typed.

Minimal `mypy` config:

```ini
[mypy]
python_version = 3.11
strict = True
```

## Common Pitfalls

- `mypy-boto3-rds` is stubs-only. It does not replace `boto3`, `botocore`, or AWS credentials.
- The PyPI name uses hyphens, but the import package is `mypy_boto3_rds`.
- Keep `boto3`, `botocore`, and the generated stubs on closely aligned version lines. Mismatches can show up as missing members or stale type errors.
- If you install `boto3-stubs-lite[rds]`, `Session.client("rds")` overloads are intentionally omitted upstream. Add explicit annotations instead of assuming inference will work.
- Guard stub imports with `TYPE_CHECKING` if your deployed runtime excludes dev-only dependencies.
- RDS has no service resource type in this package doc flow; treat the typed client as the primary entry point.
- Do not copy older boto3 examples that flatten everything to `dict[str, Any]`. The benefit of this package comes from typed clients, paginators, waiters, literals, and type defs propagating through call sites.

## Version-Sensitive Notes

- The package version and version used here both resolve cleanly to `1.42.51`.
- The docs URL is a generated latest site, not a version-pinned page. On `2026-03-11`, its install guidance already referenced local generation against `boto3==1.42.63`, which is newer than the published `mypy-boto3-rds==1.42.51` package.
- For reproducible builds, pin `boto3`, `botocore`, and `mypy-boto3-rds` to a tested set instead of assuming the latest docs page matches the currently published wheel.
- If you need stubs for a newer boto3 release before PyPI publishes the matching service package, upstream recommends generating them locally with `mypy-boto3-builder`.

## Official Sources Used

- Docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_rds/`
- PyPI project page: `https://pypi.org/project/mypy-boto3-rds/`
- Maintainer repository: `https://github.com/youtype/types-boto3`

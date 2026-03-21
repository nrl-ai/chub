---
name: mypy-boto3-qldb
description: "Type stubs for boto3 Amazon QLDB control-plane clients, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.40.54"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,qldb,boto3,mypy-boto3-qldb,boto3-stubs,typing,type-checking"
---

# mypy-boto3-qldb Python Package Guide

## Golden Rule

`mypy-boto3-qldb` is a stubs-only package for static typing. Keep using `boto3` for real AWS calls, and use this package only to type `Session().client("qldb")`, generated `TypedDict` shapes, and QLDB-specific literal values.

For most projects, install either:

- `boto3-stubs[qldb]` if you want `session.client("qldb")` overload inference in editors and type checkers
- `mypy-boto3-qldb` if you want the standalone service package and are willing to annotate `QLDBClient` explicitly

QLDB itself reached end of support on July 31, 2025, so this package is mainly for maintaining existing QLDB control-plane code rather than starting new systems.

## Install

Recommended for normal boto3 development:

```bash
python -m pip install "boto3==1.40.54" "boto3-stubs[qldb]==1.40.54"
```

Standalone service-specific stubs:

```bash
python -m pip install "boto3==1.40.54" "mypy-boto3-qldb==1.40.54"
```

Lower-memory editor fallback:

```bash
python -m pip install "boto3==1.40.54" "boto3-stubs-lite[qldb]==1.40.54"
```

Common alternatives:

```bash
uv add "boto3==1.40.54" "boto3-stubs[qldb]==1.40.54"
poetry add "boto3==1.40.54" "boto3-stubs[qldb]==1.40.54"
```

Notes:

- Keep the stubs version aligned with the `boto3` version you pin. The maintainer docs track these packages to the related boto3 release line.
- `boto3-stubs-lite[qldb]` is useful when IDE performance matters, but it does not provide the overloaded `session.client("qldb")` inference from the full `boto3-stubs` package.
- Installing only `mypy-boto3-qldb` does not install the runtime SDK.

## Authentication And Runtime Setup

`mypy-boto3-qldb` does not configure AWS credentials, regions, retries, or endpoints. Runtime behavior still comes from `boto3` and `botocore`.

Boto3's credential guide documents the normal provider chain, including:

1. Explicit parameters passed in code
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
3. Shared credentials and config files
4. Assume-role, container, or EC2 instance metadata providers

Practical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Or make the session explicit in code:

```python
from boto3.session import Session
from mypy_boto3_qldb import QLDBClient

session = Session(profile_name="dev", region_name="us-east-1")
qldb: QLDBClient = session.client("qldb")
```

Keep secrets out of source control. Prefer shared AWS config, IAM roles, IAM Identity Center, or other temporary-credential flows over hardcoded keys.

## Core Usage

### Typed control-plane client

This package types the QLDB management API exposed through `boto3.client("qldb")`. It does not add a `resource()` interface.

```python
from boto3.session import Session
from mypy_boto3_qldb import QLDBClient

client: QLDBClient = Session(region_name="us-east-1").client("qldb")

response = client.list_ledgers()

for ledger in response.get("Ledgers", []):
    print(ledger["Name"], ledger["State"])
```

### Typed request literals and response shapes

```python
from boto3.session import Session
from mypy_boto3_qldb import QLDBClient
from mypy_boto3_qldb.literals import PermissionsModeType
from mypy_boto3_qldb.type_defs import CreateLedgerResponseTypeDef

client: QLDBClient = Session(region_name="us-east-1").client("qldb")
permissions_mode: PermissionsModeType = "STANDARD"

result: CreateLedgerResponseTypeDef = client.create_ledger(
    Name="app-ledger",
    PermissionsMode=permissions_mode,
)

print(result["Name"], result["State"])
```

### Typed helper functions

```python
from boto3.session import Session
from mypy_boto3_qldb import QLDBClient
from mypy_boto3_qldb.type_defs import DescribeLedgerResponseTypeDef

def describe_ledger(name: str) -> DescribeLedgerResponseTypeDef:
    client: QLDBClient = Session(region_name="us-east-1").client("qldb")
    return client.describe_ledger(Name=name)
```

### `TYPE_CHECKING` pattern for dev-only stubs

If your production image does not install stub packages, keep stub imports out of runtime paths:

```python
from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_qldb import QLDBClient

def get_qldb_client() -> "QLDBClient":
    return Session(region_name="us-east-1").client("qldb")
```

## What This Package Does Not Cover

- It does not replace `boto3`.
- It does not execute or validate runtime requests by itself.
- It does not cover QLDB session or transaction-driver workflows used for ledger document operations.
- It does not provide `resource()`, waiter, or paginator helpers for this service surface.

AWS's QLDB developer guide distinguishes the control-plane management API from the data plane used for ledger sessions and PartiQL transactions. Use this package only for the management side exposed through `boto3.client("qldb")`.

## Common Pitfalls

### Hyphenated package name vs import name

Install with `mypy-boto3-qldb`, but import with underscores:

```python
from mypy_boto3_qldb import QLDBClient
```

### Expecting resource, waiter, or paginator modules

The generated maintainer docs for this package focus on `QLDBClient`, literals, and typed definitions. Do not assume there are service resource, waiter, or paginator modules just because other AWS service stub packages expose them.

### Treating the stubs package as the transaction SDK

Application code that works with ledger sessions and transactions uses different APIs than the control-plane operations typed here. Do not infer document-transaction support from this package alone.

### Forgetting version alignment

If `boto3`, `botocore`, and `mypy-boto3-qldb` drift too far apart, generated method signatures and typed shapes can stop matching your runtime SDK.

Safest rule:

- pin `boto3`
- pin `botocore`
- pin `mypy-boto3-qldb`
- update them together

## Version-Sensitive Notes

- The version used here `1.40.54` matched the official PyPI project page and generated maintainer docs on March 12, 2026.
- The package is tied to a now-retired AWS service. For new work, confirm that QLDB is still an intentional dependency before adding this package.
- If you need automatic client overload inference, prefer `boto3-stubs[qldb]` over the standalone service package.

## Official Sources

- PyPI project: https://pypi.org/project/mypy-boto3-qldb/
- PyPI JSON API: https://pypi.org/pypi/mypy-boto3-qldb/json
- Generated maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_qldb/
- Repository: https://github.com/youtype/types-boto3
- Boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- Amazon QLDB management and transaction API guide: https://docs.aws.amazon.com/qldb/latest/developerguide/api-reference.html
- Amazon QLDB end-of-support notice: https://docs.aws.amazon.com/qldb/latest/developerguide/what-is.html

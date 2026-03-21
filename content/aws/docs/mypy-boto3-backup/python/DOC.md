---
name: mypy-boto3-backup
description: "Type stubs for boto3 AWS Backup clients, paginators, literals, and TypedDicts in Python projects"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,backup,boto3,mypy,pyright,type-stubs,python"
---

# `mypy-boto3-backup` Python Package Guide

## Golden Rule

`mypy-boto3-backup` is a typing-only companion for `boto3`, not a runtime AWS SDK. Create real Backup clients with `boto3`, keep the stub package on the same release line as `boto3`, and add explicit `BackupClient` annotations unless you are intentionally relying on the full `boto3-stubs[backup]` overloads.

## What This Package Gives You

`mypy-boto3-backup` adds type information for the boto3 AWS Backup client surface:

- `BackupClient` for `Session.client("backup")`
- typed paginator classes for Backup list operations
- `literals` for enum-like string values
- `type_defs` for generated request and response `TypedDict` shapes

It does not perform AWS authentication, retries, endpoint resolution, or API calls by itself. Those still come from `boto3` and `botocore`.

## Install

Choose one installation path and keep the version aligned with your `boto3` line.

### Option 1: standalone service stubs

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-backup==1.42.3"
```

### Option 2: full `boto3-stubs` bundle

Use this when you want `Session.client("backup")` inference to work with minimal annotation noise.

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[backup]==1.42.3"
```

### Option 3: `boto3-stubs-lite`

Use this when IDE memory use matters more than automatic overload inference.

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[backup]==1.42.3"
```

Notes:

- `boto3-stubs-lite[backup]` keeps the service types, but upstream says it does not provide `session.client()` or `session.resource()` overloads.
- If the stubs are only needed for editor support and CI type checks, keep them in a development dependency group.

## Authentication And Setup

This package does not change AWS Backup authentication behavior. `boto3` still resolves credentials and region settings from the normal AWS provider chain:

1. Explicit `Session(...)` or `client(...)` parameters
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
3. Shared AWS config and credentials files such as `~/.aws/config` and `~/.aws/credentials`
4. Role-based providers such as assume-role, IAM Identity Center, web identity, container credentials, or instance metadata

Minimal local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

Typed client setup:

```python
from boto3.session import Session
from mypy_boto3_backup.client import BackupClient

session = Session(profile_name="dev", region_name="us-east-1")
backup: BackupClient = session.client("backup")
```

AWS Backup is regional. Set `region_name` deliberately, especially when working with backup vaults, restore jobs, or cross-account automation.

## Core Usage

### Type a normal Backup client

```python
from boto3.session import Session
from mypy_boto3_backup.client import BackupClient

backup: BackupClient = Session(region_name="us-east-1").client("backup")

response = backup.list_backup_jobs(ByState="COMPLETED", MaxResults=20)

for job in response.get("BackupJobs", []):
    print(job["BackupJobId"], job["State"], job.get("ResourceArn"))
```

With full `boto3-stubs[backup]`, the explicit `BackupClient` annotation is often optional. With standalone or lite installs, keep the annotation.

### Type paginators explicitly

AWS and the maintainer docs both expose many Backup paginators. Two common examples:

```python
from boto3.session import Session
from mypy_boto3_backup.client import BackupClient
from mypy_boto3_backup.paginator import (
    ListBackupJobsPaginator,
    ListRecoveryPointsByResourcePaginator,
)

backup: BackupClient = Session(region_name="us-east-1").client("backup")

jobs: ListBackupJobsPaginator = backup.get_paginator("list_backup_jobs")
for page in jobs.paginate(PaginationConfig={"PageSize": 100}):
    for job in page.get("BackupJobs", []):
        print(job["BackupJobId"])

recovery_points: ListRecoveryPointsByResourcePaginator = backup.get_paginator(
    "list_recovery_points_by_resource"
)
for page in recovery_points.paginate(
    ResourceArn="arn:aws:ec2:us-east-1:123456789012:volume/vol-0123456789abcdef0",
    PaginationConfig={"PageSize": 50},
):
    for point in page.get("RecoveryPoints", []):
        print(point["RecoveryPointArn"])
```

The Backup service reference currently lists paginator support for operations such as `list_backup_jobs`, `list_backup_plans`, `list_backup_vaults`, `list_copy_jobs`, `list_restore_jobs`, `list_scan_jobs`, and related resource-specific list calls.

### Use literals and TypedDicts in helper code

The generated `literals` and `type_defs` modules are useful when you want stricter checking around helper inputs and structured data.

```python
from mypy_boto3_backup.literals import AggregationPeriodType
from mypy_boto3_backup.type_defs import AdvancedBackupSettingOutputTypeDef

def normalize_period(value: AggregationPeriodType) -> AggregationPeriodType:
    return value

def build_setting() -> AdvancedBackupSettingOutputTypeDef:
    return {
        "ResourceType": "EC2",
    }
```

### Keep typing imports out of runtime-only environments

If the stubs are installed only in development or CI, use `TYPE_CHECKING` guards so production imports do not depend on `mypy-boto3-backup`.

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_backup.client import BackupClient
else:
    BackupClient = object

backup: BackupClient = Session(region_name="us-east-1").client("backup")
```

The PyPI project description notes this pattern is safe and specifically calls out a `pylint` workaround using `object` fallbacks.

## Service Shape Notes

- The current Backup boto3 reference exposes a low-level client with many `list_*`, `describe_*`, `start_*`, and `update_*` operations.
- The current Backup boto3 reference exposes paginators, but it does not expose a waiters section.
- The maintainer docs and PyPI package description for `mypy-boto3-backup` focus on the client, paginator, literals, and `type_defs` modules.

Do not assume `client.get_waiter(...)` or a typed service resource exists for Backup unless your exact installed boto3 and stub versions prove it.

## Common Pitfalls

- `mypy-boto3-backup` does not replace `boto3`; it only adds static types.
- The install name and import root differ: install `mypy-boto3-backup`, import from `mypy_boto3_backup`.
- `boto3-stubs-lite[backup]` is lighter because it drops the automatic `Session.client(...)` and `Session.resource(...)` overload behavior. Add explicit annotations in lite mode.
- Keep `boto3` and the stubs on the same release line when possible. Upstream versioning says the stub version tracks the related boto3 version.
- The maintainer docs site is a generated rolling reference. Use PyPI for exact installable version pins.
- AWS Backup authorization, cross-account access, vault policies, and restore semantics are runtime AWS concerns. The stubs help editors and type checkers, not permissions or service-side validation.
- The AWS Backup reference is newer than this pinned stub version, so verify any newly documented operations before assuming they exist in `1.42.3`.

## Version-Sensitive Notes For `1.42.3`

- The version used here `1.42.3` matches the current PyPI release checked on `2026-03-12`.
- PyPI states that `mypy-boto3-backup` requires Python `>=3.9` and is classified as `Stubs Only`.
- The PyPI project description says the package version matches the related `boto3` version.
- On `2026-03-12`, the live AWS Backup boto3 reference was on `1.42.66`, so the AWS runtime docs can be slightly ahead of the pinned stub package.
- The live AWS Backup service page exposes paginators but no waiters section. Treat waiter usage as unsupported unless you confirm it in your installed runtime and stubs.

## Official Sources

- PyPI package page: `https://pypi.org/project/mypy-boto3-backup/`
- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_backup/`
- AWS Backup boto3 reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/backup.html`
- AWS credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`

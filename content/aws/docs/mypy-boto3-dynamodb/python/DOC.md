---
name: mypy-boto3-dynamodb
description: "mypy-boto3-dynamodb package guide for typed boto3 DynamoDB clients, resources, paginators, waiters, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.55"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,dynamodb,boto3,mypy-boto3-dynamodb,boto3-stubs,typing,type-checking"
---

# mypy-boto3-dynamodb Python Package Guide

## Golden Rule

Use `boto3` for runtime AWS calls and use `mypy-boto3-dynamodb` only for typing.

If you want type inference without explicit annotations, install `boto3-stubs[dynamodb]`. If you install `mypy-boto3-dynamodb` or `boto3-stubs-lite[dynamodb]` by itself, you should expect to add explicit type annotations for clients, resources, paginators, and waiters.

## Install

### Recommended for normal boto3 code

Install the runtime SDK and the service stubs together:

```bash
python -m pip install "boto3==1.42.55" "boto3-stubs[dynamodb]==1.42.55"
```

This is the best default when you want `Session().client("dynamodb")` and `Session().resource("dynamodb")` to infer the correct types in editors and type checkers.

### Lower-memory option

```bash
python -m pip install "boto3==1.42.55" "boto3-stubs-lite[dynamodb]==1.42.55"
```

Use this when editor performance matters more than automatic overload discovery. The maintainer docs explicitly say the lite variant does not provide `session.client()` and `session.resource()` overloads, so explicit annotations become the normal workflow.

### Standalone package

```bash
python -m pip install "boto3==1.42.55" "mypy-boto3-dynamodb==1.42.55"
```

Use the standalone package when you only want the DynamoDB typing package or when you prefer guarding typing imports behind `TYPE_CHECKING`.

## Setup And Authentication

This package does not authenticate to AWS and does not create clients by itself. Authentication, regions, retries, and endpoints still come from `boto3`.

Practical setup:

1. Create a `boto3.Session(...)` with an explicit profile and region when the environment is not obvious.
2. Let boto3 load credentials from the normal AWS chain unless you are intentionally passing temporary credentials.
3. Keep runtime config on the session or client, not in the stub package.

Minimal setup:

```python
import boto3

session = boto3.Session(profile_name="dev", region_name="us-west-2")
dynamodb = session.client("dynamodb")
```

Important credential sources from the boto3 guide:

- explicit credentials passed to `boto3.client(...)`
- explicit credentials passed to `boto3.Session(...)`
- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
- assume-role and web-identity configuration
- IAM Identity Center profiles
- shared config in `~/.aws/credentials` and `~/.aws/config`
- container or EC2 instance role credentials

For most local development, use a profile instead of hardcoding keys:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

## Core Usage

### Zero-annotation workflow with `boto3-stubs[dynamodb]`

With the full service extra installed, type discovery should work on ordinary boto3 code:

```python
from boto3.session import Session

session = Session(region_name="us-west-2")
client = session.client("dynamodb")
resource = session.resource("dynamodb")

tables = client.list_tables(Limit=10)
users_table = resource.Table("users")
```

This is the simplest setup for agents generating application code.

### Explicit client annotations

Use explicit annotations when you installed the standalone package or the lite package:

```python
from boto3.session import Session
from mypy_boto3_dynamodb import DynamoDBClient

session = Session(region_name="us-west-2")
client: DynamoDBClient = session.client("dynamodb")

response = client.list_tables(Limit=10)
print(response["TableNames"])
```

### Explicit paginator and waiter annotations

The package exposes typed paginators for `list_backups`, `list_tables`, `list_tags_of_resource`, `query`, and `scan`, plus typed waiters such as `table_exists` and `table_not_exists`.

```python
from boto3.session import Session
from mypy_boto3_dynamodb import DynamoDBClient
from mypy_boto3_dynamodb.paginator import QueryPaginator
from mypy_boto3_dynamodb.waiter import TableExistsWaiter

session = Session(region_name="us-west-2")
client: DynamoDBClient = session.client("dynamodb")

query_pages: QueryPaginator = client.get_paginator("query")
table_exists: TableExistsWaiter = client.get_waiter("table_exists")

table_exists.wait(TableName="users")
```

### Explicit service-resource annotations

```python
from boto3.session import Session
from mypy_boto3_dynamodb import DynamoDBServiceResource
from mypy_boto3_dynamodb.service_resource import Table

session = Session(region_name="us-west-2")
resource: DynamoDBServiceResource = session.resource("dynamodb")
users: Table = resource.Table("users")
```

### Use literals for constrained string values

```python
from mypy_boto3_dynamodb.literals import ApproximateCreationDateTimePrecisionType

def accepts_precision(value: ApproximateCreationDateTimePrecisionType) -> str:
    return value
```

### Use `type_defs` when you want typed request or response fragments

```python
from mypy_boto3_dynamodb.type_defs import ArchivalSummaryTypeDef

def handle_summary(summary: ArchivalSummaryTypeDef) -> None:
    print(summary)
```

The upstream docs also expose service-specific `TypedDict` definitions and unions under `mypy_boto3_dynamodb.type_defs` for building helper functions and validating request-shape assumptions in typed code.

## `TYPE_CHECKING` Pattern

The maintainer docs explicitly say it is safe to keep this dependency out of production by importing it only for type checking.

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBClient
else:
    DynamoDBClient = object

session = Session(region_name="us-west-2")
client: "DynamoDBClient" = session.client("dynamodb")
```

Use this pattern when your production image should ship only `boto3`. The `object` fallback also avoids the pylint undefined-name problem called out in the upstream docs.

## Common Pitfalls

- `mypy-boto3-dynamodb` is stub-only. It does not replace `boto3`, and it does not give you a runtime DynamoDB client by itself.
- `boto3-stubs[dynamodb]` and `mypy-boto3-dynamodb` are not equivalent ergonomically. Full `boto3-stubs` gives auto-discovered client and resource overloads; standalone and lite installs usually require explicit annotations.
- Keep the `boto3` version aligned with the stub version. The maintainer docs say the stub package version matches the related `boto3` version.
- Do not put AWS credentials in code examples generated for real projects. Keep auth in profiles, environment variables, IAM Identity Center, or runtime IAM roles.
- If editor performance is poor in PyCharm, the maintainer recommends `boto3-stubs-lite` because `Literal` overloads can be slow there.
- Typed resource code can still be functionally wrong at runtime if the region, table name, or credentials are wrong. The stubs only validate shape information.
- AWS behavior docs and package release numbers are separate sources of truth. Use PyPI for the exact package version and AWS docs for credential-chain and runtime behavior.

## Version-Sensitive Notes For `1.42.55`

- PyPI lists `1.42.55` as the latest `mypy-boto3-dynamodb` release on March 12, 2026, published on February 23, 2026.
- The package description says it provides type annotations for `boto3 DynamoDB 1.42.55`, and the maintainer docs say the package version follows the related `boto3` version.
- The PyPI release history for this package skips many intermediate patch numbers. Do not assume that every `boto3` patch release has a matching standalone `mypy-boto3-dynamodb` upload.
- The AWS boto3 docs are currently on a newer rolling patch line (`1.42.66` in the docs tree), so pin against PyPI when exact package compatibility matters.
- This package was generated with `mypy-boto3-builder 8.12.0`, which matters when comparing generated names or availability across older blog posts and code snippets.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_dynamodb/
- PyPI package page: https://pypi.org/project/mypy-boto3-dynamodb/
- PyPI JSON metadata: https://pypi.org/pypi/mypy-boto3-dynamodb/json
- AWS boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html

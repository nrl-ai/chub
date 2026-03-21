---
name: mypy-boto3-athena
description: "mypy-boto3-athena type stubs for boto3 Athena clients, paginators, TypedDict request shapes, and editor/type-checker setup"
metadata:
  languages: "python"
  versions: "1.42.43"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,athena,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-athena Python Package Guide

## What This Package Is For

`mypy-boto3-athena` is a stub-only package for `boto3` Athena clients. It improves autocomplete and static type checking in tools such as mypy, pyright, Pylance, and PyCharm, but it does not make AWS calls by itself. Runtime behavior still comes from `boto3` and botocore.

Use it when your code already calls Athena through `boto3`, and you want typed clients, paginator types, generated `TypedDict` request and response shapes, and service literals.

## Golden Rule

- Install `boto3` for runtime behavior. `mypy-boto3-athena` only adds types.
- Treat Athena as a client-first boto3 service. The AWS Athena boto3 reference documents a low-level client plus paginators.
- Keep `boto3` and the Athena stubs on the same release line when request and response shapes matter.
- Prefer `boto3-stubs[athena]` for automatic client inference; use explicit annotations with the standalone or `lite` packages.

## Install

### Recommended for most projects

```bash
python -m pip install "boto3==1.42.43" "boto3-stubs[athena]==1.42.43"
```

This keeps runtime and typing packages aligned and usually gives automatic `Session().client("athena")` inference in editors and type checkers.

### Standalone Athena stubs

```bash
python -m pip install "boto3==1.42.43" "mypy-boto3-athena==1.42.43"
```

Use this when you want only the Athena service stubs instead of the `boto3-stubs` extras package.

### Lower-memory PyCharm fallback

```bash
python -m pip install "boto3==1.42.43" "boto3-stubs-lite[athena]==1.42.43"
```

The maintainer docs note that `boto3-stubs-lite[athena]` is more RAM-friendly, but it does not provide `session.client(...)` and `session.resource(...)` overload inference. In lite mode, annotate clients and paginators explicitly.

### Generate exact stubs locally

If you need stubs generated against the exact `boto3` version in your environment, the maintainer docs recommend local generation:

```bash
uvx --with "boto3==1.42.43" mypy-boto3-builder
```

The public docs root is latest-only, so this is the safest option when exact patch-level parity matters.

## Setup, Authentication, And Configuration

`mypy-boto3-athena` has no separate auth or config layer. Credentials, regions, retries, profiles, endpoints, proxies, and timeouts still come from normal `boto3` and botocore configuration.

Typical local setup:

```bash
aws configure
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

The boto3 credentials guide documents the main provider order agents usually need to remember:

1. Credentials passed to `boto3.client(...)`
2. Credentials passed to `boto3.Session(...)`
3. Environment variables
4. Assume-role providers
5. Web identity providers
6. Shared config and credentials files
7. Container or EC2 instance credentials

For client-specific behavior, pass a botocore `Config` object:

```python
from boto3.session import Session
from botocore.config import Config

session = Session(profile_name="dev", region_name="us-east-1")
athena = session.client(
    "athena",
    config=Config(
        retries={"mode": "standard", "max_attempts": 5},
        read_timeout=60,
    ),
)
```

Useful environment variables from the boto3 configuration guide:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_MAX_ATTEMPTS`
- `AWS_RETRY_MODE`

If credentials, IAM permissions, workgroup settings, or region selection are wrong, type checking still passes and the real AWS call still fails.

## Core Usage

### Typed client

Annotate the boto3 client explicitly when you use the standalone package or `lite` package:

```python
from boto3.session import Session
from mypy_boto3_athena.client import AthenaClient

def make_athena_client() -> AthenaClient:
    session = Session(profile_name="dev", region_name="us-east-1")
    return session.client("athena")
```

This gives typed access to operations such as `start_query_execution`, `get_query_execution`, `get_query_results`, `list_query_executions`, and `stop_query_execution`.

### Typed request shapes with `TypedDict`

Use generated `type_defs` when building Athena request payloads across helper functions or config layers:

```python
from boto3.session import Session
from mypy_boto3_athena.client import AthenaClient
from mypy_boto3_athena.type_defs import (
    QueryExecutionContextTypeDef,
    ResultConfigurationTypeDef,
    StartQueryExecutionOutputTypeDef,
)

athena: AthenaClient = Session(region_name="us-east-1").client("athena")

context: QueryExecutionContextTypeDef = {
    "Database": "analytics",
}
result_config: ResultConfigurationTypeDef = {
    "OutputLocation": "s3://my-query-results/athena/",
}

response: StartQueryExecutionOutputTypeDef = athena.start_query_execution(
    QueryString="SELECT 1",
    QueryExecutionContext=context,
    ResultConfiguration=result_config,
    WorkGroup="primary",
)

print(response["QueryExecutionId"])
```

This catches misspelled keys and wrong nested shapes before runtime.

### Typed paginator usage

The AWS Athena reference exposes paginators for `get_query_results`, `list_data_catalogs`, `list_databases`, `list_named_queries`, `list_query_executions`, `list_table_metadata`, and `list_tags_for_resource`. The stub package exposes matching paginator types.

```python
from boto3.session import Session
from mypy_boto3_athena.client import AthenaClient
from mypy_boto3_athena.paginator import ListQueryExecutionsPaginator

client: AthenaClient = Session(region_name="us-east-1").client("athena")
paginator: ListQueryExecutionsPaginator = client.get_paginator(
    "list_query_executions"
)

for page in paginator.paginate(WorkGroup="primary"):
    for query_id in page.get("QueryExecutionIds", []):
        print(query_id)
```

### Literals for constrained Athena values

```python
from mypy_boto3_athena.literals import QueryExecutionStateType

def is_terminal(state: QueryExecutionStateType) -> bool:
    return state in {"SUCCEEDED", "FAILED", "CANCELLED"}
```

Use literals instead of unchecked string constants when you branch on Athena states or enum-like parameters.

## Tooling Patterns

### Keep typing-only imports behind `TYPE_CHECKING`

If the stub package is installed only in development or CI, keep the imports out of runtime-only paths:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_athena.client import AthenaClient

def get_client() -> "AthenaClient":
    return Session(region_name="us-east-1").client("athena")
```

### When explicit annotations are necessary

- With `boto3-stubs[athena]`, editors usually infer `session.client("athena")` automatically.
- With `mypy-boto3-athena` standalone or `boto3-stubs-lite[athena]`, annotate `AthenaClient` and paginator variables explicitly.

## Common Pitfalls

- Installing only `mypy-boto3-athena` and expecting runtime Athena calls to work. You still need `boto3`.
- Treating the package as an auth, retry, or endpoint layer. Those settings live in `boto3` and botocore.
- Assuming successful typing proves that your result bucket, IAM permissions, workgroup, and region are valid.
- Copying docs-root examples as if they are frozen to your exact PyPI version. The docs site tracks the latest generated output.
- Expecting `boto3-stubs-lite[athena]` to preserve automatic `session.client("athena")` overload inference.
- Writing `boto3.resource("athena")` patterns by analogy with S3 or DynamoDB. Athena is documented here as a low-level client surface with paginators.

## Version-Sensitive Notes

- This entry is pinned to version used here `1.42.43`.
- PyPI shows `mypy-boto3-athena 1.42.43` was released on `2026-02-05` and requires Python `>=3.9`.
- The maintainer docs root is not frozen to `1.42.43`; when checked on `2026-03-12`, its local-generation example referenced `boto3==1.42.66`.
- If exact stub parity matters, pin `boto3` and the stubs together or generate them locally for the exact boto3 version you ship.

## Official Sources

- Docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_athena/
- Client reference: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_athena/client/
- Paginators reference: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_athena/paginators/
- Registry page: https://pypi.org/project/mypy-boto3-athena/
- Boto3 Athena reference: https://docs.aws.amazon.com/boto3/latest/reference/services/athena.html
- Boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html
- Boto3 configuration guide: https://docs.aws.amazon.com/boto3/latest/guide/configuration.html

---
name: mypy-boto3-opensearch
description: "Type stubs for boto3 OpenSearch Service clients, paginators, literals, and TypedDicts in Python"
metadata:
  languages: "python"
  versions: "1.42.64"
  revision: 3
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,opensearch,type-stubs,mypy,pyright,python"
---

# mypy-boto3-opensearch Python Package Guide

## Golden Rule

`mypy-boto3-opensearch` is a stubs-only package for static typing. It does not replace `boto3`, it does not create AWS clients by itself, and it does not handle credentials or retries. Use it to type `boto3` OpenSearch Service clients, paginator calls, literal values, and request or response `TypedDict` shapes.

## Install

For the standalone service package:

```bash
python -m pip install "boto3==1.42.64" "mypy-boto3-opensearch==1.42.64"
```

If you prefer the umbrella stubs distribution with an OpenSearch extra:

```bash
python -m pip install "boto3-stubs[opensearch]==1.42.64"
```

If you need a lower-overhead typing install:

```bash
python -m pip install "boto3-stubs-lite[opensearch]==1.42.64"
```

Practical install notes:

- `mypy-boto3-opensearch` is useful when you want only this service's stubs.
- `boto3-stubs[opensearch]` adds boto3 overloads so `boto3.client("opensearch")` and `Session.client("opensearch")` infer better.
- `boto3-stubs-lite[opensearch]` reduces type-checker overhead, but it omits those overloaded helpers, so explicit annotations matter more.
- When exact request and response shapes matter, pin `boto3` and the stubs package to the same release line.

## Authentication And Setup

This package uses normal boto3 configuration. Set credentials and region exactly as you would for any other boto3 client.

Common boto3 credential sources:

1. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
2. Shared config and credentials files under `~/.aws/config` and `~/.aws/credentials`
3. A named profile selected with `AWS_PROFILE` or `Session(profile_name=...)`
4. IAM role credentials on AWS infrastructure such as EC2, ECS, or Lambda

Typical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_opensearch.client import OpenSearchServiceClient

session = Session(profile_name="dev", region_name="us-west-2")

client: OpenSearchServiceClient = session.client(
    "opensearch",
    config=Config(
        retries={"mode": "standard", "max_attempts": 10},
    ),
)
```

If you prefer the top-level import, the package also exposes `OpenSearchServiceClient` from `mypy_boto3_opensearch`.

## Core Usage

### Typed Client Calls

Use the generated client type in helper functions instead of `Any`:

```python
from mypy_boto3_opensearch.client import OpenSearchServiceClient

def list_domains(client: OpenSearchServiceClient) -> list[str]:
    response = client.list_domain_names()
    return [item["DomainName"] for item in response.get("DomainNames", [])]
```

### Typed Request And Response Shapes

Import `type_defs` when you want stricter checking for AWS-shaped dictionaries:

```python
from mypy_boto3_opensearch.client import OpenSearchServiceClient
from mypy_boto3_opensearch.type_defs import DescribeDomainResponseTypeDef

def get_domain_arn(
    client: OpenSearchServiceClient,
    domain_name: str,
) -> str:
    response: DescribeDomainResponseTypeDef = client.describe_domain(
        DomainName=domain_name
    )
    return response["DomainStatus"]["ARN"]
```

That pattern is useful when you pass a response object deeper into your application and want type checking on nested fields such as `DomainStatus`.

### Typed Paginators

Paginator classes are generated for supported OpenSearch operations:

```python
from mypy_boto3_opensearch.client import OpenSearchServiceClient
from mypy_boto3_opensearch.paginator import ListApplicationsPaginator

def list_application_ids(client: OpenSearchServiceClient) -> list[str]:
    paginator: ListApplicationsPaginator = client.get_paginator("list_applications")
    application_ids: list[str] = []

    for page in paginator.paginate():
        application_ids.extend(
            item["Id"] for item in page.get("ApplicationSummaries", [])
        )

    return application_ids
```

### Literals For Enum-Like Strings

Generated literal types live under `mypy_boto3_opensearch.literals`:

```python
from mypy_boto3_opensearch.literals import DomainStateType

def record_state(state: DomainStateType) -> None:
    print(state)
```

Use these when you want type-checker validation for enum-like AWS string values instead of untyped `str`.

## Tooling Patterns

- Install the stubs in the same environment your editor, `mypy`, or `pyright` analyzes.
- Keep runtime settings such as retries, timeouts, regions, custom endpoints, and credentials on the `boto3` session or client.
- If you keep stubs as a dev-only dependency, gate imports with `TYPE_CHECKING`.

Example `TYPE_CHECKING` pattern:

```python
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from mypy_boto3_opensearch.client import OpenSearchServiceClient

def takes_client(client: "OpenSearchServiceClient") -> None:
    print(client.meta.service_model.service_name)
```

## Common Pitfalls

- Install `mypy-boto3-opensearch`, but import `mypy_boto3_opensearch`.
- `mypy-boto3-opensearch` does not replace `boto3`; your app still needs the runtime SDK.
- `boto3-stubs-lite[opensearch]` drops overloaded `client(...)` helpers, so unannotated client creation is less precise.
- Credentials, region, IAM permissions, custom endpoints, and retry behavior are runtime boto3 concerns, not problems this package solves.
- The maintainer docs site is rolling. Its generated install snippets can move ahead of the exact version currently published on PyPI.

## Version-Sensitive Notes

- This guide tracks `mypy-boto3-opensearch==1.42.64`, which PyPI listed as the published package version when this entry was revised.
- PyPI lists `Requires: Python >=3.9` for this release line.
- The hosted maintainer docs are generated from a rolling source and may already show newer `boto3` or `boto3-stubs` patch versions in install examples.
- Practical rule: when you want the closest type alignment, pin `boto3` and `mypy-boto3-opensearch` together on the same release line.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_opensearch/`
- Maintainer client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_opensearch/client/`
- Maintainer paginator reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_opensearch/paginators/`
- Maintainer type definitions: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_opensearch/type_defs/`
- Package page: `https://pypi.org/project/mypy-boto3-opensearch/`
- boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- boto3 OpenSearch reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/opensearch.html`

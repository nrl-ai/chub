---
name: mypy-boto3-personalize
description: "mypy-boto3-personalize package guide for typed Amazon Personalize boto3 clients, paginators, literals, and TypedDicts"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,mypy,pyright,typing,stubs,personalize,python"
---

# mypy-boto3-personalize Python Package Guide

## What It Is

`mypy-boto3-personalize` is the generated type-stub package for the Amazon Personalize `boto3` client. Use it to add static typing and editor autocomplete for:

- `Session.client("personalize")` client annotations
- paginator names and paginator object types
- generated literals and `TypedDict` request/response shapes

It does **not** replace `boto3` at runtime. Install and use the real AWS SDK packages for credentials, retries, and API calls.

## Install

Use one of these install patterns:

```bash
python -m pip install boto3 mypy-boto3-personalize
```

If you already use the umbrella stubs package, install the Amazon Personalize extra:

```bash
python -m pip install boto3-stubs[personalize]
```

If IDE performance matters more than automatic `Session.client(...)` overload inference, the project also publishes a lighter variant:

```bash
python -m pip install boto3-stubs-lite[personalize]
```

Use `boto3-stubs-lite` only if you are comfortable adding explicit annotations yourself. The upstream docs note that the lite package omits `Session.client` and `Session.resource` overloads to reduce package size.

## Initialize A Typed Client

Create the runtime client with `boto3`, then annotate it with the generated client type from the stubs package.

```python
from boto3.session import Session
from mypy_boto3_personalize.client import PersonalizeClient

session = Session(profile_name="dev", region_name="us-west-2")
personalize: PersonalizeClient = session.client("personalize")
```

The service name is exactly `"personalize"`, matching the AWS boto3 service reference.

If you keep stub packages out of production images, gate the import behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_personalize.client import PersonalizeClient

session = Session(region_name="us-west-2")
personalize: "PersonalizeClient" = session.client("personalize")
```

## Core Usage

### Typed Client Calls

The stubs keep the normal boto3 calling pattern, but the return values and parameters become discoverable to the type checker.

```python
from boto3.session import Session
from mypy_boto3_personalize.client import PersonalizeClient

session = Session(region_name="us-west-2")
personalize: PersonalizeClient = session.client("personalize")

response = personalize.list_dataset_groups()

for summary in response.get("datasetGroups", []):
    print(summary["name"], summary["datasetGroupArn"])
```

### Typed Paginators

Amazon Personalize exposes paginators for several `list_*` operations. Annotate the paginator object when you want strong typing across pagination loops.

```python
from boto3.session import Session
from mypy_boto3_personalize.client import PersonalizeClient
from mypy_boto3_personalize.paginator import ListDatasetGroupsPaginator

session = Session(region_name="us-west-2")
personalize: PersonalizeClient = session.client("personalize")

paginator: ListDatasetGroupsPaginator = personalize.get_paginator("list_dataset_groups")

for page in paginator.paginate():
    for summary in page.get("datasetGroups", []):
        print(summary["name"])
```

### Literals And TypedDicts

- Import generated literals from `mypy_boto3_personalize.literals` when a parameter must come from a fixed string set.
- Import generated request/response `TypedDict`s from `mypy_boto3_personalize.type_defs` when annotating helper functions or adapter layers.
- Use the generated docs pages to look up the exact symbol names before hardcoding them in shared libraries.

## Auth And Configuration

Authentication, region selection, retry behavior, and credential discovery all come from `boto3`, not from the stubs package.

Preferred setup:

1. Configure credentials with AWS profiles, IAM Identity Center, assume-role config, or runtime IAM roles.
2. Set the region explicitly on `Session(...)` or through shared AWS config.
3. Use the same region as the Amazon Personalize resources you want to read or modify.

Common environment variables:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Example:

```python
from boto3.session import Session
from mypy_boto3_personalize.client import PersonalizeClient

session = Session(profile_name="dev", region_name="us-west-2")
personalize: PersonalizeClient = session.client("personalize")
```

See the AWS boto3 credentials guide for the full provider chain and profile behavior.

## Common Pitfalls

- This package is type information only. It does not create clients, send requests, or vend credentials.
- Use the control-plane service name `"personalize"`. `personalize-runtime` and `personalize-events` are separate AWS clients and need their own stub packages.
- Keep `boto3`, `botocore`, and `mypy-boto3-personalize` broadly aligned. Generated methods can drift as AWS updates service models.
- If you choose `boto3-stubs-lite`, expect to add explicit `PersonalizeClient` annotations because `Session.client(...)` overloads are intentionally omitted.
- Some lint stacks still report false positives against generated stub internals. Trust the type checker and upstream stubs package before rewriting working imports.
- Do not hardcode AWS credentials in source code. Let boto3 load them from environment variables, profiles, or runtime roles.

## Version-Sensitive Notes

- The initial package metadata and the official PyPI metadata both point to version `1.42.3`.
- The PyPI JSON metadata on March 12, 2026 reports `requires_python >=3.8`.
- The generated docs root is stable and useful for symbol lookup, but it is not versioned in the URL. Use PyPI metadata when you need exact patch-level provenance for dependency pinning.
- When editor hints and runtime behavior disagree, trust the runtime `boto3` and `botocore` versions actually installed in the environment and update the stubs package to match.

## Official Sources

- PyPI project page: https://pypi.org/project/mypy-boto3-personalize/
- PyPI JSON API: https://pypi.org/pypi/mypy-boto3-personalize/json
- Maintainer docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_personalize/
- Usage page: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_personalize/usage/
- Package source code page: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_personalize/package/
- AWS boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- AWS Personalize service reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/personalize.html

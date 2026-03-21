---
name: mypy-boto3-bedrock
description: "mypy-boto3-bedrock package guide for typed boto3 Bedrock clients, paginators, literals, and type definitions"
metadata:
  languages: "python"
  versions: "1.42.63"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,bedrock,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-bedrock Python Package Guide

## What It Is

`mypy-boto3-bedrock` provides generated type annotations for the AWS Bedrock control-plane client in `boto3`.

Use it when you want:

- typed `Session.client("bedrock")`
- typed paginator objects for Bedrock listing operations
- typed request and response dictionaries from `mypy_boto3_bedrock.type_defs`
- better autocomplete and mypy or pyright coverage for Bedrock administration code

It does not replace `boto3` at runtime. Real AWS calls still go through `boto3` and `botocore`.

## Golden Rule

- Install `boto3` for runtime behavior.
- Install either `boto3-stubs[bedrock]` or the standalone `mypy-boto3-bedrock` package for typing.
- Configure AWS credentials, region, retries, and profiles through normal `boto3` configuration; this package only adds types.

## Install

Install the runtime SDK first if it is not already present:

```bash
python -m pip install boto3
```

Then choose one typing strategy.

### Option A: `boto3-stubs` Extra

Use this when you want the standard umbrella package flow:

```bash
python -m pip install "boto3-stubs[bedrock]"
```

### Option B: Standalone Service Package

Use this when you want a narrower dependency just for Bedrock typing:

```bash
python -m pip install "boto3-stubs[essential]" "mypy-boto3-bedrock==1.42.63"
```

### IDE And Type Checker Notes

- The maintainers also document lite variants of the stubs. If full overload-heavy packages slow down your IDE, use the lite workflow and annotate the client explicitly.
- Keep `boto3`, `botocore`, and the stub package in the same release family whenever possible.

## Initialization And Setup

Create the runtime client with normal boto3 session setup, then add a `BedrockClient` annotation.

```python
from boto3.session import Session
from mypy_boto3_bedrock.client import BedrockClient

session = Session(profile_name="dev", region_name="us-east-1")
bedrock: BedrockClient = session.client("bedrock")
```

If you keep stubs as a development-only dependency, guard the type import with `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_bedrock.client import BedrockClient

def make_bedrock_client() -> "BedrockClient":
    return boto3.client("bedrock", region_name="us-east-1")
```

This keeps the explicit type available to mypy and pyright without making the stub import part of normal runtime execution.

## Authentication And Region

`mypy-boto3-bedrock` does not add a Bedrock-specific auth layer. Credential and region resolution work exactly as they do in `boto3`.

Common configuration sources:

- `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`
- `AWS_PROFILE` and `AWS_DEFAULT_REGION`
- shared config files in `~/.aws/config` and `~/.aws/credentials`
- IAM roles and task roles in AWS environments

Example:

```python
import os

from boto3.session import Session
from mypy_boto3_bedrock.client import BedrockClient

session = Session(
    profile_name=os.getenv("AWS_PROFILE"),
    region_name=os.getenv("AWS_DEFAULT_REGION", "us-east-1"),
)
bedrock: BedrockClient = session.client("bedrock")
```

If Bedrock requests fail because of credentials or region access, fix the underlying `boto3` or AWS configuration. The stub package does not change runtime behavior.

## Core Usage Patterns

### Typed Bedrock Client

Use the typed client as the default entry point for control-plane operations.

```python
from boto3.session import Session
from mypy_boto3_bedrock.client import BedrockClient

session = Session(region_name="us-east-1")
bedrock: BedrockClient = session.client("bedrock")

response = bedrock.list_foundation_models()

for summary in response.get("modelSummaries", []):
    print(summary["modelId"], summary.get("providerName"))
```

### Typed Paginator

The generated docs include paginator types for multi-page listing operations such as `list_custom_models`.

```python
from boto3.session import Session
from mypy_boto3_bedrock.client import BedrockClient
from mypy_boto3_bedrock.paginator import ListCustomModelsPaginator

session = Session(region_name="us-east-1")
bedrock: BedrockClient = session.client("bedrock")

paginator: ListCustomModelsPaginator = bedrock.get_paginator("list_custom_models")

for page in paginator.paginate(PaginationConfig={"PageSize": 25}):
    for model in page.get("modelSummaries", []):
        print(model["modelArn"], model.get("modelName"))
```

### Typed Request Dictionaries

Use `type_defs` when you want request construction to be checked before it reaches the client call.

```python
from mypy_boto3_bedrock.client import BedrockClient
from mypy_boto3_bedrock.type_defs import (
    ListCustomModelsRequestPaginateTypeDef,
    PaginatorConfigTypeDef,
)

pagination: PaginatorConfigTypeDef = {"PageSize": 25}
request: ListCustomModelsRequestPaginateTypeDef = {
    "PaginationConfig": pagination,
}

def iter_custom_model_pages(bedrock: BedrockClient) -> None:
    paginator = bedrock.get_paginator("list_custom_models")
    for page in paginator.paginate(**request):
        print(page.get("modelSummaries", []))
```

## What Upstream Modules Cover

The maintainer docs for this package focus on these import areas:

- `mypy_boto3_bedrock.client` for `BedrockClient`
- `mypy_boto3_bedrock.paginator` for paginator types
- `mypy_boto3_bedrock.type_defs` for generated request and response `TypedDict`s
- `mypy_boto3_bedrock.literals` for constrained literal values
- `mypy_boto3_bedrock` top-level exports for common typing shortcuts

Plan around typed clients and paginator helpers. The published docs for this service package do not emphasize boto3 resource-style usage.

## Common Pitfalls

- `mypy-boto3-bedrock` is not the Bedrock runtime client package. Runtime calls still come from `boto3`.
- Package name and import name differ. Install `mypy-boto3-bedrock`, import from `mypy_boto3_bedrock`.
- Service name is `"bedrock"` when constructing the client.
- Bedrock control-plane typing is separate from `bedrock-runtime`; do not import runtime client types from this package.
- If you use the lite stubs workflow, expect to annotate `BedrockClient` explicitly instead of relying on inference.
- Version skew between `boto3`, `botocore`, and the stubs can surface false-positive type errors or missing symbols when AWS adds new Bedrock operations or fields.

## Version-Sensitive Notes

- This doc is keyed to the version used here `1.42.63`, and the PyPI project page also showed `1.42.63` on `2026-03-12`.
- The maintainer docs page still embeds install snippets for an older `boto3-stubs[bedrock]==1.40.61` release family even though PyPI is newer.
- The current AWS boto3 Bedrock client docs already expose Bedrock operations such as `list_foundation_models` and paginator-backed listing APIs; if your project pins older boto3 or stubs, newer methods may not type-check cleanly.
- Re-check the official docs before copying type names into code if your project is pinned to a different boto3 release line.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_bedrock/`
- Maintainer examples page: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_bedrock/examples/`
- Maintainer type definitions page: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_bedrock/type_defs/`
- PyPI project: `https://pypi.org/project/mypy-boto3-bedrock/`
- AWS boto3 Bedrock client docs: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/bedrock.html`
- AWS boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`

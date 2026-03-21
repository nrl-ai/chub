---
name: mypy-boto3-textract
description: "Type annotations for boto3 Textract in Python, covering typed clients, paginators, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,textract,boto3,mypy-boto3-textract,boto3-stubs,typing,type-checking"
---

# mypy-boto3-textract Python Package Guide

## Golden Rule

Use `boto3` for runtime AWS calls and use `mypy-boto3-textract` only for typing.

For most projects, install `boto3-stubs[textract]` so `Session().client("textract")` is typed automatically. Use `mypy-boto3-textract` when you want only the Textract typing package or when you prefer explicit annotations behind `TYPE_CHECKING`.

## Install

### Recommended for normal boto3 code

Install the runtime SDK and the full service stubs together:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[textract]==1.42.3"
```

This is the best default when you want typed `Session().client("textract")` usage in VSCode, pyright, or mypy without adding explicit annotations everywhere.

### Lower-memory option

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[textract]==1.42.3"
```

The maintainer docs say the lite package does not provide `session.client()` and `session.resource()` overloads. Use it when editor performance matters more than automatic inference.

### Standalone service package

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-textract==1.42.3"
```

Use the standalone package when you only want Textract typing support installed or when you plan to guard stub imports behind `TYPE_CHECKING`.

## Setup And Authentication

This package does not configure AWS access. Credentials, region resolution, retries, endpoints, and HTTP behavior still come from `boto3`.

Practical setup:

1. Create a `boto3.Session(...)` with an explicit region.
2. Let boto3 resolve credentials from the normal AWS chain unless you are intentionally passing temporary credentials.
3. Keep runtime config on the session or client, not in the stubs package.

Minimal setup:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
textract = session.client("textract")
```

Important credential sources from the AWS boto3 guide:

- explicit credentials passed to `boto3.client(...)`
- explicit credentials passed to `boto3.Session(...)`
- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
- assume-role and web-identity providers
- AWS IAM Identity Center profiles
- shared credentials in `~/.aws/credentials`
- shared config in `~/.aws/config`
- container credentials
- EC2 instance metadata credentials

Useful local-development defaults:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

## Core Usage

### Zero-annotation workflow with `boto3-stubs[textract]`

With the full service extra installed, ordinary boto3 code should infer the correct client type:

```python
from boto3.session import Session

session = Session(region_name="us-east-1")
client = session.client("textract")

response = client.detect_document_text(
    Document={
        "S3Object": {
            "Bucket": "incoming-documents",
            "Name": "forms/example.pdf",
        }
    }
)

for block in response.get("Blocks", []):
    if block.get("BlockType") == "LINE":
        print(block.get("Text"))
```

### Explicit client annotations

Use explicit annotations when you installed `mypy-boto3-textract` or the lite package:

```python
from boto3.session import Session
from mypy_boto3_textract import TextractClient

session = Session(region_name="us-east-1")
client: TextractClient = session.client("textract")

response = client.analyze_document(
    Document={
        "S3Object": {
            "Bucket": "incoming-documents",
            "Name": "forms/example.pdf",
        }
    },
    FeatureTypes=["FORMS", "TABLES"],
)
```

AWS exposes Textract as a low-level client surface. The current boto3 Textract reference lists client methods and two paginators, not a higher-level resource API.

### Typed paginator annotations

The maintainer docs expose typed paginators for the two paginator operations AWS currently documents for Textract:

```python
from boto3.session import Session
from mypy_boto3_textract import TextractClient
from mypy_boto3_textract.paginator import ListAdaptersPaginator

session = Session(region_name="us-east-1")
client: TextractClient = session.client("textract")

paginator: ListAdaptersPaginator = client.get_paginator("list_adapters")

for page in paginator.paginate(MaxResults=20):
    for adapter in page.get("Adapters", []):
        print(adapter["AdapterName"])
```

### Literals and TypedDicts

Use literals for constrained string values and `type_defs` for request and response shapes:

```python
from boto3.session import Session
from mypy_boto3_textract import TextractClient
from mypy_boto3_textract.literals import FeatureTypeType
from mypy_boto3_textract.type_defs import (
    DocumentLocationTypeDef,
    StartDocumentAnalysisResponseTypeDef,
)

feature: FeatureTypeType = "FORMS"
document: DocumentLocationTypeDef = {
    "S3Object": {
        "Bucket": "incoming-documents",
        "Name": "batch/job-001.pdf",
    }
}

client: TextractClient = Session(region_name="us-east-1").client("textract")
response: StartDocumentAnalysisResponseTypeDef = client.start_document_analysis(
    DocumentLocation=document,
    FeatureTypes=[feature],
)

print(response["JobId"])
```

### `TYPE_CHECKING` pattern

The PyPI project description explicitly says it is safe to keep this dependency out of production by importing it only for type checking:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_textract import TextractClient
else:
    TextractClient = object

session = Session(region_name="us-east-1")
client: "TextractClient" = session.client("textract")
```

This also avoids the pylint undefined-name issue called out in the upstream docs.

## Configuration Notes

- Prefer an explicit `Session(region_name=...)` for Textract. Region mismatches are easy to hide when credentials load successfully but the service is not enabled where you expect.
- Keep retries and timeouts on the boto3 client config, not in stub-only imports.
- For large or asynchronous document workflows, use the `start_*` Textract operations and then poll the matching `get_*` operation with the returned job ID.
- For development environments that use AWS IAM Identity Center, let the AWS CLI manage login state and then construct a normal boto3 session with that profile.

## Common Pitfalls

- `mypy-boto3-textract` is stub-only. It does not replace `boto3`, and it does not create a runtime Textract client by itself.
- `boto3-stubs[textract]` and `mypy-boto3-textract` are not equivalent ergonomically. Full `boto3-stubs` gives the smoothest inference; standalone and lite installs usually require explicit annotations.
- Keep the `boto3` version aligned with the stubs version when you want predictable method signatures. PyPI states that `mypy-boto3-textract` uses the related boto3 version.
- Do not invent `resource("textract")` patterns. The official Textract boto3 reference documents the client interface and paginators, not a resource surface.
- `boto3-stubs-lite[textract]` is intentionally incomplete for overload inference. Expect to annotate `TextractClient` and paginators yourself.
- Type safety does not validate AWS permissions, region support, document size limits, or whether S3 objects actually exist.

## Version-Sensitive Notes For `1.42.3`

- PyPI lists `1.42.3` as the latest `mypy-boto3-textract` release on March 12, 2026, published on December 4, 2025.
- PyPI states that `mypy-boto3-textract` versioning follows the related `boto3` version.
- The hosted maintainer docs currently show a local generation example using `boto3==1.42.63`, even though the published package is `1.42.3`. Treat the installed package metadata and PyPI as the source of truth for exact pinning.
- The AWS Textract reference is a rolling `latest` docs surface and currently resolves to a newer boto3 patch line than this package version. Use AWS docs for runtime behavior and available operations, but pin compatibility from the published package version.

## Official Sources

- Maintainer docs: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_textract/
- PyPI package page: https://pypi.org/project/mypy-boto3-textract/
- AWS boto3 Textract reference: https://docs.aws.amazon.com/boto3/latest/reference/services/textract.html
- AWS boto3 credentials guide: https://docs.aws.amazon.com/boto3/latest/guide/credentials.html

---
name: mypy-boto3-comprehend
description: "Typed boto3 stubs for Amazon Comprehend clients, paginators, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,comprehend,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-comprehend Python Package Guide

## Golden Rule

`mypy-boto3-comprehend` is a stubs-only package for static typing of `boto3` Comprehend code. It does not send requests, load credentials, or replace the runtime SDK.

Use one of these install modes:

- `boto3-stubs[comprehend]` for the best editor experience with automatic `Session.client("comprehend")` overloads
- `mypy-boto3-comprehend` when you want only the Comprehend stubs and are willing to annotate clients and paginators explicitly
- `boto3-stubs-lite[comprehend]` when IDE memory use matters more than automatic overload inference

## Install

### Recommended for most projects

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[comprehend]==1.42.3"
```

This is the simplest path when you want `Session().client("comprehend")` to infer `ComprehendClient` automatically in editors and type checkers.

### Standalone Comprehend stubs

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-comprehend==1.42.3"
```

Use this when you want only the Comprehend typing package. In this mode, explicit type annotations are usually necessary.

### Lower-memory editor fallback

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[comprehend]==1.42.3"
```

The lite package is more memory-friendly, but the maintainer docs say it does not provide `session.client()` overloads. Expect to annotate types explicitly.

### Generate locally for an exact boto3 pin

```bash
uvx --with 'boto3==1.42.3' mypy-boto3-builder
```

This is useful when your project is pinned to a boto3 version and you want generated stubs that match that runtime release line as closely as possible.

## Runtime Setup And AWS Auth

`mypy-boto3-comprehend` has no package-specific initialization. All runtime behavior still comes from `boto3` and botocore.

Use the normal AWS credential chain and prefer an explicit session when the region or profile is not obvious:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-west-2")
client = session.client("comprehend")
```

Common configuration sources:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- shared config in `~/.aws/config` and `~/.aws/credentials`
- assume-role, IAM Identity Center, container, or instance metadata providers

Practical rule: keep credentials and retry config on the boto3 session or client. The stub package only improves static typing.

## Core Usage

### Typed client with automatic overloads

With `boto3-stubs[comprehend]`, standard boto3 client creation should infer the correct type:

```python
from boto3.session import Session

session = Session(region_name="us-east-1")
client = session.client("comprehend")

response = client.detect_sentiment(
    Text="The delivery was fast and the packaging was excellent.",
    LanguageCode="en",
)

print(response["Sentiment"])
```

### Explicit client annotation

Use explicit annotations when you installed the standalone or lite package:

```python
from boto3.session import Session
from mypy_boto3_comprehend.client import ComprehendClient

client: ComprehendClient = Session(region_name="us-east-1").client("comprehend")

response = client.contains_pii_entities(
    Text="Contact me at jane@example.com",
    LanguageCode="en",
)

print(response["Labels"])
```

### Typed paginator

Comprehend exposes typed paginator overloads for list operations such as `list_endpoints`, `list_entities_detection_jobs`, and `list_sentiment_detection_jobs`.

```python
from boto3.session import Session
from mypy_boto3_comprehend.client import ComprehendClient
from mypy_boto3_comprehend.paginator import ListEntitiesDetectionJobsPaginator

client: ComprehendClient = Session(region_name="us-east-1").client("comprehend")
paginator: ListEntitiesDetectionJobsPaginator = client.get_paginator(
    "list_entities_detection_jobs"
)

for page in paginator.paginate(PaginationConfig={"MaxItems": 25}):
    for props in page.get("EntitiesDetectionJobPropertiesList", []):
        print(props["JobId"], props["JobStatus"])
```

### Typed request and response fragments

`type_defs` are useful when wrapper code builds request dictionaries before calling the client:

```python
from boto3.session import Session
from mypy_boto3_comprehend.client import ComprehendClient
from mypy_boto3_comprehend.type_defs import ContainsPiiEntitiesRequestTypeDef

client: ComprehendClient = Session(region_name="us-east-1").client("comprehend")

request: ContainsPiiEntitiesRequestTypeDef = {
    "Text": "Call me at 206-555-0100",
    "LanguageCode": "en",
}

response = client.contains_pii_entities(**request)
print(response["Labels"])
```

### Literal-constrained values

The generated literals help when helpers should only accept valid enum-like values:

```python
from mypy_boto3_comprehend.literals import LanguageCodeType

def normalize_language(value: LanguageCodeType) -> LanguageCodeType:
    return value
```

### `TYPE_CHECKING` pattern for dev-only stubs

If production images do not install stub packages, keep the imports behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_comprehend.client import ComprehendClient
else:
    ComprehendClient = object

client: "ComprehendClient" = Session(region_name="us-east-1").client("comprehend")
```

The `object` fallback avoids runtime import failures and the common pylint undefined-name issue that the maintainer docs call out.

## Configuration Notes

- Amazon Comprehend is regional. Set `region_name` explicitly when the execution environment is not already pinned to the correct region.
- Comprehend request parameters still follow the AWS service API. The stubs help you catch missing or misspelled fields, but they do not validate IAM permissions, model availability, or regional service support.
- Batch and async job APIs often require S3 input and output locations plus IAM roles. Those runtime requirements come from AWS service behavior, not from the stub package.
- The generated `client.exceptions` surface is typed for editor completion, but the actual exception classes still come from the runtime boto3 client.

## Common Pitfalls

- Installing only `mypy-boto3-comprehend` and expecting unannotated `Session.client("comprehend")` calls to become typed automatically. That overload behavior comes from `boto3-stubs[comprehend]`.
- Forgetting to install `boto3`. The stub package does not ship the runtime AWS client.
- Treating static typing as runtime validation. Correct type hints do not guarantee that the selected region supports the model, that the IAM role has permission, or that S3 input and output locations are valid.
- Hardcoding AWS credentials in application code. Keep auth in profiles, environment variables, IAM Identity Center, or runtime IAM roles.
- Mixing unrelated AWS SDK families. `mypy-boto3-comprehend` is for synchronous `boto3`; if your code uses `aiobotocore`, use the matching `types-aiobotocore-comprehend` family instead.
- Assuming the maintainer docs are an exact frozen patch snapshot. They are a generated rolling docs site; use PyPI as the exact release source for frontmatter and pins.

## Version-Sensitive Notes

- PyPI lists `mypy-boto3-comprehend 1.42.3` as the current release on March 12, 2026, and the version used here matches this package release.
- The package description says it provides type annotations for `boto3 Comprehend 1.42.3`, so the practical default is to keep `boto3==1.42.3` and the stub package on the same release line.
- The package was generated with `mypy-boto3-builder 8.12.0`, which matters when comparing older examples or generated symbol names.
- The maintainer docs and AWS boto3 reference are rolling documentation sources. When exact patch compatibility matters, trust PyPI for the published package version and use the docs for API shape and usage patterns.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_comprehend/`
- Maintainer client reference: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_comprehend/client/`
- PyPI package page: `https://pypi.org/project/mypy-boto3-comprehend/`
- PyPI JSON metadata: `https://pypi.org/pypi/mypy-boto3-comprehend/json`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 Comprehend reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/comprehend.html`

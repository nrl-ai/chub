---
name: mypy-boto3-ses
description: "Typed boto3 stubs for Amazon SES clients, paginators, waiters, literals, and TypedDict request shapes in Python"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,boto3,ses,typing,mypy,pyright,stub"
---

# mypy-boto3-ses Python Package Guide

## Golden Rule

Use `mypy-boto3-ses` only for static typing around `boto3.client("ses")`.

- Keep `boto3` installed for runtime behavior, credentials, retries, and request execution.
- Keep the SES region explicit. Identity verification, sandbox status, and sending permissions are region-specific.
- If you install the standalone `mypy-boto3-ses` package or `boto3-stubs-lite[ses]`, add an explicit `SESClient` annotation. Only the full `boto3-stubs[ses]` package provides the overloaded `Session.client("ses")` typing automatically.

## Install

Pick one of these typing strategies:

### Full boto3 stubs

Recommended when you want typed `Session.client(...)` overloads across boto3 services:

```bash
python -m pip install "boto3-stubs[ses]"
```

### Lite boto3 stubs

Use this when the full package is too heavy for your editor or CI environment. Upstream notes that the lite package omits the overloads, so you must annotate clients explicitly:

```bash
python -m pip install "boto3-stubs-lite[ses]"
```

### Standalone SES stubs

Use this when you only want SES typing:

```bash
python -m pip install "mypy-boto3-ses==1.42.3"
```

Runtime dependency:

```bash
python -m pip install "boto3"
```

In real projects, keep the stub package in your dev or typing dependency group and keep `boto3` in the runtime dependency set.

## Authentication And Setup

This package does not load AWS credentials or regions. `boto3` still owns that behavior through the standard AWS provider chain:

- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
- shared config in `~/.aws/config` and `~/.aws/credentials`
- IAM Identity Center, assume-role config, ECS task roles, or EC2 instance roles

Minimal typed setup with an explicit session:

```python
from boto3.session import Session
from mypy_boto3_ses.client import SESClient

session = Session(profile_name="dev", region_name="us-east-1")
ses: SESClient = session.client("ses")
```

If the stubs are dev-only and you do not want to import them at runtime, a `TYPE_CHECKING` gate is a practical pattern:

```python
from typing import TYPE_CHECKING, cast

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_ses.client import SESClient

session = Session(region_name="us-east-1")
ses = cast("SESClient", session.client("ses"))
```

## Core Usage

### Send a typed SES email request

Use the generated `TypedDict` request shapes when you want the type checker to catch field-name mistakes before runtime:

```python
from boto3.session import Session
from mypy_boto3_ses.client import SESClient
from mypy_boto3_ses.type_defs import SendEmailRequestTypeDef

session = Session(region_name="us-east-1")
ses: SESClient = session.client("ses")

request: SendEmailRequestTypeDef = {
    "Source": "noreply@example.com",
    "Destination": {
        "ToAddresses": ["ada@example.com"],
    },
    "Message": {
        "Subject": {
            "Data": "Welcome",
            "Charset": "UTF-8",
        },
        "Body": {
            "Text": {
                "Data": "Your account is ready.",
                "Charset": "UTF-8",
            }
        },
    },
}

response = ses.send_email(**request)
print(response["MessageId"])
```

This package improves static checking only. SES will still reject the call at runtime if the sender identity is not verified in the configured region or the account is still in the SES sandbox.

### Use typed paginators

`mypy-boto3-ses` includes paginator classes for paginator-enabled SES operations:

```python
from boto3.session import Session
from mypy_boto3_ses.client import SESClient
from mypy_boto3_ses.paginator import ListIdentitiesPaginator

session = Session(region_name="us-east-1")
ses: SESClient = session.client("ses")

paginator: ListIdentitiesPaginator = ses.get_paginator("list_identities")

for page in paginator.paginate(IdentityType="EmailAddress"):
    for identity in page.get("Identities", []):
        print(identity)
```

### Use typed waiters

The package also exposes SES waiter types, which is safer than inventing sleep loops when the service model already defines the waiter:

```python
from boto3.session import Session
from mypy_boto3_ses.client import SESClient
from mypy_boto3_ses.waiter import IdentityExistsWaiter

session = Session(region_name="us-east-1")
ses: SESClient = session.client("ses")

waiter: IdentityExistsWaiter = ses.get_waiter("identity_exists")
waiter.wait(Identities=["noreply@example.com"])
```

## Configuration Notes

- This package types the classic SES client from `boto3.client("ses")`, not `boto3.client("sesv2")`. If your code uses the newer SES v2 API surface, use the matching `mypy-boto3-sesv2` stubs instead.
- The generated stubs include literals, paginators, waiters, and `type_defs` modules. They are useful for editor completion and mypy or pyright validation, but they do not change boto3 runtime semantics.
- The package docs are client-focused. There is no documented `SESServiceResource` surface in the package docs, so treat this package as a typed client companion rather than a resource-layer guide.

## Common Pitfalls

- Do not replace `boto3` with `mypy-boto3-ses`; the stub package is not a runtime AWS SDK.
- Do not expect client inference from `session.client("ses")` unless you installed full `boto3-stubs[ses]`. The standalone and lite variants need explicit annotations.
- Do not assume the stub package will always publish every boto3 patch. Upstream says the version matches the related boto3 version, but the standalone service package can lag the latest boto3 patch line in practice.
- Do not point this package at SES v2 code. `ses` and `sesv2` are different boto3 service names and have different generated stubs.
- Do not forget that SES account state is regional. A verified identity or production-access status in one region does not automatically carry over to another.

## Version-Sensitive Notes For `1.42.3`

- PyPI lists `mypy-boto3-ses 1.42.3` as the current published version checked on 2026-03-12.
- PyPI marks the package as typed stubs and requires Python `>=3.9`.
- The maintainer docs for this package describe three install paths: `boto3-stubs`, `boto3-stubs-lite`, and the standalone `mypy-boto3-ses` package. Choose based on whether you want automatic overloaded client typing or a smaller type-only install.
- The docs site is a rolling generated reference, so use PyPI for exact package pinning and the maintainer docs for the available typed surfaces and import paths.

## Official Sources

- Maintainer docs root: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_ses/
- Maintainer client reference: https://youtype.github.io/boto3_stubs_docs/mypy_boto3_ses/client/
- PyPI package page: https://pypi.org/project/mypy-boto3-ses/
- boto3 SES reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/ses.html
- boto3 credentials guide: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html
- Source repository: https://github.com/youtype/boto3-stubs

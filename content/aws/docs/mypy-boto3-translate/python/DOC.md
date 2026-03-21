---
name: mypy-boto3-translate
description: "mypy-boto3-translate package guide for typed boto3 AWS Translate clients, paginators, literals, and request/response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,translate,boto3,type-stubs,mypy,pyright,pylance,python"
---

# mypy-boto3-translate Python Package Guide

`mypy-boto3-translate` is a stubs-only package for the AWS Translate service in `boto3`. It improves type checking and editor completion for `Session().client("translate")`, paginator calls, literals, and typed request/response dictionaries. It does not make AWS API calls by itself.

## Golden Rule

- Keep using `boto3` for runtime AWS calls.
- Install the stubs alongside `boto3`, or use the bundled `boto3-stubs[translate]` extra.
- Prefer explicit client annotations when you install the standalone service package.
- Let `boto3` handle credentials and region resolution. These stubs only affect typing.

## Install

Install the runtime SDK and the service stubs together:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-translate==1.42.3"
```

Maintainer-recommended bundled install:

```bash
python -m pip install "boto3-stubs[translate]"
```

Lower-memory alternative:

```bash
python -m pip install "boto3-stubs-lite[translate]"
```

Use these options deliberately:

- `mypy-boto3-translate`: service-specific standalone stubs; explicit annotations are the safest pattern.
- `boto3-stubs[translate]`: best default when you want editor auto-discovery for `session.client("translate")`.
- `boto3-stubs-lite[translate]`: lighter on memory, but the maintainer docs note that it does not provide `session.client/resource` overloads and needs more explicit annotations.

## Core Setup

Create a normal boto3 session, then annotate the Translate client:

```python
from boto3.session import Session
from mypy_boto3_translate import TranslateClient

session = Session(profile_name="dev", region_name="us-east-1")
client: TranslateClient = session.client("translate")

response = client.translate_text(
    Text="Hello, world!",
    SourceLanguageCode="en",
    TargetLanguageCode="es",
)

print(response["TranslatedText"])
```

This is the default pattern to use in agent-generated code:

1. Create a `boto3.session.Session`.
2. Build `session.client("translate")`.
3. Annotate it as `TranslateClient`.
4. Keep using normal boto3 keyword arguments and response dictionaries.

## Typed Usage

### Client annotations

`TranslateClient` is the main type you will use:

```python
from boto3.session import Session
from mypy_boto3_translate import TranslateClient

def get_client() -> TranslateClient:
    return Session(region_name="us-east-1").client("translate")
```

This gives completions and type checking for Translate methods such as:

- `translate_text`
- `translate_document`
- `list_languages`
- `list_parallel_data`
- `list_terminologies`
- `list_text_translation_jobs`
- `start_text_translation_job`
- `describe_text_translation_job`
- `stop_text_translation_job`

### Paginators

The generated client docs expose paginator overloads for `list_terminologies`:

```python
from boto3.session import Session
from mypy_boto3_translate import TranslateClient
from mypy_boto3_translate.paginator import ListTerminologiesPaginator

client: TranslateClient = Session(region_name="us-east-1").client("translate")
paginator: ListTerminologiesPaginator = client.get_paginator("list_terminologies")

for page in paginator.paginate(MaxResults=20):
    for item in page.get("TerminologyPropertiesList", []):
        print(item["Name"])
```

### Literals

Use literals when a helper should only accept documented enum-like string values:

```python
from mypy_boto3_translate.literals import BrevityType

def set_brevity(value: BrevityType) -> BrevityType:
    return value
```

### Typed request and response shapes

Use generated `type_defs` for helper functions and tests that build Translate payloads:

```python
from mypy_boto3_translate.type_defs import TermTypeDef

def build_term(source_text: str, target_text: str) -> TermTypeDef:
    return {
        "SourceText": source_text,
        "TargetText": target_text,
    }
```

## Config And Authentication

`mypy-boto3-translate` does not add a separate auth layer. Authentication and region handling come from `boto3`.

According to the official boto3 credential guide, boto3 searches for credentials in this order:

1. Credentials passed directly to `boto3.client()`
2. Credentials passed when creating `Session(...)`
3. Environment variables
4. Assume-role providers
5. Web identity providers
6. IAM Identity Center
7. Shared credentials file
8. Console login credentials
9. AWS config file
10. Boto2 config
11. Container credential provider
12. EC2 instance metadata / IAM role

Practical local setup:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
```

```python
from boto3.session import Session
from mypy_boto3_translate import TranslateClient

session = Session(profile_name="dev", region_name="us-east-1")
client: TranslateClient = session.client("translate")
```

Prefer profiles, IAM roles, or short-lived credentials over hardcoded keys.

## TYPE_CHECKING Pattern

If you do not want to require the stub package in production images, gate imports behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_translate import TranslateClient
else:
    TranslateClient = object

def get_client() -> "TranslateClient":
    return Session(region_name="us-east-1").client("translate")
```

The maintainer docs call out this pattern as safe, and note that `pylint` may need the `object` fallback to avoid undefined-name complaints.

## Common Pitfalls

- `mypy-boto3-translate` is typing metadata only. Runtime calls still come from `boto3`.
- Install `boto3` as well. The stub package does not replace the SDK.
- Keep `boto3`, `botocore`, and the stubs aligned. A surprising type error often means version skew.
- Use `TranslateClient` annotations if editor completion does not appear automatically.
- Do not assume every AWS service has waiters or multiple paginators. For Translate, the generated client docs show `list_terminologies` as the paginator overload surfaced here.
- Prefer keyword arguments for boto3 calls. The generated stubs mirror named parameters and response shapes.
- When mocking AWS responses, use `type_defs` instead of anonymous `dict[str, Any]` where possible.
- If PyCharm becomes slow on large literal overloads, the maintainer docs recommend trying `boto3-stubs-lite`.

## Version-Sensitive Notes

- PyPI lists this package as `1.42.3`, released on `2025-12-04`.
- The maintainer states that `mypy-boto3-translate` version matches the related `boto3` version.
- The docs are generated from `mypy-boto3-builder 8.12.0`.
- The docs site is a generated documentation surface, so method lists and shapes track the published stub package rather than an AWS marketing guide.
- If a field or method matters for production code, verify the exact installed versions:

```bash
python -m pip show mypy-boto3-translate
python -m pip show boto3
python -m pip show botocore
```

## Official Sources

- PyPI package page: `https://pypi.org/project/mypy-boto3-translate/`
- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_translate/`
- Maintainer client docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_translate/client/`
- Builder repository: `https://github.com/youtype/mypy_boto3_builder`
- boto3 credential guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`

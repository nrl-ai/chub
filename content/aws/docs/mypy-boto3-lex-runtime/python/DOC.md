---
name: mypy-boto3-lex-runtime
description: "Typed boto3 stubs for Amazon Lex Runtime V1 clients, literals, and TypedDict request and response shapes in Python"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,lex,lex-runtime,boto3,mypy,pyright,type-checking,stubs"
---

# mypy-boto3-lex-runtime Python Package Guide

## Golden Rule

`mypy-boto3-lex-runtime` is a stubs-only package for `boto3.client("lex-runtime")`, not the runtime SDK. Install `boto3` as well, and treat this package as a legacy Amazon Lex V1 typing aid. AWS states that support for Amazon Lex V1 ended on September 15, 2025, so new work should target Lex V2 and `mypy-boto3-lexv2-runtime` instead.

## Install

Choose one install mode and keep the stubs aligned with the `boto3` version you pin.

### Recommended: full boto3 stubs

Use this when you want `Session().client("lex-runtime")` to infer `LexRuntimeServiceClient` automatically in editors and type checkers.

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[lex-runtime]==1.42.3"
```

### Standalone service package

Use this when you only want the Lex Runtime stubs and are fine with explicit annotations.

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-lex-runtime==1.42.3"
```

### Lite package

The maintainer docs note that the lite package is more RAM-friendly but does not provide `session.client/resource` overloads, so explicit client annotations are required.

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[lex-runtime]==1.42.3"
```

### Local generation for exact lockfile alignment

PyPI and the maintainer docs recommend local generation when you need the stubs to match an exact boto3 build.

```bash
uvx --with "boto3==1.42.3" mypy-boto3-builder
```

Select `boto3-stubs` and then add the `LexRuntimeService` service.

## Authentication And Runtime Setup

`mypy-boto3-lex-runtime` adds typing only. Credentials, regions, retries, and HTTP behavior still come from `boto3`.

AWS documents that boto3 searches for credentials in a standard order, starting with explicit client or session parameters, then environment variables, shared credentials files, config files, container credentials, and finally EC2 instance metadata.

Practical local setup:

```bash
aws configure --profile lex-dev
export AWS_DEFAULT_REGION="us-east-1"
```

Or export credentials directly:

```bash
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_SESSION_TOKEN="..."   # only when using temporary credentials
export AWS_DEFAULT_REGION="us-east-1"
```

Minimal typed client setup:

```python
from boto3.session import Session
from mypy_boto3_lex_runtime import LexRuntimeServiceClient

session = Session(profile_name="lex-dev", region_name="us-east-1")
client: LexRuntimeServiceClient = session.client("lex-runtime")
```

If you use the standalone or lite package, keep the explicit annotation even though the runtime object still comes from `boto3`.

## Core Usage

### Typed `post_text`

Lex Runtime V1 exposes these main boto3 client operations: `delete_session`, `get_session`, `post_content`, `post_text`, and `put_session`.

```python
from boto3.session import Session
from mypy_boto3_lex_runtime import LexRuntimeServiceClient
from mypy_boto3_lex_runtime.type_defs import (
    PostTextRequestRequestTypeDef,
    PostTextResponseTypeDef,
)

client: LexRuntimeServiceClient = Session(region_name="us-east-1").client("lex-runtime")

request: PostTextRequestRequestTypeDef = {
    "botName": "OrderPizzaBot",
    "botAlias": "prod",
    "userId": "user-123",
    "inputText": "I want a pizza",
}

response: PostTextResponseTypeDef = client.post_text(**request)
print(response.get("dialogState"))
print(response.get("message"))
```

Use a stable `userId` that does not contain personal data. AWS documents the `userId` field as required for runtime calls.

### Typed session inspection and mutation

```python
from boto3.session import Session
from mypy_boto3_lex_runtime import LexRuntimeServiceClient
from mypy_boto3_lex_runtime.type_defs import (
    GetSessionRequestRequestTypeDef,
    PutSessionRequestRequestTypeDef,
)

client: LexRuntimeServiceClient = Session(region_name="us-east-1").client("lex-runtime")

get_request: GetSessionRequestRequestTypeDef = {
    "botName": "OrderPizzaBot",
    "botAlias": "prod",
    "userId": "user-123",
}

session_state = client.get_session(**get_request)

put_request: PutSessionRequestRequestTypeDef = {
    "botName": "OrderPizzaBot",
    "botAlias": "prod",
    "userId": "user-123",
    "accept": "text/plain; charset=utf-8",
    "sessionAttributes": {
        "channel": "web",
        "tenant": "sandbox",
    },
}

client.put_session(**put_request)
```

`put_session` is the runtime call to seed or replace session state from your application. AWS documents `accept` as controlling whether the response is text or audio.

### Literals and TypedDict helpers

The stubs package also gives you service-specific literals and request or response shapes for your own helper functions.

```python
from mypy_boto3_lex_runtime.literals import ConfirmationStatusType
from mypy_boto3_lex_runtime.type_defs import ActiveContextTimeToLiveTypeDef

def normalize_status(status: ConfirmationStatusType) -> str:
    return status

def build_ttl(seconds: int, turns: int) -> ActiveContextTimeToLiveTypeDef:
    return {
        "timeToLiveInSeconds": seconds,
        "turnsToLive": turns,
    }
```

### `TYPE_CHECKING` guard for dev-only stubs

Use this pattern when stubs are installed only in development or CI environments.

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_lex_runtime import LexRuntimeServiceClient
else:
    LexRuntimeServiceClient = object

client: LexRuntimeServiceClient = boto3.client("lex-runtime", region_name="us-east-1")
```

## Configuration Notes

- The boto3 service string is exactly `lex-runtime`.
- This package types the Lex V1 runtime client. Lex V2 uses a different client name: `lexv2-runtime`.
- Prefer `Session(profile_name=..., region_name=...)` in application setup so region and credential selection stay explicit.
- Do not hard-code credentials. AWS documents environment variables, shared config, IAM Identity Center, container credentials, and instance metadata as supported options.
- If you use `boto3-stubs-lite[lex-runtime]` or the standalone package, write explicit return annotations for client factories because automatic overload inference is reduced.

## Common Pitfalls

- Do not treat `mypy-boto3-lex-runtime` as the runtime SDK. If `boto3` is missing, type checking may pass while runtime imports fail.
- Do not use `lex-runtime` for new chatbot development in 2026. AWS ended support for Amazon Lex V1 on September 15, 2025.
- Do not confuse Lex V1 runtime calls (`post_text`, `post_content`) with Lex V2 runtime calls (`recognize_text`, `recognize_utterance`). They use different clients and request shapes.
- Do not rely on implicit type inference when using `boto3-stubs-lite[lex-runtime]` or `mypy-boto3-lex-runtime` alone. Add `LexRuntimeServiceClient` annotations explicitly.
- Do not drift `boto3` and stubs versions casually. The maintainer docs say the package version matches the related `boto3` version.
- Do not put personally identifiable information into `userId`. AWS documents that field as required and intended only as a conversation identifier.
- Do not depend on the deprecated `message` header behavior from `put_session`; AWS documents `encodedMessage` as the safer response field for locales where `message` may be null.

## Version-Sensitive Notes

- PyPI currently lists `mypy-boto3-lex-runtime 1.42.3` as the latest release, published on December 4, 2025.
- The version used here, `1.42.3`, matches the live official PyPI release as of March 12, 2026.
- The maintainer docs state that `mypy-boto3-lex-runtime` uses the related `boto3` version number. If you upgrade boto3 for this legacy integration, upgrade the stubs package in the same change.
- AWS documentation now treats Amazon Lex V1 as end-of-support. On March 12, 2026, this package is mainly useful for maintaining or migrating legacy Lex V1 code, not for greenfield bot implementations.

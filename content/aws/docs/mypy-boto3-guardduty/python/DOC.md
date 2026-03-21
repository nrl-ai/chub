---
name: mypy-boto3-guardduty
description: "mypy-boto3-guardduty typed boto3 stubs for Amazon GuardDuty clients, paginators, literals, and TypedDict shapes"
metadata:
  languages: "python"
  versions: "1.42.33"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,guardduty,boto3,python,typing,mypy,pyright,stubs"
---

# mypy-boto3-guardduty Python Package Guide

## What It Is

`mypy-boto3-guardduty` is a typing-only companion package for `boto3.client("guardduty")`.

Use it when you want static typing for:

- `GuardDutyClient`
- paginator overloads such as `ListDetectorsPaginator` and `ListFindingsPaginator`
- generated `TypedDict` request and response shapes under `mypy_boto3_guardduty.type_defs`
- literal aliases under `mypy_boto3_guardduty.literals`

It does not replace `boto3` at runtime. Authentication, retry behavior, endpoints, and actual GuardDuty API calls still come from `boto3` and botocore.

## Golden Rules

- Keep `boto3`, `botocore`, and `mypy-boto3-guardduty` on the same release line when possible.
- Use `boto3-stubs[guardduty]` if you want the best `Session.client("guardduty")` inference.
- Use standalone `mypy-boto3-guardduty` when you only need GuardDuty-specific imports and are fine with explicit annotations.
- If stubs are dev-only dependencies, keep type-only imports behind `TYPE_CHECKING`.

## Prerequisites

Before you add the stubs, make sure you already have:

- `boto3` in the Python environment that runs your application
- AWS credentials available through the normal boto3 credential chain
- a region selected for the GuardDuty detector you want to query or manage
- one typing package choice: `boto3-stubs[guardduty]`, `boto3-stubs-lite[guardduty]`, or `mypy-boto3-guardduty`

## Install

Install the runtime SDK plus one typing option.

### Recommended: full boto3 stubs

```bash
python -m pip install "boto3==1.42.33" "boto3-stubs[guardduty]==1.42.33"
```

This gives the best editor experience for `Session.client(...)`.

### Service-only package

```bash
python -m pip install "boto3==1.42.33" "mypy-boto3-guardduty==1.42.33"
```

Use this when you only need GuardDuty typings and prefer narrow dependencies.

### Lower-memory fallback

```bash
python -m pip install "boto3==1.42.33" "boto3-stubs-lite[guardduty]==1.42.33"
```

The lite package is easier on IDE memory, but the maintainer docs note that it does not provide the usual `session.client(...)` and `session.resource(...)` overloads.

### Generate locally for an exact boto3 match

The maintainer docs and package page both point to local generation when you need the stubs to exactly match the boto3 version already locked in your environment:

```bash
uvx --with "boto3==1.42.33" mypy-boto3-builder
```

Then choose `boto3-stubs` and the `GuardDuty` service.

## Initialize And Setup

Annotate the boto3 client explicitly when you are using the standalone package or the lite bundle:

```python
from boto3.session import Session
from mypy_boto3_guardduty.client import GuardDutyClient

session = Session(profile_name="dev", region_name="us-west-2")
guardduty: GuardDutyClient = session.client("guardduty")
```

If your type checker sees the full `boto3-stubs[guardduty]` bundle, this is usually enough:

```python
from boto3.session import Session

guardduty = Session(region_name="us-west-2").client("guardduty")
```

Safe pattern when the stubs are installed only in development or CI:

```python
from __future__ import annotations

from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_guardduty.client import GuardDutyClient

def get_guardduty_client() -> "GuardDutyClient":
    return Session(region_name="us-west-2").client("guardduty")
```

## Authentication And Configuration

`mypy-boto3-guardduty` does not introduce its own auth or config layer. Use the normal boto3 credential and region chain.

Common sources:

1. explicit parameters on `Session(...)` or `client(...)`
2. environment variables such as `AWS_PROFILE`, `AWS_DEFAULT_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
3. shared AWS config and credentials files
4. assume-role, IAM Identity Center, container credentials, or EC2 instance metadata

Typical local setup:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

Then create the typed client normally:

```python
from boto3.session import Session
from mypy_boto3_guardduty.client import GuardDutyClient

guardduty: GuardDutyClient = Session().client("guardduty")
```

GuardDuty is region-scoped. Set the client region deliberately so it matches the detector you want to query or manage.

## Core Usage

Most GuardDuty calls in real apps start by resolving a detector ID and then reusing it for later requests.

### Typed client calls

```python
from boto3.session import Session
from mypy_boto3_guardduty.client import GuardDutyClient

guardduty: GuardDutyClient = Session(region_name="us-west-2").client("guardduty")

detector_ids = guardduty.list_detectors().get("DetectorIds", [])
print(detector_ids)
```

Once you have a detector ID, use it in follow-up calls:

```python
from boto3.session import Session
from mypy_boto3_guardduty.client import GuardDutyClient

guardduty: GuardDutyClient = Session(region_name="us-west-2").client("guardduty")

detector_id = guardduty.list_detectors()["DetectorIds"][0]
finding_ids = guardduty.list_findings(DetectorId=detector_id).get("FindingIds", [])
print(finding_ids[:10])
```

### Typed paginator

```python
from boto3.session import Session
from mypy_boto3_guardduty.client import GuardDutyClient
from mypy_boto3_guardduty.paginator import ListFindingsPaginator

guardduty: GuardDutyClient = Session(region_name="us-west-2").client("guardduty")
detector_id = guardduty.list_detectors()["DetectorIds"][0]

paginator: ListFindingsPaginator = guardduty.get_paginator("list_findings")

for page in paginator.paginate(DetectorId=detector_id):
    for finding_id in page.get("FindingIds", []):
        print(finding_id)
```

### Typed request shapes

The generated package exposes request and response `TypedDict` definitions. Use them when you pass GuardDuty-shaped dictionaries between helpers instead of building large untyped `dict` objects.

```python
from mypy_boto3_guardduty.type_defs import AcceptAdministratorInvitationRequestTypeDef

def submit_accept_request(payload: AcceptAdministratorInvitationRequestTypeDef) -> None:
    print(payload)
```

### Literal aliases

The generated `literals` module exposes enum-like string aliases such as `AdminStatusType`. Use them when you want your type checker to reject unsupported string values before runtime.

## IDE And Type Checker Notes

The official package page explicitly documents support for:

- `mypy`
- `pyright`
- VSCode with Pylance
- PyCharm

Install the stubs into the same environment that your editor or CI type checker analyzes. If your IDE struggles with large overload sets, switch to `boto3-stubs-lite[guardduty]` or the standalone service package and annotate clients explicitly.

## Common Pitfalls

- Installing `mypy-boto3-guardduty` and assuming it replaces `boto3`. It does not.
- Mixing up the package name and import root. Install `mypy-boto3-guardduty`, import `mypy_boto3_guardduty`.
- Expecting the lite package to infer `Session.client("guardduty")` automatically. Add explicit `GuardDutyClient` annotations in lite mode.
- Letting `boto3`, `botocore`, and `mypy-boto3-guardduty` drift apart. New GuardDuty fields can appear in runtime models before matching stubs land.
- Treating typed request shapes as proof that the call will succeed. Region choice, detector state, permissions, and account configuration are still runtime concerns.

## Version-Sensitive Notes

- On `2026-03-12`, the official PyPI project page lists `mypy-boto3-guardduty 1.42.33` as the latest published release. This does not match the initial package metadata version `1.42.62`.
- The maintainer docs describe these packages as following the related boto3 version family, so pin `boto3` and `mypy-boto3-guardduty` close together when exact model coverage matters.
- The public project branding is `types-boto3`, but the published wheel and Python import root remain `mypy-boto3-guardduty` and `mypy_boto3_guardduty`.

## Official Source URLs

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_guardduty/`
- Maintainer versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-guardduty/`
- PyPI JSON API: `https://pypi.org/pypi/mypy-boto3-guardduty/json`
- Boto3 GuardDuty reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/guardduty.html`
- Boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`
- Boto3 session guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/session.html`
- Repository: `https://github.com/youtype/types-boto3`

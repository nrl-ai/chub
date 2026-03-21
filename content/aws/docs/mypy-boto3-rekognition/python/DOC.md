---
name: mypy-boto3-rekognition
description: "mypy-boto3-rekognition package guide for typed boto3 Rekognition clients, literals, TypedDicts, paginators, and waiters"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,boto3,rekognition,python,types,stubs,mypy"
---

# mypy-boto3-rekognition Python Package Guide

## Golden Rule

- `mypy-boto3-rekognition` is a typing package for `boto3`, not a runtime AWS SDK.
- Install it alongside `boto3`, or use the upstream `boto3-stubs[rekognition]` extra.
- Keep `boto3`, `botocore`, and this stub package on the same release line when request and response shapes matter.

## Version-Sensitive Notes

- The official PyPI package page shows `mypy-boto3-rekognition 1.42.3` and `Requires: Python >=3.9`.
- The upstream package page also states these stubs were generated from `boto3==1.42.3` and `botocore==1.42.3`.
- The hosted docs URL is a moving docs root, not an immutable versioned snapshot. Re-check it before pinning a newer release line.
- If you upgrade `boto3` before matching Rekognition stubs exist, runtime calls can still work while type hints lag behind the actual service model.

## Install

### Minimal Service-Specific Setup

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-rekognition==1.42.3"
```

### Bundled Upstream Install Path

```bash
python -m pip install "boto3-stubs[rekognition]==1.42.3"
```

### Lower-Memory Alternative

```bash
python -m pip install "boto3-stubs-lite[rekognition]==1.42.3"
```

Use the standalone package when you only need Rekognition types. Use the umbrella extras when your project already follows the `boto3-stubs` install path across multiple AWS services.

## Initialize And Type A Rekognition Client

AWS credentials and region handling still come from normal `boto3` configuration. The stub package only improves static analysis.

```python
from typing import cast

import boto3
from mypy_boto3_rekognition import RekognitionClient

session = boto3.Session(profile_name="vision-dev", region_name="us-east-1")
rekognition = cast(RekognitionClient, session.client("rekognition"))

response = rekognition.detect_labels(
    Image={"S3Object": {"Bucket": "incoming-images", "Name": "photo.jpg"}},
    MaxLabels=10,
    MinConfidence=80.0,
)

for label in response.get("Labels", []):
    print(label["Name"], label.get("Confidence"))
```

If your checker already infers the concrete client type from `session.client("rekognition")`, the explicit `cast(...)` may be unnecessary. Keep it when editor or CI inference is inconsistent.

## Core Typing Surfaces

### Typed Client

Import `RekognitionClient` from the package root and use it to annotate helpers that accept a Rekognition client:

```python
from mypy_boto3_rekognition import RekognitionClient

def has_face_model(client: RekognitionClient, project_version_arn: str) -> bool:
    response = client.describe_project_versions(
        ProjectArn=project_version_arn.rsplit("/", 1)[0],
        VersionNames=[project_version_arn.rsplit("/", 1)[-1]],
    )
    return bool(response.get("ProjectVersionDescriptions"))
```

### Literals

The package exposes generated literal types under `mypy_boto3_rekognition.literals` for enum-like strings:

```python
from mypy_boto3_rekognition.literals import AttributeTypeType

requested_attributes: list[AttributeTypeType] = ["DEFAULT"]
```

Use them when you want the type checker to reject misspelled Rekognition option values.

### TypedDict Shapes

Generated request and response shapes live under `mypy_boto3_rekognition.type_defs`:

```python
from mypy_boto3_rekognition.type_defs import CompareFacesRequestRequestTypeDef

request: CompareFacesRequestRequestTypeDef = {
    "SourceImage": {"S3Object": {"Bucket": "images", "Name": "source.jpg"}},
    "TargetImage": {"S3Object": {"Bucket": "images", "Name": "target.jpg"}},
    "SimilarityThreshold": 90.0,
}
```

Use these types when a helper builds request payloads separately from the final client call.

### Paginators And Waiters

The generated package also includes `paginator.py` and `waiter.py` modules.

- Use paginators through the normal `boto3` runtime API: `client.get_paginator(...)`.
- Use waiters only for the Rekognition workflows that actually expose them, mainly Custom Labels project-version operations.
- Check the generated docs page for the exact paginator and waiter names in your installed version before hardcoding them.

## AWS Config And Authentication

This package does not add a separate auth layer. Use the normal `boto3` credential provider chain:

1. Explicit credentials passed to `boto3.Session(...)` or `client(...)`
2. Environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, and `AWS_DEFAULT_REGION`
3. Shared AWS config and credentials files, including named profiles
4. IAM roles, IAM Identity Center configuration, or other runtime credential providers in AWS environments

Typical local setup:

```bash
export AWS_PROFILE=vision-dev
export AWS_DEFAULT_REGION=us-east-1
```

Then:

```python
from typing import cast

import boto3
from mypy_boto3_rekognition import RekognitionClient

rekognition = cast(RekognitionClient, boto3.Session().client("rekognition"))
```

For deployed workloads, prefer IAM roles or other short-lived credentials instead of hardcoded keys.

## Type Checker And IDE Setup

- Install the stub package in the same environment your type checker or editor analyzes.
- Import from `mypy_boto3_rekognition`, not `mypy-boto3-rekognition`. The package name uses hyphens; the Python module uses underscores.
- If you use `boto3-stubs-lite`, expect to add more explicit annotations because some session overload helpers are intentionally lighter.
- Treat the stub package as a development-time typing dependency unless your runtime code imports its types directly.

## Common Pitfalls

- Do not try to construct a client from `mypy_boto3_rekognition`; real clients still come from `boto3.client("rekognition")` or `Session.client("rekognition")`.
- Do not forget the AWS region. Static typing will not save you from runtime `NoRegionError`.
- Do not assume every Rekognition API has a paginator or waiter. Many image-analysis operations are plain request-response calls.
- Do not pin a newer `boto3` without checking whether matching Rekognition stubs are published yet.
- Do not confuse dev-only typing imports with runtime dependencies if your production environment omits stub packages.

## Official Sources

- Docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_rekognition/`
- Upstream versioning guide: `https://youtype.github.io/boto3_stubs_docs/#versioning`
- PyPI package page: `https://pypi.org/project/mypy-boto3-rekognition/`
- boto3 credential guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`

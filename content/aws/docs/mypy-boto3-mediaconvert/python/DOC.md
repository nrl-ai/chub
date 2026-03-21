---
name: mypy-boto3-mediaconvert
description: "Type stubs for boto3 MediaConvert clients, paginators, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.37"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,mediaconvert,boto3,python,typing,stubs,mypy,pyright,transcoding"
---

# mypy-boto3-mediaconvert Python Package Guide

## Golden Rule

`mypy-boto3-mediaconvert` is a stubs-only package. Keep `boto3` installed for real AWS Elemental MediaConvert API calls, and use this package only for static typing, autocomplete, literals, and `TypedDict` shapes.

For most projects:

- Use `boto3-stubs[mediaconvert]` when you want `Session().client("mediaconvert")` inference in editors and type checkers.
- Use `mypy-boto3-mediaconvert` when you want only the MediaConvert typing package and are willing to annotate `MediaConvertClient` and paginator types explicitly.
- Use `boto3-stubs-lite[mediaconvert]` if PyCharm or memory usage becomes a problem; the maintainer docs note that the lite package drops `session.client/resource` overloads.

## Install

Recommended when you want boto3 plus automatic client inference:

```bash
python -m pip install "boto3==1.42.37" "boto3-stubs[mediaconvert]==1.42.37"
```

Standalone MediaConvert stubs:

```bash
python -m pip install "boto3==1.42.37" "mypy-boto3-mediaconvert==1.42.37"
```

Lower-memory fallback:

```bash
python -m pip install "boto3==1.42.37" "boto3-stubs-lite[mediaconvert]==1.42.37"
```

Common alternatives:

```bash
uv add "boto3==1.42.37" "boto3-stubs[mediaconvert]==1.42.37"
poetry add "boto3==1.42.37" "boto3-stubs[mediaconvert]==1.42.37"
```

Notes:

- Keep the `boto3` and stub versions aligned. The maintainer package tracks the related boto3 version line.
- If you install only `mypy-boto3-mediaconvert`, runtime imports will still need `boto3`.
- The maintainer docs also support local generation with `uvx --with 'boto3==1.42.37' mypy-boto3-builder` when you need an exact pinned output.

## Initialize And Setup

Authentication and region handling still come from normal boto3 configuration. AWS documents that boto3 searches for credentials in order, including explicit client or session parameters, environment variables, assume-role providers, shared AWS config files, container credentials, and EC2 instance metadata.

Useful environment variables:

- `AWS_PROFILE`
- `AWS_DEFAULT_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_MAX_ATTEMPTS`
- `AWS_RETRY_MODE`

Typed client setup:

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_mediaconvert.client import MediaConvertClient

session = Session(profile_name="media", region_name="us-west-2")
config = Config(
    retries={
        "mode": "standard",
        "max_attempts": 10,
    }
)

mediaconvert: MediaConvertClient = session.client("mediaconvert", config=config)
```

MediaConvert-specific notes:

- Jobs, queues, presets, and templates are region-scoped. Set `region_name` or `AWS_DEFAULT_REGION` deliberately.
- AWS marks `describe_endpoints` as deprecated and says it is no longer required; new code should target the regional endpoint directly instead of building fresh endpoint-discovery logic around that call.
- The service reference documents a low-level client plus paginators. There is no documented resource-oriented stub surface for this package.

## Core Usage

### Typed client

Use the real boto3 client and annotate it with the generated client type:

```python
from boto3.session import Session
from mypy_boto3_mediaconvert.client import MediaConvertClient

def get_client() -> MediaConvertClient:
    return Session(region_name="us-west-2").client("mediaconvert")

client = get_client()
```

### Typed paginator

AWS documents `list_queues` as returning up to twenty queues at a time, and the stubs package exposes a matching paginator type:

```python
from boto3.session import Session
from mypy_boto3_mediaconvert.client import MediaConvertClient
from mypy_boto3_mediaconvert.paginator import ListQueuesPaginator

client: MediaConvertClient = Session(region_name="us-west-2").client("mediaconvert")
paginator: ListQueuesPaginator = client.get_paginator("list_queues")

for page in paginator.paginate(ListBy="NAME", Order="ASCENDING"):
    for queue in page["Queues"]:
        print(queue["Name"], queue["Arn"])
```

The maintainer docs list these paginator classes for this package:

- `DescribeEndpointsPaginator`
- `ListJobTemplatesPaginator`
- `ListJobsPaginator`
- `ListPresetsPaginator`
- `ListQueuesPaginator`
- `ListVersionsPaginator`
- `SearchJobsPaginator`

### Typed request shapes for helpers

MediaConvert job payloads are large. A practical pattern is to type helper functions with the generated `TypedDict` shapes instead of hand-waving request bodies as plain `dict`:

```python
from mypy_boto3_mediaconvert.type_defs import (
    CreateJobRequestRequestTypeDef,
    JobSettingsTypeDef,
)

def build_job_request(
    role_arn: str,
    settings: JobSettingsTypeDef,
) -> CreateJobRequestRequestTypeDef:
    return {
        "Role": role_arn,
        "Settings": settings,
    }
```

The generated type definitions include high-value MediaConvert shapes such as:

- `CreateJobRequestRequestTypeDef`
- `CreateJobTemplateRequestRequestTypeDef`
- `JobSettingsTypeDef`
- `JobTemplateTypeDef`
- `TagResourceRequestRequestTypeDef`

### Literals for enum-like values

Use the `literals` module when you want type-checked string values for MediaConvert enums:

```python
from mypy_boto3_mediaconvert.literals import AacAudioDescriptionBroadcasterMixType

mix: AacAudioDescriptionBroadcasterMixType = "BROADCASTER_MIXED_AD"
```

This is especially useful in job settings, where MediaConvert has many enum-like string fields.

### `TYPE_CHECKING` guard for dev-only stubs

If you install typing packages only in development or CI, guard the imports and fall back to `object`:

```python
from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_mediaconvert.client import MediaConvertClient
else:
    MediaConvertClient = object

client: MediaConvertClient = boto3.client("mediaconvert", region_name="us-west-2")
```

The maintainer docs explicitly recommend this pattern for `pylint` compatibility.

## Configuration And Authentication

This package has no configuration surface of its own. If a call fails because of credentials, region, retries, endpoints, or IAM, fix the underlying boto3 setup.

Useful setup patterns:

```bash
aws configure
export AWS_PROFILE=media
export AWS_DEFAULT_REGION=us-west-2
```

```python
from boto3.session import Session

session = Session(profile_name="media", region_name="us-west-2")
client = session.client("mediaconvert")
```

MediaConvert runtime constraints that typing does not solve:

- `create_job` requires an IAM role ARN, and AWS documents `Role` as required.
- Job settings are large and service-specific; start from a known-good job template or AWS-generated example rather than inventing nested payloads from memory.
- Queue, preset, and job-template names are account and region specific.

## Common Pitfalls

- Installing only `mypy-boto3-mediaconvert` and expecting runtime AWS calls to work without `boto3`.
- Importing the package name with hyphens instead of the Python import root `mypy_boto3_mediaconvert`.
- Expecting automatic `Session().client("mediaconvert")` inference when you installed only the standalone stub package or the lite bundle.
- Building new code around `describe_endpoints`. AWS marks it deprecated and recommends sending requests directly to the regional endpoint instead.
- Forgetting to set a region. MediaConvert resources are region-scoped and jobs often fail in confusing ways when the client region does not match your queues, presets, or templates.
- Treating successful type checking as proof that IAM roles, S3 access, queue names, and MediaConvert job settings are valid.
- Assuming there is a resource-oriented MediaConvert stub surface similar to S3. The documented package surface here is client, paginators, literals, and type definitions.
- Copying request shapes from newer boto3 reference pages without checking them against the exact `1.42.37` package line you pinned.

## Version-Sensitive Notes

- PyPI currently lists `mypy-boto3-mediaconvert 1.42.37`, released on `2026-01-28`.
- As of `2026-03-12`, the live AWS boto3 MediaConvert reference is already on a newer `1.42.54` to `1.42.56` patch line. Treat your installed `boto3` and stub versions as the source of truth when signatures drift.
- The maintainer docs still recommend generating local stubs with `uvx --with 'boto3==1.42.37' mypy-boto3-builder` when you need an exact match for a pinned runtime dependency.
- AWS documents `describe_endpoints` as deprecated and says it should not be used going forward. If older code relies on it, treat that as legacy compatibility work, not a pattern to copy into new code.

## Official Sources

- Maintainer docs: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_mediaconvert/`
- PyPI project: `https://pypi.org/project/mypy-boto3-mediaconvert/`
- AWS boto3 MediaConvert service reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/mediaconvert.html`
- AWS boto3 `describe_endpoints` reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/mediaconvert/client/describe_endpoints.html`
- AWS boto3 `list_queues` reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/mediaconvert/client/list_queues.html`
- AWS boto3 `create_job` reference: `https://docs.aws.amazon.com/boto3/latest/reference/services/mediaconvert/client/create_job.html`
- AWS boto3 credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS boto3 configuration guide: `https://docs.aws.amazon.com/boto3/latest/guide/configuration.html`

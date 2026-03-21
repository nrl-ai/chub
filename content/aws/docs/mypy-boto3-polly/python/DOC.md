---
name: mypy-boto3-polly
description: "mypy-boto3-polly type stubs for typed boto3 Polly clients, paginators, literals, and TypedDict request and response shapes"
metadata:
  languages: "python"
  versions: "1.42.3"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,polly,boto3,python,typing,stubs,mypy,pyright,text-to-speech"
---

# mypy-boto3-polly Python Package Guide

## What It Is

`mypy-boto3-polly` is the maintainer-generated type stub package for the Amazon Polly part of `boto3`.

Use it when you want:

- a typed `PollyClient` for `Session().client("polly")`
- typed paginator classes such as `DescribeVoicesPaginator`
- generated `Literal` aliases such as `EngineType`, `OutputFormatType`, and `VoiceIdType`
- generated `TypedDict` request and response shapes under `type_defs`
- better IDE completion and static analysis in `mypy`, Pyright, Pylance, and similar tools

It does not replace `boto3`, add AWS credentials, or change Polly runtime behavior.

PyPI classifies it as `Typing :: Stubs Only`.

## Golden Rules

- Install `boto3` for runtime behavior and `mypy-boto3-polly` only for typing.
- Prefer `boto3-stubs[polly]` when you want `Session.client("polly")` inference without explicit annotations.
- Use the standalone `mypy-boto3-polly` package or `boto3-stubs-lite[polly]` only if you are willing to annotate clients explicitly.
- Configure credentials, region, retries, endpoints, and profiles through normal boto3 and AWS settings.
- Use PyPI to pin exact released versions. Treat the maintainer docs root as a rolling latest page for package structure and symbol names, not as a version-pinned archive.

## Install

Recommended for pinned environments:

```bash
python -m pip install "boto3==1.42.3" "mypy-boto3-polly==1.42.3"
```

Maintainer-supported alternatives:

```bash
python -m pip install "boto3==1.42.3" "boto3-stubs[polly]==1.42.3"
python -m pip install "boto3==1.42.3" "boto3-stubs-lite[polly]==1.42.3"
```

Practical choice:

- use `mypy-boto3-polly` when you only want Polly typings
- use `boto3-stubs[polly]` when you also want `Session.client(...)` overload inference
- use `boto3-stubs-lite[polly]` when full overloads slow down PyCharm or use too much memory

If your project is pinned to a newer boto3 release line than the published wheel, the maintainer docs recommend local generation:

```bash
uvx --with "boto3==1.42.3" mypy-boto3-builder
```

Then select `boto3-stubs` and the `Polly` service.

## Initialize Type Checking

For the standalone package or lite package, annotate clients explicitly:

```python
from boto3.session import Session
from mypy_boto3_polly.client import PollyClient

session = Session(profile_name="dev", region_name="us-east-1")
polly: PollyClient = session.client("polly")
```

If stub packages are installed only in development or CI, use `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING

from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_polly.client import PollyClient
else:
    PollyClient = object

def make_client() -> "PollyClient":
    return Session(region_name="us-east-1").client("polly")
```

This is also the maintainer-documented workaround for `pylint` complaints about typing-only imports.

## Auth And Configuration

`mypy-boto3-polly` has no package-specific auth or config layer. Credentials, region resolution, retries, proxies, and custom endpoints all come from normal boto3 configuration.

Typical local setup:

```bash
aws configure --profile dev
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-east-1
# Or provide direct credentials when your environment requires them
# export AWS_ACCESS_KEY_ID=...
# export AWS_SECRET_ACCESS_KEY=...
# export AWS_SESSION_TOKEN=...
```

```python
from boto3.session import Session
from botocore.config import Config
from mypy_boto3_polly.client import PollyClient

session = Session(profile_name="dev", region_name="us-east-1")
polly: PollyClient = session.client(
    "polly",
    config=Config(retries={"mode": "standard", "max_attempts": 5}),
)
```

AWS documents the usual boto3 credential sources. In practice, the important ones are:

1. Explicit credentials on `Session(...)` or `client(...)`
2. Environment variables
3. Shared config and credentials files
4. IAM Identity Center or assume-role configuration
5. Container or EC2 instance credentials

## Core Usage

### Typed client

Use the typed client for normal Polly API calls:

```python
from boto3.session import Session
from mypy_boto3_polly.client import PollyClient

polly: PollyClient = Session(region_name="us-east-1").client("polly")

response = polly.describe_voices(LanguageCode="en-US", Engine="neural")

for voice in response.get("Voices", []):
    print(voice["Id"], voice["LanguageCode"], voice.get("SupportedEngines", []))
```

`describe_voices` is the safest way to validate which voices and engines are actually available before hardcoding `VoiceId` values into synthesis code.

### Typed `synthesize_speech` requests

Use generated literals and `TypedDict` shapes to keep synthesis options typed:

```python
from boto3.session import Session
from mypy_boto3_polly.client import PollyClient
from mypy_boto3_polly.literals import EngineType, OutputFormatType, VoiceIdType
from mypy_boto3_polly.type_defs import SynthesizeSpeechInputTypeDef

polly: PollyClient = Session(region_name="us-east-1").client("polly")

engine: EngineType = "neural"
output_format: OutputFormatType = "mp3"
voice_id: VoiceIdType = "Joanna"

request: SynthesizeSpeechInputTypeDef = {
    "Engine": engine,
    "OutputFormat": output_format,
    "Text": "<speak>Hello from Polly</speak>",
    "TextType": "ssml",
    "VoiceId": voice_id,
}

response = polly.synthesize_speech(**request)
audio_bytes = response["AudioStream"].read()
```

AWS documents that `synthesize_speech` accepts UTF-8 plain text or valid SSML. If you request speech marks instead of audio, `OutputFormat` must be `json`.

### Typed paginator

The maintainer docs list paginator types for `describe_voices`, `list_lexicons`, and `list_speech_synthesis_tasks`.

```python
from boto3.session import Session
from mypy_boto3_polly.client import PollyClient
from mypy_boto3_polly.paginator import DescribeVoicesPaginator

polly: PollyClient = Session(region_name="us-east-1").client("polly")
paginator: DescribeVoicesPaginator = polly.get_paginator("describe_voices")

for page in paginator.paginate(LanguageCode="en-US"):
    for voice in page.get("Voices", []):
        print(voice["Id"])
```

### Typed asynchronous synthesis task

`start_speech_synthesis_task` is the typed path when you want Polly to write output to S3 instead of returning a streaming body directly.

```python
from boto3.session import Session
from mypy_boto3_polly.client import PollyClient
from mypy_boto3_polly.type_defs import StartSpeechSynthesisTaskInputTypeDef

polly: PollyClient = Session(region_name="us-east-1").client("polly")

request: StartSpeechSynthesisTaskInputTypeDef = {
    "Engine": "neural",
    "OutputFormat": "mp3",
    "OutputS3BucketName": "my-polly-output-bucket",
    "Text": "Hello from Polly",
    "VoiceId": "Joanna",
}

result = polly.start_speech_synthesis_task(**request)
task = result["SynthesisTask"]

print(task["TaskId"], task["TaskStatus"])
```

Polly does not publish service-specific waiter types in the maintainer docs root. Poll `get_speech_synthesis_task` or paginate `list_speech_synthesis_tasks` when you need task completion tracking.

### Literals and `TypedDict` helpers

Generated literal and `TypedDict` types are useful outside the direct boto3 call site:

```python
from mypy_boto3_polly.literals import LanguageCodeType, VoiceIdType
from mypy_boto3_polly.type_defs import DescribeVoicesInputTypeDef

language: LanguageCodeType = "en-US"
voice_id: VoiceIdType = "Joanna"

request: DescribeVoicesInputTypeDef = {
    "LanguageCode": language,
}
```

This is useful when wrapping Polly inside your own helpers, adapters, or config validation code.

## Common Pitfalls

- Installing only the stub package and expecting real AWS calls to work without `boto3`.
- Importing `mypy-boto3-polly` with hyphens in Python code. The import root is `mypy_boto3_polly`.
- Expecting automatic `Session.client("polly")` inference when you installed only the standalone package or `boto3-stubs-lite[polly]`.
- Treating successful type checking as proof that IAM permissions, region, S3 output permissions, or voice availability are correct.
- Hardcoding a `VoiceId` without checking whether the selected engine and language support it. Use `describe_voices` first.
- Passing invalid SSML to `synthesize_speech`. AWS requires well-formed SSML.
- Treating the maintainer docs root as a release-pinned source. It is a rolling generated page and may drift away from the current PyPI wheel.
- Assuming Polly has resource types or documented service waiters here. This package is primarily client, paginator, literal, and `TypedDict` oriented.

## Version-Sensitive Notes

- This guide tracks the latest published standalone PyPI wheel for `mypy-boto3-polly`, which PyPI listed as `1.42.3` on 2026-03-12.
- PyPI states that `mypy-boto3-polly` version matches the related `boto3` version and marks the package as `Typing :: Stubs Only`.
- The current PyPI page says `1.42.3` was generated with `mypy-boto3-builder 8.12.0`.
- The maintainer docs root is useful for package structure and symbol discovery, but it is a rolling generated page rather than a version-pinned archive and can drift from the currently published wheel.
- If your project needs typings for a newer boto3 patch such as `1.42.66`, verify that exact wheel exists on PyPI first or generate matching stubs locally with `mypy-boto3-builder`.

## Official Sources

- Maintainer docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_polly/`
- Maintainer usage examples: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_polly/usage/`
- PyPI project page: `https://pypi.org/project/mypy-boto3-polly/`
- boto3 Polly reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/polly.html`
- boto3 `describe_voices` reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/polly/client/describe_voices.html`
- boto3 `synthesize_speech` reference: `https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/polly/client/synthesize_speech.html`
- boto3 credentials guide: `https://boto3.amazonaws.com/v1/documentation/api/latest/guide/credentials.html`

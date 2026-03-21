---
name: mypy-boto3-transcribe
description: "mypy-boto3-transcribe type stubs for typed boto3 Amazon Transcribe clients, waiters, literals, and TypedDict request/response shapes"
metadata:
  languages: "python"
  versions: "1.42.25"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aws,transcribe,boto3,type-stubs,mypy,pyright,python"
---

# mypy-boto3-transcribe Python Package Guide

## Golden Rule

`mypy-boto3-transcribe` is a typing package for Amazon Transcribe in `boto3`. It does not replace `boto3`, does not configure AWS credentials, and does not make API calls by itself.

Use one of these modes:

- Install `boto3-stubs[transcribe]` if you want `boto3.client("transcribe")` to be typed automatically in editors and type checkers.
- Install `boto3-stubs-lite[transcribe]` if you need a lower-memory option and are willing to add explicit annotations.
- Install `mypy-boto3-transcribe` if you want the standalone Transcribe stub package and explicit service-specific imports.

## Install

### Recommended for most projects

```bash
python -m pip install boto3 'boto3-stubs[transcribe]'
```

This gives automatic type discovery for `boto3.client("transcribe")` in supported IDEs and type checkers.

### Standalone package

```bash
python -m pip install boto3 mypy-boto3-transcribe
```

Use this when you specifically want the standalone Transcribe stubs package and are willing to annotate the client explicitly.

### Lower-memory fallback

```bash
python -m pip install boto3 'boto3-stubs-lite[transcribe]'
```

Upstream notes that `boto3-stubs-lite` is more RAM-friendly, but it does not provide `session.client()` or `session.resource()` overloads, so explicit annotations are usually necessary.

PyPI metadata for `mypy-boto3-transcribe 1.42.25` requires Python `>=3.9`.

## Setup And Initialization

`mypy-boto3-transcribe` has no package-specific auth or config layer. Runtime configuration still comes from normal `boto3` setup.

AWS says Boto3 requests need both credentials and a region. The usual setup is:

```bash
aws configure
```

or environment variables such as:

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
export AWS_DEFAULT_REGION=us-east-1
```

Boto3 searches for credentials in a defined order, including explicit client/session parameters, environment variables, shared credential files, config files, container credentials, and EC2 instance metadata.

For isolated or multi-account code, create an explicit session:

```python
from boto3.session import Session

session = Session(profile_name="dev", region_name="us-east-1")
```

With the standalone package or lite package, annotate the client:

```python
from boto3.session import Session
from mypy_boto3_transcribe import TranscribeServiceClient

session = Session(profile_name="dev", region_name="us-east-1")
client: TranscribeServiceClient = session.client("transcribe")
```

## Core Usage

### Typed client

PyPI documents `TranscribeServiceClient` as the type for `boto3.client("transcribe")`.

```python
from boto3.session import Session
from mypy_boto3_transcribe import TranscribeServiceClient

client: TranscribeServiceClient = Session(region_name="us-east-1").client("transcribe")

job = client.get_transcription_job(TranscriptionJobName="daily-call-001")
status = job["TranscriptionJob"]["TranscriptionJobStatus"]
print(status)
```

### Typed request payloads with `type_defs`

Use generated `TypedDict` shapes for larger request objects so mypy or pyright can catch misspelled keys before runtime.

```python
from boto3.session import Session
from mypy_boto3_transcribe import TranscribeServiceClient
from mypy_boto3_transcribe.type_defs import StartTranscriptionJobRequestRequestTypeDef

client: TranscribeServiceClient = Session(region_name="us-east-1").client("transcribe")

request: StartTranscriptionJobRequestRequestTypeDef = {
    "TranscriptionJobName": "daily-call-001",
    "LanguageCode": "en-US",
    "MediaFormat": "mp3",
    "Media": {"MediaFileUri": "s3://my-input-bucket/audio/daily-call-001.mp3"},
    "OutputBucketName": "my-output-bucket",
}

client.start_transcription_job(**request)
```

### Waiters

The package includes typed waiters under `mypy_boto3_transcribe.waiter`. PyPI lists waiters including:

- `CallAnalyticsJobCompletedWaiter`
- `LanguageModelCompletedWaiter`
- `MedicalScribeJobCompletedWaiter`
- `MedicalTranscriptionJobCompletedWaiter`
- `MedicalVocabularyReadyWaiter`
- `TranscriptionJobCompletedWaiter`
- `VocabularyReadyWaiter`

```python
from boto3.session import Session
from mypy_boto3_transcribe import TranscribeServiceClient
from mypy_boto3_transcribe.waiter import TranscriptionJobCompletedWaiter

client: TranscribeServiceClient = Session(region_name="us-east-1").client("transcribe")
waiter: TranscriptionJobCompletedWaiter = client.get_waiter("transcription_job_completed")
waiter.wait(TranscriptionJobName="daily-call-001")
```

### Literals and helper modules

The documented helper modules are:

- `mypy_boto3_transcribe.client` for `TranscribeServiceClient`
- `mypy_boto3_transcribe.waiter` for waiter types
- `mypy_boto3_transcribe.literals` for constrained string values
- `mypy_boto3_transcribe.type_defs` for `TypedDict` request and response shapes

Use `literals` when a parameter only accepts a fixed string set, and `type_defs` when you want checked request or response dictionaries.

## Tooling Patterns

### mypy and pyright

Upstream expects `boto3-stubs[transcribe]` to work with normal mypy or pyright setups. For standalone `mypy-boto3-transcribe`, install the stubs in the same interpreter environment that your editor and CI use.

### Avoiding a production dependency on the stubs

If you only want the package in dev or CI, import types behind `TYPE_CHECKING`:

```python
from typing import TYPE_CHECKING
from boto3.session import Session

if TYPE_CHECKING:
    from mypy_boto3_transcribe import TranscribeServiceClient
else:
    TranscribeServiceClient = object

def make_client() -> "TranscribeServiceClient":
    return Session(region_name="us-east-1").client("transcribe")
```

Upstream calls this pattern safe for keeping the typing dependency out of production, with the caveat that `pylint` may need the `object` fallback.

## Configuration And Authentication Notes

The stubs do not change:

- credentials
- region selection
- assume-role behavior
- retries
- endpoint overrides
- transport settings

If Transcribe calls fail at runtime, fix the `boto3` or AWS configuration rather than the stub package.

## Common Pitfalls

- Installing only `mypy-boto3-transcribe` and expecting unannotated `boto3.client("transcribe")` calls to become typed automatically. That behavior comes from `boto3-stubs[transcribe]`.
- Forgetting to install `boto3`. This package is stubs only.
- Running the editor or CI against a different interpreter than the one where the stubs are installed.
- Letting the Transcribe client flow through untyped helpers before annotating it.
- Assuming `boto3-stubs-lite[transcribe]` provides client overload inference. It does not.
- Importing stub types at runtime when the stubs are only installed in dev dependencies. Use `TYPE_CHECKING`.
- Ignoring IDE-specific tradeoffs. Upstream recommends `boto3-stubs-lite` for PyCharm when `Literal` overload performance becomes a problem.

## Version-Sensitive Notes

- This entry covers `mypy-boto3-transcribe 1.42.25`, which matches the version used here and the current PyPI release page on `2026-03-12`.
- Upstream says `mypy-boto3-transcribe` versions track the related `boto3` version.
- PyPI metadata for `1.42.25` requires Python `>=3.9`.
- If you need exact type coverage for a different installed `boto3` version, upstream recommends generating stubs locally with `uvx --with 'boto3==<version>' mypy-boto3-builder`.

## Official Source URLs

- Docs root: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_transcribe/`
- Examples page: `https://youtype.github.io/boto3_stubs_docs/mypy_boto3_transcribe/usage/#usage-examples`
- PyPI project page: `https://pypi.org/project/mypy-boto3-transcribe/`
- Builder repository: `https://github.com/youtype/mypy_boto3_builder`
- AWS credentials guide: `https://docs.aws.amazon.com/boto3/latest/guide/credentials.html`
- AWS session guide: `https://docs.aws.amazon.com/boto3/latest/guide/session.html`

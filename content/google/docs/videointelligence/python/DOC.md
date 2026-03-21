---
name: videointelligence
description: "Google Cloud Video Intelligence Python client for asynchronous video annotation and analysis"
metadata:
  languages: "python"
  versions: "2.18.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,gcp,videointelligence,video,annotation"
---

# Google Cloud Video Intelligence Python Client

## Golden Rule

Use the official `google-cloud-videointelligence` package with Application Default Credentials (ADC). The main RPC, `annotate_video()`, starts a long-running operation, so plan for `operation.result(...)` or an `output_uri` in Cloud Storage.

## Install

Pin the version if your project is already locked to the older release line:

```bash
python -m pip install "google-cloud-videointelligence==2.18.0"
```

Common alternatives:

```bash
uv add "google-cloud-videointelligence==2.18.0"
poetry add "google-cloud-videointelligence==2.18.0"
```

## Authentication And Setup

Before you create a client:

1. Select a Google Cloud project.
2. Enable the Video Intelligence API.
3. Configure ADC.

Enable the API:

```bash
gcloud services enable videointelligence.googleapis.com
```

For local development, Google's recommended ADC flow is:

```bash
gcloud auth application-default login
```

For non-interactive environments, point ADC at a service account key:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

If the API is disabled or ADC is missing, requests fail before annotation starts.

## Initialize The Client

Basic client creation:

```python
from google.cloud import videointelligence_v1 as videointelligence

client = videointelligence.VideoIntelligenceServiceClient()
```

Explicit service account file:

```python
from google.cloud import videointelligence_v1 as videointelligence

client = videointelligence.VideoIntelligenceServiceClient.from_service_account_file(
    "/absolute/path/service-account.json"
)
```

Custom endpoint or standard client options:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import videointelligence_v1 as videointelligence

client = videointelligence.VideoIntelligenceServiceClient(
    client_options=ClientOptions(api_endpoint="videointelligence.googleapis.com")
)
```

Short-lived script using a context manager:

```python
from google.cloud import videointelligence_v1 as videointelligence

with videointelligence.VideoIntelligenceServiceClient() as client:
    ...
```

Use the context-manager form only when you want the client closed at the end of the block. The generated client docs warn that exiting the block closes the underlying transport, which can break other clients sharing it.

## Core Usage

### Label Detection From Google Cloud Storage

Use `input_uri` for normal production workflows so the client does not need to hold the full video in memory.

```python
from google.cloud import videointelligence_v1 as videointelligence

def detect_labels(gcs_uri: str) -> None:
    client = videointelligence.VideoIntelligenceServiceClient()

    operation = client.annotate_video(
        request={
            "input_uri": gcs_uri,
            "features": [videointelligence.Feature.LABEL_DETECTION],
        }
    )

    response = operation.result(timeout=600)
    result = response.annotation_results[0]

    for annotation in result.segment_label_annotations:
        print(annotation.entity.description)

detect_labels("gs://cloud-samples-data/video/cat.mp4")
```

### Speech Transcription

Feature-specific behavior is configured through `VideoContext`.

```python
from google.cloud import videointelligence_v1 as videointelligence

def transcribe_video(gcs_uri: str) -> None:
    client = videointelligence.VideoIntelligenceServiceClient()

    config = videointelligence.SpeechTranscriptionConfig(
        language_code="en-US",
        enable_automatic_punctuation=True,
    )
    context = videointelligence.VideoContext(
        speech_transcription_config=config,
    )

    operation = client.annotate_video(
        request={
            "input_uri": gcs_uri,
            "features": [videointelligence.Feature.SPEECH_TRANSCRIPTION],
            "video_context": context,
        }
    )

    result = operation.result(timeout=1800).annotation_results[0]

    for transcription in result.speech_transcriptions:
        for alternative in transcription.alternatives:
            print(alternative.transcript)
```

### Local Bytes For Small Inputs

`input_content` is useful for tests or smaller files. Prefer `input_uri` for large videos.

```python
from google.cloud import videointelligence_v1 as videointelligence

def detect_shots(path: str) -> None:
    client = videointelligence.VideoIntelligenceServiceClient()

    with open(path, "rb") as fh:
        content = fh.read()

    operation = client.annotate_video(
        request={
            "input_content": content,
            "features": [videointelligence.Feature.SHOT_CHANGE_DETECTION],
        }
    )

    result = operation.result(timeout=600).annotation_results[0]

    for shot in result.shot_annotations:
        start = shot.start_time_offset.total_seconds()
        end = shot.end_time_offset.total_seconds()
        print(f"{start:.2f}s -> {end:.2f}s")
```

### Persist Results To Cloud Storage

For longer jobs or larger outputs, write results to GCS:

```python
operation = client.annotate_video(
    request={
        "input_uri": "gs://my-bucket/video.mp4",
        "output_uri": "gs://my-bucket/results/video-analysis.json",
        "features": [videointelligence.Feature.TEXT_DETECTION],
    }
)
```

## Response Handling

The common shape is:

```python
response = operation.result(timeout=600)
result = response.annotation_results[0]
```

Then inspect only the sections for the features you requested, for example:

- `segment_label_annotations`
- `shot_annotations`
- `text_annotations`
- `speech_transcriptions`
- `explicit_annotation`
- `object_annotations`

Do not assume every section is populated. The result is sparse and feature-dependent.

## Configuration Notes

- ADC is the default auth path. Avoid hard-coding credentials in source.
- `client_options` works for endpoint overrides and standard Google client configuration.
- `timeout` on `operation.result(...)` is a local wait timeout, not the API's processing limit.
- Use `output_uri` when another system will consume results later or when the response can be large.

## Common Pitfalls

### Treating `annotate_video()` as synchronous

The RPC returns a long-running operation. Always wait for it or write output to GCS.

### Sending large videos as `input_content`

This loads the file into process memory. Production jobs should usually use `input_uri`.

### Parsing fields for features you did not request

Only the result sections for requested features are populated. Guard your access accordingly.

### Closing shared transports accidentally

`with VideoIntelligenceServiceClient() as client:` closes the transport when the block exits. Do not use that pattern if the transport is shared elsewhere.

### Mixing GA and beta namespaces

The stable client in this package is `videointelligence_v1`. Some Google docs and samples for preview or older features use beta namespaces such as `videointelligence_v1p3beta1`; those request and response types are not interchangeable with the GA client.

## Version-Sensitive Notes

- As of March 12, 2026, PyPI lists `2.18.0` for `google-cloud-videointelligence`.
- The Google Cloud Python `/latest/changelog` page still tops out at `2.17.0`, so verify the newest patch-level release on PyPI or the repository when you need exact release-history detail.
- Product quickstarts sometimes show `from google.cloud import videointelligence`, while the generated Python reference is organized under `videointelligence_v1`. This guide uses `videointelligence_v1 as videointelligence` so the client, enums, and reference docs line up exactly.

## Official Sources

- PyPI: `https://pypi.org/project/google-cloud-videointelligence/`
- Python reference root: `https://cloud.google.com/python/docs/reference/videointelligence/latest`
- Client reference: `https://cloud.google.com/python/docs/reference/videointelligence/latest/google.cloud.videointelligence_v1.services.video_intelligence_service.VideoIntelligenceServiceClient`
- Product quickstart: `https://cloud.google.com/video-intelligence/docs/quickstart-client-libraries`
- Label sample: `https://cloud.google.com/video-intelligence/docs/samples/video-analyze-labels-gcs`
- ADC setup: `https://cloud.google.com/docs/authentication/provide-credentials-adc`

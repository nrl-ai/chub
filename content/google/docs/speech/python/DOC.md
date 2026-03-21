---
name: speech
description: "Google Cloud Speech-to-Text Python client library for v1 and v2 transcription workflows"
metadata:
  languages: "python"
  versions: "2.37.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,speech,speech-to-text,transcription,audio,cloud"
---

# Google Cloud Speech-to-Text Python Client Library

## Golden Rule

Use `google-cloud-speech` for Google Cloud Speech-to-Text in Python, and decide up front whether your code is using the classic v1 surface or the newer v2 surface.

- v1 uses `from google.cloud import speech` and is the shortest path for existing samples and legacy request shapes.
- v2 uses `from google.cloud.speech_v2 import SpeechClient` and adds recognizers, auto-decoding, batch workflows, regionalization, and newer model options.

Do not mix v1 and v2 message types in the same request path.

## Install

Pin the package version if you want behavior to match this doc:

```bash
python -m pip install --upgrade pip
python -m pip install "google-cloud-speech==2.37.0"
```

You also need the Speech-to-Text API enabled in the target project:

```bash
gcloud services enable speech.googleapis.com
```

## Auth And Project Setup

Google Cloud client libraries use Application Default Credentials (ADC).

ADC checks credentials in this order:

1. `GOOGLE_APPLICATION_CREDENTIALS`
2. Credentials created by `gcloud auth application-default login`
3. The attached service account from the metadata server on Google Cloud

Local development:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
export GOOGLE_CLOUD_PROJECT=YOUR_PROJECT_ID
```

If you must point to a credential file explicitly:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/credentials.json"
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

Prefer attached service accounts in production on Google Cloud. Google documents raw service account keys as a security risk; use them only when you cannot use attached identities or federation.

## Choose The Right Surface

### Use v1 when

- you already have working code based on `speech.RecognitionAudio` and `speech.RecognitionConfig`
- you need existing synchronous, long-running, or streaming samples with the classic API shape
- you are maintaining older integrations and want minimal changes

### Use v2 when

- you are building a new integration
- you want `AutoDetectDecodingConfig`
- you need recognizer resources, batch recognition, newer model selection, or regional routing
- you are migrating toward the current product direction

Google's migration guide is explicit that moving from v1 to v2 is not automatic.

## Initialize Clients

### v1

```python
from google.cloud import speech

client = speech.SpeechClient()
```

### v2

```python
from google.cloud.speech_v2 import SpeechClient

client = SpeechClient()
```

### Regional endpoint override

If you need US or EU data residency, use a matching endpoint and location:

```python
from google.api_core import client_options
from google.cloud.speech_v2 import SpeechClient

options = client_options.ClientOptions(api_endpoint="us-speech.googleapis.com")
client = SpeechClient(client_options=options)
```

Use matching resource paths such as `locations/us` or `locations/eu` when you override the endpoint.

## Core Usage

### v1: transcribe a short local file

Use this for the classic synchronous flow. Google documents local synchronous requests as limited to short audio, and the product docs call out a 60 second and 10 MB limit for local content.

```python
from google.cloud import speech

def transcribe_file(audio_file: str) -> speech.RecognizeResponse:
    client = speech.SpeechClient()

    with open(audio_file, "rb") as f:
        audio_content = f.read()

    audio = speech.RecognitionAudio(content=audio_content)
    config = speech.RecognitionConfig(
        encoding=speech.RecognitionConfig.AudioEncoding.LINEAR16,
        sample_rate_hertz=16000,
        language_code="en-US",
    )

    response = client.recognize(config=config, audio=audio)

    for result in response.results:
        print(result.alternatives[0].transcript)

    return response
```

### v1: transcribe long audio from Cloud Storage

Use a long-running operation for files that exceed the short synchronous path.

```python
from google.cloud import speech

def transcribe_gcs(gcs_uri: str) -> speech.LongRunningRecognizeResponse:
    client = speech.SpeechClient()

    audio = speech.RecognitionAudio(uri=gcs_uri)
    config = speech.RecognitionConfig(
        encoding=speech.RecognitionConfig.AudioEncoding.FLAC,
        sample_rate_hertz=44100,
        language_code="en-US",
    )

    operation = client.long_running_recognize(config=config, audio=audio)
    response = operation.result(timeout=600)

    for result in response.results:
        print(result.alternatives[0].transcript)

    return response
```

### v1: stream audio incrementally

For microphone or chunked file input, use `streaming_recognize` with a generator of `StreamingRecognizeRequest` messages.

```python
from google.cloud import speech
from typing import Iterable

def stream_chunks(chunks: Iterable[bytes]):
    client = speech.SpeechClient()
    config = speech.RecognitionConfig(
        encoding=speech.RecognitionConfig.AudioEncoding.LINEAR16,
        sample_rate_hertz=16000,
        language_code="en-US",
    )
    streaming_config = speech.StreamingRecognitionConfig(config=config, interim_results=True)

    def requests():
        for chunk in chunks:
            yield speech.StreamingRecognizeRequest(audio_content=chunk)

    responses = client.streaming_recognize(
        config=streaming_config,
        requests=requests(),
    )
    for response in responses:
        for result in response.results:
            print(result.is_final, result.alternatives[0].transcript)
```

### v2: recognize a local file with auto-decoding

This is the cleanest v2 quickstart. The sample path uses the implicit recognizer `_`, which avoids creating a recognizer resource for one-off requests.

```python
import os

from google.cloud.speech_v2 import SpeechClient
from google.cloud.speech_v2.types import cloud_speech

PROJECT_ID = os.environ["GOOGLE_CLOUD_PROJECT"]

def recognize_v2(audio_file: str) -> cloud_speech.RecognizeResponse:
    with open(audio_file, "rb") as f:
        audio_content = f.read()

    client = SpeechClient()

    config = cloud_speech.RecognitionConfig(
        auto_decoding_config=cloud_speech.AutoDetectDecodingConfig(),
        language_codes=["en-US"],
        model="short",
    )

    request = cloud_speech.RecognizeRequest(
        recognizer=f"projects/{PROJECT_ID}/locations/global/recognizers/_",
        config=config,
        content=audio_content,
    )

    response = client.recognize(request=request)

    for result in response.results:
        print(result.alternatives[0].transcript)

    return response
```

### v2: batch recognition for long files in Cloud Storage

Use this when the audio is longer than one minute or you want asynchronous processing. Google documents v2 batch recognition as Cloud Storage based, with an upper limit of 480 minutes.

```python
import os

from google.cloud.speech_v2 import SpeechClient
from google.cloud.speech_v2.types import cloud_speech

PROJECT_ID = os.environ["GOOGLE_CLOUD_PROJECT"]

def batch_recognize_v2(audio_uri: str) -> cloud_speech.BatchRecognizeResults:
    client = SpeechClient()

    config = cloud_speech.RecognitionConfig(
        auto_decoding_config=cloud_speech.AutoDetectDecodingConfig(),
        language_codes=["en-US"],
        model="long",
    )

    request = cloud_speech.BatchRecognizeRequest(
        recognizer=f"projects/{PROJECT_ID}/locations/global/recognizers/_",
        config=config,
        files=[cloud_speech.BatchRecognizeFileMetadata(uri=audio_uri)],
        recognition_output_config=cloud_speech.RecognitionOutputConfig(
            inline_response_config=cloud_speech.InlineOutputConfig(),
        ),
    )

    operation = client.batch_recognize(request=request)
    response = operation.result(timeout=1200)

    for result in response.results[audio_uri].transcript.results:
        print(result.alternatives[0].transcript)

    return response.results[audio_uri].transcript
```

## Config And Accuracy Controls

### Basic language and model selection

- v1 uses `language_code="en-US"` and optional `model="video"` or similar model strings
- v2 uses `language_codes=["en-US"]` and model names such as `short` or `long`

### Adaptation and phrase hints

The package includes adaptation clients and types for custom phrase sets and classes. Use them when domain-specific terms are being transcribed incorrectly.

- v1 and v1p1beta1 expose adaptation resources and request types
- v2 also supports phrase sets, custom classes, recognizers, and custom prompt config in the library surface

Start simple before creating reusable resources:

- add explicit language codes
- choose the right model
- use Cloud Storage for long audio instead of forcing large local payloads
- add phrase hints or adaptation resources only after you confirm baseline accuracy is not enough

## Async Usage

The library also exposes async clients:

- `google.cloud.speech_v1.services.speech.SpeechAsyncClient`
- `google.cloud.speech_v2.services.speech.SpeechAsyncClient`

Use those when your application is already async and you want to avoid blocking request handlers on long-running calls.

## Logging And Debugging

You can enable structured library logging without changing code:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

Useful narrower scopes include `google.cloud.speech_v2` and `google.auth`.

## Common Pitfalls

- Do not assume `from google.cloud import speech` is v2. It is the classic v1 import path.
- Do not mix v1 message classes with v2 clients. `speech.RecognitionAudio` is not valid for `speech_v2.SpeechClient`.
- For v2, do not forget the `recognizer` field. Most examples use `projects/{project}/locations/{location}/recognizers/_`.
- For regional endpoints, keep endpoint and resource location aligned. `us-speech.googleapis.com` should pair with `locations/us`, not `locations/global`.
- Local synchronous transcription is for short audio only. For anything larger, move to Cloud Storage plus `long_running_recognize` or `batch_recognize`.
- `GOOGLE_CLOUD_PROJECT` matters in v2 because recognizer resource names include the project.
- Do not hard-code service account key files into the repo. Prefer ADC via `gcloud auth application-default login` locally and attached service accounts in production.
- The package includes v1p1beta1. Use it only when you specifically need that surface; do not choose it by default for new code.

## Version-Sensitive Notes For 2.37.0

- PyPI lists `2.37.0` as the latest release, published on February 27, 2026.
- The package still ships v1, v1p1beta1, and v2 surfaces together, so import path choice is a real compatibility decision.
- The docs changelog page currently shows entries through `2.36.0`, so PyPI is the more reliable source for the exact latest package version.
- The published changelog notes that `credentials_file` was deprecated in `2.34.0`. Prefer ADC or explicit `credentials=` objects instead of depending on file-path constructor arguments.
- The published changelog for `2.35.0` notes automatic mTLS enablement when supported certificates are detected. If endpoint behavior looks unexpected in secured environments, inspect client options and the `GOOGLE_API_USE_MTLS_ENDPOINT` setting.
- The published changelog for `2.36.0` includes speaker diarization documentation updates and new prompt-related configuration support. Re-check request fields before copying older snippets from blogs or answers written before late 2025.

## Official Sources

- Python reference: `https://docs.cloud.google.com/python/docs/reference/speech/latest`
- PyPI package page: `https://pypi.org/project/google-cloud-speech/`
- Client library quickstart and samples: `https://docs.cloud.google.com/speech-to-text/docs/quickstarts/transcribe-client-libraries`
- v1 migration guide: `https://docs.cloud.google.com/speech-to-text/docs/migration`
- ADC guide: `https://docs.cloud.google.com/docs/authentication/application-default-credentials`

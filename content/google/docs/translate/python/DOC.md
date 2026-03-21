---
name: translate
description: "Official Google Cloud Translation client library for Python with Basic v2 and Advanced v3 setup, auth, and translation patterns"
metadata:
  languages: "python"
  versions: "3.24.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud-translate,gcp,translation,i18n,python,google-cloud"
---

# google-cloud-translate Python Package Guide

Use `google-cloud-translate` for the official Google Cloud Translation client library in Python. The package includes two distinct surfaces:

- `google.cloud.translate_v2` for Basic edition
- `google.cloud.translate_v3` for Advanced edition

For new code, prefer Advanced edition v3 unless you specifically need the older Basic edition API shape.

## Install

Install the package directly:

```bash
python -m pip install "google-cloud-translate==3.24.0"
```

With `uv`:

```bash
uv add google-cloud-translate==3.24.0
```

With Poetry:

```bash
poetry add google-cloud-translate==3.24.0
```

## Choose The Right Client

### Use Basic edition v2 when you need:

- simple text translation
- language detection
- supported-language lookup
- compatibility with older Basic edition code

Import it as:

```python
from google.cloud import translate_v2 as translate
```

### Use Advanced edition v3 when you need:

- glossaries
- document translation
- batch translation jobs
- regional resources and custom models
- the current Google Cloud Translation feature set

Import it as:

```python
from google.cloud import translate_v3
```

## Setup And Authentication

Before making requests:

1. Create or select a Google Cloud project.
2. Enable billing on that project.
3. Enable the Cloud Translation API.
4. Authenticate with Application Default Credentials (ADC) or explicit service account credentials.

Local ADC flow:

```bash
gcloud auth application-default login
```

Service account credentials:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

If you are provisioning the API in setup scripts, the service name is:

```bash
gcloud services enable translate.googleapis.com
```

Default to ADC unless you have a reason to inject credentials manually.

## Basic v2 Quick Start

```python
from google.cloud import translate_v2 as translate

client = translate.Client()

result = client.translate(
    "Hello, world!",
    target_language="es",
)

print(result["translatedText"])
print(result.get("detectedSourceLanguage"))
```

Detect language:

```python
from google.cloud import translate_v2 as translate

client = translate.Client()
result = client.detect_language("Bonjour tout le monde")

print(result["language"], result["confidence"])
```

List supported languages:

```python
from google.cloud import translate_v2 as translate

client = translate.Client()
languages = client.get_languages(target_language="en")

for language in languages[:5]:
    print(language["language"], language["name"])
```

## Advanced v3 Quick Start

Every v3 request needs a `parent` resource:

```python
project_id = "my-project"
parent = f"projects/{project_id}/locations/global"
```

Translate text:

```python
from google.cloud import translate_v3

project_id = "my-project"
parent = f"projects/{project_id}/locations/global"

client = translate_v3.TranslationServiceClient()

response = client.translate_text(
    request={
        "parent": parent,
        "contents": ["Hello, world!"],
        "mime_type": "text/plain",
        "target_language_code": "es",
    }
)

for translation in response.translations:
    print(translation.translated_text)
```

Detect language:

```python
from google.cloud import translate_v3

project_id = "my-project"
parent = f"projects/{project_id}/locations/global"

client = translate_v3.TranslationServiceClient()

response = client.detect_language(
    request={
        "parent": parent,
        "content": "Hallo Welt",
        "mime_type": "text/plain",
    }
)

for language in response.languages:
    print(language.language_code, language.confidence)
```

Get supported languages:

```python
from google.cloud import translate_v3

project_id = "my-project"
parent = f"projects/{project_id}/locations/global"

client = translate_v3.TranslationServiceClient()

response = client.get_supported_languages(
    request={
        "parent": parent,
        "display_language_code": "en",
    }
)

for language in response.languages[:5]:
    print(language.language_code, language.display_name)
```

## Regional Resources And Endpoints

Start with `locations/global` unless you are using a regional resource such as a glossary or custom model.

Glossary example:

```python
from google.cloud import translate_v3

project_id = "my-project"
location = "us-central1"
parent = f"projects/{project_id}/locations/{location}"
glossary = (
    f"projects/{project_id}/locations/{location}/glossaries/my-glossary"
)

client = translate_v3.TranslationServiceClient()

response = client.translate_text(
    request={
        "parent": parent,
        "contents": ["Product name: Example Cloud"],
        "mime_type": "text/plain",
        "target_language_code": "fr",
        "glossary_config": {"glossary": glossary},
    }
)

print(response.glossary_translations[0].translated_text)
```

If you need a regional endpoint, pass `client_options`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import translate_v3

client = translate_v3.TranslationServiceClient(
    client_options=ClientOptions(
        api_endpoint="us-central1-translate.googleapis.com"
    )
)
```

Keep the endpoint, `parent`, glossary path, and model path in the same location.

## Credentials Configuration

Explicit credentials object:

```python
from google.cloud import translate_v3
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = translate_v3.TranslationServiceClient(credentials=credentials)
```

You can also use the async client in async applications:

```python
from google.cloud.translate_v3.services.translation_service import (
    TranslationServiceAsyncClient,
)

client = TranslationServiceAsyncClient()
```

## Common Pitfalls

- Do not import `translate_v2` and `translate_v3` interchangeably. They are different APIs with different request and response shapes.
- For v3, always provide `parent` in the form `projects/PROJECT_ID/locations/LOCATION`.
- Use `mime_type="text/plain"` for plain strings and `mime_type="text/html"` only when you are sending HTML content.
- Do not mix `locations/global` with regional glossaries or custom models. The request location and resource location must match.
- The docs URL uses `latest`, so it is not a frozen `3.24.0` snapshot. Treat it as the current 3.x reference and verify anything version-sensitive against PyPI or the changelog.
- Prefer `credentials=` or ADC. The changelog marks `credentials_file` as deprecated starting in `3.22.0`.
- Be careful when copying older pre-3.x examples. Current 3.x docs consistently use request objects or keyword arguments rather than positional request fields.

## Version-Sensitive Notes For 3.24.0

- PyPI currently lists `3.24.0` as the latest release for `google-cloud-translate`.
- The official Python reference page is still the right entry point, but it is a `latest` URL rather than a version-pinned `3.24.0` page.
- The upstream changelog notes that from `3.22.0`, `credentials_file` is deprecated and `api_audience` support was added.
- The upstream changelog also notes that from `3.14.0`, the client can auto-select the mTLS endpoint when `GOOGLE_API_USE_CLIENT_CERTIFICATE=true` and a client certificate source is available.
- If you only need simple text translation, v2 remains available in the same package. For new build-out on Google Cloud, prefer v3 first and drop to v2 only when you intentionally need the Basic edition surface.

## Official Sources

- Python package reference: `https://cloud.google.com/python/docs/reference/translate/latest`
- Translation Basic edition Python guide: `https://cloud.google.com/translate/docs/reference/libraries/v2/python`
- Translation Advanced text translation guide: `https://cloud.google.com/translate/docs/advanced/translating-text-v3`
- PyPI package page: `https://pypi.org/project/google-cloud-translate/`
- Upstream changelog: `https://github.com/googleapis/google-cloud-python/blob/main/packages/google-cloud-translate/CHANGELOG.md`

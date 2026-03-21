---
name: language
description: "Google Cloud Natural Language Python client library for sentiment, entity, moderation, and text classification workflows"
metadata:
  languages: "python"
  versions: "2.19.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,natural-language,nlp,sentiment,entities,moderation,text-classification"
---

# Google Cloud Natural Language Python Client

## Golden Rule

Use the official `google-cloud-language` package with Application Default Credentials (ADC), and pick one API namespace per code path. For new code, `language_v2` is the clearest default for sentiment, entities, moderation, classification, and combined `annotate_text` requests. Older examples may still use `language_v1` or `language_v1beta2`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-language==2.19.0"
```

Common alternatives:

```bash
uv add "google-cloud-language==2.19.0"
poetry add "google-cloud-language==2.19.0"
```

## Authentication And Setup

For local development, authenticate ADC with the Google Cloud CLI:

```bash
gcloud auth application-default login
```

For deployed workloads on Google Cloud, prefer an attached user-managed service account instead of shipping key files.

If you must use a service account key locally or outside Google Cloud, point ADC at the file:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

The client picks credentials up automatically:

```python
from google.cloud import language_v2

client = language_v2.LanguageServiceClient()
```

For explicit credentials or a custom endpoint, pass `credentials=` and `client_options=`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import language_v2
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "service-account.json"
)

client = language_v2.LanguageServiceClient(
    credentials=credentials,
    client_options=ClientOptions(api_endpoint="language.googleapis.com"),
)
```

## Core Request Shape

Most calls start with a `Document`. In Python, the field is `type_`, not `type`, and you must supply either inline `content` or `gcs_content_uri`.

```python
from google.cloud import language_v2

document = language_v2.Document(
    content="Google Cloud Natural Language can score sentiment and extract entities.",
    type_=language_v2.Document.Type.PLAIN_TEXT,
    language_code="en",
)
```

Use `EncodingType.UTF8` unless you have a specific reason to map offsets differently.

## Core Usage

### Analyze Sentiment

```python
from google.cloud import language_v2

client = language_v2.LanguageServiceClient()
document = language_v2.Document(
    content="The launch went well and customers were happy with the result.",
    type_=language_v2.Document.Type.PLAIN_TEXT,
    language_code="en",
)

response = client.analyze_sentiment(
    request={
        "document": document,
        "encoding_type": language_v2.EncodingType.UTF8,
    }
)

print(response.document_sentiment.score)
print(response.document_sentiment.magnitude)
```

### Analyze Entities

```python
from google.cloud import language_v2

client = language_v2.LanguageServiceClient()
document = language_v2.Document(
    content="Google Cloud opened a new office in Seattle.",
    type_=language_v2.Document.Type.PLAIN_TEXT,
)

response = client.analyze_entities(
    request={
        "document": document,
        "encoding_type": language_v2.EncodingType.UTF8,
    }
)

for entity in response.entities:
    print(entity.name, entity.type_.name, entity.salience)
```

### Combine Features With `annotate_text`

Use `annotate_text` when you want one request that bundles the supported v2 features.

```python
from google.cloud import language_v2

client = language_v2.LanguageServiceClient()
document = language_v2.Document(
    content="Google Cloud's update improved moderation and sentiment reporting.",
    type_=language_v2.Document.Type.PLAIN_TEXT,
)
features = language_v2.AnnotateTextRequest.Features(
    extract_entities=True,
    extract_document_sentiment=True,
    classify_text=True,
    moderate_text=True,
)

response = client.annotate_text(
    request={
        "document": document,
        "features": features,
        "encoding_type": language_v2.EncodingType.UTF8,
    }
)

print(response.document_sentiment.score)
print([entity.name for entity in response.entities])
```

### Classify Text

Classification is for longer document-style content. The product docs require at least 20 words of text.

```python
from google.cloud import language_v2

client = language_v2.LanguageServiceClient()
document = language_v2.Document(
    content=(
        "This article explains how machine learning models are trained, evaluated, "
        "and deployed in production systems for business analytics teams."
    ),
    type_=language_v2.Document.Type.PLAIN_TEXT,
)

response = client.classify_text(request={"document": document})

for category in response.categories:
    print(category.name, category.confidence)
```

For large documents already in Cloud Storage, use `gcs_content_uri` instead of loading the full body into memory.

## Moderation

Use `moderate_text` or `annotate_text(..., moderate_text=True)` for safety-style classification of harmful content categories:

```python
from google.cloud import language_v2

client = language_v2.LanguageServiceClient()
document = language_v2.Document(
    content="Example text to score against moderation categories.",
    type_=language_v2.Document.Type.PLAIN_TEXT,
)

response = client.moderate_text(request={"document": document})

for category in response.moderation_categories:
    print(category.name, category.confidence)
```

## Namespace Choice: `language_v2` vs `language_v1`

The current latest reference for `language_v2.LanguageServiceClient` exposes these main methods:

- `analyze_entities`
- `analyze_sentiment`
- `annotate_text`
- `classify_text`
- `moderate_text`

If you need syntax analysis or entity-sentiment analysis because an older codebase depends on them, check the `language_v1` or `language_v1beta2` namespaces and keep the request/response types from that namespace consistent throughout the file.

## Common Pitfalls

- `Document.type_` is the Python field name. Copying REST examples that use `type` will fail.
- Set exactly one document source: inline `content` or `gcs_content_uri`.
- Prefer ADC over embedding service-account JSON in code. Use `gcloud auth application-default login` locally and attached service accounts in production.
- Google Cloud docs mix product guides and API reference pages. Use the product docs for behavioral limits, and the Python reference for exact class and field names.
- Older examples often import `language_v1`. Do not mix `language_v1.Document` with a `language_v2.LanguageServiceClient`.
- `classify_text` is not for short snippets. If the text is too short, classification quality will be poor and requests may be rejected.
- Use UTF-8 encoding offsets unless your downstream consumer explicitly expects a different encoding.

## Version-Sensitive Notes

- PyPI currently lists `google-cloud-language 2.19.0`.
- The API reference pages are current for `2.19.0`, but the changelog page under the same docs root still tops out at `2.18.0`. Validate behavior against both the reference and the product docs when investigating recent changes.
- The package still ships multiple namespaces (`language_v1`, `language_v1beta2`, `language_v2`), so many search results point at older samples. Match your imports to the namespace you intentionally chose.
- The client constructor still accepts several convenience arguments, but the changelog notes deprecations around `credentials_file`; prefer explicit `credentials=` for new code.

## Official Sources

- Python reference root: `https://cloud.google.com/python/docs/reference/language/latest`
- `language_v2.LanguageServiceClient`: `https://cloud.google.com/python/docs/reference/language/latest/google.cloud.language_v2.services.language_service.LanguageServiceClient`
- `Document` type: `https://cloud.google.com/python/docs/reference/language/latest/google.cloud.language_v2.types.Document`
- Changelog: `https://cloud.google.com/python/docs/reference/language/latest/changelog`
- ADC setup for local development: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- ADC for production workloads: `https://cloud.google.com/docs/authentication/set-up-adc-attached-service-account`
- Natural Language product docs: `https://cloud.google.com/natural-language/docs`
- Sentiment guide: `https://cloud.google.com/natural-language/docs/analyzing-sentiment`
- Classification guide: `https://cloud.google.com/natural-language/docs/classifying-text`
- PyPI package page: `https://pypi.org/project/google-cloud-language/`

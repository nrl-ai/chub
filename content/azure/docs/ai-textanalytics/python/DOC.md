---
name: ai-textanalytics
description: "Azure AI Text Analytics SDK for Python for Language service text analysis, custom classification, summarization, and healthcare entity extraction"
metadata:
  languages: "python"
  versions: "5.4.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,language,text-analytics,nlp,sentiment,entities,summarization"
---

# azure-ai-textanalytics Python Package Guide

## Golden Rule

Use `azure-ai-textanalytics` for Azure Language service text analysis in Python, and initialize `TextAnalyticsClient` with a real Language endpoint plus either `AzureKeyCredential` or an Azure Identity credential.

This entry covers package version `5.4.0`.

## Install

```bash
pip install "azure-ai-textanalytics==5.4.0"
```

Install Azure Identity only if you want Microsoft Entra ID authentication:

```bash
pip install azure-identity
```

## Python And Version Notes

- PyPI metadata for `5.4.0` requires Python `>=3.8`.
- The package supports service API versions `3.0`, `3.1`, `2022-05-01`, and `2023-04-01`; the default in the current stable line is `2023-04-01`.
- `5.2.x` and newer target Azure Cognitive Service for Language APIs rather than the older Text Analytics-only service surface.
- `5.4.0` changed continuation token format. Tokens created by older `azure-core` versions are not compatible with this release.
- If you need preview Language service API `2025-05-15-preview` or later, use the `6.0.0b1+` preview package line instead of `5.x`.

## Required Setup

You need:

- an Azure subscription
- a Cognitive Services or Language resource endpoint
- either an API key or an Azure Identity credential

Typical environment variables:

```bash
export AZURE_LANGUAGE_ENDPOINT="https://<resource-name>.cognitiveservices.azure.com/"
export AZURE_LANGUAGE_KEY="<api-key>"
```

## Authenticate And Create A Client

### API key

```python
import os

from azure.ai.textanalytics import TextAnalyticsClient
from azure.core.credentials import AzureKeyCredential

endpoint = os.environ["AZURE_LANGUAGE_ENDPOINT"]
key = os.environ["AZURE_LANGUAGE_KEY"]

client = TextAnalyticsClient(
    endpoint=endpoint,
    credential=AzureKeyCredential(key),
)
```

### Microsoft Entra ID with `DefaultAzureCredential`

```python
import os

from azure.ai.textanalytics import TextAnalyticsClient
from azure.identity import DefaultAzureCredential

endpoint = os.environ["AZURE_LANGUAGE_ENDPOINT"]

client = TextAnalyticsClient(
    endpoint=endpoint,
    credential=DefaultAzureCredential(),
)
```

Important auth caveat: regional endpoints like `https://<region>.api.cognitive.microsoft.com/` do not support Entra ID authentication. Use a custom subdomain endpoint for token-based auth.

## Client Configuration

`TextAnalyticsClient(...)` accepts several defaults that affect most requests:

- `default_language`: default document language, which otherwise defaults to English
- `default_country_hint`: default country hint, which otherwise defaults to `US`; pass `"none"` to disable it
- `api_version`: pin a specific supported Language service API version

Example:

```python
from azure.ai.textanalytics import TextAnalyticsApiVersion

client = TextAnalyticsClient(
    endpoint=endpoint,
    credential=AzureKeyCredential(key),
    default_language="en",
    default_country_hint="none",
    api_version=TextAnalyticsApiVersion.V2023_04_01,
)
```

## Input Shape

Most methods take a batch of documents, not a single bare string.

Use a list of strings for the simple case:

```python
documents = ["The hotel staff were excellent, but check-in was slow."]
```

Use dicts when each document needs its own `id`, `language`, or `country_hint`:

```python
documents = [
    {
        "id": "1",
        "language": "en",
        "text": "I loved the food but the wait was long.",
    },
    {
        "id": "2",
        "language": "en",
        "text": "Seattle is a great city for coffee.",
    },
]
```

## Core Usage

### Detect language

```python
result = client.detect_language(["Bonjour tout le monde"])[0]

if not result.is_error:
    print(result.primary_language.name)
    print(result.primary_language.iso6391_name)
```

### Analyze sentiment

```python
documents = ["The hotel staff were excellent, but check-in was slow."]
result = client.analyze_sentiment(documents, show_opinion_mining=True)[0]

if not result.is_error:
    print(result.sentiment)
    for sentence in result.sentences:
        print(sentence.sentiment, sentence.text)
```

`show_opinion_mining=True` is available only for service API `v3.1` and newer.

### Recognize entities

```python
documents = ["Satya Nadella leads Microsoft in Seattle."]
result = client.recognize_entities(documents)[0]

if not result.is_error:
    for entity in result.entities:
        print(entity.text, entity.category, entity.subcategory)
```

### Detect PII

```python
documents = ["My phone number is 555-555-5555."]
result = client.recognize_pii_entities(documents)[0]

if not result.is_error:
    print(result.redacted_text)
    for entity in result.entities:
        print(entity.text, entity.category)
```

### Extract key phrases

```python
documents = ["This SDK wraps Azure Language service features for Python."]
result = client.extract_key_phrases(documents)[0]

if not result.is_error:
    print(result.key_phrases)
```

## Long-Running Operations

Use `begin_*` methods for summarization, healthcare entity extraction, custom named entity recognition, and custom classification.

### Run multiple actions together

```python
from azure.ai.textanalytics import (
    AnalyzeSentimentAction,
    ExtractKeyPhrasesAction,
    RecognizeEntitiesAction,
)

documents = [
    "The service was fast, but the invoice address was wrong.",
]

poller = client.begin_analyze_actions(
    documents,
    actions=[
        RecognizeEntitiesAction(),
        ExtractKeyPhrasesAction(),
        AnalyzeSentimentAction(),
    ],
)

for action_results in poller.result():
    for result in action_results:
        if result.is_error:
            print("error", result.code, result.message)
        else:
            print(result.kind)
```

### Extractive summary

```python
documents = [
    "Azure AI Language can extract sentences that best summarize a document.",
]

poller = client.begin_extract_summary(documents, max_sentence_count=3)

for result in poller.result():
    if not result.is_error:
        for sentence in result.sentences:
            print(sentence.text)
```

### Custom classification

```python
poller = client.begin_single_label_classify(
    ["Please close my account immediately."],
    project_name="support-routing",
    deployment_name="production",
)

for result in poller.result():
    if not result.is_error:
        for classification in result.classifications:
            print(classification.category, classification.confidence_score)
```

Feature boundaries that matter:

- healthcare entity analysis exists on `begin_analyze_healthcare_entities(...)`
- custom entity recognition and custom classification require service API `2022-05-01` or newer
- extractive and abstractive summarization require service API `2023-04-01` or newer

## Common Options On Calls

Depending on the method, commonly useful keyword arguments include:

- `language`
- `model_version`
- `show_stats`
- `disable_service_logs`
- `string_index_type`

`disable_service_logs=True` opts out of service-side logging of your text for troubleshooting. `string_index_type` matters if you need offsets that line up with a specific client encoding model.

## Async Usage

If you are already in an async app, use the `aio` client:

```python
import os

from azure.ai.textanalytics.aio import TextAnalyticsClient
from azure.core.credentials import AzureKeyCredential

endpoint = os.environ["AZURE_LANGUAGE_ENDPOINT"]
key = os.environ["AZURE_LANGUAGE_KEY"]

async def main() -> None:
    client = TextAnalyticsClient(endpoint, AzureKeyCredential(key))
    async with client:
        results = await client.extract_key_phrases(
            ["Agent workflows depend on package docs."]
        )
        print(results[0].key_phrases)
```

## Common Pitfalls

- Do not pass a bare string where the SDK expects a batch. Use `["text"]`, not `"text"`.
- Check `result.is_error` before reading result fields. Batch responses can mix success objects and `DocumentError` objects.
- Do not assume old examples still match the current client surface. `5.2.x+` is the Language service era, not the older Text Analytics-only shape.
- If you rely on Entra ID, do not use a regional endpoint.
- If you persist continuation tokens for long-running operations, treat the `5.4.0` upgrade as a compatibility boundary.
- If offsets matter, set `string_index_type` deliberately instead of assuming another runtime's indexing behavior.
- Confirm service-side data limits before sending very large documents or batches.

## Official Sources

- Docs root: `https://learn.microsoft.com/en-us/python/api/azure-ai-textanalytics/`
- Overview and getting started: `https://learn.microsoft.com/en-us/python/api/overview/azure/ai-textanalytics-readme?view=azure-python`
- `TextAnalyticsClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-ai-textanalytics/azure.ai.textanalytics.textanalyticsclient?view=azure-python`
- PyPI package page: `https://pypi.org/project/azure-ai-textanalytics/`
- Changelog: `https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/textanalytics/azure-ai-textanalytics/CHANGELOG.md`

---
name: ai-language-text
description: "Azure AI Language Text for Python - setup, authentication, text analysis workflows, and package-selection pitfalls"
metadata:
  languages: "python"
  versions: "1.0.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "azure,language,nlp,textanalytics,sentiment,ner,summarization"
---

# azure-ai-language-text Python Package Guide

## Golden Rule

Use the package already pinned by the project before copying Azure Language samples.

- If the codebase already imports `azure.ai.language.text`, stay on `azure-ai-language-text` and verify the exact client/method names against the package API reference.
- If the codebase imports `azure.ai.textanalytics`, stay on `azure-ai-textanalytics` instead. Microsoft Learn's current Python quickstarts still use that package for most Azure Language runtime examples.
- Do not rename imports or mix both packages in the same module unless you are doing an explicit migration.

## Current Upstream State

This guide covers `azure-ai-language-text` `1.0.0`, but Microsoft's broader Python guidance is currently split:

- The Azure AI Language Python overview still highlights `azure-ai-textanalytics` as the main Python runtime package.
- Current Microsoft Learn quickstarts for language detection and summarization still show `azure-ai-textanalytics` installs and `TextAnalyticsClient` examples.
- The Azure AI Language What's New page announced a newer Python preview under `azure-ai-textanalytics 6.0.0b1`.

Inference: package naming and sample freshness are in flux. Treat this package as version-sensitive, and confirm the import surface in the repo you are editing before generating code.

## Install

```bash
pip install azure-ai-language-text==1.0.0
pip install azure-identity
```

If `pip` cannot resolve `azure-ai-language-text` in the target environment, stop and re-check the current Microsoft package mapping. The currently published Learn quickstarts may still expect:

```bash
pip install azure-ai-textanalytics==5.2.0
```

or:

```bash
pip install azure-ai-textanalytics==5.3.0
```

depending on which article/version you are following.

## Setup And Authentication

You need an Azure AI Language resource endpoint and either an API key or Microsoft Entra credentials.

Set environment variables:

```bash
export LANGUAGE_ENDPOINT="https://<resource-name>.cognitiveservices.azure.com/"
export LANGUAGE_KEY="<api-key>"
```

API key pattern used in current Python quickstarts:

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.textanalytics import TextAnalyticsClient

client = TextAnalyticsClient(
    endpoint=os.environ["LANGUAGE_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["LANGUAGE_KEY"]),
)
```

Microsoft Entra ID pattern:

```python
import os
from azure.identity import DefaultAzureCredential
from azure.ai.textanalytics import TextAnalyticsClient

client = TextAnalyticsClient(
    endpoint=os.environ["LANGUAGE_ENDPOINT"],
    credential=DefaultAzureCredential(),
)
```

For `azure-ai-language-text`, expect the same endpoint and credential objects to apply. The verified Python quickstarts above currently use `TextAnalyticsClient`; if your project already uses `azure.ai.language.text`, keep its existing import path and map the same auth pattern onto that client.

## Core Usage Pattern

The stable operational model across Azure Language runtime SDKs is:

1. Create one long-lived client with the resource endpoint and credential.
2. Send plain-text documents as a list.
3. Use direct methods for fast analyses such as language detection, sentiment, key phrases, and entity recognition.
4. Use a poller or long-running operation for summarization, healthcare, and other multi-action workflows.
5. Check per-document errors instead of assuming the whole batch succeeded.

### Verified Current Python Example: Language Detection

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.textanalytics import TextAnalyticsClient

client = TextAnalyticsClient(
    endpoint=os.environ["LANGUAGE_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["LANGUAGE_KEY"]),
)

documents = [
    "Ce document est redige en francais.",
    "Este documento esta escrito en espanol.",
]

response = client.detect_language(documents=documents, country_hint="us")

for item in response:
    if item.is_error:
        print(f"error: {item.error.code} {item.error.message}")
    else:
        print(item.primary_language.iso6391_name, item.primary_language.name)
```

### Verified Current Python Example: Extractive Summarization

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.textanalytics import TextAnalyticsClient
from azure.ai.textanalytics import ExtractiveSummaryAction

client = TextAnalyticsClient(
    endpoint=os.environ["LANGUAGE_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["LANGUAGE_KEY"]),
)

document = [
    "The tower was built in 1889 during the Paris World's Fair. "
    "It was designed by Gustave Eiffel. It remains one of the most visited landmarks in the world."
]

poller = client.begin_analyze_actions(
    document,
    actions=[ExtractiveSummaryAction(max_sentence_count=2)],
)

document_results = poller.result()

for action_results in document_results:
    for result in action_results:
        if result.is_error:
            print(f"error: {result.code} {result.message}")
        else:
            for summary in result.sentences:
                print(summary.text)
```

If the project actually uses `azure-ai-language-text`, the same workflow should still apply conceptually: one client, list-of-documents inputs, direct text analyses, and poller-based long-running operations for summarization-style tasks. Verify the exact method names on the package reference page before editing production code.

## Config And Service Boundaries

- Runtime analysis and authoring are not the same concern. Current Microsoft docs use `azure-ai-textanalytics-authoring` for authoring custom projects and `azure-ai-textanalytics` for runtime calls.
- Region and pricing tier matter. Some features are preview-only or require specific regions/SKU support.
- The endpoint must be the Azure AI Language resource endpoint, not Azure OpenAI, Translator, or a custom proxy URL unless the project intentionally fronts the service.
- Prefer `DefaultAzureCredential` for deployed apps. Use `AzureKeyCredential` for local scripts, quick tests, or when the app is already key-based.

## Common Pitfalls

- Package drift: do not assume `azure-ai-language-text` and `azure-ai-textanalytics` are drop-in rename equivalents. Check imports already present in the repo first.
- Batch error handling: Azure Language returns per-document failures; handle `is_error` per item.
- Long-running actions: summarization and some advanced analyses are poller-based, not immediate responses.
- Service quotas: large batches and long documents can hit request-size or transaction limits.
- Model-specific inputs: custom text models often need project and deployment identifiers in addition to text.
- API-version sensitivity: preview packages and preview docs can expose methods or enum values missing from GA packages.

## Version-Sensitive Notes

- Target version for this session: `azure-ai-language-text` `1.0.0`.
- Current Microsoft Learn Python quickstarts still reference `azure-ai-textanalytics` `5.2.0` or `5.3.0`.
- Microsoft announced a new Python preview line as `azure-ai-textanalytics 6.0.0b1` in October 2025.
- Practical rule: if you are writing code into an existing repository, trust the repository's pinned dependency and import path over generic blog posts or cross-language samples.

## Official Sources Used

- Package docs root: `https://learn.microsoft.com/en-us/python/api/azure-ai-language-text/`
- Azure AI Language Python overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/ai-language-readme?view=azure-python`
- Language detection quickstart: `https://learn.microsoft.com/en-us/azure/ai-services/language-service/language-detection/quickstart`
- Summarization quickstart: `https://learn.microsoft.com/en-us/azure/ai-services/language-service/summarization/quickstart`
- What's new: `https://learn.microsoft.com/en-us/azure/ai-services/language-service/whats-new`
- Registry URL: `https://pypi.org/project/azure-ai-language-text/`

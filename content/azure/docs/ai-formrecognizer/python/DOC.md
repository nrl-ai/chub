---
name: ai-formrecognizer
description: "azure-ai-formrecognizer package guide for Python covering auth, analysis clients, model administration, and version boundaries"
metadata:
  languages: "python"
  versions: "3.3.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,document-intelligence,form-recognizer,ocr,document-analysis,python,sdk"
---

# azure-ai-formrecognizer Python Package Guide

## What This Package Covers

`azure-ai-formrecognizer` is the Azure SDK package for Python code that works with Azure Form Recognizer, now branded as Azure Document Intelligence.

Use it when you need to:

- analyze PDFs or images with prebuilt models such as layout, document, invoice, or receipt
- extract structured fields from custom models
- build and manage custom document models
- build and run document classifiers

Package boundary that matters:

- `azure-ai-formrecognizer==3.3.3` covers service API versions up to `2023-07-31`
- newer Document Intelligence service previews starting at `2023-10-31-preview` moved to `azure-ai-documentintelligence`

For new code on this package line, the main imports are:

```python
from azure.ai.formrecognizer import (
    AnalysisFeature,
    DocumentAnalysisClient,
    DocumentModelAdministrationClient,
)
```

Legacy clients still exist for older service versions:

```python
from azure.ai.formrecognizer import FormRecognizerClient, FormTrainingClient
```

Use those only when maintaining older `2.0` or `2.1` integrations.

## Install

```bash
python -m pip install "azure-ai-formrecognizer==3.3.3"
```

If you want Azure AD authentication:

```bash
python -m pip install "azure-ai-formrecognizer==3.3.3" azure-identity
```

Runtime requirements from PyPI:

- Python `>=3.8`

## Required Setup

You need:

1. an Azure Form Recognizer or Cognitive Services resource
2. the resource endpoint
3. either an API key or an Azure AD credential

Typical environment variables:

```bash
export AZURE_FORM_RECOGNIZER_ENDPOINT="https://<resource-name>.cognitiveservices.azure.com/"
export AZURE_FORM_RECOGNIZER_KEY="<api-key>"
```

Endpoint rules from the official overview docs:

- key auth works with either a regional endpoint or a custom subdomain endpoint
- Azure AD auth requires a custom subdomain endpoint such as `https://<resource-name>.cognitiveservices.azure.com/`
- regional endpoints such as `https://<region>.api.cognitive.microsoft.com/` do not support Azure AD

## Authentication

### API key

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.formrecognizer import DocumentAnalysisClient

client = DocumentAnalysisClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["AZURE_FORM_RECOGNIZER_KEY"]),
)
```

### Azure AD with `DefaultAzureCredential`

Use this when the app runs with a managed identity or service principal.

```python
import os
from azure.ai.formrecognizer import DocumentAnalysisClient
from azure.identity import DefaultAzureCredential

client = DocumentAnalysisClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=DefaultAzureCredential(),
)
```

The principal needs the `Cognitive Services User` role on the resource.

## Client Selection

Use this mapping:

- `DocumentAnalysisClient`: analyze documents and run classifiers on modern service versions
- `DocumentModelAdministrationClient`: build, compose, copy, list, and delete custom models and classifiers
- `FormRecognizerClient`: legacy analysis surface for older `2.0` and `2.1` code
- `FormTrainingClient`: legacy model training surface for older `2.0` and `2.1` code

If the codebase already calls methods like `begin_recognize_invoices`, `begin_recognize_receipts`, or `begin_train_custom_model`, you are on the legacy client surface and should check the migration guide before replacing calls mechanically.

## Quick Start: Analyze A Prebuilt Invoice

All `begin_*` operations return a poller. Call `.result()` to wait for completion.

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.formrecognizer import DocumentAnalysisClient

client = DocumentAnalysisClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["AZURE_FORM_RECOGNIZER_KEY"]),
)

with open("invoice.pdf", "rb") as fh:
    poller = client.begin_analyze_document(
        "prebuilt-invoice",
        document=fh,
        locale="en-US",
    )

result = poller.result()

for invoice in result.documents:
    vendor = invoice.fields.get("VendorName")
    total = invoice.fields.get("InvoiceTotal")
    print(vendor.value if vendor else None)
    print(total.value if total else None)
```

Common prebuilt model IDs:

- `prebuilt-layout`
- `prebuilt-document`
- `prebuilt-invoice`
- `prebuilt-receipt`
- `prebuilt-idDocument`

## Analyze From URL

Use the URL form only when the file is public or the URL includes a SAS token.

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.formrecognizer import DocumentAnalysisClient

client = DocumentAnalysisClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["AZURE_FORM_RECOGNIZER_KEY"]),
)

poller = client.begin_analyze_document_from_url(
    "prebuilt-layout",
    document_url="https://example.blob.core.windows.net/forms/sample.pdf?<sas>",
)

layout = poller.result()
for page in layout.pages:
    print(page.page_number, len(page.lines))
```

## Use Optional Analysis Features

The `3.3.x` line exposes the `features=` argument on `begin_analyze_document`. Use this when you need higher-resolution OCR or extra extraction signals on the `2023-07-31` service version.

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.formrecognizer import AnalysisFeature, DocumentAnalysisClient

client = DocumentAnalysisClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["AZURE_FORM_RECOGNIZER_KEY"]),
)

with open("engineering-drawing.pdf", "rb") as fh:
    poller = client.begin_analyze_document(
        "prebuilt-layout",
        document=fh,
        features=[
            AnalysisFeature.OCR_HIGH_RESOLUTION,
            AnalysisFeature.BARCODES,
        ],
    )

result = poller.result()
print(len(result.pages))
```

Available feature flags in the official enum include:

- `OCR_HIGH_RESOLUTION`
- `LANGUAGES`
- `BARCODES`
- `FORMULAS`
- `KEY_VALUE_PAIRS`
- `STYLE_FONT`
- `QUERY_FIELDS`

## Build A Custom Document Model

Use `DocumentModelAdministrationClient` for model build and model management.

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.formrecognizer import DocumentModelAdministrationClient, ModelBuildMode

admin = DocumentModelAdministrationClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["AZURE_FORM_RECOGNIZER_KEY"]),
)

poller = admin.begin_build_document_model(
    build_mode=ModelBuildMode.TEMPLATE,
    blob_container_url=os.environ["TRAINING_CONTAINER_SAS_URL"],
    model_id="contoso-invoices",
    description="Invoice extractor",
)

model = poller.result()
print(model.model_id)
print(model.description)
```

Operational notes:

- use a container SAS URL for training data unless the container is intentionally public or accessible via managed identity
- `ModelBuildMode.TEMPLATE` and `ModelBuildMode.NEURAL` have different accuracy, training-data, and cost tradeoffs
- call `admin.get_resource_details()` when you need current model-count and quota limits

List models:

```python
for item in admin.list_document_models():
    print(item.model_id, item.created_on)
```

## Build And Run A Classifier

Classifier APIs are available on the `3.3.x` line.

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.formrecognizer import (
    BlobSource,
    ClassifierDocumentTypeDetails,
    DocumentModelAdministrationClient,
)

admin = DocumentModelAdministrationClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["AZURE_FORM_RECOGNIZER_KEY"]),
)

poller = admin.begin_build_document_classifier(
    doc_types={
        "invoice": ClassifierDocumentTypeDetails(
            source=BlobSource(
                container_url=os.environ["CLASSIFIER_CONTAINER_SAS_URL"],
                prefix="invoice/train",
            )
        ),
        "receipt": ClassifierDocumentTypeDetails(
            source=BlobSource(
                container_url=os.environ["CLASSIFIER_CONTAINER_SAS_URL"],
                prefix="receipt/train",
            )
        ),
    },
    classifier_id="finance-docs",
    description="Finance document router",
)

classifier = poller.result()
print(classifier.classifier_id)
```

Run classification:

```python
import os
from azure.core.credentials import AzureKeyCredential
from azure.ai.formrecognizer import DocumentAnalysisClient

client = DocumentAnalysisClient(
    endpoint=os.environ["AZURE_FORM_RECOGNIZER_ENDPOINT"],
    credential=AzureKeyCredential(os.environ["AZURE_FORM_RECOGNIZER_KEY"]),
)

with open("mixed-batch.pdf", "rb") as fh:
    poller = client.begin_classify_document("finance-docs", document=fh)

result = poller.result()
for document in result.documents:
    print(document.doc_type, document.confidence)
```

## Configuration And Compatibility Notes

### Service-version boundary

`azure-ai-formrecognizer==3.3.3` is the final stable line for the older package name and modernizes the client surface around:

- `DocumentAnalysisClient`
- `DocumentModelAdministrationClient`
- service API `2023-07-31`

If you need capabilities introduced in `2023-10-31-preview` or later, move to `azure-ai-documentintelligence` instead of extending this package further.

### Legacy migration boundary

Older `3.1.x` and legacy `2.x` examples use different method families. These older examples do not map one-for-one to the `3.3.3` API surface.

If you see code like:

- `begin_recognize_content`
- `begin_recognize_business_cards`
- `begin_recognize_receipts`
- `begin_train_custom_model`

do not swap method names blindly. Check the official migration guide and rework the response parsing too.

### Async support

Async clients live under `azure.ai.formrecognizer.aio` with the same service concepts as the sync clients. Use them only if the surrounding app is already async.

## Common Pitfalls

- Do not use `FormRecognizerClient` or `FormTrainingClient` for new `3.3.3` code.
- Do not forget `.result()` on pollers returned by `begin_*` methods.
- Do not use Azure AD with a regional endpoint; use a custom subdomain endpoint.
- Do not assume URL-based analysis works for private blobs without a SAS token or another publicly reachable URL.
- Do not assume old blog posts that mention `begin_recognize_*` methods apply to the current client surface.
- Do not assume this package covers the newest Document Intelligence preview APIs; it stops at `2023-07-31`.
- Do not assume `features=` is portable to older installs such as `3.2.x` or legacy `3.1.x`.

## Official Sources Used

- Microsoft Learn package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/ai-formrecognizer-readme?view=azure-python`
- Microsoft Learn API reference root: `https://learn.microsoft.com/en-us/python/api/azure-ai-formrecognizer/`
- `DocumentAnalysisClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-ai-formrecognizer/azure.ai.formrecognizer.documentanalysisclient?view=azure-python`
- `DocumentModelAdministrationClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-ai-formrecognizer/azure.ai.formrecognizer.documentmodeladministrationclient?view=azure-python`
- `AnalysisFeature` enum: `https://learn.microsoft.com/en-us/python/api/azure-ai-formrecognizer/azure.ai.formrecognizer.analysisfeature?view=azure-python`
- PyPI package page: `https://pypi.org/project/azure-ai-formrecognizer/`
- Azure SDK migration guide: `https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/formrecognizer/azure-ai-formrecognizer/MIGRATION_GUIDE.md`

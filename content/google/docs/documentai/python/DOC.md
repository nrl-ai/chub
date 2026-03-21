---
name: documentai
description: "Google Cloud Document AI client for Python with ADC auth, regional endpoints, processor workflows, and batch GCS processing"
metadata:
  languages: "python"
  versions: "3.11.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,documentai,ocr,gcp,python,client"
---

# google-cloud-documentai Python Package Guide

## What It Is

`google-cloud-documentai` is the official Google Cloud client library for Document AI in Python. Use it when you need to:

- send PDFs or images to a Document AI processor
- extract text, entities, form fields, tables, and page layout
- batch-process files from Google Cloud Storage
- manage processor discovery and processor-version resource names from Python

This package is only the client. Your code still needs:

- a Google Cloud project
- a Document AI processor in the correct region
- Application Default Credentials or another supported Google auth flow

## Version Covered

- Ecosystem: `pypi`
- Package: `google-cloud-documentai`
- Version covered: `3.11.0`
- Import path used in this doc: `from google.cloud import documentai`
- Docs URL: `https://cloud.google.com/python/docs/reference/documentai/latest`
- Stable versioned reference used for this entry: `https://cloud.google.com/python/docs/reference/documentai/3.11.0`
- Registry URL: `https://pypi.org/project/google-cloud-documentai/`

## Install

```bash
python -m pip install "google-cloud-documentai==3.11.0"
```

For local development, use Application Default Credentials:

```bash
gcloud auth application-default login
```

For a service-account key outside Google Cloud:

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

## Required Setup

Document AI is regional. Before writing code, confirm all of these line up:

- `project_id`
- `location`, usually `us` or `eu`
- `processor_id`
- the processor resource name or processor-version resource name you will call
- a MIME type that matches the actual file, such as `application/pdf`, `image/png`, `image/jpeg`, or `image/tiff`

The client endpoint must match the processor location:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import documentai

location = "us"

client = documentai.DocumentProcessorServiceClient(
    client_options=ClientOptions(
        api_endpoint=f"{location}-documentai.googleapis.com"
    )
)
```

If the processor is in `eu`, use `eu-documentai.googleapis.com`.

## Quick Start: Process One Local Document

Use `process_document` for synchronous processing of one document at a time.

```python
from google.api_core.client_options import ClientOptions
from google.cloud import documentai

project_id = "my-project"
location = "us"
processor_id = "1234567890abcdef"
file_path = "invoice.pdf"
mime_type = "application/pdf"

client = documentai.DocumentProcessorServiceClient(
    client_options=ClientOptions(
        api_endpoint=f"{location}-documentai.googleapis.com"
    )
)

name = client.processor_path(project_id, location, processor_id)

with open(file_path, "rb") as fh:
    request = documentai.ProcessRequest(
        name=name,
        raw_document=documentai.RawDocument(
            content=fh.read(),
            mime_type=mime_type,
        ),
    )

result = client.process_document(request=request)
document = result.document

print(document.text)

for entity in document.entities:
    print(entity.type_, entity.mention_text, entity.confidence)
```

Use `document.text` for full OCR output. For structured extraction, inspect `document.entities`, `document.pages`, `document.pages[n].form_fields`, and `document.pages[n].tables`.

## When To Call A Processor Version

Some workflows need a specific processor version instead of the default processor target. In that case, pass the processor-version resource name as `name`.

```python
processor_version_name = (
    f"projects/{project_id}/locations/{location}/"
    f"processors/{processor_id}/processorVersions/{processor_version_id}"
)

request = documentai.ProcessRequest(
    name=processor_version_name,
    raw_document=documentai.RawDocument(
        content=file_bytes,
        mime_type="application/pdf",
    ),
)
```

Use this pattern when the processor has multiple deployed versions and your application must pin one explicitly.

## Batch Processing From Google Cloud Storage

Use `batch_process_documents` for larger jobs or when the input and output already live in GCS. This is a long-running operation.

```python
from google.api_core.client_options import ClientOptions
from google.cloud import documentai

project_id = "my-project"
location = "us"
processor_id = "1234567890abcdef"

gcs_input_uri = "gs://my-input-bucket/forms/form-001.pdf"
gcs_output_uri = "gs://my-output-bucket/documentai-results/"

client = documentai.DocumentProcessorServiceClient(
    client_options=ClientOptions(
        api_endpoint=f"{location}-documentai.googleapis.com"
    )
)

name = client.processor_path(project_id, location, processor_id)

gcs_document = documentai.GcsDocument(
    gcs_uri=gcs_input_uri,
    mime_type="application/pdf",
)

request = documentai.BatchProcessRequest(
    name=name,
    input_documents=documentai.BatchDocumentsInputConfig(
        gcs_documents=documentai.GcsDocuments(documents=[gcs_document])
    ),
    document_output_config=documentai.DocumentOutputConfig(
        gcs_output_config=documentai.DocumentOutputConfig.GcsOutputConfig(
            gcs_uri=gcs_output_uri
        )
    ),
)

operation = client.batch_process_documents(request=request)
operation.result(timeout=900)

print("Batch processing complete")
```

Important behavior:

- the response does not return parsed documents inline
- Document AI writes JSON output files under the GCS prefix you provide
- your app must read those output objects afterward if it needs the structured results

## Listing Processors

Use `list_processors` when the app needs to discover an existing processor rather than hardcode one.

```python
from google.api_core.client_options import ClientOptions
from google.cloud import documentai

project_id = "my-project"
location = "us"

client = documentai.DocumentProcessorServiceClient(
    client_options=ClientOptions(
        api_endpoint=f"{location}-documentai.googleapis.com"
    )
)

parent = client.common_location_path(project_id, location)

for processor in client.list_processors(parent=parent):
    print(processor.name)
    print(processor.display_name)
    print(processor.type_)
```

If you create processors programmatically, first list processor types in the same region and use the returned type name for creation.

## Auth And Configuration

### Preferred Auth

Use Application Default Credentials unless you have a specific reason not to.

Common patterns:

- local development: `gcloud auth application-default login`
- Google Cloud runtime: attach a service account to the workload
- non-Google runtime: set `GOOGLE_APPLICATION_CREDENTIALS`

Do not embed service-account JSON in source code.

### Endpoint Selection

Always set the endpoint from `location`:

```python
api_endpoint = f"{location}-documentai.googleapis.com"
```

The processor resource name and the endpoint must point to the same region.

### Timeouts And Retries

Every RPC accepts the standard Google API Core call options:

```python
result = client.process_document(
    request=request,
    timeout=120,
)
```

Use explicit `timeout` values for large documents or slow networks. Only override retries when you have a concrete failure mode to address.

## Common Pitfalls

- Region mismatch: `us` processor plus `eu-documentai.googleapis.com` fails.
- Wrong resource name: `name` must be the full processor or processor-version resource, not just the processor ID.
- Wrong MIME type: Document AI validates the actual file type.
- Batch output assumptions: `batch_process_documents` writes results to GCS; it does not hand back parsed JSON in memory.
- Stale examples: older 2.x snippets may not match the 3.x request shapes.
- Rolling docs drift: the source URL uses Google Cloud's `latest` reference, which can lag behind the current PyPI package version.

## Version-Sensitive Notes

- PyPI shows `3.11.0` as the current release for `google-cloud-documentai`.
- Use the versioned reference at `https://cloud.google.com/python/docs/reference/documentai/3.11.0` when you need a stable package-specific API surface.
- The official upgrading guide for `3.0.0` documents breaking request and method changes from older releases. If you are adapting 2.x code, review that guide before copying request construction verbatim.
- Google Cloud product guides sometimes use `from google.cloud import documentai_v1` while other samples use `from google.cloud import documentai`. Both refer to the v1 client surface; keep the import style consistent within one codebase.

## Official Sources Used

- Google Cloud Python reference, versioned: `https://cloud.google.com/python/docs/reference/documentai/3.11.0`
- Google Cloud Python reference, rolling: `https://cloud.google.com/python/docs/reference/documentai/latest`
- Changelog: `https://cloud.google.com/python/docs/reference/documentai/3.11.0/changelog`
- Upgrading guide: `https://cloud.google.com/python/docs/reference/documentai/3.11.0/upgrading`
- Client-library processing guide: `https://cloud.google.com/document-ai/docs/process-documents-client-libraries`
- Batch processing sample: `https://cloud.google.com/document-ai/docs/samples/documentai-batch-process-document`
- ADC setup: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- PyPI package page: `https://pypi.org/project/google-cloud-documentai/`

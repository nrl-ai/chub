---
name: dlp
description: "google-cloud-dlp Python client for Cloud Sensitive Data Protection inspection, de-identification, re-identification, jobs, and image redaction"
metadata:
  languages: "python"
  versions: "3.34.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud-dlp,google cloud,dlp,sensitive data protection,pii,python"
---

# google-cloud-dlp Python Package Guide

`google-cloud-dlp` is the official Python client for Google Cloud Sensitive Data Protection (the DLP API). Use it to inspect text or structured data for info types, de-identify or re-identify content, redact images, and run long-running DLP jobs against sources such as Cloud Storage and BigQuery.

This guide is scoped to package version `3.34.0`. The docs URL points at the rolling `latest` reference. As of the official docs page on 2026-03-12, `latest` also resolves to `3.34.0`, but you should still check the changelog before copying older or newer samples.

## Install

```bash
pip install google-cloud-dlp==3.34.0
```

If you are not pinning yet:

```bash
pip install google-cloud-dlp
```

With `uv`:

```bash
uv add google-cloud-dlp==3.34.0
```

## Authentication And Setup

The client uses Google Application Default Credentials (ADC). For local development, the most common setup is:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

If you need explicit credentials from a service account key:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

In deployed environments, prefer attached service accounts and default credentials instead of long-lived key files.

Before calling the client, make sure:

- Enable the DLP API in the target project.
- Build resource parents as `projects/PROJECT_ID/locations/LOCATION`.
- Use `global` for common content inspection calls unless the workload must stay in a specific region.
- Keep templates, job triggers, and jobs in the same location you reference in `parent`.
- If you must override the endpoint, pass `client_options` when creating the client.

## Initialize A Client

```python
import os

from google.cloud import dlp_v2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.getenv("GOOGLE_CLOUD_LOCATION", "global")
parent = f"projects/{project_id}/locations/{location}"

client = dlp_v2.DlpServiceClient()
```

For a regional or custom endpoint:

```python
import os

from google.cloud import dlp_v2

client = dlp_v2.DlpServiceClient(
    client_options={"api_endpoint": os.environ["GOOGLE_CLOUD_DLP_ENDPOINT"]}
)
```

If you need explicit credentials handling, build credentials with `google.auth` and pass them into the client constructor. In most codebases, ADC is the correct default.

## Core Usage

### Inspect Text For Sensitive Data

Use `inspect_content` for inline text or other small payloads already in memory.

```python
from google.cloud import dlp_v2

client = dlp_v2.DlpServiceClient()
parent = "projects/my-project/locations/global"

request = {
    "parent": parent,
    "item": {"value": "Contact alice@example.com or call 415-555-0100."},
    "inspect_config": {
        "info_types": [
            {"name": "EMAIL_ADDRESS"},
            {"name": "PHONE_NUMBER"},
        ],
        "include_quote": True,
        "min_likelihood": dlp_v2.Likelihood.POSSIBLE,
    },
}

response = client.inspect_content(request=request)

for finding in response.result.findings:
    print(finding.info_type.name, finding.quote)
```

Set `include_quote` only when your downstream logic needs the matched text. If you only need counts or metadata, leave it off to reduce exposure of raw sensitive values.

### De-Identify Text

Use `deidentify_content` when you want the service to transform or mask findings instead of just reporting them.

```python
from google.cloud import dlp_v2

client = dlp_v2.DlpServiceClient()
parent = "projects/my-project/locations/global"

request = {
    "parent": parent,
    "item": {"value": "Send the report to alice@example.com."},
    "inspect_config": {
        "info_types": [{"name": "EMAIL_ADDRESS"}],
    },
    "deidentify_config": {
        "info_type_transformations": {
            "transformations": [
                {
                    "primitive_transformation": {
                        "replace_with_info_type_config": {}
                    }
                }
            ]
        }
    },
}

response = client.deidentify_content(request=request)
print(response.item.value)
```

Common alternatives are masking, character replacement, date shifting, bucketing, and crypto transforms. For structured transformation rules, keep the request in a dedicated helper so your inspect and de-identify configs stay consistent.

### Re-Identify Previously Transformed Content

Use `reidentify_content` when you previously applied a reversible transformation such as crypto-based tokenization and need to restore the original value.

```python
transformed_value = "TOKEN(3b6d...)"
original_inspect_config = {
    "info_types": [{"name": "PHONE_NUMBER"}],
}
original_reidentify_config = {
    # Use the same reversible transform family and key material
    # that produced the tokenized value in the first place.
}

request = {
    "parent": parent,
    "item": {"value": transformed_value},
    "inspect_config": original_inspect_config,
    "reidentify_config": original_reidentify_config,
}

response = client.reidentify_content(request=request)
print(response.item.value)
```

Keep the inspect and crypto settings aligned with the original de-identification flow. Re-identification will fail or return unusable output if the transform configuration does not match.

### Redact Sensitive Data From Images

Use `redact_image` for image bytes that may contain faces, text, or embedded sensitive data.

```python
from google.cloud import dlp_v2

client = dlp_v2.DlpServiceClient()
parent = "projects/my-project/locations/global"

with open("document.png", "rb") as fh:
    image_bytes = fh.read()

request = {
    "parent": parent,
    "byte_item": {
        "type_": dlp_v2.ByteContentItem.BytesType.IMAGE,
        "data": image_bytes,
    },
    "inspect_config": {
        "info_types": [{"name": "EMAIL_ADDRESS"}],
    },
    "image_redaction_configs": [
        {"info_type": {"name": "EMAIL_ADDRESS"}}
    ],
}

response = client.redact_image(request=request)

with open("document-redacted.png", "wb") as fh:
    fh.write(response.redacted_image)
```

### Async Client

The async client mirrors the sync surface and is useful when DLP is part of an async service:

```python
from google.cloud import dlp_v2

async_client = dlp_v2.DlpServiceAsyncClient()
response = await async_client.inspect_content(request=request)
```

### Templates, Triggers, And Jobs

For repeated or large-scale workloads:

- Use `create_inspect_template` and `update_inspect_template` to store reusable inspection rules.
- Use `create_deidentify_template` for repeatable transformation configs.
- Use `create_job_trigger` when you need scheduled or recurring scans.
- Use `create_dlp_job` for longer-running scans against storage systems instead of sending large payloads with `inspect_content`.

The general pattern is:

1. Create a template with the inspect or de-identify rules you want to reuse.
2. Reference that template from a job or trigger.
3. Poll job state with `get_dlp_job` or `list_dlp_jobs`.

## Configuration Notes

- `parent` is required on almost every call and must include a location.
- `item` is for inline content already in memory.
- `byte_item` is for raw bytes such as image content.
- Storage scans use storage/job configuration objects instead of `item`.
- Request dictionaries map to protobuf field names, and you can swap them for generated request objects when you want type-checked construction.
- If a Python field name ends with `_`, that usually means the proto field name would otherwise collide with a Python keyword.

## Common Pitfalls

- Do not treat DLP as a generic regex engine. Prefer built-in info types and a deliberate `inspect_config`; add custom detectors only when the built-ins are not enough.
- Do not use `inspect_content` for large files or repository-scale scans. Use DLP jobs for Cloud Storage and BigQuery workloads.
- Do not forget the location segment in resource names. `projects/my-project` is not enough; use `projects/my-project/locations/global` or a regional location.
- Do not mix locations for templates, job triggers, and jobs. Keep the `parent` location consistent across related resources.
- Do not assume findings include the original matched text. Set `include_quote` explicitly when your downstream logic needs it.
- Do not rely on static JSON key files if ADC from attached service accounts is available in the runtime environment.

## Version-Sensitive Notes For `3.34.0`

The official docs root currently lists `3.34.0` as the latest published Python package reference. The changelog entry for `3.34.0` is dated `2026-01-27`.

- `3.34.0` is not called out as a breaking release in the official changelog.
- The changelog notes Python `3.14` support in this release series.
- This release also includes routine dependency and documentation updates. If your environment pins older `protobuf` or related Google client libraries tightly, compare the changelog before upgrading.
- The docs URL is the rolling `latest` branch. Re-check it if you revisit this package later because `latest` will move after `3.34.0`.

If your project is pinned to `3.34.0`, prefer:

- request dictionaries passed as `request={...}` or the generated request types for each method
- checking the changelog before copying setup or transport details from older blog posts
- validating method arguments against the reference for the specific client class you are using

## Official Sources

- Python client reference: https://cloud.google.com/python/docs/reference/dlp/latest
- `DlpServiceClient` reference: https://cloud.google.com/python/docs/reference/dlp/latest/google.cloud.dlp_v2.services.dlp_service.DlpServiceClient
- Changelog: https://cloud.google.com/python/docs/reference/dlp/latest/changelog
- PyPI package page: https://pypi.org/project/google-cloud-dlp/
- Google Cloud authentication overview for ADC: https://cloud.google.com/docs/authentication/provide-credentials-adc

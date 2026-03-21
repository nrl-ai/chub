---
name: service-usage
description: "Google Cloud Service Usage Python client library for listing, enabling, and disabling Google APIs and services"
metadata:
  languages: "python"
  versions: "1.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,service-usage,api-enablement,iam,operations"
---

# Google Cloud Service Usage Python Client Library

## Golden Rule

Use `google-cloud-service-usage` with `from google.cloud import service_usage_v1`, authenticate with Application Default Credentials (ADC), and pass full resource names like `projects/PROJECT_NUMBER/services/SERVICE_NAME`. The mutating methods return long-running operations, so do not treat `enable_service`, `batch_enable_services`, or `disable_service` as immediate.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-service-usage==1.15.0"
```

Common alternatives:

```bash
uv add "google-cloud-service-usage==1.15.0"
poetry add "google-cloud-service-usage==1.15.0"
```

## Authentication And Setup

This client library uses ADC. Prefer these credential sources:

1. Local development: `gcloud auth application-default login`
2. Google Cloud runtime with an attached service account or workload identity
3. `GOOGLE_APPLICATION_CREDENTIALS` only when you cannot use the first two

Typical local setup:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

If you must use a service account key file:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

The caller usually needs a role such as `roles/serviceusage.serviceUsageAdmin` on the target project to enable or disable APIs.

Basic client initialization:

```python
from google.cloud import service_usage_v1

client = service_usage_v1.ServiceUsageClient()
```

Async initialization:

```python
from google.cloud import service_usage_v1

client = service_usage_v1.ServiceUsageAsyncClient()
```

## Core Usage

### Resource names

The library expects Service Usage resource names, not just service IDs:

- Project parent: `projects/PROJECT_NUMBER`
- Service name: `projects/PROJECT_NUMBER/services/SERVICE_NAME`

Example service IDs:

- `bigquery.googleapis.com`
- `run.googleapis.com`
- `serviceusage.googleapis.com`

### Check one service

Use `get_service` when you know the exact service:

```python
from google.cloud import service_usage_v1

project_number = "123456789012"
service_id = "bigquery.googleapis.com"

client = service_usage_v1.ServiceUsageClient()
service = client.get_service(
    request=service_usage_v1.GetServiceRequest(
        name=f"projects/{project_number}/services/{service_id}",
    )
)

print(service.name)
print(service.state.name)
```

### List enabled services

Use `filter="state:ENABLED"` to avoid iterating over every known service:

```python
from google.cloud import service_usage_v1

project_number = "123456789012"
client = service_usage_v1.ServiceUsageClient()

pager = client.list_services(
    request=service_usage_v1.ListServicesRequest(
        parent=f"projects/{project_number}",
        filter="state:ENABLED",
    )
)

for service in pager:
    print(service.config.name, service.state.name)
```

### Enable one service

`enable_service` returns a long-running operation. Wait for completion before assuming the API is usable:

```python
from google.cloud import service_usage_v1

project_number = "123456789012"
service_id = "bigquery.googleapis.com"

client = service_usage_v1.ServiceUsageClient()
operation = client.enable_service(
    request=service_usage_v1.EnableServiceRequest(
        name=f"projects/{project_number}/services/{service_id}",
    )
)

response = operation.result(timeout=300)
print(response.name, response.state.name)
```

### Enable several services in one call

`batch_enable_services` is the fastest path when bootstrapping a project, but the API only allows up to 20 services per request:

```python
from google.cloud import service_usage_v1

project_number = "123456789012"
service_ids = [
    "serviceusage.googleapis.com",
    "iam.googleapis.com",
    "run.googleapis.com",
]

client = service_usage_v1.ServiceUsageClient()
operation = client.batch_enable_services(
    request=service_usage_v1.BatchEnableServicesRequest(
        parent=f"projects/{project_number}",
        service_ids=service_ids,
    )
)

operation.result(timeout=600)
```

### Disable a service

Disabling can fail when other enabled services depend on the target service. Only set `disable_dependent_services=True` when you intend that cascade:

```python
from google.cloud import service_usage_v1

project_number = "123456789012"
service_id = "run.googleapis.com"

client = service_usage_v1.ServiceUsageClient()
operation = client.disable_service(
    request=service_usage_v1.DisableServiceRequest(
        name=f"projects/{project_number}/services/{service_id}",
        disable_dependent_services=False,
    )
)

response = operation.result(timeout=300)
print(response.name, response.state.name)
```

## Configuration Notes

- If your automation only needs to inspect current state, prefer `get_service`, `batch_get_services`, or `list_services` over mutation calls.
- The library also exposes `ServiceUsageAsyncClient` if your app already uses `asyncio`, but the API surface is the same as the sync client.
- The generated clients support the normal Google API Core retry and timeout parameters on each method. Use them explicitly in automation that runs under CI time limits.
- The package supports optional debug logging through the `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` environment variable. Example:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google.cloud.service_usage_v1
```

## Common Pitfalls

- Package name and import path differ: install `google-cloud-service-usage`, import `service_usage_v1`.
- Use the project number in resource names when possible. The official examples use `projects/123456789012`, not a bare project ID.
- `enable_service`, `batch_enable_services`, and `disable_service` are long-running operations. Call `.result()` and handle timeouts.
- `batch_enable_services` accepts at most 20 services per request.
- Service enable/disable requests are quota-limited. The product docs call out a default mutate quota of 2 QPS, so bulk project bootstrap code should throttle and retry.
- `disable_service` can fail unless you set `disable_dependent_services=True` for dependent APIs.
- Enabling or disabling APIs fails with permission errors unless the caller has the required Service Usage IAM role on the target project.

## Version-Sensitive Notes

- PyPI currently publishes `1.15.0`, but some Google Cloud Python reference pages indexed under `latest` still render slightly older generated versions. Use PyPI for the package version you pin, and use the reference pages for current request and response shapes.
- This package is the generated Python client for the Service Usage API. Minor releases often refresh generated surfaces and dependencies without changing the basic client workflow shown above.

## Official Sources

- PyPI project page: `https://pypi.org/project/google-cloud-service-usage/`
- Package docs root: `https://cloud.google.com/python/docs/reference/service-usage/latest`
- ServiceUsageClient reference: `https://cloud.google.com/python/docs/reference/serviceusage/latest/google.cloud.service_usage_v1.services.service_usage.ServiceUsageClient`
- Service enable/disable guide: `https://cloud.google.com/service-usage/docs/enable-disable`
- Service Usage IAM roles: `https://cloud.google.com/service-usage/docs/access-control`
- Repository package root: `https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-service-usage`

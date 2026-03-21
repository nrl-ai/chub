---
name: service-management
description: "Google Cloud Service Management Python client guide for managing services, configs, and rollouts"
metadata:
  languages: "python"
  versions: "1.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,gcp,service-management,api-management,endpoints,long-running-operations"
---

# google-cloud-service-management Python Package Guide

## What This Package Is For

`google-cloud-service-management` is the official Python client for Google Cloud Service Management. Use it when Python code needs to:

- create, fetch, list, or delete managed services
- submit new service configs
- inspect config revisions and rollouts
- validate config source before rollout

The main import surface is:

```python
from google.cloud import servicemanagement_v1
```

For asyncio code, the package also exposes `ServiceManagerAsyncClient`.

## Version-Sensitive Notes

- The initial package metadata version `1.15.0` matches the official PyPI release for this package as verified on March 12, 2026.
- Prefer the Google Cloud Python reference for API shape and request types, and PyPI for installable package version and Python compatibility.
- The docs URL points at the monorepo package directory on GitHub. That repo path is useful for repository context, but the generated Cloud Python reference is the better coding-time source for client methods, request objects, and changelog entries.

## Install

Pin the package when you need reproducible generated-client behavior:

```bash
pip install "google-cloud-service-management==1.15.0"
```

Common alternatives:

```bash
uv add "google-cloud-service-management==1.15.0"
poetry add "google-cloud-service-management==1.15.0"
```

## Authentication And Setup

This library uses Google Application Default Credentials (ADC).

Local development:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="your-producer-project-id"
export SERVICE_NAME="example-api.endpoints.your-producer-project-id.cloud.goog"
```

Service account credentials:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/abs/path/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-producer-project-id"
export SERVICE_NAME="example-api.endpoints.your-producer-project-id.cloud.goog"
```

Important setup points:

- `SERVICE_NAME` is the managed service name, not just the project ID.
- The calling identity needs Service Management permissions in the producer project.
- If you run on Cloud Run, GKE, GCE, or another Google Cloud runtime, prefer the attached workload or service account over key files.

## Initialize A Client

```python
from google.cloud import servicemanagement_v1

client = servicemanagement_v1.ServiceManagerClient()
```

If you need explicit credentials or project discovery:

```python
import google.auth
from google.cloud import servicemanagement_v1

credentials, project_id = google.auth.default(
    scopes=["https://www.googleapis.com/auth/cloud-platform"]
)

client = servicemanagement_v1.ServiceManagerClient(credentials=credentials)
print(project_id)
```

If you must override the endpoint, use `client_options`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import servicemanagement_v1

client = servicemanagement_v1.ServiceManagerClient(
    client_options=ClientOptions(
        api_endpoint="servicemanagement.googleapis.com"
    )
)
```

## Resource Naming

Most operations use the managed service name directly:

```python
import os

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
service_name = os.environ["SERVICE_NAME"]
```

Typical values look like:

- `example-api.endpoints.my-project.cloud.goog`
- another verified managed service name owned by the producer project

Config IDs and rollout IDs are returned by the API and are used in later calls such as `get_service_config(...)` and `get_service_rollout(...)`.

## List Managed Services

`list_services()` returns a pager. Iterate over it directly.

```python
import os

from google.cloud import servicemanagement_v1

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]

client = servicemanagement_v1.ServiceManagerClient()

for service in client.list_services(producer_project_id=project_id):
    print(service.service_name, service.producer_project_id)
```

## Get One Managed Service

```python
import os

from google.cloud import servicemanagement_v1

service_name = os.environ["SERVICE_NAME"]

client = servicemanagement_v1.ServiceManagerClient()
service = client.get_service(service_name=service_name)

print(service.service_name)
print(service.producer_project_id)
```

## Create A Managed Service

Creating or deleting a service returns a long-running operation. Wait for `.result()` before assuming the change is visible.

```python
import os

from google.cloud import servicemanagement_v1

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
service_name = os.environ["SERVICE_NAME"]

client = servicemanagement_v1.ServiceManagerClient()
operation = client.create_service(
    request=servicemanagement_v1.CreateServiceRequest(
        service=servicemanagement_v1.ManagedService(
            service_name=service_name,
            producer_project_id=project_id,
        )
    )
)

created = operation.result(timeout=300)
print(created.service_name)
```

## Validate And Submit Config Source

When you are pushing a new service configuration, the most practical flow is:

1. build a `ConfigSource` from your YAML or OpenAPI files
2. call `generate_config_report(...)` if you want diagnostics before rollout
3. call `submit_config_source(...)` to create a new config revision

Example submission:

```python
import os
from pathlib import Path

from google.cloud import servicemanagement_v1

service_name = os.environ["SERVICE_NAME"]
yaml_bytes = Path("openapi/service.yaml").read_bytes()

client = servicemanagement_v1.ServiceManagerClient()
operation = client.submit_config_source(
    request=servicemanagement_v1.SubmitConfigSourceRequest(
        service_name=service_name,
        config_source=servicemanagement_v1.ConfigSource(
            files=[
                {
                    "file_path": "openapi/service.yaml",
                    "file_contents": yaml_bytes,
                }
            ]
        ),
    )
)

config = operation.result(timeout=300)
print(config.id)
```

Important details:

- `file_contents` must be bytes. Use `Path(...).read_bytes()`, not a Python string.
- `submit_config_source(...)` is also a long-running operation.
- Prefer request objects when there are multiple nested fields or when you need version-stable clarity.

## Inspect Config Revisions

Use `list_service_configs(...)` when you need the config IDs created by previous submissions:

```python
import os

from google.cloud import servicemanagement_v1

service_name = os.environ["SERVICE_NAME"]

client = servicemanagement_v1.ServiceManagerClient()

for config in client.list_service_configs(service_name=service_name):
    print(config.id, config.title)
```

Fetch one specific config when you already have the config ID:

```python
import os

from google.cloud import servicemanagement_v1

service_name = os.environ["SERVICE_NAME"]
config_id = "2026-03-12r0"

client = servicemanagement_v1.ServiceManagerClient()
config = client.get_service_config(
    service_name=service_name,
    config_id=config_id,
)

print(config.id)
print(config.name)
```

## Inspect Rollouts

```python
import os

from google.cloud import servicemanagement_v1

service_name = os.environ["SERVICE_NAME"]

client = servicemanagement_v1.ServiceManagerClient()

for rollout in client.list_service_rollouts(service_name=service_name):
    print(rollout.rollout_id, rollout.status)
```

Use `get_service_rollout(...)` when you need one rollout by ID:

```python
import os

from google.cloud import servicemanagement_v1

service_name = os.environ["SERVICE_NAME"]
rollout_id = "current"

client = servicemanagement_v1.ServiceManagerClient()
rollout = client.get_service_rollout(
    service_name=service_name,
    rollout_id=rollout_id,
)

print(rollout.rollout_id)
print(rollout.status)
```

## Delete A Managed Service

```python
import os

from google.cloud import servicemanagement_v1

service_name = os.environ["SERVICE_NAME"]

client = servicemanagement_v1.ServiceManagerClient()
operation = client.delete_service(service_name=service_name)
operation.result(timeout=300)
```

## Async Client

For asyncio code, use `ServiceManagerAsyncClient`:

```python
import asyncio
import os

from google.cloud import servicemanagement_v1

async def main() -> None:
    service_name = os.environ["SERVICE_NAME"]
    client = servicemanagement_v1.ServiceManagerAsyncClient()
    service = await client.get_service(service_name=service_name)
    print(service.service_name)

asyncio.run(main())
```

## Common Pitfalls

- Confusing the package name with the import path. Use `from google.cloud import servicemanagement_v1`.
- Passing a project ID where the API expects a managed service name.
- Forgetting that mutating calls return long-running operations.
- Passing text instead of bytes for config file contents.
- Treating this package like a runtime gateway SDK. It manages service definitions, configs, and rollouts; it does not proxy or serve application traffic.

## Official Sources

- Google Cloud Python reference: https://cloud.google.com/python/docs/reference/servicemanagement/latest
- `ServiceManagerClient` reference: https://cloud.google.com/python/docs/reference/servicemanagement/latest/google.cloud.servicemanagement_v1.services.service_manager.ServiceManagerClient
- Changelog: https://cloud.google.com/python/docs/reference/servicemanagement/latest/changelog
- ADC setup: https://cloud.google.com/docs/authentication/provide-credentials-adc
- PyPI package page: https://pypi.org/project/google-cloud-service-management/
- Docs URL provided for this package: https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-service-management

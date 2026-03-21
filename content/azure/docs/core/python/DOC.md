---
name: core
description: "azure-core package guide for Python - shared pipeline, credentials, retries, transports, and errors for Azure SDK clients"
metadata:
  languages: "python"
  versions: "1.38.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-core,python,azure-sdk,pipeline,credentials,retries,http"
---

# azure-core Python Package Guide

## What This Package Is For

`azure-core` is the shared runtime layer used by Azure SDK client libraries. It provides:

- sync and async pipeline clients
- HTTP request and response types
- credential protocols and auth policies
- retries, redirects, logging, tracing, and transport abstractions
- shared exception types

In most application code, you do not install or import `azure-core` directly. You usually install a service package such as `azure-storage-blob`, `azure-keyvault-secrets`, or `azure-mgmt-resource`, and that package brings in `azure-core`.

Use `azure-core` directly when you need one of these:

- a custom client for an Azure REST API that does not already have a generated SDK
- a custom pipeline policy or transport setting
- shared retry, logging, or auth behavior across multiple Azure SDK clients
- direct handling of Azure SDK exception types

## Install And Environment

PyPI currently lists `azure-core` `1.38.2`, released on `2026-02-18`, with `Requires: Python >=3.9`.

Install the exact version you want to target:

```bash
pip install azure-core==1.38.2
```

Common alternatives:

```bash
poetry add azure-core==1.38.2
uv add azure-core==1.38.2
```

Install companion packages only when you need them:

```bash
pip install azure-identity
pip install aiohttp
pip install "azure-core[tracing]"
```

Use them for:

- `azure-identity`: Entra ID credentials such as `DefaultAzureCredential`
- `aiohttp`: async transport for `AsyncPipelineClient`
- `azure-core[tracing]`: optional OpenTelemetry dependency for native tracing

## Minimal Direct Usage

The main low-level surface is `PipelineClient` plus `HttpRequest`.

```python
from azure.core import PipelineClient
from azure.core.rest import HttpRequest

client = PipelineClient(base_url="https://management.azure.com")

request = HttpRequest(
    "GET",
    "/subscriptions",
    params={"api-version": "2022-12-01"},
    headers={"Accept": "application/json"},
)

response = client.send_request(request)
response.raise_for_status()
payload = response.json()
print(payload)
```

Practical notes:

- `base_url` can be a service root; request URLs can be relative or absolute.
- `send_request()` returns a response object and does not do non-2xx error handling for you.
- Call `response.raise_for_status()` yourself or catch `HttpResponseError`.
- Prefer a service-specific SDK when one exists. Direct `PipelineClient` usage is for low-level or custom scenarios.

## Authentication And Credentials

### Recommended Azure Auth Pattern

Microsoft recommends token-based authentication through Microsoft Entra ID over connection strings or shared keys. In app code, that usually means `DefaultAzureCredential` from `azure-identity`.

```python
from azure.core import PipelineClient
from azure.core.pipeline.policies import BearerTokenCredentialPolicy, RetryPolicy
from azure.core.rest import HttpRequest
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential()

client = PipelineClient(
    base_url="https://management.azure.com",
    policies=[
        BearerTokenCredentialPolicy(
            credential,
            "https://management.azure.com/.default",
        ),
        RetryPolicy(retry_total=5, retry_backoff_factor=0.8),
    ],
)

request = HttpRequest(
    "GET",
    "/subscriptions",
    params={"api-version": "2022-12-01"},
)

response = client.send_request(request)
response.raise_for_status()
```

Use the scope required by the target service. Management-plane clients commonly use `https://management.azure.com/.default`; data-plane services usually use a different audience.

For deployment:

- on Azure, prefer managed identity
- in local development, use developer credentials or a service principal
- avoid hard-coded secrets and connection strings when Entra ID is supported

### AzureKeyCredential And Rotating Keys

`AzureKeyCredential`, `AzureNamedKeyCredential`, and `AzureSasCredential` are credential containers. They are useful because you can rotate them without recreating a long-lived client, but raw `PipelineClient` does not automatically know how to apply them to requests.

Example with a custom API-key policy:

```python
from azure.core import PipelineClient
from azure.core.credentials import AzureKeyCredential
from azure.core.pipeline.policies import SansIOHTTPPolicy
from azure.core.rest import HttpRequest

class ApiKeyPolicy(SansIOHTTPPolicy):
    def __init__(self, credential: AzureKeyCredential):
        self._credential = credential

    def on_request(self, request):
        request.http_request.headers["x-api-key"] = self._credential.key

credential = AzureKeyCredential("initial-key")
client = PipelineClient(
    base_url="https://example.contoso.com",
    per_call_policies=[ApiKeyPolicy(credential)],
)

request = HttpRequest("GET", "/widgets")
response = client.send_request(request)
response.raise_for_status()

# Rotate the key without recreating the credential object.
credential.update("rotated-key")
```

If you are using a service-specific SDK client, prefer its documented `credential=` parameter instead of manually wiring a policy.

## Async Usage

Use `AsyncPipelineClient` for low-level async calls. If you do not pass a transport, `AioHttpTransport` is used for asynchronous transport, so `aiohttp` must be installed.

```python
import asyncio

from azure.core import AsyncPipelineClient
from azure.core.rest import HttpRequest

async def main():
    async with AsyncPipelineClient(
        base_url="https://management.azure.com"
    ) as client:
        request = HttpRequest(
            "GET",
            "/subscriptions",
            params={"api-version": "2022-12-01"},
        )
        response = await client.send_request(request)
        response.raise_for_status()
        print(response.json())

asyncio.run(main())
```

Keep sync and async stacks separate:

- `PipelineClient` defaults to `RequestsTransport`
- `AsyncPipelineClient` defaults to `AioHttpTransport`
- do not mix async transports or async credentials into sync clients, or the reverse, unless the upstream policy docs explicitly allow it

## Retry, Timeout, And Pipeline Configuration

`PipelineClient` and service SDK clients expose the knobs agents most often need:

- `retry_total`
- `retry_connect`
- `retry_read`
- `retry_status`
- `retry_backoff_factor`
- `retry_backoff_max`
- `retry_on_status_codes`
- `timeout`
- `connection_timeout`
- `read_timeout`
- `connection_verify`
- `connection_cert`
- `proxies`
- `headers`
- `logging_enable`

Custom retry example:

```python
from azure.core import PipelineClient
from azure.core.pipeline.policies import RetryPolicy

retry_policy = RetryPolicy(
    retry_total=5,
    retry_connect=2,
    retry_read=2,
    retry_status=3,
    retry_backoff_factor=0.5,
    retry_backoff_max=30,
    retry_on_status_codes=[408, 429, 500, 502, 503, 504],
)

client = PipelineClient(
    base_url="https://example.contoso.com",
    policies=[retry_policy],
    connection_timeout=10,
    read_timeout=30,
)
```

Practical guidance:

- use default retry behavior unless you have a concrete reason to change it
- `retry_read` can repeat requests that already reached the service, so be careful with non-idempotent operations
- retries add latency; keep them tighter for interactive paths than for background jobs
- if you need a custom policy before retries, use `per_call_policies`
- if you need a custom policy after retries, use `per_retry_policies`
- reuse client instances so transports can pool connections

## Logging And Diagnostics

Azure SDK libraries use Python `logging`.

```python
import logging
import sys

azure_logger = logging.getLogger("azure")
azure_logger.setLevel(logging.DEBUG)
azure_logger.addHandler(logging.StreamHandler(stream=sys.stdout))
```

Useful logger names:

- `azure` for broad SDK diagnostics
- `azure.core` for core pipeline behavior
- `azure.identity` if token acquisition is relevant
- `azure.core.pipeline.policies.http_logging_policy` for HTTP-level details

Be careful with `logging_enable=True`: request and response metadata can be emitted to logs. Treat that output as sensitive.

## Exceptions You Will Actually Catch

Most application code should catch specific `azure.core.exceptions` types:

```python
from azure.core.exceptions import (
    AzureError,
    ClientAuthenticationError,
    HttpResponseError,
    ResourceExistsError,
    ResourceNotFoundError,
    ServiceRequestError,
    ServiceResponseError,
)

try:
    response = client.send_request(request)
    response.raise_for_status()
except ClientAuthenticationError:
    ...
except ResourceNotFoundError:
    ...
except ResourceExistsError:
    ...
except HttpResponseError as exc:
    if exc.status_code == 429:
        ...
    raise
except (ServiceRequestError, ServiceResponseError):
    ...
except AzureError:
    ...
```

What they usually mean:

- `ServiceRequestError`: the request never reached the service
- `ServiceResponseError`: the request was sent but the client could not process the response
- `HttpResponseError`: the service returned a non-success status
- `ClientAuthenticationError`: authentication failed

Operational guidance:

- do not retry `401`, `403`, or most `400` errors until you fix the request or permissions
- `404` is usually not retryable unless the resource is expected to appear shortly
- consider retrying `408`, `429`, `500`, `502`, `503`, and `504`
- for support cases, capture `x-ms-request-id` from the response headers when available

## Common Pitfalls

- Do not assume `azure-core` is the main entry point for Azure services. Most application code should use the service package plus `azure-identity`.
- `PipelineClient.send_request()` and `AsyncPipelineClient.send_request()` do not raise on non-2xx by themselves.
- `AzureKeyCredential` is only a rotating secret holder. It does not inject headers into a raw `PipelineClient` unless you add a policy.
- The Learn API pages are rolling current docs, not frozen historical snapshots. For exact behavior changes, check the PyPI release history and the package changelog.
- Async transport support is opt-in. Install `aiohttp` before using `AsyncPipelineClient`.
- Built-in retry guidance for service clients and low-level `azure-core` configuration docs are not always presented the same way. When retry behavior matters, inspect the client you are actually constructing instead of assuming all defaults are identical.
- Be careful with streamed responses. Accessing content or streams in the wrong order can raise `ResponseNotReadError`, `StreamConsumedError`, or `StreamClosedError`.

## Version-Sensitive Notes For 1.38.2

- `1.38.2` fixes `PipelineClient.format_url()` so URL templates starting with `/?` preserve the leading slash.
- `1.38.1` fixes another `PipelineClient.format_url()` edge case for query-only URL templates.
- `1.38.0` changed continuation token format. Tokens generated by earlier `azure-core` versions are not compatible with `1.38.x`.
- `1.35.1` fixed a bug where `retry_backoff_max` could be ignored in `RetryPolicy` and `AsyncRetryPolicy`. If you rely on custom max backoff limits, older examples may behave differently.
- `1.34.0` dropped Python 3.8 support. Use Python 3.9+.
- `1.33.0` added native OpenTelemetry tracing support and the optional `azure-core[tracing]` extra.

## Official Context

- Docs root: https://learn.microsoft.com/en-us/python/api/azure-core/
- API overview: https://learn.microsoft.com/en-us/python/api/overview/azure/core-readme?view=azure-python
- PipelineClient reference: https://learn.microsoft.com/en-us/python/api/azure-core/azure.core.pipelineclient?view=azure-python
- AsyncPipelineClient reference: https://learn.microsoft.com/en-us/python/api/azure-core/azure.core.asyncpipelineclient?view=azure-python
- AzureKeyCredential reference: https://learn.microsoft.com/en-us/python/api/azure-core/azure.core.credentials.azurekeycredential?view=azure-python
- Exceptions reference: https://learn.microsoft.com/en-us/python/api/azure-core/azure.core.exceptions?view=azure-python
- Azure authentication overview: https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication/overview
- Azure retry guidance: https://learn.microsoft.com/en-us/azure/developer/python/sdk/fundamentals/http-pipeline-retries
- Azure error-handling guidance: https://learn.microsoft.com/en-us/azure/developer/python/sdk/fundamentals/errors
- Package registry: https://pypi.org/project/azure-core/
- Release history: https://pypi.org/project/azure-core/#history
- Source changelog: https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/core/azure-core/CHANGELOG.md

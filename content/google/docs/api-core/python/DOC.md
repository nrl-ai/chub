---
name: api-core
description: "google-api-core Python package guide for shared Google client helpers such as auth handoff, client options, retries, exceptions, pagination, and long-running operations"
metadata:
  languages: "python"
  versions: "2.30.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,python,auth,retry,pagination,long-running-operations"
---

# google-api-core Python Package Guide

## Golden Rule

Use `google-api-core` as shared infrastructure behind a real Google client library, not as a standalone SDK. Install the service-specific package you actually need, import helpers from `google.api_core`, and prefer Application Default Credentials (ADC) or an explicit credentials object over file-path shortcuts in `ClientOptions`.

## When To Use It

`google-api-core` provides the common behavior used by generated Google clients:

- auth handoff into client constructors
- `ClientOptions` for endpoint, mTLS, scopes, quota project, API key, and universe domain
- retry helpers and retry predicates
- normalized exception classes
- iterator wrappers for paginated APIs
- long-running operation futures

If you need service methods, install the service package too. For example, use `google-cloud-secret-manager`, `google-cloud-pubsub`, or `google-cloud-bigquery` for actual API calls.

## Install

Pin the version your project expects:

```bash
python -m pip install "google-api-core==2.30.0"
```

Useful extras from PyPI:

```bash
python -m pip install "google-api-core[grpc]==2.30.0"
python -m pip install "google-api-core[async-rest]==2.30.0"
```

Common alternatives:

```bash
uv add "google-api-core==2.30.0"
poetry add "google-api-core==2.30.0"
```

Notes:

- `grpc` is relevant when the client library uses gRPC transports.
- `async-rest` is relevant for async REST transports in newer generated clients.
- Many Google packages pull `google-api-core` transitively, but explicit pinning is useful when your project depends on retry, paging, or operation behavior.

## Authentication And Setup

The official auth guide for `google-api-core` expects the same Google Cloud patterns used by generated clients:

1. On Google Cloud runtimes, ADC usually works automatically.
2. For local development, use `gcloud auth application-default login`.
3. Outside Google Cloud, use a service account credential object or set `GOOGLE_APPLICATION_CREDENTIALS`.

Minimal local setup:

```bash
gcloud auth application-default login
```

Service-account setup:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

Explicit credentials object:

```python
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json",
    scopes=["https://www.googleapis.com/auth/cloud-platform"],
)
```

Generated clients normally accept either no credentials argument, which triggers ADC, or `credentials=...`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import secretmanager_v1
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json",
    scopes=["https://www.googleapis.com/auth/cloud-platform"],
)

client = secretmanager_v1.SecretManagerServiceClient(
    credentials=credentials,
    client_options=ClientOptions(quota_project_id="billing-project"),
)
```

## ClientOptions

`ClientOptions` is the main configuration object shared across Google generated clients.

Typical uses:

- `api_endpoint`: regional endpoint, private service endpoint, emulator-style host when supported
- `quota_project_id`: charge quota and billing to a different project
- `scopes`: override OAuth scopes when the client supports it
- `api_key`: only for APIs that explicitly support API keys
- `client_cert_source` or `client_encrypted_cert_source`: mTLS client certificate configuration
- `universe_domain`: non-default Google universe domain when the service supports it

Example:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import secretmanager_v1

client = secretmanager_v1.SecretManagerServiceClient(
    client_options=ClientOptions(
        api_endpoint="secretmanager.googleapis.com",
        quota_project_id="billing-project",
        universe_domain="googleapis.com",
    )
)
```

Important behavior:

- `api_endpoint` wins if both `api_endpoint` and `universe_domain` are set.
- `credentials_file` exists today but is deprecated and will be removed in the next major `google-api-core` release.
- `credentials_file` and `api_key` are mutually exclusive.
- `client_cert_source` and `client_encrypted_cert_source` are mutually exclusive.

## Core Usage Patterns

### Pass retry, timeout, and metadata on RPCs

Most GAPIC methods accept `retry=`, `timeout=`, and `metadata=` keyword arguments.

```python
from google.api_core import exceptions, retry
from google.cloud import secretmanager_v1

client = secretmanager_v1.SecretManagerServiceClient()

rpc_retry = retry.Retry(
    predicate=retry.if_exception_type(
        exceptions.TooManyRequests,
        exceptions.ServiceUnavailable,
        exceptions.DeadlineExceeded,
    ),
    initial=1.0,
    maximum=30.0,
    multiplier=2.0,
    timeout=120.0,
)

response = client.access_secret_version(
    request={"name": "projects/my-project/secrets/my-secret/versions/latest"},
    retry=rpc_retry,
    timeout=10.0,
    metadata=[("x-goog-user-project", "billing-project")],
)
```

Use `timeout` for per-call timing. Use `retry.timeout` for total retry budget. In current docs, `deadline` still works for backward compatibility, but it is deprecated in favor of `timeout`.

### Catch normalized Google API exceptions

Generated clients convert transport-specific errors into `google.api_core.exceptions.*` types.

```python
from google.api_core import exceptions

try:
    response = client.access_secret_version(
        request={"name": secret_version_name},
        retry=rpc_retry,
        timeout=10.0,
    )
except exceptions.NotFound:
    print("Secret version does not exist")
except exceptions.PermissionDenied:
    print("Caller does not have the required IAM permission")
except exceptions.ResourceExhausted:
    print("Quota or rate limit exceeded")
except exceptions.GoogleAPICallError as exc:
    print(f"Unexpected Google API failure: {exc}")
```

`RetryError` is separate: it means the retry policy exhausted its budget while the underlying exception kept matching the retry predicate.

### Work with pagers

List methods in generated clients often return pager objects backed by `google.api_core.page_iterator`.

```python
from google.cloud import secretmanager_v1

client = secretmanager_v1.SecretManagerServiceClient()
pager = client.list_secrets(request={"parent": "projects/my-project"})

for secret in pager:
    print(secret.name)
```

If you need page-level handling instead of item-level iteration:

```python
for page in pager.pages:
    print("page start")
    for secret in page:
        print(secret.name)
```

Notes:

- Iteration is lazy; additional RPCs happen as pages are consumed.
- The core library exposes both HTTP and gRPC iterator implementations. Generated clients usually hide those details behind pager objects.

### Wait for long-running operations

Operations returned by Google clients are wrapped as `google.api_core.operation.Operation` or async equivalents.

```python
operation = client.some_long_running_method(request={...})

result = operation.result(timeout=300)
print(result)

if operation.metadata is not None:
    print(operation.metadata)
```

Useful operation methods:

- `result(timeout=...)`
- `done()`
- `cancel()`
- `exception()`
- `add_done_callback(...)`

Polling behavior is controlled by a retry-style polling configuration internally. For current releases, prefer the `polling` argument over the older `retry` argument when you are working directly with `google.api_core.operation.Operation`.

### Async retry helpers

For async clients and async transports, use `google.api_core.retry_async.AsyncRetry` instead of the sync `Retry` helper:

```python
from google.api_core import exceptions, retry_async

async_retry = retry_async.AsyncRetry(
    predicate=retry_async.if_exception_type(
        exceptions.TooManyRequests,
        exceptions.ServiceUnavailable,
    ),
    timeout=60.0,
)
```

If a generated async client already has default retry behavior, pass your own `retry=` only when you need different backoff or failure handling.

## Common Pitfalls

- `google-api-core` does not expose product-specific methods. You still need the real service client package.
- The pip package name is `google-api-core`, but the import path is `google.api_core`.
- Do not assume API keys work for every Google API. Many Google Cloud clients require OAuth credentials or service-account credentials.
- Avoid `ClientOptions(credentials_file=...)` in new code. Current docs mark it deprecated; prefer ADC or an explicit credentials object.
- `retry.timeout` is not the same as per-request `timeout`. The first controls total retry budget; the second controls a single RPC attempt.
- Pagers are lazy. Calling `list(pager)` can trigger many requests and load every item into memory.
- Regional and private endpoints are service-specific. Only set `api_endpoint` when the service documentation tells you what host to use.
- `universe_domain` must match the credentials' universe domain. Do not set it casually.

## Version-Sensitive Notes For 2.30.0

- As of March 12, 2026, the version used here `2.30.0` matches the current PyPI release.
- The official docs root is a `latest` URL, not a version-pinned documentation tree. Use PyPI and the changelog to validate version-specific behavior.
- The official changelog for `2.30.0` records two notable changes: Python `>=3.9` is now required, and the minimum supported `protobuf` version is `4.25.8`.
- The `2.25.2` changelog marks `ClientOptions.credentials_file` as deprecated. Do not build new patterns around it even though the parameter still exists in current docs.
- `google-api-core` remains stable at the package level, but retry defaults, transport support, and generated-client behavior can still shift across Google service packages. Check the service package docs when behavior seems inconsistent.

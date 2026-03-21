---
name: core
description: "google-cloud-core package guide for Python with ADC setup, project resolution, client base classes, and version-aware notes"
metadata:
  languages: "python"
  versions: "2.5.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud-core,gcp,google-cloud,auth,adc,project,client"
---

# google-cloud-core Python Package Guide

## What It Is

`google-cloud-core` is the shared foundation package used by Google Cloud Python client libraries. It provides common client base classes, project and credential discovery behavior, and shared configuration conventions. It is not a product SDK by itself.

Use a service package for actual API work:

- `google-cloud-storage`
- `google-cloud-bigquery`
- `google-cloud-pubsub`

Install `google-cloud-core` directly when you are debugging dependency issues, building thin wrappers around Google Cloud clients, or working with the common `Client` / `ClientWithProject` base behavior.

## Install

Pin the version you actually want to reason about:

```bash
python -m pip install google-cloud-core==2.5.0
```

In most application code you should install the service-specific client instead and let it pull in `google-cloud-core` transitively.

## Core API Surface

The package surface that matters most in real code:

- `google.cloud.client.Client`: shared base client with credential loading, optional custom HTTP transport, and `client_options`
- `google.cloud.client.ClientWithProject`: `Client` plus explicit or inferred project handling
- `google.cloud.environment_vars`: constants for shared Google Cloud client environment variables

Practical rule:

- Use `Client` when you need to inspect or reuse base auth / transport behavior.
- Use `ClientWithProject` when your code must carry a project id explicitly.
- Use a service client for actual API methods.

## Authentication And Project Setup

`google-cloud-core` follows Google Cloud Application Default Credentials (ADC). For reliable automation:

1. Use ADC for local development and managed runtime environments.
2. Pass `project=` explicitly when project selection must be deterministic.
3. Pass explicit `credentials=` in tests, CLIs, and wrappers when ambient machine state is unreliable.

### Local ADC

```bash
gcloud auth application-default login
```

### Service Account Credentials File

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

### Explicit Credentials In Code

```python
from google.cloud.client import ClientWithProject
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = ClientWithProject(
    project="my-gcp-project",
    credentials=credentials,
)

print(client.project)
```

The official configuration docs list `GOOGLE_CLOUD_PROJECT` as the environment-variable override for default project detection. Prefer passing `project=` directly when behavior must not depend on the shell or runtime environment.

## Core Usage

### Inspect Base Credential Resolution

```python
from google.cloud.client import Client

client = Client()

print(type(client._credentials).__name__)
client.close()
```

This is useful for debugging what a larger Google Cloud Python stack will pick up from ADC, workload identity, or local developer credentials.

### Carry Project Context Explicitly

```python
from google.cloud.client import ClientWithProject

client = ClientWithProject(project="my-gcp-project")

print(client.project)
client.close()
```

Use this when your wrapper or test helper should not rely on inferred project discovery.

### Supply A Custom HTTP Session

If you pass `_http`, it should already be an authenticated session with a `request()` method.

```python
from google.auth.transport.requests import AuthorizedSession
from google.cloud.client import Client
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)
session = AuthorizedSession(credentials)

client = Client(_http=session)

try:
    response = client._http.request("GET", "https://example.com")
finally:
    client.close()
```

Pitfall:

- When `_http` is supplied, you are responsible for giving the client a ready-to-use session. Do not assume `credentials=` will also be applied to that custom transport.

### Use `client_options` For Endpoint Overrides

```python
from google.api_core.client_options import ClientOptions
from google.cloud.client import Client

client = Client(
    client_options=ClientOptions(
        api_endpoint="https://example.googleapis.com"
    )
)

client.close()
```

This matters most for libraries built on top of `google-cloud-core`; whether a particular option is honored depends on the concrete service client.

## Configuration And Environment Variables

The current official config docs highlight these shared settings:

- `GOOGLE_APPLICATION_CREDENTIALS`: credentials file path for ADC
- `GOOGLE_CLOUD_PROJECT`: override the default project id
- `GOOGLE_CLOUD_DISABLE_GRPC`: disables gRPC in dual-transport libraries that honor it

Import the shared constant names if you need them programmatically:

```python
from google.cloud import environment_vars

print(environment_vars.CREDENTIALS)
print(environment_vars.PROJECT)
print(environment_vars.DISABLE_GRPC)
```

Practical guidance:

- Prefer explicit `project=` over relying on `GOOGLE_CLOUD_PROJECT`.
- Prefer explicit credentials in automation over ambient ADC state.
- Treat transport-related environment variables as service-library specific; confirm behavior in the service client's own docs before relying on them.

## Common Pitfalls

- Do not treat `google-cloud-core` as a service SDK. It does not give you product methods like bucket, dataset, or subscription operations.
- Do not anchor new work to older `googleapis.dev` examples alone. The current canonical docs are under `docs.cloud.google.com`.
- Do not depend on project inference in tests if the project matters. Pass `project=` explicitly.
- Do not pass a custom `_http` object unless it is already authenticated and implements the expected request interface.
- Do not assume every Google Cloud Python package honors the same environment variables or transport toggles in the same way. Confirm service-specific behavior upstream.

## Version-Sensitive Notes

- The upstream changelog for `2.5.0` notes Python `3.14` support. If you are validating interpreter compatibility, use `2.5.0` or newer as the baseline for Python 3.14 environments.
- The upstream changelog for `2.4.2` notes that `Client` passes `client_options.api_key` to the auth library. If you depend on API-key-based configuration, make sure you are not reasoning from older `2.4.1` behavior.
- The canonical reference docs currently serve `latest` rather than a version-pinned `2.5.0` tree. Keep your lockfile authoritative if you are debugging a project pinned to an older transitive dependency set.

## Official Sources

- Docs root: `https://docs.cloud.google.com/python/docs/reference/google-cloud-core/latest`
- `Client` reference: `https://docs.cloud.google.com/python/docs/reference/google-cloud-core/latest/google.cloud.client.Client`
- `ClientWithProject` reference: `https://docs.cloud.google.com/python/docs/reference/google-cloud-core/latest/google.cloud.client.ClientWithProject`
- Configuration guide: `https://cloud.google.com/python/docs/reference/google-cloud-core/latest/config`
- Changelog: `https://cloud.google.com/python/docs/reference/google-cloud-core/latest/changelog`
- PyPI: `https://pypi.org/project/google-cloud-core/`
- ADC setup: `https://cloud.google.com/docs/authentication/application-default-credentials`

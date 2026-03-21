---
name: secret-manager
description: "Google Cloud Secret Manager Python client for creating secrets, adding versions, accessing payloads, and managing version state"
metadata:
  languages: "python"
  versions: "2.26.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,secret-manager,gcp,secrets,iam,adc,python"
---

# google-cloud-secret-manager Python Package Guide

Use `google-cloud-secret-manager` for Python code that needs to create secrets, add secret versions, read secret payloads, list secrets, and manage secret version state in Google Cloud Secret Manager.

## Golden Rule

- Install and import the official client: `from google.cloud import secretmanager`.
- Prefer Application Default Credentials (ADC). For local development, use `gcloud auth application-default login` or set `GOOGLE_APPLICATION_CREDENTIALS`.
- Secret values are immutable per version. To rotate a secret, add a new version instead of editing an existing one.
- Use an explicit version number when rollout control matters. `latest` is convenient, but it always resolves to the newest created version.
- Treat the secret payload as bytes and avoid logging plaintext secrets.

## Version Covered

- Package: `google-cloud-secret-manager`
- Ecosystem: `pypi`
- Version: `2.26.0`
- Python requirement from PyPI: `>=3.7`
- Registry: https://pypi.org/project/google-cloud-secret-manager/
- Docs root used for this guide: https://docs.cloud.google.com/python/docs/reference/secretmanager/latest

## Install

Pin the package version you want the agent to target:

```bash
python -m pip install google-cloud-secret-manager==2.26.0
```

If you verify payload integrity with CRC32C checksums, install `google-crc32c` too:

```bash
python -m pip install google-crc32c
```

## Authentication And Setup

This client uses Google Cloud credentials, not API keys.

For local development:

```bash
gcloud auth application-default login
```

For a service account JSON file:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="my-project-id"
```

Minimum setup:

1. Enable the Secret Manager API in the target Google Cloud project.
2. Make sure ADC resolves to the principal you expect.
3. Grant the narrowest IAM role that matches the operation.

Common IAM roles from the product docs:

- `roles/secretmanager.secretAccessor` for reading secret payloads
- `roles/secretmanager.secretVersionAdder` for adding versions to an existing secret
- `roles/secretmanager.secretVersionManager` for enabling, disabling, listing, and destroying versions
- `roles/secretmanager.admin` for creating and fully managing secrets

On Compute Engine and GKE, Secret Manager client-library auth also depends on the underlying runtime having the `cloud-platform` access scope when metadata-based credentials are used.

## Initialize The Client

The standard sync client picks up ADC automatically:

```python
from google.cloud import secretmanager

client = secretmanager.SecretManagerServiceClient()
```

The generated reference also exposes `client_options` and `transport` when you need non-default behavior:

```python
from google.cloud import secretmanager

client = secretmanager.SecretManagerServiceClient(
    transport="rest",
)
```

Use the helper methods for resource names instead of concatenating strings by hand:

```python
from google.cloud import secretmanager

client = secretmanager.SecretManagerServiceClient()

secret_name = client.secret_path("my-project", "db-password")
version_name = client.secret_version_path("my-project", "db-password", "latest")
```

## Core Usage

### Create A Secret

A secret is the metadata container. The actual secret bytes live in secret versions.

```python
from google.cloud import secretmanager

def create_secret(project_id: str, secret_id: str) -> str:
    client = secretmanager.SecretManagerServiceClient()
    parent = f"projects/{project_id}"

    secret = client.create_secret(
        request={
            "parent": parent,
            "secret_id": secret_id,
            "secret": {
                "replication": {
                    "automatic": {},
                }
            },
        }
    )
    return secret.name
```

Keep the replication policy intentional. The docs note that replication configuration is immutable after creation.

### Add A Secret Version

Rotating a secret means adding a new version.

```python
from google.cloud import secretmanager
import google_crc32c

def add_secret_version(project_id: str, secret_id: str, value: str) -> str:
    client = secretmanager.SecretManagerServiceClient()
    parent = client.secret_path(project_id, secret_id)

    payload = value.encode("utf-8")
    checksum = google_crc32c.Checksum()
    checksum.update(payload)

    version = client.add_secret_version(
        request={
            "parent": parent,
            "payload": {
                "data": payload,
                "data_crc32c": int(checksum.hexdigest(), 16),
            },
        }
    )
    return version.name
```

`payload.data` must be bytes. The CRC32C value is optional, but Google samples recommend it when integrity matters.

### Access A Secret Version

Use a fixed version number when you want stable, reviewable configuration.

```python
from google.cloud import secretmanager
import google_crc32c

def access_secret_value(
    project_id: str,
    secret_id: str,
    version_id: str = "latest",
) -> str:
    client = secretmanager.SecretManagerServiceClient()
    name = client.secret_version_path(project_id, secret_id, version_id)

    response = client.access_secret_version(request={"name": name})

    checksum = google_crc32c.Checksum()
    checksum.update(response.payload.data)
    if response.payload.data_crc32c != int(checksum.hexdigest(), 16):
        raise ValueError("Secret payload checksum verification failed")

    return response.payload.data.decode("utf-8")
```

Use `get_secret` or `get_secret_version` when you only need metadata and not the secret payload itself.

### List Secrets

List methods return pagers, so normal iteration handles pagination:

```python
from google.cloud import secretmanager

def list_secret_names(project_id: str) -> list[str]:
    client = secretmanager.SecretManagerServiceClient()
    parent = f"projects/{project_id}"
    return [secret.name for secret in client.list_secrets(request={"parent": parent})]
```

### Disable, Enable, Or Destroy A Secret Version

Disable a version to block access without deleting it:

```python
from google.cloud import secretmanager

def disable_secret_version(project_id: str, secret_id: str, version_id: str) -> str:
    client = secretmanager.SecretManagerServiceClient()
    name = client.secret_version_path(project_id, secret_id, version_id)
    version = client.disable_secret_version(request={"name": name})
    return version.name
```

Use `enable_secret_version` to restore access later. Use `destroy_secret_version` only when you intend permanent destruction.

## Regional Secrets And Endpoints

Secret Manager supports regional resources as well as the usual project-scoped global resource names.

Regional secrets use location-qualified resource names:

- Global: `projects/{project}/secrets/{secret}`
- Regional: `projects/{project}/locations/{location}/secrets/{secret}`

For regional secrets, point the client at the regional endpoint:

```python
from google.cloud import secretmanager_v1

location_id = "us-east1"

client = secretmanager_v1.SecretManagerServiceClient(
    client_options={
        "api_endpoint": f"secretmanager.{location_id}.rep.googleapis.com",
    }
)
```

Keep the resource-name format and the endpoint in sync. Mixing a regional endpoint with global resource names is a common source of `NotFound` and routing errors.

## Common Pitfalls

- A valid credential is not enough if the principal lacks Secret Manager IAM permissions on the project or secret.
- `latest` points to the newest created version, not the newest enabled version you intended to roll out.
- The import path is `from google.cloud import secretmanager`, while the generated reference often shows fully qualified modules under `secretmanager_v1`.
- Secret payloads are bytes. Encode before upload and decode after access.
- `access_secret_version` returns sensitive material. Prefer metadata-only methods if you do not need the plaintext.
- Secret rotation is version creation. Do not design code around in-place mutation of an existing version.
- Regional secrets require both regional resource names and the correct regional endpoint.

## Version-Sensitive Notes For `2.26.0`

- The official PyPI package page and Google Cloud Python reference both showed `2.26.0` on `2026-03-12`.
- PyPI declares `Requires: Python >=3.7`, so older runtimes are out of scope for this package version.
- The active repository is the `googleapis/google-cloud-python` monorepo package directory, not the archived single-package repository some older search results still reference.
- Google Cloud docs currently use the `docs.cloud.google.com` host as the canonical docs surface. Older `cloud.google.com/...` reference URLs commonly redirect there.

## Official Sources

- Google Cloud Secret Manager Python reference: https://docs.cloud.google.com/python/docs/reference/secretmanager/latest
- `SecretManagerServiceClient` reference: https://docs.cloud.google.com/python/docs/reference/secretmanager/latest/google.cloud.secretmanager_v1.services.secret_manager_service.SecretManagerServiceClient
- Secret Manager product docs: https://docs.cloud.google.com/secret-manager/docs
- Secret Manager access a secret version sample: https://docs.cloud.google.com/secret-manager/docs/samples/secretmanager-access-secret-version
- Secret Manager regional secret sample: https://docs.cloud.google.com/secret-manager/docs/samples/secretmanager-create-regional-secret
- Authentication for client libraries: https://docs.cloud.google.com/docs/authentication/client-libraries
- `google-cloud-secret-manager` PyPI page: https://pypi.org/project/google-cloud-secret-manager/
- Monorepo package directory: https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-secret-manager
- Changelog: https://github.com/googleapis/google-cloud-python/blob/main/packages/google-cloud-secret-manager/CHANGELOG.md

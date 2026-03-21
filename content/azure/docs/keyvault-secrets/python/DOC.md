---
name: keyvault-secrets
description: "Azure Key Vault Secrets client library for Python with SecretClient, DefaultAzureCredential, secret versioning, and soft-delete behavior"
metadata:
  languages: "python"
  versions: "4.10.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,key-vault,secrets,azure-keyvault-secrets,azure-identity,rbac"
---

# Azure Key Vault Secrets Python Client Library

## Golden Rule

Use `azure-keyvault-secrets` together with `azure-identity`, authenticate with `DefaultAzureCredential`, and pass a full vault URI such as `https://my-vault.vault.azure.net` to `SecretClient`. For `4.10.0`, trust PyPI and the package overview for version requirements: this release requires Python `>=3.9`.

## Install

Install the Key Vault client and the identity package together:

```bash
python -m pip install "azure-keyvault-secrets==4.10.0" "azure-identity"
```

Common alternatives:

```bash
uv add "azure-keyvault-secrets==4.10.0" "azure-identity"
poetry add "azure-keyvault-secrets==4.10.0" "azure-identity"
```

`azure-keyvault-secrets` does not ship `DefaultAzureCredential`; that comes from `azure-identity`.

## Authentication And Setup

`SecretClient` needs:

- `vault_url`: the full vault URI, not just a vault name
- `credential`: a `TokenCredential`, usually `DefaultAzureCredential()`

For local development, the Azure quickstart uses Azure CLI login:

```bash
az login
export KEY_VAULT_NAME="my-vault"
export VAULT_URL="https://${KEY_VAULT_NAME}.vault.azure.net"
```

For non-interactive environments, `DefaultAzureCredential` can use service principal environment variables:

```bash
export AZURE_TENANT_ID="..."
export AZURE_CLIENT_ID="..."
export AZURE_CLIENT_SECRET="..."
export VAULT_URL="https://my-vault.vault.azure.net"
```

Minimal client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.keyvault.secrets import SecretClient

credential = DefaultAzureCredential()
client = SecretClient(
    vault_url=os.environ["VAULT_URL"],
    credential=credential,
)
```

Authorization notes:

- The caller needs data-plane permissions for secret operations such as `get`, `set`, `list`, `delete`, `recover`, or `purge`.
- The Microsoft Learn quickstart creates the vault with RBAC enabled and assigns `Key Vault Secrets Officer` for management scenarios.
- Managed identity is the preferred production setup when the app runs on Azure.

## Core Usage

### Read the latest secret value

```python
import os

from azure.identity import DefaultAzureCredential
from azure.keyvault.secrets import SecretClient

client = SecretClient(
    vault_url=os.environ["VAULT_URL"],
    credential=DefaultAzureCredential(),
)

secret = client.get_secret("db-password")
print(secret.value)
```

`get_secret(name)` returns the latest version unless you pass `version=...`.

### Create or rotate a secret

`set_secret()` creates the secret if it does not exist. If it already exists, the call creates a new version.

```python
from datetime import datetime, timedelta, timezone

expires_on = datetime.now(timezone.utc) + timedelta(days=30)

created = client.set_secret(
    "db-password",
    "s3cr3t-value",
    content_type="text/plain",
    tags={"env": "prod", "owner": "payments"},
    expires_on=expires_on,
)

print(created.name)
print(created.properties.version)
```

### Get a specific version

```python
latest = client.get_secret("db-password")
version = latest.properties.version

same_version = client.get_secret("db-password", version=version)
print(same_version.id)
```

### Update metadata only

`update_secret_properties()` changes metadata, not the secret value.

```python
updated = client.update_secret_properties(
    "db-password",
    enabled=True,
    tags={"env": "prod", "rotated-by": "deploy-job"},
    content_type="text/plain",
)

print(updated.updated_on)
print(updated.tags)
```

If you need a new value, call `set_secret()` again.

### List secrets and versions

List operations return properties, not secret values:

```python
for props in client.list_properties_of_secrets():
    print(props.name, props.version, props.updated_on)
```

```python
for props in client.list_properties_of_secret_versions("db-password"):
    print(props.version, props.enabled, props.expires_on)
```

If you need the value, call `get_secret(...)` after listing.

### Delete, recover, and purge

Delete is a long-running operation and returns a poller:

```python
deleted = client.begin_delete_secret("db-password").result()
print(deleted.name)
print(deleted.recovery_id)
```

Recover a soft-deleted secret:

```python
recovered = client.begin_recover_deleted_secret("db-password").result()
print(recovered.name)
```

Purge permanently:

```python
client.purge_deleted_secret("db-password")
```

### Backup and restore

```python
backup_bytes = client.backup_secret("db-password")
restored = client.restore_secret_backup(backup_bytes)
print(restored.id)
```

The API reference describes the backup payload as a protected bytes format intended for Azure Key Vault.

## Configuration Notes

Important `SecretClient` options:

- `api_version`: selects the Key Vault service API version. The API reference says it defaults to the most recent supported version.
- `verify_challenge_resource`: defaults to `True` and validates the authentication challenge resource. Leave it enabled unless you have a specific trusted environment that requires different behavior.

Use explicit `api_version` pinning only when you need to preserve older service behavior.

## Async Usage

Use the async client only if the rest of the application is already asyncio-based:

```python
import asyncio
import os

from azure.identity.aio import DefaultAzureCredential
from azure.keyvault.secrets.aio import SecretClient

async def main() -> None:
    credential = DefaultAzureCredential()
    client = SecretClient(
        vault_url=os.environ["VAULT_URL"],
        credential=credential,
    )

    try:
        secret = await client.get_secret("db-password")
        print(secret.value)
    finally:
        await client.close()
        await credential.close()

asyncio.run(main())
```

Close both the async client and async credential to avoid leaving transports open.

## Common Pitfalls

- Do not install only `azure-keyvault-secrets`; most real code also needs `azure-identity`.
- Do not pass only the vault name. `SecretClient` expects a full `vault_url`.
- `401` and `403` errors are usually auth or authorization failures, not missing secret names.
- `list_properties_of_secrets()` and `list_properties_of_secret_versions()` do not include secret values.
- `update_secret_properties()` cannot change the secret value.
- `begin_delete_secret()` returns a poller; wait for completion if later steps depend on the delete having finished.
- With soft-delete enabled, a deleted-but-recoverable secret name can block immediate recreation until you recover or purge it.
- The Azure quickstart still mentions Python `3.7+`, but PyPI, the package overview, and the changelog for `4.10.0` require Python `>=3.9`.

## Version-Sensitive Notes

- `4.10.0` adds support for Key Vault service API version `7.6` and makes `7.6` the default.
- `4.10.0` drops Python `3.8`; this package line now requires Python `>=3.9`.
- `4.6.0` changed challenge-resource verification behavior, which is why `verify_challenge_resource` can matter for unusual endpoints.
- If you are maintaining older `1.x` code, re-check credential setup and method behavior before porting examples forward.

## Official Sources

- Microsoft Learn package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/keyvault-secrets-readme?view=azure-python`
- SecretClient API reference: `https://learn.microsoft.com/en-us/python/api/azure-keyvault-secrets/azure.keyvault.secrets.secretclient?view=azure-python`
- Microsoft Learn quickstart: `https://learn.microsoft.com/en-us/azure/key-vault/secrets/quick-create-python`
- Azure Key Vault soft-delete overview: `https://learn.microsoft.com/en-us/azure/key-vault/general/soft-delete-overview`
- Azure SDK changelog: `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/main/sdk/keyvault/azure-keyvault-secrets/CHANGELOG.md`
- PyPI package page: `https://pypi.org/project/azure-keyvault-secrets/`

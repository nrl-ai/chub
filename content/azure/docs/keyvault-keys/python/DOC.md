---
name: keyvault-keys
description: "Azure Key Vault Keys client library for creating, managing, and using cryptographic keys from Python"
metadata:
  languages: "python"
  versions: "4.11.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,key-vault,keys,kms,cryptography,security"
---

# Azure Key Vault Keys Python Client Library

## Golden Rule

Use `azure-keyvault-keys` for key lifecycle operations, pair it with `azure-identity` for credentials, and use `CryptographyClient` for encrypt/decrypt, sign/verify, wrap/unwrap, and related crypto operations. Treat delete and recover flows as long-running operations and wait for the returned pollers before assuming the key state changed.

## Install

Pin the package version your project expects and install `azure-identity` alongside it:

```bash
python -m pip install "azure-keyvault-keys==4.11.0" "azure-identity"
```

Common alternatives:

```bash
uv add "azure-keyvault-keys==4.11.0" azure-identity
poetry add "azure-keyvault-keys==4.11.0" azure-identity
```

If you need async clients, install an async transport too:

```bash
python -m pip install "azure-keyvault-keys==4.11.0" "azure-identity" aiohttp
```

## Authentication And Setup

The docs and quickstart use `DefaultAzureCredential`. That is the right default for local development, CI, and Azure-hosted workloads because it can use environment credentials, managed identity, Azure CLI login, and other supported credential sources.

For local development:

```bash
az login
export KEY_VAULT_URL="https://<your-vault-name>.vault.azure.net"
```

Basic setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.keyvault.keys import KeyClient

vault_url = os.environ["KEY_VAULT_URL"]
credential = DefaultAzureCredential()
client = KeyClient(vault_url=vault_url, credential=credential)
```

Authorization notes:

- `azure-keyvault-keys` does not authenticate by itself; you need a credential from `azure-identity` or another compatible Azure credential package.
- If requests fail with `403 Forbidden`, verify the principal has Key Vault data-plane permissions for the operations you are performing. The quickstart uses Azure RBAC guidance for key management roles.
- Use the exact vault URL for the target cloud. Public Azure vaults use `https://<name>.vault.azure.net`.

## Core Usage

### Create And Fetch Keys

Create an RSA key:

```python
from azure.identity import DefaultAzureCredential
from azure.keyvault.keys import KeyClient

credential = DefaultAzureCredential()
client = KeyClient(vault_url="https://my-vault.vault.azure.net", credential=credential)

created_key = client.create_rsa_key("app-signing-key", size=2048)
current_key = client.get_key("app-signing-key")

print(created_key.name)
print(current_key.properties.version)
```

Create an elliptic curve key:

```python
ec_key = client.create_ec_key("webhook-signing-key")
```

### List And Inspect Key Metadata

Use the list APIs when you only need properties instead of full key material:

```python
for props in client.list_properties_of_keys():
    print(props.name, props.enabled)

for props in client.list_properties_of_key_versions("app-signing-key"):
    print(props.version, props.created_on)
```

Fetch a deleted key if you are working with soft-delete or recovery flows:

```python
deleted = client.get_deleted_key("app-signing-key")
print(deleted.recovery_id)
```

### Update Key Properties

```python
updated = client.update_key_properties(
    "app-signing-key",
    enabled=True,
    tags={"service": "api", "owner": "platform"},
)

print(updated.properties.updated_on)
```

### Delete, Recover, And Purge Keys

Deletion and recovery are long-running operations. Wait for the poller result:

```python
delete_poller = client.begin_delete_key("app-signing-key")
deleted_key = delete_poller.result()

recover_poller = client.begin_recover_deleted_key("app-signing-key")
recovered_key = recover_poller.result()

client.purge_deleted_key("app-signing-key")

print(deleted_key.name)
print(recovered_key.name)
```

### Use `CryptographyClient` For Crypto Operations

`KeyClient` manages keys. `CryptographyClient` performs cryptographic operations with a specific key:

```python
from azure.identity import DefaultAzureCredential
from azure.keyvault.keys import KeyClient
from azure.keyvault.keys.crypto import CryptographyClient, EncryptionAlgorithm

credential = DefaultAzureCredential()
key_client = KeyClient(vault_url="https://my-vault.vault.azure.net", credential=credential)
key = key_client.get_key("app-signing-key")

crypto_client = CryptographyClient(key=key, credential=credential)

plaintext = b"secret payload"
encrypt_result = crypto_client.encrypt(EncryptionAlgorithm.rsa_oaep, plaintext)
decrypt_result = crypto_client.decrypt(
    EncryptionAlgorithm.rsa_oaep,
    encrypt_result.ciphertext,
)

assert decrypt_result.plaintext == plaintext
```

The same client also exposes `sign`, `verify`, `wrap_key`, and `unwrap_key`.

### Async Clients

Use the `.aio` imports for async code and close credentials and clients cleanly:

```python
import os
import asyncio

from azure.identity.aio import DefaultAzureCredential
from azure.keyvault.keys.aio import KeyClient

async def main() -> None:
    vault_url = os.environ["KEY_VAULT_URL"]

    async with DefaultAzureCredential() as credential:
        async with KeyClient(vault_url=vault_url, credential=credential) as client:
            key = await client.get_key("app-signing-key")
            print(key.name, key.properties.version)

asyncio.run(main())
```

## Configuration Notes

- Prefer `DefaultAzureCredential` unless your environment requires a specific credential type such as `ClientSecretCredential`.
- Reuse `KeyClient` and `CryptographyClient` instances instead of creating them per request.
- Keep the vault URL in configuration, not inline literals spread across the codebase.
- If your workload runs in Azure, prefer managed identity over client secrets.
- The package overview documents `send_request` support on the client if you need a custom data-plane request against the same pipeline, but use the typed client methods first.

## Common Pitfalls

- Do not use the legacy `azure-keyvault` package for new code. This package is the current library for keys.
- Installing only `azure-keyvault-keys` is not enough for most real applications; you still need `azure-identity` for authentication.
- `KeyClient` and `CryptographyClient` have different jobs. Use `KeyClient` for create/get/update/delete/list and `CryptographyClient` for encrypt/decrypt, sign/verify, and wrap/unwrap.
- Async usage requires `.aio` imports and an async transport such as `aiohttp`.
- Delete and recover flows are poller-based. If you skip `.result()`, the key state may not be settled when later code runs.
- The quickstart currently says Python `3.7+`, but current PyPI metadata for `4.11.0` requires Python `>=3.9`. Trust the package metadata when choosing runtime support.
- Azure authorization failures are often RBAC or access-policy issues rather than SDK bugs. Check the vault's key permissions before debugging client code.

## Version-Sensitive Notes For `4.11.0`

- The version used here `4.11.0` matches the current PyPI release as of `2026-03-12`.
- PyPI metadata for `4.11.0` requires Python `>=3.9`, so older Python examples in Azure docs or blog posts may no longer be valid for current installs.
- The official Azure SDK changelog for `4.11.0` notes support for key release policy operations and importing keys from certificates. If you are maintaining code written against older `4.10.x` examples, re-check any custom key import or HSM release flows.

## Official Source URLs

- `https://learn.microsoft.com/en-us/python/api/azure-keyvault-keys/`
- `https://learn.microsoft.com/en-us/python/api/overview/azure/keyvault-keys-readme?view=azure-python`
- `https://learn.microsoft.com/en-us/azure/key-vault/keys/quick-create-python`
- `https://pypi.org/pypi/azure-keyvault-keys/json`
- `https://github.com/Azure/azure-sdk-for-python/blob/azure-keyvault-keys_4.11.0/sdk/keyvault/azure-keyvault-keys/CHANGELOG.md`

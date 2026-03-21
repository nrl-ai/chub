---
name: mgmt-keyvault
description: "Azure Key Vault management-plane SDK for Python for creating, updating, listing, deleting, and purging vault resources"
metadata:
  languages: "python"
  versions: "13.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,key-vault,arm,management,rbac,azure-identity"
---

# Azure Key Vault Management SDK for Python

## Golden Rule

Use `azure-mgmt-keyvault` only for Azure Resource Manager control-plane work such as creating or updating vault resources, listing them, managing deleted-vault lifecycle, and configuring management properties. For data-plane work inside an existing vault, use `azure-keyvault-secrets`, `azure-keyvault-keys`, or `azure-keyvault-certificates` instead. Pair this package with `azure-identity`, and prefer JSON/dict request bodies over direct model construction when adapting older samples because `13.0.0` introduced hybrid models.

## Install

Install the management client and an Azure credential package together:

```bash
python -m pip install "azure-mgmt-keyvault==13.0.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-mgmt-keyvault==13.0.0" azure-identity
poetry add "azure-mgmt-keyvault==13.0.0" azure-identity
```

If you will use async clients, install an async transport too:

```bash
python -m pip install "azure-mgmt-keyvault==13.0.0" azure-identity aiohttp
```

## Authentication And Setup

The package README and Azure authentication guidance both point to Microsoft Entra token auth. In practice:

- Local development: sign in with `az login`, then use `DefaultAzureCredential()` or `AzureCliCredential()`
- CI or other non-interactive environments: use a service principal
- Azure-hosted workloads: prefer managed identity

Required configuration:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
```

If you authenticate with a service principal directly:

```bash
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

Basic client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.keyvault import KeyVaultManagementClient

client = KeyVaultManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

Sovereign cloud note: `KeyVaultManagementClient` in current docs accepts `cloud_setting=...` if you need a non-public Azure ARM endpoint.

## Core Usage

The current client surface exposes these main operation groups:

- `vaults`
- `managed_hsms`
- `private_endpoint_connections`
- `private_link_resources`
- `operations`

`vaults` is the main entry point for ordinary Key Vault resource management.

### Create Or Update A Vault

`begin_create_or_update(...)` is a long-running ARM operation and returns a poller. Passing a plain dictionary keeps the request shape explicit and avoids most hybrid-model surprises.

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.keyvault import KeyVaultManagementClient

client = KeyVaultManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)

poller = client.vaults.begin_create_or_update(
    resource_group_name="example-rg",
    vault_name="example-kv-1234",
    parameters={
        "location": "eastus",
        "tags": {
            "env": "dev",
            "owner": "context-hub",
        },
        "properties": {
            "tenant_id": os.environ["AZURE_TENANT_ID"],
            "sku": {
                "family": "A",
                "name": "standard",
            },
            "access_policies": [],
            "enable_rbac_authorization": True,
            "enable_soft_delete": True,
            "soft_delete_retention_in_days": 90,
            "public_network_access": "Enabled",
        },
    },
)

vault = poller.result()
print(vault.id)
print(vault.properties.vault_uri)
```

Important details from the model docs:

- `tenant_id` is required
- `access_policies` are required unless `createMode` is recovery; for RBAC-enabled vaults, an empty list is the usual shape
- `enable_rbac_authorization=True` affects data-action authorization, but management actions are always RBAC-based
- `soft_delete_retention_in_days` must be between `7` and `90`

### List And Inspect Vaults

```python
for vault in client.vaults.list_by_resource_group("example-rg"):
    print(vault.name, vault.location, vault.properties.vault_uri)

current = client.vaults.get("example-rg", "example-kv-1234")
print(current.properties.enable_rbac_authorization)
print(current.properties.public_network_access)
```

Use `list_by_resource_group(...)` when you already know the resource group and `list_by_subscription()` when you need a subscription-wide inventory.

### Check Name Availability Before Create

```python
result = client.vaults.check_name_availability(
    {
        "name": "example-kv-1234",
        "type": "Microsoft.KeyVault/vaults",
    }
)

print(result.name_available)
print(result.reason)
```

`13.0.0` added the `type` property on `VaultCheckNameAvailabilityParameters`, so older examples that omit it may be incomplete.

### Update Vault Properties

Use `update(...)` for partial changes instead of recreating the whole resource:

```python
updated = client.vaults.update(
    resource_group_name="example-rg",
    vault_name="example-kv-1234",
    parameters={
        "tags": {
            "env": "prod",
            "owner": "platform",
        },
        "properties": {
            "public_network_access": "Disabled",
        },
    },
)

print(updated.tags)
print(updated.properties.public_network_access)
```

### Delete, List Deleted, And Purge

Delete and purge are separate steps when soft delete is enabled:

```python
client.vaults.delete("example-rg", "example-kv-1234")

for deleted in client.vaults.list_deleted():
    print(deleted.name, deleted.properties.scheduled_purge_date)

purge_poller = client.vaults.begin_purge_deleted(
    vault_name="example-kv-1234",
    location="eastus",
)
purge_poller.result()
```

`get_deleted(...)` and `begin_purge_deleted(...)` both need the vault's Azure region. Keep that location in your cleanup code instead of assuming you can derive it later.

## Configuration Notes

- `subscription_id` is required; the client does not infer it from the credential.
- The current client default API version is `2025-05-01`. The docs explicitly warn that overriding `api_version` may result in unsupported behavior.
- Reuse a long-lived client in scripts or services instead of constructing one per operation.
- For non-public Azure clouds, align your credential authority host and the client's `cloud_setting`.
- `send_request(...)` exists in `13.0.0` for custom raw requests through the same pipeline, but use the typed operation groups first.

## Async Usage

Use the async client only if the rest of the application is already asyncio-based:

```python
import asyncio
import os

from azure.identity.aio import DefaultAzureCredential
from azure.mgmt.keyvault.aio import KeyVaultManagementClient

async def main() -> None:
    async with DefaultAzureCredential() as credential:
        client = KeyVaultManagementClient(
            credential=credential,
            subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
        )
        try:
            async for vault in client.vaults.list_by_subscription():
                print(vault.name)
        finally:
            await client.close()

asyncio.run(main())
```

Close both the client and the async credential to avoid leaving transports open.

## Common Pitfalls

- Using `azure-mgmt-keyvault` for secret or key CRUD inside a vault. That is data-plane work and belongs to the `azure-keyvault-*` packages.
- Installing only `azure-mgmt-keyvault` and forgetting `azure-identity`.
- Forgetting `AZURE_SUBSCRIPTION_ID`; ARM management clients need it even when auth is otherwise correct.
- Treating `begin_create_or_update(...)` or `begin_purge_deleted(...)` as immediate. They return pollers.
- Mixing older access-policy examples with `13.0.0` model code. `Permissions.keys` became `Permissions.keys_property` in this release.
- Assuming `enable_rbac_authorization=True` removes all management RBAC requirements. It only changes data-action authorization inside the vault.
- Overriding `api_version` to match an old sample after `12.0.0`. Current packages target the latest API version only.

## Version-Sensitive Notes For `13.0.0`

- `13.0.0` adds `send_request(...)` on `KeyVaultManagementClient`.
- `13.0.0` introduces hybrid models. If old constructor-based examples behave strangely, prefer JSON/dict payloads or re-check the current model docs.
- `13.0.0` renamed the `Permissions` instance variable `keys` to `keys_property`.
- `12.0.0` was the major release that removed support for older API-version folders and kept only the latest service API. If your code depends on an older non-latest ARM API shape, pin an older package and use the matching docs.

## Official Sources Used

- `https://pypi.org/project/azure-mgmt-keyvault/`
- `https://learn.microsoft.com/en-us/python/api/azure-mgmt-keyvault/`
- `https://learn.microsoft.com/en-us/python/api/azure-mgmt-keyvault/azure.mgmt.keyvault.keyvaultmanagementclient?view=azure-python`
- `https://learn.microsoft.com/en-us/python/api/azure-mgmt-keyvault/azure.mgmt.keyvault.operations.vaultsoperations?view=azure-python`
- `https://learn.microsoft.com/en-us/python/api/azure-mgmt-keyvault/azure.mgmt.keyvault.models.vaultproperties?view=azure-python`
- `https://learn.microsoft.com/en-us/python/api/overview/azure/key-vault?view=azure-python`
- `https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication/overview`

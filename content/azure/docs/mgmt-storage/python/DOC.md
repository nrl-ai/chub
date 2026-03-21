---
name: mgmt-storage
description: "Azure Storage management-plane SDK for Python for storage accounts, ARM subresources, keys, SAS, and account configuration"
metadata:
  languages: "python"
  versions: "24.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,storage,arm,management,storage-accounts,provisioning"
---

# Azure Storage Management SDK for Python

## Golden Rule

Use `azure-mgmt-storage` for Azure Resource Manager control-plane work such as creating storage accounts, updating account settings, listing or regenerating account keys, and managing ARM-exposed storage subresources. Do not use it for blob uploads, downloads, queue messages, or file content access; use data-plane packages such as `azure-storage-blob`, `azure-storage-file-share`, or `azure-storage-queue` for those operations.

For `24.0.0`, pair it with `azure-identity`, authenticate with `DefaultAzureCredential` or `AzureCliCredential`, and assume the package targets the latest storage ARM API version unless you intentionally pin an older release.

## Install

Pin the package version your project expects and install `azure-identity` with it:

```bash
python -m pip install "azure-mgmt-storage==24.0.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-mgmt-storage==24.0.0" azure-identity
poetry add "azure-mgmt-storage==24.0.0" azure-identity
```

Environment variables you usually need:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
```

If you authenticate with a service principal directly, also set:

```bash
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

## Authentication And Setup

Preferred credential patterns:

1. `DefaultAzureCredential()` for reusable app code, CI, managed identity, or workload identity
2. `AzureCliCredential()` for local scripts after `az login`
3. A service principal via environment variables only when you explicitly need that flow

Basic setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.storage import StorageManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()

client = StorageManagementClient(
    credential=credential,
    subscription_id=subscription_id,
)
```

CLI-driven local scripts can use Azure CLI credentials directly:

```python
import os

from azure.identity import AzureCliCredential
from azure.mgmt.storage import StorageManagementClient

client = StorageManagementClient(
    credential=AzureCliCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

Management-plane permissions still matter after authentication succeeds. Expect ARM authorization failures if the principal lacks rights on the subscription, resource group, or storage account scope.

## Core Usage

### Check name availability before create

Storage account names are globally unique. Check first instead of learning the rule set from a failed create call.

```python
from azure.mgmt.storage.models import StorageAccountCheckNameAvailabilityParameters

result = client.storage_accounts.check_name_availability(
    StorageAccountCheckNameAvailabilityParameters(
        name="ctxhubstorageacct01",
    )
)

if not result.name_available:
    raise RuntimeError(result.message)
```

### Create a storage account

Provisioning is a long-running ARM operation, so use the `begin_*` method and wait for the poller:

```python
poller = client.storage_accounts.begin_create(
    resource_group_name="example-rg",
    account_name="ctxhubstorageacct01",
    parameters={
        "location": "westus2",
        "kind": "StorageV2",
        "sku": {"name": "Standard_LRS"},
        "tags": {
            "env": "dev",
            "owner": "context-hub",
        },
        "enable_https_traffic_only": True,
        "minimum_tls_version": "TLS1_2",
        "allow_blob_public_access": False,
    },
)

account = poller.result()
print(account.id)
```

### List and inspect accounts

```python
for account in client.storage_accounts.list_by_resource_group("example-rg"):
    print(account.name, account.location, account.kind)

account = client.storage_accounts.get_properties(
    resource_group_name="example-rg",
    account_name="ctxhubstorageacct01",
)

print(account.primary_endpoints.blob)
print(account.sku.name)
```

### Update settings and tags

Use `update()` for patch-style account changes:

```python
updated = client.storage_accounts.update(
    resource_group_name="example-rg",
    account_name="ctxhubstorageacct01",
    parameters={
        "tags": {
            "env": "prod",
            "owner": "platform",
        },
        "allow_shared_key_access": False,
        "public_network_access": "Enabled",
    },
)

print(updated.tags)
```

### List access keys only when you actually need shared-key flows

Management APIs can retrieve storage account keys, but application code should still prefer Entra ID plus the data-plane SDKs whenever possible.

```python
keys = client.storage_accounts.list_keys(
    resource_group_name="example-rg",
    account_name="ctxhubstorageacct01",
)

for key in keys.keys:
    print(key.key_name)
```

If you need to upload or download blobs after provisioning, switch to `azure-storage-blob` and pass the account URL plus an Entra ID credential instead of reusing management APIs for data access.

## Configuration Notes

- `subscription_id` is required; `StorageManagementClient` does not infer it from the credential.
- The current Learn docs show the client default `api_version` as `2025-06-01`. Overriding API versions globally is possible, but current Azure SDK release notes state the package targets the latest available API version, so older API assumptions are risky in `24.x`.
- Long-running storage management operations return `LROPoller` objects. Call `.result()` when you need completion before the next step.
- `DefaultAzureCredential` tries multiple credential sources. If auth succeeds or fails in a surprising way, inspect which credential source was actually used instead of assuming Azure CLI handled it.
- `24.0.0` adds a `cloud_setting` parameter on `StorageManagementClient`. For Azure Government or other non-public clouds, keep the client cloud setting aligned with the credential authority host instead of changing only one side.

## Common Pitfalls

- `azure-mgmt-storage` is control plane, not data plane. Provision accounts here; upload blobs with `azure-storage-blob`.
- Forgetting `AZURE_SUBSCRIPTION_ID` is common. The credential can resolve successfully while the client still cannot be constructed correctly.
- Storage account names are stricter than many examples imply: use lower-case letters and digits only, and check availability before create.
- `begin_create()`, `begin_delete()`, `begin_failover()`, and similar calls are asynchronous ARM operations. Do not assume they are finished until you wait on the poller.
- Newer generated Azure management models are keyword-only. If old examples pass positional constructor arguments, rewrite them using named fields.
- Retrieving account keys from ARM works, but it often leads agents into shared-key data access flows when Entra ID plus RBAC would be safer and simpler.

## Version-Sensitive Notes

### `24.0.0`

The official changelog for `24.0.0` calls out two high-value changes:

- `StorageManagementClient` gained a `cloud_setting` argument for cloud environment selection.
- `StorageTaskAssignmentsOperations.list()` now uses `top` instead of `maxpagesize`.

If copied code still calls `storage_task_assignments.list(maxpagesize=...)`, treat it as outdated for `24.0.0`.

### `23.x` and later

The `23.0.0` changelog states that the package now supports only the latest available API version. If your project depends on an older ARM storage API shape, pin an older `azure-mgmt-storage` release and verify examples against that release instead of forcing `24.x` examples into an older environment.

### Older examples

Older Azure blog posts and generated samples often assume:

- positional model constructors
- different operation-group pagination arguments
- older API-version folders in the package

Treat those as migration candidates, not copy-paste-ready examples for `24.0.0`.

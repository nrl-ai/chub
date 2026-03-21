---
name: mgmt-resource
description: "Azure Resource Manager SDK for Python for resource groups, generic ARM resources, providers, and tags"
metadata:
  languages: "python"
  versions: "25.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,arm,resource-manager,management,resource-groups,tags"
---

# Azure Resource Manager SDK for Python

## Golden Rule

Use `azure-mgmt-resource` for Azure Resource Manager management-plane work such as resource groups, generic ARM resources, provider metadata, and tags. Install `azure-identity` with it, authenticate with `DefaultAzureCredential` or `AzureCliCredential`, and do not assume older `azure.mgmt.resource` examples still match `25.x` because several operation groups were split into separate packages.

## Install

Pin the package version your project expects and install `azure-identity` alongside it:

```bash
python -m pip install "azure-mgmt-resource==25.0.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-mgmt-resource==25.0.0" azure-identity
poetry add "azure-mgmt-resource==25.0.0" azure-identity
```

If you need functionality that was split out of `azure-mgmt-resource`, install the matching companion package explicitly. Common examples in the `25.x` line include:

- `azure-mgmt-resource-deployments`
- `azure-mgmt-resource-subscriptions`
- `azure-mgmt-resource-features`
- `azure-mgmt-resource-locks`
- `azure-mgmt-resource-policy`

## Authentication And Setup

Use one of these credential patterns:

- `AzureCliCredential()` for local scripts after `az login`
- `DefaultAzureCredential()` for reusable code, CI, managed identity, or workload identity

Required environment:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
```

If you use a service principal directly, also set:

```bash
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

Basic client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.resource import ResourceManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()

client = ResourceManagementClient(
    credential=credential,
    subscription_id=subscription_id,
)
```

Local CLI-driven scripts can use Azure CLI credentials directly:

```python
import os

from azure.identity import AzureCliCredential
from azure.mgmt.resource import ResourceManagementClient

client = ResourceManagementClient(
    credential=AzureCliCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

For sovereign clouds, keep the authority host and ARM cloud setting aligned:

```python
import os

from azure.identity import AzureAuthorityHosts, DefaultAzureCredential
from azure.mgmt.resource import ResourceManagementClient
from msrestazure.azure_cloud import AZURE_US_GOV_CLOUD

credential = DefaultAzureCredential(authority=AzureAuthorityHosts.AZURE_GOVERNMENT)
client = ResourceManagementClient(
    credential=credential,
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
    cloud_setting=AZURE_US_GOV_CLOUD,
)
```

## Core Usage

The current `ResourceManagementClient` docs expose these main operation groups:

- `providers`
- `provider_resource_types`
- `resources`
- `resource_groups`
- `tags`

### Create Or Update A Resource Group

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.resource import ResourceManagementClient

client = ResourceManagementClient(
    DefaultAzureCredential(),
    os.environ["AZURE_SUBSCRIPTION_ID"],
)

resource_group = client.resource_groups.create_or_update(
    "example-rg",
    {
        "location": "westus2",
        "tags": {
            "env": "dev",
            "owner": "context-hub",
        },
    },
)

print(resource_group.id)
```

### List, Get, And Delete Resource Groups

```python
for group in client.resource_groups.list():
    print(group.name, group.location)

group = client.resource_groups.get("example-rg")
print(group.id)

delete_poller = client.resource_groups.begin_delete("example-rg")
delete_poller.result()
```

Deletion is a long-running operation. Call `.result()` when you need to wait for completion.

### Discover Provider Resource Types Before Generic ARM Calls

Generic ARM resource operations require a provider API version. Inspect the provider metadata first instead of guessing:

```python
resource_types = client.provider_resource_types.list("Microsoft.Storage")

for item in resource_types:
    print(item.resource_type)
    print(item.api_versions)
```

### Read A Generic Resource By ARM ID

```python
resource = client.resources.get_by_id(
    "/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/example-rg/providers/Microsoft.Storage/storageAccounts/exampleacct",
    api_version="2023-05-01",
)

print(resource.type)
print(resource.location)
```

Use `client.resources` when you have an ARM ID and the provider API version, not when a service-specific SDK already gives you a better typed client.

## Configuration Notes

- `subscription_id` is required. `ResourceManagementClient` does not infer it from the credential.
- The client constructor has an `api_version` parameter, but the Learn docs warn that overriding it may result in unsupported behavior. Prefer per-call provider API versions for `client.resources.*` operations instead of forcing a global override.
- `DefaultAzureCredential` tries multiple credential sources. If authentication succeeds or fails unexpectedly, inspect the active credential in your environment rather than assuming Azure CLI was used.
- Management operations often depend on RBAC at the subscription, resource-group, or resource scope. Authentication success does not imply authorization success.

## Version-Sensitive Notes

### `25.0.0`

PyPI release notes for `25.0.0` call out these major changes:

- The package now targets only the latest available API version.
- `ResourceManagementClient.deployments` moved to `azure-mgmt-resource-deployments` via `DeploymentsMgmtClient`.
- These modules were split into separate packages: `subscriptions`, `features`, `links`, `locks`, `policy`, `managedapplications`, `databoundaries`, `changes`, and `privatelinks`.

If your code needs those older clients or older API-version folders, pin an earlier package version and verify the corresponding docs before copying `25.x` examples.

### `24.0.0`

The `24.0.0` release already split these features into separate packages:

- `deploymentstacks` -> `azure-mgmt-resource-deploymentstacks`
- `deploymentscripts` -> `azure-mgmt-resource-deploymentscripts`
- `templatespecs` -> `azure-mgmt-resource-templatespecs`

### Be Careful With Older Official Samples

Some task-oriented Learn articles still show:

```python
resource_client.deployments.begin_create_or_update(...)
```

That is not the safe default for `azure-mgmt-resource 25.x`. For ARM template deployments in current projects, verify whether you should be using `azure-mgmt-resource-deployments` instead.

## Common Pitfalls

- Installing only `azure-mgmt-resource` and forgetting `azure-identity`
- Using this package for data-plane operations instead of ARM management
- Omitting `AZURE_SUBSCRIPTION_ID`
- Copying pre-`25.x` examples that expect `deployments`, `subscriptions`, or `locks` to still hang off `azure.mgmt.resource`
- Guessing an ARM provider `api_version` for `client.resources.*` calls instead of inspecting provider metadata first
- Forgetting that many delete and update flows return pollers and need `.result()`

## Official Sources Used

- https://pypi.org/project/azure-mgmt-resource/
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-resource/
- https://learn.microsoft.com/en-us/python/api/overview/azure/resources?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-resource/azure.mgmt.resource.resourcemanagementclient?view=azure-python
- https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/manage-resource-groups-python
- https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.defaultazurecredential?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.azureauthorityhosts?view=azure-python

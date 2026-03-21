---
name: mgmt-core
description: "azure-mgmt-core package guide for Python - Azure Resource Manager pipeline, polling, policies, and resource ID helpers"
metadata:
  languages: "python"
  versions: "1.6.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,arm,management,python,pypi"
---

# azure-mgmt-core Python Package Guide

## What This Package Is For

`azure-mgmt-core` is the shared management-plane foundation used by Azure Resource Manager SDKs for Python.

Use it directly when you need lower-level ARM building blocks:

- `ARMPipelineClient` or `AsyncARMPipelineClient` for raw ARM HTTP pipeline access
- ARM long-running-operation polling helpers
- ARM-specific policies such as challenge auth and auxiliary auth
- resource ID helpers such as `resource_id`, `parse_resource_id`, and `is_valid_resource_id`
- cloud endpoint helpers such as `get_arm_endpoints`

Do not treat `azure-mgmt-core` as a service client by itself. For actual Azure services, install a service package such as `azure-mgmt-resource`, `azure-mgmt-compute`, or `azure-mgmt-network`, which uses this package underneath.

## Installation

Install `azure-mgmt-core` directly if you are writing shared ARM helpers or custom management-plane client code:

```bash
pip install azure-mgmt-core==1.6.0
```

In most real projects you also need Azure credentials from `azure-identity`:

```bash
pip install azure-identity
```

If you are working with a concrete ARM service, install the service package too:

```bash
pip install azure-mgmt-resource
```

For new projects, avoid the deprecated umbrella `azure` package. Install the specific SDK packages you actually use.

## Authentication And Setup

`azure-mgmt-core` does not obtain credentials on its own. Pair it with `azure-identity`, most commonly `DefaultAzureCredential`.

Typical service-principal environment variables:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

Or authenticate locally with Azure CLI:

```bash
az login
az account set --subscription "$AZURE_SUBSCRIPTION_ID"
```

Basic setup:

```python
import os

from azure.identity import DefaultAzureCredential

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()
```

For Azure Resource Manager, the public-cloud endpoint is typically `https://management.azure.com` and the default token scope is `https://management.azure.com/.default`.

## Core Usage

### Build And Parse ARM Resource IDs

This is the safest and most common direct use of `azure-mgmt-core` in shared utility code.

```python
from azure.mgmt.core.tools import is_valid_resource_id, parse_resource_id, resource_id

vm_id = resource_id(
    subscription="00000000-0000-0000-0000-000000000000",
    resource_group="rg-demo",
    namespace="Microsoft.Compute",
    type="virtualMachines",
    name="vm-01",
)

if not is_valid_resource_id(vm_id):
    raise ValueError(f"Invalid ARM resource ID: {vm_id}")

parts = parse_resource_id(vm_id)

print(vm_id)
print(parts["resource_group"])
print(parts["namespace"])
print(parts["type"])
print(parts["name"])
```

Use these helpers instead of manually concatenating ARM IDs. They are less error-prone and easier to reuse across service clients.

### Override ARM Long-Running Operation Polling

Generated ARM clients already use management-plane polling, but you can override polling behavior explicitly when you need tighter control.

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.core.polling.arm_polling import ARMPolling
from azure.mgmt.resource import ResourceManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()

client = ResourceManagementClient(credential, subscription_id)

poller = client.resource_groups.begin_delete(
    "rg-demo",
    polling=ARMPolling(timeout=5),
)

poller.result()
```

Use this pattern when you need to tune polling intervals or enforce ARM polling behavior on long-running management operations.

### Work With Sovereign Clouds

`azure-mgmt-core` `1.6.0` includes `get_arm_endpoints`, which helps map a cloud setting to the correct ARM endpoint values.

Practical guidance:

- use `get_arm_endpoints` when your code must support Azure Government, Azure China, or other non-public clouds
- do not hardcode `management.azure.com` if the deployment target is cloud-dependent
- keep cloud selection in configuration instead of scattering endpoint constants throughout the codebase

If you are already using higher-level management clients, pass the correct cloud endpoint or authority configuration into the credential and client setup rather than rewriting ARM endpoint logic yourself.

### Use Raw ARM Pipeline Access Sparingly

`ARMPipelineClient` and `AsyncARMPipelineClient` exist for cases where generated clients are missing or you are building shared infrastructure around ARM requests.

Prefer generated service clients whenever possible because they already handle:

- API-version selection
- model serialization and deserialization
- pagination
- ARM long-running operations
- service-specific request shapes

Reach for raw pipeline clients only when you are intentionally building lower-level management tooling.

## Policies And Logging

The package exposes ARM-specific policies under `azure.mgmt.core.policies`, including challenge authentication, auxiliary authentication, automatic resource-provider registration, and ARM-specific HTTP logging.

Typical patterns:

- pair ARM auth policies with an `azure-identity` credential
- use `AuxiliaryAuthenticationPolicy` only for scenarios that require auxiliary tokens across tenants
- enable request logging with `logging_enable=True` only when debugging

Be careful with verbose HTTP logging in production. ARM requests can expose subscription IDs, resource names, request bodies, or sensitive headers if logging is too broad.

## Common Pitfalls

- `azure-mgmt-core` is not a full service SDK. Installing only this package will not give you `ResourceManagementClient`, `ComputeManagementClient`, and similar clients.
- This package is for Azure management plane, not data-plane SDKs such as Blob, Key Vault secrets, or Service Bus messaging.
- Do not hand-build ARM resource IDs with string concatenation. Use `resource_id` and validate with `is_valid_resource_id`.
- Do not hardcode public-cloud endpoints if the code may run in sovereign clouds.
- Do not assume every Azure package example on the web applies to management-plane code. Data-plane clients use different endpoints, auth scopes, and request shapes.
- If you use `DefaultAzureCredential`, local development may silently use Azure CLI, Visual Studio Code, or other available identities. Be explicit about the intended credential source when debugging auth failures.
- Automatic resource-provider registration can be convenient, but it may require permissions your principal does not have. If registration fails, check RBAC before blaming the client code.

## Version-Sensitive Notes For `1.6.0`

- The version used here for this session is `1.6.0`, and the current PyPI page also lists `1.6.0` as the latest release.
- `get_arm_endpoints` was added in the `1.5.0` line, so it is available in `1.6.0` but not in older installs such as `1.0.0`.
- `AuxiliaryAuthenticationPolicy` was added in the `1.4.0` line. Older environments may not have it.
- The `1.6.0` release updated `ARMChallengeAuthenticationPolicy` to adopt the newer `on_challenge` flow used by `azure-core`. If you subclass auth behavior or depend on challenge handling details, retest that code on upgrade.
- Current official package metadata lists Python `>=3.9`. If you are maintaining older Azure SDK environments, verify interpreter support before pinning to `1.6.0`.

## Official Sources

- Azure management core library package docs: https://learn.microsoft.com/en-us/python/api/azure-mgmt-core/?view=azure-python
- `ARMPipelineClient`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-core/azure.mgmt.core.armpipelineclient?view=azure-python
- `AsyncARMPipelineClient`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-core/azure.mgmt.core.asyncarmpipelineclient?view=azure-python
- `ARMPolling`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-core/azure.mgmt.core.polling.arm_polling.armpolling?view=azure-python
- `azure.mgmt.core.tools`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-core/azure.mgmt.core.tools?view=azure-python
- `ARMChallengeAuthenticationPolicy`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-core/azure.mgmt.core.policies.armchallengeauthenticationpolicy?view=azure-python
- Azure Identity overview: https://learn.microsoft.com/en-us/python/api/overview/azure/identity-readme?view=azure-python
- PyPI package page and release history: https://pypi.org/project/azure-mgmt-core/

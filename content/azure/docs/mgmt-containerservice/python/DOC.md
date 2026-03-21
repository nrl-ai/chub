---
name: mgmt-containerservice
description: "Azure Kubernetes Service management SDK for Python for managing AKS clusters, node pools, credentials, upgrades, and related ARM resources"
metadata:
  languages: "python"
  versions: "40.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,aks,kubernetes,containers,arm,management"
---

# Azure Kubernetes Service Management SDK for Python

## Golden Rule

Use `azure-mgmt-containerservice` for Azure Resource Manager control-plane operations against AKS, authenticate with a modern `TokenCredential` such as `DefaultAzureCredential`, and treat most mutating calls as long-running `begin_*` operations that need `.result()`. This package manages AKS resources; it does not replace `kubectl`, the Kubernetes Python client, or in-cluster APIs.

## Install

Pin the stable package version your project expects and install `azure-identity` with it:

```bash
python -m pip install "azure-mgmt-containerservice==40.2.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-mgmt-containerservice==40.2.0" azure-identity
poetry add "azure-mgmt-containerservice==40.2.0" azure-identity
```

If you need Kubernetes API access after fetching kubeconfig credentials, also install the Kubernetes client separately:

```bash
python -m pip install kubernetes
```

## Authentication And Setup

Use one of these credential patterns:

- `DefaultAzureCredential()` for reusable code, CI, managed identity, or workload identity
- `AzureCliCredential()` for local scripts after `az login`

Required environment:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
```

For service-principal auth through environment variables:

```bash
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

Basic client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.containerservice import ContainerServiceClient

client = ContainerServiceClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

CLI-driven local scripts can use Azure CLI credentials directly:

```python
import os

from azure.identity import AzureCliCredential
from azure.mgmt.containerservice import ContainerServiceClient

client = ContainerServiceClient(
    credential=AzureCliCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

For sovereign clouds or custom ARM endpoints, the current `40.x` line also supports `cloud_setting=` on the client constructor. Keep the identity authority host and ARM cloud setting aligned.

## Core Usage

Two package behaviors matter everywhere:

- Mutating methods usually start with `begin_` and return a long-running-operation poller. Call `.result()` before assuming the ARM change finished.
- `list*` methods typically return paged iterators. Iterate over them instead of assuming an eager list.

### List AKS clusters in a subscription or resource group

```python
for cluster in client.managed_clusters.list():
    print(cluster.name, cluster.location, cluster.kubernetes_version)

for cluster in client.managed_clusters.list_by_resource_group("example-rg"):
    print(cluster.name, cluster.provisioning_state)
```

### Get a cluster and inspect key properties

```python
cluster = client.managed_clusters.get(
    resource_group_name="example-rg",
    resource_name="example-aks",
)

print(cluster.name)
print(cluster.kubernetes_version)
print(cluster.dns_prefix)
print(cluster.node_resource_group)
print(cluster.provisioning_state)
```

Useful fields to inspect before writing follow-up code:

- `kubernetes_version`
- `agent_pool_profiles`
- `identity`
- `network_profile`
- `auto_upgrade_profile`
- `aad_profile` or `security_profile` when auth and policy behavior matter

### Fetch kubeconfig credentials

The official model returns base64-encoded kubeconfig payloads inside `CredentialResults.kubeconfigs`.

```python
import base64

creds = client.managed_clusters.list_cluster_user_credentials(
    resource_group_name="example-rg",
    resource_name="example-aks",
)

kubeconfig_b64 = creds.kubeconfigs[0].value
kubeconfig_yaml = base64.b64decode(kubeconfig_b64).decode("utf-8")

print(kubeconfig_yaml[:200])
```

Admin credentials use a separate call:

```python
admin_creds = client.managed_clusters.list_cluster_admin_credentials(
    resource_group_name="example-rg",
    resource_name="example-aks",
)
```

Prefer user credentials unless the task explicitly requires cluster-admin access.

### Check available Kubernetes upgrades

```python
profile = client.managed_clusters.get_upgrade_profile(
    resource_group_name="example-rg",
    resource_name="example-aks",
)

print("Current:", profile.control_plane_profile.kubernetes_version)

for upgrade in profile.control_plane_profile.upgrades or []:
    print("Available:", upgrade.kubernetes_version, upgrade.is_preview)
```

Use the upgrade profile before submitting any version change. Do not guess upgrade targets.

### Run a command on the cluster

The management API exposes `run_command` for AKS command execution.

```python
result = client.managed_clusters.run_command(
    resource_group_name="example-rg",
    resource_name="example-aks",
    request_payload={
        "command": "kubectl get nodes -o wide",
    },
)

print(result.logs)
```

Use this sparingly. For repeated operational workflows, writing kubeconfig and using `kubectl` directly is usually easier to reason about.

### Create or update a cluster

Cluster provisioning is possible through `begin_create_or_update(...)`, but the request body is much larger than agents usually expect because it combines identity, node pools, networking, Linux profile, and AKS feature flags. Treat this as infrastructure code, not a quick one-off API call.

```python
poller = client.managed_clusters.begin_create_or_update(
    resource_group_name="example-rg",
    resource_name="example-aks",
    parameters={
        "location": "eastus",
        "dns_prefix": "example-aks",
        "identity": {"type": "SystemAssigned"},
        "agent_pool_profiles": [
            {
                "name": "systempool",
                "count": 1,
                "vm_size": "Standard_DS2_v2",
                "mode": "System",
                "type": "VirtualMachineScaleSets",
                "os_type": "Linux",
            }
        ],
        "linux_profile": {
            "admin_username": "azureuser",
            "ssh": {
                "public_keys": [
                    {"key_data": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQ..."}
                ]
            },
        },
        "enable_rbac": True,
    },
)

cluster = poller.result()
print(cluster.id)
```

Before automating creation, verify the exact payload against current AKS requirements for your region, network plugin, and identity mode. For many teams, Bicep, ARM templates, or Terraform are safer for initial cluster provisioning, with this SDK used later for inspection and targeted updates.

## Configuration Notes

- Authentication and subscription targeting are separate. A credential can succeed while the client still fails if `subscription_id` is missing or wrong.
- This is an ARM management SDK. After you obtain kubeconfig, Kubernetes API calls belong to `kubectl` or the `kubernetes` Python package, not `azure-mgmt-containerservice`.
- The package constructor supports cloud and endpoint customization. Use those only when you are targeting a sovereign or custom cloud environment.
- Cluster mutations are region- and SKU-sensitive. Read the current upgrade profile, node pool details, and network profile before applying partial updates.

## Version-Sensitive Notes For `40.2.0`

- PyPI shows `40.2.0` as the current stable release on `2026-03-12`.
- PyPI also shows newer prereleases in the `41.0.0b*` line. Pin `40.2.0` if you want stable behavior instead of preview API surface.
- `40.2.0` adds an `OSSKU` enum member; it is a small update, not a major API-shape change.
- `40.1.0` added the `managed_namespaces` operation group. Do not assume older `40.0.0` examples include it.
- `40.0.0` added `cloud_setting` support on the client.
- `39.0.0` removed older service-version namespaces and now targets only the latest API version shipped by the package. If your code imported versioned subpackages from earlier releases, verify them before copying old examples.

## Common Pitfalls

- Installing `azure-mgmt-containerservice` without `azure-identity`
- Assuming `DefaultAzureCredential()` also chooses the Azure subscription for you
- Treating `begin_*` calls as synchronous and forgetting `.result()`
- Using admin credentials when user credentials are enough
- Forgetting that `CredentialResults.kubeconfigs` contains base64-encoded kubeconfig data
- Treating this management SDK as a replacement for Kubernetes API clients
- Copying older AKS examples that depended on package-versioned namespaces removed before `40.x`
- Creating or updating clusters with underspecified payloads because Python dicts are permissive while ARM validation is not
- Mixing stable `40.2.0` guidance with preview-only `41.0.0b*` or `view=azure-python-preview` examples

## Recommended Workflow For Coding Agents

1. Confirm you are doing AKS management-plane work, not Kubernetes in-cluster API work.
2. Pin `azure-mgmt-containerservice==40.2.0` and install `azure-identity`.
3. Authenticate with `DefaultAzureCredential()` or `AzureCliCredential()` and pass `AZURE_SUBSCRIPTION_ID` explicitly.
4. Start with `managed_clusters.get()` or `list()` to inspect the existing cluster shape before writing update code.
5. Use `get_upgrade_profile()` before any Kubernetes version change.
6. Decode kubeconfig credentials only when you actually need to hand off to `kubectl` or the Kubernetes Python client.
7. For large provisioning changes, validate the payload against the current Microsoft Learn reference or prefer IaC.

## Official Sources Used

- https://pypi.org/project/azure-mgmt-containerservice/
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-containerservice/?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-containerservice/azure.mgmt.containerservice.containerserviceclient?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-containerservice/azure.mgmt.containerservice.operations.managedclustersoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-containerservice/azure.mgmt.containerservice.operations.managednamespacesoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-containerservice/azure.mgmt.containerservice.models.credentialresults?view=azure-python
- https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication-overview
- https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.defaultazurecredential?view=azure-python

---
name: mgmt-compute
description: "Azure Compute management SDK for Python for managing virtual machines, VM scale sets, disks, snapshots, images, galleries, and compute SKUs through Azure Resource Manager"
metadata:
  languages: "python"
  versions: "37.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,compute,virtual-machines,vmss,disks,arm,python"
---

# Azure Compute Management SDK for Python

## Golden Rule

Use `azure-mgmt-compute` for Azure Compute management-plane operations, authenticate it with a modern `TokenCredential` such as `DefaultAzureCredential`, and remember that most write operations are long-running `begin_*` calls that need `.result()`. This package manages Azure resources through ARM; it does not replace guest-level SSH, WinRM, or OS provisioning tools.

## Install

Pin the package version your project expects and install a credential package alongside it:

```bash
python -m pip install "azure-mgmt-compute==37.2.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-mgmt-compute==37.2.0" azure-identity
poetry add "azure-mgmt-compute==37.2.0" azure-identity
```

## Authentication And Setup

The standard setup is `DefaultAzureCredential` plus `AZURE_SUBSCRIPTION_ID`.

Environment variables for service-principal auth:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

`DefaultAzureCredential` tries several identities in order, including environment-configured service principal credentials, workload identity, managed identity, Azure CLI login, Azure PowerShell login, and Azure Developer CLI login. That makes it the safest default for local development and Azure-hosted workloads.

Basic client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.compute import ComputeManagementClient

client = ComputeManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

For sovereign or custom ARM endpoints, pass `base_url=` or `cloud_setting=` when constructing `ComputeManagementClient`.

## Core Usage Patterns

Two SDK patterns matter everywhere in this package:

- `begin_*` methods return Azure long-running operation pollers. Call `.result()` before assuming the mutation finished.
- `list*` methods typically return paged iterators. Iterate over them instead of assuming a fully materialized list.

### List virtual machines in a resource group

```python
for vm in client.virtual_machines.list("my-resource-group"):
    print(vm.name, vm.location)
```

### Get VM details and runtime state

```python
vm = client.virtual_machines.get(
    resource_group_name="my-resource-group",
    vm_name="my-vm",
)

print(vm.hardware_profile.vm_size)
print(vm.provisioning_state)

instance_view = client.virtual_machines.instance_view(
    resource_group_name="my-resource-group",
    vm_name="my-vm",
)

for status in instance_view.statuses or []:
    print(status.code, status.display_status)
```

### Start, stop billing, or run a command on a VM

```python
client.virtual_machines.begin_start(
    resource_group_name="my-resource-group",
    vm_name="my-vm",
).result()

client.virtual_machines.begin_deallocate(
    resource_group_name="my-resource-group",
    vm_name="my-vm",
).result()
```

`begin_power_off()` stops a VM but does not release compute billing. `begin_deallocate()` is the call that releases compute resources.

### Create a managed disk

```python
from azure.mgmt.compute.models import CreationData, Disk

disk = client.disks.begin_create_or_update(
    resource_group_name="my-resource-group",
    disk_name="my-managed-disk",
    disk=Disk(
        location="eastus",
        disk_size_gb=128,
        sku={"name": "StandardSSD_LRS"},
        creation_data=CreationData(create_option="Empty"),
    ),
).result()

print(disk.id)
```

Important model rules from the official docs:

- `Disk.location` is required.
- `CreationData.create_option` is required.
- `Disk.disk_size_gb` is mandatory when `creation_data.create_option == "Empty"`.

### Grant and revoke temporary disk access

```python
from azure.mgmt.compute.models import GrantAccessData

access = client.disks.begin_grant_access(
    resource_group_name="my-resource-group",
    disk_name="my-managed-disk",
    grant_access_data=GrantAccessData(
        access="Read",
        duration_in_seconds=3600,
    ),
).result()

print(access.access_sas)

client.disks.begin_revoke_access(
    resource_group_name="my-resource-group",
    disk_name="my-managed-disk",
).result()
```

`GrantAccessData` requires both `access` and `duration_in_seconds`.

### Discover available compute SKUs

```python
for sku in client.resource_skus.list(filter="location eq 'eastus'"):
    if sku.resource_type == "virtualMachines":
        print(sku.name, sku.locations)
```

`resource_skus.list()` supports only a location filter.

## Creating Or Updating Virtual Machines

VM creation works through `virtual_machines.begin_create_or_update(...)`, but the request shape is larger than many agents expect because the compute resource depends on image, storage, OS, and network data.

```python
from azure.mgmt.compute.models import VirtualMachine

vm = VirtualMachine(
    location="eastus",
    hardware_profile={"vm_size": "Standard_D2s_v5"},
    storage_profile={
        "image_reference": {
            "publisher": "Canonical",
            "offer": "0001-com-ubuntu-server-jammy",
            "sku": "22_04-lts-gen2",
            "version": "latest",
        }
    },
    os_profile={
        "computer_name": "my-vm",
        "admin_username": "azureuser",
        "admin_password": "replace-me",
    },
    network_profile={
        "network_interfaces": [
            {
                "id": "/subscriptions/.../resourceGroups/.../providers/Microsoft.Network/networkInterfaces/my-nic",
                "primary": True,
            }
        ]
    },
)

created_vm = client.virtual_machines.begin_create_or_update(
    resource_group_name="my-resource-group",
    vm_name="my-vm",
    parameters=vm,
).result()

print(created_vm.id)
```

The `VirtualMachine` model requires `location`, and real deployments usually also depend on sibling Azure packages:

- `azure-mgmt-resource` for resource groups
- `azure-mgmt-network` for VNets, subnets, NICs, and public IPs
- valid marketplace image references, gallery image IDs, or existing managed disks

## Configuration Notes

- Authentication and subscription targeting are separate. `DefaultAzureCredential` can succeed while the SDK still fails if `subscription_id` is missing or wrong.
- Use one `ComputeManagementClient` per target subscription and cloud endpoint.
- Keep region names consistent across related resources; disk, NIC, VM, and image mismatches are a common cause of ARM validation failures.
- For scripts and simple automation, the sync client is usually enough. Add async only if the rest of the application already depends on an async stack.

## Version-Sensitive Notes For 37.2.0

- PyPI lists `37.2.0` as released on `2026-01-27`.
- PyPI states the package is tested with Python `3.9+`.
- `37.2.0` adds `GalleryScriptsOperations` and `GalleryScriptVersionsOperations`.
- `37.0.0` added `VirtualMachineScaleSetsOperations.begin_scale_out` and several gallery-related operation groups.
- `36.0.0` is the breaking release where the package stopped shipping older Azure Compute API-version subpackages and now targets only the latest API version. If your code depends on a specific non-latest API version, pin to an earlier package release.
- Do not copy Track 1 Azure auth samples that use `ServicePrincipalCredentials`; that is not the current pattern for `37.2.0`.

## Common Pitfalls

- Installing only `azure-mgmt-compute` and forgetting `azure-identity`.
- Omitting `AZURE_SUBSCRIPTION_ID` and assuming authentication alone is enough.
- Treating `begin_*` methods as synchronous and skipping `.result()`.
- Calling `begin_power_off()` when you actually need `begin_deallocate()` to stop compute billing.
- Trying to create a VM with compute models only; the NIC and network resources usually come from `azure-mgmt-network`.
- Assuming older service-version namespaces still exist in current releases after the `36.0.0` API-version cleanup.
- Sending underspecified `Disk` or `VirtualMachine` payloads because Python model constructors are permissive while the ARM service is not.

## Recommended Workflow For Coding Agents

1. Install `azure-mgmt-compute` and `azure-identity`, then confirm the project uses `DefaultAzureCredential` or another `TokenCredential`.
2. Start at `ComputeManagementClient`, then narrow to the exact operation group you need: `virtual_machines`, `disks`, `snapshots`, `virtual_machine_scale_sets`, or `resource_skus`.
3. Before writing a create or update payload, check the exact model constructor for required fields in `azure.mgmt.compute.models`.
4. If the task creates or updates VMs, inspect whether the project already manages network resources elsewhere; `azure-mgmt-compute` is usually only one part of the workflow.
5. If you need older API-version namespaces, do not improvise. Pin an earlier package version and fetch docs that match that older release.

## Official Sources Used

- Microsoft package API root: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/`
- Microsoft `ComputeManagementClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.computemanagementclient?view=azure-python`
- Microsoft `VirtualMachinesOperations` reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.operations.virtualmachinesoperations?view=azure-python`
- Microsoft `DisksOperations` reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.operations.disksoperations?view=azure-python`
- Microsoft `ResourceSkusOperations` reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.operations.resourceskusoperations?view=azure-python`
- Microsoft `VirtualMachine` model reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.models.virtualmachine?view=azure-python`
- Microsoft `Disk` model reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.models.disk?view=azure-python`
- Microsoft `CreationData` model reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.models.creationdata?view=azure-python`
- Microsoft `GrantAccessData` model reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-compute/azure.mgmt.compute.models.grantaccessdata?view=azure-python`
- Microsoft `DefaultAzureCredential` reference: `https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.defaultazurecredential?view=azure-python`
- PyPI project page and release metadata: `https://pypi.org/project/azure-mgmt-compute/`
- PyPI JSON metadata: `https://pypi.org/pypi/azure-mgmt-compute/json`

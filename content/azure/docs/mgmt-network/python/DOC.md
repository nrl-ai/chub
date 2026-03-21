---
name: mgmt-network
description: "Azure Network management SDK for Python for virtual networks, subnets, NSGs, public IPs, NICs, load balancers, and other ARM networking resources"
metadata:
  languages: "python"
  versions: "30.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,network,arm,virtual-network,subnet,nsg,public-ip,nic"
---

# Azure Network Management SDK for Python

## Golden Rule

Use `azure-mgmt-network` for Azure networking management-plane work through Azure Resource Manager, not for data-plane traffic or socket-level networking. Install `azure-identity` with it, authenticate with a `TokenCredential`, pass `AZURE_SUBSCRIPTION_ID` explicitly, and expect most create, update, and delete calls to be `begin_*` long-running operations that need `.result()`.

## Install

Pin the package version your project expects and install an Azure credential package alongside it:

```bash
python -m pip install "azure-mgmt-network==30.2.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-mgmt-network==30.2.0" azure-identity
poetry add "azure-mgmt-network==30.2.0" azure-identity
```

## Authentication And Setup

For most projects, use one of these patterns:

- `DefaultAzureCredential()` for reusable code that should work locally, in CI, and on Azure
- `AzureCliCredential()` for local scripts after `az login`

Required environment:

```bash
export AZURE_SUBSCRIPTION_ID="00000000-0000-0000-0000-000000000000"
```

If you authenticate with a service principal directly, also set:

```bash
export AZURE_TENANT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_ID="00000000-0000-0000-0000-000000000000"
export AZURE_CLIENT_SECRET="your-client-secret"
```

Basic client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.network import NetworkManagementClient

client = NetworkManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

CLI-driven local setup:

```python
import os

from azure.identity import AzureCliCredential
from azure.mgmt.network import NetworkManagementClient

client = NetworkManagementClient(
    credential=AzureCliCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

For sovereign clouds or custom ARM endpoints, align your credential authority and client cloud/base URL settings rather than assuming the public Azure cloud defaults.

## Core Usage Patterns

The main operation groups agents usually need are:

- `virtual_networks`
- `subnets`
- `network_security_groups`
- `public_ip_addresses`
- `network_interfaces`
- `load_balancers`
- `route_tables`

Two SDK behaviors matter everywhere in this package:

- Most write operations are `begin_*` calls that return a poller. Call `.result()` before using the created resource.
- Most `list*` methods return paged iterators. Iterate over them instead of assuming a materialized list.

### List virtual networks in a resource group

```python
for vnet in client.virtual_networks.list("my-resource-group"):
    print(vnet.name, vnet.location)
```

### Create a virtual network and an initial subnet

```python
from azure.mgmt.network.models import AddressSpace, Subnet, VirtualNetwork

vnet = client.virtual_networks.begin_create_or_update(
    resource_group_name="my-resource-group",
    virtual_network_name="my-vnet",
    parameters=VirtualNetwork(
        location="eastus",
        address_space=AddressSpace(address_prefixes=["10.0.0.0/16"]),
        subnets=[
            Subnet(
                name="default",
                address_prefix="10.0.1.0/24",
            )
        ],
    ),
).result()

print(vnet.id)
```

Practical notes:

- `VirtualNetwork.location` is required.
- Keep address spaces and subnet CIDRs non-overlapping.
- Use `client.subnets.begin_create_or_update(...)` later when you need to add or change a subnet independently of the VNet.

### Create or update a subnet after the VNet exists

```python
from azure.mgmt.network.models import Subnet

subnet = client.subnets.begin_create_or_update(
    resource_group_name="my-resource-group",
    virtual_network_name="my-vnet",
    subnet_name="app-subnet",
    subnet_parameters=Subnet(address_prefix="10.0.2.0/24"),
).result()

print(subnet.id)
```

### Create a network security group with an inbound rule

```python
from azure.mgmt.network.models import NetworkSecurityGroup, SecurityRule

nsg = client.network_security_groups.begin_create_or_update(
    resource_group_name="my-resource-group",
    network_security_group_name="web-nsg",
    parameters=NetworkSecurityGroup(
        location="eastus",
        security_rules=[
            SecurityRule(
                name="allow-https-in",
                protocol="Tcp",
                source_port_range="*",
                destination_port_range="443",
                source_address_prefix="*",
                destination_address_prefix="*",
                access="Allow",
                priority=200,
                direction="Inbound",
            )
        ],
    ),
).result()

print(nsg.id)
```

Important `SecurityRule` fields to set explicitly:

- `protocol`
- `source_port_range` or `source_port_ranges`
- `destination_port_range` or `destination_port_ranges`
- `source_address_prefix` or `source_address_prefixes`
- `destination_address_prefix` or `destination_address_prefixes`
- `access`
- `priority`
- `direction`

Azure enforces rule uniqueness by priority and direction within the NSG. Pick priorities carefully instead of copying the same value into every rule.

### Create a public IP address

```python
from azure.mgmt.network.models import PublicIPAddress

public_ip = client.public_ip_addresses.begin_create_or_update(
    resource_group_name="my-resource-group",
    public_ip_address_name="my-public-ip",
    parameters=PublicIPAddress(
        location="eastus",
        public_ip_allocation_method="Static",
        sku={"name": "Standard"},
    ),
).result()

print(public_ip.ip_address)
```

Be careful with SKU and allocation choices. Standard public IPs are the common modern default; older Basic examples are not the safest template to copy forward.

### Create a network interface attached to a subnet and public IP

```python
nic = client.network_interfaces.begin_create_or_update(
    resource_group_name="my-resource-group",
    network_interface_name="my-nic",
    parameters={
        "location": "eastus",
        "network_security_group": {"id": nsg.id},
        "ip_configurations": [
            {
                "name": "primary",
                "subnet": {"id": subnet.id},
                "public_ip_address": {"id": public_ip.id},
                "private_ip_allocation_method": "Dynamic",
                "primary": True,
            }
        ],
    },
).result()

print(nic.id)
```

This is the usual handoff point for VM creation in `azure-mgmt-compute`: create the NIC first with `azure-mgmt-network`, then pass the NIC resource ID into the compute VM payload.

## Async Usage

The package also exposes `azure.mgmt.network.aio.NetworkManagementClient`. Use it only when the rest of the application is already async.

```python
import os
import asyncio

from azure.identity.aio import DefaultAzureCredential
from azure.mgmt.network.aio import NetworkManagementClient

async def main():
    credential = DefaultAzureCredential()
    client = NetworkManagementClient(
        credential=credential,
        subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
    )

    try:
        async for vnet in client.virtual_networks.list("my-resource-group"):
            print(vnet.name)
    finally:
        await client.close()
        await credential.close()

asyncio.run(main())
```

For async write calls, `await` the `begin_*` call first and then `await poller.result()`.

## Configuration Notes

- `subscription_id` is required. The credential does not supply it automatically.
- This package manages ARM resources, so the caller needs Azure RBAC on the subscription, resource group, or resource scope in addition to successful authentication.
- Resource groups must already exist. Create them with `azure-mgmt-resource` or through deployment tooling before creating network resources here.
- Keep regions aligned across related resources. VNets, NICs, public IPs, load balancers, and VMs often fail validation when regions or dependent IDs do not match.
- Long-running operations can take noticeable time. Treat `.result()` as a real wait point and handle timeouts or eventual consistency in automation.

## Version-Sensitive Notes For 30.2.0

- PyPI lists `30.2.0` as the current release, published on `2026-02-11`.
- PyPI states the package targets Python `>=3.9`.
- The current Learn API reference documents both sync and async clients in the `azure.mgmt.network` and `azure.mgmt.network.aio` namespaces.
- Track 2 Azure authentication patterns apply here: use `azure-identity` credentials, not older Track 1 auth helpers such as `ServicePrincipalCredentials`.
- Older blog posts often show inline dictionaries copied from much older API versions. For `30.2.0`, verify model names and operation-group locations against the current Learn reference before copying them.

## Common Pitfalls

- Installing `azure-mgmt-network` without `azure-identity`
- Forgetting `AZURE_SUBSCRIPTION_ID`
- Confusing management-plane resource provisioning with data-plane traffic handling
- Treating `begin_*` methods as synchronous and skipping `.result()`
- Reusing overlapping CIDR blocks across VNets and subnets
- Creating NSG rules with duplicate priorities or the wrong direction
- Building a NIC before the referenced subnet, NSG, or public IP exists
- Mixing public-cloud defaults with sovereign-cloud credentials or endpoints

## Recommended Workflow For Coding Agents

1. Confirm the task is ARM networking management, not service-specific data-plane access.
2. Create or locate the resource group first.
3. Authenticate with `DefaultAzureCredential` or a narrower Azure Identity credential.
4. Build dependencies in order: VNet, subnet, NSG/public IP, NIC, then compute or load-balancer resources that depend on them.
5. Prefer explicit models or documented field names from Learn pages over ad hoc dictionaries copied from third-party snippets.

## Official Sources Used

- https://pypi.org/project/azure-mgmt-network/
- https://pypi.org/pypi/azure-mgmt-network/json
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.networkmanagementclient?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.aio.networkmanagementclient?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.operations.virtualnetworksoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.operations.subnetsoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.operations.networksecuritygroupsoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.operations.publicipaddressesoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.operations.networkinterfacesoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.models.virtualnetwork?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.models.addressspace?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.models.subnet?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.models.networksecuritygroup?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.models.securityrule?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.models.publicipaddress?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-network/azure.mgmt.network.models.networkinterface?view=azure-python
- https://learn.microsoft.com/en-us/python/api/overview/azure/identity-readme?view=azure-python

---
name: mgmt-subscription
description: "Azure Subscription Management client library for Python for tenant, subscription, alias, and subscription-policy operations"
metadata:
  languages: "python"
  versions: "3.1.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,arm,subscription,management,identity,tenant"
---

# azure-mgmt-subscription Python Package Guide

## What This Package Is For

`azure-mgmt-subscription` is the Azure Resource Manager management-plane SDK for working with:

- subscription aliases
- tenant and subscription discovery
- subscription lifecycle operations
- billing account policy lookup
- tenant subscription policies

Import path:

```python
from azure.mgmt.subscription import SubscriptionClient
```

In the current Microsoft Learn reference for this package line, `SubscriptionClient` exposes these operation groups: `alias`, `billing_account`, `operations`, `subscription`, `subscription_policy`, `subscriptions`, and `tenants`.

Use this package when you need account-level ARM metadata or subscription lifecycle APIs. Do not use it for service-specific resources like storage accounts, virtual machines, or Key Vault objects; those belong in service-specific ARM or data-plane SDKs.

## Install

```bash
pip install azure-mgmt-subscription==3.1.1 azure-identity
```

PyPI lists `3.1.1` as a Python 3 wheel and describes the package as tested with Python `3.7+`.

If you use `uv`:

```bash
uv add azure-mgmt-subscription azure-identity
```

## Authentication And Setup

Use `DefaultAzureCredential` unless you have a specific reason to force a narrower credential.

For local development:

```bash
az login
```

For service-principal based auth, set the standard Azure Identity environment variables:

```bash
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_ID="<client-id>"
export AZURE_CLIENT_SECRET="<client-secret>"
```

Basic setup:

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient

credential = DefaultAzureCredential()
client = SubscriptionClient(credential)
```

Important setup notes:

- `SubscriptionClient` does not require a `subscription_id` constructor argument.
- The public Azure ARM endpoint is the default `base_url` in the generated client docs.
- For sovereign clouds, use the matching Azure Identity authority and ARM endpoint for that cloud instead of mixing public-cloud defaults with non-public endpoints.

## Core Usage

### List Subscriptions Available To The Credential

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient

credential = DefaultAzureCredential()
client = SubscriptionClient(credential)

for sub in client.subscriptions.list():
    print(sub.subscription_id, sub.display_name, sub.state)
```

### Get A Specific Subscription And Its Available Locations

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient

subscription_id = "00000000-0000-0000-0000-000000000000"

client = SubscriptionClient(DefaultAzureCredential())
subscription = client.subscriptions.get(subscription_id)
print(subscription.display_name)

for location in client.subscriptions.list_locations(subscription_id):
    print(location.name)
```

### List Tenants

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient

client = SubscriptionClient(DefaultAzureCredential())

for tenant in client.tenants.list():
    print(tenant.tenant_id)
```

### Inspect Supported ARM Operations

This is useful when you are debugging permissions or confirming what the installed client exposes.

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient

client = SubscriptionClient(DefaultAzureCredential())

for operation in client.operations.list():
    print(operation.name)
```

### Work With Subscription Aliases

List aliases:

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient

client = SubscriptionClient(DefaultAzureCredential())
aliases = client.alias.list()

for alias in aliases.value:
    print(alias.name)
```

Get one alias:

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient

client = SubscriptionClient(DefaultAzureCredential())
alias = client.alias.get("finance-prod-alias")

print(alias.id)
print(alias.name)
```

Create or update an alias for an existing subscription:

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient
from azure.mgmt.subscription.models import PutAliasRequest, PutAliasRequestProperties

client = SubscriptionClient(DefaultAzureCredential())

poller = client.alias.begin_create(
    alias_name="finance-prod-alias",
    body=PutAliasRequest(
        properties=PutAliasRequestProperties(
            display_name="Finance Production",
            workload="Production",
            subscription_id="00000000-0000-0000-0000-000000000000",
        )
    ),
)

alias = poller.result()
print(alias.id)
```

If you are creating a new subscription rather than aliasing an existing one, `PutAliasRequestProperties` also supports `billing_scope`, `reseller_id`, and `additional_properties`.

### Manage Tenant Subscription Policy

This operation group exists in the 3.x package line and is useful when you need tenant-level subscription ingress or egress controls.

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.subscription import SubscriptionClient
from azure.mgmt.subscription.models import PutTenantPolicyRequestProperties

client = SubscriptionClient(DefaultAzureCredential())

policy = client.subscription_policy.add_update_policy_for_tenant(
    PutTenantPolicyRequestProperties(
        block_subscriptions_leaving_tenant=True,
        block_subscriptions_into_tenant=False,
        exempted_principals=["00000000-0000-0000-0000-000000000000"],
    )
)

print(policy.block_subscriptions_leaving_tenant)
```

## Async Client

The package line documented on PyPI includes official async support. Use the `aio` namespace if the rest of your codebase is async.

```python
from azure.identity.aio import DefaultAzureCredential
from azure.mgmt.subscription.aio import SubscriptionClient

async def main() -> None:
    credential = DefaultAzureCredential()
    client = SubscriptionClient(credential)
    try:
        async for tenant in client.tenants.list():
            print(tenant.tenant_id)
    finally:
        await client.close()
        await credential.close()
```

## Configuration Notes

- Operation classes such as `SubscriptionsOperations` and `AliasOperations` are attached to `SubscriptionClient`; do not instantiate them directly.
- `subscriptions.list()` and `tenants.list()` return paged iterables. `alias.list()` returns a `SubscriptionAliasListResult`, so iterate through `.value`.
- Long-running methods use the Azure SDK `begin_*` pattern and return an `LROPoller`.
- Alias names are request identifiers, not the same thing as the Azure subscription display name.
- If you keep clients around for a long time, close them when done.

## Common Pitfalls

- Do not use old `msrestazure` or `azure.common.credentials` authentication examples. Current package lines use `azure-identity`.
- Do not assume every operation takes a subscription ID at client construction time. In this package, subscription IDs are passed to individual methods that need them.
- Do not treat this package as a general ARM client for resource CRUD. It only covers subscription-level management surfaces.
- Do not assume `alias.begin_create()` only works for brand-new subscriptions. It can also create an alias for an existing `subscription_id`.
- Do not copy preview-package examples unless your dependency really is a preview. PyPI shows a `3.2.0b1` pre-release in addition to stable `3.1.1`.
- If your credential can authenticate but returns no subscriptions or tenants, check Azure RBAC and tenant visibility before debugging the SDK surface.

## Version-Sensitive Notes For `3.1.1`

- This doc is pinned to stable PyPI version `3.1.1`, published on September 6, 2022.
- PyPI release notes for `3.1.1` say it fixes an `api_version` error in an operation.
- `3.0.0` restored the `subscriptions` and `tenants` operation groups after `2.0.0` had removed them.
- `2.0.0` added `billing_account`, `subscription_policy`, `subscription.begin_accept_ownership`, and `additional_properties` on `PutAliasRequestProperties`.
- PyPI also lists a newer preview release, `3.2.0b1`. Unless your project intentionally tracks previews, keep examples and lockfiles on `3.1.1`.
- Microsoft Learn shows the current generated reference for this package line. Keep your dependency pinned when reproducing examples so you do not silently drift into preview-only behavior.

## Official Sources

- Microsoft Learn package reference: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/
- Microsoft Learn `SubscriptionClient`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.subscriptionclient?view=azure-python
- Microsoft Learn `SubscriptionsOperations`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.operations.subscriptionsoperations?view=azure-python
- Microsoft Learn `TenantsOperations`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.operations.tenantsoperations?view=azure-python
- Microsoft Learn `AliasOperations`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.operations.aliasoperations?view=azure-python
- Microsoft Learn `SubscriptionOperations`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.operations.subscriptionoperations?view=azure-python
- Microsoft Learn `SubscriptionPolicyOperations`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.operations.subscriptionpolicyoperations?view=azure-python
- Microsoft Learn `PutAliasRequestProperties`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.models.putaliasrequestproperties?view=azure-python
- Microsoft Learn `PutTenantPolicyRequestProperties`: https://learn.microsoft.com/en-us/python/api/azure-mgmt-subscription/azure.mgmt.subscription.models.puttenantpolicyrequestproperties?view=azure-python
- Microsoft Learn Azure authentication overview for Python: https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication-overview
- Microsoft Learn local development service-principal auth: https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication/local-development-service-principal
- PyPI package page and release history: https://pypi.org/project/azure-mgmt-subscription/

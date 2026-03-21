---
name: mgmt-sql
description: "Azure SQL management SDK for Python: create and manage logical servers, databases, firewall rules, and other ARM resources"
metadata:
  languages: "python"
  versions: "3.0.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-sql,sql,management,arm,database"
---

# Azure SQL Management SDK For Python

## Golden Rule

Use `azure-mgmt-sql` for Azure Resource Manager control-plane tasks such as creating logical servers, databases, firewall rules, failover groups, and other Azure SQL resources. Do not use it to open SQL connections or run queries against a database; for that, use a data-plane driver such as `pyodbc` or SQLAlchemy on top of a SQL Server driver.

The normal Azure pattern is:

1. Authenticate with `azure-identity`
2. Construct `SqlManagementClient`
3. Call an operation group such as `servers`, `databases`, or `firewall_rules`
4. Wait on `.result()` for any `begin_*` long-running operation

## Install

Pin the management package and install `azure-identity` alongside it for Microsoft Entra ID auth:

```bash
python -m pip install "azure-mgmt-sql==3.0.1" "azure-identity"
```

Common alternatives:

```bash
uv add "azure-mgmt-sql==3.0.1" azure-identity
poetry add "azure-mgmt-sql==3.0.1" azure-identity
```

If you authenticate with a service principal in CI, you usually also need these environment variables:

```bash
export AZURE_SUBSCRIPTION_ID="<subscription-id>"
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_ID="<client-id>"
export AZURE_CLIENT_SECRET="<client-secret>"
```

For local development, Microsoft recommends signing in with Azure developer tooling and letting `DefaultAzureCredential` reuse it:

```bash
az login
```

## Authentication And Client Setup

`SqlManagementClient` is an Azure management-plane client. You need:

- an Azure credential, usually `DefaultAzureCredential()`
- a subscription ID
- ARM permissions on the target subscription or resource group

Basic setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.sql import SqlManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()

client = SqlManagementClient(
    credential=credential,
    subscription_id=subscription_id,
)
```

Practical notes:

- Reuse one credential and one client per process instead of recreating them for every call.
- `DefaultAzureCredential` is the fastest path for code that must run both locally and in Azure.
- Local sign-in success is not enough by itself; the identity also needs permission to manage the target SQL resources.

### Sovereign clouds

If you are not using the public Azure cloud, align both the credential authority and the management endpoint. For example, sovereign-cloud setups often need a non-default `authority` for the credential and a non-default `base_url` for `SqlManagementClient`.

## Core Usage

### List logical servers in a resource group

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.sql import SqlManagementClient

credential = DefaultAzureCredential()
client = SqlManagementClient(credential, subscription_id="00000000-0000-0000-0000-000000000000")

for server in client.servers.list_by_resource_group("rg-app-prod"):
    print(server.name, server.location, server.fully_qualified_domain_name)
```

### Create or update a logical server

Server create and update calls are long-running operations, so use the `begin_` method and wait for the poller result.

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.sql import SqlManagementClient
from azure.mgmt.sql.models import Server

credential = DefaultAzureCredential()
client = SqlManagementClient(credential, subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"])

poller = client.servers.begin_create_or_update(
    resource_group_name="rg-app-prod",
    server_name="my-sql-server",
    parameters=Server(
        location="eastus",
        administrator_login="sqladminuser",
        administrator_login_password=os.environ["AZURE_SQL_ADMIN_PASSWORD"],
        version="12.0",
        minimal_tls_version="1.2",
        public_network_access="Enabled",
    ),
)

server = poller.result()
print(server.id)
print(server.fully_qualified_domain_name)
```

### Create a database

The database payload typically includes `location` and a `Sku`. For basic provisioning flows, constructing the model classes explicitly is clearer than guessing the JSON shape.

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.sql import SqlManagementClient
from azure.mgmt.sql.models import Database, Sku

credential = DefaultAzureCredential()
client = SqlManagementClient(credential, subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"])

poller = client.databases.begin_create_or_update(
    resource_group_name="rg-app-prod",
    server_name="my-sql-server",
    database_name="appdb",
    parameters=Database(
        location="eastus",
        sku=Sku(name="Basic", tier="Basic"),
    ),
)

database = poller.result()
print(database.name, database.status)
```

### List databases on a server

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.sql import SqlManagementClient

credential = DefaultAzureCredential()
client = SqlManagementClient(credential, subscription_id="00000000-0000-0000-0000-000000000000")

for database in client.databases.list_by_server("rg-app-prod", "my-sql-server"):
    print(database.name, database.status)
```

### Create or update a firewall rule

Firewall rules are managed through `client.firewall_rules`. This operation is not exposed as `begin_*`, so it returns the resource directly.

```python
from azure.identity import DefaultAzureCredential
from azure.mgmt.sql import SqlManagementClient
from azure.mgmt.sql.models import FirewallRule

credential = DefaultAzureCredential()
client = SqlManagementClient(credential, subscription_id="00000000-0000-0000-0000-000000000000")

rule = client.firewall_rules.create_or_update(
    resource_group_name="rg-app-prod",
    server_name="my-sql-server",
    firewall_rule_name="office-ip",
    parameters=FirewallRule(
        start_ip_address="203.0.113.10",
        end_ip_address="203.0.113.10",
    ),
)

print(rule.name, rule.start_ip_address, rule.end_ip_address)
```

### Delete a database or server

Delete operations are usually long-running as well:

```python
client.databases.begin_delete(
    resource_group_name="rg-app-prod",
    server_name="my-sql-server",
    database_name="appdb",
).result()

client.servers.begin_delete(
    resource_group_name="rg-app-prod",
    server_name="my-sql-server",
).result()
```

## Configuration Notes

- `subscription_id` is required. Keep it explicit instead of assuming the default Azure CLI subscription is always the right one.
- `SqlManagementClient` targets Azure Resource Manager. For public Azure, the management endpoint is `https://management.azure.com`.
- Many create and update calls require a full resource payload, not just the field you want to mutate. Read the model page before sending partial objects.
- The package exposes many operation groups beyond the common ones above, including elastic pools, failover groups, managed instances, backup policies, and sync groups. Check the operation group on the client before assuming a resource is unsupported.

## Common Pitfalls

- `azure-mgmt-sql` is management-plane only. It provisions and configures Azure SQL resources but does not execute SQL statements.
- Many write operations use `begin_*`. If you forget `.result()`, your code may exit before the ARM operation finishes.
- Resource names are ARM resource names, not connection strings or DNS names. Keep `resource_group_name`, `server_name`, and `database_name` distinct.
- Azure authentication problems often come from missing RBAC, not broken credentials. A successful `az login` does not guarantee write access.
- The special firewall rule address `0.0.0.0` is Azure SQL's "allow Azure services" behavior, not a generic internet allowlist. Do not use it casually.
- Use the same region assumptions across related resources. Server and database provisioning often fail or behave unexpectedly when the payload does not match the intended location and SKU combination.
- This package is generated from Azure management APIs and still uses older Azure SDK model patterns. When in doubt, check the exact model type in Microsoft Learn before inventing field names.

## Version-Sensitive Notes

- PyPI currently lists `3.0.1` as the latest stable release for `azure-mgmt-sql`.
- The PyPI release page also shows a prerelease line (`4.0.0b24`). Do not adopt the beta line unless your project intentionally targets it.
- `3.0.1` is an older management package release. The Microsoft Learn reference is still live, but newer Azure SQL platform features may appear in REST docs or portal UX before they show up in examples for this Python SDK.
- PyPI metadata for `3.0.1` lists older Python classifiers. If your project runs on a newer Python version, validate install and smoke-test the specific management operations you need instead of assuming the package follows the newer baseline used by other Azure SDK libraries.

## Official Sources

- Microsoft Learn package index: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/`
- Microsoft Learn client reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.sqlmanagementclient?view=azure-python`
- Microsoft Learn servers operations: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.operations.serversoperations?view=azure-python`
- Microsoft Learn databases operations: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.operations.databasesoperations?view=azure-python`
- Microsoft Learn firewall rules operations: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.operations.firewallrulesoperations?view=azure-python`
- Microsoft Learn server model: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.models.server?view=azure-python`
- Microsoft Learn database model: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.models.database?view=azure-python`
- Microsoft Learn firewall rule model: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.models.firewallrule?view=azure-python`
- Microsoft Learn SKU model: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-sql/azure.mgmt.sql.models.sku?view=azure-python`
- Microsoft Learn Azure auth overview: `https://learn.microsoft.com/en-us/azure/developer/python/sdk/authentication/overview`
- Microsoft Learn Azure Identity README: `https://learn.microsoft.com/en-us/python/api/overview/azure/identity-readme?view=azure-python`
- PyPI package page: `https://pypi.org/project/azure-mgmt-sql/`

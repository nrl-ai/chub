---
name: mgmt-cosmosdb
description: "azure-mgmt-cosmosdb package guide for Python with ARM auth, Cosmos DB management patterns, and 9.9.0 version notes"
metadata:
  languages: "python"
  versions: "9.9.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,cosmosdb,arm,management,python"
---

# azure-mgmt-cosmosdb Python Package Guide

Use `azure-mgmt-cosmosdb` when Python code needs to provision or manage Azure Cosmos DB resources through Azure Resource Manager (ARM).

This is the management SDK. It is not the data-plane SDK for working with documents inside a container.

## Golden Rule

- Use `azure.mgmt.cosmosdb.CosmosDBManagementClient` for control-plane work such as listing accounts, reading keys, creating SQL databases or containers, and updating account settings.
- Use `azure-identity` with `DefaultAzureCredential` unless the project already has a stricter Azure credential policy.
- Use `azure-cosmos` for item CRUD, queries, transactional batch operations, and other data-plane tasks.

## Install

```bash
python -m pip install "azure-mgmt-cosmosdb==9.9.0" azure-identity
```

If you are not pinning yet:

```bash
python -m pip install azure-mgmt-cosmosdb azure-identity
```

## Authentication And Setup

For local development, the usual setup is Azure CLI login plus a subscription ID:

```bash
az login
export AZURE_SUBSCRIPTION_ID="<subscription-id>"
```

For CI or service-principal auth, configure the environment variables `DefaultAzureCredential` expects:

```bash
export AZURE_TENANT_ID="<tenant-id>"
export AZURE_CLIENT_ID="<client-id>"
export AZURE_CLIENT_SECRET="<client-secret>"
export AZURE_SUBSCRIPTION_ID="<subscription-id>"
```

Important setup rules:

- The credential must have Azure RBAC permission to manage Cosmos DB resources in the target subscription or resource group.
- `DefaultAzureCredential` tries multiple identities in order, including environment credentials, workload identity, managed identity, Azure CLI, Azure PowerShell, and Azure Developer CLI.
- Authentication success is not enough by itself. You must still pass the correct Azure subscription ID to `CosmosDBManagementClient`.

## Initialize The Client

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.cosmosdb import CosmosDBManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()

client = CosmosDBManagementClient(
    credential=credential,
    subscription_id=subscription_id,
)
```

The current client constructor also accepts `cloud_setting` for non-public Azure clouds. Leave `api_version` alone unless you have a specific, tested reason to override it. The official docs warn that overriding the default API version can produce unsupported behavior.

## Core Usage

### List Cosmos DB Accounts

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.cosmosdb import CosmosDBManagementClient

client = CosmosDBManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)

for account in client.database_accounts.list():
    print(account.name)
    print(account.location)
```

`database_accounts.get(...)` is the usual next step when you already know the resource group and account name and need the full ARM resource payload.

### Read Account Keys

```python
keys = client.database_accounts.list_keys(
    resource_group_name="my-resource-group",
    account_name="my-cosmos-account",
)

print(keys.primary_master_key)
print(keys.secondary_master_key)
```

Treat returned keys like secrets. Do not log them or write them to build output.

### Create Or Update A SQL Database

The SQL resource operations live under `client.sql_resources`. Create and update methods are long-running ARM operations and return pollers:

```python
from azure.mgmt.cosmosdb import models

db_params = models.SqlDatabaseCreateUpdateParameters(
    resource=models.SqlDatabaseResource(id="appdb"),
)

poller = client.sql_resources.begin_create_update_sql_database(
    resource_group_name="my-resource-group",
    account_name="my-cosmos-account",
    database_name="appdb",
    create_update_sql_database_parameters=db_params,
)

database = poller.result()
print(database.name)
```

### Create Or Update A SQL Container

```python
from azure.mgmt.cosmosdb import models

container_params = models.SqlContainerCreateUpdateParameters(
    resource=models.SqlContainerResource(
        id="items",
        partition_key=models.ContainerPartitionKey(
            paths=["/tenantId"],
            kind="Hash",
        ),
    ),
)

poller = client.sql_resources.begin_create_update_sql_container(
    resource_group_name="my-resource-group",
    account_name="my-cosmos-account",
    database_name="appdb",
    container_name="items",
    create_update_sql_container_parameters=container_params,
)

container = poller.result()
print(container.name)
```

### List SQL Databases And Containers

```python
for database in client.sql_resources.list_sql_databases(
    resource_group_name="my-resource-group",
    account_name="my-cosmos-account",
):
    print(database.name)

for container in client.sql_resources.list_sql_containers(
    resource_group_name="my-resource-group",
    account_name="my-cosmos-account",
    database_name="appdb",
):
    print(container.name)
```

### Other Useful Management Operations

The current SDK surface also includes:

- `database_accounts.begin_create_or_update(...)` for provisioning an account
- `sql_resources.begin_migrate_sql_database_to_autoscale(...)` and `begin_migrate_sql_database_to_manual_throughput(...)` for throughput mode changes
- SQL role definition and role assignment operations under `sql_resources` when you need Cosmos DB SQL API RBAC management

## Operation Groups To Know

The client exposes multiple operation groups. The common ones for day-to-day automation are:

- `database_accounts` for account lifecycle, keys, connection strings, failover, and account metadata
- `sql_resources` for SQL databases, containers, stored procedures, triggers, user-defined functions, throughput, and SQL role assignments
- `mongo_db_resources`, `cassandra_resources`, `gremlin_resources`, and `table_resources` for non-SQL Cosmos DB APIs

Pick the operation group that matches the API type of the target Cosmos DB account.

## Common Pitfalls

- `azure-mgmt-cosmosdb` does not query or mutate documents inside a container. Use `azure-cosmos` for that.
- The install name and import path differ. Install `azure-mgmt-cosmosdb`, then import `azure.mgmt.cosmosdb`.
- Most create, update, delete, and migration methods are long-running operations. Call `.result()` before assuming the resource is ready.
- `list_keys` and similar calls need Azure resource identifiers such as resource group and account name, not a Cosmos DB endpoint URL.
- If you hard-code the wrong operation group, you will miss the methods you need. SQL API management lives under `sql_resources`, not `database_accounts`.
- For local development, `DefaultAzureCredential` may silently use a cached Azure CLI or IDE login. If the wrong tenant or subscription is active, you will get confusing authorization failures.
- Avoid overriding `api_version` unless you are intentionally targeting a specific ARM API version and have tested it against the account features you use.

## Version-Sensitive Notes

- Frontmatter tracks the version used here `9.9.0`, which matched the latest stable PyPI release on 2026-03-12.
- PyPI release history also lists `10.0.0b5` as a prerelease. Do not install with `--pre` unless the project explicitly wants preview features or preview API changes.
- The `9.9.0` release notes say `CosmosDBManagementClient` added a `cloud_setting` constructor parameter and new `fleet`, `fleetspace`, and `fleetspace_account` operation groups.
- PyPI and the Azure SDK repo both state that current supported Python versions for this package line start at `3.9`.

## Official Sources Used

- Microsoft Learn package reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-cosmosdb/?view=azure-python`
- Microsoft Learn `CosmosDBManagementClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-cosmosdb/azure.mgmt.cosmosdb.cosmosdbmanagementclient?view=azure-python`
- Microsoft Learn `DatabaseAccountsOperations` reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-cosmosdb/azure.mgmt.cosmosdb.operations.databaseaccountsoperations?view=azure-python`
- Microsoft Learn `SqlResourcesOperations` reference: `https://learn.microsoft.com/en-us/python/api/azure-mgmt-cosmosdb/azure.mgmt.cosmosdb.operations.sqlresourcesoperations?view=azure-python`
- Microsoft Learn `DefaultAzureCredential` reference: `https://learn.microsoft.com/en-us/python/api/azure-identity/azure.identity.defaultazurecredential?view=azure-python`
- PyPI package page and release history: `https://pypi.org/project/azure-mgmt-cosmosdb/`
- Azure SDK for Python repository: `https://github.com/Azure/azure-sdk-for-python`

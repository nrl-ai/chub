---
name: data-tables
description: "Azure Data Tables Python client for Azure Table Storage and Azure Cosmos DB Table workloads"
metadata:
  languages: "python"
  versions: "12.7.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,data-tables,azure-storage,cosmosdb,tables,nosql,odata"
---

# Azure Data Tables Python Client

## Golden Rule

Use `azure-data-tables` for both Azure Table Storage and Azure Cosmos DB for Table. Start from `TableServiceClient` when you need account-level operations or table creation, then get a `TableClient` for entity reads and writes. Every entity must include `PartitionKey` and `RowKey`, and most correctness issues come from partition design, optimistic concurrency, or using the wrong auth setup for the target endpoint.

## Install

Pin the package version your project expects:

```bash
python -m pip install "azure-data-tables==12.7.0"
```

If you authenticate with Microsoft Entra ID, install `azure-identity` too:

```bash
python -m pip install "azure-data-tables==12.7.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-data-tables==12.7.0"
poetry add "azure-data-tables==12.7.0"
```

## Authentication And Client Setup

The SDK supports several credential patterns:

- Connection string
- Shared key via `AzureNamedKeyCredential`
- SAS via `AzureSasCredential`
- `TokenCredential` such as `DefaultAzureCredential`

Use connection strings for quick local setup. Prefer `DefaultAzureCredential` in deployed Azure environments.

### Service client from a connection string

Use this when you want to create or list tables and then get per-table clients:

```python
from azure.data.tables import TableServiceClient

conn_str = "DefaultEndpointsProtocol=https;AccountName=...;AccountKey=...;EndpointSuffix=core.windows.net"

service = TableServiceClient.from_connection_string(conn_str=conn_str)
table = service.create_table_if_not_exists(table_name="products")
```

### Service client with `DefaultAzureCredential`

For Azure Table Storage, this is the clean default:

```python
from azure.data.tables import TableServiceClient
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential()

service = TableServiceClient(
    endpoint="https://<storage-account>.table.core.windows.net",
    credential=credential,
)
```

For Microsoft Entra ID on storage endpoints, the caller typically needs `Storage Table Data Contributor` or `Storage Table Data Reader`.

### Table client directly from a connection string

Use this when the table already exists and you only need entity operations:

```python
from azure.data.tables import TableClient

table = TableClient.from_connection_string(
    conn_str="DefaultEndpointsProtocol=https;AccountName=...;AccountKey=...;EndpointSuffix=core.windows.net",
    table_name="products",
)
```

### Cosmos DB for Table quickstart pattern

Official Cosmos DB for Table quickstarts use the same package and `TableServiceClient` with `DefaultAzureCredential`:

```python
from azure.data.tables import TableServiceClient
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential()
service = TableServiceClient(
    endpoint="<azure-cosmos-db-table-account-endpoint>",
    credential=credential,
)
table = service.get_table_client("products")
```

## Core Usage

### Create or get a table

`get_table_client()` does not create the table. Use `create_table_if_not_exists()` when setup should be idempotent.

```python
from azure.data.tables import TableServiceClient

service = TableServiceClient.from_connection_string(conn_str=conn_str)
table = service.create_table_if_not_exists(table_name="products")

same_table = service.get_table_client("products")
```

### Insert or upsert an entity

Every entity must include `PartitionKey` and `RowKey`.

```python
from azure.data.tables import TableClient

table = TableClient.from_connection_string(conn_str=conn_str, table_name="products")

entity = {
    "PartitionKey": "inventory",
    "RowKey": "sku-1001",
    "name": "Widget",
    "price": 9.99,
    "in_stock": True,
}

table.upsert_entity(entity=entity)
```

`upsert_entity()` is the safest default when you want create-or-update behavior.

### Read one entity

```python
entity = table.get_entity(
    partition_key="inventory",
    row_key="sku-1001",
)

print(entity["name"], entity["price"])
```

### Query entities with OData filters

Prefer parameterized filters over string interpolation when possible:

```python
entities = table.query_entities(
    query_filter="PartitionKey eq @pk and price gt @minimum",
    parameters={"pk": "inventory", "minimum": 5},
    select=["RowKey", "name", "price"],
)

for item in entities:
    print(item["RowKey"], item["name"], item["price"])
```

If you build filters manually, escape single quotes in string values to keep them OData-compliant.

### Merge vs replace updates

`MERGE` only updates supplied properties. `REPLACE` overwrites the entity and drops properties you omit.

```python
from azure.data.tables import UpdateMode

entity = table.get_entity(partition_key="inventory", row_key="sku-1001")
entity["price"] = 8.99

table.update_entity(entity=entity, mode=UpdateMode.MERGE)
```

Use `UpdateMode.REPLACE` only when you intentionally want the submitted entity to become the full stored shape.

### Delete an entity

```python
table.delete_entity(
    partition_key="inventory",
    row_key="sku-1001",
)
```

### Batch multiple operations in one transaction

`submit_transaction()` is atomic, but only for entities in the same partition.

```python
operations = [
    ("create", {"PartitionKey": "inventory", "RowKey": "sku-1002", "name": "Pen", "price": 1.25}),
    ("upsert", {"PartitionKey": "inventory", "RowKey": "sku-1003", "name": "Pencil", "price": 0.75}),
    ("update", {"PartitionKey": "inventory", "RowKey": "sku-1001", "price": 7.99}, {"mode": "merge"}),
]

result = table.submit_transaction(operations)
print(result)
```

Design constraints from Azure Table storage still matter here:

- all entities in the transaction must share a partition key
- a transaction can include at most 100 entities
- total transaction payload must stay under 4 MiB

### Async client usage

Async clients live under `azure.data.tables.aio` and should usually be used with `async with`:

```python
import asyncio
from azure.data.tables.aio import TableClient

async def main() -> None:
    async with TableClient.from_connection_string(
        conn_str=conn_str,
        table_name="products",
    ) as table:
        entity = await table.get_entity(
            partition_key="inventory",
            row_key="sku-1001",
        )
        print(entity["name"])

asyncio.run(main())
```

## Configuration Notes

### Endpoints

- Azure Storage table endpoints commonly look like `https://<account>.table.core.windows.net`
- Sovereign cloud storage endpoints use different domains such as `table.core.usgovcloudapi.net`
- Cosmos DB for Table uses its own account endpoint; prefer the portal, SDK quickstart, or connection string rather than guessing the hostname

### `audience` for `TokenCredential`

When you use `TokenCredential`, the client defaults to the public cloud audience. Set `audience` explicitly for sovereign clouds or when targeting Cosmos-specific audiences.

Examples from Microsoft Learn include:

- `https://storage.azure.com`
- `https://storage.azure.us`
- `https://storage.azure.cn`
- `https://cosmos.azure.com`
- `https://cosmos.azure.us`
- `https://cosmos.azure.cn`

### Retries and transport options

The package uses `azure-core` pipeline options. Common client keyword args include retry settings such as `retry_total`, `retry_connect`, `retry_read`, and `retry_status`.

Example:

```python
from azure.data.tables import TableServiceClient

service = TableServiceClient.from_connection_string(
    conn_str=conn_str,
    retry_total=5,
    retry_status=5,
)
```

## Common Pitfalls

- `get_table_client("name")` only constructs a client. It does not create the table.
- Every entity must include both `PartitionKey` and `RowKey`; the pair must be unique.
- `query_entities()` uses OData filters, not SQL. Parameterize filters or escape string values correctly.
- `update_entity(mode=MERGE)` does not remove properties that are absent from your payload. `REPLACE` does.
- Optimistic concurrency matters. Use `etag` and `match_condition` when you need to prevent overwriting a newer version of an entity.
- Transaction batches only work within a single partition and are capped at 100 entities.
- Table design is driven by `PartitionKey` and `RowKey`; there are no general secondary indexes like in a relational database.
- `DefaultAzureCredential` behavior depends on the environment. Local failures are often missing Azure CLI login, missing environment variables, or missing managed identity role assignments.
- For Entra ID in sovereign clouds, set both the authority host on the credential and the matching `audience` on the tables client.

## Version-Sensitive Notes

- PyPI currently lists `12.7.0` as the latest stable release, published on May 6, 2025.
- The official package overview and API reference both document the `12.x` client family; the version used here `12.7.0` matches current stable PyPI metadata as of March 12, 2026.
- The package is the supported replacement for `azure-cosmosdb-tables`. Do not start new work on the deprecated package unless you are maintaining legacy code during a migration.

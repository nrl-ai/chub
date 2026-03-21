---
name: cosmos
description: "Azure Cosmos DB for NoSQL Python client with practical guidance for auth, partition keys, CRUD, queries, retries, and async usage"
metadata:
  languages: "python"
  versions: "4.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,cosmos,cosmos-db,nosql,python,database"
---

# Azure Cosmos DB Python Client

## Golden Rule

Use `azure-cosmos` for Azure Cosmos DB for NoSQL from Python, and model partition keys up front. Reuse a long-lived `CosmosClient`, prefer point reads when you know `id` and partition key, and only use queries when you actually need them. This SDK is not the client for Cosmos DB MongoDB, Cassandra, Gremlin, or Table workloads.

## Install

Pin the version your project expects:

```bash
python -m pip install "azure-cosmos==4.15.0"
```

If you want Microsoft Entra ID authentication, install `azure-identity` too:

```bash
python -m pip install "azure-cosmos==4.15.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-cosmos==4.15.0"
poetry add "azure-cosmos==4.15.0"
```

## Authentication And Setup

You need an Azure Cosmos DB for NoSQL account endpoint such as `https://<account>.documents.azure.com:443/`.

### Account key auth

```bash
export COSMOS_ENDPOINT="https://<account>.documents.azure.com:443/"
export COSMOS_KEY="<account-key>"
```

```python
import os
from azure.cosmos import CosmosClient

client = CosmosClient(
    url=os.environ["COSMOS_ENDPOINT"],
    credential=os.environ["COSMOS_KEY"],
)
```

### Microsoft Entra ID auth

```bash
export COSMOS_ENDPOINT="https://<account>.documents.azure.com:443/"
```

```python
import os
from azure.cosmos import CosmosClient
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential()
client = CosmosClient(
    url=os.environ["COSMOS_ENDPOINT"],
    credential=credential,
)
```

For Entra ID auth, the principal needs Cosmos DB data-plane permissions on the account. Azure management roles by themselves are not enough.

### Connection string auth

```bash
export COSMOS_CONNECTION_STRING="AccountEndpoint=...;AccountKey=...;"
```

```python
import os
from azure.cosmos import CosmosClient

client = CosmosClient.from_connection_string(
    conn_str=os.environ["COSMOS_CONNECTION_STRING"]
)
```

## Client Initialization

`CosmosClient` initialization is heavy. Reuse one client for the lifetime of your process or app component instead of creating one per request.

Example with a few useful knobs:

```python
import os
from azure.cosmos import CosmosClient

client = CosmosClient(
    url=os.environ["COSMOS_ENDPOINT"],
    credential=os.environ["COSMOS_KEY"],
    preferred_locations=["West US 2", "East US"],
    retry_total=9,
    retry_backoff_max=30,
    connection_timeout=5,
    no_response_on_write=True,
)
```

Useful options from the API reference:

- `preferred_locations`: prefer specific read regions
- `excluded_locations`: exclude specific regions
- `retry_total` and `retry_backoff_max`: tune transient retry behavior
- `connection_timeout`: fail faster on bad endpoints or broken network paths
- `no_response_on_write`: reduce payload size for write-heavy paths when you do not need the full written document back

## Core Usage

### Create or get a database and container

Use `create_*_if_not_exists` for setup scripts, local development, and tests. In hot production paths, prefer `get_database_client(...)` and `get_container_client(...)` when the resources already exist.

```python
from azure.cosmos import CosmosClient, PartitionKey

client = CosmosClient(url=endpoint, credential=credential)

database = client.create_database_if_not_exists(id="appdb")
container = database.create_container_if_not_exists(
    id="items",
    partition_key=PartitionKey(path="/tenantId"),
    offer_throughput=400,
)
```

### Create or upsert an item

```python
item = {
    "id": "item-1",
    "tenantId": "acme",
    "name": "Example",
    "price": 42,
}

saved = container.upsert_item(item)
print(saved["id"])
```

`upsert_item` creates or replaces by `id`. Use `create_item` when the operation must fail if the item already exists.

### Read one item

Point reads are the cheapest and fastest way to fetch a single document, but you must know both the `id` and the partition key value.

```python
doc = container.read_item(
    item="item-1",
    partition_key="acme",
)

print(doc["name"])
```

### Query items

Parameterize queries instead of string formatting, and scope them to one partition when you can:

```python
query = """
SELECT c.id, c.name, c.price
FROM c
WHERE c.tenantId = @tenant_id AND c.price >= @min_price
"""

results = container.query_items(
    query=query,
    parameters=[
        {"name": "@tenant_id", "value": "acme"},
        {"name": "@min_price", "value": 10},
    ],
    partition_key="acme",
)

for row in results:
    print(row["id"], row["price"])
```

If you omit the partition key and fan out across partitions, latency and RU cost go up.

### Patch or replace an item

Use `patch_item` for partial updates and `replace_item` when you want to send the full document:

```python
updated = container.patch_item(
    item="item-1",
    partition_key="acme",
    patch_operations=[
        {"op": "replace", "path": "/price", "value": 50},
        {"op": "add", "path": "/tags", "value": ["featured"]},
    ],
)
```

### Delete an item

```python
container.delete_item(
    item="item-1",
    partition_key="acme",
)
```

## Async Usage

Use the async client from `azure.cosmos.aio` in async applications. The official docs call out that you should enter the client so account metadata is cached before first use.

```python
import os
from azure.cosmos.aio import CosmosClient
from azure.identity.aio import DefaultAzureCredential

async def main() -> None:
    credential = DefaultAzureCredential()

    async with CosmosClient(
        url=os.environ["COSMOS_ENDPOINT"],
        credential=credential,
    ) as client:
        database = client.get_database_client("appdb")
        container = database.get_container_client("items")

        item = await container.read_item(
            item="item-1",
            partition_key="acme",
        )
        print(item["id"])

    await credential.close()
```

If you cannot use `async with`, call `await client.__aenter__()` before using the async client.

## Configuration Notes

- Keep endpoint and credentials in environment variables or your secret manager, not in source code.
- Use one shared `CosmosClient`; client construction is not a lightweight health check.
- Treat the partition key as required application context. Reads, patches, and deletes usually need it.
- `create_database_if_not_exists` and `create_container_if_not_exists` are convenient, but they still make control-plane calls.
- `no_response_on_write=True` is useful for ingestion-heavy services that do not need the returned document body.
- `preferred_locations` helps multi-region accounts prefer local reads; `excluded_locations` helps steer around regions you do not want to use.
- Retry settings only help transient failures. If you are being throttled, you still need to think about RU budget and workload shape.

## Common Pitfalls

- Using this SDK for the wrong Cosmos API surface. `azure-cosmos` here is for Cosmos DB for NoSQL.
- Designing the data model first and the partition key later. That usually leads to expensive queries and awkward rewrites.
- Creating a new `CosmosClient` per request or per operation.
- Forgetting the partition key on point reads, patches, and deletes.
- Using `upsert_item` where create-only semantics were required.
- Building SQL strings manually instead of using query parameters.
- Using Entra ID without assigning Cosmos DB data-plane permissions.
- Forgetting `async with` or `await client.__aenter__()` when using `azure.cosmos.aio.CosmosClient`.

## Version-Sensitive Notes For 4.15.0

- As of March 12, 2026, PyPI and Microsoft Learn both show `4.15.0`.
- The Azure SDK changelog for `4.15.0` notes a higher minimum `azure-core` requirement (`1.30.0`) and support for `excluded_locations`.
- The package still requires Python `>=3.9`.
- If you find older v3-era snippets using `DocumentClient`, treat them as obsolete and rewrite them to the current `CosmosClient` API.

## Official Sources

- Overview README: `https://learn.microsoft.com/en-us/python/api/overview/azure/cosmos-readme?view=azure-python`
- API reference root: `https://learn.microsoft.com/en-us/python/api/azure-cosmos/`
- Sync `CosmosClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-cosmos/azure.cosmos.cosmos_client.cosmosclient?view=azure-python`
- Async `CosmosClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-cosmos/azure.cosmos.aio.cosmosclient?view=azure-python`
- Azure Cosmos DB for NoSQL Python quickstart: `https://learn.microsoft.com/en-us/azure/cosmos-db/nosql/quickstart-python`
- PyPI package page: `https://pypi.org/project/azure-cosmos/`
- Azure SDK changelog: `https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/cosmos/azure-cosmos/CHANGELOG.md`

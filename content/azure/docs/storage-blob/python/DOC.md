---
name: storage-blob
description: "Azure Blob Storage Python SDK guide for authentication, clients, transfers, listing, and 12.28.0 version-sensitive behavior"
metadata:
  languages: "python"
  versions: "12.28.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,blob-storage,azure-storage,python,cloud,storage"
---

# Azure Blob Storage Python SDK

## Golden Rule

Use `azure-storage-blob` for blob operations and pair it with `azure-identity` plus `DefaultAzureCredential` for real Azure environments. For version `12.28.0`, follow the package overview and PyPI metadata for runtime requirements: Python `>=3.9`.

## Install

Pin the version your project expects:

```bash
python -m pip install "azure-storage-blob==12.28.0" azure-identity
```

For async usage, add an async transport:

```bash
python -m pip install "azure-storage-blob==12.28.0" azure-identity aiohttp
```

Common environment variables:

```bash
export AZURE_STORAGE_ACCOUNT_URL="https://<account>.blob.core.windows.net"
export AZURE_STORAGE_CONTAINER="documents"
```

Use `AZURE_STORAGE_CONNECTION_STRING` instead of `AZURE_STORAGE_ACCOUNT_URL` when you are developing against Azurite or using connection-string auth.

## Authentication And Setup

### Preferred: Microsoft Entra ID with `DefaultAzureCredential`

This is the official recommended path for Azure-hosted apps and normal local development.

Local setup:

1. Install `azure-identity`.
2. Sign in with `az login` or another supported developer credential source.
3. Grant the principal Azure RBAC access on the storage account or container.

Role guidance:

- Read-only flows such as listing or downloading need `Storage Blob Data Reader` or higher.
- Read/write flows such as upload, create, or delete need `Storage Blob Data Contributor` or higher.

RBAC assignments can take a few minutes to propagate. If a brand-new role assignment fails, retry after a short wait instead of rewriting the auth flow.

```python
import os

from azure.identity import DefaultAzureCredential
from azure.storage.blob import BlobServiceClient

account_url = os.environ["AZURE_STORAGE_ACCOUNT_URL"]
credential = DefaultAzureCredential()

service = BlobServiceClient(account_url=account_url, credential=credential)
```

### Connection string or Azurite

Connection strings are still useful for local storage emulators, migration tooling, and environments where you already have a storage secret.

```python
import os

from azure.storage.blob import BlobServiceClient

service = BlobServiceClient.from_connection_string(
    os.environ["AZURE_STORAGE_CONNECTION_STRING"]
)
```

Version `12.28.0` also adds support for the Azurite shorthand:

```python
service = BlobServiceClient.from_connection_string("UseDevelopmentStorage=true;")
```

### SAS token or account key

The SDK also supports SAS-token and account-key auth. Use them when required, but prefer Entra ID for application code because it avoids embedding long-lived storage secrets.

## Client Model

The SDK is organized around three client layers:

- `BlobServiceClient`: account scope
- `ContainerClient`: one container
- `BlobClient`: one blob

Typical flow:

```python
container = service.get_container_client("documents")
blob = container.get_blob_client("reports/hello.txt")
```

Create the container before expecting upload or download calls to work. The SDK does not auto-create missing containers.

## Core Usage

### Basic sync flow

```python
import os

from azure.core.exceptions import ResourceExistsError
from azure.identity import DefaultAzureCredential
from azure.storage.blob import BlobServiceClient

account_url = os.environ["AZURE_STORAGE_ACCOUNT_URL"]
container_name = os.getenv("AZURE_STORAGE_CONTAINER", "documents")

service = BlobServiceClient(account_url=account_url, credential=DefaultAzureCredential())
container = service.get_container_client(container_name)

try:
    container.create_container()
except ResourceExistsError:
    pass

blob = container.get_blob_client("reports/hello.txt")
blob.upload_blob(b"hello from azure-storage-blob\n", overwrite=True)

text = blob.download_blob(encoding="UTF-8").readall()
print(text)

for item in container.list_blobs(name_starts_with="reports/"):
    print(item.name, item.size)
```

### Upload

`upload_blob()` automatically chooses a single request or block-based upload depending on object size and transfer settings.

```python
from azure.storage.blob import ContentSettings

with open("report.json", "rb") as data:
    blob.upload_blob(
        data,
        overwrite=True,
        tags={"kind": "report"},
        content_settings=ContentSettings(content_type="application/json"),
    )
```

Useful upload notes:

- `overwrite=True` is usually necessary for idempotent application code.
- `standard_blob_tier=` can set `Hot`, `Cool`, `Cold`, or `Archive` for block blobs.
- For manual block staging, use `stage_block()` plus `commit_block_list()`.

### Download

`download_blob()` returns a `StorageStreamDownloader`. Use `readall()` for small payloads, `readinto()` for file writes, or `chunks()` for streaming.

```python
download_stream = blob.download_blob(max_concurrency=4)

with open("report.bin", "wb") as fh:
    download_stream.readinto(fh)
```

For chunked processing:

```python
stream = blob.download_blob()

for chunk in stream.chunks():
    process(chunk)
```

### List

Flat listing:

```python
for item in container.list_blobs(name_starts_with="reports/2026/"):
    print(item.name)
```

Hierarchical listing with virtual directories:

```python
from azure.storage.blob import BlobPrefix

for item in container.walk_blobs(name_starts_with="reports/", delimiter="/"):
    if isinstance(item, BlobPrefix):
        print("dir", item.name)
    else:
        print("blob", item.name)
```

Blob Storage is fundamentally flat. Folder-like behavior comes from blob naming plus a delimiter. Microsoft Learn also notes that blob snapshots cannot be listed with hierarchical listing.

## Async Pattern

Use async clients from `azure.storage.blob.aio`, and close both credentials and clients. The top-level service client should usually be managed with `async with`; child clients share its connection pool.

```python
import os

from azure.identity.aio import DefaultAzureCredential
from azure.storage.blob.aio import BlobServiceClient

async def main() -> None:
    account_url = os.environ["AZURE_STORAGE_ACCOUNT_URL"]

    async with DefaultAzureCredential() as credential:
        async with BlobServiceClient(account_url, credential=credential) as service:
            container = service.get_container_client("documents")
            async for item in container.list_blobs(name_starts_with="reports/"):
                print(item.name)
```

## Configuration Notes

The official overview and task guides call out a few settings that matter in production:

- Retry policy: `retry_total`, `retry_connect`, `retry_read`, `retry_status`, `retry_to_secondary`
- Timeouts: `connection_timeout`, `read_timeout`
- Logging: `logging_enable=True` for request diagnostics
- Upload tuning: `max_block_size`, `max_single_put_size`, and per-call `max_concurrency`
- Download tuning: `max_chunk_get_size`, `max_single_get_size`, and per-call `max_concurrency`

Start with defaults unless you have evidence that transfers or retry behavior need tuning for a specific workload.

## Common Pitfalls

- Some Microsoft Learn task articles still say Python `3.8+`, but the package overview and PyPI metadata for `12.28.0` require Python `>=3.9`. Follow the package metadata.
- Older overview snippets still mention async support on Python `3.5+`; that text is stale for current stable releases.
- `upload_blob()` does not protect you from concurrent writers. Microsoft explicitly documents that the client libraries do not support concurrent writes to the same blob.
- Containers must exist before upload or download examples succeed.
- Use `name_starts_with=` for listing filters; that is the keyword used by the current docs and examples.
- `DefaultAzureCredential` failures are often environment issues: not signed in, wrong tenant, or RBAC propagation delay.
- Debug logging can include request and response bodies. Do not enable it casually in production.

## Version-Sensitive Notes For `12.28.0`

- `12.28.0` is the stable release of the `12.28.0b1` feature set.
- `ContainerClient.list_blobs()`, `list_blob_names()`, and `walk_blobs()` gained a `start_from` keyword in the `12.28.0` feature line.
- `BlobClient.download_blob()` gained a `decompress` keyword in the `12.28.0` feature line.
- `BlobServiceClient.from_connection_string()` now accepts `UseDevelopmentStorage=true;` for Azurite.
- The default `connection_data_block_size` changed from `4 KiB` to `256 KiB`, which the changelog says should improve throughput for many large downloads.

## Official Sources Used

- Microsoft Learn package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/storage-blob-readme?view=azure-python`
- Microsoft Learn API root: `https://learn.microsoft.com/en-us/python/api/azure-storage-blob/`
- Microsoft Learn quickstart: `https://learn.microsoft.com/en-us/azure/storage/blobs/storage-quickstart-blobs-python`
- Microsoft Learn upload guide: `https://learn.microsoft.com/en-us/azure/storage/blobs/storage-blob-upload-python`
- Microsoft Learn download guide: `https://learn.microsoft.com/en-us/azure/storage/blobs/storage-blob-download-python`
- Microsoft Learn list guide: `https://learn.microsoft.com/en-us/azure/storage/blobs/storage-blobs-list-python`
- PyPI package page: `https://pypi.org/project/azure-storage-blob/`
- Azure SDK changelog: `https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/storage/azure-storage-blob/CHANGELOG.md`

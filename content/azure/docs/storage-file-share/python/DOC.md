---
name: storage-file-share
description: "Azure Files client library for Python with practical guidance for auth, shares, directories, file transfers, and version-sensitive behavior"
metadata:
  languages: "python"
  versions: "12.24.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,storage,file-share,python,files,sas,entra-id"
---

# Azure Files Share Python Client Library

## Golden Rule

Use `azure-storage-file-share` for Azure Files data-plane access, import it from `azure.storage.fileshare`, and use the Azure Files endpoint shape `https://<account>.file.core.windows.net`. Do not mix Azure Files examples with Blob Storage endpoints or the legacy `azure-storage-file` package.

## Install

Pin the version your project expects:

```bash
python -m pip install "azure-storage-file-share==12.24.0"
```

Common alternatives:

```bash
uv add "azure-storage-file-share==12.24.0"
poetry add "azure-storage-file-share==12.24.0"
```

Install auth or async companions only when needed:

```bash
python -m pip install "azure-storage-file-share==12.24.0" azure-identity
python -m pip install "azure-storage-file-share==12.24.0" aiohttp
```

## Authentication And Setup

You can construct clients with:

- a connection string
- an account key or `AzureNamedKeyCredential`
- a SAS token or `AzureSasCredential`
- a `TokenCredential` such as `DefaultAzureCredential`

Use the file service endpoint, not blob or dfs endpoints:

```text
https://<storage-account>.file.core.windows.net
```

### Connection String

This is the fastest local setup when you already have the storage account connection string:

```python
import os
from azure.storage.fileshare import ShareServiceClient

service_client = ShareServiceClient.from_connection_string(
    os.environ["AZURE_STORAGE_CONNECTION_STRING"]
)
share_client = service_client.get_share_client("agent-docs")
```

### Microsoft Entra ID

Prefer `DefaultAzureCredential` in Azure-hosted environments and in local development that already uses Azure CLI or workload identity. For Azure Files token auth, set `token_intent` explicitly.

```python
from azure.identity import DefaultAzureCredential
from azure.storage.fileshare import ShareServiceClient

credential = DefaultAzureCredential()

service_client = ShareServiceClient(
    account_url="https://<storage-account>.file.core.windows.net",
    credential=credential,
    token_intent="backup",
)
```

Practical notes:

- `token_intent` is specific to Azure Files token-credential scenarios and is easy to miss if you copy older snippets.
- The principal still needs the correct Azure Files data-plane role assignment.
- If you do not need Entra ID, a connection string is usually simpler.

### Shared Key Or SAS

Use shared key credentials for direct account-level access, or SAS for narrower permissions:

```python
from azure.core.credentials import AzureNamedKeyCredential
from azure.storage.fileshare import ShareServiceClient

service_client = ShareServiceClient(
    account_url="https://<storage-account>.file.core.windows.net",
    credential=AzureNamedKeyCredential("<storage-account>", "<account-key>"),
)
```

```python
from azure.core.credentials import AzureSasCredential
from azure.storage.fileshare import ShareClient

share_client = ShareClient(
    account_url="https://<storage-account>.file.core.windows.net",
    share_name="agent-docs",
    credential=AzureSasCredential("<sas-token>"),
)
```

## Core Usage

The normal sync flow is:

1. Create a `ShareServiceClient`.
2. Get or create a `ShareClient`.
3. Create parent directories before uploading files.
4. Use `ShareFileClient` to upload or download content.

### Create A Share, Upload A File, Download It

```python
import os
from azure.storage.fileshare import ShareServiceClient

service_client = ShareServiceClient.from_connection_string(
    os.environ["AZURE_STORAGE_CONNECTION_STRING"]
)

share_client = service_client.get_share_client("agent-docs")
if not share_client.exists():
    share_client.create_share()

directory_client = share_client.get_directory_client("incoming")
if not directory_client.exists():
    directory_client.create_directory()

file_client = directory_client.get_file_client("hello.txt")
file_client.upload_file(b"hello from Azure Files\n")

downloader = file_client.download_file()
content = downloader.readall().decode("utf-8")
print(content)
```

### List Directories And Files

```python
for item in directory_client.list_directories_and_files():
    print(item["name"], item["is_directory"])
```

### Work Directly With A Known File Path

```python
import os
from azure.storage.fileshare import ShareFileClient

file_client = ShareFileClient.from_connection_string(
    conn_str=os.environ["AZURE_STORAGE_CONNECTION_STRING"],
    share_name="agent-docs",
    file_path="incoming/hello.txt",
)

data = file_client.download_file().readall()
print(data.decode("utf-8"))
```

### Async Usage

Async clients live under `azure.storage.fileshare.aio`. Close them with `async with`, and close credential objects when they support async cleanup.

```python
import asyncio
import os
from azure.storage.fileshare.aio import ShareFileClient

async def main() -> None:
    file_client = ShareFileClient.from_connection_string(
        conn_str=os.environ["AZURE_STORAGE_CONNECTION_STRING"],
        share_name="agent-docs",
        file_path="incoming/hello.txt",
    )

    async with file_client:
        downloader = await file_client.download_file(max_concurrency=4)
        data = await downloader.readall()
        print(data.decode("utf-8"))

asyncio.run(main())
```

## Configuration Notes

The package overview documents several transfer tuning knobs:

- `max_range_size`: upload chunk size, default `4 * 1024 * 1024`
- `max_single_put_size`: single-request upload threshold, default `64 * 1024 * 1024`
- `max_single_get_size`: single-request download threshold, default `32 * 1024 * 1024`
- `connection_data_block_size`: socket read block size for downloads
- `max_concurrency`: parallel workers for uploads and downloads

In `12.24.0`, the package overview notes that `connection_data_block_size` changed from `4 * 1024` to `256 * 1024`. If you are comparing throughput with older `12.x` examples, this is one of the first settings to check.

Start with defaults unless you are actively tuning large-file transfer performance.

## Common Pitfalls

- The install name is `azure-storage-file-share`, but the import path is `azure.storage.fileshare`.
- Old docs or blog posts may still refer to `azure-storage-file`; prefer current `12.x` docs and namespace names.
- Use the Files endpoint `file.core.windows.net`; Blob Storage examples use different clients and endpoints.
- Create the share and parent directories before uploading a file into them.
- `download_file()` returns a downloader object, not raw bytes. Use `readall()` or `readinto()`.
- Async usage needs `aiohttp`, and async clients should be closed cleanly.
- Token-credential flows for Azure Files need `token_intent`; missing it can look like a generic auth or permission failure.
- Azure Files permissions are data-plane permissions. Valid Azure login state alone does not guarantee the required share access.

## Version-Sensitive Notes

- This guide targets `12.24.0`, which is the stable version listed on PyPI and in the current Azure package overview on March 12, 2026.
- `12.24.0` adds support for storage service API version `2026-02-06`.
- The `12.24.0` changelog also adds `ShareServiceClient.get_user_delegation_key` and user delegation SAS support. If your code needs user delegation SAS for Azure Files, earlier versions will not match current docs.
- The PyPI package page still documents the rename from `azure-storage-file` to `azure-storage-file-share`; that matters when updating old requirements files or import paths.

## Official Sources

- Package overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/storage-file-share-readme?view=azure-python`
- API reference root: `https://learn.microsoft.com/en-us/python/api/azure-storage-file-share/`
- `ShareServiceClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-storage-file-share/azure.storage.fileshare.shareserviceclient?view=azure-python`
- Azure Files Python how-to: `https://learn.microsoft.com/en-us/azure/storage/files/storage-python-how-to-use-file-storage`
- Changelog: `https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/storage/azure-storage-file-share/CHANGELOG.md`
- PyPI package page: `https://pypi.org/project/azure-storage-file-share/`

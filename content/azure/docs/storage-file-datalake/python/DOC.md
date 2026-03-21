---
name: storage-file-datalake
description: "Azure Storage Data Lake client library for Python for ADLS Gen2 filesystem, directory, file, and ACL operations"
metadata:
  languages: "python"
  versions: "12.23.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,storage,data-lake,adls,adls-gen2,filesystem,files,acl"
---

# Azure Storage Data Lake Client Library For Python

## Golden Rule

Use `azure-storage-file-datalake` when you need Azure Data Lake Storage Gen2 features that depend on hierarchical namespaces: filesystems, directories, files, and POSIX-style ACLs. Construct clients against the `dfs` endpoint, prefer Microsoft Entra authentication via `DefaultAzureCredential`, and pin the stable package release your project expects. As of March 12, 2026, PyPI lists `12.23.0` as the latest stable release and `12.24.0b1` as a preview; some official Azure overview pages still show preview-era `pip install --pre` snippets, so do not copy those unless you intentionally want the beta.

## Install

Pin the stable version unless the project explicitly depends on a preview:

```bash
python -m pip install "azure-storage-file-datalake==12.23.0" "azure-identity>=1.17.0"
```

Common alternatives:

```bash
uv add "azure-storage-file-datalake==12.23.0" azure-identity
poetry add "azure-storage-file-datalake==12.23.0"
poetry add azure-identity
```

If you are using the async clients, install an async HTTP transport too:

```bash
python -m pip install "azure-storage-file-datalake==12.23.0" "azure-identity>=1.17.0" aiohttp
```

Notes:

- `azure-identity` is a separate package; `DefaultAzureCredential` does not come from `azure-storage-file-datalake`.
- Data Lake Storage Gen2 features require a storage account with hierarchical namespace enabled.
- Use `--pre` only when you intentionally want a preview build such as `12.24.0b1`.

## Authentication And Endpoint Setup

For most new code, authenticate with Microsoft Entra ID and `DefaultAzureCredential`:

```python
import os
from azure.identity import DefaultAzureCredential
from azure.storage.filedatalake import DataLakeServiceClient

account_name = os.environ["AZURE_STORAGE_ACCOUNT"]
account_url = f"https://{account_name}.dfs.core.windows.net"

service_client = DataLakeServiceClient(
    account_url=account_url,
    credential=DefaultAzureCredential(),
)
```

Important setup rules:

- Use the `dfs.core.windows.net` endpoint, not `blob.core.windows.net`, for Data Lake clients.
- The four core sync clients are `DataLakeServiceClient`, `FileSystemClient`, `DataLakeDirectoryClient`, and `DataLakeFileClient`.
- `DefaultAzureCredential` works well for local development with Azure CLI or developer login, and for deployed workloads with managed identity.
- If the storage account is in a sovereign cloud, change the hostname to that cloud's DFS endpoint rather than hard-coding `core.windows.net`.

Other supported credential styles:

```python
import os
from azure.storage.filedatalake import DataLakeServiceClient

service_client = DataLakeServiceClient(
    account_url="https://myaccount.dfs.core.windows.net",
    credential=os.environ["AZURE_STORAGE_ACCOUNT_KEY"],
)

sas_client = DataLakeServiceClient(
    account_url="https://myaccount.dfs.core.windows.net",
    credential=os.environ["AZURE_STORAGE_SAS_TOKEN"],
)
```

Connection string setup is also supported:

```python
import os
from azure.storage.filedatalake import DataLakeServiceClient

service_client = DataLakeServiceClient.from_connection_string(
    os.environ["AZURE_STORAGE_CONNECTION_STRING"]
)
```

For local emulator workflows, the `12.23.0` line supports the short Azurite connection string form:

```python
from azure.storage.filedatalake import DataLakeServiceClient

service_client = DataLakeServiceClient.from_connection_string("UseDevelopmentStorage=true;")
```

## Client Model

The client hierarchy mirrors the storage hierarchy:

- `DataLakeServiceClient`: account-level operations and filesystem creation
- `FileSystemClient`: one filesystem (container) within the account
- `DataLakeDirectoryClient`: one directory path
- `DataLakeFileClient`: one file path

Typical client construction:

```python
from azure.identity import DefaultAzureCredential
from azure.storage.filedatalake import DataLakeServiceClient

service = DataLakeServiceClient(
    account_url="https://myaccount.dfs.core.windows.net",
    credential=DefaultAzureCredential(),
)

file_system = service.get_file_system_client("raw")
directory = file_system.get_directory_client("incoming/2026/03/12")
file_client = file_system.get_file_client("incoming/2026/03/12/events.jsonl")
```

## Core Usage

### Create A Filesystem And Upload A File

```python
from azure.core.exceptions import ResourceExistsError
from azure.identity import DefaultAzureCredential
from azure.storage.filedatalake import DataLakeServiceClient

service = DataLakeServiceClient(
    account_url="https://myaccount.dfs.core.windows.net",
    credential=DefaultAzureCredential(),
)

try:
    file_system = service.create_file_system(file_system="raw")
except ResourceExistsError:
    file_system = service.get_file_system_client("raw")

directory = file_system.get_directory_client("incoming/2026/03/12")

try:
    directory.create_directory()
except ResourceExistsError:
    pass

file_client = file_system.get_file_client("incoming/2026/03/12/events.jsonl")
payload = b'{"id": 1}\n{"id": 2}\n'

file_client.create_file()
file_client.upload_data(payload, overwrite=True)
```

`upload_data()` is the simplest whole-body write API. Pass `overwrite=True` when replacing an existing file.

### Append Incrementally And Flush

If you need explicit append/commit control, use `append_data()` followed by `flush_data()`:

```python
from azure.storage.filedatalake import DataLakeServiceClient

service = DataLakeServiceClient.from_connection_string(
    "UseDevelopmentStorage=true;"
)

file_system = service.get_file_system_client("raw")
file_client = file_system.get_file_client("incoming/events.jsonl")

file_client.create_file()

chunks = [b'{"id": 1}\n', b'{"id": 2}\n']
offset = 0

for chunk in chunks:
    file_client.append_data(chunk, offset=offset, length=len(chunk))
    offset += len(chunk)

file_client.flush_data(offset)
```

If you skip `flush_data()`, the appended bytes are not committed as a readable file.

### Download A File

```python
from azure.identity import DefaultAzureCredential
from azure.storage.filedatalake import DataLakeServiceClient

service = DataLakeServiceClient(
    account_url="https://myaccount.dfs.core.windows.net",
    credential=DefaultAzureCredential(),
)

file_client = service.get_file_system_client("raw").get_file_client(
    "incoming/2026/03/12/events.jsonl"
)

download = file_client.download_file()
content = download.readall().decode("utf-8")
print(content)
```

For large reads, prefer streaming methods like `chunks()` or `readinto()` instead of materializing the whole file into memory.

### List Paths

```python
from azure.identity import DefaultAzureCredential
from azure.storage.filedatalake import DataLakeServiceClient

service = DataLakeServiceClient(
    account_url="https://myaccount.dfs.core.windows.net",
    credential=DefaultAzureCredential(),
)

file_system = service.get_file_system_client("raw")

for path in file_system.get_paths(path="incoming", recursive=True):
    print(path.name, path.is_directory)
```

`get_paths()` returns path items relative to the filesystem. The returned `name` is not a full URL.

### Manage ACLs On A Directory

Use the Data Lake clients when you need POSIX-style ownership and ACL behavior:

```python
from azure.identity import DefaultAzureCredential
from azure.storage.filedatalake import DataLakeServiceClient

service = DataLakeServiceClient(
    account_url="https://myaccount.dfs.core.windows.net",
    credential=DefaultAzureCredential(),
)

directory = service.get_file_system_client("raw").get_directory_client("incoming")

directory.set_access_control(acl="user::rwx,group::r-x,other::---")

acl_info = directory.get_access_control(upn=True)
print(acl_info["acl"])
```

For existing directory trees, apply ACLs recursively:

```python
directory = service.get_file_system_client("raw").get_directory_client("incoming")

result = directory.set_access_control_recursive(
    acl="user::rwx,group::r-x,other::---"
)

print(result.counters)
```

`upn=True` converts Microsoft Entra user object IDs to user principal names when possible. It does not convert group or application object IDs.

## SAS Helpers

The package includes helper functions for shared access signatures:

- `generate_file_system_sas`
- `generate_directory_sas`
- `generate_file_sas`

Use SAS only when you already have a clear account-key or user-delegation-key workflow. For most first-party Azure code, Microsoft Entra authentication is the better default.

## Async Usage

Async clients live under `azure.storage.filedatalake.aio`, and the async credential types live under `azure.identity.aio`:

```python
import asyncio
from azure.identity.aio import DefaultAzureCredential
from azure.storage.filedatalake.aio import DataLakeServiceClient

async def main() -> None:
    async with DefaultAzureCredential() as credential:
        async with DataLakeServiceClient(
            account_url="https://myaccount.dfs.core.windows.net",
            credential=credential,
        ) as service:
            file_system = service.get_file_system_client("raw")

            async for path in file_system.get_paths(path="incoming", recursive=True):
                print(path.name)

asyncio.run(main())
```

Close async credentials and clients cleanly with `async with` or explicit `close()` calls.

## Configuration Notes

- Prefer `DefaultAzureCredential` unless the deployment environment requires a SAS token, account key, or connection string.
- Keep the account URL explicit in config so agents do not accidentally mix the Blob and DFS endpoints.
- If your code path needs both Blob APIs and Data Lake ACL or directory semantics, keep it clear which client is operating on which endpoint.
- For local developer shells, `az login` plus `DefaultAzureCredential` is usually simpler than manually managing storage keys.
- Recursive ACL changes can touch large directory trees; treat them as potentially long-running operations and inspect the returned counters.

## Common Pitfalls

- `DataLakeServiceClient` should target `https://<account>.dfs.core.windows.net`. Using the Blob endpoint causes confusing failures.
- Hierarchical namespace must be enabled on the storage account. Without it, Data Lake directory and ACL semantics are not available.
- `azure-storage-file-datalake` does not install `azure-identity` for you.
- `append_data()` is not enough by itself; you must call `flush_data()` to commit the buffered bytes.
- `upload_data()` replaces content only when `overwrite=True` is set. Otherwise existing files can raise `ResourceExistsError`.
- Azure RBAC and POSIX ACLs are separate controls. A principal can have Azure-level access yet still fail on a specific path because of ACLs.
- `get_access_control(upn=True)` only rewrites user IDs to UPNs when Azure can resolve them. Groups and service principals still come back as object IDs.
- Some official Azure overview pages still show preview-era install commands. For stable production code, trust PyPI and the package changelog for release selection.

## Version-Sensitive Notes For 12.23.0

- PyPI lists `12.23.0` as the latest stable package release on March 12, 2026.
- PyPI also shows a newer preview, `12.24.0b1`, so agents should avoid `--pre` unless a project explicitly depends on preview behavior.
- The `12.23.0` line supports the short Azurite connection string form `UseDevelopmentStorage=true;`, which is useful for local emulator tests.
- PyPI requires Python `>=3.9`, so Python 3.8 environments need an older package line.

## Official Source URLs

- Microsoft Learn overview: `https://learn.microsoft.com/en-us/python/api/overview/azure/storage-file-datalake-readme?view=azure-python`
- Microsoft Learn API reference: `https://learn.microsoft.com/en-us/python/api/azure-storage-file-datalake/azure.storage.filedatalake?view=azure-python`
- Microsoft Learn directory and file guide: `https://learn.microsoft.com/en-us/azure/storage/blobs/data-lake-storage-directory-file-acl-python`
- Microsoft Learn ACL guide: `https://learn.microsoft.com/en-us/azure/storage/blobs/data-lake-storage-acl-python`
- PyPI package page: `https://pypi.org/project/azure-storage-file-datalake/`
- Azure SDK changelog: `https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/storage/azure-storage-file-datalake/CHANGELOG.md`

---
name: onedrive
description: "Microsoft OneDrive API coding guidelines for Python using the official Microsoft Graph SDK"
metadata:
  languages: "python"
  versions: "1.48.0"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "microsoft,onedrive,storage,graph-api,files"
---

# Microsoft OneDrive API Coding Guidelines (Python)

You are a **Microsoft OneDrive API coding expert**. Help me write correct, idiomatic Python code that accesses OneDrive files and folders using the official Microsoft Graph SDK.

Use **only official Microsoft sources** for behavior, fields, and constraints. This guide summarizes key patterns for Python applications.

> Ground truth: Microsoft Graph OneDrive API documentation on learn.microsoft.com.


## Golden Rule: Use the Official Microsoft Graph SDK

**ALWAYS use `msgraph-sdk` version 1.48.0 or later** for OneDrive operations. This is the official Microsoft Graph Python SDK that provides access to OneDrive, SharePoint, and all other Microsoft Graph APIs.

**DO NOT use**:
- Deprecated `onedrivesdk` package (obsolete)
- Direct REST calls without the SDK (unless absolutely necessary)
- Unofficial third-party OneDrive libraries

**Install (Python):**
```bash
pip install msgraph-sdk
pip install azure-identity
```


## Installation

### Complete Setup for Python Applications

```bash
# Core Microsoft Graph SDK
pip install msgraph-sdk

# Azure authentication library
pip install azure-identity

# For async support (recommended)
pip install aiohttp

# Environment variable management
pip install python-dotenv
```

### Using requirements.txt

```text
msgraph-sdk>=1.48.0
azure-identity>=1.19.0
python-dotenv>=1.0.0
aiohttp>=3.9.0
```

### Using pyproject.toml (Poetry/UV)

```toml
[project]
dependencies = [
    "msgraph-sdk>=1.48.0",
    "azure-identity>=1.19.0",
    "python-dotenv>=1.0.0",
    "aiohttp>=3.9.0"
]
```


## Authentication

OneDrive access through Microsoft Graph requires OAuth 2.0 authentication. You need to register an application in Azure Active Directory (Azure AD) to obtain credentials.

### Azure AD App Registration

1. Go to [Azure Portal](https://portal.azure.com) → Azure Active Directory → App registrations
2. Create a new registration
3. Note your **Application (client) ID** and **Directory (tenant) ID**
4. Create a client secret under "Certificates & secrets"
5. Add API permissions: Microsoft Graph → **Files.Read**, **Files.ReadWrite**, **Files.Read.All**, **Files.ReadWrite.All**
6. Grant admin consent for the permissions

### Required Scopes

Common OneDrive permission scopes:

```python
# Read-only access to user's files
SCOPES_READONLY = ['https://graph.microsoft.com/Files.Read']

# Read/write access to user's files
SCOPES_READWRITE = ['https://graph.microsoft.com/Files.ReadWrite']

# Read all files user can access (including shared)
SCOPES_READ_ALL = ['https://graph.microsoft.com/Files.Read.All']

# Full access to all files user can access
SCOPES_READWRITE_ALL = ['https://graph.microsoft.com/Files.ReadWrite.All']

# Application permissions (no user context, requires admin consent)
SCOPES_APP = ['https://graph.microsoft.com/.default']
```


## Initialization

### Async Client with Client Credentials (Service/Daemon Apps)

For server-side applications using application permissions:

```python
import asyncio
from azure.identity.aio import ClientSecretCredential
from msgraph import GraphServiceClient
from msgraph.generated.users.item.user_item_request_builder import UserItemRequestBuilder
import os
from dotenv import load_dotenv

load_dotenv()

# Environment variables
tenant_id = os.getenv('AZURE_TENANT_ID')
client_id = os.getenv('AZURE_CLIENT_ID')
client_secret = os.getenv('AZURE_CLIENT_SECRET')

# Create credential
credentials = ClientSecretCredential(
    tenant_id=tenant_id,
    client_id=client_id,
    client_secret=client_secret
)

# Scopes for application permissions
scopes = ['https://graph.microsoft.com/.default']

# Initialize Microsoft Graph client
client = GraphServiceClient(credentials=credentials, scopes=scopes)

# Example: List files in user's OneDrive root
async def list_files(user_id: str):
    result = await client.users.by_user_id(user_id).drive.root.children.get()
    return result.value

async def main():
    user_id = os.getenv('USER_ID')
    files = await list_files(user_id)

    for file in files:
        print(f"{file.name} - {file.size} bytes")

if __name__ == '__main__':
    asyncio.run(main())
```

### Sync Client with Client Credentials

For synchronous applications:

```python
from azure.identity import ClientSecretCredential
from msgraph import GraphServiceClient
import os

credentials = ClientSecretCredential(
    tenant_id=os.getenv('AZURE_TENANT_ID'),
    client_id=os.getenv('AZURE_CLIENT_ID'),
    client_secret=os.getenv('AZURE_CLIENT_SECRET')
)

scopes = ['https://graph.microsoft.com/.default']
client = GraphServiceClient(credentials=credentials, scopes=scopes)

# Use sync methods
def list_files_sync(user_id: str):
    # Note: The SDK is primarily async, synchronous usage requires running in event loop
    import asyncio
    loop = asyncio.get_event_loop()
    result = loop.run_until_complete(
        client.users.by_user_id(user_id).drive.root.children.get()
    )
    return result.value
```

### Async Client with Device Code Flow (Interactive)

For CLI applications that need user interaction:

```python
from azure.identity.aio import DeviceCodeCredential
from msgraph import GraphServiceClient
import os

async def init_graph_client():
    credentials = DeviceCodeCredential(
        client_id=os.getenv('AZURE_CLIENT_ID'),
        tenant_id=os.getenv('AZURE_TENANT_ID')
    )

    scopes = ['Files.ReadWrite', 'Files.Read.All']
    client = GraphServiceClient(credentials=credentials, scopes=scopes)

    return client

async def main():
    client = await init_graph_client()

    # List files in current user's drive
    result = await client.me.drive.root.children.get()

    for item in result.value:
        print(f"{item.name}")

asyncio.run(main())
```

### Async Client with Interactive Browser

For desktop applications with interactive login:

```python
from azure.identity.aio import InteractiveBrowserCredential
from msgraph import GraphServiceClient
import os

async def init_graph_client():
    credentials = InteractiveBrowserCredential(
        client_id=os.getenv('AZURE_CLIENT_ID'),
        tenant_id=os.getenv('AZURE_TENANT_ID')
    )

    scopes = ['Files.ReadWrite']
    client = GraphServiceClient(credentials=credentials, scopes=scopes)

    return client
```

### Environment Variables Setup

Create a `.env` file:

```bash
AZURE_TENANT_ID=your-tenant-id
AZURE_CLIENT_ID=your-client-id
AZURE_CLIENT_SECRET=your-client-secret
USER_ID=user@domain.com
```

Load environment variables:

```python
from dotenv import load_dotenv
import os

load_dotenv()

tenant_id = os.getenv('AZURE_TENANT_ID')
client_id = os.getenv('AZURE_CLIENT_ID')
client_secret = os.getenv('AZURE_CLIENT_SECRET')
```


## Core API Surfaces


### 1. Listing Files and Folders

#### List Root Folder Contents

```python
from msgraph import GraphServiceClient

async def list_root_files(client: GraphServiceClient):
    # Current user's OneDrive root
    result = await client.me.drive.root.children.get()

    if result and result.value:
        for item in result.value:
            item_type = "Folder" if item.folder else "File"
            print(f"{item_type}: {item.name} (ID: {item.id})")

    return result.value
```

#### List Specific Folder Contents

```python
# By item ID
async def list_folder_by_id(client: GraphServiceClient, item_id: str):
    result = await client.me.drive.items.by_drive_item_id(item_id).children.get()
    return result.value

# By path
async def list_folder_by_path(client: GraphServiceClient, folder_path: str):
    # Example: /Documents/Projects
    result = await client.me.drive.root.item_with_path(folder_path).children.get()
    return result.value

# With pagination
async def list_all_items_paginated(client: GraphServiceClient):
    items = []
    result = await client.me.drive.root.children.get()

    while result:
        if result.value:
            items.extend(result.value)

        # Check for next page
        if hasattr(result, 'odata_next_link') and result.odata_next_link:
            # Fetch next page
            result = await client.me.drive.root.children.get()
        else:
            break

    return items
```

#### Advanced Listing with Query Parameters

```python
from msgraph.generated.users.item.drive.root.children.children_request_builder import ChildrenRequestBuilder

async def list_with_filters(client: GraphServiceClient):
    # Configure request
    query_params = ChildrenRequestBuilder.ChildrenRequestBuilderGetQueryParameters(
        select=['id', 'name', 'size', 'created_date_time', 'last_modified_date_time'],
        filter='file ne null',  # Only files
        orderby=['name asc'],
        top=10
    )

    request_config = ChildrenRequestBuilder.ChildrenRequestBuilderGetRequestConfiguration(
        query_parameters=query_params
    )

    result = await client.me.drive.root.children.get(request_configuration=request_config)
    return result.value

# Filter folders only
async def list_folders_only(client: GraphServiceClient):
    query_params = ChildrenRequestBuilder.ChildrenRequestBuilderGetQueryParameters(
        filter='folder ne null'
    )

    request_config = ChildrenRequestBuilder.ChildrenRequestBuilderGetRequestConfiguration(
        query_parameters=query_params
    )

    result = await client.me.drive.root.children.get(request_configuration=request_config)
    return result.value

# Order by last modified date
async def list_by_modified_date(client: GraphServiceClient):
    query_params = ChildrenRequestBuilder.ChildrenRequestBuilderGetQueryParameters(
        orderby=['lastModifiedDateTime desc'],
        top=20
    )

    request_config = ChildrenRequestBuilder.ChildrenRequestBuilderGetRequestConfiguration(
        query_parameters=query_params
    )

    result = await client.me.drive.root.children.get(request_configuration=request_config)
    return result.value
```

#### List All Drives

```python
# List all drives accessible to user
async def list_drives(client: GraphServiceClient):
    drives = await client.me.drives.get()

    if drives and drives.value:
        for drive in drives.value:
            print(f"Drive: {drive.name} (ID: {drive.id})")
            print(f"  Type: {drive.drive_type}")
            print(f"  Owner: {drive.owner.user.display_name if drive.owner else 'N/A'}")

    return drives.value

# Get default drive
async def get_default_drive(client: GraphServiceClient):
    drive = await client.me.drive.get()
    return drive

# Get specific drive
async def get_drive_by_id(client: GraphServiceClient, drive_id: str):
    drive = await client.drives.by_drive_id(drive_id).get()
    return drive
```


### 2. Getting File/Folder Metadata

#### Get Item Metadata

```python
# By item ID
async def get_item_metadata(client: GraphServiceClient, item_id: str):
    item = await client.me.drive.items.by_drive_item_id(item_id).get()

    print(f"ID: {item.id}")
    print(f"Name: {item.name}")
    print(f"Size: {item.size} bytes")
    print(f"Created: {item.created_date_time}")
    print(f"Modified: {item.last_modified_date_time}")
    print(f"Web URL: {item.web_url}")

    if item.file:
        print(f"MIME Type: {item.file.mime_type}")
        if item.file.hashes:
            print(f"SHA1 Hash: {item.file.hashes.sha1_hash}")

    return item

# By path
async def get_item_by_path(client: GraphServiceClient, file_path: str):
    # Example: /Documents/report.pdf
    item = await client.me.drive.root.item_with_path(file_path).get()
    return item
```

#### Get File with Specific Properties

```python
from msgraph.generated.users.item.drive.items.item.drive_item_item_request_builder import DriveItemItemRequestBuilder

async def get_item_with_select(client: GraphServiceClient, item_id: str):
    query_params = DriveItemItemRequestBuilder.DriveItemItemRequestBuilderGetQueryParameters(
        select=['id', 'name', 'size', 'file', 'createdDateTime']
    )

    request_config = DriveItemItemRequestBuilder.DriveItemItemRequestBuilderGetRequestConfiguration(
        query_parameters=query_params
    )

    item = await client.me.drive.items.by_drive_item_id(item_id).get(
        request_configuration=request_config
    )

    return item
```


### 3. Downloading Files

#### Simple Download (All File Sizes)

```python
import aiofiles

async def download_file(client: GraphServiceClient, item_id: str, save_path: str):
    # Get file content as stream
    stream = await client.me.drive.items.by_drive_item_id(item_id).content.get()

    # Save to disk
    async with aiofiles.open(save_path, 'wb') as f:
        await f.write(stream)

    print(f"Downloaded to {save_path}")

# Download by path
async def download_file_by_path(client: GraphServiceClient, file_path: str, save_path: str):
    # Example: /Documents/report.pdf
    stream = await client.me.drive.root.item_with_path(file_path).content.get()

    async with aiofiles.open(save_path, 'wb') as f:
        await f.write(stream)
```

#### Download Multiple Files

```python
import os
import asyncio

async def download_folder(client: GraphServiceClient, folder_id: str, local_path: str):
    # Create local directory
    os.makedirs(local_path, exist_ok=True)

    # Get folder contents
    items = await client.me.drive.items.by_drive_item_id(folder_id).children.get()

    if not items or not items.value:
        return

    for item in items.value:
        if item.folder:
            # Recursively download subfolder
            subfolder_path = os.path.join(local_path, item.name)
            await download_folder(client, item.id, subfolder_path)
        elif item.file:
            # Download file
            print(f"Downloading {item.name}...")
            file_path = os.path.join(local_path, item.name)

            stream = await client.me.drive.items.by_drive_item_id(item.id).content.get()

            async with aiofiles.open(file_path, 'wb') as f:
                await f.write(stream)

            print(f"Downloaded {item.name}")
```


### 4. Uploading Files

#### Simple Upload (Files < 4MB)

```python
async def upload_small_file(client: GraphServiceClient, file_path: str, upload_path: str):
    """
    Upload a file smaller than 4MB

    Args:
        file_path: Local file path
        upload_path: OneDrive path (e.g., '/Documents/file.pdf')
    """
    async with aiofiles.open(file_path, 'rb') as f:
        content = await f.read()

    # Upload to specific path
    item = await client.me.drive.root.item_with_path(upload_path).content.put(content)

    print(f"Uploaded: {item.name} (ID: {item.id})")
    return item

# Upload to specific folder by ID
async def upload_to_folder(client: GraphServiceClient, file_path: str, folder_id: str, filename: str):
    async with aiofiles.open(file_path, 'rb') as f:
        content = await f.read()

    item = await client.me.drive.items.by_drive_item_id(folder_id).item_with_path(filename).content.put(content)

    return item
```

#### Large File Upload (Files > 4MB) - Resumable Upload Session

```python
import math
import aiohttp

async def upload_large_file(client: GraphServiceClient, file_path: str, upload_path: str):
    """
    Upload large files using resumable upload session

    Args:
        file_path: Local file path
        upload_path: OneDrive path (e.g., '/Documents/video.mp4')
    """
    import os
    from msgraph.generated.drives.item.items.item.create_upload_session.create_upload_session_post_request_body import CreateUploadSessionPostRequestBody
    from msgraph.generated.models.drive_item_uploadable_properties import DriveItemUploadableProperties

    file_size = os.path.getsize(file_path)

    # Step 1: Create upload session
    props = DriveItemUploadableProperties()
    props.odata_type = "#microsoft.graph.driveItemUploadableProperties"
    props.microsoft_graph_conflict_behavior = "rename"  # or 'fail', 'replace'
    props.name = os.path.basename(file_path)

    request_body = CreateUploadSessionPostRequestBody()
    request_body.item = props

    upload_session = await client.me.drive.root.item_with_path(upload_path).create_upload_session.post(request_body)

    # Step 2: Upload file in chunks
    chunk_size = 320 * 1024 * 10  # 3.2 MB chunks (must be multiple of 320 KB)

    async with aiofiles.open(file_path, 'rb') as f:
        file_content = await f.read()

    num_chunks = math.ceil(file_size / chunk_size)

    for i in range(num_chunks):
        start = i * chunk_size
        end = min(start + chunk_size, file_size)
        chunk = file_content[start:end]

        content_range = f"bytes {start}-{end-1}/{file_size}"

        headers = {
            'Content-Length': str(len(chunk)),
            'Content-Range': content_range
        }

        # Upload chunk
        async with aiohttp.ClientSession() as session:
            async with session.put(
                upload_session.upload_url,
                data=chunk,
                headers=headers
            ) as response:
                if response.status in [200, 201, 202]:
                    progress = (end / file_size) * 100
                    print(f"Upload progress: {progress:.2f}%")
                else:
                    error_text = await response.text()
                    raise Exception(f"Upload failed: {error_text}")

    print("Upload complete!")
    return upload_session

# Alternative: Using helper function for large uploads
async def upload_large_file_simple(client: GraphServiceClient, file_path: str, folder_id: str):
    """
    Simplified large file upload
    """
    import os
    from msgraph.generated.drives.item.items.item.create_upload_session.create_upload_session_post_request_body import CreateUploadSessionPostRequestBody
    from msgraph.generated.models.drive_item_uploadable_properties import DriveItemUploadableProperties

    filename = os.path.basename(file_path)
    file_size = os.path.getsize(file_path)

    # Create upload session
    props = DriveItemUploadableProperties()
    props.microsoft_graph_conflict_behavior = "replace"
    props.name = filename

    request_body = CreateUploadSessionPostRequestBody()
    request_body.item = props

    upload_session = await client.me.drive.items.by_drive_item_id(folder_id).item_with_path(filename).create_upload_session.post(request_body)

    # Upload in chunks
    chunk_size = 10 * 1024 * 1024  # 10 MB

    async with aiofiles.open(file_path, 'rb') as f:
        offset = 0

        while offset < file_size:
            chunk = await f.read(chunk_size)
            chunk_len = len(chunk)

            content_range = f"bytes {offset}-{offset + chunk_len - 1}/{file_size}"

            headers = {
                'Content-Length': str(chunk_len),
                'Content-Range': content_range
            }

            async with aiohttp.ClientSession() as session:
                async with session.put(
                    upload_session.upload_url,
                    data=chunk,
                    headers=headers
                ) as response:
                    if response.status not in [200, 201, 202]:
                        raise Exception(f"Upload failed: {await response.text()}")

            offset += chunk_len
            progress = (offset / file_size) * 100
            print(f"Uploaded: {progress:.1f}%")

    print("Upload complete!")
```


### 5. Creating Folders

```python
from msgraph.generated.models.drive_item import DriveItem
from msgraph.generated.models.folder import Folder

async def create_folder(client: GraphServiceClient, folder_name: str, parent_id: str = None):
    """
    Create a new folder

    Args:
        folder_name: Name of the folder to create
        parent_id: Parent folder ID (None for root)
    """
    drive_item = DriveItem()
    drive_item.name = folder_name
    drive_item.folder = Folder()
    drive_item.microsoft_graph_conflict_behavior = "rename"  # or 'fail', 'replace'

    if parent_id:
        folder = await client.me.drive.items.by_drive_item_id(parent_id).children.post(drive_item)
    else:
        folder = await client.me.drive.root.children.post(drive_item)

    print(f"Created folder: {folder.name} (ID: {folder.id})")
    return folder

# Create folder at specific path
async def create_folder_by_path(client: GraphServiceClient, folder_path: str):
    """
    Create folder at path (e.g., '/Documents/Projects/New Folder')
    """
    drive_item = DriveItem()
    drive_item.name = os.path.basename(folder_path)
    drive_item.folder = Folder()
    drive_item.microsoft_graph_conflict_behavior = "fail"

    parent_path = os.path.dirname(folder_path)

    if parent_path and parent_path != '/':
        folder = await client.me.drive.root.item_with_path(parent_path).children.post(drive_item)
    else:
        folder = await client.me.drive.root.children.post(drive_item)

    return folder

# Create nested folder structure
async def create_nested_folders(client: GraphServiceClient, path_parts: list):
    """
    Create nested folders

    Args:
        path_parts: List of folder names ['Parent', 'Child', 'Grandchild']
    """
    parent_id = None

    for folder_name in path_parts:
        folder = await create_folder(client, folder_name, parent_id)
        parent_id = folder.id

    return parent_id
```


### 6. Searching Files

```python
from msgraph.generated.users.item.drive.root.search_with_q.search_with_q_request_builder import SearchWithQRequestBuilder

async def search_files(client: GraphServiceClient, query: str):
    """
    Search for files and folders

    Args:
        query: Search query string
    """
    result = await client.me.drive.root.search_with_q(query).get()

    if result and result.value:
        for item in result.value:
            item_type = "Folder" if item.folder else "File"
            print(f"{item_type}: {item.name} - {item.web_url}")

    return result.value

# Search in specific folder
async def search_in_folder(client: GraphServiceClient, folder_id: str, query: str):
    result = await client.me.drive.items.by_drive_item_id(folder_id).search_with_q(query).get()
    return result.value

# Search with filters
async def search_pdfs(client: GraphServiceClient):
    """Search for PDF files only"""
    result = await client.me.drive.root.search_with_q('.pdf').get()

    # Additional filtering in code
    pdf_files = [item for item in result.value if item.file and item.name.lower().endswith('.pdf')]

    return pdf_files

# Advanced search with query parameters
async def advanced_search(client: GraphServiceClient, query: str):
    query_params = SearchWithQRequestBuilder.SearchWithQRequestBuilderGetQueryParameters(
        select=['id', 'name', 'size', 'webUrl'],
        top=10
    )

    request_config = SearchWithQRequestBuilder.SearchWithQRequestBuilderGetRequestConfiguration(
        query_parameters=query_params
    )

    result = await client.me.drive.root.search_with_q(query).get(request_configuration=request_config)
    return result.value
```


### 7. Updating/Renaming Files and Folders

```python
from msgraph.generated.models.drive_item import DriveItem
from msgraph.generated.models.item_reference import ItemReference

async def rename_item(client: GraphServiceClient, item_id: str, new_name: str):
    """Rename a file or folder"""
    drive_item = DriveItem()
    drive_item.name = new_name

    updated = await client.me.drive.items.by_drive_item_id(item_id).patch(drive_item)

    print(f"Renamed to: {updated.name}")
    return updated

# Update file metadata
async def update_metadata(client: GraphServiceClient, item_id: str, description: str):
    drive_item = DriveItem()
    drive_item.description = description

    updated = await client.me.drive.items.by_drive_item_id(item_id).patch(drive_item)
    return updated

# Move file to different folder
async def move_item(client: GraphServiceClient, item_id: str, new_parent_id: str):
    """Move file or folder to new location"""
    drive_item = DriveItem()

    parent_ref = ItemReference()
    parent_ref.id = new_parent_id
    drive_item.parent_reference = parent_ref

    moved = await client.me.drive.items.by_drive_item_id(item_id).patch(drive_item)

    print(f"Moved {moved.name} to new location")
    return moved

# Move and rename simultaneously
async def move_and_rename(client: GraphServiceClient, item_id: str, new_parent_id: str, new_name: str):
    drive_item = DriveItem()
    drive_item.name = new_name

    parent_ref = ItemReference()
    parent_ref.id = new_parent_id
    drive_item.parent_reference = parent_ref

    updated = await client.me.drive.items.by_drive_item_id(item_id).patch(drive_item)
    return updated
```


### 8. Copying Files

```python
import asyncio
from msgraph.generated.drives.item.items.item.copy.copy_post_request_body import CopyPostRequestBody
from msgraph.generated.models.item_reference import ItemReference

async def copy_file(client: GraphServiceClient, item_id: str, destination_folder_id: str, new_name: str = None):
    """
    Copy file to another location

    Args:
        item_id: Source file ID
        destination_folder_id: Destination folder ID
        new_name: Optional new name for copied file
    """
    request_body = CopyPostRequestBody()

    parent_ref = ItemReference()
    parent_ref.id = destination_folder_id
    request_body.parent_reference = parent_ref

    if new_name:
        request_body.name = new_name

    # Initiate copy operation
    await client.me.drive.items.by_drive_item_id(item_id).copy.post(request_body)

    # Copy is async operation; monitor using returned location header if needed
    print(f"Copy operation initiated")

    # Wait a bit for copy to complete
    await asyncio.sleep(2)
```


### 9. Deleting Files and Folders

```python
async def delete_item(client: GraphServiceClient, item_id: str):
    """Delete a file or folder"""
    await client.me.drive.items.by_drive_item_id(item_id).delete()
    print(f"Deleted item: {item_id}")

# Delete by path
async def delete_by_path(client: GraphServiceClient, item_path: str):
    """
    Delete file or folder by path

    Args:
        item_path: Path like '/Documents/old-file.pdf'
    """
    await client.me.drive.root.item_with_path(item_path).delete()
    print(f"Deleted: {item_path}")

# Safe delete with confirmation
async def safe_delete(client: GraphServiceClient, item_id: str):
    """Delete with metadata check first"""
    # Get item info first
    item = await client.me.drive.items.by_drive_item_id(item_id).get()

    print(f"About to delete: {item.name}")
    print(f"Size: {item.size} bytes")
    print(f"Modified: {item.last_modified_date_time}")

    # Confirm and delete
    await client.me.drive.items.by_drive_item_id(item_id).delete()
    print("Deleted successfully")
```


### 10. Sharing and Permissions

#### Create Sharing Link

```python
from msgraph.generated.drives.item.items.item.create_link.create_link_post_request_body import CreateLinkPostRequestBody

async def create_share_link(client: GraphServiceClient, item_id: str, link_type: str = "view", scope: str = "anonymous"):
    """
    Create a sharing link

    Args:
        item_id: File or folder ID
        link_type: 'view', 'edit', or 'embed'
        scope: 'anonymous' or 'organization'
    """
    request_body = CreateLinkPostRequestBody()
    request_body.type = link_type
    request_body.scope = scope

    permission = await client.me.drive.items.by_drive_item_id(item_id).create_link.post(request_body)

    print(f"Share link: {permission.link.web_url}")
    return permission.link.web_url

# Create link with expiration
async def create_expiring_link(client: GraphServiceClient, item_id: str, expiration_date: str):
    """
    Create link with expiration

    Args:
        expiration_date: ISO 8601 format like '2025-12-31T23:59:59Z'
    """
    request_body = CreateLinkPostRequestBody()
    request_body.type = "view"
    request_body.scope = "anonymous"
    request_body.expiration_date_time = expiration_date

    permission = await client.me.drive.items.by_drive_item_id(item_id).create_link.post(request_body)
    return permission.link.web_url

# Create password-protected link
async def create_protected_link(client: GraphServiceClient, item_id: str, password: str):
    request_body = CreateLinkPostRequestBody()
    request_body.type = "view"
    request_body.scope = "anonymous"
    request_body.password = password

    permission = await client.me.drive.items.by_drive_item_id(item_id).create_link.post(request_body)
    return permission.link.web_url
```

#### Grant Permissions to Specific Users

```python
from msgraph.generated.drives.item.items.item.invite.invite_post_request_body import InvitePostRequestBody
from msgraph.generated.models.drive_recipient import DriveRecipient

async def invite_users(client: GraphServiceClient, item_id: str, email_addresses: list, role: str = "read"):
    """
    Invite users to access a file

    Args:
        item_id: File or folder ID
        email_addresses: List of email addresses
        role: 'read' or 'write'
    """
    request_body = InvitePostRequestBody()
    request_body.require_sign_in = True
    request_body.send_invitation = True
    request_body.roles = [role]
    request_body.message = "I've shared a file with you"

    recipients = []
    for email in email_addresses:
        recipient = DriveRecipient()
        recipient.email = email
        recipients.append(recipient)

    request_body.recipients = recipients

    permissions = await client.me.drive.items.by_drive_item_id(item_id).invite.post(request_body)

    print(f"Invited {len(email_addresses)} users")
    return permissions
```

#### List Permissions

```python
async def list_permissions(client: GraphServiceClient, item_id: str):
    """Get all permissions for an item"""
    permissions = await client.me.drive.items.by_drive_item_id(item_id).permissions.get()

    if permissions and permissions.value:
        for perm in permissions.value:
            print(f"Permission ID: {perm.id}")
            print(f"Roles: {perm.roles}")

            if perm.granted_to_v2:
                print(f"Granted to: {perm.granted_to_v2.user.display_name}")

            if perm.link:
                print(f"Link: {perm.link.web_url}")

            print("---")

    return permissions.value

# Remove permission
async def remove_permission(client: GraphServiceClient, item_id: str, permission_id: str):
    """Delete a specific permission"""
    await client.me.drive.items.by_drive_item_id(item_id).permissions.by_permission_id(permission_id).delete()
    print(f"Removed permission: {permission_id}")
```

#### List Files Shared With Me

```python
async def list_shared_with_me(client: GraphServiceClient):
    """Get files shared with the current user"""
    shared_items = await client.me.drive.shared_with_me.get()

    if shared_items and shared_items.value:
        for item in shared_items.value:
            print(f"Shared file: {item.name}")
            if item.remote_item:
                print(f"  Size: {item.remote_item.size}")
                if item.remote_item.created_by:
                    print(f"  Owner: {item.remote_item.created_by.user.display_name}")

    return shared_items.value
```


### 11. Thumbnails

```python
async def get_thumbnails(client: GraphServiceClient, item_id: str):
    """Get thumbnails for an item"""
    thumbnails = await client.me.drive.items.by_drive_item_id(item_id).thumbnails.get()

    if thumbnails and thumbnails.value:
        thumb_set = thumbnails.value[0]

        if thumb_set.small:
            print(f"Small: {thumb_set.small.url}")
        if thumb_set.medium:
            print(f"Medium: {thumb_set.medium.url}")
        if thumb_set.large:
            print(f"Large: {thumb_set.large.url}")

    return thumbnails.value

# Get specific thumbnail size
async def get_medium_thumbnail(client: GraphServiceClient, item_id: str):
    thumbnail = await client.me.drive.items.by_drive_item_id(item_id).thumbnails.by_thumbnail_set_id("0").medium.get()

    if thumbnail:
        print(f"Thumbnail URL: {thumbnail.url}")
        print(f"Size: {thumbnail.width}x{thumbnail.height}")

    return thumbnail
```


### 12. Delta (Change Tracking)

```python
async def get_initial_delta(client: GraphServiceClient):
    """Get initial delta token and all items"""
    all_items = []
    delta_link = None

    result = await client.me.drive.root.delta.get()

    while result:
        if result.value:
            all_items.extend(result.value)

        # Check for next page or delta link
        if hasattr(result, 'odata_next_link') and result.odata_next_link:
            # More pages to fetch
            result = await client.me.drive.root.delta.get()
        elif hasattr(result, 'odata_delta_link') and result.odata_delta_link:
            # Save delta link for future syncs
            delta_link = result.odata_delta_link
            break
        else:
            break

    print(f"Initial sync: {len(all_items)} items")
    print(f"Delta link: {delta_link}")

    return all_items, delta_link

# Get changes since last sync
async def get_delta_changes(client: GraphServiceClient, delta_token: str):
    """
    Get changes since last delta sync

    Args:
        delta_token: Token from previous delta sync
    """
    # Note: Use the full delta link URL saved from previous call
    # This is a simplified example
    result = await client.me.drive.root.delta.get()

    changes = []

    if result and result.value:
        for item in result.value:
            if hasattr(item, 'deleted') and item.deleted:
                print(f"Deleted: {item.id}")
                changes.append(('deleted', item))
            else:
                print(f"Added/Modified: {item.name}")
                changes.append(('modified', item))

    return changes
```


### 13. Special Folders

```python
async def get_special_folders(client: GraphServiceClient):
    """Access special OneDrive folders"""

    # Documents folder
    documents = await client.me.drive.special.by_drive_item_id("documents").get()
    print(f"Documents: {documents.name}")

    # Photos folder
    photos = await client.me.drive.special.by_drive_item_id("photos").get()
    print(f"Photos: {photos.name}")

    # Camera roll
    cameraroll = await client.me.drive.special.by_drive_item_id("cameraroll").get()
    print(f"Camera Roll: {cameraroll.name}")

    # App root folder
    approot = await client.me.drive.special.by_drive_item_id("approot").get()
    print(f"App Root: {approot.name}")

# List children of special folder
async def list_documents_folder(client: GraphServiceClient):
    files = await client.me.drive.special.by_drive_item_id("documents").children.get()

    if files and files.value:
        for file in files.value:
            print(f"{file.name}")

    return files.value
```


### 14. Working with SharePoint Document Libraries

```python
async def get_sharepoint_drive(client: GraphServiceClient, site_id: str):
    """Access SharePoint site drive"""
    drive = await client.sites.by_site_id(site_id).drive.get()

    print(f"Drive: {drive.name}")
    print(f"Type: {drive.drive_type}")

    return drive

# List SharePoint document library contents
async def list_sharepoint_files(client: GraphServiceClient, site_id: str):
    items = await client.sites.by_site_id(site_id).drive.root.children.get()

    if items and items.value:
        for item in items.value:
            print(f"{item.name}")

    return items.value

# Upload to SharePoint
async def upload_to_sharepoint(client: GraphServiceClient, site_id: str, file_path: str, upload_path: str):
    async with aiofiles.open(file_path, 'rb') as f:
        content = await f.read()

    item = await client.sites.by_site_id(site_id).drive.root.item_with_path(upload_path).content.put(content)

    return item

# Get site by URL
async def get_site_by_path(client: GraphServiceClient, hostname: str, server_relative_path: str):
    """
    Get SharePoint site by path

    Example:
        hostname: 'contoso.sharepoint.com'
        server_relative_path: '/sites/marketing'
    """
    site = await client.sites.by_site_id(f"{hostname}:{server_relative_path}").get()
    return site
```


### 15. Batch Requests

```python
from msgraph.generated.models.batch_request_content import BatchRequestContent
from msgraph.generated.models.batch_response_content import BatchResponseContent

async def batch_requests(client: GraphServiceClient):
    """Execute multiple requests in a single batch"""

    # Create batch request
    batch_request_content = BatchRequestContent()

    # Add requests to batch
    request1_id = batch_request_content.add_batch_request_step(
        {
            "id": "1",
            "method": "GET",
            "url": "/me/drive/root/children"
        }
    )

    request2_id = batch_request_content.add_batch_request_step(
        {
            "id": "2",
            "method": "GET",
            "url": "/me/drive/special/documents"
        }
    )

    # Execute batch
    batch_response = await client.batch.post(batch_request_content)

    # Process responses
    for response_id, response in batch_response.get_responses().items():
        print(f"Request {response_id}: Status {response.status}")
        print(f"Body: {response.body}")
```


## Error Handling

```python
from msgraph.generated.models.o_data_errors.o_data_error import ODataError
from azure.core.exceptions import HttpResponseError

async def safe_get_item(client: GraphServiceClient, item_id: str):
    """Example of comprehensive error handling"""
    try:
        item = await client.me.drive.items.by_drive_item_id(item_id).get()
        return item

    except ODataError as e:
        print(f"OData Error: {e.error.code if e.error else 'Unknown'}")
        print(f"Message: {e.error.message if e.error else 'Unknown'}")

        if e.response_status_code == 404:
            print("Item not found")
        elif e.response_status_code == 401:
            print("Unauthorized - check authentication")
        elif e.response_status_code == 403:
            print("Forbidden - check permissions")
        elif e.response_status_code == 429:
            print("Too many requests - rate limited")

        return None

    except HttpResponseError as e:
        print(f"HTTP Error: {e.status_code}")
        print(f"Message: {e.message}")
        return None

    except Exception as e:
        print(f"Unexpected error: {type(e).__name__}")
        print(f"Details: {str(e)}")
        return None
```


## Complete Working Examples


### Example 1: File Backup Script

```python
import asyncio
import os
from azure.identity.aio import ClientSecretCredential
from msgraph import GraphServiceClient
from dotenv import load_dotenv
import aiofiles

load_dotenv()

async def backup_onedrive_folder(folder_id: str, local_backup_path: str):
    """Backup an entire OneDrive folder to local disk"""

    # Initialize client
    credentials = ClientSecretCredential(
        tenant_id=os.getenv('AZURE_TENANT_ID'),
        client_id=os.getenv('AZURE_CLIENT_ID'),
        client_secret=os.getenv('AZURE_CLIENT_SECRET')
    )

    client = GraphServiceClient(
        credentials=credentials,
        scopes=['https://graph.microsoft.com/.default']
    )

    user_id = os.getenv('USER_ID')

    async def download_folder_recursive(folder_id: str, local_path: str):
        os.makedirs(local_path, exist_ok=True)

        items = await client.users.by_user_id(user_id).drive.items.by_drive_item_id(folder_id).children.get()

        if not items or not items.value:
            return

        for item in items.value:
            if item.folder:
                subfolder_path = os.path.join(local_path, item.name)
                print(f"Backing up folder: {item.name}")
                await download_folder_recursive(item.id, subfolder_path)

            elif item.file:
                file_path = os.path.join(local_path, item.name)
                print(f"Backing up file: {item.name} ({item.size} bytes)")

                stream = await client.users.by_user_id(user_id).drive.items.by_drive_item_id(item.id).content.get()

                async with aiofiles.open(file_path, 'wb') as f:
                    await f.write(stream)

    await download_folder_recursive(folder_id, local_backup_path)
    print("Backup complete!")

if __name__ == '__main__':
    asyncio.run(backup_onedrive_folder('FOLDER_ID_HERE', './backup'))
```


### Example 2: Sync Local Directory to OneDrive

```python
import asyncio
import os
from pathlib import Path
from azure.identity.aio import ClientSecretCredential
from msgraph import GraphServiceClient
from msgraph.generated.models.drive_item import DriveItem
from msgraph.generated.models.folder import Folder
import aiofiles

async def sync_to_onedrive(local_path: str, onedrive_folder_id: str):
    """Sync local directory to OneDrive"""

    credentials = ClientSecretCredential(
        tenant_id=os.getenv('AZURE_TENANT_ID'),
        client_id=os.getenv('AZURE_CLIENT_ID'),
        client_secret=os.getenv('AZURE_CLIENT_SECRET')
    )

    client = GraphServiceClient(
        credentials=credentials,
        scopes=['https://graph.microsoft.com/.default']
    )

    user_id = os.getenv('USER_ID')

    async def sync_folder(local_dir: str, parent_id: str):
        for entry in os.listdir(local_dir):
            entry_path = os.path.join(local_dir, entry)

            if os.path.isdir(entry_path):
                # Create folder in OneDrive
                drive_item = DriveItem()
                drive_item.name = entry
                drive_item.folder = Folder()
                drive_item.microsoft_graph_conflict_behavior = "replace"

                folder = await client.users.by_user_id(user_id).drive.items.by_drive_item_id(parent_id).children.post(drive_item)
                print(f"Created folder: {entry}")

                # Recursively sync subfolder
                await sync_folder(entry_path, folder.id)

            else:
                # Upload file
                file_size = os.path.getsize(entry_path)
                print(f"Uploading {entry} ({file_size} bytes)...")

                if file_size < 4 * 1024 * 1024:
                    # Small file
                    async with aiofiles.open(entry_path, 'rb') as f:
                        content = await f.read()

                    await client.users.by_user_id(user_id).drive.items.by_drive_item_id(parent_id).item_with_path(entry).content.put(content)
                else:
                    # Large file - would need upload session (simplified here)
                    print(f"Skipping large file: {entry}")

                print(f"Uploaded: {entry}")

    await sync_folder(local_path, onedrive_folder_id)
    print("Sync complete!")

if __name__ == '__main__':
    asyncio.run(sync_to_onedrive('./local-folder', 'ONEDRIVE_FOLDER_ID'))
```


### Example 3: Share Files with Team

```python
import asyncio
from azure.identity.aio import ClientSecretCredential
from msgraph import GraphServiceClient
from msgraph.generated.drives.item.items.item.invite.invite_post_request_body import InvitePostRequestBody
from msgraph.generated.models.drive_recipient import DriveRecipient

async def share_folder_with_team(folder_id: str, team_emails: list):
    """Share a folder with team members"""

    credentials = ClientSecretCredential(
        tenant_id=os.getenv('AZURE_TENANT_ID'),
        client_id=os.getenv('AZURE_CLIENT_ID'),
        client_secret=os.getenv('AZURE_CLIENT_SECRET')
    )

    client = GraphServiceClient(
        credentials=credentials,
        scopes=['https://graph.microsoft.com/.default']
    )

    user_id = os.getenv('USER_ID')

    # Get folder info
    folder = await client.users.by_user_id(user_id).drive.items.by_drive_item_id(folder_id).get()
    print(f"Sharing folder: {folder.name}")

    # Invite users
    request_body = InvitePostRequestBody()
    request_body.require_sign_in = True
    request_body.send_invitation = True
    request_body.roles = ["write"]
    request_body.message = f"You've been invited to collaborate on {folder.name}"

    recipients = []
    for email in team_emails:
        recipient = DriveRecipient()
        recipient.email = email
        recipients.append(recipient)

    request_body.recipients = recipients

    permissions = await client.users.by_user_id(user_id).drive.items.by_drive_item_id(folder_id).invite.post(request_body)

    print(f"Successfully shared with {len(team_emails)} team members")

    return permissions

if __name__ == '__main__':
    team = ['alice@example.com', 'bob@example.com', 'charlie@example.com']
    asyncio.run(share_folder_with_team('FOLDER_ID', team))
```


## Rate Limiting and Throttling

```python
import asyncio
from msgraph.generated.models.o_data_errors.o_data_error import ODataError

async def request_with_retry(client, request_func, max_retries=3):
    """Make request with automatic retry on rate limit"""
    retries = 0

    while retries < max_retries:
        try:
            return await request_func()

        except ODataError as e:
            if e.response_status_code == 429:
                # Rate limited
                retry_after = 5  # Default wait time

                if e.response_headers and 'Retry-After' in e.response_headers:
                    retry_after = int(e.response_headers['Retry-After'])

                print(f"Rate limited. Waiting {retry_after} seconds...")
                await asyncio.sleep(retry_after)
                retries += 1
            else:
                raise

    raise Exception("Max retries exceeded")

# Usage
async def example_with_retry(client: GraphServiceClient):
    result = await request_with_retry(
        client,
        lambda: client.me.drive.root.children.get()
    )
    return result
```


## Testing Helper Functions

```python
async def test_connection(client: GraphServiceClient):
    """Test if client is properly configured"""
    try:
        me = await client.me.get()
        print(f"Connected as: {me.display_name}")
        print(f"Email: {me.user_principal_name}")
        return True
    except Exception as e:
        print(f"Connection test failed: {e}")
        return False

async def get_drive_info(client: GraphServiceClient):
    """Get OneDrive quota and usage information"""
    drive = await client.me.drive.get()

    quota = drive.quota

    if quota:
        total = quota.total / (1024**3)  # Convert to GB
        used = quota.used / (1024**3)
        remaining = quota.remaining / (1024**3)

        print(f"OneDrive Storage:")
        print(f"  Total: {total:.2f} GB")
        print(f"  Used: {used:.2f} GB")
        print(f"  Remaining: {remaining:.2f} GB")
        print(f"  State: {quota.state}")

    return quota
```

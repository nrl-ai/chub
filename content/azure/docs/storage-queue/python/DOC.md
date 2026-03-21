---
name: storage-queue
description: "Azure Queue Storage SDK for Python with QueueServiceClient, QueueClient, and async support"
metadata:
  languages: "python"
  versions: "12.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-storage,queue,storage,python"
---

# azure-storage-queue Python Package Guide

## Golden Rule

Use `azure-storage-queue` for Azure Queue Storage, and write against the modern 12.x client surface:

- `QueueServiceClient` for account-level operations
- `QueueClient` for a specific queue
- `azure.storage.queue.aio` for async code

Do not mix this with legacy `QueueService` examples from old Azure Storage SDK posts or archived code.

## Install

PyPI for `12.15.0` lists `Requires: Python >=3.9`.

```bash
python -m venv .venv
source .venv/bin/activate
pip install "azure-storage-queue==12.15.0"
```

If you want Microsoft Entra ID authentication, install `azure-identity` too:

```bash
pip install "azure-storage-queue==12.15.0" azure-identity
```

Common dependency choices:

- `azure-identity` for `DefaultAzureCredential`
- `python-dotenv` if you load local env files yourself

## Setup And Authentication

The SDK supports the three practical auth patterns most agents need:

1. `DefaultAzureCredential` with an account URL
2. Connection string for local development or simple service wiring
3. Shared key or SAS when you already have those credentials

### Option 1: Microsoft Entra ID with `DefaultAzureCredential`

Use this for deployed apps running in Azure or local dev with Azure CLI / developer login already configured.

```bash
export AZURE_STORAGE_QUEUE_ACCOUNT_URL="https://<account>.queue.core.windows.net"
```

```python
import os

from azure.identity import DefaultAzureCredential
from azure.storage.queue import QueueServiceClient

service = QueueServiceClient(
    account_url=os.environ["AZURE_STORAGE_QUEUE_ACCOUNT_URL"],
    credential=DefaultAzureCredential(),
)
```

The official quickstart uses this flow and requires a data-plane role such as `Storage Queue Data Contributor`.

### Option 2: Connection String

Use this for local tools, scripts, or when the environment already gives you a storage connection string.

```bash
export AZURE_STORAGE_CONNECTION_STRING="DefaultEndpointsProtocol=https;AccountName=...;AccountKey=...;EndpointSuffix=core.windows.net"
export AZURE_STORAGE_QUEUE_NAME="jobs"
```

```python
import os

from azure.storage.queue import QueueClient

queue = QueueClient.from_connection_string(
    conn_str=os.environ["AZURE_STORAGE_CONNECTION_STRING"],
    queue_name=os.environ["AZURE_STORAGE_QUEUE_NAME"],
)
```

For Azurite, the official readme documents `UseDevelopmentStorage=true;` support in this package line.

### Option 3: Shared Key

Use this when you explicitly manage account keys.

```python
from azure.core.credentials import AzureNamedKeyCredential
from azure.storage.queue import QueueServiceClient

account_name = "myaccount"
credential = AzureNamedKeyCredential(account_name, "<account-key>")

service = QueueServiceClient(
    account_url=f"https://{account_name}.queue.core.windows.net",
    credential=credential,
)
```

## Initialize Clients

Use `QueueServiceClient` once, then derive queue-specific clients from it.

```python
from azure.storage.queue import QueueServiceClient

service = QueueServiceClient.from_connection_string(
    conn_str="DefaultEndpointsProtocol=https;AccountName=...;AccountKey=...;EndpointSuffix=core.windows.net"
)

queue = service.get_queue_client("jobs")
```

This is the cleanest pattern when the code touches multiple queues.

If the code only needs one queue, constructing `QueueClient` directly is fine.

## Core Usage

### Create A Queue If Needed

```python
from azure.core.exceptions import ResourceExistsError

try:
    queue.create_queue()
except ResourceExistsError:
    pass
```

### Send Messages

```python
queue.send_message("rebuild-search-index")
queue.send_message('{"job":"sync-users","tenant":"acme"}')
```

Use application-level JSON serialization for structured payloads.

### Receive, Process, And Delete Messages

```python
messages = queue.receive_messages(messages_per_page=10)

for message_batch in messages.by_page():
    for message in message_batch:
        payload = message.content
        print(payload)

        # Delete only after work succeeds.
        queue.delete_message(message)
```

Operationally important:

- Receiving a message does not remove it permanently.
- The message stays leased for its visibility timeout, then can reappear if you do not delete it.
- Delete only after the handler finishes successfully.

### Peek Without Taking A Processing Lease

```python
for message in queue.peek_messages(max_messages=5):
    print(message.content)
```

Use `peek_messages` for inspection or diagnostics when you do not want to affect consumer flow.

### Update A Message In Place

```python
messages = queue.receive_messages()
message = next(messages)

queue.update_message(
    message=message,
    content='{"status":"retrying"}',
    visibility_timeout=30,
)
```

Use this when the worker needs to extend visibility or change the payload before reprocessing.

## Async Usage

The package includes async clients under `azure.storage.queue.aio`.

```python
import os
import asyncio

from azure.storage.queue.aio import QueueClient

async def main() -> None:
    queue = QueueClient.from_connection_string(
        conn_str=os.environ["AZURE_STORAGE_CONNECTION_STRING"],
        queue_name="jobs",
    )

    async with queue:
        await queue.send_message("async-job")

asyncio.run(main())
```

Use the async surface when queue I/O is part of a larger async application. Do not mix sync clients into an async request path unless you intentionally want blocking I/O.

## Configuration And Diagnostics

### Logging

The official package readme enables HTTP logging through the Azure SDK logger:

```python
import sys
import logging

logger = logging.getLogger("azure")
logger.setLevel(logging.DEBUG)
handler = logging.StreamHandler(stream=sys.stdout)
logger.addHandler(handler)
```

Turn this on when debugging auth failures, retry behavior, or request signing issues.

### Message Encoding

Azure Queue Storage messages are XML-backed text payloads. If you need to send binary data or content that should be base64-encoded, set explicit encode/decode policies instead of assuming the SDK will do it automatically.

```python
from azure.storage.queue import (
    BinaryBase64EncodePolicy,
    BinaryBase64DecodePolicy,
    QueueClient,
)

queue = QueueClient.from_connection_string(
    conn_str="UseDevelopmentStorage=true;",
    queue_name="jobs",
    message_encode_policy=BinaryBase64EncodePolicy(),
    message_decode_policy=BinaryBase64DecodePolicy(),
)
```

### Local Development

For Azurite, the documented shortcut is:

```text
UseDevelopmentStorage=true;
```

That is safer than hand-assembling a fake endpoint string for local development.

## Common Pitfalls

### Confusing 12.x With The Legacy `QueueService` SDK

Current official docs and PyPI pages are for the modern client library. If the code imports `QueueService`, you are in the old SDK generation and this doc does not apply.

### Forgetting `azure-identity`

`DefaultAzureCredential` examples require the separate `azure-identity` package. Installing only `azure-storage-queue` is not enough for credential-chain auth.

### Receiving Without Deleting

`receive_messages()` leases messages. If the worker never deletes them, they can become visible again and be processed twice.

### Assuming Binary Payloads Just Work

If you pass bytes or non-XML-safe content, configure message encode/decode policies explicitly. Do not assume transparent base64 behavior.

### Queue Name Rules

Queue names are not arbitrary labels. Follow Azure Queue Storage naming rules from the service docs and keep names stable across producers and consumers.

## Version-Sensitive Notes

- PyPI shows `12.15.0` as the covered package version, released on `2026-01-07`.
- The Microsoft Learn package readme for `azure-storage-queue` also targets `12.15.0`.
- The official package page says `Requires: Python >=3.9`.
- The older Azure Queue quickstart page still says Python `3.8` or later. Treat the package metadata as authoritative for environment pinning.
- The official docs now include `UseDevelopmentStorage=true;` support for Azurite in this package line.

When copying examples from blogs or old Azure answers, check whether they use `QueueService` or `QueueClient`. That is the fastest way to detect whether the example belongs to the wrong SDK generation.

## Official Sources

- Microsoft Learn package index: `https://learn.microsoft.com/en-us/python/api/azure-storage-queue/`
- Microsoft Learn package readme: `https://learn.microsoft.com/en-us/python/api/overview/azure/storage-queue-readme?view=azure-python`
- Microsoft Learn `QueueClient` reference: `https://learn.microsoft.com/en-us/python/api/azure-storage-queue/azure.storage.queue.queueclient?view=azure-python`
- Microsoft Learn Azure Queue quickstart for Python: `https://learn.microsoft.com/en-us/azure/storage/queues/storage-quickstart-queues-python`
- PyPI project page: `https://pypi.org/project/azure-storage-queue/`

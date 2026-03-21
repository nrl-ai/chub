---
name: eventhub
description: "azure-eventhub package guide for Python 5.15.1 with install, auth, producer/consumer setup, checkpointing, and transport notes"
metadata:
  languages: "python"
  versions: "5.15.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,eventhubs,messaging,streaming,amqp"
---

# azure-eventhub Python Package Guide

## What It Is

`azure-eventhub` is the Azure SDK for sending events to, and receiving events from, Azure Event Hubs from Python.

This entry is for package version `5.15.1`. The official Microsoft Learn API and overview pages match the modern track-2 client surface built around `EventHubProducerClient`, `EventHubConsumerClient`, `EventData`, and `azure.eventhub.aio`.

## Install

Base package:

```bash
python -m pip install "azure-eventhub==5.15.1"
```

Passwordless Microsoft Entra authentication:

```bash
python -m pip install "azure-eventhub==5.15.1" azure-identity
```

Blob-backed checkpointing:

```bash
python -m pip install "azure-eventhub==5.15.1" azure-eventhub-checkpointstoreblob
```

Async consumers with blob-backed checkpointing:

```bash
python -m pip install "azure-eventhub==5.15.1" azure-eventhub-checkpointstoreblob-aio azure-identity aiohttp
```

## Golden Rules

- Prefer the `5.x` client types: `EventHubProducerClient` and `EventHubConsumerClient`.
- If you use a namespace connection string, pass `eventhub_name` unless the connection string already includes `EntityPath=...`.
- Set `starting_position` deliberately for consumers. The documented default is `@latest`, which only receives newly enqueued events.
- Use a checkpoint store for long-running consumers that need durable progress and partition load balancing.
- Do not share one client instance across threads or across concurrent coroutines; the SDK documents clients as neither thread-safe nor coroutine-safe.

## Authentication And Setup

### Connection string

This is the simplest setup path and works well for local development and service principals using SAS policies.

```bash
export EVENT_HUB_CONNECTION_STR="Endpoint=sb://<namespace>.servicebus.windows.net/;SharedAccessKeyName=<policy>;SharedAccessKey=<key>"
export EVENT_HUB_NAME="<event-hub-name>"
```

If the connection string already includes `EntityPath=<event-hub-name>`, you can usually omit `EVENT_HUB_NAME`.

### Microsoft Entra credentials

For production Azure deployments, prefer `DefaultAzureCredential()` with RBAC.

Minimum roles depend on the operation:

- Sending: `Azure Event Hubs Data Sender`
- Receiving: `Azure Event Hubs Data Receiver`
- Full access: `Azure Event Hubs Data Owner`
- Blob checkpointing: `Storage Blob Data Contributor` on the storage account or container

```python
import os

from azure.eventhub import EventHubProducerClient
from azure.identity import DefaultAzureCredential

producer = EventHubProducerClient(
    fully_qualified_namespace=os.environ["EVENT_HUB_HOSTNAME"],
    eventhub_name=os.environ["EVENT_HUB_NAME"],
    credential=DefaultAzureCredential(),
)
```

## Core Usage

### Send events

```python
import os

from azure.eventhub import EventData, EventHubProducerClient

producer = EventHubProducerClient.from_connection_string(
    conn_str=os.environ["EVENT_HUB_CONNECTION_STR"],
    eventhub_name=os.environ.get("EVENT_HUB_NAME"),
)

try:
    batch = producer.create_batch()
    batch.add(EventData("first event"))
    batch.add(EventData("second event"))
    producer.send_batch(batch)
finally:
    producer.close()
```

Use `create_batch()` instead of guessing the AMQP frame size yourself. If `batch.add(...)` fails, start a new batch or reduce payload size.

### Receive events

```python
import os

from azure.eventhub import EventHubConsumerClient

def on_event(partition_context, event):
    print(
        partition_context.partition_id,
        event.sequence_number,
        event.body_as_str(encoding="UTF-8"),
    )

consumer = EventHubConsumerClient.from_connection_string(
    conn_str=os.environ["EVENT_HUB_CONNECTION_STR"],
    consumer_group="$Default",
    eventhub_name=os.environ.get("EVENT_HUB_NAME"),
)

try:
    consumer.receive(
        on_event=on_event,
        starting_position="-1",  # "-1" reads from the earliest available event.
    )
finally:
    consumer.close()
```

Use `starting_position="-1"` for backlog replay. Use `starting_position="@latest"` only when you explicitly want new events only.

### Async clients

Use the `.aio` namespace for async code.

```python
import os

from azure.eventhub.aio import EventHubProducerClient

async def make_producer():
    producer = EventHubProducerClient.from_connection_string(
        conn_str=os.environ["EVENT_HUB_CONNECTION_STR"],
        eventhub_name=os.environ.get("EVENT_HUB_NAME"),
    )
    return producer
```

For passwordless async code, use `azure.identity.aio.DefaultAzureCredential`.

## Checkpointing And Load Balancing

For multi-partition or long-running consumers, add a blob checkpoint store instead of keeping offsets only in memory.

```python
import os

from azure.eventhub import EventHubConsumerClient
from azure.eventhub.extensions.checkpointstoreblob import BlobCheckpointStore

checkpoint_store = BlobCheckpointStore.from_connection_string(
    conn_str=os.environ["BLOB_STORAGE_CONNECTION_STRING"],
    container_name=os.environ["BLOB_CONTAINER_NAME"],
)

consumer = EventHubConsumerClient.from_connection_string(
    conn_str=os.environ["EVENT_HUB_CONNECTION_STR"],
    consumer_group="$Default",
    eventhub_name=os.environ.get("EVENT_HUB_NAME"),
    checkpoint_store=checkpoint_store,
)
```

Operational notes from the official docs:

- `BlobCheckpointStore` is intended for `EventHubConsumerClient`, not as a general-purpose blob wrapper.
- Keep one blob container per consumer group.
- Do not use a hierarchical-namespace-enabled storage account for checkpointing.
- Disable blob versioning, soft delete, and point-in-time restore on the checkpointing account.

If you skip `checkpoint_store`, the consumer can still receive events, but durable checkpoints and cooperative partition load balancing are not available.

## Configuration Notes

### Transport selection

The SDK supports:

- `TransportType.Amqp` on port `5671`
- `TransportType.AmqpOverWebsocket` on port `443`

If firewalls or proxies block `5671`, switch to websockets:

```python
import os

from azure.eventhub import EventHubProducerClient, TransportType

producer = EventHubProducerClient.from_connection_string(
    conn_str=os.environ["EVENT_HUB_CONNECTION_STR"],
    eventhub_name=os.environ.get("EVENT_HUB_NAME"),
    transport_type=TransportType.AmqpOverWebsocket,
)
```

### Consumer coordination and diagnostics

- Use `owner_level` when you need an exclusive reader for a partition.
- Set `track_last_enqueued_event_properties=True` if you need partition lag metadata on received events.
- Use `logging_enable=True` and the `azure.eventhub` logger when troubleshooting connection or auth issues.
- Tune `prefetch` if a consumer is buffering too much data in memory for your workload.

## Version-Sensitive Notes

- `5.15.1` is the current modern client line. Do not paste legacy `EventHubClient` examples from pre-5.x material into this version.
- The package now uses a pure-Python AMQP stack by default. `uamqp` is optional and only used when installed and explicitly requested with `uamqp_transport=True`.
- The current docs and samples are split between sync imports under `azure.eventhub` and async imports under `azure.eventhub.aio`.

## Common Pitfalls

- Forgetting `eventhub_name` when using a namespace-scoped connection string without `EntityPath`.
- Receiving no backlog because `starting_position` was left at the default `@latest`.
- Reusing one client object across threads or concurrent coroutines.
- Running multiple consumers against the same partitions and consumer group without understanding duplicate-processing behavior.
- Using blob checkpointing on a storage account with hierarchical namespace, soft delete, or versioning still enabled.
- Debugging network connectivity without trying `TransportType.AmqpOverWebsocket`.

## Official Sources Used

- PyPI package page: `https://pypi.org/project/azure-eventhub/5.15.1/`
- Microsoft Learn API index: `https://learn.microsoft.com/en-us/python/api/azure-eventhub/?view=azure-python`
- Microsoft Learn overview/readme: `https://learn.microsoft.com/en-us/python/api/overview/azure/eventhub-readme?view=azure-python`
- Microsoft Learn producer client reference: `https://learn.microsoft.com/en-us/python/api/azure-eventhub/azure.eventhub.eventhubproducerclient?view=azure-python`
- Microsoft Learn consumer client reference: `https://learn.microsoft.com/en-us/python/api/azure-eventhub/azure.eventhub.eventhubconsumerclient?view=azure-python`
- Microsoft Learn transport type reference: `https://learn.microsoft.com/en-us/python/api/azure-eventhub/azure.eventhub.transporttype?view=azure-python`
- Microsoft Learn checkpoint store reference: `https://learn.microsoft.com/en-us/python/api/azure-eventhub-checkpointstoreblob/azure.eventhub.extensions.checkpointstoreblob.blobcheckpointstore?view=azure-python`
- Microsoft Learn Event Hubs Python quickstart: `https://learn.microsoft.com/en-us/azure/event-hubs/event-hubs-python-get-started-send`

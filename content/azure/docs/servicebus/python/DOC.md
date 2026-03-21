---
name: servicebus
description: "Azure Service Bus Python client library for queues, topics, subscriptions, sessions, and message settlement"
metadata:
  languages: "python"
  versions: "7.14.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,service-bus,messaging,amqp,queues,topics,subscriptions,sessions"
---

# Azure Service Bus Python Client Library

## Golden Rule

Use `azure-servicebus` with `ServiceBusClient` or `azure.servicebus.aio`, prefer `DefaultAzureCredential` for production auth, and do not copy pre-7.x examples that use `QueueClient`, `TopicClient`, or the old `Message` API. The current SDK line is `7.14.3`, and PyPI now requires Python `>=3.9`.

## Install

Pin the version your project expects:

```bash
python -m pip install "azure-servicebus==7.14.3"
```

Common companions:

```bash
python -m pip install "azure-identity>=1.17.0"
python -m pip install "aiohttp>=3.9.0"
```

Notes:

- Install `azure-identity` when you want Microsoft Entra ID auth via `DefaultAzureCredential`.
- Install `aiohttp` for the async client path under `azure.servicebus.aio`.
- Do not build new code around `uamqp_transport`; the Azure SDK changelog marks it deprecated in `7.14.2`.

## Authentication And Setup

The SDK supports both connection strings and token credentials.

Recommended environment variables:

```bash
export SERVICEBUS_CONNECTION_STRING="Endpoint=sb://<namespace>.servicebus.windows.net/;SharedAccessKeyName=..."
export SERVICEBUS_FULLY_QUALIFIED_NAMESPACE="<namespace>.servicebus.windows.net"
export SERVICEBUS_QUEUE_NAME="orders"
export SERVICEBUS_TOPIC_NAME="events"
export SERVICEBUS_SUBSCRIPTION_NAME="worker-a"
```

### Preferred: `DefaultAzureCredential`

Use passwordless auth in production, CI, and Azure-hosted workloads:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.servicebus import ServiceBusClient

credential = DefaultAzureCredential()

client = ServiceBusClient(
    fully_qualified_namespace=os.environ["SERVICEBUS_FULLY_QUALIFIED_NAMESPACE"],
    credential=credential,
)
```

The identity needs a Service Bus data-plane role such as `Azure Service Bus Data Owner`, `Azure Service Bus Data Sender`, or `Azure Service Bus Data Receiver` depending on what the code does.

### Connection string fallback

Connection strings are still the simplest option for local scripts and quick tests:

```python
import os

from azure.servicebus import ServiceBusClient

client = ServiceBusClient.from_connection_string(
    conn_str=os.environ["SERVICEBUS_CONNECTION_STRING"]
)
```

## Core Usage

### Send to a queue

Batch sends are the safest default when you may emit more than one message:

```python
import os

from azure.servicebus import ServiceBusClient, ServiceBusMessage

queue_name = os.environ["SERVICEBUS_QUEUE_NAME"]

with ServiceBusClient.from_connection_string(
    os.environ["SERVICEBUS_CONNECTION_STRING"]
) as client:
    with client.get_queue_sender(queue_name=queue_name) as sender:
        batch = sender.create_message_batch()
        has_messages = False

        for body in ("order-1001", "order-1002", "order-1003"):
            try:
                batch.add_message(ServiceBusMessage(body))
                has_messages = True
            except ValueError:
                sender.send_messages(batch)
                batch = sender.create_message_batch()
                batch.add_message(ServiceBusMessage(body))
                has_messages = True

        if has_messages:
            sender.send_messages(batch)
```

`add_message()` raises `ValueError` when the next message would overflow the current AMQP batch.

### Receive and settle from a queue

The default receive mode is `PEEK_LOCK`, which is what you usually want for workers:

```python
import os

from azure.servicebus import ServiceBusClient

queue_name = os.environ["SERVICEBUS_QUEUE_NAME"]

with ServiceBusClient.from_connection_string(
    os.environ["SERVICEBUS_CONNECTION_STRING"]
) as client:
    with client.get_queue_receiver(queue_name=queue_name, max_wait_time=5) as receiver:
        for message in receiver.receive_messages(max_message_count=10, max_wait_time=5):
            try:
                print(message.message_id, message.delivery_count)
                receiver.complete_message(message)
            except Exception:
                receiver.abandon_message(message)
                raise
```

Settlement methods you will use most often:

- `complete_message(message)`: remove the message after successful processing
- `abandon_message(message)`: unlock it for another delivery attempt
- `dead_letter_message(message, reason=..., error_description=...)`: move it to the dead-letter subqueue
- `defer_message(message)`: keep it for later retrieval by sequence number

### Topic and subscription flow

```python
import os

from azure.servicebus import ServiceBusClient, ServiceBusMessage

topic_name = os.environ["SERVICEBUS_TOPIC_NAME"]
subscription_name = os.environ["SERVICEBUS_SUBSCRIPTION_NAME"]

with ServiceBusClient.from_connection_string(
    os.environ["SERVICEBUS_CONNECTION_STRING"]
) as client:
    with client.get_topic_sender(topic_name=topic_name) as sender:
        sender.send_messages(ServiceBusMessage("inventory.updated"))

    with client.get_subscription_receiver(
        topic_name=topic_name,
        subscription_name=subscription_name,
        max_wait_time=5,
    ) as receiver:
        messages = receiver.receive_messages(max_message_count=10, max_wait_time=5)
        for message in messages:
            receiver.complete_message(message)
```

### Dead-letter reads

Use the dead-letter subqueue explicitly when triaging failures:

```python
import os

from azure.servicebus import ServiceBusSubQueue
from azure.servicebus import ServiceBusClient

with ServiceBusClient.from_connection_string(
    os.environ["SERVICEBUS_CONNECTION_STRING"]
) as client:
    with client.get_queue_receiver(
        queue_name=os.environ["SERVICEBUS_QUEUE_NAME"],
        sub_queue=ServiceBusSubQueue.DEAD_LETTER,
    ) as receiver:
        messages = receiver.receive_messages(max_message_count=10, max_wait_time=5)
        for message in messages:
            print(message.message_id)
```

### Sessions

If the entity requires sessions, each sent message needs a `session_id`, and receivers connect to a specific session or ask for the next available one:

```python
import os

from azure.servicebus import ServiceBusClient, ServiceBusMessage

queue_name = os.environ["SERVICEBUS_QUEUE_NAME"]

with ServiceBusClient.from_connection_string(
    os.environ["SERVICEBUS_CONNECTION_STRING"]
) as client:
    with client.get_queue_sender(queue_name=queue_name) as sender:
        sender.send_messages(
            ServiceBusMessage("step-1", session_id="customer-42")
        )

    with client.get_queue_receiver(
        queue_name=queue_name,
        session_id="customer-42",
        max_wait_time=5,
    ) as receiver:
        messages = receiver.receive_messages(max_message_count=1, max_wait_time=5)
        for message in messages:
            receiver.complete_message(message)
```

Session-enabled entities use a session lock rather than separate per-message locks. Renew the session lock for long-running work instead of assuming the default lock duration is enough.

### Async API

Use `azure.servicebus.aio` for async senders and receivers, and close both the Service Bus client and Azure Identity credential:

```python
import os

from azure.identity.aio import DefaultAzureCredential
from azure.servicebus import ServiceBusMessage
from azure.servicebus.aio import ServiceBusClient

async def main() -> None:
    credential = DefaultAzureCredential()
    queue_name = os.environ["SERVICEBUS_QUEUE_NAME"]

    try:
        async with ServiceBusClient(
            fully_qualified_namespace=os.environ["SERVICEBUS_FULLY_QUALIFIED_NAMESPACE"],
            credential=credential,
        ) as client:
            sender = client.get_queue_sender(queue_name=queue_name)
            async with sender:
                await sender.send_messages(ServiceBusMessage("hello from aio"))
    finally:
        await credential.close()
```

## Administration

Use `ServiceBusAdministrationClient` for entity administration inside an existing namespace:

```python
import os

from azure.servicebus.management import ServiceBusAdministrationClient

admin = ServiceBusAdministrationClient.from_connection_string(
    os.environ["SERVICEBUS_CONNECTION_STRING"]
)

queue = admin.get_queue(queue_name=os.environ["SERVICEBUS_QUEUE_NAME"])
print(queue.max_delivery_count)
```

Use Azure Resource Manager tools such as `azure-mgmt-servicebus`, the Azure CLI, or the portal when you need namespace-level provisioning rather than queue or subscription administration inside a namespace.

## Configuration Notes

- The namespace must be the fully qualified host name, for example `my-namespace.servicebus.windows.net`.
- If outbound AMQP on port `5671` is blocked, set `transport_type=TransportType.AmqpOverWebsocket` and use WebSockets over `443`.
- `prefetch_count` can improve throughput, but it increases the number of messages held client-side under lock.
- Long-running handlers should use `AutoLockRenewer` or explicit lock-renewal methods so locks do not expire before settlement.
- `client_identifier` is useful when you need clearer broker-side diagnostics in Service Bus errors.
- Retry and socket behavior are configurable on the client, sender, and receiver; tune them when running through proxies, high-latency links, or flaky networks.

## Common Pitfalls

- Do not copy old `azure-servicebus` `0.50.x` or `1.0.0` samples. Modern code uses `ServiceBusClient`, entity-specific senders and receivers, and explicit settlement methods.
- The clients are not thread-safe or coroutine-safe. Do not share a sender, receiver, or client instance across concurrent threads or tasks without your own synchronization.
- `RECEIVE_AND_DELETE` is lossy if the process crashes after the broker hands out a message. Use `PEEK_LOCK` for normal workers.
- The docs warn that combining `RECEIVE_AND_DELETE` with prefetch can lose prefetched messages if the process exits before processing them.
- Messages and session locks expire. If processing can run longer than the lock duration, renew the lock or use `AutoLockRenewer`.
- `peek_messages()` does not lock messages and those peeked messages cannot be settled.
- Async code needs `azure.servicebus.aio` plus an async transport like `aiohttp`; remember to close the credential.
- Service Bus emulator support does not cover `ServiceBusAdministrationClient`.

## Version-Sensitive Notes For 7.14.x

- PyPI shows `7.14.3` as the latest published package on March 12, 2026. The Azure SDK repo changelog on the `main` branch still labels `7.14.3` as unreleased, so prefer PyPI and Learn when you need release-state confirmation.
- `7.14.2` removed Python `3.8` support and deprecated `uamqp_transport`. Keep new code on Python `3.9+` and avoid transport settings that depend on the deprecated path.
- `7.14.0` added Azure Service Bus emulator support, but the changelog notes that `ServiceBusAdministrationClient` is not supported against the emulator.
- The migration guide confirms the 7.x API shift away from legacy client types and settlement patterns. When updating older code, map those concepts explicitly instead of trying to port line-for-line.

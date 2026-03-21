---
name: pubsub
description: "Google Cloud Pub/Sub Python client library for publishing, subscribing, and local emulator workflows"
metadata:
  languages: "python"
  versions: "2.35.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,gcp,pubsub,messaging,events,queues"
---

# Google Cloud Pub/Sub Python Client

## Golden Rule

Use `google-cloud-pubsub` for Pub/Sub work in Python, authenticate with Application Default Credentials (ADC), and write subscriber handlers as idempotent code. Pub/Sub delivery is at-least-once by default, so duplicate deliveries are normal unless you explicitly enable exactly-once delivery on the subscription.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-pubsub==2.35.0"
```

If you need the migration helper for older pre-2.0 code:

```bash
python -m pip install "google-cloud-pubsub[libcst]==2.35.0"
```

## Auth And Setup

Pub/Sub client libraries use ADC.

Local development with user credentials:

```bash
gcloud init
gcloud auth application-default login
gcloud auth application-default set-quota-project YOUR_PROJECT_ID
```

Service account JSON:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

Minimum Google Cloud setup:

1. Create or choose a project.
2. Enable the Pub/Sub API.
3. Authenticate with ADC.
4. Grant the runtime identity Pub/Sub permissions for the topics and subscriptions it uses.

## Initialize Clients

```python
import os
from google.cloud import pubsub_v1

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]

publisher = pubsub_v1.PublisherClient()
subscriber = pubsub_v1.SubscriberClient()

topic_path = publisher.topic_path(project_id, "orders")
subscription_path = subscriber.subscription_path(project_id, "orders-worker")
```

Use `PublisherClient` for topic and publish operations, and `SubscriberClient` for subscription and pull operations.

## Create Topics And Subscriptions

Most generated API methods in the 2.x client accept either a `request={...}` object or flattened keyword arguments. Do not mix both styles in the same call.

```python
from google.api_core.exceptions import AlreadyExists
from google.cloud import pubsub_v1

publisher = pubsub_v1.PublisherClient()
subscriber = pubsub_v1.SubscriberClient()

topic_path = publisher.topic_path("my-project", "orders")
subscription_path = subscriber.subscription_path("my-project", "orders-worker")

try:
    publisher.create_topic(request={"name": topic_path})
except AlreadyExists:
    pass

try:
    subscriber.create_subscription(
        request={
            "name": subscription_path,
            "topic": topic_path,
        }
    )
except AlreadyExists:
    pass
```

## Publish Messages

`publish()` is one of the handwritten methods whose convenient signature stayed stable across the 2.x line. It takes a topic path, raw bytes payload, and optional string attributes.

```python
import json
from google.cloud import pubsub_v1

publisher = pubsub_v1.PublisherClient(
    batch_settings=pubsub_v1.types.BatchSettings(
        max_bytes=1_000_000,
        max_latency=0.01,
        max_messages=100,
    )
)

topic_path = publisher.topic_path("my-project", "orders")

payload = json.dumps({"order_id": "123", "status": "created"}).encode("utf-8")
future = publisher.publish(
    topic_path,
    payload,
    event_type="order.created",
    source="checkout",
)

message_id = future.result()
print(message_id)

# Flush outstanding publishes before process shutdown.
publisher.stop()
```

Notes:

- `data` must be `bytes`, not `str`.
- Message attributes are strings.
- Default batch settings are `max_bytes=1_000_000`, `max_latency=0.01`, and `max_messages=100`.
- If you enable ordered delivery, publish with `ordering_key=...` and call `resume_publish(topic_path, ordering_key)` after a recoverable publish failure for that key.

## Subscribe With A Streaming Callback

Use `subscribe()` for the common async consumer pattern:

```python
from concurrent.futures import TimeoutError
from google.cloud import pubsub_v1

subscriber = pubsub_v1.SubscriberClient()
subscription_path = subscriber.subscription_path("my-project", "orders-worker")

flow_control = pubsub_v1.types.FlowControl(
    max_messages=100,
    max_bytes=10 * 1024 * 1024,
)

def callback(message: pubsub_v1.subscriber.message.Message) -> None:
    try:
        body = message.data.decode("utf-8")
        print(body, dict(message.attributes), message.delivery_attempt)
        message.ack()
    except Exception:
        message.nack()

streaming_pull_future = subscriber.subscribe(
    subscription_path,
    callback=callback,
    flow_control=flow_control,
    await_callbacks_on_shutdown=True,
)

with subscriber:
    try:
        streaming_pull_future.result(timeout=30)
    except TimeoutError:
        streaming_pull_future.cancel()
        streaming_pull_future.result()
```

Notes:

- Default subscriber flow control is `max_messages=1000`, `max_bytes=100 MiB`, `max_lease_duration=3600`.
- `message.data` is always `bytes`; decode it yourself.
- `message.delivery_attempt` is only populated when the subscription has a dead-letter policy.
- Use `ack()` only after durable processing succeeds.
- Use `nack()` for retryable failures.

## Exactly-Once Delivery

Default Pub/Sub acknowledgements are best effort. The client docs explicitly warn that your handler should remain idempotent because messages may be delivered more than once.

If the subscription has exactly-once delivery enabled, use `ack_with_response()` when you need acknowledgement success tracking:

```python
ack_future = message.ack_with_response()
ack_future.result()
```

Without exactly-once delivery, `ack_with_response()` still returns success immediately, but re-delivery is still possible.

## Emulator For Local Tests

Start the local Pub/Sub emulator through the Google Cloud CLI:

```bash
gcloud beta emulators pubsub start
$(gcloud beta emulators pubsub env-init)
export PUBSUB_PROJECT_ID="local-project"
```

The Python client automatically uses `PUBSUB_EMULATOR_HOST` when it is set.

Example against the emulator:

```python
from google.cloud import pubsub_v1

project_id = "local-project"
publisher = pubsub_v1.PublisherClient()
subscriber = pubsub_v1.SubscriberClient()

topic_path = publisher.topic_path(project_id, "demo")
subscription_path = subscriber.subscription_path(project_id, "demo-sub")

publisher.create_topic(request={"name": topic_path})
subscriber.create_subscription(request={"name": subscription_path, "topic": topic_path})
publisher.publish(topic_path, b"hello emulator").result()
```

## Common Pitfalls

- Do not pass a text string to `publish()`. Encode to bytes first.
- Do not mix `request={...}` with flattened keyword arguments on the same generated API call.
- Do not create `SubscriberClient` path helpers with the publisher client. In the 2.x line, `subscription_path()` lives on `SubscriberClient`.
- Do not assume a successful `ack()` means the message can never be re-delivered. Keep handlers idempotent.
- Do not let the process exit with pending publish futures. Wait on futures and call `publisher.stop()`.
- Do not share client instances across forked processes. Create clients after `os.fork()` or inside each worker process.
- Do not rely on emulator behavior for production semantics. Google documents that the emulator may be incomplete or differ from the real service.

## Version-Sensitive Notes

- `2.35.0` is the latest PyPI release as of March 12, 2026.
- PyPI requires Python `>=3.9`. The archived Google repository notes that `2.34.0` was the last release supporting Python 3.7 and 3.8.
- The official 2.0 migration guide still matters when reading older examples: most generated client methods moved to request-object style, while `publish()` and `subscribe()` largely kept their higher-level signatures.
- In the 2.x API, `request` and flattened keyword arguments are mutually exclusive on generated methods.
- Google archived the standalone `googleapis/python-pubsub` repository in March 2026 and points maintainers to the `google-cloud-python` monorepo. Prefer the Google Cloud docs site and PyPI metadata over old repository examples when they conflict.

## Official Sources

- Client library reference: `https://cloud.google.com/python/docs/reference/pubsub/latest`
- Publisher client reference: `https://cloud.google.com/python/docs/reference/pubsub/latest/google.cloud.pubsub_v1.publisher.client.Client`
- Subscriber client reference: `https://cloud.google.com/python/docs/reference/pubsub/latest/google.cloud.pubsub_v1.subscriber.client.Client`
- Migration guide: `https://cloud.google.com/python/docs/reference/pubsub/latest/upgrading`
- Multiprocessing note: `https://cloud.google.com/python/docs/reference/pubsub/latest/multiprocessing`
- Auth guide: `https://cloud.google.com/pubsub/docs/authentication`
- Emulator guide: `https://cloud.google.com/pubsub/docs/emulator`
- Package registry: `https://pypi.org/project/google-cloud-pubsub/`

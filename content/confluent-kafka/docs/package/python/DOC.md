---
name: package
description: "confluent-kafka Python package guide for Apache Kafka producers, consumers, admin APIs, and Schema Registry"
metadata:
  languages: "python"
  versions: "2.13.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "confluent,kafka,streaming,producer,consumer,schema-registry"
---

# confluent-kafka Python Package Guide

## Golden Rule

Use `confluent-kafka` for Python Kafka clients when you need production-grade Kafka support, Confluent Cloud support, transactions, admin APIs, or Schema Registry integration. Start with the current PyPI package and maintainer README, and treat the Confluent-hosted API reference as useful but slightly behind the current package release when versions disagree.

## Install

Pin the package version your project expects:

```bash
python -m pip install "confluent-kafka==2.13.2"
```

Schema Registry extras:

```bash
python -m pip install "confluent-kafka[avro,schemaregistry]==2.13.2"
python -m pip install "confluent-kafka[json,schemaregistry]==2.13.2"
python -m pip install "confluent-kafka[protobuf,schemaregistry]==2.13.2"
```

If you use Data Contracts rules, including CSFLE:

```bash
python -m pip install "confluent-kafka[avro,schemaregistry,rules]==2.13.2"
```

Important install note:

- Pre-built Linux wheels do not include SASL Kerberos/GSSAPI support.
- If you need Kerberos, or if your platform does not have prebuilt wheels, install from source after installing `librdkafka` and the platform build dependencies.

Source install examples from the maintainer install guide:

```bash
# macOS
brew install librdkafka
python -m pip install --no-binary confluent-kafka confluent-kafka
```

## Core Imports

```python
from confluent_kafka import Consumer, Producer
from confluent_kafka.admin import AdminClient, NewTopic
from confluent_kafka.schema_registry import SchemaRegistryClient
```

Async producer:

```python
from confluent_kafka.aio import AIOProducer
```

## Authentication And Setup

### Minimal local or self-managed broker config

```python
common_config = {
    "bootstrap.servers": "localhost:9092",
}
```

### Confluent Cloud with API key and secret

```python
cloud_config = {
    "bootstrap.servers": "pkc-xxxxx.region.provider.confluent.cloud:9092",
    "security.protocol": "SASL_SSL",
    "sasl.mechanisms": "PLAIN",
    "sasl.username": "<CLUSTER_API_KEY>",
    "sasl.password": "<CLUSTER_API_SECRET>",
}
```

Consumer-specific fields:

```python
consumer_config = {
    **cloud_config,
    "group.id": "orders-service",
    "auto.offset.reset": "earliest",
}
```

Notes:

- `bootstrap.servers` is the one required setting everywhere.
- `group.id` is required for normal consumer group usage.
- `auto.offset.reset` matters only when there is no committed offset yet.
- For Confluent Cloud, the common starting auth shape is `SASL_SSL` + `PLAIN` + API key/secret.

### Kafka OAuth on Confluent Cloud

For OIDC or workload identity setups, Confluent documents Python client configs using:

```python
oauth_config = {
    "bootstrap.servers": "your-bootstrap-server:9092",
    "security.protocol": "SASL_SSL",
    "sasl.mechanism": "OAUTHBEARER",
    "sasl.oauthbearer.token.endpoint.url": "https://<your-idp>/oauth2/token",
    "sasl.oauthbearer.client.id": "<client-id>",
    "sasl.oauthbearer.client.secret": "<client-secret>",
    "sasl.oauthbearer.scope": "kafka:read kafka:write",
    "sasl.oauthbearer.extensions": "logicalCluster=<lkc-xxxxx>,identityPoolId=<pool-yyyyy>",
    "oauth_cb": oauth_token_refresh_cb,
}
```

The Python client also exposes `oauth_cb` in the main client configuration for `sasl.mechanisms=OAUTHBEARER`.

### Schema Registry config

Basic client:

```python
schema_registry_conf = {
    "url": "https://<schema-registry-endpoint>",
    "basic.auth.credentials.source": "USER_INFO",
    "basic.auth.user.info": "<schema-registry-api-key>:<schema-registry-api-secret>",
}

schema_registry_client = SchemaRegistryClient(schema_registry_conf)
```

Notes:

- `schema.registry.url` is the canonical config property in Confluent docs; in Python code you pass it as `url` to `SchemaRegistryClient`.
- `basic.auth.user.info` must be `user:password`.
- `basic.auth.credentials.source` supports `URL`, `USER_INFO`, and `SASL_INHERIT`.
- For Schema Registry OAuth, Confluent documents built-in OAuth handling on the client side and notes that `SchemaRegistryClient` does not use `oauth_cb`.

## Producer

The synchronous `Producer` is the default choice for scripts, workers, and high-throughput pipelines.

```python
from confluent_kafka import Producer

def delivery_report(err, msg) -> None:
    if err is not None:
        print(f"delivery failed: {err}")
    else:
        print(f"delivered to {msg.topic()} [{msg.partition()}] @ {msg.offset()}")

producer = Producer({"bootstrap.servers": "localhost:9092"})

for value in ["one", "two", "three"]:
    producer.poll(0)
    producer.produce("events", value=value.encode("utf-8"), callback=delivery_report)

producer.flush()
```

Important behavior:

- `produce()` is asynchronous and queues work locally.
- Delivery callbacks are served by `poll()` or `flush()`.
- If the internal queue fills, `produce()` can raise `BufferError`.

### Delivery guarantees

For stronger durability on a producer:

```python
producer = Producer(
    {
        "bootstrap.servers": "localhost:9092",
        "acks": "all",
        "enable.idempotence": True,
    }
)
```

Use this pattern when duplicate suppression and safer retries matter more than raw latency.

## Consumer

```python
from confluent_kafka import Consumer

consumer = Consumer(
    {
        "bootstrap.servers": "localhost:9092",
        "group.id": "demo-group",
        "auto.offset.reset": "earliest",
    }
)

consumer.subscribe(["events"])

try:
    while True:
        msg = consumer.poll(1.0)
        if msg is None:
            continue
        if msg.error():
            print(f"consumer error: {msg.error()}")
            continue

        print(msg.key(), msg.value())
finally:
    consumer.close()
```

Notes:

- Always call `close()` so the client leaves the consumer group cleanly and commits final offsets when configured to do so.
- `poll()` returns `None` on timeout.
- Handle `msg.error()` before reading the payload.

## Admin API

`AdminClient` methods are asynchronous and return a `dict` of futures keyed by the entity you requested.

```python
from confluent_kafka.admin import AdminClient, NewTopic

admin = AdminClient({"bootstrap.servers": "localhost:9092"})

futures = admin.create_topics(
    [NewTopic("events", num_partitions=3, replication_factor=1)]
)

for topic, future in futures.items():
    try:
        future.result()
        print(f"created {topic}")
    except Exception as exc:
        print(f"failed to create {topic}: {exc}")
```

Use a higher replication factor in real multi-broker production clusters.

## Schema Registry And Serialization

Confluent recommends direct serializers and deserializers instead of the experimental `SerializingProducer` and `DeserializingConsumer`.

Avro example:

```python
from confluent_kafka import Producer
from confluent_kafka.schema_registry import SchemaRegistryClient
from confluent_kafka.schema_registry.avro import AvroSerializer
from confluent_kafka.serialization import MessageField, SerializationContext

schema_registry_client = SchemaRegistryClient(
    {
        "url": "https://<schema-registry-endpoint>",
        "basic.auth.credentials.source": "USER_INFO",
        "basic.auth.user.info": "<schema-registry-api-key>:<schema-registry-api-secret>",
    }
)

schema_str = """
{
  "type": "record",
  "name": "User",
  "fields": [{"name": "id", "type": "string"}]
}
"""

serializer = AvroSerializer(
    schema_registry_client,
    schema_str,
    to_dict=lambda obj, ctx: obj,
)

producer = Producer({"bootstrap.servers": "localhost:9092"})

payload = serializer({"id": "u-123"}, SerializationContext("users", MessageField.VALUE))
producer.produce("users", value=payload)
producer.flush()
```

Use the same Schema Registry client with JSON Schema or Protobuf serializers when your schema format differs.

## AsyncIO Producer

Use `AIOProducer` when you are already inside an event loop and do not want Kafka I/O to block it.

```python
import asyncio
from confluent_kafka.aio import AIOProducer

async def main() -> None:
    producer = AIOProducer({"bootstrap.servers": "localhost:9092"})
    try:
        delivery_future = await producer.produce("events", value=b"hello")
        await delivery_future
        await producer.flush()
    finally:
        await producer.close()

asyncio.run(main())
```

Important limitation:

- The batched async produce path does not support per-message headers.
- If you need headers from an async app, use the synchronous `Producer.produce()` on a worker thread or executor for that path.

## Transactions And Exactly-Once Processing

Transactions build on the idempotent producer and require a unique `transactional.id`.

```python
transactional_producer = Producer(
    {
        "bootstrap.servers": "localhost:9092",
        "transactional.id": "orders-service-v1",
    }
)

transactional_producer.init_transactions()
transactional_producer.begin_transaction()
```

When consuming input as part of the transaction:

- Configure the consumer with `enable.auto.commit=false`.
- Use `isolation.level=read_committed` on transaction-aware consumers.
- Send offsets to the transaction before committing it.

This is the path to use for exactly-once pipelines, but it is more operationally strict than simple at-least-once processing.

## Common Pitfalls

- Forgetting `poll()` and `flush()` on the producer means delivery callbacks, stats callbacks, throttle callbacks, and some logging callbacks will not be served.
- Forgetting `consumer.close()` leads to slow or messy group rebalances.
- Treating `produce()` as synchronous is wrong; it only queues the message locally.
- `SerializingProducer` and `DeserializingConsumer` are still marked experimental in the official API docs.
- `AvroProducer` and `AvroConsumer` are legacy and deprecated.
- Confluent Cloud Kafka credentials and Schema Registry credentials are often different. Do not assume one API key works for both.
- On Linux, Kerberos support is not included in the prebuilt wheel; source install is required.
- For very old Kafka brokers, the client docs still note `broker.version.fallback` and `api.version.request` concerns. You usually do not need these on Kafka 0.10+ or modern Confluent deployments.

## Version-Sensitive Notes For 2.13.2

- PyPI lists `2.13.2` as the current package release as of 2026-03-12.
- The maintainer README on PyPI and GitHub documents current features such as `AIOProducer`, Schema Registry async support, and package extras.
- The Confluent-hosted API reference at the docs URL still renders `confluent-kafka 2.11.0`, so use it for API shape and class docs, but cross-check current package behavior against the maintainer README and PyPI metadata when working on `2.13.2`.
- The official API docs explicitly recommend direct serializers instead of relying on the experimental `SerializingProducer` and `DeserializingConsumer`.

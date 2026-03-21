---
name: package
description: "kombu package guide for Python - broker connections, producers, consumers, transports, and retries"
metadata:
  languages: "python"
  versions: "5.6.2"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "kombu,messaging,amqp,rabbitmq,redis,sqs,queues"
---

# kombu Python Package Guide

## What It Is

`kombu` is the low-level messaging library used by Celery. Use it when you need direct broker access in Python without adopting Celery's task layer.

The core objects are:

- `Connection` for broker and transport connectivity
- `Exchange` and `Queue` for routing
- `Producer` for publishing
- `Consumer` for receiving and acknowledging messages
- `SimpleQueue` for the quickest request/response or one-queue workflows

If you only need a task queue, use Celery. If you need broker-aware application code, transport-specific setup, or explicit publish/consume control, use `kombu`.

## Install

Use the version used here unless the project already pins another compatible release:

```bash
python -m pip install "kombu==5.6.2"
```

For lockfile-based projects:

```bash
uv add "kombu==5.6.2"
poetry add "kombu==5.6.2"
```

AMQP works with Kombu's standard dependencies. Other transports can require transport-specific client libraries and configuration in the host application environment.

## Initialization And Broker Setup

`Connection` is lazy. Creating it does not open the socket yet. Errors usually appear on `connect()`, `drain_events()`, channel creation, or publish/consume operations.

### Common broker URLs

```python
from kombu import Connection

# RabbitMQ / AMQP
amqp = Connection("amqp://guest:guest@localhost:5672//")

# Redis transport
redis = Connection("redis://localhost:6379/0")

# Amazon SQS transport
sqs = Connection(
    "sqs://",
    transport_options={"region": "us-east-1"},
)

# In-memory transport for single-process tests
memory = Connection("memory://")
```

Use a context manager so channels and sockets close cleanly:

```python
from kombu import Connection

with Connection("amqp://guest:guest@localhost:5672//", heartbeat=30) as conn:
    conn.connect()
    print(conn.connected)
```

### Keep configuration outside code

```python
import os

from kombu import Connection

broker_url = os.environ["BROKER_URL"]

transport_options = {}
if broker_url.startswith("sqs://"):
    transport_options["region"] = os.getenv("AWS_REGION", "us-east-1")

with Connection(broker_url, transport_options=transport_options) as conn:
    conn.connect()
```

For SQS, upstream Celery docs recommend either IAM/environment credentials with `sqs://` or carefully URL-encoding credentials if you embed them in the URL. Prefer IAM roles or environment-based auth over hardcoding secrets.

### SSL and transport options

Pass broker-specific TLS and transport configuration at connection creation time:

```python
import ssl

from kombu import Connection

conn = Connection(
    "amqps://user:password@broker.example.com:5671//",
    ssl={"cert_reqs": ssl.CERT_REQUIRED},
    heartbeat=30,
)
```

Keep `transport_options` close to the broker URL. Redis, SQS, AMQP, and virtual transports do not share the same options.

## Publish Messages

The normal pattern is:

1. Create an `Exchange`
2. Create a `Queue`
3. Declare them on a channel
4. Publish with an explicit serializer and retry policy

```python
from kombu import Connection, Exchange, Producer, Queue

exchange = Exchange("tasks", type="direct")
queue = Queue("tasks", exchange=exchange, routing_key="tasks.process")

with Connection("amqp://guest:guest@localhost:5672//") as conn:
    with conn.channel() as channel:
        producer = Producer(channel, exchange=exchange, routing_key="tasks.process")

        producer.publish(
            {"task": "resize-image", "image_id": 42},
            serializer="json",
            declare=[queue],
            retry=True,
            retry_policy={
                "max_retries": 3,
                "interval_start": 0,
                "interval_step": 0.5,
                "interval_max": 2,
            },
        )
```

Use `serializer="json"` unless you have a specific interoperability reason not to. Kombu's serializer docs explicitly treat pickle and YAML as disabled by default because of security risk.

## Consume Messages

For durable workers:

- set `accept` explicitly
- `ack()` only after successful processing
- use `reject()` or let the broker retry according to your queue semantics
- set `prefetch_count` deliberately instead of accepting the default

```python
from kombu import Connection, Consumer, Exchange, Queue

exchange = Exchange("tasks", type="direct")
queue = Queue("tasks", exchange=exchange, routing_key="tasks.process")

def handle_message(body, message):
    try:
        print("received", body)
        message.ack()
    except Exception:
        message.reject()
        raise

with Connection("amqp://guest:guest@localhost:5672//", heartbeat=30) as conn:
    with Consumer(
        conn,
        queues=[queue],
        callbacks=[handle_message],
        accept=["json"],
        prefetch_count=10,
    ):
        while True:
            conn.drain_events(timeout=1)
            conn.heartbeat_check()
```

If you are building a long-running worker process, upstream docs recommend `ConsumerMixin` instead of hand-writing the lifecycle loop each time.

## Fastest Working API: `SimpleQueue`

Use `SimpleQueue` when you only need one queue and do not need custom routing yet.

```python
from kombu import Connection

with Connection("amqp://guest:guest@localhost:5672//") as conn:
    simple_queue = conn.SimpleQueue("demo")
    try:
        simple_queue.put({"hello": "world"}, serializer="json")

        message = simple_queue.get(block=True, timeout=5)
        print(message.payload)
        message.ack()
    finally:
        simple_queue.close()
```

This is the quickest path for scripts, smoke tests, and integration tests. Move to explicit `Exchange` and `Queue` objects once routing or broker topology matters.

## Retry, Failover, And Pools

### Multiple broker URLs

```python
from kombu import Connection

conn = Connection(
    [
        "amqp://guest:guest@rabbitmq-a:5672//",
        "amqp://guest:guest@rabbitmq-b:5672//",
    ],
    failover_strategy="round-robin",
)
```

### Retry a broker operation with `ensure`

```python
from kombu import Connection, Exchange, Producer

exchange = Exchange("events", type="fanout")

with Connection("amqp://guest:guest@localhost:5672//") as conn:
    with conn.channel() as channel:
        producer = Producer(channel, exchange=exchange)
        safe_publish = conn.ensure(
            producer,
            producer.publish,
            max_retries=3,
        )

        safe_publish({"event": "started"}, serializer="json")
```

### Reuse pooled connections

```python
from kombu import Connection

conn = Connection("amqp://guest:guest@localhost:5672//")
pool = conn.Pool(limit=10)

with pool.acquire(block=True) as pooled_conn:
    pooled_conn.connect()
    print("pooled connection ready")
```

Use retries for network and broker availability failures. Do not bury application-level validation errors inside retry loops.

## Serialization And Message Safety

Upstream Kombu docs default to safe serializers:

- JSON is the right default for most application code
- binary payloads should be sent as raw bytes or explicitly encoded
- pickle, YAML, and msgpack are disabled by default unless you enable insecure serializers

Examples:

```python
producer.publish({"ok": True}, serializer="json")
producer.publish(b"raw-bytes", content_type="application/octet-stream", content_encoding="binary")
```

If you need non-JSON serializers, enable them intentionally and keep the consumer `accept` list in sync.

## Common Pitfalls

- Constructing `Connection(...)` does not prove the broker is reachable. Call `connect()` or perform a real broker operation.
- `drain_events()` blocks until a message arrives or the timeout expires. Wrap it in retry and shutdown handling for worker processes.
- If you set a heartbeat on the connection, call `heartbeat_check()` regularly inside manual consume loops.
- Do not rely on broad serializer defaults. Set `serializer=` on publish and `accept=` on consume.
- `SimpleQueue` is convenient but limited. Do not use it when you need exchange types, routing keys, or broker topology control.
- Keep broker credentials out of source files. Use environment variables, secrets managers, or platform credentials.
- Transport options are transport-specific. A Redis option copied into an SQS or AMQP connection will not do what you expect.

## Version-Sensitive Notes For 5.6.x

- The docs URL `https://kombu.readthedocs.io/en/latest/` is a rolling docs entry point. For `5.6.2`, prefer the `stable` docs pages plus the `5.6.2` PyPI release page when you need version-pinned behavior.
- The 5.6 changelog adds `max_prefetch` support to QoS so workers can restore a configured prefetch ceiling after temporary reductions.
- The Redis transport in 5.6 respects `polling_interval` from `transport_options`, which matters for busy polling loops and latency tuning.
- Kombu 5.5 introduced native delayed delivery queue support for RabbitMQ quorum queues. The changelog notes that direct exchanges are not supported for that path.
- Kombu 5.6 also formalizes the Python 3.9+ baseline. Do not copy older Python 3.8-era examples forward unchanged.

## Official Sources Used

- Stable docs root: https://docs.celeryq.dev/projects/kombu/en/stable/
- Connections guide: https://docs.celeryq.dev/projects/kombu/en/stable/userguide/connections.html
- Producers guide: https://docs.celeryq.dev/projects/kombu/en/stable/userguide/producers.html
- Consumers guide: https://docs.celeryq.dev/projects/kombu/en/stable/userguide/consumers.html
- Simple interface guide: https://docs.celeryq.dev/projects/kombu/en/stable/userguide/simple.html
- Pools guide: https://docs.celeryq.dev/projects/kombu/en/stable/userguide/pools.html
- Serialization guide: https://docs.celeryq.dev/projects/kombu/en/stable/userguide/serialization.html
- Changelog: https://docs.celeryq.dev/projects/kombu/en/stable/changelog.html
- PyPI release page: https://pypi.org/project/kombu/5.6.2/
- Celery SQS broker auth notes: https://docs.celeryq.dev/en/main/getting-started/backends-and-brokers/sqs.html

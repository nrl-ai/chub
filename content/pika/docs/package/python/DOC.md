---
name: package
description: "Pika Python package guide for AMQP 0-9-1 messaging with RabbitMQ and other compatible brokers"
metadata:
  languages: "python"
  versions: "1.3.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pika,rabbitmq,amqp,messaging,queues,python"
---

# Pika Python Package Guide

## Golden Rule

Use `pika` for AMQP 0-9-1 messaging in Python, choose the adapter that matches your runtime model, and treat connections and channels as stateful broker resources instead of stateless request clients. `BlockingConnection` is the practical default for scripts and workers; use `AsyncioConnection` or another async adapter when you are already inside an event loop.

## Install

Pin the version your project expects:

```bash
python -m pip install "pika==1.3.2"
```

Common alternatives:

```bash
uv add "pika==1.3.2"
poetry add "pika==1.3.2"
```

## Connection Setup

### AMQP URL

For most apps, start from an AMQP URL and let `URLParameters` parse it:

```python
import pika

params = pika.URLParameters(
    "amqp://guest:guest@localhost:5672/%2F?heartbeat=30&blocked_connection_timeout=300"
)

connection = pika.BlockingConnection(params)
channel = connection.channel()
```

Notes:

- Use `amqps://...` when your broker requires TLS.
- The default RabbitMQ virtual host `/` appears as `%2F` in the URL.
- `heartbeat` and `blocked_connection_timeout` are worth setting explicitly for long-running workers.

### Structured connection parameters

Use `ConnectionParameters` when the broker config comes from separate fields:

```python
import pika

credentials = pika.PlainCredentials("app-user", "secret-password")

params = pika.ConnectionParameters(
    host="rabbitmq.internal",
    port=5672,
    virtual_host="/",
    credentials=credentials,
    heartbeat=30,
    blocked_connection_timeout=300,
    client_properties={"connection_name": "billing-worker"},
)

connection = pika.BlockingConnection(params)
channel = connection.channel()
```

### TLS

For TLS, build an `ssl.SSLContext` and pass it through `SSLOptions`:

```python
import ssl
import pika

context = ssl.create_default_context(cafile="/etc/ssl/certs/ca.pem")
context.load_cert_chain(
    certfile="/etc/ssl/certs/client-cert.pem",
    keyfile="/etc/ssl/private/client-key.pem",
)

params = pika.ConnectionParameters(
    host="rabbitmq.example.com",
    port=5671,
    virtual_host="/",
    credentials=pika.PlainCredentials("app-user", "secret-password"),
    ssl_options=pika.SSLOptions(context, "rabbitmq.example.com"),
)

connection = pika.BlockingConnection(params)
```

## Core Usage

### Publish a message

Declare the queue before publishing, enable publisher confirms when delivery matters, and use persistent messages with durable queues if the message must survive broker restarts.

```python
import json
import pika

params = pika.URLParameters("amqp://guest:guest@localhost:5672/%2F")
connection = pika.BlockingConnection(params)
channel = connection.channel()

channel.queue_declare(queue="jobs", durable=True)
channel.confirm_delivery()

payload = {"job_id": "job-123", "task": "rebuild-index"}

channel.basic_publish(
    exchange="",
    routing_key="jobs",
    body=json.dumps(payload).encode("utf-8"),
    properties=pika.BasicProperties(
        content_type="application/json",
        delivery_mode=pika.DeliveryMode.Persistent,
    ),
    mandatory=True,
)

connection.close()
```

### Consume with manual acknowledgements

Use manual acknowledgements for real work queues so messages can be retried when the worker crashes partway through processing.

```python
import json
import pika

params = pika.URLParameters(
    "amqp://guest:guest@localhost:5672/%2F?heartbeat=30&blocked_connection_timeout=300"
)
connection = pika.BlockingConnection(params)
channel = connection.channel()

channel.queue_declare(queue="jobs", durable=True)
channel.basic_qos(prefetch_count=1)

def handle_message(ch, method, properties, body):
    try:
        payload = json.loads(body)
        print(f"processing {payload['job_id']}")
        ch.basic_ack(delivery_tag=method.delivery_tag)
    except Exception:
        ch.basic_nack(delivery_tag=method.delivery_tag, requeue=True)

channel.basic_consume(queue="jobs", on_message_callback=handle_message, auto_ack=False)
channel.start_consuming()
```

### Pull one message synchronously

`basic_get()` is fine for admin tools, one-off scripts, or tests. Do not use it as a high-throughput polling loop unless you intentionally want pull-style semantics.

```python
import pika

connection = pika.BlockingConnection(
    pika.URLParameters("amqp://guest:guest@localhost:5672/%2F")
)
channel = connection.channel()

method, properties, body = channel.basic_get(queue="jobs", auto_ack=False)

if method is not None:
    print(body.decode("utf-8"))
    channel.basic_ack(method.delivery_tag)

connection.close()
```

## Async Adapters And Threading

Pika ships multiple adapters. Use the adapter that matches the concurrency model you already have:

- `BlockingConnection`: simplest choice for scripts, CLIs, and worker processes
- `AsyncioConnection`: for applications already using `asyncio`
- `SelectConnection`: event-loop driven without `asyncio`

Threading rules that matter in practice:

- Do not share a channel across threads.
- Do not assume `BlockingConnection` is generally thread-safe.
- If another thread must notify a blocking connection, use `add_callback_threadsafe()` instead of touching channels directly from that thread.

## Recovery And Retry Pattern

Pika can raise connection and channel exceptions when the broker is unavailable, closes the connection, or rejects topology changes. For long-running consumers, wrap startup in a reconnect loop and recreate the connection, channel, queue declarations, QoS, and consumer bindings after reconnect.

```python
import time
import pika

def handle_message(ch, method, properties, body):
    print(body.decode("utf-8"))
    ch.basic_ack(delivery_tag=method.delivery_tag)

while True:
    try:
        connection = pika.BlockingConnection(
            pika.URLParameters("amqp://guest:guest@localhost:5672/%2F")
        )
        channel = connection.channel()
        channel.queue_declare(queue="jobs", durable=True)
        channel.basic_qos(prefetch_count=1)
        channel.basic_consume(queue="jobs", on_message_callback=handle_message)
        channel.start_consuming()
    except pika.exceptions.AMQPConnectionError:
        time.sleep(5)
```

Keep the reconnect policy outside your message handler so failures during startup and failures during message processing are handled separately.

## Configuration Notes

- Broker credentials are usually username/password plus virtual host, passed either in the AMQP URL or through `PlainCredentials`.
- Use `ExternalCredentials` only when your broker is configured for external SASL auth and the upstream broker docs explicitly require it.
- Set `client_properties["connection_name"]` so RabbitMQ management UI and logs identify your app instance clearly.
- Set `heartbeat` for long-lived connections; leaving it implicit makes timeout behavior harder to reason about.
- Set `blocked_connection_timeout` when using `BlockingConnection` so synchronous calls fail instead of hanging indefinitely during broker resource alarms.

## Common Pitfalls

- Durable queues are not enough on their own. If the message must persist, publish with `delivery_mode=pika.DeliveryMode.Persistent`.
- `auto_ack=True` acknowledges delivery before your code finishes processing. That is unsafe for jobs that must be retried on failure.
- Publisher confirms are separate from queue durability. Use `channel.confirm_delivery()` when your producer needs broker-level publish acknowledgement.
- Blocking adapters can appear hung when the broker emits `Connection.Blocked`. The official examples recommend `blocked_connection_timeout` to break that state.
- Redeclare topology after reconnect. A fresh TCP connection does not preserve your previous channel state, consumers, or QoS configuration.
- AMQP URLs are easy to get subtly wrong. Encode the `/` virtual host as `%2F` and switch to `amqps://` for TLS.
- If your app already runs on `asyncio`, do not hide `BlockingConnection` inside random thread pools unless you have a clear reason. Use `AsyncioConnection` instead.

## Version-Sensitive Notes

- The version used here, PyPI stable release, and the stable docs URL all align on `1.3.2`.
- The upstream GitHub releases page also shows a newer beta line (`1.4.0b0`). Do not assume beta examples or unreleased changes apply to a project pinned to `1.3.2`.
- `1.3.2` is the current stable line documented at `readthedocs.io/en/stable/`; if your project upgrades to a later stable release, re-check adapter behavior and connection parameter defaults before copying old snippets.

## Official Sources

- Stable docs: https://pika.readthedocs.io/en/stable/
- Connection parameters: https://pika.readthedocs.io/en/stable/modules/parameters.html
- Credentials: https://pika.readthedocs.io/en/stable/modules/credentials.html
- Blocking consumer example: https://pika.readthedocs.io/en/stable/examples/blocking_consume.html
- URL parameters example: https://pika.readthedocs.io/en/stable/examples/using_urlparameters.html
- Heartbeat and blocked timeout example: https://pika.readthedocs.io/en/stable/examples/heartbeat_and_blocked_timeouts.html
- TLS example: https://pika.readthedocs.io/en/stable/examples/tls_server_authentication.html
- Asyncio consumer example: https://pika.readthedocs.io/en/stable/examples/asyncio_consumer.html
- FAQ: https://pika.readthedocs.io/en/stable/faq.html
- PyPI: https://pypi.org/project/pika/
- GitHub repository and releases: https://github.com/pika/pika

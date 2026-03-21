---
name: message-queue
description: "RabbitMQ Pika coding guidelines for Python message broker interactions"
metadata:
  languages: "python"
  versions: "1.3.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "rabbitmq,queue,amqp,messaging,async"
---

# RabbitMQ Pika Coding Guidelines (Python)

You are a RabbitMQ Pika coding expert. Help me with writing code using RabbitMQ message broker via the official Pika library.

## Golden Rule: Use the Correct and Current SDK

Always use the official Pika library for all RabbitMQ (AMQP 0-9-1) interactions in Python.

- **Library Name:** Pika
- **PyPI Package:** `pika`
- **Current Version:** 1.3.2 or higher
- **Python Support:** Python 3.9+

**Installation:**

```bash
pip install pika
```

**IMPORTANT:** Pika is the official pure-Python RabbitMQ/AMQP 0-9-1 client library. Do not use deprecated alternatives.

**Import Pattern:**

```python
import pika
```

## Initialization and Connection

### Environment Variables

Configure RabbitMQ connection using environment variables:

```python
import os
from dotenv import load_dotenv

load_dotenv()

# .env file
RABBITMQ_HOST=localhost
RABBITMQ_PORT=5672
RABBITMQ_USER=guest
RABBITMQ_PASS=guest
RABBITMQ_VHOST=/
```

### Basic Connection (Blocking)

```python
import pika

# Simple connection to localhost
connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

print("Connected to RabbitMQ")
```

### Connection with Parameters

```python
import pika
import os

credentials = pika.PlainCredentials(
    username=os.getenv('RABBITMQ_USER', 'guest'),
    password=os.getenv('RABBITMQ_PASS', 'guest')
)

parameters = pika.ConnectionParameters(
    host=os.getenv('RABBITMQ_HOST', 'localhost'),
    port=int(os.getenv('RABBITMQ_PORT', 5672)),
    virtual_host=os.getenv('RABBITMQ_VHOST', '/'),
    credentials=credentials,
    heartbeat=30,  # Heartbeat interval in seconds
    blocked_connection_timeout=300
)

connection = pika.BlockingConnection(parameters)
channel = connection.channel()
```

### Connection with URL

```python
import pika
import os

# Connection URL format: amqp://username:password@host:port/vhost
url = os.getenv('RABBITMQ_URL', 'amqp://guest:guest@localhost:5672/')

parameters = pika.URLParameters(url)
connection = pika.BlockingConnection(parameters)
channel = connection.channel()
```

### Connection Options

```python
import pika

parameters = pika.ConnectionParameters(
    host='localhost',
    port=5672,
    virtual_host='/',
    credentials=pika.PlainCredentials('guest', 'guest'),
    heartbeat=30,                      # Heartbeat interval in seconds
    blocked_connection_timeout=300,    # Timeout for blocked connection
    connection_attempts=3,             # Number of connection attempts
    retry_delay=2,                     # Delay between retries in seconds
    socket_timeout=10,                 # Socket timeout in seconds
    stack_timeout=15,                  # Stack timeout in seconds
    channel_max=2047,                  # Maximum number of channels
    frame_max=131072,                  # Maximum frame size
    locale='en_US'                     # Locale for error messages
)

connection = pika.BlockingConnection(parameters)
channel = connection.channel()
```

## Core Messaging Patterns

### 1. Simple Queue - Producer

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Declare queue (create if it doesn't exist)
channel.queue_declare(queue='hello', durable=True)

# Send message
message = 'Hello World!'
channel.basic_publish(
    exchange='',
    routing_key='hello',
    body=message,
    properties=pika.BasicProperties(
        delivery_mode=pika.DeliveryMode.Persistent  # Make message persistent
    )
)

print(f"Sent: {message}")

connection.close()
```

### 2. Simple Queue - Consumer

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Declare queue
channel.queue_declare(queue='hello', durable=True)

# Callback function for message processing
def callback(ch, method, properties, body):
    message = body.decode('utf-8')
    print(f"Received: {message}")

    # Acknowledge message
    ch.basic_ack(delivery_tag=method.delivery_tag)

# Consume messages
channel.basic_consume(
    queue='hello',
    on_message_callback=callback,
    auto_ack=False  # Manual acknowledgment
)

print('Waiting for messages. Press CTRL+C to exit.')
channel.start_consuming()
```

### 3. Work Queue with Prefetch

```python
import pika
import time

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.queue_declare(queue='task_queue', durable=True)

# Fair dispatch - only send one message at a time
channel.basic_qos(prefetch_count=1)

def callback(ch, method, properties, body):
    message = body.decode('utf-8')
    print(f"Processing: {message}")

    # Simulate work (one dot = 1 second)
    time.sleep(message.count('.'))

    print(f"Done: {message}")
    ch.basic_ack(delivery_tag=method.delivery_tag)

channel.basic_consume(
    queue='task_queue',
    on_message_callback=callback
)

print('Worker waiting for tasks. Press CTRL+C to exit.')
channel.start_consuming()
```

### 4. Publish/Subscribe with Fanout Exchange

**Publisher:**

```python
import pika
import sys

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Declare fanout exchange
channel.exchange_declare(exchange='logs', exchange_type='fanout')

message = ' '.join(sys.argv[1:]) or "info: Hello World!"

# Publish to exchange
channel.basic_publish(
    exchange='logs',
    routing_key='',  # Ignored for fanout exchange
    body=message
)

print(f"Sent: {message}")
connection.close()
```

**Subscriber:**

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Declare exchange
channel.exchange_declare(exchange='logs', exchange_type='fanout')

# Create exclusive queue (auto-delete when consumer disconnects)
result = channel.queue_declare(queue='', exclusive=True)
queue_name = result.method.queue

# Bind queue to exchange
channel.queue_bind(exchange='logs', queue=queue_name)

print(f'Waiting for logs. Queue: {queue_name}')

def callback(ch, method, properties, body):
    print(f"Received: {body.decode('utf-8')}")

channel.basic_consume(
    queue=queue_name,
    on_message_callback=callback,
    auto_ack=True
)

channel.start_consuming()
```

### 5. Routing with Direct Exchange

**Emitter:**

```python
import pika
import sys

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Declare direct exchange
channel.exchange_declare(exchange='direct_logs', exchange_type='direct')

severity = sys.argv[1] if len(sys.argv) > 1 else 'info'
message = ' '.join(sys.argv[2:]) or 'Hello World!'

# Publish with routing key
channel.basic_publish(
    exchange='direct_logs',
    routing_key=severity,
    body=message
)

print(f"Sent [{severity}]: {message}")
connection.close()
```

**Receiver:**

```python
import pika
import sys

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.exchange_declare(exchange='direct_logs', exchange_type='direct')

result = channel.queue_declare(queue='', exclusive=True)
queue_name = result.method.queue

# Severities to listen for
severities = sys.argv[1:] if len(sys.argv) > 1 else ['error', 'warning']

# Bind queue for each severity
for severity in severities:
    channel.queue_bind(
        exchange='direct_logs',
        queue=queue_name,
        routing_key=severity
    )

print(f'Waiting for logs with severities: {", ".join(severities)}')

def callback(ch, method, properties, body):
    print(f"[{method.routing_key}]: {body.decode('utf-8')}")

channel.basic_consume(
    queue=queue_name,
    on_message_callback=callback,
    auto_ack=True
)

channel.start_consuming()
```

### 6. Topics with Topic Exchange

**Emit Log Topic:**

```python
import pika
import sys

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Declare topic exchange
channel.exchange_declare(exchange='topic_logs', exchange_type='topic')

routing_key = sys.argv[1] if len(sys.argv) > 1 else 'anonymous.info'
message = ' '.join(sys.argv[2:]) or 'Hello World!'

channel.basic_publish(
    exchange='topic_logs',
    routing_key=routing_key,
    body=message
)

print(f"Sent [{routing_key}]: {message}")
connection.close()
```

**Receive Log Topic:**

```python
import pika
import sys

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.exchange_declare(exchange='topic_logs', exchange_type='topic')

result = channel.queue_declare(queue='', exclusive=True)
queue_name = result.method.queue

# Binding keys with pattern matching
# * (star) substitutes for exactly one word
# # (hash) substitutes for zero or more words
binding_keys = sys.argv[1:] if len(sys.argv) > 1 else ['#']

for binding_key in binding_keys:
    channel.queue_bind(
        exchange='topic_logs',
        queue=queue_name,
        routing_key=binding_key
    )

print(f'Waiting for logs matching: {", ".join(binding_keys)}')

def callback(ch, method, properties, body):
    print(f"[{method.routing_key}]: {body.decode('utf-8')}")

channel.basic_consume(
    queue=queue_name,
    on_message_callback=callback,
    auto_ack=True
)

channel.start_consuming()
```

### 7. RPC (Remote Procedure Call) Pattern

**RPC Client:**

```python
import pika
import uuid

class FibonacciRpcClient:
    def __init__(self):
        self.connection = pika.BlockingConnection(
            pika.ConnectionParameters('localhost')
        )
        self.channel = self.connection.channel()

        # Declare callback queue
        result = self.channel.queue_declare(queue='', exclusive=True)
        self.callback_queue = result.method.queue

        self.channel.basic_consume(
            queue=self.callback_queue,
            on_message_callback=self.on_response,
            auto_ack=True
        )

        self.response = None
        self.corr_id = None

    def on_response(self, ch, method, props, body):
        if self.corr_id == props.correlation_id:
            self.response = body

    def call(self, n):
        self.response = None
        self.corr_id = str(uuid.uuid4())

        self.channel.basic_publish(
            exchange='',
            routing_key='rpc_queue',
            properties=pika.BasicProperties(
                reply_to=self.callback_queue,
                correlation_id=self.corr_id
            ),
            body=str(n)
        )

        # Wait for response
        self.connection.process_data_events(time_limit=None)
        while self.response is None:
            self.connection.process_data_events(time_limit=None)

        return int(self.response)

    def close(self):
        self.connection.close()

# Usage
fibonacci_rpc = FibonacciRpcClient()

print("Requesting fib(30)")
response = fibonacci_rpc.call(30)
print(f"Result: {response}")

fibonacci_rpc.close()
```

**RPC Server:**

```python
import pika

def fib(n):
    if n == 0:
        return 0
    elif n == 1:
        return 1
    else:
        return fib(n - 1) + fib(n - 2)

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.queue_declare(queue='rpc_queue')

def on_request(ch, method, props, body):
    n = int(body)
    print(f"Computing fib({n})")

    response = fib(n)

    # Send response
    ch.basic_publish(
        exchange='',
        routing_key=props.reply_to,
        properties=pika.BasicProperties(
            correlation_id=props.correlation_id
        ),
        body=str(response)
    )

    ch.basic_ack(delivery_tag=method.delivery_tag)

channel.basic_qos(prefetch_count=1)
channel.basic_consume(queue='rpc_queue', on_message_callback=on_request)

print("Awaiting RPC requests")
channel.start_consuming()
```

## Advanced Configuration

### Message Properties

```python
import pika
import time

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.queue_declare(queue='advanced_queue', durable=True)

message = 'Important message'

# Comprehensive message properties
properties = pika.BasicProperties(
    delivery_mode=pika.DeliveryMode.Persistent,  # Message persistence
    priority=5,                                   # Message priority (0-255)
    content_type='text/plain',                    # MIME type
    content_encoding='utf-8',                     # Encoding
    expiration='60000',                           # TTL in milliseconds
    message_id='msg-123',                         # Application message ID
    timestamp=int(time.time()),                   # Timestamp
    type='notification',                          # Message type
    user_id='guest',                              # Creating user
    app_id='my-app',                              # Application ID
    headers={                                     # Custom headers
        'x-custom-header': 'value',
        'retry-count': 0
    }
)

channel.basic_publish(
    exchange='',
    routing_key='advanced_queue',
    body=message,
    properties=properties
)

connection.close()
```

### Queue Options

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Queue with advanced options
channel.queue_declare(
    queue='advanced_queue',
    durable=True,              # Queue survives broker restart
    exclusive=False,           # Can be accessed by other connections
    auto_delete=False,         # Queue won't auto-delete
    arguments={
        'x-message-ttl': 60000,                # Message TTL in milliseconds
        'x-expires': 300000,                   # Queue expires after 5 minutes
        'x-max-length': 1000,                  # Maximum queue length
        'x-max-priority': 10,                  # Enable priority queue (0-10)
        'x-dead-letter-exchange': 'dlx',       # Dead letter exchange
        'x-dead-letter-routing-key': 'dead.letter',  # DLX routing key
        'x-queue-mode': 'lazy'                 # Lazy queue mode
    }
)

connection.close()
```

### Exchange Options

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Exchange with advanced options
channel.exchange_declare(
    exchange='advanced_exchange',
    exchange_type='topic',
    durable=True,           # Exchange survives broker restart
    auto_delete=False,      # Exchange won't auto-delete
    internal=False,         # Can be published to by clients
    arguments={
        'alternate-exchange': 'alternate_exchange'  # AE for unroutable messages
    }
)

connection.close()
```

### Dead Letter Queue (DLQ)

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Declare dead letter exchange
channel.exchange_declare(exchange='dlx', exchange_type='direct', durable=True)

# Declare dead letter queue
channel.queue_declare(queue='dead_letter_queue', durable=True)

# Bind DLQ to DLX
channel.queue_bind(
    queue='dead_letter_queue',
    exchange='dlx',
    routing_key='dead.letter'
)

# Declare main queue with DLX
channel.queue_declare(
    queue='main_queue',
    durable=True,
    arguments={
        'x-dead-letter-exchange': 'dlx',
        'x-dead-letter-routing-key': 'dead.letter',
        'x-message-ttl': 10000  # Messages expire after 10 seconds
    }
)

print("Dead letter queue setup complete")
connection.close()
```

### Consuming from Dead Letter Queue

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.queue_declare(queue='dead_letter_queue', durable=True)

def callback(ch, method, properties, body):
    print(f"Dead letter received: {body.decode('utf-8')}")
    print(f"Original routing key: {method.routing_key}")

    # Check death reason
    if properties.headers and 'x-death' in properties.headers:
        print(f"Death reason: {properties.headers['x-death']}")

    # Process or log dead letter
    ch.basic_ack(delivery_tag=method.delivery_tag)

channel.basic_consume(
    queue='dead_letter_queue',
    on_message_callback=callback
)

print('Waiting for dead letter messages. Press CTRL+C to exit.')
channel.start_consuming()
```

### Message Rejection and Requeuing

```python
import pika
import random

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.queue_declare(queue='task_queue', durable=True)
channel.basic_qos(prefetch_count=1)

def callback(ch, method, properties, body):
    try:
        message = body.decode('utf-8')
        print(f"Processing: {message}")

        # Simulate processing with potential failure
        if random.random() < 0.3:
            raise Exception('Random failure')

        # Success - acknowledge
        ch.basic_ack(delivery_tag=method.delivery_tag)

    except Exception as e:
        print(f"Processing failed: {e}")

        # Check retry count
        retry_count = 0
        if properties.headers and 'x-retry-count' in properties.headers:
            retry_count = properties.headers['x-retry-count']

        if retry_count < 3:
            # Reject and requeue for retry
            ch.basic_nack(delivery_tag=method.delivery_tag, requeue=True)
        else:
            # Max retries reached - reject without requeue (goes to DLQ)
            ch.basic_nack(delivery_tag=method.delivery_tag, requeue=False)

channel.basic_consume(
    queue='task_queue',
    on_message_callback=callback
)

print('Consumer with retry waiting for tasks. Press CTRL+C to exit.')
channel.start_consuming()
```

### Publisher Confirms

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Enable publisher confirms
channel.confirm_delivery()

channel.queue_declare(queue='confirm_queue', durable=True)

message = 'Important message'

try:
    channel.basic_publish(
        exchange='',
        routing_key='confirm_queue',
        body=message,
        properties=pika.BasicProperties(
            delivery_mode=pika.DeliveryMode.Persistent
        ),
        mandatory=True
    )
    print("Message confirmed by broker")
except pika.exceptions.UnroutableError:
    print("Message was returned (unroutable)")
except pika.exceptions.NackError:
    print("Message was nacked by broker")

connection.close()
```

### Batch Publishing with Confirms

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.confirm_delivery()
channel.queue_declare(queue='batch_queue', durable=True)

messages = [f'Message {i}' for i in range(100)]

confirmed = 0
nacked = 0

for message in messages:
    try:
        channel.basic_publish(
            exchange='',
            routing_key='batch_queue',
            body=message,
            properties=pika.BasicProperties(
                delivery_mode=pika.DeliveryMode.Persistent
            )
        )
        confirmed += 1
    except (pika.exceptions.UnroutableError, pika.exceptions.NackError):
        nacked += 1

print(f"Confirmed: {confirmed}, Nacked: {nacked}")
connection.close()
```

## Error Handling and Reconnection

### Connection Error Handling

```python
import pika
import time

def connect_rabbitmq():
    while True:
        try:
            connection = pika.BlockingConnection(
                pika.ConnectionParameters('localhost')
            )
            channel = connection.channel()
            print("Connected to RabbitMQ")
            return connection, channel

        except pika.exceptions.AMQPConnectionError as e:
            print(f"Connection failed: {e}")
            print("Retrying in 5 seconds...")
            time.sleep(5)

# Usage
connection, channel = connect_rabbitmq()
```

### Robust Connection Manager

```python
import pika
import time
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class RabbitMQConnection:
    def __init__(self, host='localhost', max_retries=5, retry_delay=5):
        self.host = host
        self.max_retries = max_retries
        self.retry_delay = retry_delay
        self.connection = None
        self.channel = None

    def connect(self):
        retries = 0

        while retries < self.max_retries:
            try:
                self.connection = pika.BlockingConnection(
                    pika.ConnectionParameters(self.host)
                )
                self.channel = self.connection.channel()
                logger.info("Connected to RabbitMQ")
                return self.channel

            except pika.exceptions.AMQPConnectionError as e:
                retries += 1
                logger.error(f"Connection failed (attempt {retries}/{self.max_retries}): {e}")

                if retries < self.max_retries:
                    logger.info(f"Retrying in {self.retry_delay} seconds...")
                    time.sleep(self.retry_delay)
                else:
                    logger.error("Max retries reached. Giving up.")
                    raise

    def close(self):
        if self.connection and not self.connection.is_closed:
            self.connection.close()
            logger.info("Connection closed")

# Usage
manager = RabbitMQConnection(host='localhost', max_retries=5)
channel = manager.connect()

# Use channel...

manager.close()
```

### Graceful Shutdown

```python
import pika
import signal
import sys

connection = None
channel = None

def setup():
    global connection, channel

    connection = pika.BlockingConnection(
        pika.ConnectionParameters('localhost')
    )
    channel = connection.channel()

    channel.queue_declare(queue='shutdown_queue', durable=True)

    channel.basic_consume(
        queue='shutdown_queue',
        on_message_callback=callback
    )

def callback(ch, method, properties, body):
    print(f"Processing: {body.decode('utf-8')}")
    ch.basic_ack(delivery_tag=method.delivery_tag)

def cleanup(sig, frame):
    print("\nShutting down gracefully...")

    if channel and channel.is_open:
        channel.stop_consuming()
        channel.close()

    if connection and connection.is_open:
        connection.close()

    print("Cleanup complete")
    sys.exit(0)

# Register signal handlers
signal.signal(signal.SIGINT, cleanup)
signal.signal(signal.SIGTERM, cleanup)

setup()
print('Waiting for messages. Press CTRL+C to exit.')
channel.start_consuming()
```

## Common Patterns

### Message Retry with Delay

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Main queue
channel.queue_declare(
    queue='main_queue',
    durable=True,
    arguments={
        'x-dead-letter-exchange': 'retry',
        'x-dead-letter-routing-key': 'retry'
    }
)

# Retry exchange
channel.exchange_declare(exchange='retry', exchange_type='direct', durable=True)

# Retry queue with delay
channel.queue_declare(
    queue='retry_queue',
    durable=True,
    arguments={
        'x-message-ttl': 30000,  # 30 second delay
        'x-dead-letter-exchange': '',
        'x-dead-letter-routing-key': 'main_queue'
    }
)

channel.queue_bind(queue='retry_queue', exchange='retry', routing_key='retry')

print("Retry queue setup complete")
connection.close()
```

### Priority Queue

```python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

# Queue with priority support
channel.queue_declare(
    queue='priority_queue',
    durable=True,
    arguments={'x-max-priority': 10}
)

# Send messages with different priorities
messages = [
    ('Low priority', 1),
    ('High priority', 10),
    ('Medium priority', 5)
]

for message, priority in messages:
    channel.basic_publish(
        exchange='',
        routing_key='priority_queue',
        body=message,
        properties=pika.BasicProperties(priority=priority)
    )

print("Priority messages sent")
connection.close()
```

### Rate Limiting Consumer

```python
import pika
import time

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.queue_declare(queue='rate_limited_queue', durable=True)

# Process only 1 message at a time
channel.basic_qos(prefetch_count=1)

def callback(ch, method, properties, body):
    print(f"Processing: {body.decode('utf-8')}")

    # Simulate rate-limited processing (2 seconds delay)
    time.sleep(2)

    ch.basic_ack(delivery_tag=method.delivery_tag)

channel.basic_consume(
    queue='rate_limited_queue',
    on_message_callback=callback
)

print('Rate-limited consumer started. Press CTRL+C to exit.')
channel.start_consuming()
```

### JSON Message Handling

```python
import pika
import json

connection = pika.BlockingConnection(
    pika.ConnectionParameters('localhost')
)
channel = connection.channel()

channel.queue_declare(queue='json_queue', durable=True)

# Send JSON message
data = {
    'user': 'john',
    'action': 'login',
    'timestamp': 1609459200
}

message = json.dumps(data)

channel.basic_publish(
    exchange='',
    routing_key='json_queue',
    body=message,
    properties=pika.BasicProperties(
        content_type='application/json',
        delivery_mode=pika.DeliveryMode.Persistent
    )
)

print(f"Sent JSON: {data}")

# Receive JSON message
def callback(ch, method, properties, body):
    try:
        data = json.loads(body)
        print(f"Received JSON: {data}")
        ch.basic_ack(delivery_tag=method.delivery_tag)
    except json.JSONDecodeError as e:
        print(f"Invalid JSON: {e}")
        ch.basic_nack(delivery_tag=method.delivery_tag, requeue=False)

channel.basic_consume(
    queue='json_queue',
    on_message_callback=callback
)

# channel.start_consuming()  # Uncomment to start consuming
connection.close()
```

## Async Usage with Select Connection

### Async Producer

```python
import pika

def on_open(connection):
    connection.channel(on_open_callback=on_channel_open)

def on_channel_open(channel):
    channel.queue_declare(
        queue='async_queue',
        durable=True,
        callback=lambda method: on_queue_declared(channel)
    )

def on_queue_declared(channel):
    message = 'Hello Async World!'

    channel.basic_publish(
        exchange='',
        routing_key='async_queue',
        body=message,
        properties=pika.BasicProperties(
            delivery_mode=pika.DeliveryMode.Persistent
        )
    )

    print(f"Sent: {message}")
    channel.connection.close()

# Create async connection
parameters = pika.ConnectionParameters('localhost')
connection = pika.SelectConnection(parameters, on_open_callback=on_open)

try:
    connection.ioloop.start()
except KeyboardInterrupt:
    connection.close()
    connection.ioloop.start()
```

### Async Consumer

```python
import pika

class AsyncConsumer:
    def __init__(self, queue_name):
        self.queue_name = queue_name
        self.connection = None
        self.channel = None

    def connect(self):
        parameters = pika.ConnectionParameters('localhost')
        return pika.SelectConnection(
            parameters,
            on_open_callback=self.on_connection_open
        )

    def on_connection_open(self, connection):
        self.connection = connection
        self.connection.channel(on_open_callback=self.on_channel_open)

    def on_channel_open(self, channel):
        self.channel = channel
        self.channel.queue_declare(
            queue=self.queue_name,
            durable=True,
            callback=self.on_queue_declared
        )

    def on_queue_declared(self, method):
        self.channel.basic_qos(prefetch_count=1)
        self.channel.basic_consume(
            queue=self.queue_name,
            on_message_callback=self.on_message
        )
        print(f'Waiting for messages in {self.queue_name}')

    def on_message(self, ch, method, properties, body):
        print(f"Received: {body.decode('utf-8')}")
        ch.basic_ack(delivery_tag=method.delivery_tag)

    def run(self):
        self.connection = self.connect()
        try:
            self.connection.ioloop.start()
        except KeyboardInterrupt:
            self.connection.close()
            self.connection.ioloop.start()

# Usage
consumer = AsyncConsumer('async_queue')
consumer.run()
```

## Useful Links

- **Official Documentation:** https://www.rabbitmq.com/docs
- **Pika Documentation:** https://pika.readthedocs.io/
- **Pika GitHub:** https://github.com/pika/pika
- **RabbitMQ Tutorials (Python):** https://www.rabbitmq.com/tutorials
- **AMQP 0-9-1 Reference:** https://www.rabbitmq.com/amqp-0-9-1-reference.html

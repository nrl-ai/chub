---
name: streaming
description: "kafka-python - Apache Kafka client for Python streaming and messaging"
metadata:
  languages: "python"
  versions: "2.0.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "kafka,streaming,messaging,queue,events"
---

# kafka-python - Apache Kafka Client for Python

## Golden Rule

**ALWAYS use `kafka-python` version 2.0.2 or later.**

```bash
pip install kafka-python
```

**DO NOT use:**
- `pykafka` (different library)
- `confluent-kafka` (different library - C-based client)
- Outdated versions below 2.0.0

kafka-python is a pure Python client for Apache Kafka with support for producers, consumers, and admin operations.

## Installation

### Basic Installation

```bash
pip install kafka-python
```

### With UV

```bash
uv pip install kafka-python
```

### Adding to pyproject.toml

```toml
[project]
dependencies = [
    "kafka-python>=2.0.2"
]
```

### With Optional Compression Support

```bash
pip install kafka-python[snappy]
pip install kafka-python[lz4]
pip install kafka-python[zstd]
```

Or all compression codecs:

```bash
pip install kafka-python[snappy,lz4,zstd]
```

### Environment Variables

Create a `.env` file:

```bash
KAFKA_BROKERS=localhost:9092
KAFKA_USERNAME=your-username
KAFKA_PASSWORD=your-password
KAFKA_TOPIC=my-topic
KAFKA_GROUP_ID=my-group
```

Load environment variables in your application:

```python
import os
from dotenv import load_dotenv

load_dotenv()

brokers = os.getenv('KAFKA_BROKERS', 'localhost:9092').split(',')
topic = os.getenv('KAFKA_TOPIC', 'my-topic')
group_id = os.getenv('KAFKA_GROUP_ID', 'my-group')
```

## Initialization

### Basic Producer

```python
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')
```

### Basic Consumer

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer('my-topic',
                        bootstrap_servers='localhost:9092',
                        group_id='my-group')
```

### With Environment Variables

```python
import os
from kafka import KafkaProducer, KafkaConsumer

producer = KafkaProducer(
    bootstrap_servers=os.getenv('KAFKA_BROKERS', 'localhost:9092').split(',')
)

consumer = KafkaConsumer(
    os.getenv('KAFKA_TOPIC', 'my-topic'),
    bootstrap_servers=os.getenv('KAFKA_BROKERS', 'localhost:9092').split(','),
    group_id=os.getenv('KAFKA_GROUP_ID', 'my-group')
)
```

### With Multiple Brokers

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers=['kafka1:9092', 'kafka2:9092', 'kafka3:9092']
)
```

## Producer

### Creating a Producer with Configuration

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    client_id='my-producer',
    acks='all',
    retries=5,
    max_in_flight_requests_per_connection=5
)
```

### Producer with JSON Serialization

```python
import json
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    value_serializer=lambda v: json.dumps(v).encode('utf-8')
)
```

### Producer with Key and Value Serialization

```python
import json
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    key_serializer=lambda k: k.encode('utf-8'),
    value_serializer=lambda v: json.dumps(v).encode('utf-8')
)
```

### Producer with Compression

```python
from kafka import KafkaProducer

# GZIP compression
producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    compression_type='gzip'
)

# Snappy compression
producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    compression_type='snappy'
)

# LZ4 compression
producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    compression_type='lz4'
)

# ZSTD compression
producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    compression_type='zstd'
)
```

### Producer Configuration Options

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    client_id='my-app',
    acks='all',                              # 0, 1, or 'all'
    compression_type='gzip',                 # None, 'gzip', 'snappy', 'lz4', 'zstd'
    retries=5,                               # Number of retries
    batch_size=16384,                        # Batch size in bytes
    linger_ms=10,                            # Wait time before sending
    buffer_memory=33554432,                  # Total memory for buffering
    max_block_ms=60000,                      # Max blocking time for send
    max_request_size=1048576,                # Max request size
    request_timeout_ms=30000,                # Request timeout
    connections_max_idle_ms=540000,          # Close idle connections after
    retry_backoff_ms=100,                    # Retry backoff time
    max_in_flight_requests_per_connection=5, # Max concurrent requests
    enable_idempotence=False,                # Idempotent producer
    metadata_max_age_ms=300000               # Metadata refresh interval
)
```

### Sending Messages - Basic

```python
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

# Send raw bytes
future = producer.send('my-topic', b'Hello Kafka')

# Wait for send to complete
record_metadata = future.get(timeout=10)
print(f'Sent to partition {record_metadata.partition} at offset {record_metadata.offset}')
```

### Sending Messages with Key

```python
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

future = producer.send('my-topic', key=b'user-123', value=b'User logged in')
record_metadata = future.get(timeout=10)
```

### Sending Messages with Partition

```python
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

# Send to specific partition
future = producer.send('my-topic', value=b'Hello', partition=0)
record_metadata = future.get(timeout=10)
```

### Sending Messages with Timestamp

```python
import time
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

timestamp_ms = int(time.time() * 1000)
future = producer.send('my-topic', value=b'Hello', timestamp_ms=timestamp_ms)
record_metadata = future.get(timeout=10)
```

### Sending Messages with Headers

```python
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

headers = [
    ('correlation-id', b'12345'),
    ('user-id', b'user-123')
]

future = producer.send('my-topic', value=b'Hello', headers=headers)
record_metadata = future.get(timeout=10)
```

### Sending JSON Messages

```python
import json
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers='localhost:9092',
    value_serializer=lambda v: json.dumps(v).encode('utf-8')
)

data = {
    'user_id': 'user-123',
    'action': 'login',
    'timestamp': 1234567890
}

future = producer.send('user-events', data)
record_metadata = future.get(timeout=10)
```

### Sending with Key and Value Serialization

```python
import json
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers='localhost:9092',
    key_serializer=lambda k: k.encode('utf-8'),
    value_serializer=lambda v: json.dumps(v).encode('utf-8')
)

future = producer.send('my-topic', key='user-123', value={'action': 'login'})
record_metadata = future.get(timeout=10)
```

### Async Send with Callback

```python
from kafka import KafkaProducer
from kafka.errors import KafkaError

producer = KafkaProducer(bootstrap_servers='localhost:9092')

def on_send_success(record_metadata):
    print(f'Sent to partition {record_metadata.partition} at offset {record_metadata.offset}')

def on_send_error(excp):
    print(f'Error: {excp}')

future = producer.send('my-topic', b'Hello Kafka')
future.add_callback(on_send_success)
future.add_errback(on_send_error)

# Continue without blocking
# Use flush() to wait for all messages
producer.flush()
```

### Flush Messages

```python
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

producer.send('my-topic', b'Message 1')
producer.send('my-topic', b'Message 2')
producer.send('my-topic', b'Message 3')

# Wait for all messages to be sent
producer.flush()
```

### Close Producer

```python
from kafka import KafkaProducer

producer = KafkaProducer(bootstrap_servers='localhost:9092')

producer.send('my-topic', b'Hello')
producer.flush()
producer.close()
```

### Error Handling

```python
from kafka import KafkaProducer
from kafka.errors import KafkaError, KafkaTimeoutError

producer = KafkaProducer(bootstrap_servers='localhost:9092')

try:
    future = producer.send('my-topic', b'Hello Kafka')
    record_metadata = future.get(timeout=10)
    print(f'Success: partition={record_metadata.partition}, offset={record_metadata.offset}')
except KafkaTimeoutError:
    print('Request timed out')
except KafkaError as e:
    print(f'Kafka error: {e}')
except Exception as e:
    print(f'Unexpected error: {e}')
finally:
    producer.close()
```

## Consumer

### Creating a Consumer

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)
```

### Consumer with Multiple Topics

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'topic-a', 'topic-b', 'topic-c',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)
```

### Consumer with Pattern Subscription

```python
import re
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

# Subscribe to topics matching pattern
consumer.subscribe(pattern='^awesome.*')
```

### Consumer Configuration Options

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers=['localhost:9092'],
    client_id='my-consumer',
    group_id='my-group',
    auto_offset_reset='earliest',           # 'earliest' or 'latest'
    enable_auto_commit=True,                # Auto commit offsets
    auto_commit_interval_ms=5000,           # Auto commit interval
    max_poll_records=500,                   # Max records per poll
    max_poll_interval_ms=300000,            # Max poll interval
    session_timeout_ms=10000,               # Session timeout
    heartbeat_interval_ms=3000,             # Heartbeat interval
    fetch_min_bytes=1,                      # Min fetch bytes
    fetch_max_bytes=52428800,               # Max fetch bytes
    fetch_max_wait_ms=500,                  # Max fetch wait time
    max_partition_fetch_bytes=1048576,      # Max bytes per partition
    request_timeout_ms=305000,              # Request timeout
    connections_max_idle_ms=540000,         # Close idle connections
    consumer_timeout_ms=None                # Stop iteration timeout
)
```

### Consumer with JSON Deserialization

```python
import json
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    value_deserializer=lambda m: json.loads(m.decode('utf-8'))
)
```

### Consumer with Key and Value Deserialization

```python
import json
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    key_deserializer=lambda k: k.decode('utf-8') if k else None,
    value_deserializer=lambda v: json.loads(v.decode('utf-8'))
)
```

### Consuming Messages - Basic Iteration

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

for message in consumer:
    print(f'Topic: {message.topic}')
    print(f'Partition: {message.partition}')
    print(f'Offset: {message.offset}')
    print(f'Key: {message.key}')
    print(f'Value: {message.value}')
    print(f'Timestamp: {message.timestamp}')
    print(f'Headers: {message.headers}')
```

### Consuming from Beginning

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    auto_offset_reset='earliest',
    enable_auto_commit=False
)

for message in consumer:
    print(message.value)
```

### Consuming with Timeout

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    consumer_timeout_ms=1000  # Stop iteration after 1 second of no messages
)

for message in consumer:
    print(message.value)

print('No more messages')
```

### Consuming JSON Messages

```python
import json
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'user-events',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    value_deserializer=lambda m: json.loads(m.decode('utf-8'))
)

for message in consumer:
    data = message.value
    print(f"User: {data['user_id']}, Action: {data['action']}")
```

### Consuming with Headers

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

for message in consumer:
    # Headers is a list of tuples: [(key, value), ...]
    headers_dict = {k: v.decode('utf-8') for k, v in message.headers} if message.headers else {}

    correlation_id = headers_dict.get('correlation-id')
    user_id = headers_dict.get('user-id')

    print(f'Correlation ID: {correlation_id}')
    print(f'User ID: {user_id}')
    print(f'Message: {message.value}')
```

### Batch Consumption with poll()

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    enable_auto_commit=False
)

while True:
    messages = consumer.poll(timeout_ms=1000, max_records=100)

    for topic_partition, records in messages.items():
        print(f'Received {len(records)} messages from {topic_partition}')

        for record in records:
            print(f'Offset: {record.offset}, Value: {record.value}')

    # Commit offsets after processing
    consumer.commit()
```

### Manual Partition Assignment

```python
from kafka import KafkaConsumer, TopicPartition

consumer = KafkaConsumer(bootstrap_servers='localhost:9092')

# Manually assign partitions
partition = TopicPartition('my-topic', 0)
consumer.assign([partition])

for message in consumer:
    print(message.value)
```

### Manual Offset Management

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    enable_auto_commit=False
)

for message in consumer:
    process_message(message)

    # Manual commit
    consumer.commit()
```

### Commit Specific Offsets

```python
from kafka import KafkaConsumer, TopicPartition, OffsetAndMetadata

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    enable_auto_commit=False
)

for message in consumer:
    process_message(message)

    # Commit specific offset
    tp = TopicPartition(message.topic, message.partition)
    offsets = {tp: OffsetAndMetadata(message.offset + 1, None)}
    consumer.commit(offsets=offsets)
```

### Async Commit

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    enable_auto_commit=False
)

def on_commit_complete(offsets, response):
    if isinstance(response, Exception):
        print(f'Commit failed: {response}')
    else:
        print(f'Commit successful: {offsets}')

for message in consumer:
    process_message(message)
    consumer.commit_async(callback=on_commit_complete)
```

### Seek to Offset

```python
from kafka import KafkaConsumer, TopicPartition

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

# Seek to specific offset
partition = TopicPartition('my-topic', 0)
consumer.seek(partition, 100)

for message in consumer:
    print(f'Offset: {message.offset}')
```

### Seek to Beginning

```python
from kafka import KafkaConsumer, TopicPartition

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

# Seek all assigned partitions to beginning
consumer.seek_to_beginning()

for message in consumer:
    print(message.value)
```

### Seek to End

```python
from kafka import KafkaConsumer, TopicPartition

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

# Seek all assigned partitions to end
consumer.seek_to_end()

for message in consumer:
    print(message.value)
```

### Get Offset Information

```python
from kafka import KafkaConsumer, TopicPartition

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

partition = TopicPartition('my-topic', 0)

# Get current position
position = consumer.position(partition)
print(f'Current position: {position}')

# Get committed offset
committed = consumer.committed(partition)
print(f'Committed offset: {committed}')

# Get beginning offset
beginning = consumer.beginning_offsets([partition])
print(f'Beginning offset: {beginning}')

# Get end offset
end = consumer.end_offsets([partition])
print(f'End offset: {end}')
```

### Pause and Resume

```python
from kafka import KafkaConsumer, TopicPartition

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

partition = TopicPartition('my-topic', 0)

# Pause consumption
consumer.pause(partition)

# Check paused partitions
paused = consumer.paused()
print(f'Paused partitions: {paused}')

# Resume consumption
consumer.resume(partition)
```

### Get Topic and Partition Information

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(bootstrap_servers='localhost:9092')

# List all topics
topics = consumer.topics()
print(f'Topics: {topics}')

# Get partitions for a topic
partitions = consumer.partitions_for_topic('my-topic')
print(f'Partitions: {partitions}')
```

### Close Consumer

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group'
)

# Process some messages
for message in consumer:
    if should_stop:
        break

# Close consumer
consumer.close()
```

### Close with Auto Commit

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='localhost:9092',
    group_id='my-group',
    enable_auto_commit=False
)

try:
    for message in consumer:
        process_message(message)
finally:
    # Close and commit offsets
    consumer.close(autocommit=True)
```

## Admin Operations

### Creating Admin Client

```python
from kafka import KafkaAdminClient

admin = KafkaAdminClient(
    bootstrap_servers='localhost:9092',
    client_id='admin-client'
)
```

### List Topics

```python
from kafka import KafkaAdminClient

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

topics = admin.list_topics()
print(f'Topics: {topics}')
```

### Create Topics

```python
from kafka import KafkaAdminClient
from kafka.admin import NewTopic

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

topic = NewTopic(
    name='my-new-topic',
    num_partitions=3,
    replication_factor=1
)

admin.create_topics([topic])
```

### Create Topics with Configuration

```python
from kafka import KafkaAdminClient
from kafka.admin import NewTopic, ConfigResource, ConfigResourceType

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

topic = NewTopic(
    name='my-new-topic',
    num_partitions=3,
    replication_factor=1,
    topic_configs={
        'retention.ms': '86400000',
        'cleanup.policy': 'delete'
    }
)

admin.create_topics([topic])
```

### Delete Topics

```python
from kafka import KafkaAdminClient

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

admin.delete_topics(['topic-to-delete'])
```

### Describe Topics

```python
from kafka import KafkaAdminClient

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

metadata = admin.describe_topics(['my-topic'])
print(metadata)
```

### Create Partitions

```python
from kafka import KafkaAdminClient
from kafka.admin import NewPartitions

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

topic_partitions = {
    'my-topic': NewPartitions(total_count=5)
}

admin.create_partitions(topic_partitions)
```

### Describe Consumer Groups

```python
from kafka import KafkaAdminClient

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

groups = admin.describe_consumer_groups(['my-group'])
print(groups)
```

### List Consumer Groups

```python
from kafka import KafkaAdminClient

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

groups = admin.list_consumer_groups()
print(groups)
```

### List Consumer Group Offsets

```python
from kafka import KafkaAdminClient
from kafka.admin import ConsumerGroupOffsets

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

offsets = admin.list_consumer_group_offsets('my-group')
print(offsets)
```

### Delete Consumer Groups

```python
from kafka import KafkaAdminClient

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

admin.delete_consumer_groups(['my-group'])
```

### Describe Configs

```python
from kafka import KafkaAdminClient
from kafka.admin import ConfigResource, ConfigResourceType

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

resource = ConfigResource(ConfigResourceType.TOPIC, 'my-topic')
configs = admin.describe_configs([resource])
print(configs)
```

### Alter Configs

```python
from kafka import KafkaAdminClient
from kafka.admin import ConfigResource, ConfigResourceType

admin = KafkaAdminClient(bootstrap_servers='localhost:9092')

resource = ConfigResource(
    ConfigResourceType.TOPIC,
    'my-topic',
    configs={'retention.ms': '604800000'}
)

admin.alter_configs([resource])
```

## Security and Authentication

### SSL Configuration

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers='kafka:9093',
    security_protocol='SSL',
    ssl_check_hostname=True,
    ssl_cafile='/path/to/ca-cert',
    ssl_certfile='/path/to/client-cert',
    ssl_keyfile='/path/to/client-key'
)
```

### SSL Consumer

```python
from kafka import KafkaConsumer

consumer = KafkaConsumer(
    'my-topic',
    bootstrap_servers='kafka:9093',
    group_id='my-group',
    security_protocol='SSL',
    ssl_check_hostname=True,
    ssl_cafile='/path/to/ca-cert',
    ssl_certfile='/path/to/client-cert',
    ssl_keyfile='/path/to/client-key'
)
```

### SASL PLAIN Authentication

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers='kafka:9092',
    security_protocol='SASL_PLAINTEXT',
    sasl_mechanism='PLAIN',
    sasl_plain_username='username',
    sasl_plain_password='password'
)
```

### SASL SCRAM-SHA-256

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers='kafka:9092',
    security_protocol='SASL_SSL',
    sasl_mechanism='SCRAM-SHA-256',
    sasl_plain_username='username',
    sasl_plain_password='password',
    ssl_cafile='/path/to/ca-cert'
)
```

### SASL SCRAM-SHA-512

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers='kafka:9092',
    security_protocol='SASL_SSL',
    sasl_mechanism='SCRAM-SHA-512',
    sasl_plain_username='username',
    sasl_plain_password='password',
    ssl_cafile='/path/to/ca-cert'
)
```

### SSL with SASL

```python
from kafka import KafkaProducer

producer = KafkaProducer(
    bootstrap_servers='kafka:9093',
    security_protocol='SASL_SSL',
    sasl_mechanism='PLAIN',
    sasl_plain_username='username',
    sasl_plain_password='password',
    ssl_check_hostname=True,
    ssl_cafile='/path/to/ca-cert'
)
```

## Complete Examples

### Complete Producer Example

```python
import json
import time
from kafka import KafkaProducer
from kafka.errors import KafkaError

def create_producer():
    return KafkaProducer(
        bootstrap_servers=['localhost:9092'],
        client_id='order-producer',
        acks='all',
        retries=5,
        key_serializer=lambda k: k.encode('utf-8'),
        value_serializer=lambda v: json.dumps(v).encode('utf-8'),
        compression_type='gzip'
    )

def send_order(producer, order):
    try:
        future = producer.send(
            'orders',
            key=order['order_id'],
            value=order,
            headers=[
                ('correlation-id', str(time.time()).encode('utf-8')),
                ('source', b'order-service')
            ]
        )

        record_metadata = future.get(timeout=10)
        print(f"Order {order['order_id']} sent successfully")
        print(f"  Partition: {record_metadata.partition}")
        print(f"  Offset: {record_metadata.offset}")
        return True

    except KafkaError as e:
        print(f"Failed to send order {order['order_id']}: {e}")
        return False

def main():
    producer = create_producer()

    try:
        orders = [
            {'order_id': 'order-001', 'amount': 99.99, 'customer': 'customer-123'},
            {'order_id': 'order-002', 'amount': 149.99, 'customer': 'customer-456'},
            {'order_id': 'order-003', 'amount': 79.99, 'customer': 'customer-789'}
        ]

        for order in orders:
            send_order(producer, order)
            time.sleep(0.1)

        # Wait for all messages to be sent
        producer.flush()
        print("All orders sent successfully")

    except Exception as e:
        print(f"Error: {e}")
    finally:
        producer.close()

if __name__ == '__main__':
    main()
```

### Complete Consumer Example

```python
import json
import signal
import sys
from kafka import KafkaConsumer
from kafka.errors import KafkaError

class OrderConsumer:
    def __init__(self):
        self.consumer = KafkaConsumer(
            'orders',
            bootstrap_servers=['localhost:9092'],
            group_id='order-processor',
            client_id='order-consumer',
            auto_offset_reset='earliest',
            enable_auto_commit=False,
            max_poll_records=100,
            key_deserializer=lambda k: k.decode('utf-8') if k else None,
            value_deserializer=lambda v: json.loads(v.decode('utf-8'))
        )
        self.running = True

        # Handle graceful shutdown
        signal.signal(signal.SIGINT, self.shutdown)
        signal.signal(signal.SIGTERM, self.shutdown)

    def shutdown(self, signum, frame):
        print("\nShutting down consumer...")
        self.running = False

    def process_order(self, order):
        print(f"Processing order {order['order_id']}")
        print(f"  Customer: {order['customer']}")
        print(f"  Amount: ${order['amount']}")
        # Process the order here
        return True

    def run(self):
        print("Starting order consumer...")

        try:
            while self.running:
                messages = self.consumer.poll(timeout_ms=1000, max_records=10)

                for topic_partition, records in messages.items():
                    for message in records:
                        try:
                            # Get headers
                            headers = {k: v.decode('utf-8')
                                     for k, v in message.headers} if message.headers else {}

                            correlation_id = headers.get('correlation-id', 'unknown')

                            print(f"\nReceived message (correlation-id: {correlation_id})")
                            print(f"  Partition: {message.partition}")
                            print(f"  Offset: {message.offset}")

                            # Process the order
                            if self.process_order(message.value):
                                print(f"  Order {message.value['order_id']} processed successfully")

                        except Exception as e:
                            print(f"Error processing message: {e}")

                    # Commit after processing batch
                    self.consumer.commit()

        except KafkaError as e:
            print(f"Kafka error: {e}")
        finally:
            self.consumer.close()
            print("Consumer closed")

if __name__ == '__main__':
    consumer = OrderConsumer()
    consumer.run()
```

### Complete Admin Example

```python
from kafka import KafkaAdminClient
from kafka.admin import NewTopic, ConfigResource, ConfigResourceType
from kafka.errors import TopicAlreadyExistsError, KafkaError

def create_admin_client():
    return KafkaAdminClient(
        bootstrap_servers=['localhost:9092'],
        client_id='admin-client'
    )

def create_topic(admin, topic_name, num_partitions=3, replication_factor=1):
    topic = NewTopic(
        name=topic_name,
        num_partitions=num_partitions,
        replication_factor=replication_factor,
        topic_configs={
            'retention.ms': '86400000',  # 1 day
            'cleanup.policy': 'delete',
            'compression.type': 'gzip'
        }
    )

    try:
        admin.create_topics([topic], validate_only=False)
        print(f"Topic '{topic_name}' created successfully")
        return True
    except TopicAlreadyExistsError:
        print(f"Topic '{topic_name}' already exists")
        return False
    except KafkaError as e:
        print(f"Failed to create topic '{topic_name}': {e}")
        return False

def list_topics(admin):
    topics = admin.list_topics()
    print(f"Available topics: {topics}")
    return topics

def describe_topic(admin, topic_name):
    try:
        metadata = admin.describe_topics([topic_name])
        print(f"\nTopic '{topic_name}' details:")
        print(f"  Metadata: {metadata}")
        return metadata
    except KafkaError as e:
        print(f"Failed to describe topic '{topic_name}': {e}")
        return None

def delete_topic(admin, topic_name):
    try:
        admin.delete_topics([topic_name])
        print(f"Topic '{topic_name}' deleted successfully")
        return True
    except KafkaError as e:
        print(f"Failed to delete topic '{topic_name}': {e}")
        return False

def list_consumer_groups(admin):
    try:
        groups = admin.list_consumer_groups()
        print(f"\nConsumer groups:")
        for group in groups:
            print(f"  - {group}")
        return groups
    except KafkaError as e:
        print(f"Failed to list consumer groups: {e}")
        return []

def describe_consumer_group(admin, group_id):
    try:
        groups = admin.describe_consumer_groups([group_id])
        print(f"\nConsumer group '{group_id}' details:")
        for group in groups:
            print(f"  {group}")
        return groups
    except KafkaError as e:
        print(f"Failed to describe consumer group '{group_id}': {e}")
        return None

def main():
    admin = create_admin_client()

    try:
        # List existing topics
        print("=== Listing Topics ===")
        list_topics(admin)

        # Create new topic
        print("\n=== Creating Topic ===")
        create_topic(admin, 'orders', num_partitions=3, replication_factor=1)

        # Describe topic
        print("\n=== Describing Topic ===")
        describe_topic(admin, 'orders')

        # List consumer groups
        print("\n=== Listing Consumer Groups ===")
        list_consumer_groups(admin)

    except Exception as e:
        print(f"Error: {e}")
    finally:
        admin.close()

if __name__ == '__main__':
    main()
```

### Error Handling Example

```python
import json
import time
from kafka import KafkaProducer, KafkaConsumer
from kafka.errors import (
    KafkaError,
    KafkaTimeoutError,
    NoBrokersAvailable,
    MessageSizeTooLargeError,
    UnknownTopicOrPartitionError
)

def resilient_producer():
    max_retries = 3
    retry_count = 0

    while retry_count < max_retries:
        try:
            producer = KafkaProducer(
                bootstrap_servers=['localhost:9092'],
                value_serializer=lambda v: json.dumps(v).encode('utf-8'),
                acks='all',
                retries=5,
                max_in_flight_requests_per_connection=1
            )

            data = {'message': 'Hello Kafka', 'timestamp': time.time()}

            future = producer.send('my-topic', data)
            record_metadata = future.get(timeout=10)

            print(f"Message sent successfully to partition {record_metadata.partition}")
            producer.close()
            return True

        except NoBrokersAvailable:
            print("No brokers available. Retrying...")
            retry_count += 1
            time.sleep(2)

        except KafkaTimeoutError:
            print("Request timed out. Retrying...")
            retry_count += 1
            time.sleep(2)

        except MessageSizeTooLargeError:
            print("Message too large. Cannot retry.")
            return False

        except UnknownTopicOrPartitionError:
            print("Topic does not exist. Cannot retry.")
            return False

        except KafkaError as e:
            print(f"Kafka error: {e}")
            retry_count += 1
            time.sleep(2)

    print("Max retries exceeded")
    return False

if __name__ == '__main__':
    resilient_producer()
```

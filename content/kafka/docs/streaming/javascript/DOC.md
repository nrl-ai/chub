---
name: streaming
description: "KafkaJS - Apache Kafka client for Node.js streaming and messaging"
metadata:
  languages: "javascript"
  versions: "2.2.4"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "kafka,streaming,messaging,queue,events"
---

# KafkaJS - Apache Kafka Client for Node.js

## Golden Rule

**ALWAYS use `kafkajs` version 2.2.4 or later.**

```bash
npm install kafkajs
```

**DO NOT use:**
- `kafka-node` (deprecated)
- `no-kafka` (deprecated)
- `node-rdkafka` (different use case - C++ binding)

KafkaJS is the modern, pure JavaScript Apache Kafka client for Node.js. It provides a complete implementation of the Kafka protocol with support for producers, consumers, admin operations, and transactions.

## Installation

### Basic Installation

```bash
npm install kafkajs
```

### With TypeScript

```bash
npm install kafkajs
npm install --save-dev @types/node
```

### Environment Variables

Create a `.env` file:

```bash
KAFKA_BROKERS=localhost:9092
KAFKA_CLIENT_ID=my-app
KAFKA_USERNAME=your-username
KAFKA_PASSWORD=your-password
```

Load environment variables in your application:

```javascript
require('dotenv').config()

const brokers = process.env.KAFKA_BROKERS.split(',')
const clientId = process.env.KAFKA_CLIENT_ID
```

## Initialization

### Basic Client

```javascript
const { Kafka } = require('kafkajs')

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092']
})
```

### With Environment Variables

```javascript
const { Kafka } = require('kafkajs')

const kafka = new Kafka({
  clientId: process.env.KAFKA_CLIENT_ID || 'my-app',
  brokers: process.env.KAFKA_BROKERS.split(',') || ['localhost:9092']
})
```

### With SSL/TLS

```javascript
const fs = require('fs')
const { Kafka } = require('kafkajs')

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9093', 'kafka2:9093'],
  ssl: {
    rejectUnauthorized: false,
    ca: [fs.readFileSync('/path/to/ca-cert', 'utf-8')],
    key: fs.readFileSync('/path/to/client-key', 'utf-8'),
    cert: fs.readFileSync('/path/to/client-cert', 'utf-8')
  }
})
```

For basic SSL without custom certificates:

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9093'],
  ssl: true
})
```

### With SASL Authentication

#### PLAIN

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9092'],
  sasl: {
    mechanism: 'plain',
    username: process.env.KAFKA_USERNAME,
    password: process.env.KAFKA_PASSWORD
  }
})
```

#### SCRAM-SHA-256

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9092'],
  sasl: {
    mechanism: 'scram-sha-256',
    username: process.env.KAFKA_USERNAME,
    password: process.env.KAFKA_PASSWORD
  }
})
```

#### SCRAM-SHA-512

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9092'],
  sasl: {
    mechanism: 'scram-sha-512',
    username: process.env.KAFKA_USERNAME,
    password: process.env.KAFKA_PASSWORD
  }
})
```

#### AWS IAM

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9092'],
  sasl: {
    mechanism: 'aws',
    authorizationIdentity: 'AIDAIOSFODNN7EXAMPLE',
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
    sessionToken: process.env.AWS_SESSION_TOKEN // optional
  }
})
```

#### OAUTHBEARER

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9092'],
  sasl: {
    mechanism: 'oauthbearer',
    oauthBearerProvider: async () => {
      const token = await getOAuthToken() // Your token fetching logic
      return {
        value: token
      }
    }
  }
})
```

### With SSL and SASL Combined

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9093'],
  ssl: true,
  sasl: {
    mechanism: 'plain',
    username: process.env.KAFKA_USERNAME,
    password: process.env.KAFKA_PASSWORD
  }
})
```

### Connection Configuration Options

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['kafka1:9092', 'kafka2:9092'],
  connectionTimeout: 1000,        // Time in ms to wait for successful connection
  requestTimeout: 30000,          // Time in ms to wait for successful requests
  enforceRequestTimeout: true,    // Enforce request timeout
  retry: {
    initialRetryTime: 300,        // Initial retry time in ms
    retries: 5,                   // Max number of retries
    maxRetryTime: 30000,          // Max retry time in ms
    multiplier: 2,                // Exponential backoff multiplier
    factor: 0.2                   // Randomization factor
  }
})
```

### Dynamic Broker Discovery

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: async () => {
    // Fetch broker list dynamically
    const response = await fetch('http://service-discovery/kafka-brokers')
    const data = await response.json()
    return data.brokers
  }
})
```

## Producer

### Creating a Producer

```javascript
const producer = kafka.producer()
```

### Producer Configuration Options

```javascript
const producer = kafka.producer({
  allowAutoTopicCreation: true,   // Allow auto topic creation
  transactionTimeout: 60000,      // Transaction timeout in ms
  idempotent: false,              // Enable idempotent producer
  maxInFlightRequests: null,      // Max concurrent requests
  metadataMaxAge: 300000          // Metadata refresh interval in ms
})
```

### Connecting and Disconnecting

```javascript
await producer.connect()
// ... produce messages
await producer.disconnect()
```

### Sending Messages - Basic

```javascript
const producer = kafka.producer()
await producer.connect()

await producer.send({
  topic: 'my-topic',
  messages: [
    { value: 'Hello Kafka' }
  ]
})

await producer.disconnect()
```

### Sending Messages with Keys

```javascript
await producer.send({
  topic: 'user-events',
  messages: [
    { key: 'user-123', value: 'User logged in' },
    { key: 'user-456', value: 'User logged out' }
  ]
})
```

### Sending Messages with Headers

```javascript
await producer.send({
  topic: 'my-topic',
  messages: [
    {
      key: 'key1',
      value: 'Hello World',
      headers: {
        'correlation-id': '2bfb68bb-893a-423b-a7fa-7b568cad5b67',
        'user-id': 'user-123'
      }
    }
  ]
})
```

### Sending to Specific Partition

```javascript
await producer.send({
  topic: 'my-topic',
  messages: [
    {
      partition: 0,
      value: 'Message to partition 0'
    }
  ]
})
```

### Sending with Timestamp

```javascript
await producer.send({
  topic: 'my-topic',
  messages: [
    {
      value: 'Hello Kafka',
      timestamp: Date.now()
    }
  ]
})
```

### Sending Multiple Messages

```javascript
const messages = [
  { key: 'key1', value: 'Message 1' },
  { key: 'key2', value: 'Message 2' },
  { key: 'key3', value: 'Message 3' }
]

await producer.send({
  topic: 'my-topic',
  messages: messages
})
```

### Send with Acknowledgment Control

```javascript
// Wait for all in-sync replicas
await producer.send({
  topic: 'my-topic',
  messages: [{ value: 'Important message' }],
  acks: -1  // -1 = all replicas, 0 = no acks, 1 = leader only
})

// No acknowledgment
await producer.send({
  topic: 'my-topic',
  messages: [{ value: 'Fire and forget' }],
  acks: 0
})

// Leader acknowledgment only
await producer.send({
  topic: 'my-topic',
  messages: [{ value: 'Quick send' }],
  acks: 1
})
```

### Send with Timeout

```javascript
await producer.send({
  topic: 'my-topic',
  messages: [{ value: 'Hello' }],
  timeout: 5000  // Timeout in milliseconds
})
```

### Send with Compression

```javascript
const { CompressionTypes } = require('kafkajs')

await producer.send({
  topic: 'my-topic',
  compression: CompressionTypes.GZIP,
  messages: [
    { value: 'Compressed message 1' },
    { value: 'Compressed message 2' }
  ]
})
```

Available compression types:
- `CompressionTypes.None`
- `CompressionTypes.GZIP`
- `CompressionTypes.Snappy` (requires `kafkajs-snappy` package)
- `CompressionTypes.LZ4` (requires `kafkajs-lz4` package)
- `CompressionTypes.ZSTD` (requires `@kafkajs/zstd` package)

### Batch Sending to Multiple Topics

```javascript
await producer.sendBatch({
  topicMessages: [
    {
      topic: 'topic-a',
      messages: [
        { key: 'key1', value: 'Message for topic A' }
      ]
    },
    {
      topic: 'topic-b',
      messages: [
        { key: 'key2', value: 'Message for topic B' }
      ]
    }
  ]
})
```

### Batch Sending with Compression

```javascript
const { CompressionTypes } = require('kafkajs')

await producer.sendBatch({
  compression: CompressionTypes.GZIP,
  topicMessages: [
    {
      topic: 'topic-a',
      messages: [{ value: 'Message A' }]
    },
    {
      topic: 'topic-b',
      messages: [{ value: 'Message B' }]
    }
  ]
})
```

### Custom Partitioner

```javascript
const MyPartitioner = () => {
  return ({ topic, partitionMetadata, message }) => {
    // Custom partitioning logic
    const numPartitions = partitionMetadata.length
    const partition = Math.abs(hashCode(message.key)) % numPartitions
    return partition
  }
}

const producer = kafka.producer({
  createPartitioner: MyPartitioner
})

function hashCode(str) {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash
  }
  return hash
}
```

### Using Legacy Partitioner

```javascript
const { Partitioners } = require('kafkajs')

const producer = kafka.producer({
  createPartitioner: Partitioners.LegacyPartitioner
})
```

### Sending JSON Data

```javascript
await producer.send({
  topic: 'user-events',
  messages: [
    {
      key: 'user-123',
      value: JSON.stringify({
        userId: 'user-123',
        action: 'login',
        timestamp: Date.now()
      })
    }
  ]
})
```

### Error Handling

```javascript
try {
  await producer.send({
    topic: 'my-topic',
    messages: [{ value: 'Hello Kafka' }]
  })
} catch (error) {
  console.error('Error sending message:', error)
  // Handle specific errors
  if (error.type === 'TOPIC_AUTHORIZATION_FAILED') {
    console.error('Not authorized to produce to topic')
  }
}
```

## Consumer

### Creating a Consumer

```javascript
const consumer = kafka.consumer({ groupId: 'my-group' })
```

### Consumer Configuration Options

```javascript
const consumer = kafka.consumer({
  groupId: 'my-group',
  sessionTimeout: 30000,          // Session timeout in ms
  heartbeatInterval: 3000,        // Heartbeat interval in ms
  maxBytesPerPartition: 1048576,  // Max bytes per partition (1MB)
  maxBytes: 10485760,             // Max bytes per fetch (10MB)
  maxWaitTimeInMs: 5000,          // Max wait time for fetch
  retry: {
    retries: 5
  },
  allowAutoTopicCreation: true,   // Allow auto topic creation
  maxInFlightRequests: null,      // Max concurrent requests
  readUncommitted: false,         // Read uncommitted messages
  rackId: null                    // Enable follower fetching
})
```

### Connecting and Subscribing

```javascript
const consumer = kafka.consumer({ groupId: 'my-group' })
await consumer.connect()
await consumer.subscribe({ topics: ['my-topic'] })
```

### Subscribe to Multiple Topics

```javascript
await consumer.subscribe({ topics: ['topic-a', 'topic-b', 'topic-c'] })
```

### Subscribe with Regex Pattern

```javascript
await consumer.subscribe({ topics: [/topic-(eu|us)-.*/i] })
```

### Subscribe from Beginning

```javascript
await consumer.subscribe({
  topics: ['my-topic'],
  fromBeginning: true
})
```

### Consuming Messages - eachMessage

```javascript
await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    console.log({
      topic: topic,
      partition: partition,
      offset: message.offset,
      value: message.value.toString(),
      key: message.key?.toString(),
      headers: message.headers
    })
  }
})
```

### Consuming with Heartbeat

```javascript
await consumer.run({
  eachMessage: async ({ topic, partition, message, heartbeat }) => {
    // For long-running processing
    await processMessage(message)
    await heartbeat()
  }
})
```

### Consuming with Pause/Resume

```javascript
await consumer.run({
  eachMessage: async ({ topic, partition, message, pause }) => {
    try {
      await processMessage(message)
    } catch (error) {
      if (error.retryable) {
        // Pause for 30 seconds on retryable error
        const pausedConsumer = pause()
        setTimeout(() => pausedConsumer.resume(), 30000)
      }
    }
  }
})
```

### Consuming JSON Messages

```javascript
await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    const data = JSON.parse(message.value.toString())
    console.log('Received:', data)
  }
})
```

### Consuming Messages with Headers

```javascript
await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    const correlationId = message.headers['correlation-id']?.toString()
    const userId = message.headers['user-id']?.toString()

    console.log('Correlation ID:', correlationId)
    console.log('User ID:', userId)
    console.log('Message:', message.value.toString())
  }
})
```

### Batch Processing - eachBatch

```javascript
await consumer.run({
  eachBatch: async ({ batch, resolveOffset, heartbeat, isRunning, isStale }) => {
    for (let message of batch.messages) {
      if (!isRunning() || isStale()) break

      console.log({
        topic: batch.topic,
        partition: batch.partition,
        offset: message.offset,
        value: message.value.toString()
      })

      resolveOffset(message.offset)
      await heartbeat()
    }
  }
})
```

### Batch Processing with Manual Commit

```javascript
await consumer.run({
  autoCommitInterval: null,
  autoCommitThreshold: null,
  eachBatch: async ({ batch, resolveOffset, commitOffsetsIfNecessary, heartbeat }) => {
    for (let message of batch.messages) {
      await processMessage(message)
      resolveOffset(message.offset)
    }

    await commitOffsetsIfNecessary()
    await heartbeat()
  }
})
```

### Batch Processing with Auto-Resolve

```javascript
await consumer.run({
  eachBatchAutoResolve: true,
  eachBatch: async ({ batch, heartbeat }) => {
    console.log(`Received batch of ${batch.messages.length} messages`)

    for (let message of batch.messages) {
      await processMessage(message)
    }

    await heartbeat()
  }
})
```

### Concurrent Processing by Partition

```javascript
await consumer.run({
  partitionsConsumedConcurrently: 3,
  eachMessage: async ({ topic, partition, message }) => {
    // This will process up to 3 partitions concurrently
    // Messages within the same partition are still processed sequentially
    await processMessage(message)
  }
})
```

### Manual Offset Management

```javascript
const consumer = kafka.consumer({
  groupId: 'my-group',
  autoCommit: false
})

await consumer.connect()
await consumer.subscribe({ topics: ['my-topic'] })

await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    await processMessage(message)

    // Manual commit
    await consumer.commitOffsets([
      {
        topic: topic,
        partition: partition,
        offset: (parseInt(message.offset) + 1).toString()
      }
    ])
  }
})
```

### Auto Commit Configuration

```javascript
await consumer.run({
  autoCommit: true,
  autoCommitInterval: 5000,      // Commit every 5 seconds
  autoCommitThreshold: 100,      // Or after 100 messages
  eachMessage: async ({ topic, partition, message }) => {
    await processMessage(message)
  }
})
```

### Seek to Offset

```javascript
// Seek before running consumer
await consumer.subscribe({ topics: ['my-topic'] })

consumer.on(consumer.events.GROUP_JOIN, async ({ payload }) => {
  // Seek to specific offset
  consumer.seek({ topic: 'my-topic', partition: 0, offset: 12345 })
})

await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    console.log(`Offset: ${message.offset}`)
  }
})
```

### Seek to Beginning

```javascript
consumer.on(consumer.events.GROUP_JOIN, async ({ payload }) => {
  const { groupId } = payload
  console.log(`Consumer ${groupId} joined`)

  // Seek all partitions to beginning
  const assignments = consumer.assignment()
  for (const assignment of assignments) {
    consumer.seek({
      topic: assignment.topic,
      partition: assignment.partition,
      offset: '0'
    })
  }
})
```

### Pause and Resume Consumption

```javascript
// Pause consumption
consumer.pause([{ topic: 'my-topic', partitions: [0, 1] }])

// Resume after some time
setTimeout(() => {
  consumer.resume([{ topic: 'my-topic', partitions: [0, 1] }])
}, 60000)
```

### Error Handling

```javascript
await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    try {
      await processMessage(message)
    } catch (error) {
      console.error('Error processing message:', error)
      // Optionally: send to dead letter queue
      await sendToDeadLetterQueue(topic, message, error)
    }
  }
})
```

### Graceful Shutdown

```javascript
const consumer = kafka.consumer({ groupId: 'my-group' })

const shutdown = async () => {
  console.log('Shutting down consumer...')
  await consumer.disconnect()
  process.exit(0)
}

process.on('SIGINT', shutdown)
process.on('SIGTERM', shutdown)

await consumer.connect()
await consumer.subscribe({ topics: ['my-topic'] })

await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    await processMessage(message)
  }
})
```

## Admin Operations

### Creating Admin Client

```javascript
const admin = kafka.admin()
await admin.connect()
```

### List Topics

```javascript
const topics = await admin.listTopics()
console.log('Topics:', topics)
```

### Create Topic

```javascript
await admin.createTopics({
  topics: [
    {
      topic: 'my-topic',
      numPartitions: 3,
      replicationFactor: 2
    }
  ]
})
```

### Create Topic with Configuration

```javascript
await admin.createTopics({
  topics: [
    {
      topic: 'my-topic',
      numPartitions: 3,
      replicationFactor: 2,
      configEntries: [
        { name: 'cleanup.policy', value: 'compact' },
        { name: 'retention.ms', value: '86400000' }
      ]
    }
  ]
})
```

### Create Multiple Topics

```javascript
await admin.createTopics({
  topics: [
    { topic: 'topic-a', numPartitions: 1 },
    { topic: 'topic-b', numPartitions: 2 },
    { topic: 'topic-c', numPartitions: 3 }
  ]
})
```

### Delete Topics

```javascript
await admin.deleteTopics({
  topics: ['topic-to-delete']
})
```

### Fetch Topic Metadata

```javascript
const metadata = await admin.fetchTopicMetadata({
  topics: ['my-topic']
})

console.log('Topic metadata:', metadata)
```

### Fetch Topic Offsets

```javascript
const offsets = await admin.fetchTopicOffsets('my-topic')

console.log('Topic offsets:', offsets)
// Output: [{ partition: 0, offset: '100', high: '100', low: '0' }]
```

### Fetch Offsets by Timestamp

```javascript
const timestamp = Date.now() - 3600000 // 1 hour ago

const offsets = await admin.fetchTopicOffsetsByTimestamp('my-topic', timestamp)
console.log('Offsets at timestamp:', offsets)
```

### Create Partitions

```javascript
await admin.createPartitions({
  topicPartitions: [
    {
      topic: 'my-topic',
      count: 5  // New total partition count
    }
  ]
})
```

### List Consumer Groups

```javascript
const groups = await admin.listGroups()
console.log('Consumer groups:', groups.groups)
```

### Describe Consumer Group

```javascript
const group = await admin.describeGroups(['my-group'])
console.log('Group details:', group)
```

### Fetch Consumer Group Offsets

```javascript
const offsets = await admin.fetchOffsets({
  groupId: 'my-group',
  topics: ['my-topic']
})

console.log('Consumer group offsets:', offsets)
```

### Reset Consumer Group Offsets

```javascript
// Reset to earliest
await admin.resetOffsets({
  groupId: 'my-group',
  topic: 'my-topic',
  earliest: true
})

// Reset to latest
await admin.resetOffsets({
  groupId: 'my-group',
  topic: 'my-topic'
})
```

### Set Consumer Group Offsets

```javascript
await admin.setOffsets({
  groupId: 'my-group',
  topic: 'my-topic',
  partitions: [
    { partition: 0, offset: '100' },
    { partition: 1, offset: '200' }
  ]
})
```

### Delete Consumer Group

```javascript
await admin.deleteGroups(['my-group'])
```

### Describe Cluster

```javascript
const cluster = await admin.describeCluster()

console.log('Cluster ID:', cluster.clusterId)
console.log('Controller:', cluster.controller)
console.log('Brokers:', cluster.brokers)
```

### Describe Configs

```javascript
const configs = await admin.describeConfigs({
  resources: [
    {
      type: 2, // TOPIC
      name: 'my-topic'
    }
  ]
})

console.log('Topic configs:', configs)
```

### Alter Configs

```javascript
await admin.alterConfigs({
  resources: [
    {
      type: 2, // TOPIC
      name: 'my-topic',
      configEntries: [
        { name: 'retention.ms', value: '604800000' }
      ]
    }
  ]
})
```

### Delete Topic Records

```javascript
// Delete records up to offset 100
await admin.deleteTopicRecords({
  topic: 'my-topic',
  partitions: [
    { partition: 0, offset: '100' }
  ]
})
```

### Disconnect Admin

```javascript
await admin.disconnect()
```

## Transactions

### Creating Transactional Producer

```javascript
const producer = kafka.producer({
  transactionalId: 'my-transactional-producer',
  maxInFlightRequests: 1,
  idempotent: true
})
```

### Basic Transaction

```javascript
await producer.connect()

const transaction = await producer.transaction()

try {
  await transaction.send({
    topic: 'my-topic',
    messages: [
      { value: 'Message 1' },
      { value: 'Message 2' }
    ]
  })

  await transaction.commit()
} catch (error) {
  await transaction.abort()
  throw error
}
```

### Transaction with Multiple Topics

```javascript
const transaction = await producer.transaction()

try {
  await transaction.send({
    topic: 'topic-a',
    messages: [{ value: 'Message A' }]
  })

  await transaction.send({
    topic: 'topic-b',
    messages: [{ value: 'Message B' }]
  })

  await transaction.commit()
} catch (error) {
  await transaction.abort()
}
```

### Transaction with sendOffsets

```javascript
const consumer = kafka.consumer({ groupId: 'my-group' })
await consumer.connect()
await consumer.subscribe({ topics: ['input-topic'] })

await consumer.run({
  eachMessage: async ({ topic, partition, message }) => {
    const transaction = await producer.transaction()

    try {
      // Process and send to output topic
      const outputMessage = processMessage(message)

      await transaction.send({
        topic: 'output-topic',
        messages: [{ value: outputMessage }]
      })

      // Commit consumer offset atomically
      await transaction.sendOffsets({
        consumerGroupId: 'my-group',
        topics: [
          {
            topic: topic,
            partitions: [
              {
                partition: partition,
                offset: (parseInt(message.offset) + 1).toString()
              }
            ]
          }
        ]
      })

      await transaction.commit()
    } catch (error) {
      await transaction.abort()
      throw error
    }
  }
})
```

## Logging

### Setting Log Level

```javascript
const { Kafka, logLevel } = require('kafkajs')

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
  logLevel: logLevel.ERROR
})
```

Available log levels:
- `logLevel.NOTHING`
- `logLevel.ERROR`
- `logLevel.WARN`
- `logLevel.INFO` (default)
- `logLevel.DEBUG`

### Custom Logger

```javascript
const { Kafka, logLevel } = require('kafkajs')

const customLogger = () => {
  return {
    info: (message) => console.log(`[INFO] ${message.message}`),
    error: (message) => console.error(`[ERROR] ${message.message}`),
    warn: (message) => console.warn(`[WARN] ${message.message}`),
    debug: (message) => console.debug(`[DEBUG] ${message.message}`)
  }
}

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
  logLevel: logLevel.INFO,
  logCreator: customLogger
})
```

### Change Log Level at Runtime

```javascript
kafka.logger().setLogLevel(logLevel.DEBUG)
```

### Environment Variable for Logging

```bash
export KAFKAJS_LOG_LEVEL=DEBUG
```

## Error Handling and Retries

### Producer Retry Configuration

```javascript
const producer = kafka.producer({
  retry: {
    initialRetryTime: 100,
    retries: 8,
    maxRetryTime: 30000,
    multiplier: 2,
    factor: 0.2
  }
})
```

### Consumer Retry Configuration

```javascript
const consumer = kafka.consumer({
  groupId: 'my-group',
  retry: {
    initialRetryTime: 100,
    retries: 8
  }
})
```

### Custom Restart on Failure

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
  retry: {
    retries: 5
  },
  restartOnFailure: async (error) => {
    console.error('Error occurred:', error)
    // Return true to restart consumer, false to stop
    return error.type !== 'FATAL_ERROR'
  }
})
```

## Advanced Configuration

### Custom Socket Factory

```javascript
const net = require('net')
const tls = require('tls')

const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
  socketFactory: ({ host, port, ssl, onConnect }) => {
    const socket = ssl
      ? tls.connect({ host, port, servername: host }, onConnect)
      : net.connect({ host, port }, onConnect)

    socket.setKeepAlive(true, 60000)
    return socket
  }
})
```

### Rack Awareness for Follower Fetching

```javascript
const consumer = kafka.consumer({
  groupId: 'my-group',
  rackId: 'us-east-1a'
})
```

### Authentication Timeout

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
  sasl: {
    mechanism: 'plain',
    username: process.env.KAFKA_USERNAME,
    password: process.env.KAFKA_PASSWORD
  },
  authenticationTimeout: 10000
})
```

### Reauthentication

```javascript
const kafka = new Kafka({
  clientId: 'my-app',
  brokers: ['localhost:9092'],
  sasl: {
    mechanism: 'oauthbearer',
    oauthBearerProvider: async () => ({ value: await getToken() })
  },
  reauthenticationThreshold: 10000
})
```

## Events

### Producer Events

```javascript
const producer = kafka.producer()

producer.on(producer.events.CONNECT, () => {
  console.log('Producer connected')
})

producer.on(producer.events.DISCONNECT, () => {
  console.log('Producer disconnected')
})

producer.on(producer.events.REQUEST_TIMEOUT, ({ payload }) => {
  console.log('Request timeout:', payload)
})
```

### Consumer Events

```javascript
const consumer = kafka.consumer({ groupId: 'my-group' })

consumer.on(consumer.events.GROUP_JOIN, ({ payload }) => {
  console.log('Consumer joined group:', payload)
})

consumer.on(consumer.events.CRASH, ({ payload }) => {
  console.error('Consumer crashed:', payload.error)
})

consumer.on(consumer.events.REBALANCING, ({ payload }) => {
  console.log('Consumer rebalancing')
})

consumer.on(consumer.events.STOP, () => {
  console.log('Consumer stopped')
})

consumer.on(consumer.events.CONNECT, () => {
  console.log('Consumer connected')
})

consumer.on(consumer.events.DISCONNECT, () => {
  console.log('Consumer disconnected')
})
```

## Complete Examples

### Complete Producer Example

```javascript
const { Kafka, CompressionTypes, logLevel } = require('kafkajs')

const kafka = new Kafka({
  clientId: 'order-service',
  brokers: [process.env.KAFKA_BROKER || 'localhost:9092'],
  logLevel: logLevel.INFO,
  retry: {
    retries: 5
  }
})

const producer = kafka.producer({
  allowAutoTopicCreation: false,
  idempotent: true
})

async function sendOrders() {
  await producer.connect()

  try {
    const orders = [
      { orderId: '123', amount: 99.99 },
      { orderId: '124', amount: 149.99 }
    ]

    for (const order of orders) {
      await producer.send({
        topic: 'orders',
        compression: CompressionTypes.GZIP,
        messages: [
          {
            key: order.orderId,
            value: JSON.stringify(order),
            headers: {
              'correlation-id': generateId(),
              'timestamp': Date.now().toString()
            }
          }
        ]
      })

      console.log(`Sent order ${order.orderId}`)
    }
  } catch (error) {
    console.error('Error sending orders:', error)
    throw error
  } finally {
    await producer.disconnect()
  }
}

sendOrders().catch(console.error)

function generateId() {
  return Math.random().toString(36).substring(7)
}
```

### Complete Consumer Example

```javascript
const { Kafka, logLevel } = require('kafkajs')

const kafka = new Kafka({
  clientId: 'order-processor',
  brokers: [process.env.KAFKA_BROKER || 'localhost:9092'],
  logLevel: logLevel.INFO
})

const consumer = kafka.consumer({
  groupId: 'order-processing-group',
  sessionTimeout: 30000,
  heartbeatInterval: 3000
})

async function processOrders() {
  await consumer.connect()
  await consumer.subscribe({
    topics: ['orders'],
    fromBeginning: true
  })

  await consumer.run({
    autoCommit: true,
    autoCommitInterval: 5000,
    eachMessage: async ({ topic, partition, message }) => {
      const order = JSON.parse(message.value.toString())
      const correlationId = message.headers['correlation-id']?.toString()

      console.log(`Processing order ${order.orderId}`, {
        partition,
        offset: message.offset,
        correlationId
      })

      try {
        await processOrder(order)
        console.log(`Order ${order.orderId} processed successfully`)
      } catch (error) {
        console.error(`Error processing order ${order.orderId}:`, error)
        // Send to DLQ or handle error
      }
    }
  })
}

async function processOrder(order) {
  // Simulate order processing
  await new Promise(resolve => setTimeout(resolve, 100))
}

const shutdown = async () => {
  console.log('Shutting down...')
  await consumer.disconnect()
  process.exit(0)
}

process.on('SIGINT', shutdown)
process.on('SIGTERM', shutdown)

processOrders().catch(console.error)
```

### Complete Transaction Example

```javascript
const { Kafka } = require('kafkajs')

const kafka = new Kafka({
  clientId: 'transaction-app',
  brokers: ['localhost:9092']
})

const producer = kafka.producer({
  transactionalId: 'my-transactional-id',
  maxInFlightRequests: 1,
  idempotent: true
})

const consumer = kafka.consumer({
  groupId: 'transaction-group'
})

async function runTransactionalProcessing() {
  await producer.connect()
  await consumer.connect()

  await consumer.subscribe({ topics: ['input-topic'] })

  await consumer.run({
    autoCommit: false,
    eachMessage: async ({ topic, partition, message }) => {
      const transaction = await producer.transaction()

      try {
        // Parse input message
        const input = JSON.parse(message.value.toString())

        // Process the message
        const result = {
          original: input,
          processed: true,
          timestamp: Date.now()
        }

        // Send to output topic
        await transaction.send({
          topic: 'output-topic',
          messages: [
            {
              key: message.key,
              value: JSON.stringify(result)
            }
          ]
        })

        // Commit offset atomically
        await transaction.sendOffsets({
          consumerGroupId: 'transaction-group',
          topics: [
            {
              topic: topic,
              partitions: [
                {
                  partition: partition,
                  offset: (parseInt(message.offset) + 1).toString()
                }
              ]
            }
          ]
        })

        await transaction.commit()
        console.log(`Processed message at offset ${message.offset}`)
      } catch (error) {
        console.error('Transaction failed:', error)
        await transaction.abort()
      }
    }
  })
}

runTransactionalProcessing().catch(console.error)
```

### Complete Admin Example

```javascript
const { Kafka } = require('kafkajs')

const kafka = new Kafka({
  clientId: 'admin-client',
  brokers: ['localhost:9092']
})

const admin = kafka.admin()

async function manageTopics() {
  await admin.connect()

  try {
    // List existing topics
    const existingTopics = await admin.listTopics()
    console.log('Existing topics:', existingTopics)

    // Create new topic
    const topicName = 'my-new-topic'

    if (!existingTopics.includes(topicName)) {
      await admin.createTopics({
        topics: [
          {
            topic: topicName,
            numPartitions: 3,
            replicationFactor: 1,
            configEntries: [
              { name: 'retention.ms', value: '86400000' },
              { name: 'cleanup.policy', value: 'delete' }
            ]
          }
        ]
      })
      console.log(`Topic ${topicName} created`)
    }

    // Fetch metadata
    const metadata = await admin.fetchTopicMetadata({
      topics: [topicName]
    })
    console.log('Topic metadata:', JSON.stringify(metadata, null, 2))

    // Fetch offsets
    const offsets = await admin.fetchTopicOffsets(topicName)
    console.log('Topic offsets:', offsets)

    // List consumer groups
    const groups = await admin.listGroups()
    console.log('Consumer groups:', groups.groups)

  } catch (error) {
    console.error('Error managing topics:', error)
  } finally {
    await admin.disconnect()
  }
}

manageTopics().catch(console.error)
```

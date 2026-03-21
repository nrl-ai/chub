---
name: message-queue
description: "RabbitMQ amqplib coding guidelines for JavaScript/Node.js message broker interactions"
metadata:
  languages: "javascript"
  versions: "0.10.9"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "rabbitmq,queue,amqp,messaging,async"
---

# RabbitMQ amqplib Coding Guidelines (JavaScript/Node.js)

You are a RabbitMQ amqplib coding expert. Help me with writing code using RabbitMQ message broker via the official amqplib library.

## Golden Rule: Use the Correct and Current SDK

Always use the official amqplib library for all RabbitMQ (AMQP 0-9-1) interactions in Node.js.

- **Library Name:** amqplib
- **NPM Package:** `amqplib`
- **Current Version:** 0.10.9 or higher
- **Minimum Required Version:** 0.10.7+ (for RabbitMQ 4.1.0+ compatibility)

**Installation:**

```bash
npm install amqplib
```

**IMPORTANT:** Do not use the deprecated `amqp` or `node-amqp` packages. Always use `amqplib`.

**Import Patterns:**

```javascript
// Callback API
const amqp = require('amqplib/callback_api');

// Promise/async-await API (recommended)
const amqp = require('amqplib');

// ES6 import
import amqp from 'amqplib';
```

## Initialization and Connection

### Environment Variables

Configure RabbitMQ connection using environment variables:

```javascript
// .env file
RABBITMQ_URL=amqp://username:password@localhost:5672
RABBITMQ_HOST=localhost
RABBITMQ_PORT=5672
RABBITMQ_USER=guest
RABBITMQ_PASS=guest
RABBITMQ_VHOST=/
```

### Basic Connection (Promise API)

```javascript
const amqp = require('amqplib');

async function connect() {
  try {
    // Using connection URL from environment
    const connection = await amqp.connect(process.env.RABBITMQ_URL || 'amqp://localhost');

    console.log('Connected to RabbitMQ');
    return connection;
  } catch (error) {
    console.error('Connection failed:', error);
    throw error;
  }
}
```

### Connection with Options

```javascript
const amqp = require('amqplib');

async function connectWithOptions() {
  const connection = await amqp.connect({
    protocol: 'amqp',
    hostname: process.env.RABBITMQ_HOST || 'localhost',
    port: parseInt(process.env.RABBITMQ_PORT) || 5672,
    username: process.env.RABBITMQ_USER || 'guest',
    password: process.env.RABBITMQ_PASS || 'guest',
    vhost: process.env.RABBITMQ_VHOST || '/',
    heartbeat: 30,  // Heartbeat interval in seconds (recommended: 30)
  });

  return connection;
}
```

### Connection with Socket Options

```javascript
const amqp = require('amqplib');

async function connectWithSocketOptions() {
  const connection = await amqp.connect(
    process.env.RABBITMQ_URL || 'amqp://localhost',
    {
      timeout: 10000,        // Socket timeout in milliseconds
      noDelay: true,         // Disable Nagle's algorithm
      keepAlive: true,       // Enable TCP keep-alive
      keepAliveDelay: 30000  // Keep-alive delay in ms
    }
  );

  return connection;
}
```

### Callback API Connection

```javascript
const amqp = require('amqplib/callback_api');

amqp.connect('amqp://localhost', function(error, connection) {
  if (error) {
    throw error;
  }
  console.log('Connected to RabbitMQ');

  // Use connection here
});
```

## Channel API

### Creating a Channel

```javascript
const amqp = require('amqplib');

async function createChannel() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  return { connection, channel };
}
```

### Creating a Confirm Channel

```javascript
const amqp = require('amqplib');

async function createConfirmChannel() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createConfirmChannel();

  // Confirm channel provides publisher confirms
  return { connection, channel };
}
```

## Core Messaging Patterns

### 1. Simple Queue - Producer

```javascript
const amqp = require('amqplib');

async function sendMessage(queueName, message) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  // Assert queue exists (create if it doesn't)
  await channel.assertQueue(queueName, {
    durable: true  // Queue survives broker restart
  });

  // Send message
  channel.sendToQueue(queueName, Buffer.from(message), {
    persistent: true  // Message survives broker restart
  });

  console.log(`Sent: ${message}`);

  // Close connection after a delay to ensure message is sent
  setTimeout(() => {
    connection.close();
  }, 500);
}

// Usage
sendMessage('task_queue', 'Hello World!');
```

### 2. Simple Queue - Consumer

```javascript
const amqp = require('amqplib');

async function receiveMessages(queueName) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue(queueName, {
    durable: true
  });

  console.log(`Waiting for messages in ${queueName}`);

  // Consume messages
  channel.consume(queueName, (msg) => {
    if (msg !== null) {
      const content = msg.content.toString();
      console.log(`Received: ${content}`);

      // Acknowledge message
      channel.ack(msg);
    }
  }, {
    noAck: false  // Manual acknowledgment
  });
}

// Usage
receiveMessages('task_queue');
```

### 3. Work Queue with Prefetch

```javascript
const amqp = require('amqplib');

async function worker(queueName) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue(queueName, {
    durable: true
  });

  // Fair dispatch - only send one message at a time to each worker
  channel.prefetch(1);

  console.log(`Worker waiting for tasks in ${queueName}`);

  channel.consume(queueName, async (msg) => {
    if (msg !== null) {
      const task = msg.content.toString();
      console.log(`Processing: ${task}`);

      // Simulate work
      const workTime = (task.match(/\./g) || []).length * 1000;
      await new Promise(resolve => setTimeout(resolve, workTime));

      console.log(`Done: ${task}`);
      channel.ack(msg);
    }
  }, {
    noAck: false
  });
}

// Usage
worker('task_queue');
```

### 4. Publish/Subscribe with Fanout Exchange

**Publisher:**

```javascript
const amqp = require('amqplib');

async function publishLog(message) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  const exchange = 'logs';

  await channel.assertExchange(exchange, 'fanout', {
    durable: false
  });

  channel.publish(exchange, '', Buffer.from(message));
  console.log(`Sent: ${message}`);

  setTimeout(() => {
    connection.close();
  }, 500);
}

// Usage
publishLog('Hello World!');
```

**Subscriber:**

```javascript
const amqp = require('amqplib');

async function subscribeLogs() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  const exchange = 'logs';

  await channel.assertExchange(exchange, 'fanout', {
    durable: false
  });

  // Create exclusive queue (auto-delete when consumer disconnects)
  const q = await channel.assertQueue('', {
    exclusive: true
  });

  console.log(`Waiting for logs. Queue: ${q.queue}`);

  // Bind queue to exchange
  channel.bindQueue(q.queue, exchange, '');

  channel.consume(q.queue, (msg) => {
    if (msg !== null) {
      console.log(`Received: ${msg.content.toString()}`);
    }
  }, {
    noAck: true
  });
}

// Usage
subscribeLogs();
```

### 5. Routing with Direct Exchange

**Emitter:**

```javascript
const amqp = require('amqplib');

async function emitLog(severity, message) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  const exchange = 'direct_logs';

  await channel.assertExchange(exchange, 'direct', {
    durable: false
  });

  channel.publish(exchange, severity, Buffer.from(message));
  console.log(`Sent [${severity}]: ${message}`);

  setTimeout(() => {
    connection.close();
  }, 500);
}

// Usage
emitLog('error', 'Critical system error!');
emitLog('info', 'System started');
emitLog('warning', 'Disk space low');
```

**Receiver:**

```javascript
const amqp = require('amqplib');

async function receiveLogs(severities) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  const exchange = 'direct_logs';

  await channel.assertExchange(exchange, 'direct', {
    durable: false
  });

  const q = await channel.assertQueue('', {
    exclusive: true
  });

  console.log(`Waiting for logs with severities: ${severities.join(', ')}`);

  // Bind queue for each severity
  for (const severity of severities) {
    channel.bindQueue(q.queue, exchange, severity);
  }

  channel.consume(q.queue, (msg) => {
    if (msg !== null) {
      console.log(`[${msg.fields.routingKey}]: ${msg.content.toString()}`);
    }
  }, {
    noAck: true
  });
}

// Usage - receive only error and warning logs
receiveLogs(['error', 'warning']);
```

### 6. Topics with Topic Exchange

**Emit Log Topic:**

```javascript
const amqp = require('amqplib');

async function emitLogTopic(routingKey, message) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  const exchange = 'topic_logs';

  await channel.assertExchange(exchange, 'topic', {
    durable: false
  });

  channel.publish(exchange, routingKey, Buffer.from(message));
  console.log(`Sent [${routingKey}]: ${message}`);

  setTimeout(() => {
    connection.close();
  }, 500);
}

// Usage - routing keys with dot-separated words
emitLogTopic('kern.critical', 'A critical kernel error');
emitLogTopic('kern.info', 'Kernel information');
emitLogTopic('auth.warning', 'Authentication warning');
```

**Receive Log Topic:**

```javascript
const amqp = require('amqplib');

async function receiveLogsTopic(bindingKeys) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  const exchange = 'topic_logs';

  await channel.assertExchange(exchange, 'topic', {
    durable: false
  });

  const q = await channel.assertQueue('', {
    exclusive: true
  });

  console.log(`Waiting for logs matching: ${bindingKeys.join(', ')}`);

  // Bind queue with patterns
  // * (star) can substitute for exactly one word
  // # (hash) can substitute for zero or more words
  for (const key of bindingKeys) {
    channel.bindQueue(q.queue, exchange, key);
  }

  channel.consume(q.queue, (msg) => {
    if (msg !== null) {
      console.log(`[${msg.fields.routingKey}]: ${msg.content.toString()}`);
    }
  }, {
    noAck: true
  });
}

// Usage - pattern matching
receiveLogsTopic(['kern.*']);           // All kernel messages
receiveLogsTopic(['*.critical']);       // All critical messages
receiveLogsTopic(['kern.#']);          // All kern.* messages
receiveLogsTopic(['#']);               // All messages
receiveLogsTopic(['kern.critical', 'auth.*']); // Multiple patterns
```

### 7. RPC (Remote Procedure Call) Pattern

**RPC Client:**

```javascript
const amqp = require('amqplib');
const { v4: uuidv4 } = require('uuid');

class RPCClient {
  constructor() {
    this.connection = null;
    this.channel = null;
    this.replyQueue = null;
    this.responseHandlers = new Map();
  }

  async connect() {
    this.connection = await amqp.connect('amqp://localhost');
    this.channel = await this.connection.createChannel();

    // Create reply queue
    const q = await this.channel.assertQueue('', {
      exclusive: true
    });

    this.replyQueue = q.queue;

    // Consume from reply queue
    this.channel.consume(this.replyQueue, (msg) => {
      const correlationId = msg.properties.correlationId;
      const handler = this.responseHandlers.get(correlationId);

      if (handler) {
        handler(msg.content.toString());
        this.responseHandlers.delete(correlationId);
      }
    }, {
      noAck: true
    });
  }

  async call(n) {
    const correlationId = uuidv4();

    return new Promise((resolve) => {
      // Store response handler
      this.responseHandlers.set(correlationId, resolve);

      // Send RPC request
      this.channel.sendToQueue('rpc_queue', Buffer.from(n.toString()), {
        correlationId: correlationId,
        replyTo: this.replyQueue
      });
    });
  }

  async close() {
    await this.connection.close();
  }
}

// Usage
async function makeRPCCall() {
  const client = new RPCClient();
  await client.connect();

  console.log('Requesting fib(30)');
  const result = await client.call(30);
  console.log(`Result: ${result}`);

  await client.close();
}

makeRPCCall();
```

**RPC Server:**

```javascript
const amqp = require('amqplib');

function fibonacci(n) {
  if (n === 0 || n === 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

async function startRPCServer() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  const queue = 'rpc_queue';

  await channel.assertQueue(queue, {
    durable: false
  });

  channel.prefetch(1);
  console.log('Awaiting RPC requests');

  channel.consume(queue, (msg) => {
    const n = parseInt(msg.content.toString());

    console.log(`Computing fibonacci(${n})`);
    const result = fibonacci(n);

    // Send response
    channel.sendToQueue(
      msg.properties.replyTo,
      Buffer.from(result.toString()),
      {
        correlationId: msg.properties.correlationId
      }
    );

    channel.ack(msg);
  });
}

startRPCServer();
```

## Advanced Configuration

### Message Properties

```javascript
const amqp = require('amqplib');

async function sendWithProperties(queue, message) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue(queue, { durable: true });

  channel.sendToQueue(queue, Buffer.from(message), {
    persistent: true,                    // Survive broker restart
    expiration: '60000',                 // Message TTL in milliseconds
    priority: 5,                         // Message priority (0-255)
    contentType: 'application/json',     // MIME type
    contentEncoding: 'utf-8',            // Encoding
    timestamp: Date.now(),               // Timestamp
    messageId: 'msg-123',                // Application message ID
    userId: 'guest',                     // Creating user
    appId: 'my-app',                     // Application ID
    headers: {                           // Custom headers
      'x-custom-header': 'value'
    }
  });

  setTimeout(() => connection.close(), 500);
}
```

### Queue Options

```javascript
const amqp = require('amqplib');

async function declareQueueWithOptions() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue('advanced_queue', {
    durable: true,           // Queue survives broker restart
    exclusive: false,        // Can be accessed by other connections
    autoDelete: false,       // Queue won't be deleted when no consumers
    messageTtl: 60000,       // Message TTL in milliseconds
    expires: 300000,         // Queue expires after 5 minutes of non-use
    maxLength: 1000,         // Maximum queue length
    maxPriority: 10,         // Enable priority queue (0-10)
    deadLetterExchange: 'dlx',          // DLX for rejected messages
    deadLetterRoutingKey: 'dead.letter' // Routing key for DLX
  });

  await connection.close();
}
```

### Exchange Options

```javascript
const amqp = require('amqplib');

async function declareExchangeWithOptions() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertExchange('advanced_exchange', 'topic', {
    durable: true,        // Exchange survives broker restart
    autoDelete: false,    // Exchange won't be deleted when no bindings
    internal: false,      // Can be published to by clients
    alternateExchange: 'alternate_exchange'  // AE for unroutable messages
  });

  await connection.close();
}
```

### Dead Letter Queue (DLQ)

```javascript
const amqp = require('amqplib');

async function setupDeadLetterQueue() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  // Declare dead letter exchange
  await channel.assertExchange('dlx', 'direct', {
    durable: true
  });

  // Declare dead letter queue
  await channel.assertQueue('dead_letter_queue', {
    durable: true
  });

  // Bind DLQ to DLX
  await channel.bindQueue('dead_letter_queue', 'dlx', 'dead.letter');

  // Declare main queue with DLX
  await channel.assertQueue('main_queue', {
    durable: true,
    deadLetterExchange: 'dlx',
    deadLetterRoutingKey: 'dead.letter',
    messageTtl: 10000  // Messages expire after 10 seconds
  });

  console.log('Dead letter queue setup complete');
  await connection.close();
}

setupDeadLetterQueue();
```

### Consuming from Dead Letter Queue

```javascript
const amqp = require('amqplib');

async function consumeDeadLetters() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue('dead_letter_queue', {
    durable: true
  });

  console.log('Waiting for dead letter messages');

  channel.consume('dead_letter_queue', (msg) => {
    if (msg !== null) {
      console.log('Dead letter received:', msg.content.toString());
      console.log('Original routing key:', msg.fields.routingKey);
      console.log('Death reason:', msg.properties.headers['x-death']);

      // Process or log dead letter
      channel.ack(msg);
    }
  });
}

consumeDeadLetters();
```

### Message Rejection and Requeuing

```javascript
const amqp = require('amqplib');

async function consumerWithRetry() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue('task_queue', {
    durable: true
  });

  channel.consume('task_queue', async (msg) => {
    if (msg !== null) {
      try {
        const task = msg.content.toString();
        console.log('Processing:', task);

        // Simulate processing
        await processTask(task);

        // Success - acknowledge
        channel.ack(msg);
      } catch (error) {
        console.error('Processing failed:', error);

        // Check retry count
        const retryCount = (msg.properties.headers?.['x-retry-count'] || 0);

        if (retryCount < 3) {
          // Reject and requeue for retry
          channel.nack(msg, false, true);
        } else {
          // Max retries reached - reject without requeue (goes to DLQ)
          channel.nack(msg, false, false);
        }
      }
    }
  });
}

async function processTask(task) {
  // Simulate work
  if (Math.random() < 0.3) {
    throw new Error('Random failure');
  }
}

consumerWithRetry();
```

### Publisher Confirms

```javascript
const amqp = require('amqplib');

async function publishWithConfirm() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createConfirmChannel();

  const queue = 'confirm_queue';
  await channel.assertQueue(queue, { durable: true });

  const message = 'Important message';

  try {
    channel.sendToQueue(queue, Buffer.from(message), {
      persistent: true
    });

    // Wait for confirmation
    await channel.waitForConfirms();
    console.log('Message confirmed by broker');
  } catch (error) {
    console.error('Message nacked by broker:', error);
  }

  await connection.close();
}

publishWithConfirm();
```

### Batch Publishing with Confirms

```javascript
const amqp = require('amqplib');

async function batchPublishWithConfirms() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createConfirmChannel();

  const queue = 'batch_queue';
  await channel.assertQueue(queue, { durable: true });

  const messages = Array.from({ length: 100 }, (_, i) => `Message ${i}`);

  for (const message of messages) {
    channel.sendToQueue(queue, Buffer.from(message), {
      persistent: true
    });
  }

  try {
    await channel.waitForConfirms();
    console.log('All messages confirmed');
  } catch (error) {
    console.error('Some messages were nacked:', error);
  }

  await connection.close();
}

batchPublishWithConfirms();
```

## Error Handling and Reconnection

### Connection Error Handling

```javascript
const amqp = require('amqplib');

async function connectWithErrorHandling() {
  let connection;

  try {
    connection = await amqp.connect('amqp://localhost');

    // Handle connection errors
    connection.on('error', (err) => {
      console.error('Connection error:', err);
    });

    // Handle connection close
    connection.on('close', () => {
      console.log('Connection closed');
      // Implement reconnection logic here
      setTimeout(() => {
        console.log('Attempting to reconnect...');
        connectWithErrorHandling();
      }, 5000);
    });

    const channel = await connection.createChannel();

    // Handle channel errors
    channel.on('error', (err) => {
      console.error('Channel error:', err);
    });

    // Handle channel close
    channel.on('close', () => {
      console.log('Channel closed');
    });

    return { connection, channel };
  } catch (error) {
    console.error('Failed to connect:', error);

    // Retry connection after delay
    setTimeout(() => {
      console.log('Retrying connection...');
      connectWithErrorHandling();
    }, 5000);
  }
}

connectWithErrorHandling();
```

### Robust Connection Manager

```javascript
const amqp = require('amqplib');

class RabbitMQConnection {
  constructor(url) {
    this.url = url;
    this.connection = null;
    this.channel = null;
    this.reconnecting = false;
  }

  async connect() {
    try {
      this.connection = await amqp.connect(this.url);
      this.channel = await this.connection.createChannel();

      console.log('Connected to RabbitMQ');

      this.connection.on('error', (err) => {
        console.error('Connection error:', err);
        this.reconnect();
      });

      this.connection.on('close', () => {
        console.log('Connection closed');
        this.reconnect();
      });

      return this.channel;
    } catch (error) {
      console.error('Connection failed:', error);
      this.reconnect();
    }
  }

  async reconnect() {
    if (this.reconnecting) return;

    this.reconnecting = true;
    console.log('Reconnecting in 5 seconds...');

    setTimeout(async () => {
      this.reconnecting = false;
      await this.connect();
    }, 5000);
  }

  async close() {
    if (this.channel) {
      await this.channel.close();
    }
    if (this.connection) {
      await this.connection.close();
    }
  }
}

// Usage
const manager = new RabbitMQConnection('amqp://localhost');
const channel = await manager.connect();
```

### Graceful Shutdown

```javascript
const amqp = require('amqplib');

let connection = null;
let channel = null;

async function setup() {
  connection = await amqp.connect('amqp://localhost');
  channel = await connection.createChannel();

  await channel.assertQueue('shutdown_queue', { durable: true });

  channel.consume('shutdown_queue', (msg) => {
    if (msg !== null) {
      console.log('Processing:', msg.content.toString());
      channel.ack(msg);
    }
  });
}

async function cleanup() {
  console.log('Shutting down gracefully...');

  if (channel) {
    // Cancel consumers
    await channel.close();
  }

  if (connection) {
    await connection.close();
  }

  console.log('Cleanup complete');
  process.exit(0);
}

// Handle shutdown signals
process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);

setup();
```

## Common Patterns

### Message Retry with Delay

```javascript
const amqp = require('amqplib');

async function setupRetryQueue() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  // Main queue
  await channel.assertQueue('main_queue', {
    durable: true,
    deadLetterExchange: 'retry',
    deadLetterRoutingKey: 'retry'
  });

  // Retry exchange
  await channel.assertExchange('retry', 'direct', {
    durable: true
  });

  // Retry queue with delay
  await channel.assertQueue('retry_queue', {
    durable: true,
    messageTtl: 30000,  // 30 second delay
    deadLetterExchange: '',
    deadLetterRoutingKey: 'main_queue'
  });

  await channel.bindQueue('retry_queue', 'retry', 'retry');

  console.log('Retry queue setup complete');
  await connection.close();
}

setupRetryQueue();
```

### Priority Queue

```javascript
const amqp = require('amqplib');

async function setupPriorityQueue() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  // Queue with priority support
  await channel.assertQueue('priority_queue', {
    durable: true,
    maxPriority: 10
  });

  // Send messages with different priorities
  channel.sendToQueue('priority_queue', Buffer.from('Low priority'), {
    priority: 1
  });

  channel.sendToQueue('priority_queue', Buffer.from('High priority'), {
    priority: 10
  });

  channel.sendToQueue('priority_queue', Buffer.from('Medium priority'), {
    priority: 5
  });

  setTimeout(() => connection.close(), 500);
}

setupPriorityQueue();
```

### Rate Limiting Consumer

```javascript
const amqp = require('amqplib');

async function rateLimitedConsumer() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue('rate_limited_queue', {
    durable: true
  });

  // Process only 1 message at a time
  channel.prefetch(1);

  let processing = false;

  channel.consume('rate_limited_queue', async (msg) => {
    if (msg !== null && !processing) {
      processing = true;

      console.log('Processing:', msg.content.toString());

      // Simulate rate-limited processing
      await new Promise(resolve => setTimeout(resolve, 2000));

      channel.ack(msg);
      processing = false;
    }
  });
}

rateLimitedConsumer();
```

### JSON Message Handling

```javascript
const amqp = require('amqplib');

async function sendJSON(queue, data) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue(queue, { durable: true });

  const message = JSON.stringify(data);

  channel.sendToQueue(queue, Buffer.from(message), {
    contentType: 'application/json',
    persistent: true
  });

  console.log('Sent JSON:', data);

  setTimeout(() => connection.close(), 500);
}

async function receiveJSON(queue) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();

  await channel.assertQueue(queue, { durable: true });

  channel.consume(queue, (msg) => {
    if (msg !== null) {
      try {
        const data = JSON.parse(msg.content.toString());
        console.log('Received JSON:', data);
        channel.ack(msg);
      } catch (error) {
        console.error('Invalid JSON:', error);
        channel.nack(msg, false, false);
      }
    }
  });
}

// Usage
sendJSON('json_queue', { user: 'john', action: 'login', timestamp: Date.now() });
receiveJSON('json_queue');
```

## Useful Links

- **Official Documentation:** https://www.rabbitmq.com/docs
- **amqplib GitHub:** https://github.com/amqp-node/amqplib
- **amqplib API Reference:** https://amqp-node.github.io/amqplib/channel_api.html
- **RabbitMQ Tutorials:** https://www.rabbitmq.com/tutorials
- **AMQP 0-9-1 Reference:** https://www.rabbitmq.com/amqp-0-9-1-reference.html

---
name: key-value
description: "Redis JavaScript client (node-redis) for key-value storage, caching, and pub/sub messaging"
metadata:
  languages: "javascript"
  versions: "5.9.0"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "redis,database,cache,key-value,pubsub"
---

# Redis JavaScript Client (node-redis) - Complete Integration Guide

## GOLDEN RULE

**ALWAYS use the official `redis` npm package (node-redis) for Redis integration.**

```bash
npm install redis
```

**DO NOT use:**
- `ioredis` (alternative client, not official)
- `redis-node` (unofficial package)
- `node_redis` (deprecated naming)
- Any other third-party Redis clients unless specifically required

The official `redis` package is maintained by Redis and is the recommended client for Node.js applications.

---

## Installation

### Basic Installation

```bash
npm install redis
```

### Installation with TypeScript Support

```bash
npm install redis
npm install --save-dev @types/node
```

The `redis` package includes built-in TypeScript definitions.

### Environment Setup

Create a `.env` file in your project root:

```env
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=your_password_here
REDIS_DB=0
REDIS_URL=redis://username:password@localhost:6379
```

Install dotenv to load environment variables:

```bash
npm install dotenv
```

---

## Initialization

### Basic Connection (Localhost)

```javascript
import { createClient } from 'redis';

const client = createClient();

client.on('error', (err) => {
  console.error('Redis Client Error', err);
});

await client.connect();

// Use the client...

await client.quit();
```

### Connection with Environment Variables

```javascript
import { createClient } from 'redis';
import dotenv from 'dotenv';

dotenv.config();

const client = createClient({
  socket: {
    host: process.env.REDIS_HOST || 'localhost',
    port: parseInt(process.env.REDIS_PORT || '6379')
  },
  password: process.env.REDIS_PASSWORD,
  database: parseInt(process.env.REDIS_DB || '0')
});

client.on('error', (err) => {
  console.error('Redis Client Error', err);
});

await client.connect();
```

### Connection Using URL

```javascript
import { createClient } from 'redis';
import dotenv from 'dotenv';

dotenv.config();

const client = createClient({
  url: process.env.REDIS_URL || 'redis://localhost:6379'
});

client.on('error', (err) => {
  console.error('Redis Client Error', err);
});

await client.connect();
```

### Connection with Authentication

```javascript
import { createClient } from 'redis';

const client = createClient({
  url: 'redis://alice:foobared@awesome.redis.server:6380'
});

// Or using socket configuration
const clientAlt = createClient({
  socket: {
    host: 'awesome.redis.server',
    port: 6380
  },
  username: 'alice',
  password: 'foobared'
});

await client.connect();
```

### Connection with TLS/SSL

```javascript
import { createClient } from 'redis';
import fs from 'fs';

const client = createClient({
  socket: {
    host: 'redis.example.com',
    port: 6380,
    tls: true,
    key: fs.readFileSync('/path/to/client-key.pem'),
    cert: fs.readFileSync('/path/to/client-cert.pem'),
    ca: [fs.readFileSync('/path/to/ca-cert.pem')]
  }
});

await client.connect();
```

### Connection Status Checks

```javascript
// Check if client is ready to execute commands
if (client.isReady) {
  console.log('Client is ready');
}

// Check if underlying socket is open
if (client.isOpen) {
  console.log('Connection is open');
}
```

### Duplicate Connection (for Pub/Sub)

```javascript
const client = createClient();
await client.connect();

// Create a duplicate connection for pub/sub
const subscriber = client.duplicate();
await subscriber.connect();
```

### Graceful Shutdown

```javascript
// Graceful disconnect
await client.quit();

// Force disconnect
await client.disconnect();
```

---

## Core API Surfaces

## String Operations

### Basic SET and GET

```javascript
// Set a string value
await client.set('key', 'value');

// Get a string value
const value = await client.get('key');
console.log(value); // 'value'
```

### SET with Options

```javascript
// Set with expiration (EX = seconds)
await client.set('session:123', 'user_data', {
  EX: 3600 // expires in 1 hour
});

// Set with expiration (PX = milliseconds)
await client.set('temp:key', 'value', {
  PX: 5000 // expires in 5 seconds
});

// Set only if key doesn't exist (NX)
await client.set('key', 'value', {
  NX: true
});

// Set only if key exists (XX)
await client.set('key', 'new_value', {
  XX: true
});

// Get old value and set new value
const oldValue = await client.set('key', 'new_value', {
  GET: true
});
```

### Multiple Keys

```javascript
// Set multiple keys at once
await client.mSet({
  'key1': 'value1',
  'key2': 'value2',
  'key3': 'value3'
});

// Get multiple keys at once
const values = await client.mGet(['key1', 'key2', 'key3']);
console.log(values); // ['value1', 'value2', 'value3']
```

### Increment and Decrement

```javascript
// Set initial value
await client.set('counter', 0);

// Increment by 1
await client.incr('counter'); // Returns 1

// Increment by specific amount
await client.incrBy('counter', 10); // Returns 11

// Increment float
await client.incrByFloat('price', 2.5); // Increment by 2.5

// Decrement by 1
await client.decr('counter');

// Decrement by specific amount
await client.decrBy('counter', 5);
```

### String Manipulation

```javascript
// Append to string
await client.set('message', 'Hello');
await client.append('message', ' World'); // 'Hello World'

// Get substring
const substr = await client.getRange('message', 0, 4); // 'Hello'

// Get length
const len = await client.strLen('message'); // 11

// Set range
await client.setRange('message', 6, 'Redis'); // 'Hello Redis'
```

---

## Hash Operations

### Basic Hash Operations

```javascript
// Set a single field in a hash
await client.hSet('user:1000', 'name', 'John Doe');

// Get a single field from a hash
const name = await client.hGet('user:1000', 'name');
console.log(name); // 'John Doe'

// Set multiple fields at once
await client.hSet('user:1000', {
  name: 'John Doe',
  email: '[email protected]',
  age: 30
});

// Get all fields and values
const user = await client.hGetAll('user:1000');
console.log(user);
// { name: 'John Doe', email: '[email protected]', age: '30' }
```

### Advanced Hash Operations

```javascript
// Check if field exists
const exists = await client.hExists('user:1000', 'email'); // true

// Get all field names
const fields = await client.hKeys('user:1000');
// ['name', 'email', 'age']

// Get all values
const values = await client.hVals('user:1000');
// ['John Doe', '[email protected]', '30']

// Get number of fields
const count = await client.hLen('user:1000'); // 3

// Get multiple fields
const userData = await client.hmGet('user:1000', ['name', 'email']);
// ['John Doe', '[email protected]']

// Delete fields
await client.hDel('user:1000', 'age');

// Increment numeric field
await client.hSet('user:1000', 'loginCount', 0);
await client.hIncrBy('user:1000', 'loginCount', 1); // 1

// Increment float field
await client.hIncrByFloat('user:1000', 'balance', 10.50);

// Set only if field doesn't exist
await client.hSetNX('user:1000', 'created', Date.now());
```

### Scan Hash Fields

```javascript
// Scan hash fields (for large hashes)
for await (const { field, value } of client.hScanIterator('user:1000')) {
  console.log(`${field}: ${value}`);
}
```

---

## List Operations

### Basic List Operations

```javascript
// Push elements to the right (end) of list
await client.rPush('tasks', 'task1');
await client.rPush('tasks', ['task2', 'task3']); // Multiple values

// Push elements to the left (beginning) of list
await client.lPush('tasks', 'urgent_task');
await client.lPush('tasks', ['task0', 'task-1']);

// Get list length
const length = await client.lLen('tasks');

// Get range of elements
const tasks = await client.lRange('tasks', 0, -1); // Get all
const firstThree = await client.lRange('tasks', 0, 2); // Get first 3

// Get element by index
const task = await client.lIndex('tasks', 0);

// Pop from right (end)
const lastTask = await client.rPop('tasks');

// Pop from left (beginning)
const firstTask = await client.lPop('tasks');
```

### Advanced List Operations

```javascript
// Blocking pop (wait for element)
const task = await client.blPop('tasks', 10); // Wait up to 10 seconds
// Returns: { key: 'tasks', element: 'task1' }

const taskRight = await client.brPop('tasks', 10);

// Set element at index
await client.lSet('tasks', 0, 'updated_task');

// Insert before/after element
await client.lInsert('tasks', 'BEFORE', 'task2', 'new_task');
await client.lInsert('tasks', 'AFTER', 'task2', 'another_task');

// Remove elements
await client.lRem('tasks', 2, 'task1'); // Remove first 2 occurrences
await client.lRem('tasks', -1, 'task2'); // Remove last occurrence
await client.lRem('tasks', 0, 'task3'); // Remove all occurrences

// Trim list to range
await client.lTrim('tasks', 0, 9); // Keep only first 10 elements

// Move element between lists
const element = await client.rPopLPush('source', 'destination');

// Blocking move
const moved = await client.brPopLPush('source', 'destination', 5);
```

---

## Set Operations

### Basic Set Operations

```javascript
// Add members to set
await client.sAdd('tags', 'javascript');
await client.sAdd('tags', ['nodejs', 'redis', 'database']);

// Check if member exists
const exists = await client.sIsMember('tags', 'nodejs'); // true

// Get all members
const allTags = await client.sMembers('tags');
// ['javascript', 'nodejs', 'redis', 'database']

// Get number of members
const count = await client.sCard('tags'); // 4

// Remove members
await client.sRem('tags', 'database');
await client.sRem('tags', ['nodejs', 'redis']);

// Pop random member
const randomTag = await client.sPop('tags');

// Get random member without removing
const random = await client.sRandMember('tags');
const randomThree = await client.sRandMemberCount('tags', 3);
```

### Set Operations Between Multiple Sets

```javascript
// Create sets
await client.sAdd('set1', ['a', 'b', 'c']);
await client.sAdd('set2', ['b', 'c', 'd']);
await client.sAdd('set3', ['c', 'd', 'e']);

// Union (combine all unique members)
const union = await client.sUnion(['set1', 'set2']);
// ['a', 'b', 'c', 'd']

// Store union in new set
await client.sUnionStore('result', ['set1', 'set2']);

// Intersection (common members)
const inter = await client.sInter(['set1', 'set2']);
// ['b', 'c']

// Store intersection
await client.sInterStore('result', ['set1', 'set2']);

// Difference (members in first set but not in others)
const diff = await client.sDiff(['set1', 'set2']);
// ['a']

// Store difference
await client.sDiffStore('result', ['set1', 'set2']);

// Move member between sets
await client.sMove('set1', 'set2', 'a');
```

### Scan Set Members

```javascript
// Scan set members (for large sets)
for await (const member of client.sScanIterator('tags')) {
  console.log(member);
}
```

---

## Sorted Set Operations

### Basic Sorted Set Operations

```javascript
// Add members with scores
await client.zAdd('leaderboard', { score: 100, value: 'player1' });
await client.zAdd('leaderboard', [
  { score: 200, value: 'player2' },
  { score: 150, value: 'player3' }
]);

// Get rank (0-based, lowest to highest)
const rank = await client.zRank('leaderboard', 'player1'); // 0

// Get reverse rank (highest to lowest)
const revRank = await client.zRevRank('leaderboard', 'player2'); // 0

// Get score
const score = await client.zScore('leaderboard', 'player2'); // 200

// Get number of members
const count = await client.zCard('leaderboard'); // 3

// Increment score
await client.zIncrBy('leaderboard', 50, 'player1'); // 150
```

### Range Queries

```javascript
// Get range by rank (lowest to highest)
const bottom3 = await client.zRange('leaderboard', 0, 2);
// ['player1', 'player3', 'player2']

// Get range with scores
const withScores = await client.zRangeWithScores('leaderboard', 0, 2);
// [{ value: 'player1', score: 150 }, ...]

// Get range by rank (highest to lowest)
const top3 = await client.zRevRange('leaderboard', 0, 2);

// Get range by score
const range = await client.zRangeByScore('leaderboard', 100, 200);

// Get range by score with limit
const limited = await client.zRangeByScore('leaderboard', 100, 200, {
  LIMIT: {
    offset: 0,
    count: 10
  }
});

// Get reverse range by score
const revRange = await client.zRevRangeByScore('leaderboard', 200, 100);
```

### Advanced Sorted Set Operations

```javascript
// Count members in score range
const count = await client.zCount('leaderboard', 100, 200);

// Count members by lexicographical range
const lexCount = await client.zLexCount('leaderboard', '[a', '[z');

// Remove members
await client.zRem('leaderboard', 'player1');
await client.zRem('leaderboard', ['player2', 'player3']);

// Remove by rank range
await client.zRemRangeByRank('leaderboard', 0, 1); // Remove bottom 2

// Remove by score range
await client.zRemRangeByScore('leaderboard', 0, 100);

// Pop highest/lowest scoring member
const highest = await client.zPopMax('leaderboard');
const lowest = await client.zPopMin('leaderboard');

// Pop with count
const top3 = await client.zPopMax('leaderboard', 3);

// Blocking pop
const member = await client.bzPopMax('leaderboard', 5); // Wait up to 5 sec
const minMember = await client.bzPopMin('leaderboard', 5);
```

### Sorted Set Operations Between Multiple Sets

```javascript
// Union of sorted sets
await client.zUnionStore('result', ['set1', 'set2']);

// Union with weights
await client.zUnionStore('result', ['set1', 'set2'], {
  WEIGHTS: [2, 3]
});

// Union with aggregate function
await client.zUnionStore('result', ['set1', 'set2'], {
  AGGREGATE: 'MAX' // or 'MIN', 'SUM'
});

// Intersection
await client.zInterStore('result', ['set1', 'set2']);

// Intersection with weights and aggregate
await client.zInterStore('result', ['set1', 'set2'], {
  WEIGHTS: [1, 2],
  AGGREGATE: 'SUM'
});
```

### Scan Sorted Set

```javascript
// Scan sorted set members
for await (const member of client.zScanIterator('leaderboard')) {
  console.log(member);
}
```

---

## Key Management Operations

### Key Operations

```javascript
// Check if key exists
const exists = await client.exists('mykey'); // 1 if exists, 0 if not
const multiExists = await client.exists(['key1', 'key2']); // Count

// Delete keys
await client.del('mykey');
await client.del(['key1', 'key2', 'key3']);

// Set expiration in seconds
await client.expire('mykey', 60); // Expire in 60 seconds

// Set expiration at specific timestamp
const timestamp = Math.floor(Date.now() / 1000) + 3600;
await client.expireAt('mykey', timestamp);

// Set expiration in milliseconds
await client.pExpire('mykey', 60000); // 60 seconds

// Get time to live in seconds
const ttl = await client.ttl('mykey'); // -1 if no expiration, -2 if not exists

// Get time to live in milliseconds
const pttl = await client.pTtl('mykey');

// Remove expiration
await client.persist('mykey');

// Rename key
await client.rename('oldkey', 'newkey');

// Rename only if new key doesn't exist
const renamed = await client.renameNX('oldkey', 'newkey');

// Get key type
const type = await client.type('mykey'); // 'string', 'list', 'set', etc.

// Get random key
const randomKey = await client.randomKey();
```

### Scanning Keys

```javascript
// Scan all keys (use instead of KEYS for production)
const keys = [];
for await (const key of client.scanIterator()) {
  keys.push(key);
}

// Scan with pattern
for await (const key of client.scanIterator({
  MATCH: 'user:*',
  COUNT: 100
})) {
  console.log(key);
}

// Scan specific key type
for await (const key of client.scanIterator({
  TYPE: 'string',
  COUNT: 100
})) {
  console.log(key);
}
```

---

## Transactions

### Basic Transaction (MULTI/EXEC)

```javascript
// Execute multiple commands atomically
await client.set('another-key', 'another-value');

const results = await client
  .multi()
  .set('key', 'value')
  .get('another-key')
  .incr('counter')
  .exec();

console.log(results); // ['OK', 'another-value', 1]
```

### Transaction with Error Handling

```javascript
try {
  const results = await client
    .multi()
    .set('key1', 'value1')
    .set('key2', 'value2')
    .get('key1')
    .exec();

  // Check each result
  results.forEach((result, index) => {
    if (result instanceof Error) {
      console.error(`Command ${index} failed:`, result);
    } else {
      console.log(`Command ${index} result:`, result);
    }
  });
} catch (error) {
  console.error('Transaction failed:', error);
}
```

### Transaction with WATCH (Optimistic Locking)

```javascript
// Watch a key for changes
await client.watch('balance');

const balance = parseInt(await client.get('balance'));

if (balance >= 100) {
  // This transaction will fail if 'balance' changed after WATCH
  const results = await client
    .multi()
    .decrBy('balance', 100)
    .incrBy('purchases', 1)
    .exec();

  if (results === null) {
    console.log('Transaction aborted - balance was modified');
  } else {
    console.log('Purchase successful');
  }
} else {
  await client.unwatch();
  console.log('Insufficient balance');
}
```

### Complex Transaction Example

```javascript
async function transferMoney(fromAccount, toAccount, amount) {
  await client.watch([fromAccount, toAccount]);

  const fromBalance = parseFloat(await client.get(fromAccount));

  if (fromBalance < amount) {
    await client.unwatch();
    throw new Error('Insufficient funds');
  }

  const results = await client
    .multi()
    .decrByFloat(fromAccount, amount)
    .incrByFloat(toAccount, amount)
    .exec();

  if (results === null) {
    throw new Error('Transaction failed - accounts modified during transfer');
  }

  return results;
}

// Usage
try {
  await transferMoney('account:1', 'account:2', 50.00);
  console.log('Transfer successful');
} catch (error) {
  console.error('Transfer failed:', error.message);
}
```

---

## Pipelining

### Automatic Pipelining

Node-redis automatically pipelines commands that are queued in the same tick:

```javascript
// These commands are automatically batched and sent together
const [result1, result2, result3] = await Promise.all([
  client.set('key1', 'value1'),
  client.set('key2', 'value2'),
  client.get('key1')
]);

console.log(result1); // 'OK'
console.log(result2); // 'OK'
console.log(result3); // 'value1'
```

### Manual Pipelining with Multi (No Atomicity)

```javascript
// Execute commands as pipeline without transaction semantics
const results = await client
  .multi()
  .set('key1', 'value1')
  .set('key2', 'value2')
  .get('key1')
  .mGet(['key1', 'key2'])
  .exec();

console.log(results);
// ['OK', 'OK', 'value1', ['value1', 'value2']]
```

### Large Batch Operations

```javascript
async function batchSetKeys(keyValuePairs) {
  const pipeline = client.multi();

  for (const [key, value] of Object.entries(keyValuePairs)) {
    pipeline.set(key, value);
  }

  const results = await pipeline.exec();
  return results;
}

// Usage
const data = {
  'user:1': 'Alice',
  'user:2': 'Bob',
  'user:3': 'Charlie'
};

await batchSetKeys(data);
```

---

## Pub/Sub

### Basic Subscriber

```javascript
import { createClient } from 'redis';

const client = createClient();
await client.connect();

// Create dedicated connection for subscribing
const subscriber = client.duplicate();
await subscriber.connect();

// Subscribe to channel
await subscriber.subscribe('notifications', (message) => {
  console.log('Received message:', message);
});

console.log('Subscribed to notifications channel');
```

### Basic Publisher

```javascript
import { createClient } from 'redis';

const publisher = createClient();
await publisher.connect();

// Publish messages
await publisher.publish('notifications', 'Hello, World!');
await publisher.publish('notifications', JSON.stringify({
  type: 'alert',
  message: 'System update'
}));

console.log('Messages published');
```

### Multiple Channels

```javascript
// Subscribe to multiple channels
await subscriber.subscribe('channel1', (message) => {
  console.log('Channel1:', message);
});

await subscriber.subscribe('channel2', (message) => {
  console.log('Channel2:', message);
});

await subscriber.subscribe('channel3', (message) => {
  console.log('Channel3:', message);
});
```

### Pattern-Based Subscription

```javascript
// Subscribe to channels matching pattern
await subscriber.pSubscribe('user:*', (message, channel) => {
  console.log(`Message from ${channel}:`, message);
});

// Publish to matching channels
await publisher.publish('user:1000', 'User 1000 logged in');
await publisher.publish('user:2000', 'User 2000 logged out');
```

### Unsubscribe

```javascript
// Unsubscribe from specific channel
await subscriber.unsubscribe('channel1');

// Unsubscribe from all channels
await subscriber.unsubscribe();

// Unsubscribe from pattern
await subscriber.pUnsubscribe('user:*');
```

### Complete Pub/Sub Example

```javascript
import { createClient } from 'redis';

// Subscriber setup
async function setupSubscriber() {
  const client = createClient();
  await client.connect();

  const subscriber = client.duplicate();
  await subscriber.connect();

  await subscriber.subscribe('chat:room1', (message) => {
    const data = JSON.parse(message);
    console.log(`${data.user}: ${data.text}`);
  });

  return subscriber;
}

// Publisher setup
async function setupPublisher() {
  const publisher = createClient();
  await publisher.connect();
  return publisher;
}

// Usage
const subscriber = await setupSubscriber();
const publisher = await setupPublisher();

// Publish chat messages
await publisher.publish('chat:room1', JSON.stringify({
  user: 'Alice',
  text: 'Hello everyone!'
}));

await publisher.publish('chat:room1', JSON.stringify({
  user: 'Bob',
  text: 'Hi Alice!'
}));
```

---

## Redis Streams

### Add to Stream (XADD)

```javascript
// Add entry to stream
const id = await client.xAdd('events', '*', {
  user: 'alice',
  action: 'login',
  timestamp: Date.now()
});

console.log('Entry ID:', id); // '1234567890123-0'

// Add with specific ID
await client.xAdd('events', '1234567890123-1', {
  user: 'bob',
  action: 'logout'
});

// Add with maxLen (limit stream size)
await client.xAdd('events', '*', {
  user: 'charlie',
  action: 'signup'
}, {
  TRIM: {
    strategy: 'MAXLEN',
    strategyModifier: '~', // Approximate
    threshold: 1000
  }
});
```

### Read from Stream (XREAD)

```javascript
// Read new entries
const messages = await client.xRead(
  { key: 'events', id: '0' },
  { COUNT: 10 }
);

console.log(messages);
// [{ name: 'events', messages: [{ id: '...', message: {...} }] }]

// Read from multiple streams
const multiStream = await client.xRead([
  { key: 'stream1', id: '0' },
  { key: 'stream2', id: '0' }
]);

// Blocking read (wait for new entries)
const newMessages = await client.xRead(
  { key: 'events', id: '$' }, // '$' means only new messages
  { BLOCK: 5000 } // Wait up to 5 seconds
);
```

### Stream Range Queries

```javascript
// Read all entries
const all = await client.xRange('events', '-', '+');

// Read range with IDs
const range = await client.xRange('events', '1234567890000', '1234567899999');

// Read with limit
const limited = await client.xRange('events', '-', '+', { COUNT: 100 });

// Reverse range
const reverse = await client.xRevRange('events', '+', '-', { COUNT: 10 });
```

### Stream Length and Info

```javascript
// Get stream length
const length = await client.xLen('events');

// Get stream info
const info = await client.xInfo('events');
```

### Consumer Groups

```javascript
// Create consumer group
await client.xGroupCreate('events', 'processors', '0', {
  MKSTREAM: true // Create stream if it doesn't exist
});

// Read as consumer group
const messages = await client.xReadGroup(
  'processors',
  'consumer1',
  { key: 'events', id: '>' }, // '>' means undelivered messages
  { COUNT: 10, BLOCK: 5000 }
);

// Process messages and acknowledge
for (const stream of messages) {
  for (const message of stream.messages) {
    // Process message
    console.log('Processing:', message.id, message.message);

    // Acknowledge message
    await client.xAck('events', 'processors', message.id);
  }
}
```

### Delete from Stream

```javascript
// Delete specific entries
await client.xDel('events', ['1234567890123-0', '1234567890123-1']);

// Trim stream
await client.xTrim('events', 'MAXLEN', 1000);

// Approximate trim
await client.xTrim('events', 'MAXLEN', { strategyModifier: '~', threshold: 1000 });
```

### Complete Stream Example

```javascript
import { createClient } from 'redis';

const client = createClient();
await client.connect();

// Producer
async function addEvent(eventData) {
  return await client.xAdd('events', '*', eventData);
}

// Consumer with consumer group
async function processEvents() {
  // Create group if not exists
  try {
    await client.xGroupCreate('events', 'workers', '0', { MKSTREAM: true });
  } catch (err) {
    // Group already exists
  }

  while (true) {
    const messages = await client.xReadGroup(
      'workers',
      'worker1',
      { key: 'events', id: '>' },
      { COUNT: 10, BLOCK: 5000 }
    );

    if (messages && messages.length > 0) {
      for (const stream of messages) {
        for (const message of stream.messages) {
          try {
            // Process event
            console.log('Processing event:', message.message);

            // Acknowledge
            await client.xAck('events', 'workers', message.id);
          } catch (error) {
            console.error('Error processing:', error);
          }
        }
      }
    }
  }
}

// Usage
await addEvent({ type: 'user_login', user: 'alice' });
await addEvent({ type: 'user_logout', user: 'bob' });

// Start consumer
processEvents();
```

---

## Scripting with Lua

### Basic Script Execution (EVAL)

```javascript
// Execute Lua script
const result = await client.eval(
  `return redis.call('SET', KEYS[1], ARGV[1])`,
  {
    keys: ['mykey'],
    arguments: ['myvalue']
  }
);

console.log(result); // 'OK'
```

### Script with Multiple Operations

```javascript
const script = `
  local current = redis.call('GET', KEYS[1])
  if current == false then
    current = 0
  end
  local new = tonumber(current) + tonumber(ARGV[1])
  redis.call('SET', KEYS[1], new)
  return new
`;

const newValue = await client.eval(script, {
  keys: ['counter'],
  arguments: ['5']
});

console.log('New counter value:', newValue);
```

### Load and Execute Scripts (SCRIPT LOAD / EVALSHA)

```javascript
// Load script and get SHA1
const sha = await client.scriptLoad(`
  return redis.call('GET', KEYS[1])
`);

console.log('Script SHA:', sha);

// Execute by SHA1 (more efficient for repeated calls)
const value = await client.evalSha(sha, {
  keys: ['mykey'],
  arguments: []
});
```

### Rate Limiting with Lua Script

```javascript
const rateLimitScript = `
  local key = KEYS[1]
  local limit = tonumber(ARGV[1])
  local window = tonumber(ARGV[2])
  local current = redis.call('INCR', key)
  if current == 1 then
    redis.call('EXPIRE', key, window)
  end
  if current > limit then
    return 0
  else
    return 1
  end
`;

async function checkRateLimit(userId, limit = 10, window = 60) {
  const allowed = await client.eval(rateLimitScript, {
    keys: [`ratelimit:${userId}`],
    arguments: [limit.toString(), window.toString()]
  });

  return allowed === 1;
}

// Usage
const allowed = await checkRateLimit('user:1000', 10, 60);
if (allowed) {
  console.log('Request allowed');
} else {
  console.log('Rate limit exceeded');
}
```

### Atomic Get and Delete

```javascript
const getAndDelete = `
  local value = redis.call('GET', KEYS[1])
  if value then
    redis.call('DEL', KEYS[1])
  end
  return value
`;

const value = await client.eval(getAndDelete, {
  keys: ['temp:key'],
  arguments: []
});
```

---

## JSON Support (RedisJSON)

### Basic JSON Operations

```javascript
// Set JSON document
await client.json.set('user:1', '$', {
  name: 'John Doe',
  email: '[email protected]',
  age: 30,
  tags: ['developer', 'redis']
});

// Get entire JSON document
const user = await client.json.get('user:1');
console.log(user);

// Get specific path
const name = await client.json.get('user:1', { path: '$.name' });
const age = await client.json.get('user:1', { path: '$.age' });
```

### Update JSON Fields

```javascript
// Set specific field
await client.json.set('user:1', '$.email', '"[email protected]"');

// Update nested field
await client.json.set('user:1', '$.address.city', '"New York"');

// Delete field
await client.json.del('user:1', '$.age');
```

### JSON Array Operations

```javascript
// Append to array
await client.json.arrAppend('user:1', '$.tags', '"nodejs"', '"javascript"');

// Get array length
const length = await client.json.arrLen('user:1', '$.tags');

// Pop from array
await client.json.arrPop('user:1', '$.tags', -1); // Pop last element

// Insert into array
await client.json.arrInsert('user:1', '$.tags', 0, '"beginner"');
```

### JSON Numeric Operations

```javascript
// Increment numeric value
await client.json.numIncrBy('user:1', '$.loginCount', 1);

// Multiply numeric value
await client.json.numMultBy('user:1', '$.score', 1.5);
```

---

## Time Series (RedisTimeSeries)

### Basic Time Series Operations

```javascript
// Create time series
await client.ts.create('temperature:sensor1', {
  RETENTION: 86400000, // Retain for 24 hours
  LABELS: { sensor: 'temp', location: 'room1' }
});

// Add data points
await client.ts.add('temperature:sensor1', '*', 22.5); // Current time
await client.ts.add('temperature:sensor1', Date.now(), 23.0); // Specific time

// Add multiple data points
await client.ts.mAdd([
  { key: 'temperature:sensor1', timestamp: '*', value: 22.8 },
  { key: 'temperature:sensor2', timestamp: '*', value: 21.5 }
]);
```

### Query Time Series

```javascript
// Get range of data
const data = await client.ts.range('temperature:sensor1', '-', '+');

// Get range with aggregation
const hourlyAvg = await client.ts.range('temperature:sensor1', '-', '+', {
  AGGREGATION: {
    type: 'AVG',
    timeBucket: 3600000 // 1 hour
  }
});

// Get last value
const latest = await client.ts.get('temperature:sensor1');
```

---

## Search and Query (RediSearch)

### Create Index

```javascript
// Create index on hash
await client.ft.create('idx:users', {
  '$.name': {
    type: 'TEXT',
    AS: 'name'
  },
  '$.age': {
    type: 'NUMERIC',
    AS: 'age'
  },
  '$.email': {
    type: 'TAG',
    AS: 'email'
  }
}, {
  ON: 'HASH',
  PREFIX: 'user:'
});
```

### Search Documents

```javascript
// Search by text
const results = await client.ft.search('idx:users', 'John');

// Search with filters
const filtered = await client.ft.search('idx:users', '@age:[25 35]');

// Search with multiple conditions
const complex = await client.ft.search('idx:users', '@name:John @age:[20 40]');

// Get search count
const count = await client.ft.search('idx:users', '*', { LIMIT: { from: 0, size: 0 } });
```

---

## Geospatial Operations

### Add Geo Points

```javascript
// Add location
await client.geoAdd('locations', {
  longitude: -122.4194,
  latitude: 37.7749,
  member: 'San Francisco'
});

// Add multiple locations
await client.geoAdd('locations', [
  { longitude: -118.2437, latitude: 34.0522, member: 'Los Angeles' },
  { longitude: -73.9352, latitude: 40.7306, member: 'New York' }
]);
```

### Query Geo Points

```javascript
// Get position
const position = await client.geoPos('locations', 'San Francisco');
console.log(position); // [{ longitude: -122.4194, latitude: 37.7749 }]

// Get distance between points
const distance = await client.geoDist('locations', 'San Francisco', 'Los Angeles', 'mi');
console.log(`Distance: ${distance} miles`);

// Search by radius
const nearby = await client.geoRadius('locations', {
  longitude: -122.4194,
  latitude: 37.7749
}, 500, 'mi');

// Search by member
const nearSF = await client.geoRadiusByMember('locations', 'San Francisco', 600, 'mi', {
  WITHDIST: true,
  WITHCOORD: true
});
```

---

## HyperLogLog (Cardinality Estimation)

```javascript
// Add elements
await client.pfAdd('unique:visitors', ['user1', 'user2', 'user3']);
await client.pfAdd('unique:visitors', ['user2', 'user4']); // user2 counted once

// Get count
const count = await client.pfCount('unique:visitors'); // ~4

// Merge multiple HyperLogLogs
await client.pfMerge('combined', ['unique:visitors:day1', 'unique:visitors:day2']);
```

---

## Bitmap Operations

```javascript
// Set bit
await client.setBit('login:2024-01-15', 100, 1); // User 100 logged in

// Get bit
const bit = await client.getBit('login:2024-01-15', 100); // 1

// Count set bits
const count = await client.bitCount('login:2024-01-15');

// Bitwise operations
await client.bitOp('AND', 'result', ['bitmap1', 'bitmap2']);
await client.bitOp('OR', 'result', ['bitmap1', 'bitmap2']);
await client.bitOp('XOR', 'result', ['bitmap1', 'bitmap2']);

// Find first bit
const pos = await client.bitPos('login:2024-01-15', 1); // First set bit
```

---

## Server and Connection Management

### Server Information

```javascript
// Get server info
const info = await client.info();
console.log(info);

// Get specific section
const stats = await client.info('stats');
const memory = await client.info('memory');

// Ping server
const pong = await client.ping(); // 'PONG'

// Get server time
const time = await client.time();
```

### Database Management

```javascript
// Select database (0-15 by default)
await client.select(1);

// Get database size
const size = await client.dbSize();

// Flush current database
await client.flushDb();

// Flush all databases
await client.flushAll();

// Save database to disk
await client.save();

// Background save
await client.bgSave();

// Get last save time
const lastSave = await client.lastSave();
```

### Client Management

```javascript
// Get client list
const clients = await client.clientList();

// Get client ID
const id = await client.clientId();

// Set client name
await client.clientSetName('my-app');

// Get client name
const name = await client.clientGetName();

// Get client info
const clientInfo = await client.clientInfo();
```

---

## Error Handling

### Connection Errors

```javascript
import { createClient } from 'redis';

const client = createClient({
  socket: {
    host: 'localhost',
    port: 6379,
    reconnectStrategy: (retries) => {
      if (retries > 10) {
        return new Error('Max retries reached');
      }
      return Math.min(retries * 100, 3000);
    }
  }
});

client.on('error', (err) => {
  console.error('Redis error:', err);
});

client.on('connect', () => {
  console.log('Connected to Redis');
});

client.on('reconnecting', () => {
  console.log('Reconnecting to Redis...');
});

client.on('ready', () => {
  console.log('Redis client ready');
});

await client.connect();
```

### Command Errors

```javascript
try {
  await client.get('nonexistent');
} catch (error) {
  console.error('Command error:', error);
}

// Multiple operations with individual error handling
const results = await client.multi()
  .set('key1', 'value1')
  .incr('not-a-number') // This will error
  .get('key1')
  .exec();

results.forEach((result, i) => {
  if (result instanceof Error) {
    console.error(`Command ${i} failed:`, result.message);
  } else {
    console.log(`Command ${i} result:`, result);
  }
});
```

---

## Connection Pooling and Performance

### Configure Connection Pool

```javascript
const client = createClient({
  socket: {
    host: 'localhost',
    port: 6379,
    connectTimeout: 5000,
    keepAlive: 5000
  },
  password: 'your_password'
});
```

### Readonly Replicas

```javascript
const client = createClient({
  socket: {
    host: 'localhost',
    port: 6379
  }
});

// Send read commands to replicas
await client.sendCommand(['READONLY']);
```

---

## Cluster Support

### Connect to Cluster

```javascript
import { createCluster } from 'redis';

const cluster = createCluster({
  rootNodes: [
    { url: 'redis://localhost:7000' },
    { url: 'redis://localhost:7001' },
    { url: 'redis://localhost:7002' }
  ]
});

cluster.on('error', (err) => console.error('Cluster error:', err));

await cluster.connect();

// Use cluster like regular client
await cluster.set('key', 'value');
const value = await cluster.get('key');

await cluster.quit();
```

---

## Complete Application Example

```javascript
import { createClient } from 'redis';
import dotenv from 'dotenv';

dotenv.config();

class RedisManager {
  constructor() {
    this.client = null;
    this.subscriber = null;
  }

  async connect() {
    this.client = createClient({
      url: process.env.REDIS_URL || 'redis://localhost:6379',
      password: process.env.REDIS_PASSWORD
    });

    this.client.on('error', (err) => {
      console.error('Redis Client Error:', err);
    });

    await this.client.connect();
    console.log('Connected to Redis');
  }

  async cacheGet(key) {
    const value = await this.client.get(key);
    return value ? JSON.parse(value) : null;
  }

  async cacheSet(key, value, ttl = 3600) {
    await this.client.set(key, JSON.stringify(value), { EX: ttl });
  }

  async invalidateCache(pattern) {
    const keys = [];
    for await (const key of this.client.scanIterator({ MATCH: pattern })) {
      keys.push(key);
    }
    if (keys.length > 0) {
      await this.client.del(keys);
    }
  }

  async trackUserActivity(userId, action) {
    const key = `activity:${userId}:${new Date().toISOString().split('T')[0]}`;
    await this.client.rPush(key, JSON.stringify({
      action,
      timestamp: Date.now()
    }));
    await this.client.expire(key, 86400 * 7); // Keep for 7 days
  }

  async getUserActivity(userId, date) {
    const key = `activity:${userId}:${date}`;
    const activities = await this.client.lRange(key, 0, -1);
    return activities.map(a => JSON.parse(a));
  }

  async incrementPageView(pageId) {
    await this.client.incr(`pageviews:${pageId}`);
  }

  async getPageViews(pageId) {
    const views = await this.client.get(`pageviews:${pageId}`);
    return parseInt(views || '0');
  }

  async addToLeaderboard(userId, score) {
    await this.client.zAdd('leaderboard', { score, value: userId });
  }

  async getLeaderboard(count = 10) {
    const results = await this.client.zRangeWithScores('leaderboard', 0, count - 1, {
      REV: true
    });
    return results.map(r => ({ userId: r.value, score: r.score }));
  }

  async disconnect() {
    if (this.subscriber) {
      await this.subscriber.quit();
    }
    if (this.client) {
      await this.client.quit();
    }
    console.log('Disconnected from Redis');
  }
}

// Usage
const redis = new RedisManager();
await redis.connect();

// Cache operations
await redis.cacheSet('user:1000', { name: 'John', email: '[email protected]' });
const user = await redis.cacheGet('user:1000');
console.log(user);

// Track activity
await redis.trackUserActivity('user:1000', 'login');
await redis.trackUserActivity('user:1000', 'view_profile');

// Get activity
const today = new Date().toISOString().split('T')[0];
const activities = await redis.getUserActivity('user:1000', today);
console.log(activities);

// Leaderboard
await redis.addToLeaderboard('user:1000', 500);
await redis.addToLeaderboard('user:2000', 750);
await redis.addToLeaderboard('user:3000', 250);

const topUsers = await redis.getLeaderboard(3);
console.log('Top users:', topUsers);

await redis.disconnect();
```

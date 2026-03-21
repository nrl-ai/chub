---
name: key-value
description: "Redis Python client (redis-py) for key-value storage, caching, and pub/sub messaging"
metadata:
  languages: "python"
  versions: "7.0.1"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "redis,database,cache,key-value,pubsub"
---

# Redis Python Client (redis-py) - Complete Integration Guide

## GOLDEN RULE

**ALWAYS use the official `redis` package (redis-py) for Redis integration.**

```bash
pip install redis
```

**For enhanced performance, install with hiredis support:**

```bash
pip install redis[hiredis]
```

**DO NOT use:**
- `redis-py-cluster` (deprecated, functionality merged into redis-py)
- `aioredis` (deprecated, async support now in redis-py)
- `walrus` (third-party wrapper)
- Any other unofficial Redis clients

The official `redis` package is maintained by Redis and supports both synchronous and asynchronous operations.

---

## Installation

### Basic Installation

```bash
pip install redis
```

### Installation with Performance Optimization

```bash
pip install redis[hiredis]
```

The hiredis library provides a compiled response parser that significantly improves performance.

### Installation with All Optional Dependencies

```bash
pip install redis[hiredis,cryptography]
```

### Environment Setup

Create a `.env` file in your project root:

```env
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=your_password_here
REDIS_DB=0
REDIS_URL=redis://username:password@localhost:6379/0
```

Install python-dotenv to load environment variables:

```bash
pip install python-dotenv
```

---

## Initialization

### Basic Connection (Localhost)

```python
import redis

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

# Test connection
r.ping()  # Returns True

# Use the client...

r.close()
```

### Connection with Environment Variables

```python
import redis
import os
from dotenv import load_dotenv

load_dotenv()

r = redis.Redis(
    host=os.getenv('REDIS_HOST', 'localhost'),
    port=int(os.getenv('REDIS_PORT', 6379)),
    password=os.getenv('REDIS_PASSWORD'),
    db=int(os.getenv('REDIS_DB', 0)),
    decode_responses=True
)

r.ping()
```

### Connection Using URL

```python
import redis
import os
from dotenv import load_dotenv

load_dotenv()

redis_url = os.getenv('REDIS_URL', 'redis://localhost:6379/0')
r = redis.from_url(redis_url, decode_responses=True)

r.ping()
```

### Connection with Authentication

```python
import redis

r = redis.Redis(
    host='redis.example.com',
    port=6379,
    username='default',
    password='your_password',
    db=0,
    decode_responses=True
)
```

### Connection with TLS/SSL

```python
import redis
import ssl

r = redis.Redis(
    host='redis.example.com',
    port=6380,
    password='your_password',
    ssl=True,
    ssl_certfile='/path/to/client-cert.pem',
    ssl_keyfile='/path/to/client-key.pem',
    ssl_ca_certs='/path/to/ca-cert.pem',
    ssl_cert_reqs=ssl.CERT_REQUIRED,
    decode_responses=True
)
```

### Connection Pool

```python
import redis

# Create connection pool
pool = redis.ConnectionPool(
    host='localhost',
    port=6379,
    password='your_password',
    db=0,
    max_connections=10,
    decode_responses=True
)

# Create client from pool
r = redis.Redis(connection_pool=pool)

# Multiple clients can share the pool
r2 = redis.Redis(connection_pool=pool)
```

### Context Manager (Auto-Close)

```python
import redis

with redis.Redis(host='localhost', port=6379, decode_responses=True) as r:
    r.set('key', 'value')
    value = r.get('key')
    print(value)
# Connection automatically closed
```

### Decode Responses

```python
# Without decode_responses (default - returns bytes)
r = redis.Redis(host='localhost', port=6379)
value = r.get('key')  # Returns: b'value'

# With decode_responses (returns strings)
r = redis.Redis(host='localhost', port=6379, decode_responses=True)
value = r.get('key')  # Returns: 'value'
```

---

## Core API Surfaces

## String Operations

### Basic SET and GET

```python
# Set a string value
r.set('key', 'value')

# Get a string value
value = r.get('key')
print(value)  # 'value'
```

### SET with Options

```python
# Set with expiration (seconds)
r.set('session:123', 'user_data', ex=3600)  # Expires in 1 hour

# Set with expiration (milliseconds)
r.set('temp:key', 'value', px=5000)  # Expires in 5 seconds

# Set only if key doesn't exist (NX)
result = r.set('key', 'value', nx=True)  # Returns True if set, None if exists

# Set only if key exists (XX)
result = r.set('key', 'new_value', xx=True)  # Returns True if set, None if not exists

# Get old value and set new value
old_value = r.set('key', 'new_value', get=True)  # Returns old value

# Set with multiple options
r.set('key', 'value', ex=60, nx=True)
```

### Multiple Keys

```python
# Set multiple keys at once
r.mset({
    'key1': 'value1',
    'key2': 'value2',
    'key3': 'value3'
})

# Get multiple keys at once
values = r.mget(['key1', 'key2', 'key3'])
print(values)  # ['value1', 'value2', 'value3']

# Set multiple only if none exist
result = r.msetnx({
    'key4': 'value4',
    'key5': 'value5'
})  # Returns True if all set, False if any exist
```

### Increment and Decrement

```python
# Set initial value
r.set('counter', 0)

# Increment by 1
r.incr('counter')  # Returns 1

# Increment by specific amount
r.incrby('counter', 10)  # Returns 11

# Increment float
r.incrbyfloat('price', 2.5)  # Increment by 2.5

# Decrement by 1
r.decr('counter')

# Decrement by specific amount
r.decrby('counter', 5)
```

### String Manipulation

```python
# Append to string
r.set('message', 'Hello')
r.append('message', ' World')  # 'Hello World'

# Get substring
substr = r.getrange('message', 0, 4)  # 'Hello'

# Get length
length = r.strlen('message')  # 11

# Set range
r.setrange('message', 6, 'Redis')  # 'Hello Redis'

# Get and delete
value = r.getdel('message')  # Returns value and deletes key

# Get and set expiration
value = r.getex('key', ex=60)  # Returns value and sets expiration
```

---

## Hash Operations

### Basic Hash Operations

```python
# Set a single field in a hash
r.hset('user:1000', 'name', 'John Doe')

# Get a single field from a hash
name = r.hget('user:1000', 'name')
print(name)  # 'John Doe'

# Set multiple fields at once
r.hset('user:1000', mapping={
    'name': 'John Doe',
    'email': '[email protected]',
    'age': 30
})

# Get all fields and values
user = r.hgetall('user:1000')
print(user)
# {'name': 'John Doe', 'email': '[email protected]', 'age': '30'}
```

### Advanced Hash Operations

```python
# Check if field exists
exists = r.hexists('user:1000', 'email')  # True

# Get all field names
fields = r.hkeys('user:1000')
# ['name', 'email', 'age']

# Get all values
values = r.hvals('user:1000')
# ['John Doe', '[email protected]', '30']

# Get number of fields
count = r.hlen('user:1000')  # 3

# Get multiple fields
user_data = r.hmget('user:1000', ['name', 'email'])
# ['John Doe', '[email protected]']

# Delete fields
r.hdel('user:1000', 'age')
r.hdel('user:1000', 'field1', 'field2', 'field3')  # Multiple fields

# Increment numeric field
r.hset('user:1000', 'login_count', 0)
r.hincrby('user:1000', 'login_count', 1)  # 1

# Increment float field
r.hincrbyfloat('user:1000', 'balance', 10.50)

# Set only if field doesn't exist
r.hsetnx('user:1000', 'created', '2024-01-01')

# Get random field
field = r.hrandfield('user:1000')  # Random field name
fields_and_values = r.hrandfield('user:1000', 2, withvalues=True)
# [('name', 'John Doe'), ('email', '[email protected]')]
```

### Scan Hash Fields

```python
# Scan hash fields (for large hashes)
cursor = 0
while True:
    cursor, fields = r.hscan('user:1000', cursor, count=10)
    for field, value in fields.items():
        print(f"{field}: {value}")
    if cursor == 0:
        break

# Using hscan_iter (simpler)
for field, value in r.hscan_iter('user:1000', count=10):
    print(f"{field}: {value}")
```

---

## List Operations

### Basic List Operations

```python
# Push elements to the right (end) of list
r.rpush('tasks', 'task1')
r.rpush('tasks', 'task2', 'task3')  # Multiple values

# Push elements to the left (beginning) of list
r.lpush('tasks', 'urgent_task')
r.lpush('tasks', 'task0', 'task-1')

# Get list length
length = r.llen('tasks')

# Get range of elements
tasks = r.lrange('tasks', 0, -1)  # Get all
first_three = r.lrange('tasks', 0, 2)  # Get first 3

# Get element by index
task = r.lindex('tasks', 0)

# Pop from right (end)
last_task = r.rpop('tasks')

# Pop from left (beginning)
first_task = r.lpop('tasks')

# Pop multiple elements
tasks = r.lpop('tasks', count=3)  # Pop 3 elements
```

### Advanced List Operations

```python
# Blocking pop (wait for element)
task = r.blpop('tasks', timeout=10)  # Wait up to 10 seconds
# Returns: ('tasks', 'task1') or None if timeout

task_right = r.brpop('tasks', timeout=10)

# Block on multiple lists
task = r.blpop(['queue1', 'queue2', 'queue3'], timeout=5)

# Set element at index
r.lset('tasks', 0, 'updated_task')

# Insert before/after element
r.linsert('tasks', 'BEFORE', 'task2', 'new_task')
r.linsert('tasks', 'AFTER', 'task2', 'another_task')

# Remove elements
r.lrem('tasks', 2, 'task1')  # Remove first 2 occurrences
r.lrem('tasks', -1, 'task2')  # Remove last occurrence
r.lrem('tasks', 0, 'task3')  # Remove all occurrences

# Trim list to range
r.ltrim('tasks', 0, 9)  # Keep only first 10 elements

# Move element between lists
element = r.rpoplpush('source', 'destination')

# Blocking move
moved = r.brpoplpush('source', 'destination', timeout=5)

# Move element (new command)
moved = r.lmove('source', 'destination', 'LEFT', 'RIGHT')
moved = r.blmove('source', 'destination', 'RIGHT', 'LEFT', timeout=5)
```

---

## Set Operations

### Basic Set Operations

```python
# Add members to set
r.sadd('tags', 'javascript')
r.sadd('tags', 'nodejs', 'redis', 'database')

# Check if member exists
exists = r.sismember('tags', 'nodejs')  # True

# Get all members
all_tags = r.smembers('tags')
# {'javascript', 'nodejs', 'redis', 'database'}

# Get number of members
count = r.scard('tags')  # 4

# Remove members
r.srem('tags', 'database')
r.srem('tags', 'nodejs', 'redis')

# Pop random member
random_tag = r.spop('tags')

# Pop multiple random members
random_tags = r.spop('tags', count=2)

# Get random member without removing
random = r.srandmember('tags')
random_three = r.srandmember('tags', 3)
```

### Set Operations Between Multiple Sets

```python
# Create sets
r.sadd('set1', 'a', 'b', 'c')
r.sadd('set2', 'b', 'c', 'd')
r.sadd('set3', 'c', 'd', 'e')

# Union (combine all unique members)
union = r.sunion('set1', 'set2')
# {'a', 'b', 'c', 'd'}

# Store union in new set
r.sunionstore('result', 'set1', 'set2')

# Intersection (common members)
inter = r.sinter('set1', 'set2')
# {'b', 'c'}

# Store intersection
r.sinterstore('result', 'set1', 'set2')

# Intersection of multiple sets
inter = r.sinter('set1', 'set2', 'set3')  # {'c'}

# Difference (members in first set but not in others)
diff = r.sdiff('set1', 'set2')
# {'a'}

# Store difference
r.sdiffstore('result', 'set1', 'set2')

# Move member between sets
r.smove('set1', 'set2', 'a')

# Check multiple members
result = r.smismember('tags', 'nodejs', 'python', 'redis')
# [True, False, True]
```

### Scan Set Members

```python
# Scan set members (for large sets)
cursor = 0
while True:
    cursor, members = r.sscan('tags', cursor, count=10)
    for member in members:
        print(member)
    if cursor == 0:
        break

# Using sscan_iter (simpler)
for member in r.sscan_iter('tags', match='node*', count=10):
    print(member)
```

---

## Sorted Set Operations

### Basic Sorted Set Operations

```python
# Add members with scores
r.zadd('leaderboard', {'player1': 100})
r.zadd('leaderboard', {'player2': 200, 'player3': 150})

# Add with options
r.zadd('leaderboard', {'player4': 175}, nx=True)  # Only if doesn't exist
r.zadd('leaderboard', {'player1': 120}, xx=True)  # Only if exists
r.zadd('leaderboard', {'player1': 130}, gt=True)  # Only if new score greater
r.zadd('leaderboard', {'player1': 110}, lt=True)  # Only if new score less

# Get rank (0-based, lowest to highest)
rank = r.zrank('leaderboard', 'player1')  # 0

# Get reverse rank (highest to lowest)
rev_rank = r.zrevrank('leaderboard', 'player2')  # 0

# Get score
score = r.zscore('leaderboard', 'player2')  # 200.0

# Get number of members
count = r.zcard('leaderboard')  # 3

# Increment score
r.zincrby('leaderboard', 50, 'player1')  # 150.0
```

### Range Queries

```python
# Get range by rank (lowest to highest)
bottom3 = r.zrange('leaderboard', 0, 2)
# ['player1', 'player3', 'player2']

# Get range with scores
with_scores = r.zrange('leaderboard', 0, 2, withscores=True)
# [('player1', 150.0), ('player3', 150.0), ('player2', 200.0)]

# Get range by rank (highest to lowest)
top3 = r.zrevrange('leaderboard', 0, 2)

# Get range with scores (descending)
top3_scores = r.zrevrange('leaderboard', 0, 2, withscores=True)

# Get range by score
range_result = r.zrangebyscore('leaderboard', 100, 200)

# Get range by score with limit
limited = r.zrangebyscore('leaderboard', 100, 200, start=0, num=10)

# Get range by score with scores
range_scores = r.zrangebyscore('leaderboard', 100, 200, withscores=True)

# Get reverse range by score
rev_range = r.zrevrangebyscore('leaderboard', 200, 100)

# Infinity boundaries
high_scores = r.zrangebyscore('leaderboard', 150, '+inf')
low_scores = r.zrangebyscore('leaderboard', '-inf', 150)
```

### Advanced Sorted Set Operations

```python
# Count members in score range
count = r.zcount('leaderboard', 100, 200)

# Count members by lexicographical range
r.zadd('words', {'apple': 0, 'banana': 0, 'cherry': 0})
lex_count = r.zlexcount('words', '[a', '[c')

# Get lexicographical range
lex_range = r.zrangebylex('words', '[a', '[c')

# Remove members
r.zrem('leaderboard', 'player1')
r.zrem('leaderboard', 'player2', 'player3')

# Remove by rank range
r.zremrangebyrank('leaderboard', 0, 1)  # Remove bottom 2

# Remove by score range
r.zremrangebyscore('leaderboard', 0, 100)

# Remove by lex range
r.zremrangebylex('words', '[a', '[b')

# Pop highest/lowest scoring member
highest = r.zpopmax('leaderboard')  # [('player2', 200.0)]
lowest = r.zpopmin('leaderboard')  # [('player1', 100.0)]

# Pop with count
top3 = r.zpopmax('leaderboard', count=3)

# Blocking pop
member = r.bzpopmax('leaderboard', timeout=5)
# ('leaderboard', 'player2', 200.0)
min_member = r.bzpopmin('leaderboard', timeout=5)

# Block on multiple sorted sets
member = r.bzpopmax(['set1', 'set2', 'set3'], timeout=5)
```

### Sorted Set Operations Between Multiple Sets

```python
# Union of sorted sets
r.zunionstore('result', ['set1', 'set2'])

# Union with weights
r.zunionstore('result', {'set1': 2, 'set2': 3})

# Union with aggregate function
r.zunionstore('result', ['set1', 'set2'], aggregate='MAX')  # or 'MIN', 'SUM'

# Intersection
r.zinterstore('result', ['set1', 'set2'])

# Intersection with weights and aggregate
r.zinterstore('result', {'set1': 1, 'set2': 2}, aggregate='SUM')

# Get union without storing
union = r.zunion(['set1', 'set2'], withscores=True)

# Get intersection without storing
inter = r.zinter(['set1', 'set2'], withscores=True)

# Difference between sorted sets
diff = r.zdiff(['set1', 'set2'])
diff_scores = r.zdiff(['set1', 'set2'], withscores=True)

# Store difference
r.zdiffstore('result', ['set1', 'set2'])
```

### Scan Sorted Set

```python
# Scan sorted set members
cursor = 0
while True:
    cursor, members = r.zscan('leaderboard', cursor, count=10)
    for member, score in members:
        print(f"{member}: {score}")
    if cursor == 0:
        break

# Using zscan_iter (simpler)
for member, score in r.zscan_iter('leaderboard', count=10):
    print(f"{member}: {score}")
```

---

## Key Management Operations

### Key Operations

```python
# Check if key exists
exists = r.exists('mykey')  # 1 if exists, 0 if not
multi_exists = r.exists('key1', 'key2', 'key3')  # Count

# Delete keys
r.delete('mykey')
r.delete('key1', 'key2', 'key3')

# Set expiration in seconds
r.expire('mykey', 60)  # Expire in 60 seconds

# Set expiration at specific timestamp
import time
timestamp = int(time.time()) + 3600
r.expireat('mykey', timestamp)

# Set expiration in milliseconds
r.pexpire('mykey', 60000)  # 60 seconds

# Set expiration at timestamp (milliseconds)
r.pexpireat('mykey', int(time.time() * 1000) + 60000)

# Get time to live in seconds
ttl = r.ttl('mykey')  # -1 if no expiration, -2 if not exists

# Get time to live in milliseconds
pttl = r.pttl('mykey')

# Remove expiration
r.persist('mykey')

# Rename key
r.rename('oldkey', 'newkey')

# Rename only if new key doesn't exist
renamed = r.renamenx('oldkey', 'newkey')  # Returns True if renamed

# Get key type
key_type = r.type('mykey')  # 'string', 'list', 'set', etc.

# Get random key
random_key = r.randomkey()

# Copy key
r.copy('source', 'destination')
r.copy('source', 'destination', replace=True)  # Overwrite if exists

# Touch keys (update last access time)
r.touch('key1', 'key2', 'key3')

# Get object encoding
encoding = r.object('encoding', 'mykey')

# Get object idle time
idle_time = r.object('idletime', 'mykey')
```

### Scanning Keys

```python
# Scan all keys (use instead of KEYS for production)
keys = []
cursor = 0
while True:
    cursor, partial_keys = r.scan(cursor, count=100)
    keys.extend(partial_keys)
    if cursor == 0:
        break

# Using scan_iter (simpler)
for key in r.scan_iter(count=100):
    print(key)

# Scan with pattern
for key in r.scan_iter(match='user:*', count=100):
    print(key)

# Scan specific key type
for key in r.scan_iter(_type='string', count=100):
    print(key)

# Scan with pattern and type
for key in r.scan_iter(match='session:*', _type='hash', count=100):
    print(key)
```

---

## Transactions

### Basic Transaction (MULTI/EXEC)

```python
# Execute multiple commands atomically
r.set('another-key', 'another-value')

pipe = r.pipeline()
pipe.set('key', 'value')
pipe.get('another-key')
pipe.incr('counter')
results = pipe.execute()

print(results)  # [True, 'another-value', 1]
```

### Pipeline Without Transaction

```python
# Execute commands in pipeline without atomicity
pipe = r.pipeline(transaction=False)
pipe.set('key1', 'value1')
pipe.set('key2', 'value2')
pipe.get('key1')
results = pipe.execute()
```

### Transaction with Error Handling

```python
try:
    pipe = r.pipeline()
    pipe.set('key1', 'value1')
    pipe.set('key2', 'value2')
    pipe.get('key1')
    results = pipe.execute()

    for i, result in enumerate(results):
        print(f"Command {i} result: {result}")
except redis.exceptions.ResponseError as e:
    print(f"Transaction failed: {e}")
```

### Transaction with WATCH (Optimistic Locking)

```python
# Watch a key for changes
with r.pipeline() as pipe:
    while True:
        try:
            pipe.watch('balance')
            balance = int(pipe.get('balance') or 0)

            if balance >= 100:
                pipe.multi()
                pipe.decrby('balance', 100)
                pipe.incrby('purchases', 1)
                pipe.execute()
                print('Purchase successful')
                break
            else:
                pipe.unwatch()
                print('Insufficient balance')
                break
        except redis.WatchError:
            print('Balance was modified, retrying...')
            continue
```

### Complex Transaction Example

```python
def transfer_money(r, from_account, to_account, amount):
    """Transfer money between accounts with optimistic locking"""
    with r.pipeline() as pipe:
        while True:
            try:
                pipe.watch(from_account, to_account)

                from_balance = float(pipe.get(from_account) or 0)

                if from_balance < amount:
                    pipe.unwatch()
                    raise ValueError('Insufficient funds')

                pipe.multi()
                pipe.incrbyfloat(from_account, -amount)
                pipe.incrbyfloat(to_account, amount)
                pipe.execute()
                return True
            except redis.WatchError:
                # Another client modified the keys, retry
                continue

# Usage
try:
    transfer_money(r, 'account:1', 'account:2', 50.00)
    print('Transfer successful')
except ValueError as e:
    print(f'Transfer failed: {e}')
```

---

## Pipelining

### Basic Pipelining

```python
# Execute multiple commands in one round trip
pipe = r.pipeline(transaction=False)
pipe.set('key1', 'value1')
pipe.set('key2', 'value2')
pipe.get('key1')
pipe.mget(['key1', 'key2'])
results = pipe.execute()

print(results)  # [True, True, 'value1', ['value1', 'value2']]
```

### Large Batch Operations

```python
def batch_set_keys(r, key_value_pairs):
    """Set many keys efficiently using pipeline"""
    pipe = r.pipeline(transaction=False)

    for key, value in key_value_pairs.items():
        pipe.set(key, value)

    results = pipe.execute()
    return results

# Usage
data = {
    'user:1': 'Alice',
    'user:2': 'Bob',
    'user:3': 'Charlie'
}

batch_set_keys(r, data)
```

### Chunked Pipeline

```python
def batch_operation_chunked(r, operations, chunk_size=1000):
    """Execute operations in chunks to avoid memory issues"""
    results = []

    for i in range(0, len(operations), chunk_size):
        chunk = operations[i:i + chunk_size]
        pipe = r.pipeline(transaction=False)

        for op in chunk:
            getattr(pipe, op['command'])(*op['args'])

        results.extend(pipe.execute())

    return results

# Usage
operations = [
    {'command': 'set', 'args': [f'key:{i}', f'value:{i}']}
    for i in range(10000)
]

batch_operation_chunked(r, operations, chunk_size=1000)
```

---

## Pub/Sub

### Basic Subscriber

```python
import redis

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

# Create PubSub object
pubsub = r.pubsub()

# Subscribe to channel
pubsub.subscribe('notifications')

# Listen for messages
for message in pubsub.listen():
    if message['type'] == 'message':
        print(f"Received: {message['data']}")
```

### Basic Publisher

```python
import redis

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

# Publish messages
r.publish('notifications', 'Hello, World!')
r.publish('notifications', 'System update')

print('Messages published')
```

### Multiple Channels

```python
# Subscribe to multiple channels
pubsub = r.pubsub()
pubsub.subscribe('channel1', 'channel2', 'channel3')

for message in pubsub.listen():
    if message['type'] == 'message':
        print(f"Channel: {message['channel']}, Message: {message['data']}")
```

### Pattern-Based Subscription

```python
# Subscribe to channels matching pattern
pubsub = r.pubsub()
pubsub.psubscribe('user:*')

for message in pubsub.listen():
    if message['type'] == 'pmessage':
        print(f"Pattern: {message['pattern']}, Channel: {message['channel']}, Message: {message['data']}")

# Publish to matching channels
r.publish('user:1000', 'User 1000 logged in')
r.publish('user:2000', 'User 2000 logged out')
```

### Message Handler Functions

```python
def message_handler(message):
    """Handle messages from channel"""
    print(f"Received: {message['data']}")

def error_handler(message):
    """Handle errors"""
    print(f"Error: {message}")

# Subscribe with handler
pubsub = r.pubsub()
pubsub.subscribe(**{
    'notifications': message_handler,
    'errors': error_handler
})

# Run event loop
thread = pubsub.run_in_thread(sleep_time=0.01)

# Do other work...

# Stop listening
thread.stop()
```

### Unsubscribe

```python
# Unsubscribe from specific channel
pubsub.unsubscribe('channel1')

# Unsubscribe from all channels
pubsub.unsubscribe()

# Unsubscribe from pattern
pubsub.punsubscribe('user:*')
```

### Complete Pub/Sub Example

```python
import redis
import json
import threading
import time

def subscriber_worker():
    """Subscriber in separate thread"""
    r = redis.Redis(host='localhost', port=6379, decode_responses=True)
    pubsub = r.pubsub()
    pubsub.subscribe('chat:room1')

    for message in pubsub.listen():
        if message['type'] == 'message':
            data = json.loads(message['data'])
            print(f"{data['user']}: {data['text']}")

def publisher_worker():
    """Publisher in separate thread"""
    r = redis.Redis(host='localhost', port=6379, decode_responses=True)

    time.sleep(1)  # Wait for subscriber

    # Publish chat messages
    r.publish('chat:room1', json.dumps({
        'user': 'Alice',
        'text': 'Hello everyone!'
    }))

    r.publish('chat:room1', json.dumps({
        'user': 'Bob',
        'text': 'Hi Alice!'
    }))

# Start subscriber
sub_thread = threading.Thread(target=subscriber_worker, daemon=True)
sub_thread.start()

# Start publisher
pub_thread = threading.Thread(target=publisher_worker)
pub_thread.start()
pub_thread.join()

time.sleep(2)  # Let subscriber process messages
```

---

## Redis Streams

### Add to Stream (XADD)

```python
# Add entry to stream
entry_id = r.xadd('events', {
    'user': 'alice',
    'action': 'login',
    'timestamp': time.time()
})

print(f'Entry ID: {entry_id}')  # '1234567890123-0'

# Add with specific ID
r.xadd('events', {'user': 'bob', 'action': 'logout'}, id='1234567890123-1')

# Add with maxlen (limit stream size)
r.xadd('events', {'user': 'charlie', 'action': 'signup'}, maxlen=1000)

# Add with approximate trimming
r.xadd('events', {'user': 'dave', 'action': 'purchase'}, maxlen=1000, approximate=True)

# Add with minid trimming
r.xadd('events', {'data': 'value'}, minid='1234567890000-0')
```

### Read from Stream (XREAD)

```python
# Read entries from beginning
messages = r.xread({'events': '0'}, count=10)

for stream, entries in messages:
    for entry_id, data in entries:
        print(f"ID: {entry_id}, Data: {data}")

# Read from multiple streams
messages = r.xread({'stream1': '0', 'stream2': '0'})

# Blocking read (wait for new entries)
messages = r.xread({'events': '$'}, block=5000)  # Wait up to 5 seconds

# Read only new messages
messages = r.xread({'events': '$'}, block=0)  # Block indefinitely
```

### Stream Range Queries

```python
# Read all entries
all_entries = r.xrange('events', '-', '+')

# Read range with IDs
range_entries = r.xrange('events', '1234567890000', '1234567899999')

# Read with limit
limited = r.xrange('events', '-', '+', count=100)

# Reverse range
reverse = r.xrevrange('events', '+', '-', count=10)

# Iterate through entries
for entry_id, data in r.xrange('events', '-', '+'):
    print(f"{entry_id}: {data}")
```

### Stream Length and Info

```python
# Get stream length
length = r.xlen('events')

# Get stream info
info = r.xinfo_stream('events')
print(f"Length: {info['length']}")
print(f"First entry: {info['first-entry']}")
print(f"Last entry: {info['last-entry']}")
```

### Consumer Groups

```python
# Create consumer group
try:
    r.xgroup_create('events', 'processors', '0', mkstream=True)
except redis.exceptions.ResponseError:
    # Group already exists
    pass

# Read as consumer group
messages = r.xreadgroup(
    'processors',
    'consumer1',
    {'events': '>'},  # '>' means undelivered messages
    count=10,
    block=5000
)

# Process messages and acknowledge
for stream, entries in messages:
    for entry_id, data in entries:
        # Process message
        print(f'Processing: {entry_id}, {data}')

        # Acknowledge message
        r.xack('events', 'processors', entry_id)

# Get pending messages
pending = r.xpending('events', 'processors')
print(f"Pending messages: {pending['pending']}")

# Get detailed pending info
pending_details = r.xpending_range('events', 'processors', '-', '+', count=10)

# Claim pending messages (take over from another consumer)
claimed = r.xclaim('events', 'processors', 'consumer2', 3600000, ['1234567890123-0'])
```

### Delete from Stream

```python
# Delete specific entries
r.xdel('events', '1234567890123-0', '1234567890123-1')

# Trim stream
r.xtrim('events', maxlen=1000)

# Approximate trim
r.xtrim('events', maxlen=1000, approximate=True)

# Trim by minid
r.xtrim('events', minid='1234567890000-0')
```

### Complete Stream Example

```python
import redis
import time
import json

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def add_event(event_data):
    """Add event to stream"""
    return r.xadd('events', event_data)

def process_events():
    """Process events using consumer group"""
    # Create group if not exists
    try:
        r.xgroup_create('events', 'workers', '0', mkstream=True)
    except redis.exceptions.ResponseError:
        pass

    while True:
        messages = r.xreadgroup(
            'workers',
            'worker1',
            {'events': '>'},
            count=10,
            block=5000
        )

        if messages:
            for stream, entries in messages:
                for entry_id, data in entries:
                    try:
                        # Process event
                        print(f'Processing event: {data}')

                        # Acknowledge
                        r.xack('events', 'workers', entry_id)
                    except Exception as e:
                        print(f'Error processing: {e}')

# Add events
add_event({'type': 'user_login', 'user': 'alice'})
add_event({'type': 'user_logout', 'user': 'bob'})

# Process events
process_events()
```

---

## Scripting with Lua

### Basic Script Execution (EVAL)

```python
# Execute Lua script
script = "return redis.call('SET', KEYS[1], ARGV[1])"
result = r.eval(script, 1, 'mykey', 'myvalue')
print(result)  # b'OK' or 'OK' with decode_responses=True
```

### Script with Multiple Operations

```python
script = """
local current = redis.call('GET', KEYS[1])
if current == false then
    current = 0
end
local new = tonumber(current) + tonumber(ARGV[1])
redis.call('SET', KEYS[1], new)
return new
"""

new_value = r.eval(script, 1, 'counter', 5)
print(f'New counter value: {new_value}')
```

### Register Script (SCRIPT LOAD / EVALSHA)

```python
# Register script
script = "return redis.call('GET', KEYS[1])"
sha = r.script_load(script)
print(f'Script SHA: {sha}')

# Execute by SHA (more efficient for repeated calls)
value = r.evalsha(sha, 1, 'mykey')

# Check if script exists
exists = r.script_exists(sha)
print(f'Script exists: {exists}')  # [True]

# Flush all scripts
r.script_flush()
```

### Rate Limiting with Lua Script

```python
rate_limit_script = """
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
"""

def check_rate_limit(user_id, limit=10, window=60):
    """Check if user is within rate limit"""
    allowed = r.eval(
        rate_limit_script,
        1,
        f'ratelimit:{user_id}',
        limit,
        window
    )
    return allowed == 1

# Usage
if check_rate_limit('user:1000', limit=10, window=60):
    print('Request allowed')
else:
    print('Rate limit exceeded')
```

### Atomic Get and Delete

```python
get_and_delete = """
local value = redis.call('GET', KEYS[1])
if value then
    redis.call('DEL', KEYS[1])
end
return value
"""

value = r.eval(get_and_delete, 1, 'temp:key')
print(f'Retrieved and deleted: {value}')
```

### Script Class for Reusability

```python
class RedisScripts:
    """Reusable Redis Lua scripts"""

    def __init__(self, redis_client):
        self.r = redis_client

        # Load scripts on initialization
        self.rate_limit_sha = self.r.script_load("""
            local key = KEYS[1]
            local limit = tonumber(ARGV[1])
            local window = tonumber(ARGV[2])
            local current = redis.call('INCR', key)
            if current == 1 then
                redis.call('EXPIRE', key, window)
            end
            return current <= limit and 1 or 0
        """)

        self.atomic_increment_sha = self.r.script_load("""
            local current = redis.call('GET', KEYS[1])
            if not current then
                current = 0
            end
            local new = tonumber(current) + tonumber(ARGV[1])
            redis.call('SET', KEYS[1], new)
            return new
        """)

    def rate_limit(self, key, limit, window):
        return self.r.evalsha(self.rate_limit_sha, 1, key, limit, window) == 1

    def atomic_increment(self, key, amount):
        return self.r.evalsha(self.atomic_increment_sha, 1, key, amount)

# Usage
scripts = RedisScripts(r)
allowed = scripts.rate_limit('user:1000', 10, 60)
new_value = scripts.atomic_increment('counter', 5)
```

---

## Geospatial Operations

### Add Geo Points

```python
# Add location
r.geoadd('locations', -122.4194, 37.7749, 'San Francisco')

# Add multiple locations
r.geoadd('locations',
    -118.2437, 34.0522, 'Los Angeles',
    -73.9352, 40.7306, 'New York'
)
```

### Query Geo Points

```python
# Get position
position = r.geopos('locations', 'San Francisco')
print(position)  # [(-122.4194, 37.7749)]

# Get multiple positions
positions = r.geopos('locations', 'San Francisco', 'Los Angeles')

# Get distance between points
distance = r.geodist('locations', 'San Francisco', 'Los Angeles', unit='mi')
print(f'Distance: {distance} miles')

# Available units: 'm', 'km', 'mi', 'ft'

# Search by radius (from coordinates)
nearby = r.georadius('locations', -122.4194, 37.7749, 500, unit='mi')

# Search with additional info
detailed = r.georadius(
    'locations', -122.4194, 37.7749, 500, unit='mi',
    withdist=True,
    withcoord=True,
    withhash=True,
    count=10,
    sort='ASC'
)

# Search by member
near_sf = r.georadiusbymember('locations', 'San Francisco', 600, unit='mi')

# Search by member with details
detailed = r.georadiusbymember(
    'locations', 'San Francisco', 600, unit='mi',
    withdist=True,
    withcoord=True
)

# Get geohash
geohash = r.geohash('locations', 'San Francisco')
```

---

## HyperLogLog (Cardinality Estimation)

```python
# Add elements
r.pfadd('unique:visitors', 'user1', 'user2', 'user3')
r.pfadd('unique:visitors', 'user2', 'user4')  # user2 counted once

# Get count
count = r.pfcount('unique:visitors')  # ~4
print(f'Unique visitors: {count}')

# Count multiple HyperLogLogs
total = r.pfcount('unique:day1', 'unique:day2', 'unique:day3')

# Merge multiple HyperLogLogs
r.pfmerge('unique:combined', 'unique:day1', 'unique:day2')
```

---

## Bitmap Operations

```python
# Set bit
r.setbit('login:2024-01-15', 100, 1)  # User 100 logged in

# Get bit
bit = r.getbit('login:2024-01-15', 100)  # 1

# Count set bits
count = r.bitcount('login:2024-01-15')

# Count bits in range (bytes)
count = r.bitcount('login:2024-01-15', 0, 10)

# Bitwise operations
r.bitop('AND', 'result', 'bitmap1', 'bitmap2')
r.bitop('OR', 'result', 'bitmap1', 'bitmap2')
r.bitop('XOR', 'result', 'bitmap1', 'bitmap2')
r.bitop('NOT', 'result', 'bitmap1')

# Find first bit
pos = r.bitpos('login:2024-01-15', 1)  # First set bit
pos = r.bitpos('login:2024-01-15', 0)  # First unset bit

# Find bit in range
pos = r.bitpos('login:2024-01-15', 1, 0, 10)
```

---

## Server and Connection Management

### Server Information

```python
# Get server info
info = r.info()
print(info)

# Get specific section
stats = r.info('stats')
memory = r.info('memory')
replication = r.info('replication')

# Ping server
pong = r.ping()  # True

# Echo message
echo = r.echo('Hello Redis')

# Get server time
server_time = r.time()  # (seconds, microseconds)
```

### Database Management

```python
# Select database (0-15 by default)
r.execute_command('SELECT', 1)

# Get database size
size = r.dbsize()

# Flush current database
r.flushdb()

# Flush all databases
r.flushall()

# Save database to disk
r.save()

# Background save
r.bgsave()

# Get last save time
last_save = r.lastsave()  # Unix timestamp

# Background rewrite AOF
r.bgrewriteaof()
```

### Client Management

```python
# Get client list
clients = r.client_list()

# Get client ID
client_id = r.client_id()

# Set client name
r.client_setname('my-app')

# Get client name
name = r.client_getname()

# Get client info
info = r.client_info()

# Kill client
r.client_kill('127.0.0.1:6379')

# Pause all clients
r.client_pause(1000)  # Pause for 1000ms
```

### Memory Management

```python
# Get memory stats
memory_stats = r.memory_stats()

# Get memory usage of key
usage = r.memory_usage('mykey')

# Purge memory
r.memory_purge()
```

---

## Async/Await Support

### Async Redis Client

```python
import asyncio
import redis.asyncio as aioredis

async def main():
    # Create async client
    r = await aioredis.from_url('redis://localhost:6379', decode_responses=True)

    # Set value
    await r.set('key', 'value')

    # Get value
    value = await r.get('key')
    print(value)

    # Close connection
    await r.close()

# Run async code
asyncio.run(main())
```

### Async Pipeline

```python
async def async_pipeline():
    r = await aioredis.from_url('redis://localhost:6379', decode_responses=True)

    async with r.pipeline(transaction=True) as pipe:
        await pipe.set('key1', 'value1')
        await pipe.set('key2', 'value2')
        await pipe.get('key1')
        results = await pipe.execute()

    print(results)
    await r.close()

asyncio.run(async_pipeline())
```

### Async Pub/Sub

```python
async def reader(channel: aioredis.client.PubSub):
    while True:
        message = await channel.get_message(ignore_subscribe_messages=True)
        if message is not None:
            print(f"Received: {message['data']}")

async def async_pubsub():
    r = await aioredis.from_url('redis://localhost:6379', decode_responses=True)

    async with r.pubsub() as pubsub:
        await pubsub.subscribe('notifications')

        # Create task to read messages
        future = asyncio.create_task(reader(pubsub))

        # Publish messages
        await r.publish('notifications', 'Hello Async!')

        await asyncio.sleep(1)
        future.cancel()

    await r.close()

asyncio.run(async_pubsub())
```

---

## Error Handling

### Connection Errors

```python
import redis
from redis.exceptions import ConnectionError, TimeoutError, RedisError

try:
    r = redis.Redis(
        host='localhost',
        port=6379,
        socket_connect_timeout=5,
        socket_timeout=5,
        retry_on_timeout=True
    )
    r.ping()
except ConnectionError as e:
    print(f'Connection error: {e}')
except TimeoutError as e:
    print(f'Timeout error: {e}')
except RedisError as e:
    print(f'Redis error: {e}')
```

### Command Errors

```python
from redis.exceptions import ResponseError, DataError

try:
    r.get('nonexistent')
except ResponseError as e:
    print(f'Command error: {e}')
except DataError as e:
    print(f'Data error: {e}')
```

### Pipeline Errors

```python
pipe = r.pipeline()
pipe.set('key1', 'value1')
pipe.incr('not-a-number')  # Will error
pipe.get('key1')

try:
    results = pipe.execute()
except ResponseError as e:
    print(f'Pipeline error: {e}')
```

### Retry Logic

```python
import time
from redis.exceptions import ConnectionError

def redis_operation_with_retry(operation, max_retries=3):
    """Execute Redis operation with retry logic"""
    for attempt in range(max_retries):
        try:
            return operation()
        except ConnectionError:
            if attempt == max_retries - 1:
                raise
            time.sleep(2 ** attempt)  # Exponential backoff

# Usage
result = redis_operation_with_retry(lambda: r.get('mykey'))
```

---

## Connection Pooling

### Configure Connection Pool

```python
pool = redis.ConnectionPool(
    host='localhost',
    port=6379,
    password='your_password',
    db=0,
    max_connections=50,
    socket_timeout=5,
    socket_connect_timeout=5,
    socket_keepalive=True,
    socket_keepalive_options={
        1: 1,  # TCP_KEEPIDLE
        2: 1,  # TCP_KEEPINTVL
        3: 3   # TCP_KEEPCNT
    },
    decode_responses=True
)

r = redis.Redis(connection_pool=pool)
```

### Multiple Clients Sharing Pool

```python
pool = redis.ConnectionPool(
    host='localhost',
    port=6379,
    max_connections=20,
    decode_responses=True
)

# Multiple clients share the pool
client1 = redis.Redis(connection_pool=pool)
client2 = redis.Redis(connection_pool=pool)
client3 = redis.Redis(connection_pool=pool)
```

---

## Cluster Support

### Connect to Cluster

```python
from redis.cluster import RedisCluster

# Connect to cluster
cluster = RedisCluster(
    host='localhost',
    port=7000,
    decode_responses=True
)

# Use cluster like regular client
cluster.set('key', 'value')
value = cluster.get('key')

cluster.close()
```

### Cluster with Multiple Nodes

```python
from redis.cluster import RedisCluster, ClusterNode

startup_nodes = [
    ClusterNode('localhost', 7000),
    ClusterNode('localhost', 7001),
    ClusterNode('localhost', 7002)
]

cluster = RedisCluster(
    startup_nodes=startup_nodes,
    decode_responses=True
)
```

---

## Complete Application Example

```python
import redis
import json
import time
import os
from typing import Optional, List, Dict, Any
from dotenv import load_dotenv

load_dotenv()

class RedisManager:
    """Redis manager with common operations"""

    def __init__(self):
        self.pool = redis.ConnectionPool(
            host=os.getenv('REDIS_HOST', 'localhost'),
            port=int(os.getenv('REDIS_PORT', 6379)),
            password=os.getenv('REDIS_PASSWORD'),
            db=int(os.getenv('REDIS_DB', 0)),
            max_connections=10,
            decode_responses=True
        )
        self.client = redis.Redis(connection_pool=self.pool)

    def cache_get(self, key: str) -> Optional[Any]:
        """Get value from cache"""
        value = self.client.get(key)
        return json.loads(value) if value else None

    def cache_set(self, key: str, value: Any, ttl: int = 3600):
        """Set value in cache with TTL"""
        self.client.set(key, json.dumps(value), ex=ttl)

    def invalidate_cache(self, pattern: str):
        """Invalidate cache keys matching pattern"""
        keys = list(self.client.scan_iter(match=pattern))
        if keys:
            self.client.delete(*keys)

    def track_user_activity(self, user_id: str, action: str):
        """Track user activity"""
        today = time.strftime('%Y-%m-%d')
        key = f'activity:{user_id}:{today}'

        self.client.rpush(key, json.dumps({
            'action': action,
            'timestamp': time.time()
        }))
        self.client.expire(key, 86400 * 7)  # Keep for 7 days

    def get_user_activity(self, user_id: str, date: str) -> List[Dict]:
        """Get user activity for date"""
        key = f'activity:{user_id}:{date}'
        activities = self.client.lrange(key, 0, -1)
        return [json.loads(a) for a in activities]

    def increment_page_view(self, page_id: str) -> int:
        """Increment page view counter"""
        return self.client.incr(f'pageviews:{page_id}')

    def get_page_views(self, page_id: str) -> int:
        """Get page view count"""
        views = self.client.get(f'pageviews:{page_id}')
        return int(views) if views else 0

    def add_to_leaderboard(self, user_id: str, score: float):
        """Add or update user score in leaderboard"""
        self.client.zadd('leaderboard', {user_id: score})

    def get_leaderboard(self, count: int = 10) -> List[Dict]:
        """Get top users from leaderboard"""
        results = self.client.zrevrange('leaderboard', 0, count - 1, withscores=True)
        return [{'user_id': user, 'score': score} for user, score in results]

    def get_user_rank(self, user_id: str) -> Optional[int]:
        """Get user rank in leaderboard (1-based)"""
        rank = self.client.zrevrank('leaderboard', user_id)
        return rank + 1 if rank is not None else None

    def rate_limit(self, key: str, limit: int = 10, window: int = 60) -> bool:
        """Check rate limit"""
        pipe = self.client.pipeline()
        now = int(time.time())
        window_start = now - window

        pipe.zremrangebyscore(key, 0, window_start)
        pipe.zcard(key)
        pipe.zadd(key, {str(now): now})
        pipe.expire(key, window)

        results = pipe.execute()
        return results[1] < limit

    def close(self):
        """Close connection"""
        self.client.close()

# Usage example
if __name__ == '__main__':
    redis_mgr = RedisManager()

    # Cache operations
    redis_mgr.cache_set('user:1000', {
        'name': 'John',
        'email': '[email protected]'
    }, ttl=3600)

    user = redis_mgr.cache_get('user:1000')
    print(f'Cached user: {user}')

    # Track activity
    redis_mgr.track_user_activity('user:1000', 'login')
    redis_mgr.track_user_activity('user:1000', 'view_profile')

    # Get activity
    today = time.strftime('%Y-%m-%d')
    activities = redis_mgr.get_user_activity('user:1000', today)
    print(f'Activities: {activities}')

    # Page views
    views = redis_mgr.increment_page_view('page:home')
    print(f'Page views: {views}')

    # Leaderboard
    redis_mgr.add_to_leaderboard('user:1000', 500)
    redis_mgr.add_to_leaderboard('user:2000', 750)
    redis_mgr.add_to_leaderboard('user:3000', 250)

    top_users = redis_mgr.get_leaderboard(3)
    print(f'Top users: {top_users}')

    rank = redis_mgr.get_user_rank('user:1000')
    print(f'User rank: {rank}')

    # Rate limiting
    for i in range(12):
        allowed = redis_mgr.rate_limit('api:user:1000', limit=10, window=60)
        print(f'Request {i + 1}: {"Allowed" if allowed else "Blocked"}')

    redis_mgr.close()
```

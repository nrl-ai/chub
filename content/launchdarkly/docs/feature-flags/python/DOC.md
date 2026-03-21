---
name: feature-flags
description: "LaunchDarkly Python Server SDK for feature flag management and experimentation"
metadata:
  languages: "python"
  versions: "9.12.3"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "launchdarkly,feature-flags,toggles,experimentation,rollout"
---

# LaunchDarkly Python Server SDK

## Golden Rule

**Always use `launchdarkly-server-sdk` for server-side Python applications.**

```bash
pip install launchdarkly-server-sdk
```

Do NOT use:
- `launchdarkly-client` (deprecated, last updated 2019)
- `launchdarkly-python-client` (unofficial package)
- Any unofficial or third-party LaunchDarkly packages

The official package is `launchdarkly-server-sdk` maintained by LaunchDarkly.

Requires Python 3.9 or higher (as of v9.12+).

---

## Installation

```bash
pip install launchdarkly-server-sdk
```

Optional observability plugin (requires v9.12+):

```bash
pip install launchdarkly-observability
```

---

## Environment Variables

Set your SDK key as an environment variable:

```bash
export LAUNCHDARKLY_SDK_KEY="sdk-key-123abc"
```

---

## Initialization

### Basic Initialization

```python
import ldclient
from ldclient.config import Config
import os

ldclient.set_config(Config(os.environ['LAUNCHDARKLY_SDK_KEY']))
client = ldclient.get()

# Wait for initialization
if client.is_initialized():
    print('LaunchDarkly client initialized')
else:
    print('LaunchDarkly client failed to initialize')
```

### With Timeout

```python
import ldclient
from ldclient.config import Config
import os

ldclient.set_config(Config(os.environ['LAUNCHDARKLY_SDK_KEY']))
client = ldclient.get()

# Wait up to 10 seconds for initialization
if client.wait_for_initialization(10):
    print('Client initialized successfully')
else:
    print('Client initialization timeout')
```

### With Configuration Options

```python
import ldclient
from ldclient.config import Config
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    stream=True,
    connect_timeout=10,
    read_timeout=15,
    offline=False,
    all_attributes_private=False,
    private_attributes={'email', 'ssn'},
    flush_interval=5,
    poll_interval=30,
    diagnostic_opt_out=False,
    wrapper_name='my-wrapper',
    wrapper_version='1.0.0'
)

ldclient.set_config(config)
client = ldclient.get()
```

### With Observability Plugin

```python
import ldclient
from ldclient.config import Config
from ldobserve import ObservabilityPlugin
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    plugins=[ObservabilityPlugin()]
)

ldclient.set_config(config)
client = ldclient.get()
```

### Singleton Pattern

**Critical:** The `ldclient` module maintains a singleton. Use `ldclient.get()` to access the shared instance.

```python
# Initialize once at application startup
import ldclient
from ldclient.config import Config
import os

def initialize_launchdarkly():
    config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
    ldclient.set_config(config)

    client = ldclient.get()
    if not client.wait_for_initialization(10):
        raise Exception('LaunchDarkly failed to initialize')

    return client

# Call once at startup
client = initialize_launchdarkly()

# Access anywhere in application
def get_ld_client():
    return ldclient.get()
```

### Error Handling During Initialization

```python
import ldclient
from ldclient.config import Config
import os

try:
    config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
    ldclient.set_config(config)
    client = ldclient.get()

    if client.wait_for_initialization(10):
        print('Client initialized successfully')
    else:
        print('Client initialization timeout - will use default values')
except Exception as e:
    print(f'Initialization error: {e}')
```

---

## Contexts

### Simple User Context

```python
from ldclient import Context

context = Context.builder('user-key-123abc') \
    .name('Sandy Smith') \
    .build()
```

### Context with Custom Attributes

```python
from ldclient import Context

context = Context.builder('user-key-123abc') \
    .name('Sandy Smith') \
    .set('email', 'sandy@example.com') \
    .set('plan', 'premium') \
    .set('betaTester', True) \
    .build()
```

### Simple Context (Shorthand)

```python
from ldclient import Context

# Just a key (defaults to 'user' kind)
context = Context.create('user-key-123abc')

# With kind specified
context = Context.create('org-key-456', 'organization')
```

### Anonymous Context

```python
from ldclient import Context

context = Context.builder('anonymous-123') \
    .anonymous(True) \
    .build()
```

### Multi-Context (Multiple Kinds)

```python
from ldclient import Context

user_context = Context.builder('user-key-123') \
    .kind('user') \
    .name('Sandy') \
    .build()

org_context = Context.builder('org-key-456') \
    .kind('organization') \
    .name('Acme Corp') \
    .build()

device_context = Context.builder('device-key-789') \
    .kind('device') \
    .set('platform', 'iOS') \
    .build()

multi_context = Context.create_multi(user_context, org_context, device_context)
```

### Private Attributes

Mark specific attributes as private (not sent to LaunchDarkly):

```python
from ldclient import Context

context = Context.builder('user-key-123abc') \
    .name('Sandy Smith') \
    .set('email', 'sandy@example.com') \
    .set('ssn', '123-45-6789') \
    .private('email', 'ssn') \
    .build()
```

### Context from Dictionary

```python
from ldclient import Context

context_dict = {
    'kind': 'user',
    'key': 'user-key-123abc',
    'name': 'Sandy Smith',
    'email': 'sandy@example.com'
}

context = Context.from_dict(context_dict)
```

---

## Flag Evaluation

### Boolean Flag

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

show_feature = client.variation('flag-key-123abc', context, False)

if show_feature:
    print('Feature enabled')
else:
    print('Feature disabled')
```

### String Flag

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

theme = client.variation('theme-flag', context, 'light')

print(f'User theme: {theme}')  # 'dark' or 'light'
```

### Number Flag

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

max_items = client.variation('max-items', context, 10)

print(f'Max items: {max_items}')  # e.g., 50
```

### JSON Flag

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

config = client.variation('config-flag', context, {
    'timeout': 30,
    'retries': 3
})

print(f"Timeout: {config['timeout']}, Retries: {config['retries']}")
```

---

## Flag Evaluation with Details

### Get Evaluation Reason

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

result = client.variation_detail('flag-key-123abc', context, False)

print('Value:', result.value)
print('Variation Index:', result.variation_index)
print('Reason:', result.reason)
```

### Reason Object Structure

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

result = client.variation_detail('flag-key-123abc', context, False)

# result.reason examples:
# {'kind': 'OFF'}
# {'kind': 'FALLTHROUGH'}
# {'kind': 'TARGET_MATCH'}
# {'kind': 'RULE_MATCH', 'ruleIndex': 0, 'ruleId': 'rule-id'}
# {'kind': 'PREREQUISITE_FAILED', 'prerequisiteKey': 'other-flag'}
# {'kind': 'ERROR', 'errorKind': 'MALFORMED_FLAG'}
```

### Using Evaluation Details for Debugging

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

result = client.variation_detail('experiment-flag', context, 'control')

if result.reason['kind'] == 'ERROR':
    print(f"Flag evaluation error: {result.reason['errorKind']}")
elif result.reason['kind'] == 'RULE_MATCH':
    print(f"Matched rule: {result.reason['ruleIndex']}")
elif result.reason['kind'] == 'FALLTHROUGH':
    print('Using fallthrough variation')

print(f'Serving variation: {result.value}')
```

---

## All Flags State

### Get All Flags for a Context

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

state = client.all_flags_state(context)

if state.valid:
    all_flags = state.to_values_map()
    print('All flags:', all_flags)
    # {'flag-1': True, 'flag-2': 'value', 'flag-3': 42}
```

### For Client-Side Bootstrapping

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

state = client.all_flags_state(
    context,
    client_side_only=True,
    with_reasons=True,
    details_only_for_tracked_flags=False
)

if state.valid:
    bootstrap_data = state.to_json_dict()
    # Send to client-side
    return {'flags': bootstrap_data}
```

### Get Individual Flag from State

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

state = client.all_flags_state(context)

flag_value = state.get_flag_value('flag-key-123abc')
flag_reason = state.get_flag_reason('flag-key-123abc')

print(f'Value: {flag_value}')
print(f'Reason: {flag_reason}')
```

---

## Event Tracking

### Track Custom Event

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

client.track('button-clicked', context)
```

### Track Event with Data

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

client.track('purchase-completed', context, data={
    'item_id': 'item-123',
    'price': 29.99,
    'currency': 'USD'
})
```

### Track Event with Numeric Metric

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

client.track('purchase-completed', context, data={
    'item_id': 'item-123'
}, metric_value=29.99)
```

### Identify Context

Send context attributes to LaunchDarkly for targeting:

```python
import ldclient
from ldclient import Context

client = ldclient.get()

context = Context.builder('user-key-123abc') \
    .name('Sandy Smith') \
    .set('email', 'sandy@example.com') \
    .build()

client.identify(context)
```

### Flush Events

Force immediate delivery of pending events:

```python
import ldclient

client = ldclient.get()
client.flush()
```

### Auto-Flush on Shutdown

```python
import ldclient
import signal
import sys

def shutdown_handler(signum, frame):
    print('Shutting down...')
    client = ldclient.get()
    client.flush()
    client.close()
    sys.exit(0)

signal.signal(signal.SIGTERM, shutdown_handler)
signal.signal(signal.SIGINT, shutdown_handler)
```

---

## Data Modes

### Streaming Mode (Default)

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    stream=True
)

ldclient.set_config(config)
```

### Polling Mode

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    stream=False,
    poll_interval=60  # seconds
)

ldclient.set_config(config)
```

### Offline Mode

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    offline=True
)

ldclient.set_config(config)
client = ldclient.get()

# All flags return default values
result = client.variation('flag-key', context, False)
# Will always return False (the default)
```

---

## HTTP Configuration

### Configure HTTP Timeouts

```python
from ldclient.config import Config, HTTPConfig
import ldclient
import os

http_config = HTTPConfig(
    connect_timeout=10,
    read_timeout=15
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    http=http_config
)

ldclient.set_config(config)
```

### Configure HTTP Proxy

```python
from ldclient.config import Config, HTTPConfig
import ldclient
import os

http_config = HTTPConfig(
    http_proxy='http://proxy.example.com:8080'
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    http=http_config
)

ldclient.set_config(config)
```

### Environment Variable Proxy

```python
import os

os.environ['HTTPS_PROXY'] = 'https://proxy.example.com:8080'

# SDK will automatically use the proxy
```

### Custom CA Certificates

```python
from ldclient.config import Config, HTTPConfig
import ldclient
import os

http_config = HTTPConfig(
    ca_certs='/path/to/ca-bundle.crt'
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    http=http_config
)

ldclient.set_config(config)
```

### Disable SSL Verification (Not Recommended)

```python
from ldclient.config import Config, HTTPConfig
import ldclient
import os

http_config = HTTPConfig(
    disable_ssl_verification=True
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    http=http_config
)

ldclient.set_config(config)
```

---

## Relay Proxy

### Using LaunchDarkly Relay Proxy

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    base_uri='http://relay-proxy.example.com',
    stream_uri='http://relay-proxy.example.com',
    events_uri='http://relay-proxy.example.com'
)

ldclient.set_config(config)
```

---

## Feature Stores

### Redis Feature Store

```python
from ldclient.config import Config
from ldclient.integrations import Redis
import ldclient
import os

redis_store = Redis.new_feature_store(
    url='redis://localhost:6379',
    prefix='ld',
    max_connections=16,
    expiration=30
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    feature_store=redis_store
)

ldclient.set_config(config)
```

### DynamoDB Feature Store

```python
from ldclient.config import Config
from ldclient.integrations import DynamoDB
import ldclient
import os

dynamodb_store = DynamoDB.new_feature_store(
    'feature-flags-table',
    prefix='ld',
    expiration=30
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    feature_store=dynamodb_store
)

ldclient.set_config(config)
```

### Consul Feature Store

```python
from ldclient.config import Config
from ldclient.integrations import Consul
import ldclient
import os

consul_store = Consul.new_feature_store(
    host='localhost',
    port=8500,
    prefix='ld',
    expiration=30
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    feature_store=consul_store
)

ldclient.set_config(config)
```

---

## File-Based Data Source

### Load Flags from File

```python
from ldclient.config import Config
from ldclient.integrations import Files
import ldclient
import os

file_source = Files.new_data_source(
    paths=['./flags.json'],
    auto_update=True,
    poll_interval=1
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    update_processor_class=file_source
)

ldclient.set_config(config)
```

### flags.json Format

```json
{
  "flags": {
    "flag-key-123abc": {
      "key": "flag-key-123abc",
      "on": true,
      "variations": [true, false],
      "fallthrough": {
        "variation": 0
      }
    }
  },
  "segments": {}
}
```

---

## Big Segments

### Configure Big Segments Store

```python
from ldclient.config import Config, BigSegmentsConfig
from ldclient.integrations import Redis
import ldclient
import os

redis_store = Redis.new_big_segment_store(
    url='redis://localhost:6379',
    prefix='big-segments'
)

big_segments = BigSegmentsConfig(
    store=redis_store,
    context_cache_size=1000,
    context_cache_time=5,
    status_poll_interval=5,
    stale_after=120
)

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    big_segments=big_segments
)

ldclient.set_config(config)
```

### Check Big Segments Status

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

result = client.variation_detail('segment-flag', context, False)

if result.reason.get('bigSegmentsStatus') == 'STALE':
    print('Big segments data is stale')
```

---

## Hooks

### Create a Hook

```python
from ldclient.hook import Hook, Metadata

class LoggingHook(Hook):
    @property
    def metadata(self):
        return Metadata(name='logging-hook')

    def before_evaluation(self, series_context, data):
        print(f'Evaluating flag: {series_context.flag_key}')
        return data

    def after_evaluation(self, series_context, data, detail):
        print(f'Flag result: {detail.value}')
        return data
```

### Register Hook via Configuration

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    hooks=[LoggingHook()]
)

ldclient.set_config(config)
```

### Add Hook at Runtime

```python
import ldclient

client = ldclient.get()
hook = LoggingHook()
client.add_hook(hook)
```

---

## Migrations

### Create Migration

```python
import ldclient
from ldclient.migrations import MigratorBuilder, Stage, ExecutionOrder

client = ldclient.get()

def old_read(payload):
    return old_database.read(payload['id'])

def new_read(payload):
    return new_database.read(payload['id'])

def old_write(payload):
    old_database.write(payload['id'], payload['data'])

def new_write(payload):
    new_database.write(payload['id'], payload['data'])

def check_consistency(old_value, new_value):
    return old_value == new_value

migrator = MigratorBuilder(client) \
    .read(old_read, new_read, check_consistency) \
    .write(old_write, new_write) \
    .read_execution_order(ExecutionOrder.PARALLEL) \
    .track_latency(True) \
    .track_errors(True) \
    .build()
```

### Execute Migration Read

```python
import ldclient
from ldclient import Context
from ldclient.migrations import Stage

client = ldclient.get()
context = Context.create('user-key-123abc')

stage_str = client.variation('migration-flag', context, 'off')
stage = Stage.from_str(stage_str)

result = migrator.read(
    'migration-flag',
    context,
    stage,
    payload={'id': 'record-123'}
)

if result.is_success():
    print('Data:', result.value)
else:
    print('Migration read failed')
```

### Execute Migration Write

```python
import ldclient
from ldclient import Context
from ldclient.migrations import Stage

client = ldclient.get()
context = Context.create('user-key-123abc')

stage_str = client.variation('migration-flag', context, 'off')
stage = Stage.from_str(stage_str)

result = migrator.write(
    'migration-flag',
    context,
    stage,
    payload={'id': 'record-123', 'data': {'name': 'Example'}}
)

if result.is_success():
    print('Write completed')
```

### Migration Stages

```python
from ldclient.migrations import Stage

# Available stages:
Stage.OFF        # Use old implementation only
Stage.DUALWRITE  # Write to both, read from old
Stage.SHADOW     # Read from both, write to both, use old for responses
Stage.LIVE       # Read from both, write to both, use new for responses
Stage.RAMPDOWN   # Write to both, read from new
Stage.COMPLETE   # Use new implementation only
```

### Execution Order

```python
from ldclient.migrations import ExecutionOrder

ExecutionOrder.SERIAL    # Execute old then new sequentially
ExecutionOrder.RANDOM    # Randomize execution order
ExecutionOrder.PARALLEL  # Execute both concurrently
```

---

## Worker-Based Servers (uWSGI, Gunicorn)

### uWSGI Configuration

```python
import ldclient
from ldclient.config import Config
import os

# Initialize before forking
config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
ldclient.set_config(config)
client = ldclient.get()

# Reinitialize after fork
try:
    import uwsgidecorators

    @uwsgidecorators.postfork
    def reinit_ld_after_fork():
        ldclient.get().postfork()
        print('LaunchDarkly reinitialized after fork')
except ImportError:
    pass
```

### Gunicorn Configuration

```python
# gunicorn_config.py
import ldclient
from ldclient.config import Config
import os

def on_starting(server):
    """Called just before the master process is initialized."""
    config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
    ldclient.set_config(config)

def post_fork(server, worker):
    """Called just after a worker has been forked."""
    ldclient.get().postfork()
```

### Without Decorator

```python
import ldclient
from ldclient.config import Config
import os

# After forking in worker process
config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
ldclient.set_config(config)
client = ldclient.get()

# Reinitialize
client.postfork()
```

---

## Application Metadata

### Set Application Info

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    application={
        'id': 'my-app',
        'version': '1.2.3',
        'name': 'My Application',
        'version_name': 'v1.2.3-beta'
    }
)

ldclient.set_config(config)
```

---

## Secure Mode

### Generate Secure Mode Hash

```python
import hashlib
import hmac
import os

def generate_secure_mode_hash(sdk_key, context_key):
    """Generate secure mode hash for client-side SDK."""
    return hmac.new(
        sdk_key.encode(),
        context_key.encode(),
        hashlib.sha256
    ).hexdigest()

hash_value = generate_secure_mode_hash(
    os.environ['LAUNCHDARKLY_SDK_KEY'],
    'user-key-123abc'
)

# Send hash to client-side
return {'hash': hash_value}
```

---

## Logging

### Configure Logger

```python
import logging
from ldclient.config import Config
import ldclient
import os

# Set up logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger('ldclient')
logger.setLevel(logging.DEBUG)

config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
ldclient.set_config(config)
```

### Custom Log Level

```python
import logging
from ldclient.config import Config
import ldclient
import os

# Create custom logger
ld_logger = logging.getLogger('ldclient')
ld_logger.setLevel(logging.WARNING)

handler = logging.StreamHandler()
handler.setLevel(logging.WARNING)
formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
handler.setFormatter(formatter)
ld_logger.addHandler(handler)

config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
ldclient.set_config(config)
```

---

## Private Attributes Configuration

### Global Private Attributes

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    private_attributes={'email', 'ssn', 'address'}
)

ldclient.set_config(config)
```

### All Attributes Private

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    all_attributes_private=True
)

ldclient.set_config(config)
```

---

## Diagnostics

### Disable Diagnostic Events

```python
from ldclient.config import Config
import ldclient
import os

config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    diagnostic_opt_out=True
)

ldclient.set_config(config)
```

---

## Testing

### Using File Data Source for Tests

```python
from ldclient.config import Config
from ldclient.integrations import Files
import ldclient

# Create test flags file
test_flags = {
    'flags': {
        'test-flag': {
            'key': 'test-flag',
            'on': True,
            'variations': [True, False],
            'fallthrough': {'variation': 0}
        }
    },
    'segments': {}
}

# Write to file
import json
with open('test-flags.json', 'w') as f:
    json.dump(test_flags, f)

# Configure client with file source
file_source = Files.new_data_source(paths=['test-flags.json'])

config = Config(
    sdk_key='test-sdk-key',
    update_processor_class=file_source,
    send_events=False
)

ldclient.set_config(config)
client = ldclient.get()
```

### Mock Client for Unit Tests

```python
from unittest.mock import Mock, MagicMock
from ldclient import EvaluationDetail

mock_client = Mock()
mock_client.variation = Mock(return_value=True)
mock_client.variation_detail = Mock(return_value=EvaluationDetail(
    value=True,
    variation_index=0,
    reason={'kind': 'OFF'}
))
mock_client.track = Mock()
mock_client.identify = Mock()
mock_client.flush = Mock()
mock_client.close = Mock()
```

---

## Shutdown and Cleanup

### Graceful Shutdown

```python
import ldclient

def shutdown():
    print('Flushing events...')
    client = ldclient.get()
    client.flush()

    print('Closing client...')
    client.close()

    print('Shutdown complete')
```

### Check Initialization Status

```python
import ldclient

client = ldclient.get()

if client.is_initialized():
    print('Client is ready')
else:
    print('Client is not initialized yet')
```

---

## Complete Flask Example

```python
from flask import Flask, request, jsonify
import ldclient
from ldclient.config import Config
from ldclient import Context
import os
import signal
import sys

app = Flask(__name__)

# Initialize LaunchDarkly
def init_launchdarkly():
    config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
    ldclient.set_config(config)

    client = ldclient.get()
    if not client.wait_for_initialization(10):
        print('LaunchDarkly initialization timeout')
    else:
        print('LaunchDarkly initialized')

    return client

# Route using feature flag
@app.route('/feature')
def feature():
    client = ldclient.get()

    user_id = request.headers.get('X-User-ID', 'anonymous')
    user_email = request.headers.get('X-User-Email')

    context = Context.builder(user_id) \
        .set('email', user_email) \
        .build()

    show_new_feature = client.variation('new-feature-flag', context, False)

    if show_new_feature:
        return jsonify({
            'feature': 'new',
            'message': 'Welcome to the new feature!'
        })
    else:
        return jsonify({
            'feature': 'old',
            'message': 'Using legacy feature'
        })

# Track custom event
@app.route('/track-event', methods=['POST'])
def track_event():
    client = ldclient.get()

    data = request.get_json()
    user_id = data.get('userId')

    context = Context.create(user_id)

    client.track('button-clicked', context, data={
        'button_id': data.get('buttonId')
    })

    return jsonify({'success': True})

# Graceful shutdown
def shutdown_handler(signum, frame):
    print('Shutting down...')
    client = ldclient.get()
    client.flush()
    client.close()
    sys.exit(0)

signal.signal(signal.SIGTERM, shutdown_handler)
signal.signal(signal.SIGINT, shutdown_handler)

if __name__ == '__main__':
    init_launchdarkly()
    app.run(port=5000)
```

---

## Complete Django Example

```python
# settings.py
import ldclient
from ldclient.config import Config
import os

# Initialize LaunchDarkly
def init_launchdarkly():
    config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
    ldclient.set_config(config)

    client = ldclient.get()
    if client.wait_for_initialization(10):
        print('LaunchDarkly initialized')

    return client

LAUNCHDARKLY_CLIENT = init_launchdarkly()


# views.py
from django.http import JsonResponse
from ldclient import Context
import ldclient

def feature_view(request):
    client = ldclient.get()

    user_id = request.META.get('HTTP_X_USER_ID', 'anonymous')
    user_email = request.META.get('HTTP_X_USER_EMAIL')

    context = Context.builder(user_id) \
        .set('email', user_email) \
        .build()

    show_new_feature = client.variation('new-feature-flag', context, False)

    if show_new_feature:
        return JsonResponse({
            'feature': 'new',
            'message': 'Welcome to the new feature!'
        })
    else:
        return JsonResponse({
            'feature': 'old',
            'message': 'Using legacy feature'
        })


# apps.py (for cleanup on shutdown)
from django.apps import AppConfig
import ldclient

class MyAppConfig(AppConfig):
    name = 'myapp'

    def ready(self):
        import signal
        import sys

        def shutdown_handler(signum, frame):
            client = ldclient.get()
            client.flush()
            client.close()
            sys.exit(0)

        signal.signal(signal.SIGTERM, shutdown_handler)
```

---

## Complete FastAPI Example

```python
from fastapi import FastAPI, Header
from pydantic import BaseModel
import ldclient
from ldclient.config import Config
from ldclient import Context
import os

app = FastAPI()

# Initialize LaunchDarkly
@app.on_event("startup")
async def startup_event():
    config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
    ldclient.set_config(config)

    client = ldclient.get()
    if not client.wait_for_initialization(10):
        print('LaunchDarkly initialization timeout')
    else:
        print('LaunchDarkly initialized')

# Shutdown
@app.on_event("shutdown")
async def shutdown_event():
    client = ldclient.get()
    client.flush()
    client.close()

# Route using feature flag
@app.get("/feature")
async def get_feature(
    x_user_id: str = Header(default='anonymous'),
    x_user_email: str = Header(default=None)
):
    client = ldclient.get()

    context = Context.builder(x_user_id) \
        .set('email', x_user_email) \
        .build()

    show_new_feature = client.variation('new-feature-flag', context, False)

    if show_new_feature:
        return {
            'feature': 'new',
            'message': 'Welcome to the new feature!'
        }
    else:
        return {
            'feature': 'old',
            'message': 'Using legacy feature'
        }

# Track custom event
class TrackEventRequest(BaseModel):
    userId: str
    buttonId: str

@app.post("/track-event")
async def track_event(event: TrackEventRequest):
    client = ldclient.get()

    context = Context.create(event.userId)

    client.track('button-clicked', context, data={
        'button_id': event.buttonId
    })

    return {'success': True}
```

---

## OpenTelemetry Integration

### Setup OpenTelemetry with LaunchDarkly

```python
import ldclient
from ldclient.config import Config
from ldobserve import ObservabilityPlugin
from opentelemetry import metrics
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.exporter.prometheus import PrometheusMetricReader
import os

# Set up OpenTelemetry
prometheus_reader = PrometheusMetricReader()
meter_provider = MeterProvider(metric_readers=[prometheus_reader])
metrics.set_meter_provider(meter_provider)

# Initialize LaunchDarkly with observability
config = Config(
    sdk_key=os.environ['LAUNCHDARKLY_SDK_KEY'],
    plugins=[ObservabilityPlugin()]
)

ldclient.set_config(config)
client = ldclient.get()
```

---

## Error Handling

### Handle Initialization Errors

```python
import ldclient
from ldclient.config import Config
import os

try:
    config = Config(os.environ['LAUNCHDARKLY_SDK_KEY'])
    ldclient.set_config(config)
    client = ldclient.get()

    if not client.wait_for_initialization(10):
        print('Initialization timeout - using default values')
except Exception as e:
    print(f'LaunchDarkly initialization error: {e}')
```

### Handle Evaluation Errors

```python
import ldclient
from ldclient import Context

client = ldclient.get()
context = Context.create('user-key-123abc')

result = client.variation_detail('flag-key', context, False)

if result.reason['kind'] == 'ERROR':
    error_kind = result.reason['errorKind']

    if error_kind == 'MALFORMED_FLAG':
        print('Flag configuration is invalid')
    elif error_kind == 'FLAG_NOT_FOUND':
        print('Flag does not exist')
    elif error_kind == 'USER_NOT_SPECIFIED':
        print('Context is invalid')
    elif error_kind == 'WRONG_TYPE':
        print('Flag type mismatch')
    else:
        print(f'Unknown error: {error_kind}')
```

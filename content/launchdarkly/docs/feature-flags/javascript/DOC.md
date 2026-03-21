---
name: feature-flags
description: "LaunchDarkly Node.js Server SDK for feature flag management and experimentation"
metadata:
  languages: "javascript"
  versions: "9.10.2"
  updated-on: "2026-03-02"
  source: maintainer
  tags: "launchdarkly,feature-flags,toggles,experimentation,rollout"
---

# LaunchDarkly Node.js Server SDK

## Golden Rule

**Always use `@launchdarkly/node-server-sdk` for server-side Node.js applications.**

```bash
npm install @launchdarkly/node-server-sdk
```

Do NOT use:
- `launchdarkly-node-server-sdk` (deprecated, last updated 2021)
- `launchdarkly-node-client-sdk` (client-side only, different use case)
- Any unofficial or third-party LaunchDarkly packages

The official package is `@launchdarkly/node-server-sdk` maintained by LaunchDarkly.

---

## Installation

```bash
npm install @launchdarkly/node-server-sdk
```

Optional observability plugin (requires v9.10+):

```bash
npm install @launchdarkly/observability-node
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

```javascript
import { init } from '@launchdarkly/node-server-sdk';

const client = init(process.env.LAUNCHDARKLY_SDK_KEY);

// Wait for initialization before evaluating flags
await client.waitForInitialization({ timeout: 10 });
```

### With Configuration Options

```javascript
import { init } from '@launchdarkly/node-server-sdk';

const options = {
  timeout: 10,
  capacity: 1000,
  flushInterval: 30,
  stream: true,
  allAttributesPrivate: false,
  privateAttributes: ['email', 'ssn'],
  offline: false,
  diagnosticOptOut: false,
  wrapperName: 'my-wrapper',
  wrapperVersion: '1.0.0'
};

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, options);
await client.waitForInitialization({ timeout: 10 });
```

### With Observability Plugin

```javascript
import { init } from '@launchdarkly/node-server-sdk';
import { Observability } from '@launchdarkly/observability-node';

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  plugins: [new Observability()]
});

await client.waitForInitialization({ timeout: 10 });
```

### Singleton Pattern

**Critical:** Make the LDClient a singleton. Do NOT create multiple instances per request.

```javascript
// app.js or server initialization
import { init } from '@launchdarkly/node-server-sdk';

let ldClient;

async function initializeLaunchDarkly() {
  ldClient = init(process.env.LAUNCHDARKLY_SDK_KEY);
  await ldClient.waitForInitialization({ timeout: 10 });
  return ldClient;
}

export function getLDClient() {
  if (!ldClient) {
    throw new Error('LaunchDarkly client not initialized');
  }
  return ldClient;
}

// Initialize once at app startup
await initializeLaunchDarkly();
```

### Error Handling During Initialization

```javascript
import { init } from '@launchdarkly/node-server-sdk';

const client = init(process.env.LAUNCHDARKLY_SDK_KEY);

client.once('ready', () => {
  console.log('LaunchDarkly client initialized successfully');
});

client.once('failed', () => {
  console.error('LaunchDarkly client failed to initialize');
});

try {
  await client.waitForInitialization({ timeout: 10 });
  console.log('Client ready');
} catch (error) {
  console.error('Initialization timeout:', error);
  // Client will still work but may return default values
}
```

---

## Contexts

### Simple User Context

```javascript
const context = {
  kind: 'user',
  key: 'user-key-123abc',
  name: 'Sandy Smith',
  email: 'sandy@example.com'
};
```

### Context with Custom Attributes

```javascript
const context = {
  kind: 'user',
  key: 'user-key-123abc',
  name: 'Sandy Smith',
  email: 'sandy@example.com',
  plan: 'premium',
  betaTester: true,
  customAttribute: 'custom-value'
};
```

### Anonymous Context

```javascript
const context = {
  kind: 'user',
  key: 'anonymous-user-123',
  anonymous: true
};
```

### Multi-Context (Multiple Kinds)

```javascript
const multiContext = {
  kind: 'multi',
  user: {
    key: 'user-key-123',
    name: 'Sandy'
  },
  organization: {
    key: 'org-key-456',
    name: 'Acme Corp'
  },
  device: {
    key: 'device-key-789',
    platform: 'iOS'
  }
};
```

### Private Attributes

Mark specific attributes as private (not sent to LaunchDarkly):

```javascript
const context = {
  kind: 'user',
  key: 'user-key-123abc',
  name: 'Sandy Smith',
  email: 'sandy@example.com',
  ssn: '123-45-6789',
  _meta: {
    privateAttributes: ['email', 'ssn']
  }
};
```

---

## Flag Evaluation

### Boolean Flag

```javascript
const showFeature = await client.variation('flag-key-123abc', context, false);

if (showFeature) {
  // Feature enabled
} else {
  // Feature disabled
}
```

### String Flag

```javascript
const theme = await client.variation('theme-flag', context, 'light');

console.log(`User theme: ${theme}`); // 'dark' or 'light'
```

### Number Flag

```javascript
const maxItems = await client.variation('max-items', context, 10);

console.log(`Max items: ${maxItems}`); // e.g., 50
```

### JSON Flag

```javascript
const config = await client.variation('config-flag', context, {
  timeout: 30,
  retries: 3
});

console.log(`Timeout: ${config.timeout}, Retries: ${config.retries}`);
```

### Callback Style (Legacy)

```javascript
client.variation('flag-key-123abc', context, false, (err, showFeature) => {
  if (err) {
    console.error('Error evaluating flag:', err);
    return;
  }

  if (showFeature) {
    // Feature enabled
  }
});
```

### Synchronous Variation (After Initialization)

```javascript
// Only use after client is fully initialized
const showFeature = client.variation('flag-key-123abc', context, false);
```

---

## Flag Evaluation with Details

### Get Evaluation Reason

```javascript
const result = await client.variationDetail('flag-key-123abc', context, false);

console.log('Value:', result.value);
console.log('Variation Index:', result.variationIndex);
console.log('Reason:', result.reason);
```

### Reason Object Structure

```javascript
const result = await client.variationDetail('flag-key-123abc', context, false);

// result.reason can be:
// { kind: 'OFF' }
// { kind: 'FALLTHROUGH' }
// { kind: 'TARGET_MATCH' }
// { kind: 'RULE_MATCH', ruleIndex: 0, ruleId: 'rule-id' }
// { kind: 'PREREQUISITE_FAILED', prerequisiteKey: 'other-flag' }
// { kind: 'ERROR', errorKind: 'MALFORMED_FLAG' }
```

### Using Evaluation Details for Debugging

```javascript
const result = await client.variationDetail('experiment-flag', context, 'control');

if (result.reason.kind === 'ERROR') {
  console.error('Flag evaluation error:', result.reason.errorKind);
} else if (result.reason.kind === 'RULE_MATCH') {
  console.log('Matched rule:', result.reason.ruleIndex);
} else if (result.reason.kind === 'FALLTHROUGH') {
  console.log('Using fallthrough variation');
}

console.log('Serving variation:', result.value);
```

---

## All Flags State

### Get All Flags for a Context

```javascript
const state = await client.allFlagsState(context);

const allFlags = state.allValues();
console.log('All flags:', allFlags);
// { 'flag-1': true, 'flag-2': 'value', 'flag-3': 42 }
```

### For Client-Side Bootstrapping

```javascript
const state = await client.allFlagsState(context, {
  clientSideOnly: true,
  withReasons: true,
  detailsOnlyForTrackedFlags: false
});

const bootstrapData = state.toJSON();

// Send to client-side
res.json({
  flags: bootstrapData
});
```

### Check If State Is Valid

```javascript
const state = await client.allFlagsState(context);

if (state.valid) {
  console.log('Successfully retrieved all flags');
  const flags = state.allValues();
} else {
  console.error('Failed to retrieve flags');
}
```

---

## Event Tracking

### Track Custom Event

```javascript
client.track('button-clicked', context);
```

### Track Event with Data

```javascript
client.track('purchase-completed', context, {
  itemId: 'item-123',
  price: 29.99,
  currency: 'USD'
});
```

### Track Event with Numeric Metric

```javascript
client.track('purchase-completed', context, {
  itemId: 'item-123'
}, 29.99);
```

### Identify Context

Send context attributes to LaunchDarkly for targeting:

```javascript
client.identify(context);
```

### Flush Events

Force immediate delivery of pending events:

```javascript
await client.flush();
```

### Auto-Flush on Shutdown

```javascript
process.on('SIGTERM', async () => {
  await client.flush();
  await client.close();
  process.exit(0);
});
```

---

## Data Modes

### Streaming Mode (Default)

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  stream: true
});
```

### Polling Mode

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  stream: false,
  pollInterval: 60 // seconds
});
```

### Offline Mode

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  offline: true
});

// All flags return default values
const result = await client.variation('flag-key', context, false);
// Will always return false (the default)
```

---

## Relay Proxy

### Using LaunchDarkly Relay Proxy

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  baseUri: 'http://relay-proxy.example.com',
  streamUri: 'http://relay-proxy.example.com',
  eventsUri: 'http://relay-proxy.example.com'
});
```

### Daemon Mode

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  useLdd: true,
  featureStore: myFeatureStore
});
```

---

## Feature Stores

### Redis Feature Store

```javascript
import { init } from '@launchdarkly/node-server-sdk';
import { RedisFeatureStore } from '@launchdarkly/node-server-sdk-redis';

const redisStore = RedisFeatureStore({
  redisOpts: {
    host: 'localhost',
    port: 6379
  },
  prefix: 'ld',
  cacheTTL: 30
});

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  featureStore: redisStore
});
```

### DynamoDB Feature Store

```javascript
import { DynamoDBFeatureStore } from '@launchdarkly/node-server-sdk-dynamodb';

const dynamoStore = DynamoDBFeatureStore('feature-flags-table', {
  cacheTTL: 30
});

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  featureStore: dynamoStore
});
```

---

## Bootstrapping

### File-Based Data Source

```javascript
import { init } from '@launchdarkly/node-server-sdk';
import { FileDataSource } from '@launchdarkly/node-server-sdk-file';

const fileSource = FileDataSource({
  paths: ['./flags.json']
});

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  updateProcessor: fileSource
});
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

## Subscribing to Flag Changes

### Listen for Specific Flag Changes

```javascript
client.on('update:flag-key-123abc', (newValue, oldValue) => {
  console.log(`Flag changed from ${oldValue} to ${newValue}`);
});
```

### Listen for All Flag Updates

```javascript
client.on('update', (settings) => {
  console.log('Flags updated');
});
```

---

## Big Segments

### Configure Big Segments Store

```javascript
import { init } from '@launchdarkly/node-server-sdk';
import { RedisBigSegmentStore } from '@launchdarkly/node-server-sdk-redis';

const bigSegmentStore = RedisBigSegmentStore({
  redisOpts: {
    host: 'localhost',
    port: 6379
  },
  prefix: 'big-segments'
});

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  bigSegments: {
    store: bigSegmentStore,
    contextCacheSize: 1000,
    contextCacheTime: 5,
    statusPollInterval: 5,
    staleAfter: 120
  }
});
```

### Check Big Segments Status

```javascript
const result = await client.variationDetail('segment-flag', context, false);

if (result.reason.bigSegmentsStatus === 'STALE') {
  console.warn('Big segments data is stale');
}
```

---

## Hooks

### Create a Hook

```javascript
class LoggingHook {
  getMetadata() {
    return { name: 'logging-hook' };
  }

  beforeEvaluation(hookContext, data) {
    console.log('Evaluating flag:', hookContext.flagKey);
    return data;
  }

  afterEvaluation(hookContext, data, detail) {
    console.log('Flag result:', detail.value);
    return data;
  }
}
```

### Register Hook via Configuration

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  hooks: [new LoggingHook()]
});
```

### Add Hook at Runtime

```javascript
const hook = new LoggingHook();
client.addHook(hook);
```

---

## Migrations

### Create Migration

```javascript
import { createMigration } from '@launchdarkly/node-server-sdk';

const migration = createMigration(client, {
  execution: 'parallel',
  latencyTracking: true,
  errorTracking: true,

  readOld: async (payload) => {
    return await oldDatabase.read(payload.id);
  },

  readNew: async (payload) => {
    return await newDatabase.read(payload.id);
  },

  writeOld: async (payload) => {
    await oldDatabase.write(payload.id, payload.data);
  },

  writeNew: async (payload) => {
    await newDatabase.write(payload.id, payload.data);
  },

  check: (oldValue, newValue) => {
    return JSON.stringify(oldValue) === JSON.stringify(newValue);
  }
});
```

### Execute Migration Read

```javascript
const stage = await client.variation('migration-flag', context, 'off');

const result = await migration.read('migration-flag', context, stage, {
  id: 'record-123'
});

if (result.isSuccessful()) {
  console.log('Data:', result.getValue());
} else {
  console.error('Migration read failed');
}
```

### Execute Migration Write

```javascript
const stage = await client.variation('migration-flag', context, 'off');

const result = await migration.write('migration-flag', context, stage, {
  id: 'record-123',
  data: { name: 'Example' }
});

if (result.isSuccessful()) {
  console.log('Write completed');
}
```

### Migration Stages

- `'off'` - Use old implementation only
- `'dualwrite'` - Write to both, read from old
- `'shadow'` - Read from both, write to both, use old for responses
- `'live'` - Read from both, write to both, use new for responses
- `'rampdown'` - Write to both, read from new
- `'complete'` - Use new implementation only

---

## HTTP Configuration

### Configure Proxy

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  proxyOptions: {
    host: 'proxy.example.com',
    port: 8080,
    auth: 'username:password'
  }
});
```

### Custom TLS Options

```javascript
import * as fs from 'fs';

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  tlsOptions: {
    ca: fs.readFileSync('./ca-cert.pem'),
    cert: fs.readFileSync('./client-cert.pem'),
    key: fs.readFileSync('./client-key.pem')
  }
});
```

### Custom HTTP Headers

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  requestHeaderTransform: (headers) => {
    headers['X-Custom-Header'] = 'custom-value';
    return headers;
  }
});
```

---

## Application Metadata

### Set Application Info

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  application: {
    id: 'my-app',
    version: '1.2.3',
    name: 'My Application',
    versionName: 'v1.2.3-beta'
  }
});
```

---

## Secure Mode

### Generate Secure Mode Hash

```javascript
import { createHmac } from 'crypto';

function generateSecureModeHash(sdkKey, contextKey) {
  return createHmac('sha256', sdkKey)
    .update(contextKey)
    .digest('hex');
}

const hash = generateSecureModeHash(
  process.env.LAUNCHDARKLY_SDK_KEY,
  context.key
);

// Send hash to client-side
res.json({ hash });
```

---

## Testing

### Using Test Data Source

```javascript
import { init, TestData } from '@launchdarkly/node-server-sdk';

const td = TestData();

td.update(td.flag('flag-key-123abc')
  .variations(true, false)
  .variationForAll(true)
);

const client = init('sdk-key', {
  updateProcessor: td.getFactory()
});

// Update flag during test
td.update(td.flag('flag-key-123abc').variationForAll(false));
```

### Mock Client for Unit Tests

```javascript
const mockClient = {
  variation: jest.fn().mockResolvedValue(true),
  variationDetail: jest.fn().mockResolvedValue({
    value: true,
    variationIndex: 0,
    reason: { kind: 'OFF' }
  }),
  track: jest.fn(),
  identify: jest.fn(),
  flush: jest.fn().mockResolvedValue(undefined),
  close: jest.fn().mockResolvedValue(undefined)
};
```

---

## Error Handling

### Handle Initialization Errors

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY);

client.on('error', (error) => {
  console.error('LaunchDarkly error:', error);
});

try {
  await client.waitForInitialization({ timeout: 10 });
} catch (error) {
  console.error('Initialization timeout');
  // Client will still work with default values
}
```

### Handle Evaluation Errors

```javascript
const result = await client.variationDetail('flag-key', context, false);

if (result.reason.kind === 'ERROR') {
  switch (result.reason.errorKind) {
    case 'MALFORMED_FLAG':
      console.error('Flag configuration is invalid');
      break;
    case 'FLAG_NOT_FOUND':
      console.error('Flag does not exist');
      break;
    case 'USER_NOT_SPECIFIED':
      console.error('Context is invalid');
      break;
    case 'WRONG_TYPE':
      console.error('Flag type mismatch');
      break;
    default:
      console.error('Unknown error:', result.reason.errorKind);
  }
}
```

---

## Logging

### Custom Logger

```javascript
import { BasicLogger } from '@launchdarkly/node-server-sdk';

const logger = BasicLogger.get();

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  logger: logger
});
```

### Set Log Level

```javascript
import { BasicLogger } from '@launchdarkly/node-server-sdk';

const logger = BasicLogger.get();
logger.setLevel('debug'); // 'debug', 'info', 'warn', 'error'

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  logger: logger
});
```

### Custom Logger Implementation

```javascript
class CustomLogger {
  debug(message) {
    console.log('[DEBUG]', message);
  }

  info(message) {
    console.log('[INFO]', message);
  }

  warn(message) {
    console.warn('[WARN]', message);
  }

  error(message) {
    console.error('[ERROR]', message);
  }
}

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  logger: new CustomLogger()
});
```

---

## Complete Express.js Example

```javascript
import express from 'express';
import { init } from '@launchdarkly/node-server-sdk';

const app = express();
let ldClient;

// Initialize LaunchDarkly client
async function initLaunchDarkly() {
  ldClient = init(process.env.LAUNCHDARKLY_SDK_KEY, {
    timeout: 10,
    stream: true
  });

  await ldClient.waitForInitialization({ timeout: 10 });
  console.log('LaunchDarkly initialized');
}

// Middleware to add LD client to request
app.use((req, res, next) => {
  req.ldClient = ldClient;
  next();
});

// Route using feature flag
app.get('/feature', async (req, res) => {
  const context = {
    kind: 'user',
    key: req.headers['x-user-id'] || 'anonymous',
    email: req.headers['x-user-email']
  };

  const showNewFeature = await ldClient.variation(
    'new-feature-flag',
    context,
    false
  );

  if (showNewFeature) {
    res.json({ feature: 'new', message: 'Welcome to the new feature!' });
  } else {
    res.json({ feature: 'old', message: 'Using legacy feature' });
  }
});

// Track custom event
app.post('/track-event', async (req, res) => {
  const context = {
    kind: 'user',
    key: req.body.userId
  };

  ldClient.track('button-clicked', context, {
    buttonId: req.body.buttonId
  });

  res.json({ success: true });
});

// Graceful shutdown
process.on('SIGTERM', async () => {
  console.log('Shutting down...');
  await ldClient.flush();
  await ldClient.close();
  process.exit(0);
});

// Start server
const PORT = process.env.PORT || 3000;

initLaunchDarkly()
  .then(() => {
    app.listen(PORT, () => {
      console.log(`Server running on port ${PORT}`);
    });
  })
  .catch((error) => {
    console.error('Failed to initialize LaunchDarkly:', error);
    process.exit(1);
  });
```

---

## TypeScript Support

### Type Definitions

```typescript
import {
  init,
  LDClient,
  LDContext,
  LDOptions,
  LDFlagValue,
  LDEvaluationDetail
} from '@launchdarkly/node-server-sdk';

const options: LDOptions = {
  timeout: 10,
  stream: true
};

const client: LDClient = init(process.env.LAUNCHDARKLY_SDK_KEY!, options);

const context: LDContext = {
  kind: 'user',
  key: 'user-123',
  name: 'Sandy'
};

const value: LDFlagValue = await client.variation('flag-key', context, false);
```

### Typed Flag Values

```typescript
interface FeatureConfig {
  maxItems: number;
  enabled: boolean;
  theme: string;
}

const config = await client.variation<FeatureConfig>(
  'config-flag',
  context,
  { maxItems: 10, enabled: false, theme: 'light' }
);

console.log(config.maxItems); // Type-safe
```

---

## OpenTelemetry Integration

### Setup OpenTelemetry with LaunchDarkly

```javascript
import { init } from '@launchdarkly/node-server-sdk';
import { Observability } from '@launchdarkly/observability-node';
import { NodeSDK } from '@opentelemetry/sdk-node';
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node';
import { PrometheusExporter } from '@opentelemetry/exporter-prometheus';

const sdk = new NodeSDK({
  metricReader: new PrometheusExporter({
    port: 9464
  }),
  instrumentations: [getNodeAutoInstrumentations()]
});

sdk.start();

const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  plugins: [new Observability()]
});
```

---

## Private Attributes Configuration

### Global Private Attributes

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  privateAttributes: ['email', 'ssn', 'address'],
  allAttributesPrivate: false
});
```

### All Attributes Private

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  allAttributesPrivate: true
});
```

---

## Diagnostics

### Disable Diagnostic Events

```javascript
const client = init(process.env.LAUNCHDARKLY_SDK_KEY, {
  diagnosticOptOut: true
});
```

---

## Shutdown and Cleanup

### Graceful Shutdown

```javascript
async function shutdown() {
  console.log('Flushing events...');
  await client.flush();

  console.log('Closing client...');
  await client.close();

  console.log('Shutdown complete');
}

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
```

### Check Initialization Status

```javascript
if (client.initialized()) {
  console.log('Client is ready');
} else {
  console.log('Client is not initialized yet');
}
```

---
name: kinesis
description: "AWS SDK for JavaScript v3 client for Amazon Kinesis Data Streams in JavaScript applications"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,kinesis,data-streams,streaming,javascript,nodejs"
---

# AWS Kinesis SDK for JavaScript (v3)

Use `@aws-sdk/client-kinesis` to write records to Amazon Kinesis Data Streams and inspect streams, shards, and consumers from modern JavaScript and TypeScript code.

## Golden Rule

- Install `@aws-sdk/client-kinesis`, not the legacy `aws-sdk` v2 package.
- The package version covered here is `3.1006.0`.
- Import from the package root only. Do not deep-import from `dist-*` paths.
- Prefer `KinesisClient` plus explicit commands for most application code.
- Encode text or JSON into bytes before sending it as record `Data`.

## Install

```bash
npm install @aws-sdk/client-kinesis
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Client Setup

### Minimal Node.js client

```javascript
import { KinesisClient } from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { KinesisClient } from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

## Credentials and Region

In Node.js, the default credential provider chain is usually enough if you have already configured AWS access through environment variables, shared config files, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

If you rely on shared AWS config instead of environment variables, keep the client initialization simple and set only the region in code.

## Core Usage Pattern

The v3 SDK uses client-plus-command calls:

```javascript
import {
  KinesisClient,
  PutRecordCommand,
} from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({ region: "us-east-1" });

await kinesis.send(
  new PutRecordCommand({
    StreamName: "orders",
    PartitionKey: "customer#123",
    Data: new TextEncoder().encode(
      JSON.stringify({ orderId: "ord_123", status: "created" }),
    ),
  }),
);
```

`PartitionKey` affects which shard receives the record. Records with the same partition key are ordered within their shard, but ordering is not global across the stream.

## Common Operations

### Write multiple records in one request

`PutRecords` can succeed partially, so always inspect `FailedRecordCount` and retry only the failed entries.

```javascript
import {
  KinesisClient,
  PutRecordsCommand,
} from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({ region: "us-east-1" });

const records = [
  { orderId: "ord_123", status: "created" },
  { orderId: "ord_124", status: "created" },
];

const response = await kinesis.send(
  new PutRecordsCommand({
    StreamName: "orders",
    Records: records.map((record) => ({
      PartitionKey: record.orderId,
      Data: new TextEncoder().encode(JSON.stringify(record)),
    })),
  }),
);

if ((response.FailedRecordCount ?? 0) > 0) {
  const failedRecords = (response.Records ?? [])
    .map((result, index) => ({ result, record: records[index] }))
    .filter(({ result }) => result.ErrorCode);

  console.log("Retry these records", failedRecords);
}
```

### List streams with pagination

```javascript
import {
  KinesisClient,
  paginateListStreams,
} from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({ region: "us-east-1" });

for await (const page of paginateListStreams(
  { client: kinesis },
  { Limit: 25 },
)) {
  for (const streamName of page.StreamNames ?? []) {
    console.log(streamName);
  }
}
```

### List shard IDs for a stream

```javascript
import {
  KinesisClient,
  ListShardsCommand,
} from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({ region: "us-east-1" });

const response = await kinesis.send(
  new ListShardsCommand({
    StreamName: "orders",
  }),
);

for (const shard of response.Shards ?? []) {
  console.log(shard.ShardId);
}
```

### Read records from a shard

`GetShardIterator` gives you a starting iterator, then `GetRecords` returns a `NextShardIterator` for the next poll.

```javascript
import { Buffer } from "node:buffer";
import {
  GetRecordsCommand,
  GetShardIteratorCommand,
  KinesisClient,
} from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({ region: "us-east-1" });

const { ShardIterator } = await kinesis.send(
  new GetShardIteratorCommand({
    StreamName: "orders",
    ShardId: "shardId-000000000000",
    ShardIteratorType: "LATEST",
  }),
);

const response = await kinesis.send(
  new GetRecordsCommand({
    ShardIterator,
    Limit: 100,
  }),
);

for (const record of response.Records ?? []) {
  const text = Buffer.from(record.Data).toString("utf8");
  console.log(record.SequenceNumber, text);
}

const nextIterator = response.NextShardIterator;
```

### Inspect stream status

```javascript
import {
  DescribeStreamSummaryCommand,
  KinesisClient,
} from "@aws-sdk/client-kinesis";

const kinesis = new KinesisClient({ region: "us-east-1" });

const response = await kinesis.send(
  new DescribeStreamSummaryCommand({
    StreamName: "orders",
  }),
);

console.log(response.StreamDescriptionSummary?.StreamStatus);
console.log(response.StreamDescriptionSummary?.OpenShardCount);
```

## Kinesis-Specific Gotchas

- Record `Data` is binary. Use `TextEncoder`, `Buffer`, or another byte conversion step instead of sending plain strings directly.
- `PutRecords` is not all-or-nothing. A successful HTTP response can still contain per-record failures.
- Ordering guarantees are per shard, not across the whole stream. Your partition-key strategy matters for both ordering and hot-shard avoidance.
- `GetShardIterator` returns a short-lived iterator. Use `NextShardIterator` from each `GetRecords` response rather than reusing an old iterator indefinitely.
- `LATEST` starts from newly arriving data, while `TRIM_HORIZON` starts from the oldest retained records. Pick the iterator type deliberately.
- Shard layouts can change after resharding. Refresh shard IDs instead of caching them forever in long-running consumers.
- Low-level polling with `GetRecords` is useful for simple consumers and tools, but production consumers often need extra coordination, checkpointing, or fan-out strategy beyond this package alone.
- For LocalStack or other emulators, set `endpoint`, an explicit region, and test credentials.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: shared config, SSO, STS assume-role flows, and other credential helpers.
- `@aws-sdk/client-firehose`: managed delivery to downstream destinations when you do not want to build your own stream consumer pipeline.
- `@aws-sdk/client-cloudwatch`: alarms and metrics around producer or consumer health.

---
name: sqs
description: "AWS SDK for JavaScript v3 client for Amazon SQS in Node.js applications"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,sqs,queues,messaging,javascript"
---

# AWS SQS SDK for JavaScript (v3)

Use `@aws-sdk/client-sqs` to create queues and send, receive, and delete Amazon SQS messages from modern JavaScript and TypeScript code.

## Golden Rule

- Install `@aws-sdk/client-sqs`, not the legacy `aws-sdk` v2 package.
- The package version covered here is `3.1006.0`.
- Import from the package root only. Do not deep-import from `dist-*` paths.
- Prefer `SQSClient` plus explicit commands for most application code.

## Install

```bash
npm install @aws-sdk/client-sqs
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Client Setup

### Minimal Node.js client

```javascript
import { SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({
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
import { SendMessageCommand, SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

await sqs.send(
  new SendMessageCommand({
    QueueUrl: "https://sqs.us-east-1.amazonaws.com/123456789012/jobs",
    MessageBody: JSON.stringify({ taskId: "123" }),
  }),
);
```

Prefer storing or resolving the full `QueueUrl` before sending traffic. Do not build queue URLs by string concatenation.

## Common Operations

### Resolve a queue URL from a queue name

```javascript
import { GetQueueUrlCommand, SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

const { QueueUrl } = await sqs.send(
  new GetQueueUrlCommand({ QueueName: "jobs" }),
);

console.log(QueueUrl);
```

### Receive and delete a message

```javascript
import {
  DeleteMessageCommand,
  ReceiveMessageCommand,
  SQSClient,
} from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });
const queueUrl = "https://sqs.us-east-1.amazonaws.com/123456789012/jobs";

const { Messages = [] } = await sqs.send(
  new ReceiveMessageCommand({
    QueueUrl: queueUrl,
    MaxNumberOfMessages: 10,
    WaitTimeSeconds: 20,
    VisibilityTimeout: 30,
  }),
);

for (const message of Messages) {
  console.log(message.Body);

  if (!message.ReceiptHandle) {
    continue;
  }

  await sqs.send(
    new DeleteMessageCommand({
      QueueUrl: queueUrl,
      ReceiptHandle: message.ReceiptHandle,
    }),
  );
}
```

Delete only after successful processing. If the handler fails, let the visibility timeout expire so the message can be retried.

## SQS-Specific Gotchas

- SQS delivery is at least once. Make consumers idempotent.
- Delete messages with `ReceiptHandle`, not `MessageId`.
- Use long polling with `WaitTimeSeconds` to reduce empty responses and API churn.
- `ReceiveMessage` returns at most 10 messages per call, so batch or poll accordingly.
- FIFO queue names must end with `.fifo`, and FIFO sends require `MessageGroupId`.
- If FIFO content-based deduplication is off, supply `MessageDeduplicationId` on sends.
- Queue attribute values are strings when you create or update queues.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, Cognito, STS assume-role flows, and other credential helpers.

## Common SQS Operations

### Create a standard queue

Queue attributes are string values.

```javascript
import { CreateQueueCommand, SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

const { QueueUrl } = await sqs.send(
  new CreateQueueCommand({
    QueueName: "jobs",
    Attributes: {
      ReceiveMessageWaitTimeSeconds: "20",
      VisibilityTimeout: "30",
    },
  }),
);

console.log(QueueUrl);
```

### Resolve a queue URL

`SendMessage`, `ReceiveMessage`, and `DeleteMessage` all need the full queue URL.

```javascript
import { GetQueueUrlCommand, SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

const { QueueUrl } = await sqs.send(
  new GetQueueUrlCommand({ QueueName: "jobs" }),
);
```

### Send a message with attributes

```javascript
import { SendMessageCommand, SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

await sqs.send(
  new SendMessageCommand({
    QueueUrl: "https://sqs.us-east-1.amazonaws.com/123456789012/jobs",
    MessageBody: JSON.stringify({ taskId: "123" }),
    MessageAttributes: {
      eventType: {
        DataType: "String",
        StringValue: "job.created",
      },
    },
  }),
);
```

### Send to a FIFO queue

FIFO queue names end with `.fifo`, and the send input must include `MessageGroupId`.

```javascript
import { SendMessageCommand, SQSClient } from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

await sqs.send(
  new SendMessageCommand({
    QueueUrl: "https://sqs.us-east-1.amazonaws.com/123456789012/jobs.fifo",
    MessageBody: JSON.stringify({ taskId: "123" }),
    MessageGroupId: "jobs",
    MessageDeduplicationId: "task-123",
  }),
);
```

If the queue has content-based deduplication enabled, the explicit `MessageDeduplicationId` can be omitted.

### List queues with pagination

```javascript
import {
  paginateListQueues,
  SQSClient,
} from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

for await (const page of paginateListQueues(
  { client: sqs },
  { QueueNamePrefix: "jobs" },
)) {
  for (const queueUrl of page.QueueUrls ?? []) {
    console.log(queueUrl);
  }
}
```

### Extend visibility while processing

If work takes longer than the original visibility timeout, extend it before the message becomes visible again.

```javascript
import {
  ChangeMessageVisibilityCommand,
  SQSClient,
} from "@aws-sdk/client-sqs";

const sqs = new SQSClient({ region: "us-east-1" });

await sqs.send(
  new ChangeMessageVisibilityCommand({
    QueueUrl: "https://sqs.us-east-1.amazonaws.com/123456789012/jobs",
    ReceiptHandle: "AQEB...example...",
    VisibilityTimeout: 120,
  }),
);
```

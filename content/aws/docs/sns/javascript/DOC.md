---
name: sns
description: "AWS SDK for JavaScript v3 client for Amazon SNS in Node.js applications"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,sns,notifications,pubsub,messaging,javascript"
---

# AWS SNS SDK for JavaScript (v3)

Use `@aws-sdk/client-sns` to create topics, publish messages, and manage Amazon SNS subscriptions from modern JavaScript and TypeScript code.

## Golden Rule

- Install `@aws-sdk/client-sns`, not the legacy `aws-sdk` v2 package.
- The package version covered here is `3.1006.0`.
- Import from the package root only. Do not deep-import from `dist-*` paths.
- Prefer `SNSClient` plus explicit commands for most application code.

## Install

```bash
npm install @aws-sdk/client-sns
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Client Setup

### Minimal Node.js client

```javascript
import { SNSClient } from "@aws-sdk/client-sns";

const sns = new SNSClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { SNSClient } from "@aws-sdk/client-sns";

const sns = new SNSClient({
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
import { PublishCommand, SNSClient } from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

await sns.send(
  new PublishCommand({
    TopicArn: "arn:aws:sns:us-east-1:123456789012:orders",
    Subject: "order-created",
    Message: JSON.stringify({ orderId: "123", status: "created" }),
  }),
);
```

`PublishCommand` requires exactly one destination: `TopicArn`, `TargetArn`, or `PhoneNumber`.

## Common Operations

### Create a topic

```javascript
import { CreateTopicCommand, SNSClient } from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

const { TopicArn } = await sns.send(
  new CreateTopicCommand({
    Name: "orders",
  }),
);

console.log(TopicArn);
```

### Subscribe an SQS queue to a topic

```javascript
import { SNSClient, SubscribeCommand } from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

const { SubscriptionArn } = await sns.send(
  new SubscribeCommand({
    TopicArn: "arn:aws:sns:us-east-1:123456789012:orders",
    Protocol: "sqs",
    Endpoint: "arn:aws:sqs:us-east-1:123456789012:orders-queue",
  }),
);

console.log(SubscriptionArn);
```

For SQS subscriptions, the queue also needs a resource policy that allows the SNS topic to send messages to it.

## SNS-Specific Gotchas

- `PublishCommand` targets one destination at a time. Set only one of `TopicArn`, `TargetArn`, or `PhoneNumber`.
- Email and HTTP(S) subscriptions can remain `PendingConfirmation` until the endpoint confirms the subscription.
- SQS and Lambda integrations usually need an additional queue policy or function permission before delivery works.
- FIFO topics must end with `.fifo`, and publishing to them requires `MessageGroupId`; `MessageDeduplicationId` is also required unless content-based deduplication is enabled.
- `MessageStructure: "json"` expects `Message` to be a JSON string containing per-protocol payloads, not a nested JavaScript object.
- Phone-number publishing uses E.164 format such as `+12065550100`.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: Cognito, `fromIni`, assume-role flows, and other explicit credential helpers.
- `@aws-sdk/client-sqs`: create queues and manage queue policies for SNS-to-SQS fan-out.
- `@aws-sdk/client-lambda`: manage Lambda permissions when subscribing functions to topics.

## Common SNS Operations

### Publish a message with attributes

```javascript
import { PublishCommand, SNSClient } from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

await sns.send(
  new PublishCommand({
    TopicArn: "arn:aws:sns:us-east-1:123456789012:orders",
    Message: JSON.stringify({ orderId: "123", status: "created" }),
    MessageAttributes: {
      eventType: {
        DataType: "String",
        StringValue: "order.created",
      },
      tenantId: {
        DataType: "String",
        StringValue: "acme",
      },
    },
  }),
);
```

### Publish protocol-specific payloads

Use `MessageStructure: "json"` when different subscribers should receive different message bodies.

```javascript
import { PublishCommand, SNSClient } from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

await sns.send(
  new PublishCommand({
    TopicArn: "arn:aws:sns:us-east-1:123456789012:orders",
    MessageStructure: "json",
    Message: JSON.stringify({
      default: "Order 123 created",
      email: "Order 123 was created and is ready for review.",
      sqs: JSON.stringify({ orderId: "123", status: "created" }),
    }),
  }),
);
```

### Publish to a FIFO topic

FIFO topic names end with `.fifo`.

```javascript
import { PublishCommand, SNSClient } from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

await sns.send(
  new PublishCommand({
    TopicArn: "arn:aws:sns:us-east-1:123456789012:orders.fifo",
    Message: JSON.stringify({ orderId: "123", status: "created" }),
    MessageGroupId: "orders",
    MessageDeduplicationId: "order-123-created",
  }),
);
```

If the topic enables content-based deduplication, `MessageDeduplicationId` can be omitted.

### List topics with pagination

```javascript
import {
  paginateListTopics,
  SNSClient,
} from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

for await (const page of paginateListTopics({ client: sns }, {})) {
  for (const topic of page.Topics ?? []) {
    console.log(topic.TopicArn);
  }
}
```

### Confirm a subscription from a token

This is useful when your application receives the confirmation token out of band.

```javascript
import {
  ConfirmSubscriptionCommand,
  SNSClient,
} from "@aws-sdk/client-sns";

const sns = new SNSClient({ region: "us-east-1" });

const { SubscriptionArn } = await sns.send(
  new ConfirmSubscriptionCommand({
    TopicArn: "arn:aws:sns:us-east-1:123456789012:orders",
    Token: "token-from-confirmation-message",
  }),
);

console.log(SubscriptionArn);
```

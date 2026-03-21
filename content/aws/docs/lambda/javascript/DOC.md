---
name: lambda
description: "AWS SDK for JavaScript v3 client for invoking and managing AWS Lambda functions"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,lambda,serverless,functions,cloud"
---

# AWS Lambda SDK for JavaScript (v3)

Use `@aws-sdk/client-lambda` to invoke Lambda functions and manage Lambda resources from modern JavaScript and TypeScript code.

## Golden Rule

- Install `@aws-sdk/client-lambda`, not the legacy `aws-sdk` v2 package.
- This doc covers package version `3.1006.0`.
- Prefer `LambdaClient` plus individual commands over the aggregated `Lambda` client.
- Set `region` explicitly in code or through standard AWS config.
- `InvokeCommand` request and response bodies are byte payloads; encode with `TextEncoder` and decode with `TextDecoder`.
- A completed invocation can still represent a Lambda runtime error, so inspect `FunctionError` and the decoded payload.

## Install

```bash
npm install @aws-sdk/client-lambda
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Client Setup

### Minimal Node.js client

```javascript
import { LambdaClient } from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { LambdaClient } from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

For browser runtimes, use explicit browser-safe credentials and verify that your IAM and CORS setup intentionally allows direct Lambda calls.

## Core Usage Pattern

The usual v3 flow is `client.send(new Command(input))`.

```javascript
import { InvokeCommand, LambdaClient } from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8");

const response = await lambda.send(
  new InvokeCommand({
    FunctionName: "process-order",
    Payload: encoder.encode(
      JSON.stringify({
        orderId: "1234",
      }),
    ),
  }),
);

const body = response.Payload
  ? JSON.parse(decoder.decode(response.Payload))
  : undefined;

if (response.FunctionError) {
  throw new Error(body?.errorMessage ?? "Lambda invocation failed");
}

console.log(body);
```

## Common Operations

### Invoke asynchronously

Use `InvocationType: "Event"` when you want Lambda to queue the event and return immediately.

```javascript
import { InvokeCommand, LambdaClient } from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });

await lambda.send(
  new InvokeCommand({
    FunctionName: "process-order",
    InvocationType: "Event",
    Payload: new TextEncoder().encode(
      JSON.stringify({
        orderId: "1234",
      }),
    ),
  }),
);
```

### Read deployed function metadata

```javascript
import { GetFunctionCommand, LambdaClient } from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });

const response = await lambda.send(
  new GetFunctionCommand({
    FunctionName: "process-order",
  }),
);

console.log(response.Configuration?.Runtime);
console.log(response.Configuration?.LastModified);
```

### List functions with pagination

```javascript
import {
  LambdaClient,
  paginateListFunctions,
} from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });

for await (const page of paginateListFunctions({ client: lambda }, {})) {
  for (const item of page.Functions ?? []) {
    console.log(item.FunctionName);
  }
}
```

## Credentials and Region

In Node.js, the default credential provider chain is usually enough if you already use environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Lambda-Specific Gotchas

- `InvokeCommand` does not automatically parse JSON payloads for you.
- `InvocationType: "Event"` does not return the handler result.
- `LogType: "Tail"` returns base64-encoded log tail data on synchronous invokes.
- Lambda handler failures may come back as `FunctionError` plus an error payload instead of an SDK exception.
- Resource-based invoke permissions for services such as EventBridge or S3 use `AddPermissionCommand` on the function.
- For LocalStack or other emulators, set `endpoint`, `region`, and explicit test credentials.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: Cognito, STS assume-role flows, shared config helpers, and browser-safe credential setup.
- `@aws-sdk/client-cloudwatch-logs`: read or filter execution logs without depending on invoke log tails.
- `@aws-sdk/client-iam`: manage execution roles and related IAM policies.

## Common Lambda Operations

### Update function configuration

Use `UpdateFunctionConfigurationCommand` for memory, timeout, environment variables, runtime settings, and similar configuration changes.

```javascript
import {
  LambdaClient,
  UpdateFunctionConfigurationCommand,
} from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });

await lambda.send(
  new UpdateFunctionConfigurationCommand({
    FunctionName: "process-order",
    MemorySize: 512,
    Timeout: 30,
    Environment: {
      Variables: {
        STAGE: "prod",
      },
    },
  }),
);
```

### Update function code from a zip file

```javascript
import { readFile } from "node:fs/promises";
import {
  LambdaClient,
  UpdateFunctionCodeCommand,
} from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });
const zipFile = await readFile("./dist/function.zip");

await lambda.send(
  new UpdateFunctionCodeCommand({
    FunctionName: "process-order",
    ZipFile: zipFile,
    Publish: true,
  }),
);
```

### Add invoke permission for another AWS service

Use a function resource policy when another AWS service should invoke the function.

```javascript
import {
  AddPermissionCommand,
  LambdaClient,
} from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });

await lambda.send(
  new AddPermissionCommand({
    FunctionName: "process-order",
    StatementId: "allow-eventbridge-rule",
    Action: "lambda:InvokeFunction",
    Principal: "events.amazonaws.com",
    SourceArn: "arn:aws:events:us-east-1:123456789012:rule/process-order",
  }),
);
```

### Create an event source mapping

Use an event source mapping for poll-based integrations such as SQS or DynamoDB Streams.

```javascript
import {
  CreateEventSourceMappingCommand,
  LambdaClient,
} from "@aws-sdk/client-lambda";

const lambda = new LambdaClient({ region: "us-east-1" });

await lambda.send(
  new CreateEventSourceMappingCommand({
    FunctionName: "process-order",
    EventSourceArn: "arn:aws:sqs:us-east-1:123456789012:orders",
    BatchSize: 10,
    Enabled: true,
  }),
);
```

### Notes

- Configuration and code updates are separate APIs.
- If later steps depend on the updated state, re-read the function configuration before assuming the change is active everywhere.
- For event source mappings, the event source itself usually needs its own service-specific permissions and setup.

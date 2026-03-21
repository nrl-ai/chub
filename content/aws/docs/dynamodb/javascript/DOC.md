---
name: dynamodb
description: "AWS SDK for JavaScript v3 DynamoDB client for low-level table and item operations."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,dynamodb,javascript,nodejs,browser,nosql"
---

# `@aws-sdk/client-dynamodb`

Use this package for the low-level Amazon DynamoDB API in AWS SDK for JavaScript v3. It works in Node.js, browsers, and React Native, and it follows the v3 client-plus-command pattern.

## Install

```bash
npm install @aws-sdk/client-dynamodb
```

Prefer `DynamoDBClient` plus explicit command imports. The aggregated `DynamoDB` client exists for v2-style usage, but AWS documents it as a bigger-bundle compatibility path.

## Initialize the client

```javascript
import { DynamoDBClient } from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });
```

In Node.js, the SDK can use the default credential provider chain, so setting the region is often enough in application code.

## Credentials and Region

- Node.js: the SDK has a default credential provider chain, so credentials often come from environment variables, shared AWS config files, IAM roles, or IAM Identity Center.
- Browser runtimes: use an explicit credential provider such as Cognito identity; do not hard-code access keys in client-side code.
- Region is required somewhere. Set it in the client constructor, via `AWS_REGION`, or via shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Low-Level Item Model

This client uses raw DynamoDB `AttributeValue` shapes, not plain JavaScript objects.

```javascript
const item = {
  pk: { S: "user#123" },
  sk: { S: "profile" },
  age: { N: "42" },
  active: { BOOL: true },
};
```

If you want plain-object reads and writes, use `@aws-sdk/lib-dynamodb` or `marshall`/`unmarshall` from `@aws-sdk/util-dynamodb` alongside this client.

## Core Usage Pattern

```javascript
import {
  DynamoDBClient,
  GetItemCommand,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

const response = await dynamodb.send(
  new GetItemCommand({
    TableName: "Users",
    Key: {
      pk: { S: "user#123" },
      sk: { S: "profile" },
    },
  }),
);

console.log(response.Item);
```

## Common Operations

### Put an item

```javascript
import {
  DynamoDBClient,
  PutItemCommand,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

await dynamodb.send(
  new PutItemCommand({
    TableName: "Users",
    Item: {
      pk: { S: "user#123" },
      sk: { S: "profile" },
      email: { S: "ada@example.com" },
      active: { BOOL: true },
    },
    ConditionExpression: "attribute_not_exists(pk)",
  }),
);
```

### Query a partition

```javascript
import {
  DynamoDBClient,
  QueryCommand,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

const response = await dynamodb.send(
  new QueryCommand({
    TableName: "Users",
    KeyConditionExpression: "pk = :pk AND begins_with(sk, :prefix)",
    ExpressionAttributeValues: {
      ":pk": { S: "user#123" },
      ":prefix": { S: "order#" },
    },
    ScanIndexForward: false,
    Limit: 25,
  }),
);

console.log(response.Items ?? []);
```

## DynamoDB-Specific Gotchas

- `@aws-sdk/client-dynamodb` is the low-level API; every `Item`, `Key`, and `ExpressionAttributeValues` entry must use DynamoDB attribute descriptors like `{ S: "..." }` and `{ N: "..." }`.
- `Query` is for key-based access patterns. `Scan` reads the whole table or index and is usually the wrong default for application code.
- `GetItem` returns no item on a miss; handle `response.Item` being absent.
- `BatchWriteItem` can succeed partially and return `UnprocessedItems`; retry those instead of assuming the batch fully wrote.
- New tables are not immediately usable after `CreateTable`; wait until the table is active before writing.
- Prefer command imports over v2-style aggregated client usage.

## When To Reach For Other Packages

- `@aws-sdk/lib-dynamodb`: document client wrapper for plain JavaScript objects.
- `@aws-sdk/util-dynamodb`: `marshall` and `unmarshall` helpers.
- `@aws-sdk/credential-providers`: Cognito and other explicit credential providers.

## Common DynamoDB Operations

### Create a table

```javascript
import {
  CreateTableCommand,
  DynamoDBClient,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

await dynamodb.send(
  new CreateTableCommand({
    TableName: "Users",
    BillingMode: "PAY_PER_REQUEST",
    AttributeDefinitions: [
      { AttributeName: "pk", AttributeType: "S" },
      { AttributeName: "sk", AttributeType: "S" },
    ],
    KeySchema: [
      { AttributeName: "pk", KeyType: "HASH" },
      { AttributeName: "sk", KeyType: "RANGE" },
    ],
  }),
);
```

Table creation is asynchronous. Do not immediately assume the table is ready for reads and writes.

### Get an item

```javascript
import {
  DynamoDBClient,
  GetItemCommand,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

const { Item } = await dynamodb.send(
  new GetItemCommand({
    TableName: "Users",
    Key: {
      pk: { S: "user#123" },
      sk: { S: "profile" },
    },
  }),
);

if (!Item) {
  console.log("not found");
}
```

### Update an item

```javascript
import {
  DynamoDBClient,
  UpdateItemCommand,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

const response = await dynamodb.send(
  new UpdateItemCommand({
    TableName: "Users",
    Key: {
      pk: { S: "user#123" },
      sk: { S: "profile" },
    },
    UpdateExpression: "SET #name = :name, updatedAt = :updatedAt",
    ExpressionAttributeNames: {
      "#name": "name",
    },
    ExpressionAttributeValues: {
      ":name": { S: "Ada Lovelace" },
      ":updatedAt": { S: new Date().toISOString() },
    },
    ReturnValues: "ALL_NEW",
  }),
);

console.log(response.Attributes);
```

### Query a partition key

Use `Query` when you know the partition key and optionally want to narrow by sort key.

```javascript
import {
  DynamoDBClient,
  QueryCommand,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

const response = await dynamodb.send(
  new QueryCommand({
    TableName: "Users",
    KeyConditionExpression: "pk = :pk AND begins_with(sk, :prefix)",
    ExpressionAttributeValues: {
      ":pk": { S: "user#123" },
      ":prefix": { S: "order#" },
    },
    Limit: 25,
  }),
);

console.log(response.Items ?? []);
console.log(response.LastEvaluatedKey);
```

If `LastEvaluatedKey` is present, send another `QueryCommand` with `ExclusiveStartKey` to fetch the next page.

### Scan a table carefully

`Scan` is a full-table or full-index read. Use it for tooling, backfills, or admin tasks more than for request-path application logic.

```javascript
import {
  DynamoDBClient,
  ScanCommand,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

const response = await dynamodb.send(
  new ScanCommand({
    TableName: "Users",
    Limit: 25,
    FilterExpression: "attribute_exists(email)",
  }),
);

console.log(response.Items ?? []);
console.log(response.LastEvaluatedKey);
```

### Batch write with retry handling

`BatchWriteItem` can return unprocessed work even when the request itself succeeds.

```javascript
import {
  BatchWriteItemCommand,
  DynamoDBClient,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

let requestItems = {
  Users: [
    {
      PutRequest: {
        Item: {
          pk: { S: "user#123" },
          sk: { S: "profile" },
        },
      },
    },
    {
      DeleteRequest: {
        Key: {
          pk: { S: "user#999" },
          sk: { S: "profile" },
        },
      },
    },
  ],
};

do {
  const response = await dynamodb.send(
    new BatchWriteItemCommand({
      RequestItems: requestItems,
    }),
  );

  requestItems = response.UnprocessedItems ?? {};
} while (Object.keys(requestItems).length > 0);
```

## Advanced Patterns and Migration Notes

### Browser credentials

In browser runtimes, do not assume the Node.js default credential provider chain exists. Use an explicit credential provider.

```javascript
import { DynamoDBClient } from "@aws-sdk/client-dynamodb";
import { fromCognitoIdentityPool } from "@aws-sdk/credential-providers";

const dynamodb = new DynamoDBClient({
  region: "us-east-1",
  credentials: fromCognitoIdentityPool({
    clientConfig: { region: "us-east-1" },
    identityPoolId: "us-east-1:example-pool-id",
  }),
});
```

Do not embed long-lived access keys in browser code.

### Marshall and unmarshall data

This package is intentionally low-level. If your application works with normal JavaScript objects, convert them at the boundary.

```javascript
import { DynamoDBClient, PutItemCommand } from "@aws-sdk/client-dynamodb";
import { marshall, unmarshall } from "@aws-sdk/util-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

await dynamodb.send(
  new PutItemCommand({
    TableName: "Users",
    Item: marshall({
      pk: "user#123",
      sk: "profile",
      active: true,
      age: 42,
    }),
  }),
);

const raw = {
  pk: { S: "user#123" },
  active: { BOOL: true },
};

console.log(unmarshall(raw));
```

For a higher-level wrapper around the same service, use `@aws-sdk/lib-dynamodb`.

### Wait for table readiness

New tables can take time to become active. If your workflow creates a table and then immediately writes to it, wait first.

```javascript
import {
  DynamoDBClient,
  waitUntilTableExists,
} from "@aws-sdk/client-dynamodb";

const dynamodb = new DynamoDBClient({ region: "us-east-1" });

await waitUntilTableExists(
  { client: dynamodb, maxWaitTime: 60 },
  { TableName: "Users" },
);
```

### Migration notes from `aws-sdk` v2

- Replace `aws-sdk` DynamoDB usage with `@aws-sdk/client-dynamodb`.
- Prefer `DynamoDBClient` plus explicit commands over `new AWS.DynamoDB()`.
- Treat the aggregated client as a compatibility path, not the default import style.
- Do not copy v2 examples that rely on callbacks or global AWS SDK configuration objects.
- If you previously used the document client heavily, plan the migration around `@aws-sdk/lib-dynamodb` rather than hand-writing every `AttributeValue` shape.

### Query and consistency notes

- `Query` is the normal read path when your key design matches the access pattern.
- `Scan` is expensive and usually reserved for maintenance flows, migrations, or admin tooling.
- Strongly consistent reads are opt-in on supported operations such as `GetItem` and `Query`, but they are not a substitute for fixing a poor key design.

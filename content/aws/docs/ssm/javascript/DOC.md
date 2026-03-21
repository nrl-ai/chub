---
name: ssm
description: "AWS SDK for JavaScript v3 Systems Manager client for Parameter Store, Run Command, and related SSM APIs."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,ssm,systems-manager,javascript,nodejs,parameter-store,run-command"
---

# `@aws-sdk/client-ssm`

Use this package for Amazon Systems Manager APIs in AWS SDK for JavaScript v3. The most common application use cases are Parameter Store reads and writes, parameter hierarchy traversal, and Run Command execution against managed nodes.

## Install

```bash
npm install @aws-sdk/client-ssm
```

Prefer `SSMClient` plus explicit command imports. The package also exposes an aggregated `SSM` client, but command-based imports are the safer default for clearer dependencies and smaller bundles.

## Initialize the client

```javascript
import { SSMClient } from "@aws-sdk/client-ssm";

const ssm = new SSMClient({ region: "us-east-1" });
```

Most SSM workloads run from trusted server-side code. In Node.js, the default credential provider chain is usually enough if you already configured AWS access through environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

The v3 SDK uses client-plus-command calls:

```javascript
import {
  GetParameterCommand,
  SSMClient,
} from "@aws-sdk/client-ssm";

const ssm = new SSMClient({ region: "us-east-1" });

const response = await ssm.send(
  new GetParameterCommand({
    Name: "/app/prod/db/password",
    WithDecryption: true,
  }),
);

console.log(response.Parameter?.Value);
```

Use `WithDecryption: true` when reading `SecureString` parameters and ensure the caller also has permission to use the backing KMS key.

## Common Operations

### Put or update a parameter

```javascript
import {
  PutParameterCommand,
  SSMClient,
} from "@aws-sdk/client-ssm";

const ssm = new SSMClient({ region: "us-east-1" });

await ssm.send(
  new PutParameterCommand({
    Name: "/app/prod/api/base-url",
    Value: "https://api.example.com",
    Type: "String",
    Overwrite: true,
  }),
);
```

Overwriting a parameter creates a new parameter version instead of mutating history in place.

### Walk a parameter hierarchy

```javascript
import {
  GetParametersByPathCommand,
  SSMClient,
} from "@aws-sdk/client-ssm";

const ssm = new SSMClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await ssm.send(
    new GetParametersByPathCommand({
      Path: "/app/prod",
      Recursive: true,
      WithDecryption: true,
      NextToken: nextToken,
    }),
  );

  for (const parameter of page.Parameters ?? []) {
    console.log(parameter.Name, parameter.Value);
  }

  nextToken = page.NextToken;
} while (nextToken);
```

Parameter Store names are hierarchical slash-delimited strings. Traverse by path instead of maintaining a hard-coded list when your app groups configuration under a shared prefix.

### Send a Run Command and inspect the invocation

```javascript
import {
  GetCommandInvocationCommand,
  SendCommandCommand,
  SSMClient,
} from "@aws-sdk/client-ssm";

const ssm = new SSMClient({ region: "us-east-1" });

const sendResult = await ssm.send(
  new SendCommandCommand({
    InstanceIds: ["i-0123456789abcdef0"],
    DocumentName: "AWS-RunShellScript",
    Parameters: {
      commands: ["echo hello from systems manager"],
    },
  }),
);

const commandId = sendResult.Command?.CommandId;

if (!commandId) {
  throw new Error("Missing command id");
}

const invocation = await ssm.send(
  new GetCommandInvocationCommand({
    CommandId: commandId,
    InstanceId: "i-0123456789abcdef0",
  }),
);

console.log(invocation.Status, invocation.StandardOutputContent);
```

`SendCommand` is asynchronous. A successful send only means Systems Manager accepted the request, not that the command finished successfully on the target.

## SSM-Specific Gotchas

- `SecureString` reads need `WithDecryption: true` and matching KMS permissions.
- Many list-style SSM APIs use `NextToken`; do not assume a single response contains everything.
- Parameter names are case-sensitive and path-like naming conventions are operationally easier to manage.
- Run Command requires managed nodes that are actually registered with Systems Manager and have the right IAM/agent setup.
- `SendCommand` acceptance is not execution success; inspect invocation status and output separately.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, Cognito, and assume-role credential helpers.
- `@aws-sdk/client-kms`: direct KMS key management or cryptographic workflows outside normal Parameter Store reads and writes.

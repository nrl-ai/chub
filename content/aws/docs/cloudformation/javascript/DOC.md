---
name: cloudformation
description: "AWS SDK for JavaScript v3 CloudFormation client for creating, updating, and inspecting stacks."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,cloudformation,javascript,nodejs,browser,infrastructure"
---

# `@aws-sdk/client-cloudformation`

Use this package for Amazon CloudFormation from AWS SDK for JavaScript v3. It follows the v3 client-plus-command pattern and works in Node.js, browsers, and React Native.

Prefer `CloudFormationClient` plus explicit command imports. The package also exposes an aggregated `CloudFormation` client, but command-based imports are the safer default for clearer usage and smaller bundles.

## Install

```bash
npm install @aws-sdk/client-cloudformation
```

## Initialize the client

```javascript
import { CloudFormationClient } from "@aws-sdk/client-cloudformation";

const cloudformation = new CloudFormationClient({ region: "us-east-1" });
```

In Node.js, the default credential provider chain is usually enough if AWS access is already configured through environment variables, shared config files, IAM roles, or IAM Identity Center.

## Credentials and Region

- Node.js: credentials often come from environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.
- Browser runtimes: use an explicit browser-safe credential provider such as Cognito identity; do not hard-code access keys in client-side code.
- Region is required somewhere. Set it in the client constructor, via `AWS_REGION`, or via shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

```javascript
import {
  CloudFormationClient,
  DescribeStacksCommand,
} from "@aws-sdk/client-cloudformation";

const cloudformation = new CloudFormationClient({ region: "us-east-1" });

const response = await cloudformation.send(
  new DescribeStacksCommand({
    StackName: "app-infra",
  }),
);

const stack = response.Stacks?.[0];
console.log(stack?.StackStatus);
```

## Common Operations

### Create a stack from a template URL

```javascript
import {
  CloudFormationClient,
  CreateStackCommand,
} from "@aws-sdk/client-cloudformation";

const cloudformation = new CloudFormationClient({ region: "us-east-1" });

const response = await cloudformation.send(
  new CreateStackCommand({
    StackName: "app-infra",
    TemplateURL: "https://s3.amazonaws.com/example-bucket/templates/app.yaml",
    Parameters: [
      {
        ParameterKey: "Environment",
        ParameterValue: "prod",
      },
    ],
    Capabilities: ["CAPABILITY_NAMED_IAM"],
    Tags: [{ Key: "service", Value: "billing" }],
  }),
);

console.log(response.StackId);
```

Use `Capabilities` only when the template requires them, such as when it creates or updates IAM resources.

### Update a stack

```javascript
import {
  CloudFormationClient,
  UpdateStackCommand,
} from "@aws-sdk/client-cloudformation";

const cloudformation = new CloudFormationClient({ region: "us-east-1" });

try {
  const response = await cloudformation.send(
    new UpdateStackCommand({
      StackName: "app-infra",
      TemplateURL: "https://s3.amazonaws.com/example-bucket/templates/app.yaml",
      Parameters: [
        {
          ParameterKey: "Environment",
          ParameterValue: "prod",
        },
      ],
      Capabilities: ["CAPABILITY_NAMED_IAM"],
    }),
  );

  console.log(response.StackId);
} catch (error) {
  if (
    error.name === "ValidationError" &&
    error.message?.includes("No updates are to be performed")
  ) {
    console.log("Stack is already up to date.");
  } else {
    throw error;
  }
}
```

### Delete a stack

```javascript
import {
  CloudFormationClient,
  DeleteStackCommand,
} from "@aws-sdk/client-cloudformation";

const cloudformation = new CloudFormationClient({ region: "us-east-1" });

await cloudformation.send(
  new DeleteStackCommand({
    StackName: "app-infra",
  }),
);
```

### Describe recent stack events

```javascript
import {
  CloudFormationClient,
  DescribeStackEventsCommand,
} from "@aws-sdk/client-cloudformation";

const cloudformation = new CloudFormationClient({ region: "us-east-1" });

const response = await cloudformation.send(
  new DescribeStackEventsCommand({
    StackName: "app-infra",
  }),
);

for (const event of response.StackEvents ?? []) {
  console.log(event.Timestamp, event.ResourceStatus, event.LogicalResourceId);
}
```

## CloudFormation-Specific Gotchas

- `CreateStack`, `UpdateStack`, and `DeleteStack` are asynchronous. Do not assume success until the stack reaches a terminal status.
- Use change sets when you want to review the impact of a template change before execution.
- `TemplateBody` is convenient for small inline templates; larger templates are usually stored in S3 and referenced with `TemplateURL`.
- A template that creates or updates IAM resources requires an explicit capability acknowledgement such as `CAPABILITY_IAM` or `CAPABILITY_NAMED_IAM`.
- `UpdateStack` commonly returns a `ValidationError` with `No updates are to be performed` when the resolved template and parameters do not change the stack.
- Parameter values are passed as strings, even when the template interprets them as numbers or booleans.
- Stack deletion can fail when resources still have external dependencies or retention settings.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: Cognito, STS assume-role flows, shared config helpers, and other credential providers.
- `@aws-sdk/client-s3`: storing and versioning templates in S3 before deploying them with CloudFormation.

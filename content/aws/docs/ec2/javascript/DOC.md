---
name: ec2
description: "AWS SDK for JavaScript v3 EC2 client for managing instances and related compute resources."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,ec2,javascript,nodejs,browser,compute,instances"
---

# `@aws-sdk/client-ec2`

Use this package for the low-level Amazon EC2 API in AWS SDK for JavaScript v3. Prefer `EC2Client` plus explicit command imports for instance lifecycle, inventory, tagging, networking, and other EC2 control-plane operations.

## Install

```bash
npm install @aws-sdk/client-ec2
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Initialize the client

```javascript
import { EC2Client } from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });
```

In Node.js, the default credential provider chain is usually enough if your AWS access is already configured.

## Credentials and Region

- Node.js: credentials often come from environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.
- Browser runtimes: use explicit browser-safe credentials such as Cognito. Direct EC2 access from the browser is uncommon and needs intentional IAM and CORS design.
- Region is required somewhere: set it in code, via `AWS_REGION`, or through shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

`DescribeInstances` is a common read path, but remember that instances are nested under reservations.

```javascript
import {
  DescribeInstancesCommand,
  EC2Client,
} from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

const response = await ec2.send(
  new DescribeInstancesCommand({
    Filters: [
      {
        Name: "instance-state-name",
        Values: ["running"],
      },
    ],
  }),
);

const instances = (response.Reservations ?? []).flatMap(
  (reservation) => reservation.Instances ?? [],
);

for (const instance of instances) {
  console.log(instance.InstanceId, instance.State?.Name);
}
```

Prefer `EC2Client` plus explicit commands for most code. The package also exports an aggregated `EC2` client, but command-based imports keep dependencies and bundle size more predictable.

## Common Operations

### Launch an instance

```javascript
import { EC2Client, RunInstancesCommand } from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

const response = await ec2.send(
  new RunInstancesCommand({
    ImageId: "ami-0123456789abcdef0",
    InstanceType: "t3.micro",
    MinCount: 1,
    MaxCount: 1,
    SubnetId: "subnet-0123456789abcdef0",
    SecurityGroupIds: ["sg-0123456789abcdef0"],
    ClientToken: "launch-web-1",
    TagSpecifications: [
      {
        ResourceType: "instance",
        Tags: [
          {
            Key: "Name",
            Value: "web-1",
          },
        ],
      },
    ],
  }),
);

console.log(response.Instances?.[0]?.InstanceId);
```

### Stop an instance

```javascript
import { EC2Client, StopInstancesCommand } from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

await ec2.send(
  new StopInstancesCommand({
    InstanceIds: ["i-0123456789abcdef0"],
  }),
);
```

### List instances with pagination

```javascript
import {
  EC2Client,
  paginateDescribeInstances,
} from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

const paginator = paginateDescribeInstances(
  { client: ec2 },
  {
    Filters: [
      {
        Name: "tag:Environment",
        Values: ["prod"],
      },
    ],
  },
);

for await (const page of paginator) {
  for (const reservation of page.Reservations ?? []) {
    for (const instance of reservation.Instances ?? []) {
      console.log(instance.InstanceId);
    }
  }
}
```

## EC2-Specific Gotchas

- `DescribeInstances` and similar APIs return `Reservations`, then `Instances`; do not expect a flat top-level array.
- Many mutating calls are asynchronous. After `RunInstances`, `StartInstances`, `StopInstances`, or `TerminateInstances`, wait for the state you need before assuming follow-up calls will succeed.
- `RunInstances` requires both `MinCount` and `MaxCount`, even when launching exactly one instance.
- Use `ClientToken` on launch flows so retries do not create duplicate instances.
- `DryRun: true` does not return a normal success response; EC2 commonly signals permission checks through `DryRunOperation` or `UnauthorizedOperation` errors.
- For repeated launch configuration, prefer launch templates instead of duplicating large `RunInstances` payloads.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: Cognito, STS assume-role flows, shared config helpers, and browser-safe credential setup.
- `@aws-sdk/client-auto-scaling`: manage Auto Scaling groups instead of hand-rolling group behavior around individual instance launches.
- `@aws-sdk/client-ssm`: fetch AMI IDs or configuration values from Parameter Store, or work with EC2 instances through Systems Manager workflows.

## Common EC2 Operations And Waiters

### Wait for a launched instance to be running

EC2 launches are asynchronous. If later steps need the instance to exist in a running state, wait explicitly.

```javascript
import {
  EC2Client,
  RunInstancesCommand,
  waitUntilInstanceRunning,
} from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

const launch = await ec2.send(
  new RunInstancesCommand({
    ImageId: "ami-0123456789abcdef0",
    InstanceType: "t3.micro",
    MinCount: 1,
    MaxCount: 1,
  }),
);

const instanceId = launch.Instances?.[0]?.InstanceId;

if (!instanceId) {
  throw new Error("RunInstances did not return an instance ID");
}

await waitUntilInstanceRunning(
  { client: ec2, maxWaitTime: 300 },
  { InstanceIds: [instanceId] },
);
```

### Terminate an instance

```javascript
import {
  EC2Client,
  TerminateInstancesCommand,
} from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

await ec2.send(
  new TerminateInstancesCommand({
    InstanceIds: ["i-0123456789abcdef0"],
  }),
);
```

### Add or update tags

Use `CreateTagsCommand` for instances, volumes, snapshots, ENIs, and other EC2 resources that support tags.

```javascript
import {
  CreateTagsCommand,
  EC2Client,
} from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

await ec2.send(
  new CreateTagsCommand({
    Resources: ["i-0123456789abcdef0"],
    Tags: [
      {
        Key: "Environment",
        Value: "prod",
      },
      {
        Key: "Owner",
        Value: "platform",
      },
    ],
  }),
);
```

### Use `DryRun` to check permissions

Many EC2 APIs support `DryRun`. Treat `DryRunOperation` as proof that the caller is authorized for the action.

```javascript
import { EC2Client, StopInstancesCommand } from "@aws-sdk/client-ec2";

const ec2 = new EC2Client({ region: "us-east-1" });

try {
  await ec2.send(
    new StopInstancesCommand({
      InstanceIds: ["i-0123456789abcdef0"],
      DryRun: true,
    }),
  );
} catch (error) {
  if (error.name === "DryRunOperation") {
    console.log("The caller is allowed to stop this instance.");
  } else if (error.name === "UnauthorizedOperation") {
    console.log("The caller is not allowed to stop this instance.");
  } else {
    throw error;
  }
}
```

### Notes

- Waiters poll describe-style APIs and are a better default than hand-written sleep loops.
- EC2 has eventual consistency across instance, volume, ENI, and tagging state; read-after-write flows should expect short propagation windows.
- Launch templates are usually the right abstraction once `RunInstances` inputs become repetitive or environment-specific.

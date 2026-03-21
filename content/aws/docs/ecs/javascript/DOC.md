---
name: ecs
description: "AWS SDK for JavaScript v3 ECS client for clusters, services, task definitions, and task lifecycle operations."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,ecs,javascript,nodejs,containers,fargate"
---

# `@aws-sdk/client-ecs`

Use this package for Amazon ECS control-plane operations such as describing services, running one-off tasks, registering task definitions, and inspecting task state. It follows the AWS SDK for JavaScript v3 client-plus-command pattern.

## Install

```bash
npm install @aws-sdk/client-ecs
```

Prefer `ECSClient` plus explicit command imports for application code.

## Initialize the client

```javascript
import { ECSClient } from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });
```

In Node.js, the default credential provider chain is usually enough if credentials are already configured. If this code runs inside an ECS task, the SDK can also use task-role credentials exposed through the container metadata endpoint.

## Credentials and Region

- In Node.js, credentials can come from environment variables, shared AWS config files, IAM Identity Center, ECS task roles, or EC2 instance profiles.
- In browser runtimes, use an explicit browser-safe credential provider such as Cognito. Do not ship long-lived AWS keys in client code.
- Region is required somewhere. Set it in code, via `AWS_REGION`, or via shared AWS config.
- For workloads already running on ECS, prefer IAM task roles instead of injecting static credentials into the container.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

Describe a service to inspect desired count, running count, and active deployments:

```javascript
import {
  DescribeServicesCommand,
  ECSClient,
} from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });

const { services = [], failures = [] } = await ecs.send(
  new DescribeServicesCommand({
    cluster: "prod-cluster",
    services: ["api"],
  }),
);

console.log(services[0]?.runningCount);
console.log(services[0]?.deployments);
console.log(failures);
```

## Common Operations

### Run a one-off Fargate task

```javascript
import { ECSClient, RunTaskCommand } from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });

const response = await ecs.send(
  new RunTaskCommand({
    cluster: "prod-cluster",
    taskDefinition: "worker:12",
    launchType: "FARGATE",
    networkConfiguration: {
      awsvpcConfiguration: {
        subnets: ["subnet-abc123"],
        securityGroups: ["sg-abc123"],
        assignPublicIp: "DISABLED",
      },
    },
    overrides: {
      containerOverrides: [
        {
          name: "worker",
          environment: [{ name: "JOB_ID", value: "123" }],
        },
      ],
    },
  }),
);

console.log(response.tasks?.[0]?.taskArn);
console.log(response.failures);
```

Check `failures` even when the request itself succeeds.

### List and describe running tasks

```javascript
import {
  DescribeTasksCommand,
  ECSClient,
  ListTasksCommand,
} from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });

const { taskArns = [] } = await ecs.send(
  new ListTasksCommand({
    cluster: "prod-cluster",
    serviceName: "api",
    desiredStatus: "RUNNING",
  }),
);

if (taskArns.length > 0) {
  const { tasks = [] } = await ecs.send(
    new DescribeTasksCommand({
      cluster: "prod-cluster",
      tasks: taskArns,
    }),
  );

  console.log(
    tasks.map((task) => ({
      taskArn: task.taskArn,
      lastStatus: task.lastStatus,
      stoppedReason: task.stoppedReason,
    })),
  );
}
```

### Update a service deployment

```javascript
import { ECSClient, UpdateServiceCommand } from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });

await ecs.send(
  new UpdateServiceCommand({
    cluster: "prod-cluster",
    service: "api",
    desiredCount: 4,
    forceNewDeployment: true,
  }),
);
```

After `UpdateService`, poll `DescribeServicesCommand` until the deployment settles before assuming the rollout is complete.

## ECS-Specific Gotchas

- `RunTask`, `StartTask`, and `DescribeServices` can return `failures` alongside otherwise successful responses.
- `ListTasks` returns task ARNs only. Use `DescribeTasks` to inspect status, container exit codes, and stop reasons.
- Fargate tasks need a compatible task definition and a runtime `networkConfiguration` with `awsvpcConfiguration`.
- `launchType` and `capacityProviderStrategy` are alternative scheduling modes. Do not send both in the same request.
- Application AWS permissions belong on the task role. Image pulls, log delivery, and secret injection use the execution role.
- ECS service changes are eventually consistent. Poll for steady state after mutating service configuration.
- Do not deep-import package internals from build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: explicit credential flows such as Cognito, `fromIni`, and assume-role helpers.
- `@aws-sdk/client-cloudwatch-logs`: manage log groups and log streams referenced by ECS task definitions.
- `@aws-sdk/client-service-discovery`: manage Cloud Map namespaces and services used by ECS service discovery.

## Common ECS Operations

### Register a Fargate task definition

Use a task definition compatible with `awsvpc` networking and Fargate sizing.

```javascript
import {
  ECSClient,
  RegisterTaskDefinitionCommand,
} from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });

const { taskDefinition } = await ecs.send(
  new RegisterTaskDefinitionCommand({
    family: "api",
    networkMode: "awsvpc",
    requiresCompatibilities: ["FARGATE"],
    cpu: "512",
    memory: "1024",
    executionRoleArn:
      "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
    taskRoleArn: "arn:aws:iam::123456789012:role/apiTaskRole",
    containerDefinitions: [
      {
        name: "api",
        image: "123456789012.dkr.ecr.us-east-1.amazonaws.com/api:latest",
        essential: true,
        portMappings: [
          {
            containerPort: 3000,
            protocol: "tcp",
          },
        ],
        logConfiguration: {
          logDriver: "awslogs",
          options: {
            "awslogs-group": "/ecs/api",
            "awslogs-region": "us-east-1",
            "awslogs-stream-prefix": "ecs",
          },
        },
      },
    ],
  }),
);

console.log(taskDefinition?.taskDefinitionArn);
```

### Stop a running task

```javascript
import { ECSClient, StopTaskCommand } from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });

const { task } = await ecs.send(
  new StopTaskCommand({
    cluster: "prod-cluster",
    task: "arn:aws:ecs:us-east-1:123456789012:task/prod-cluster/abc123",
    reason: "manual rollback",
  }),
);

console.log(task?.lastStatus);
```

After a stop request, use `DescribeTasksCommand` to inspect `stoppedReason` and per-container `exitCode` values.

### List services with manual pagination

```javascript
import { ECSClient, ListServicesCommand } from "@aws-sdk/client-ecs";

const ecs = new ECSClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await ecs.send(
    new ListServicesCommand({
      cluster: "prod-cluster",
      nextToken,
      maxResults: 20,
    }),
  );

  for (const serviceArn of page.serviceArns ?? []) {
    console.log(serviceArn);
  }

  nextToken = page.nextToken;
} while (nextToken);
```

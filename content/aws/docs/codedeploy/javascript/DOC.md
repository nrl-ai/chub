---
name: codedeploy
description: "AWS SDK for JavaScript v3 client for managing AWS CodeDeploy applications, deployment groups, and deployments"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,codedeploy,deployments,devops,ci-cd"
---

# AWS CodeDeploy SDK for JavaScript (v3)

Use `@aws-sdk/client-codedeploy` to start deployments, inspect deployment state, and report lifecycle hook results for CodeDeploy applications targeting EC2/on-premises hosts, Lambda, or ECS.

## Golden Rule

- Install `@aws-sdk/client-codedeploy`, not the legacy `aws-sdk` v2 package.
- This doc covers package version `3.1006.0`.
- Prefer `CodeDeployClient` plus individual commands over the aggregated `CodeDeploy` client.
- Set `region` explicitly in code or via standard AWS config.
- `CreateDeployment` starts work but does not mean the deployment succeeded; poll `GetDeployment` or use the waiter.
- Most list operations use `nextToken`; use the generated paginators for anything beyond a single page.
- CodeDeploy does not build or upload your revision artifact for you; you provide an existing S3 revision, a GitHub revision for EC2/on-premises deployments, or one of the supported Lambda/ECS AppSpec-based revision payloads.

## Install

```bash
npm install @aws-sdk/client-codedeploy
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Client Setup

### Minimal Node.js client

```javascript
import { CodeDeployClient } from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { CodeDeployClient } from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

In practice, CodeDeploy calls usually run from backend automation, CI/CD jobs, or Lambda functions. Browser use is uncommon because these APIs are operationally sensitive and usually require privileged IAM permissions.

## Core Usage Pattern

The normal v3 flow is `client.send(new Command(input))`.

```javascript
import {
  CreateDeploymentCommand,
  CodeDeployClient,
} from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({ region: "us-east-1" });

const response = await codedeploy.send(
  new CreateDeploymentCommand({
    applicationName: "orders-service",
    deploymentGroupName: "orders-service-prod",
    revision: {
      revisionType: "S3",
      s3Location: {
        bucket: "my-codedeploy-artifacts",
        key: "orders-service/releases/2026-03-11.zip",
        bundleType: "zip",
      },
    },
    description: "Deploy release 2026-03-11",
  }),
);

console.log(response.deploymentId);
```

For S3 revisions, `bundleType` must match the uploaded artifact format exactly (`zip`, `tgz`, `tar`, `YAML`, or `JSON`).

## Common Operations

### Start a deployment from an S3 revision bundle

```javascript
import {
  CreateDeploymentCommand,
  CodeDeployClient,
} from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({ region: "us-east-1" });

const { deploymentId } = await codedeploy.send(
  new CreateDeploymentCommand({
    applicationName: "orders-service",
    deploymentGroupName: "orders-service-prod",
    revision: {
      revisionType: "S3",
      s3Location: {
        bucket: "my-codedeploy-artifacts",
        key: "orders-service/releases/2026-03-11.zip",
        bundleType: "zip",
      },
    },
    description: "Release 2026-03-11",
    ignoreApplicationStopFailures: false,
  }),
);

if (!deploymentId) {
  throw new Error("CodeDeploy did not return a deployment id");
}

console.log(`Started deployment ${deploymentId}`);
```

`CreateDeployment` assumes the application and deployment group already exist. This package manages the deployment workflow, not your artifact packaging pipeline.

### Wait for a deployment to finish and inspect the result

```javascript
import {
  CodeDeployClient,
  GetDeploymentCommand,
  waitUntilDeploymentSuccessful,
} from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({ region: "us-east-1" });
const deploymentId = "d-ABCDEFGHI";

await waitUntilDeploymentSuccessful(
  { client: codedeploy, maxWaitTime: 30 * 60 },
  { deploymentId },
);

const { deploymentInfo } = await codedeploy.send(
  new GetDeploymentCommand({ deploymentId }),
);

console.log({
  status: deploymentInfo?.status,
  overview: deploymentInfo?.deploymentOverview,
  messages: deploymentInfo?.deploymentStatusMessages,
});
```

The generated waiter treats `Succeeded` as success and `Failed` or `Stopped` as failure. If you need custom handling, poll `GetDeployment` yourself and inspect `deploymentInfo.status`.

### List applications and deployment groups with paginators

```javascript
import {
  CodeDeployClient,
  paginateListApplications,
  paginateListDeploymentGroups,
} from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({ region: "us-east-1" });

for await (const page of paginateListApplications({ client: codedeploy }, {})) {
  for (const applicationName of page.applications ?? []) {
    console.log("application", applicationName);
  }
}

for await (const page of paginateListDeploymentGroups(
  { client: codedeploy },
  { applicationName: "orders-service" },
)) {
  for (const deploymentGroupName of page.deploymentGroups ?? []) {
    console.log("deployment group", deploymentGroupName);
  }
}
```

The list APIs return paginated slices. Do not assume large accounts fit into one response.

### List recent deployments for a deployment group

```javascript
import {
  CodeDeployClient,
  paginateListDeployments,
} from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({ region: "us-east-1" });

for await (const page of paginateListDeployments(
  { client: codedeploy },
  {
    applicationName: "orders-service",
    deploymentGroupName: "orders-service-prod",
    includeOnlyStatuses: ["Created", "Queued", "InProgress", "Succeeded", "Failed"],
  },
)) {
  for (const deploymentId of page.deployments ?? []) {
    console.log(deploymentId);
  }
}
```

Use `includeOnlyStatuses` when you only care about active or recently failed deployments instead of the full history.

### Report lifecycle hook success from a validation Lambda

CodeDeploy Lambda and ECS deployments can invoke a validation Lambda hook. That function must call `PutLifecycleEventHookExecutionStatus` with the IDs from the event payload.

```javascript
import {
  CodeDeployClient,
  PutLifecycleEventHookExecutionStatusCommand,
} from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});

export const handler = async (event) => {
  await codedeploy.send(
    new PutLifecycleEventHookExecutionStatusCommand({
      deploymentId: event.DeploymentId,
      lifecycleEventHookExecutionId: event.LifecycleEventHookExecutionId,
      status: "Succeeded",
    }),
  );

  return { ok: true };
};
```

For Lambda deployments, the validation hooks are `BeforeAllowTraffic` and `AfterAllowTraffic`. For ECS deployments, CodeDeploy can also invoke hooks such as `BeforeInstall`, `AfterInstall`, and `AfterAllowTestTraffic`.

### Stop an in-progress deployment

```javascript
import {
  CodeDeployClient,
  StopDeploymentCommand,
} from "@aws-sdk/client-codedeploy";

const codedeploy = new CodeDeployClient({ region: "us-east-1" });

const response = await codedeploy.send(
  new StopDeploymentCommand({
    deploymentId: "d-ABCDEFGHI",
    autoRollbackEnabled: true,
  }),
);

console.log(response.status, response.statusMessage);
```

Stopping a deployment is asynchronous. The stop request can be accepted before rollback or final cleanup has finished, so keep checking deployment status afterward.

## CodeDeploy-Specific Gotchas

- `CreateDeployment` only starts the deployment; treat success as `GetDeployment().deploymentInfo.status === "Succeeded"` or a successful waiter result.
- `GetDeployment` can report `Ready` for blue/green workflows that are waiting for traffic rerouting or manual continuation.
- `ContinueDeployment` is only for blue/green deployments and supports `deploymentWaitType` values such as `READY_WAIT` and `TERMINATION_WAIT`.
- `StopDeployment` is not an instant rollback confirmation; it is a request to stop, and you still need to monitor the deployment afterward.
- S3-based revisions require a pre-uploaded artifact and a correct `bundleType`; CodeDeploy will not infer the format for you.
- GitHub revisions are limited to EC2/on-premises deployments; Lambda and ECS deployments use the Lambda/ECS AppSpec-style revision forms instead.
- The lifecycle hook callback API is for Lambda and ECS deployment hooks, not general deployment status updates.
- Large environments often require pagination for applications, deployment groups, instances, targets, and deployments.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-s3`: upload and version deployment artifacts before handing them to CodeDeploy.
- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, and assume-role credential helpers.
- `@aws-sdk/client-lambda`: invoke or manage the Lambda functions that participate in validation hooks.
- `@aws-sdk/client-ecs`: inspect or manage the ECS services attached to a CodeDeploy deployment group.

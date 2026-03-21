---
name: codepipeline
description: "AWS SDK for JavaScript v3 CodePipeline client for pipeline definitions, execution control, and delivery-state inspection."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,codepipeline,javascript,nodejs,cicd,deployments"
---

# `@aws-sdk/client-codepipeline`

Use this package for AWS CodePipeline control-plane operations such as reading pipeline definitions, starting and stopping executions, retrying failed stages, and inspecting stage or action status. It follows the AWS SDK for JavaScript v3 client-plus-command pattern.

Prefer `CodePipelineClient` plus explicit command imports for application code.

## Install

```bash
npm install @aws-sdk/client-codepipeline
```

## Initialize the client

```javascript
import { CodePipelineClient } from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });
```

In Node.js, the default credential provider chain is usually enough if AWS credentials are already configured. In practice, CodePipeline calls are usually made from backend tools, CI helpers, or automation jobs rather than browser code because these APIs often need broad deployment permissions.

## Credentials and Region

- In Node.js, credentials can come from environment variables, shared AWS config files, IAM Identity Center, ECS task roles, or EC2 instance profiles.
- In browser runtimes, use an explicit browser-safe credential provider such as Cognito, but prefer server-side execution for CodePipeline administration.
- Region is required somewhere. Set it in code, via `AWS_REGION`, or via shared AWS config.
- Pipelines are regional. A client pointed at the wrong region will not see or control the pipeline you expect.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

Inspect the current state of a pipeline and read the latest execution status for each stage:

```javascript
import {
  CodePipelineClient,
  GetPipelineStateCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

const response = await codepipeline.send(
  new GetPipelineStateCommand({
    name: "MyPipeline",
  }),
);

for (const stage of response.stageStates ?? []) {
  console.log({
    stageName: stage.stageName,
    latestStatus: stage.latestExecution?.status,
    inboundEnabled: stage.inboundTransitionState?.enabled,
  });
}
```

`GetPipelineStateCommand` is the fastest way to answer operational questions such as “which stage is failing?” and “what execution ID is currently active?”

## Common Operations

### Start a pipeline execution

By default, CodePipeline starts the latest source revision currently visible to the pipeline.

```javascript
import {
  CodePipelineClient,
  StartPipelineExecutionCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

const { pipelineExecutionId } = await codepipeline.send(
  new StartPipelineExecutionCommand({
    name: "MyPipeline",
    variables: [{ name: "Environment", value: "prod" }],
  }),
);

console.log(pipelineExecutionId);
```

If your pipeline supports overrides, you can also pass `sourceRevisions` with values such as `COMMIT_ID`, `IMAGE_DIGEST`, `S3_OBJECT_VERSION_ID`, or `S3_OBJECT_KEY` for the relevant source action.

### List recent pipeline executions

Use execution history when you need run-level status rather than stage-level status.

```javascript
import {
  CodePipelineClient,
  ListPipelineExecutionsCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await codepipeline.send(
    new ListPipelineExecutionsCommand({
      pipelineName: "MyPipeline",
      maxResults: 25,
      nextToken,
    }),
  );

  for (const execution of page.pipelineExecutionSummaries ?? []) {
    console.log({
      pipelineExecutionId: execution.pipelineExecutionId,
      status: execution.status,
      startTime: execution.startTime,
      lastUpdateTime: execution.lastUpdateTime,
    });
  }

  nextToken = page.nextToken;
} while (nextToken);
```

### Retry a failed stage execution

Retry a failed stage using the execution ID returned by `GetPipelineStateCommand` or `ListPipelineExecutionsCommand`.

```javascript
import {
  CodePipelineClient,
  RetryStageExecutionCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

const { pipelineExecutionId } = await codepipeline.send(
  new RetryStageExecutionCommand({
    pipelineName: "MyPipeline",
    stageName: "Deploy",
    pipelineExecutionId: "b59babff-5f34-EXAMPLE",
    retryMode: "FAILED_ACTIONS",
  }),
);

console.log(pipelineExecutionId);
```

Use `FAILED_ACTIONS` when you only want to rerun the failed work. Use `ALL_ACTIONS` when the entire stage must be replayed.

### Stop an in-progress execution

```javascript
import {
  CodePipelineClient,
  StopPipelineExecutionCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

await codepipeline.send(
  new StopPipelineExecutionCommand({
    pipelineName: "MyPipeline",
    pipelineExecutionId: "d-EXAMPLE",
    reason: "Stopping after the current build finishes",
    abandon: false,
  }),
);
```

With `abandon: false`, CodePipeline stops after in-progress actions finish. With `abandon: true`, it abandons in-progress actions instead, which is faster but can leave external systems midway through deployment work.

### Inspect action-level execution details

Use action execution details when a pipeline run failed but stage-level state is too coarse.

```javascript
import {
  CodePipelineClient,
  ListActionExecutionsCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

const response = await codepipeline.send(
  new ListActionExecutionsCommand({
    pipelineName: "MyPipeline",
    filter: {
      pipelineExecutionId: "EXAMPLE0-adfc-488e-bf4c-1111111720d3",
    },
  }),
);

for (const detail of response.actionExecutionDetails ?? []) {
  console.log({
    stageName: detail.stageName,
    actionName: detail.actionName,
    status: detail.status,
    externalExecutionId: detail.output?.executionResult?.externalExecutionId,
    externalExecutionSummary:
      detail.output?.executionResult?.externalExecutionSummary,
  });
}
```

## CodePipeline-Specific Gotchas

- This package manages CodePipeline itself. It does not replace service-specific clients such as CodeBuild, CodeDeploy, Lambda, or S3 that power individual pipeline actions.
- `StartPipelineExecutionCommand` runs the latest visible source by default. Use `variables` and `sourceRevisions` only when you intentionally need an override.
- `RetryStageExecutionCommand` needs the exact failed `pipelineExecutionId` plus a `retryMode` of `FAILED_ACTIONS` or `ALL_ACTIONS`.
- `StopPipelineExecutionCommand` behaves differently based on `abandon`. `false` drains in-progress work; `true` abandons it.
- `GetPipelineStateCommand` is better for current operational state, while `ListPipelineExecutionsCommand` is better for run history.
- `ListPipelinesCommand`, `ListPipelineExecutionsCommand`, and `ListActionExecutionsCommand` can paginate; loop on `nextToken` for complete results.
- `UpdatePipelineCommand` expects a full `pipeline` declaration. If you start from `GetPipelineCommand` output, reuse the `pipeline` object only and drop response-only metadata before sending the update.
- CodePipeline control-plane calls often need broad IAM permissions and are usually safer on the server side than in browser apps.
- Do not deep-import package internals from build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-codebuild`: inspect or manage build projects used by CodePipeline actions.
- `@aws-sdk/client-codedeploy`: inspect deployments launched from deploy stages.
- `@aws-sdk/client-s3`: inspect artifact buckets and source artifacts used by S3-backed pipelines.
- `@aws-sdk/credential-providers`: explicit credential flows such as `fromIni`, IAM Identity Center, Cognito, and assume-role helpers.

## Common CodePipeline Operations

### List pipelines in a region

```javascript
import {
  CodePipelineClient,
  ListPipelinesCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await codepipeline.send(
    new ListPipelinesCommand({
      maxResults: 50,
      nextToken,
    }),
  );

  for (const pipeline of page.pipelines ?? []) {
    console.log({
      name: pipeline.name,
      version: pipeline.version,
      created: pipeline.created,
      updated: pipeline.updated,
    });
  }

  nextToken = page.nextToken;
} while (nextToken);
```

### Fetch a pipeline definition before editing it

```javascript
import {
  CodePipelineClient,
  GetPipelineCommand,
} from "@aws-sdk/client-codepipeline";

const codepipeline = new CodePipelineClient({ region: "us-east-1" });

const { pipeline } = await codepipeline.send(
  new GetPipelineCommand({
    name: "MyPipeline",
  }),
);

console.log(JSON.stringify(pipeline, null, 2));
```

This is the safest starting point before reviewing stages, action configuration, artifact stores, variables, triggers, or execution mode.

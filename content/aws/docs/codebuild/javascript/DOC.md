---
name: codebuild
description: "AWS SDK for JavaScript v3 client for starting, stopping, and inspecting AWS CodeBuild builds and projects"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,codebuild,ci,builds,devops"
---

# AWS CodeBuild SDK for JavaScript (v3)

Use `@aws-sdk/client-codebuild` to trigger builds, inspect project configuration, and query build results from Node.js automation and backend services.

## Golden Rule

- Install `@aws-sdk/client-codebuild`, not the legacy `aws-sdk` v2 package.
- This doc covers package version `3.1006.0`.
- Prefer `CodeBuildClient` plus individual commands over the aggregated `CodeBuild` client.
- Set `region` explicitly in code or through standard AWS shared config.
- Treat `StartBuildCommand` as asynchronous acceptance, not final build completion.
- Many list APIs return names or IDs first; use `BatchGetProjectsCommand` or `BatchGetBuildsCommand` to load full details.
- Use this client from trusted server-side code. Direct browser use is uncommon because it requires AWS credentials with build permissions.

## Install

```bash
npm install @aws-sdk/client-codebuild
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Auth And Region

The client uses the standard AWS SDK for JavaScript v3 credential provider chain.

Common environment variables:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

If you rely on shared AWS config instead of environment variables, keep the client initialization simple and set only the region in code.

## Client Setup

### Minimal Node.js client

```javascript
import { CodeBuildClient } from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { CodeBuildClient } from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

## Core Usage Pattern

The typical v3 flow is `client.send(new Command(input))`.

```javascript
import {
  CodeBuildClient,
  StartBuildCommand,
} from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({ region: "us-east-1" });

const response = await codebuild.send(
  new StartBuildCommand({
    projectName: "my-build-project",
  }),
);

console.log(response.build?.id, response.build?.buildStatus);
```

`StartBuildCommand` returns build metadata immediately. If you need the final result, poll with `BatchGetBuildsCommand` until the build reaches a terminal status.

## Common Operations

### Start a build with per-run overrides

Use overrides when you need to change the source version, buildspec, or environment variables for a single build without editing the underlying project.

```javascript
import {
  CodeBuildClient,
  StartBuildCommand,
} from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({ region: "us-east-1" });

const result = await codebuild.send(
  new StartBuildCommand({
    projectName: "my-build-project",
    sourceVersion: "refs/heads/main",
    buildspecOverride: "buildspec.release.yml",
    environmentVariablesOverride: [
      {
        name: "DEPLOY_ENV",
        value: "staging",
        type: "PLAINTEXT",
      },
    ],
  }),
);

console.log(result.build?.id);
```

### Poll a build until it finishes

```javascript
import {
  BatchGetBuildsCommand,
  CodeBuildClient,
  StartBuildCommand,
} from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({ region: "us-east-1" });
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const startResult = await codebuild.send(
  new StartBuildCommand({
    projectName: "my-build-project",
  }),
);

const buildId = startResult.build?.id;

if (!buildId) {
  throw new Error("StartBuild did not return a build id");
}

const terminalStatuses = new Set([
  "SUCCEEDED",
  "FAILED",
  "FAULT",
  "STOPPED",
  "TIMED_OUT",
]);

let build;

while (true) {
  const response = await codebuild.send(
    new BatchGetBuildsCommand({
      ids: [buildId],
    }),
  );

  build = response.builds?.[0];

  if (!build) {
    throw new Error(`Build not found: ${buildId}`);
  }

  console.log(build.buildStatus, build.currentPhase);

  if (terminalStatuses.has(build.buildStatus ?? "")) {
    break;
  }

  await sleep(5000);
}

if (build.buildStatus !== "SUCCEEDED") {
  throw new Error(`Build ended with status ${build.buildStatus}`);
}
```

### Inspect project configuration

`BatchGetProjectsCommand` is the easiest way to fetch project details when you already know the project name.

```javascript
import {
  BatchGetProjectsCommand,
  CodeBuildClient,
} from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({ region: "us-east-1" });

const response = await codebuild.send(
  new BatchGetProjectsCommand({
    names: ["my-build-project"],
  }),
);

const project = response.projects?.[0];

console.log({
  name: project?.name,
  sourceType: project?.source?.type,
  image: project?.environment?.image,
  serviceRole: project?.serviceRole,
});
```

### List project names

`ListProjectsCommand` returns names, not full project objects.

```javascript
import {
  CodeBuildClient,
  ListProjectsCommand,
} from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await codebuild.send(
    new ListProjectsCommand({
      nextToken,
    }),
  );

  for (const name of page.projects ?? []) {
    console.log(name);
  }

  nextToken = page.nextToken;
} while (nextToken);
```

### Stop a running build

`StopBuildCommand` takes a build ID, not a project name.

```javascript
import {
  CodeBuildClient,
  StopBuildCommand,
} from "@aws-sdk/client-codebuild";

const codebuild = new CodeBuildClient({ region: "us-east-1" });

await codebuild.send(
  new StopBuildCommand({
    id: "my-build-project:11111111-2222-3333-4444-555555555555",
  }),
);
```

## CodeBuild-Specific Gotchas

- `StartBuildCommand` only queues and creates the build record. It does not wait for phases to finish.
- `ListProjectsCommand` returns project names only; call `BatchGetProjectsCommand` for source, environment, cache, VPC, and artifact settings.
- `ListBuildsForProjectCommand` returns build IDs only; call `BatchGetBuildsCommand` to inspect status, phases, logs, and artifact metadata.
- `sourceVersion` meaning depends on the project source provider. It may be a branch, tag, commit SHA, or provider-specific revision string.
- `environmentVariablesOverride` applies only to that run. It does not persist back into the CodeBuild project definition.
- Avoid hard-coding secrets into `PLAINTEXT` overrides when Parameter Store or Secrets Manager references are a better fit.
- Build logs and artifacts are delivered through the destinations configured on the project, commonly CloudWatch Logs and S3.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-cloudwatch-logs`: inspect build logs when the CodeBuild project publishes to CloudWatch Logs.
- `@aws-sdk/client-s3`: download or inspect S3 build artifacts.
- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, assume-role, and other credential helpers.
- `@aws-sdk/client-codepipeline`: coordinate builds as one step in a broader AWS delivery pipeline.

---
name: glue
description: "AWS SDK for JavaScript v3 Glue client for Data Catalog reads, crawler control, and ETL job orchestration."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,glue,javascript,nodejs,data-catalog,crawlers,etl"
---

# `@aws-sdk/client-glue`

Use this package for AWS Glue control-plane APIs in AWS SDK for JavaScript v3. The most common application use cases are reading Data Catalog metadata, creating or starting crawlers, creating jobs, starting job runs, and polling job status.

Most Glue workloads run from trusted server-side code. The package can be used anywhere the AWS SDK v3 works, but browser usage usually needs an explicit credential strategy and careful permission scoping.

## Install

```bash
npm install @aws-sdk/client-glue
```

Prefer `GlueClient` plus explicit command imports. The package also exposes an aggregated `Glue` client, but command-based imports are the safer default for smaller bundles and clearer dependencies.

## Initialize the client

```javascript
import { GlueClient } from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });
```

In Node.js, the default credential provider chain is usually enough if you already configured AWS access through environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

Read table metadata from the Glue Data Catalog with client-plus-command calls:

```javascript
import {
  GetTableCommand,
  GlueClient,
} from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });

const response = await glue.send(
  new GetTableCommand({
    DatabaseName: "analytics",
    Name: "events",
  }),
);

const columns = response.Table?.StorageDescriptor?.Columns ?? [];

for (const column of columns) {
  console.log(column.Name, column.Type);
}
```

For Hive compatibility, Glue database and table names are expected to be lowercase. If you fetch metadata for mixed-case names created elsewhere, normalize your own naming conventions before relying on catalog lookups.

## Common Operations

### List Data Catalog databases with pagination

```javascript
import {
  GetDatabasesCommand,
  GlueClient,
} from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await glue.send(
    new GetDatabasesCommand({
      NextToken: nextToken,
      MaxResults: 100,
    }),
  );

  for (const database of page.DatabaseList ?? []) {
    console.log(database.Name);
  }

  nextToken = page.NextToken;
} while (nextToken);
```

Glue list operations commonly return `NextToken`. Do not assume one response contains every database, table, crawler, or job.

### List tables in a database

```javascript
import {
  GetTablesCommand,
  GlueClient,
} from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });

const response = await glue.send(
  new GetTablesCommand({
    DatabaseName: "analytics",
    Expression: "events_*",
    MaxResults: 50,
  }),
);

for (const table of response.TableList ?? []) {
  console.log(table.Name, table.TableType);
}
```

Use `Expression` when you want server-side name filtering instead of listing a whole database and filtering in application code.

### Create and start an S3 crawler

```javascript
import {
  CreateCrawlerCommand,
  GlueClient,
  StartCrawlerCommand,
} from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });

await glue.send(
  new CreateCrawlerCommand({
    Name: "orders-landing-zone",
    Role: "arn:aws:iam::123456789012:role/AWSGlueServiceRoleDefault",
    DatabaseName: "analytics",
    Targets: {
      S3Targets: [
        {
          Path: "s3://my-bucket/raw/orders/",
        },
      ],
    },
    TablePrefix: "raw_",
  }),
);

await glue.send(
  new StartCrawlerCommand({
    Name: "orders-landing-zone",
  }),
);
```

`CreateCrawler` requires a role and at least one target. Targets can point to S3, JDBC, DynamoDB, Glue catalog targets, Delta, Iceberg, or Hudi sources.

### Create a job definition

```javascript
import {
  CreateJobCommand,
  GlueClient,
} from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });

await glue.send(
  new CreateJobCommand({
    Name: "nightly-orders-etl",
    Role: "arn:aws:iam::123456789012:role/AWSGlueServiceRoleDefault",
    Command: {
      Name: "glueetl",
      ScriptLocation: "s3://my-bucket/glue-scripts/orders-etl.py",
    },
    DefaultArguments: {
      "--job-language": "python",
      "--TempDir": "s3://my-bucket/glue-temporary/",
    },
    GlueVersion: "4.0",
    WorkerType: "G.1X",
    NumberOfWorkers: 2,
  }),
);
```

Use `Command.Name` to match the job type:

- `glueetl` for Spark ETL jobs
- `gluestreaming` for Spark streaming jobs
- `pythonshell` for Python shell jobs
- `glueray` for Ray jobs

### Start a job run and poll its status

```javascript
import {
  GetJobRunCommand,
  GlueClient,
  StartJobRunCommand,
} from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });

const start = await glue.send(
  new StartJobRunCommand({
    JobName: "nightly-orders-etl",
    Arguments: {
      "--source_bucket": "my-bucket",
      "--target_prefix": "processed/orders/",
    },
    WorkerType: "G.1X",
    NumberOfWorkers: 2,
    ExecutionClass: "STANDARD",
  }),
);

const runId = start.JobRunId;

if (!runId) {
  throw new Error("Glue did not return a job run id");
}

const terminalStates = new Set([
  "SUCCEEDED",
  "FAILED",
  "STOPPED",
  "TIMEOUT",
  "ERROR",
  "EXPIRED",
]);

let state = "STARTING";

while (!terminalStates.has(state)) {
  const result = await glue.send(
    new GetJobRunCommand({
      JobName: "nightly-orders-etl",
      RunId: runId,
    }),
  );

  state = result.JobRun?.JobRunState ?? "UNKNOWN";
  console.log(state, result.JobRun?.ErrorMessage ?? "");

  if (!terminalStates.has(state)) {
    await new Promise((resolve) => setTimeout(resolve, 15000));
  }
}
```

`StartJobRun` only queues a run. Check `GetJobRun` until the state reaches a terminal value if your workflow depends on completion.

### Read table partitions

```javascript
import {
  GetPartitionsCommand,
  GlueClient,
} from "@aws-sdk/client-glue";

const glue = new GlueClient({ region: "us-east-1" });

const response = await glue.send(
  new GetPartitionsCommand({
    DatabaseName: "analytics",
    TableName: "events",
    MaxResults: 25,
  }),
);

for (const partition of response.Partitions ?? []) {
  console.log(partition.Values);
}
```

Use `GetPartitions` for browsing or filtering partitions, and `BatchGetPartition` when you already know the exact partition values to fetch.

## Credentials and Region

- Node.js: the default credential provider chain is usually enough for server-side Glue automation.
- Browser runtimes: use an explicit credential provider and tightly scoped IAM permissions; most Glue operations are better kept off the client.
- Region must be configured somewhere, either in the client constructor, `AWS_REGION`, or shared AWS config.
- Glue jobs and crawlers also need service roles with the right permissions for S3, CloudWatch Logs, and any other resources they touch.

## Glue-Specific Gotchas

- Job arguments may be logged; do not pass plaintext secrets in `DefaultArguments` or `Arguments`.
- For Glue 2.0+ jobs, prefer `WorkerType` plus `NumberOfWorkers` instead of `MaxCapacity`.
- `StartCrawlerCommand` throws `CrawlerRunningException` if the crawler is already active.
- `StartJobRunCommand` can fail with `ConcurrentRunsExceededException` when the job is already at its allowed run concurrency.
- `GetJobRun` history is retained for 90 days, so do not treat it as your long-term execution archive.
- Data Catalog pagination is common; handle `NextToken` on reads that can span many resources.
- Table and database names are lowercase-oriented for Hive compatibility.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-s3`: store Glue ETL scripts, crawler targets, and job temporary data in S3.
- `@aws-sdk/client-athena`: query the tables registered in the Glue Data Catalog.
- `@aws-sdk/client-lakeformation`: manage Data Catalog permissions and governance in Lake Formation-aware environments.
- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, assume-role, and browser-safe credential helpers.

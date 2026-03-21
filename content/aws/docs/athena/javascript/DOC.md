---
name: athena
description: "AWS SDK for JavaScript v3 client for Amazon Athena query execution, result retrieval, and metadata APIs."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,athena,javascript,nodejs,sql,analytics,query"
---

# `@aws-sdk/client-athena`

Use this package for Amazon Athena APIs in AWS SDK for JavaScript v3. The most common application workflow is: submit a SQL statement with `StartQueryExecution`, poll `GetQueryExecution` until the query reaches a terminal state, then page through `GetQueryResults` if you need rows in-process.

Athena is usually called from trusted server-side code. The client works anywhere AWS SDK v3 works, but browser usage needs explicit credentials and careful permission scoping for both Athena and the S3 bucket that stores query results.

## Install

```bash
npm install @aws-sdk/client-athena
```

Prefer `AthenaClient` plus explicit command imports. The package also exposes an aggregated `Athena` client, but command-based imports are the safer default for smaller bundles and clearer dependency boundaries.

## Initialize the client

```javascript
import { AthenaClient } from "@aws-sdk/client-athena";

const athena = new AthenaClient({
  region: "us-east-1",
});
```

In Node.js, the default credential provider chain is usually enough if you already configured AWS access through environment variables, shared config files, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## What This Client Covers

`@aws-sdk/client-athena` covers Athena SQL query execution and most Athena control-plane APIs, including:

- running SQL statements and prepared statements
- polling query state and runtime statistics
- reading query results and manifests
- listing query history, data catalogs, databases, and table metadata
- managing workgroups, notebooks, and Spark session-oriented Athena APIs

Most application code only needs the query execution flow plus lightweight metadata lookups.

## Core Usage Pattern

Start a query with a database context and an output location for results:

```javascript
import {
  AthenaClient,
  StartQueryExecutionCommand,
} from "@aws-sdk/client-athena";

const athena = new AthenaClient({ region: "us-east-1" });

const start = await athena.send(
  new StartQueryExecutionCommand({
    QueryString: "SELECT order_id, total FROM analytics.orders LIMIT 10",
    QueryExecutionContext: {
      Catalog: "AwsDataCatalog",
      Database: "analytics",
    },
    ResultConfiguration: {
      OutputLocation: "s3://my-athena-results/query-results/",
    },
    WorkGroup: "primary",
  }),
);

const queryExecutionId = start.QueryExecutionId;

if (!queryExecutionId) {
  throw new Error("Athena did not return a query execution id");
}

console.log(queryExecutionId);
```

If your workgroup enforces its own configuration, Athena can ignore client-side result settings such as `OutputLocation` and encryption options.

## Common Operations

### Start a query and poll until it finishes

```javascript
import {
  AthenaClient,
  GetQueryExecutionCommand,
  StartQueryExecutionCommand,
} from "@aws-sdk/client-athena";

const athena = new AthenaClient({ region: "us-east-1" });
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const start = await athena.send(
  new StartQueryExecutionCommand({
    QueryString: `
      SELECT date_trunc('day', created_at) AS day, count(*) AS orders
      FROM analytics.orders
      GROUP BY 1
      ORDER BY 1 DESC
      LIMIT 7
    `,
    QueryExecutionContext: {
      Catalog: "AwsDataCatalog",
      Database: "analytics",
    },
    ResultConfiguration: {
      OutputLocation: "s3://my-athena-results/query-results/",
    },
    ResultReuseConfiguration: {
      ResultReuseByAgeConfiguration: {
        Enabled: true,
        MaxAgeInMinutes: 15,
      },
    },
    WorkGroup: "primary",
  }),
);

const queryExecutionId = start.QueryExecutionId;

if (!queryExecutionId) {
  throw new Error("Athena did not return a query execution id");
}

const terminalStates = new Set(["SUCCEEDED", "FAILED", "CANCELLED"]);
let state = "QUEUED";

while (!terminalStates.has(state)) {
  const result = await athena.send(
    new GetQueryExecutionCommand({
      QueryExecutionId: queryExecutionId,
    }),
  );

  const execution = result.QueryExecution;
  state = execution?.Status?.State ?? "UNKNOWN";

  console.log({
    state,
    bytesScanned: execution?.Statistics?.DataScannedInBytes,
    queueMs: execution?.Statistics?.QueryQueueTimeInMillis,
  });

  if (state === "FAILED" || state === "CANCELLED") {
    throw new Error(
      execution?.Status?.AthenaError?.ErrorMessage ??
        execution?.Status?.StateChangeReason ??
        `Athena query ended in state ${state}`,
    );
  }

  if (state !== "SUCCEEDED") {
    await sleep(2000);
  }
}
```

`StartQueryExecution` only submits work. If the next step depends on results, always poll `GetQueryExecution` until the state is terminal.

### Read rows from a completed query

```javascript
import {
  AthenaClient,
  GetQueryResultsCommand,
} from "@aws-sdk/client-athena";

const athena = new AthenaClient({ region: "us-east-1" });

async function getAllRows(queryExecutionId) {
  let nextToken;
  const pages = [];

  do {
    const page = await athena.send(
      new GetQueryResultsCommand({
        QueryExecutionId: queryExecutionId,
        MaxResults: 1000,
        NextToken: nextToken,
      }),
    );

    pages.push(page);
    nextToken = page.NextToken;
  } while (nextToken);

  const firstPage = pages[0];
  const columnNames =
    firstPage?.ResultSet?.ResultSetMetadata?.ColumnInfo?.map(
      (column) => column.Name ?? "",
    ) ?? [];

  const rows = pages.flatMap((page) => page.ResultSet?.Rows ?? []);

  return rows.slice(1).map((row) => {
    return Object.fromEntries(
      columnNames.map((name, index) => [
        name,
        row.Data?.[index]?.VarCharValue ?? null,
      ]),
    );
  });
}

const rows = await getAllRows("1234abcd-12ab-34cd-56ef-1234567890ab");
console.log(rows);
```

Use `ResultSetMetadata.ColumnInfo` to build stable column mappings. `GetQueryResults` is paginated, so do not assume a single response contains the full result set.

### List databases in a data catalog

```javascript
import {
  AthenaClient,
  ListDatabasesCommand,
} from "@aws-sdk/client-athena";

const athena = new AthenaClient({ region: "us-east-1" });

let nextToken;

do {
  const page = await athena.send(
    new ListDatabasesCommand({
      CatalogName: "AwsDataCatalog",
      MaxResults: 50,
      NextToken: nextToken,
    }),
  );

  for (const database of page.DatabaseList ?? []) {
    console.log(database.Name);
  }

  nextToken = page.NextToken;
} while (nextToken);
```

Use the Athena metadata APIs when you need lightweight catalog visibility from the same client that runs queries.

### Stop a running query

```javascript
import {
  AthenaClient,
  StopQueryExecutionCommand,
} from "@aws-sdk/client-athena";

const athena = new AthenaClient({ region: "us-east-1" });

await athena.send(
  new StopQueryExecutionCommand({
    QueryExecutionId: "1234abcd-12ab-34cd-56ef-1234567890ab",
  }),
);
```

Cancellation is useful when a user abandons an interactive request or a workflow exceeds a cost or time budget.

## Athena-Specific Gotchas

- `StartQueryExecution` does not wait for completion; you must inspect `GetQueryExecution` for `QUEUED`, `RUNNING`, `SUCCEEDED`, `FAILED`, or `CANCELLED`.
- Query results need an output location. Supply `ResultConfiguration.OutputLocation` unless the workgroup already defines and enforces the results configuration.
- `GetQueryResults` also depends on Amazon S3 access to the Athena results location, not just Athena API permissions.
- Query results are paginated with `NextToken`. Large result sets and manifest responses require repeated calls.
- `ListQueryExecutions` returns query history for only 45 days.
- Metadata APIs such as `ListDatabases` and `ListDataCatalogs` can require `WorkGroup` in IAM Identity Center-enabled setups.
- For failed queries, inspect both `Status.StateChangeReason` and `Status.AthenaError` instead of logging only the terminal state.

## When To Reach For Other Packages

- `@aws-sdk/client-s3`: manage Athena result buckets or read raw result objects and manifests directly from S3.
- `@aws-sdk/client-glue`: broader Glue Data Catalog, crawler, and ETL management beyond Athena's own metadata endpoints.
- `@aws-sdk/credential-providers`: explicit credential providers for Cognito, shared config, or other non-default auth flows.

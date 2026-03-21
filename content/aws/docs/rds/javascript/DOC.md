---
name: rds
description: "AWS SDK for JavaScript v3 RDS client for managing DB instances, clusters, snapshots, and related database resources."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,rds,javascript,nodejs,browser,database,aurora,postgres,mysql"
---

# `@aws-sdk/client-rds`

Use this package for Amazon RDS control-plane operations in AWS SDK for JavaScript v3. It manages DB instances, DB clusters, snapshots, subnet groups, parameter groups, tags, and other RDS resources.

Use `@aws-sdk/client-rds` to create, inspect, modify, and delete RDS resources. Do not use it to open SQL connections to PostgreSQL, MySQL, MariaDB, SQL Server, or Oracle engines running inside RDS.

Prefer `RDSClient` plus explicit command imports. The package also exposes an aggregated `RDS` client, but command-based imports are the safer default for smaller bundles and clearer dependency boundaries.

## Install

```bash
npm install @aws-sdk/client-rds
```

Common companion packages:

```bash
# Shared config, IAM Identity Center, STS assume-role, Cognito, and other credential helpers
npm install @aws-sdk/credential-providers

# IAM database authentication token generation
npm install @aws-sdk/rds-signer

# Aurora Serverless Data API over HTTPS
npm install @aws-sdk/client-rds-data
```

## Initialize the client

```javascript
import { RDSClient } from "@aws-sdk/client-rds";

const rds = new RDSClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

## Credentials and Region

- Node.js: the default credential provider chain usually works if AWS access is already configured through environment variables, shared AWS config files, ECS, EC2, or IAM Identity Center.
- Browser runtimes: use explicit browser-safe credentials, and avoid exposing privileged RDS management permissions directly to end users.
- Region is required somewhere. Set it in the client constructor, via `AWS_REGION`, or through shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

The v3 SDK uses `client.send(new Command(input))`.

```javascript
import {
  DescribeDBInstancesCommand,
  RDSClient,
} from "@aws-sdk/client-rds";

const rds = new RDSClient({ region: "us-east-1" });

const response = await rds.send(
  new DescribeDBInstancesCommand({
    MaxRecords: 20,
  }),
);

for (const instance of response.DBInstances ?? []) {
  console.log({
    id: instance.DBInstanceIdentifier,
    engine: instance.Engine,
    status: instance.DBInstanceStatus,
    endpoint: instance.Endpoint?.Address,
    port: instance.Endpoint?.Port,
  });
}
```

Use instance-level APIs for traditional RDS instances and cluster-level APIs for Aurora cluster resources.

## Common Operations

### List DB instances across pages

Many RDS `Describe*` operations are paginated with `Marker` and `MaxRecords`.

```javascript
import {
  DescribeDBInstancesCommand,
  RDSClient,
} from "@aws-sdk/client-rds";

const rds = new RDSClient({ region: "us-east-1" });

let marker;

do {
  const page = await rds.send(
    new DescribeDBInstancesCommand({
      MaxRecords: 100,
      Marker: marker,
    }),
  );

  for (const instance of page.DBInstances ?? []) {
    console.log(instance.DBInstanceIdentifier, instance.DBInstanceStatus);
  }

  marker = page.Marker;
} while (marker);
```

### Inspect an Aurora or Multi-AZ DB cluster

Use cluster APIs when the resource is modeled as a DB cluster instead of a standalone instance.

```javascript
import {
  DescribeDBClustersCommand,
  RDSClient,
} from "@aws-sdk/client-rds";

const rds = new RDSClient({ region: "us-east-1" });

const { DBClusters } = await rds.send(
  new DescribeDBClustersCommand({
    DBClusterIdentifier: "app-cluster",
  }),
);

const cluster = DBClusters?.[0];

console.log({
  id: cluster?.DBClusterIdentifier,
  engine: cluster?.Engine,
  status: cluster?.Status,
  endpoint: cluster?.Endpoint,
  readerEndpoint: cluster?.ReaderEndpoint,
});
```

### Create a manual DB snapshot

```javascript
import {
  CreateDBSnapshotCommand,
  RDSClient,
} from "@aws-sdk/client-rds";

const rds = new RDSClient({ region: "us-east-1" });

const snapshot = await rds.send(
  new CreateDBSnapshotCommand({
    DBInstanceIdentifier: "app-db-1",
    DBSnapshotIdentifier: `app-db-1-manual-${Date.now()}`,
  }),
);

console.log(snapshot.DBSnapshot?.DBSnapshotIdentifier);
```

### Modify an existing DB instance

RDS updates are often asynchronous. Submit the change, then poll `DescribeDBInstances` until the resource reaches the state you expect.

```javascript
import {
  ModifyDBInstanceCommand,
  RDSClient,
} from "@aws-sdk/client-rds";

const rds = new RDSClient({ region: "us-east-1" });

await rds.send(
  new ModifyDBInstanceCommand({
    DBInstanceIdentifier: "app-db-1",
    BackupRetentionPeriod: 7,
    ApplyImmediately: true,
  }),
);
```

### Delete a DB instance

Choose either `SkipFinalSnapshot: true` or provide `FinalDBSnapshotIdentifier` when your workflow requires a final backup.

```javascript
import {
  DeleteDBInstanceCommand,
  RDSClient,
} from "@aws-sdk/client-rds";

const rds = new RDSClient({ region: "us-east-1" });

await rds.send(
  new DeleteDBInstanceCommand({
    DBInstanceIdentifier: "app-db-1",
    SkipFinalSnapshot: true,
  }),
);
```

### Tag an RDS resource by ARN

RDS tagging operations target the resource ARN rather than a short identifier.

```javascript
import {
  AddTagsToResourceCommand,
  RDSClient,
} from "@aws-sdk/client-rds";

const rds = new RDSClient({ region: "us-east-1" });

await rds.send(
  new AddTagsToResourceCommand({
    ResourceName:
      "arn:aws:rds:us-east-1:123456789012:db:app-db-1",
    Tags: [
      { Key: "Environment", Value: "prod" },
      { Key: "ManagedBy", Value: "automation" },
    ],
  }),
);
```

## RDS-Specific Gotchas

- `@aws-sdk/client-rds` is a management client. Use `pg`, `mysql2`, `mssql`, or another database driver for actual SQL connections.
- Aurora and some newer RDS topologies expose important state at the cluster layer. Use `DescribeDBClusters` when instance-level calls are not enough.
- Many create, modify, restore, failover, and delete operations are asynchronous. Do not assume the endpoint is ready immediately after the API call returns.
- `instance.Endpoint` or cluster endpoints can be absent while provisioning, restoring, or deleting.
- `DeleteDBInstance` often requires an explicit final-snapshot decision. Make that choice intentionally in automation.
- IAM database authentication token generation belongs to `@aws-sdk/rds-signer`, not `@aws-sdk/client-rds`.
- Aurora Serverless Data API calls use `@aws-sdk/client-rds-data`, not `@aws-sdk/client-rds`.
- Import from the package root only. Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `pg`, `mysql2`, `mssql`, or another engine-specific driver: connect to the database endpoint and run SQL.
- `@aws-sdk/rds-signer`: generate IAM auth tokens for passwordless database login.
- `@aws-sdk/client-rds-data`: call the Aurora Data API instead of opening a database socket.
- `@aws-sdk/credential-providers`: use `fromIni`, IAM Identity Center, Cognito, STS assume-role, and other explicit credential helpers.

## Practical Notes For Agents

- Pick the API family that matches the resource model: instance commands for DB instances, cluster commands for Aurora or DB cluster resources.
- Log both identifiers and status fields during automation. RDS workflows are long-running and easier to debug when you capture `DBInstanceIdentifier`, `DBClusterIdentifier`, and current state.
- For idempotent automation, read current state with `Describe*` first and branch on whether the resource already exists.
- Treat endpoint discovery as a separate step from resource creation. The endpoint may not be available until the resource is fully ready.

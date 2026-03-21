---
name: redshift
description: "AWS SDK for JavaScript v3 Redshift client for provisioning and managing Amazon Redshift clusters and snapshots."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,redshift,javascript,nodejs,browser,data-warehouse,analytics"
---

# `@aws-sdk/client-redshift`

Use this package for Amazon Redshift control-plane APIs in AWS SDK for JavaScript v3. Typical tasks include describing clusters, creating or deleting clusters, managing snapshots, and changing cluster configuration.

Prefer `RedshiftClient` plus explicit command imports. The package also exposes an aggregated `Redshift` client, but command-based imports are the safer default for smaller bundles and clearer dependencies.

## Install

```bash
npm install @aws-sdk/client-redshift
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Initialize the client

```javascript
import { RedshiftClient } from "@aws-sdk/client-redshift";

const redshift = new RedshiftClient({
  region: "us-east-1",
});
```

In Node.js, the default credential provider chain is usually enough if your AWS access is already configured.

## Credentials and Region

- Node.js: credentials often come from environment variables, shared AWS config files, IAM roles, ECS task roles, or IAM Identity Center.
- Browser runtimes: use explicit browser-safe credentials only. In practice, Redshift administration from the browser is uncommon and is usually better handled on a backend.
- Region is required somewhere: set it in the client constructor, via `AWS_REGION`, or through shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## What This Client Does

`@aws-sdk/client-redshift` manages Redshift resources. It does not run SQL queries against your warehouse.

Use this client for:

- cluster lifecycle operations
- snapshots and restores
- parameter groups, subnet groups, and IAM-role attachments
- inventory and status checks

Do not use this client for:

- running SQL statements
- fetching query results
- opening PostgreSQL-compatible database connections

For those workflows, use `@aws-sdk/client-redshift-data` or a PostgreSQL-compatible driver such as `pg`, depending on whether you want the Redshift Data API or a direct database connection.

## Core Usage Pattern

`DescribeClusters` is a common read path when you need cluster status, endpoint details, or inventory.

```javascript
import {
  DescribeClustersCommand,
  RedshiftClient,
} from "@aws-sdk/client-redshift";

const redshift = new RedshiftClient({ region: "us-east-1" });

const response = await redshift.send(
  new DescribeClustersCommand({
    ClusterIdentifier: "analytics-prod",
  }),
);

const cluster = response.Clusters?.[0];

console.log(cluster?.ClusterIdentifier);
console.log(cluster?.ClusterStatus);
console.log(cluster?.Endpoint?.Address);
console.log(cluster?.Endpoint?.Port);
```

## Common Workflow

For many automation tasks, the flow is:

1. Create or modify a cluster.
2. Poll `DescribeClusters` until the cluster reaches the state you need, such as `available`.
3. Read the endpoint from `cluster.Endpoint`.
4. Run SQL through the Redshift Data API or a PostgreSQL-compatible client.

Most Redshift mutations are asynchronous. A successful command usually means the request was accepted, not that the cluster is immediately ready for the next step.

## Common Redshift Operations

### Describe a cluster

```javascript
import {
  DescribeClustersCommand,
  RedshiftClient,
} from "@aws-sdk/client-redshift";

const redshift = new RedshiftClient({ region: "us-east-1" });

const response = await redshift.send(
  new DescribeClustersCommand({
    ClusterIdentifier: "analytics-prod",
  }),
);

const cluster = response.Clusters?.[0];

if (!cluster) {
  throw new Error("Cluster not found");
}

console.log({
  status: cluster.ClusterStatus,
  nodeType: cluster.NodeType,
  dbName: cluster.DBName,
  endpoint: cluster.Endpoint,
});
```

### List clusters across pages

Redshift list-style APIs use marker-based pagination. Continue until the service stops returning `Marker`.

```javascript
import {
  DescribeClustersCommand,
  RedshiftClient,
} from "@aws-sdk/client-redshift";

const redshift = new RedshiftClient({ region: "us-east-1" });

let marker;

do {
  const page = await redshift.send(
    new DescribeClustersCommand({
      Marker: marker,
      MaxRecords: 100,
    }),
  );

  for (const cluster of page.Clusters ?? []) {
    console.log(cluster.ClusterIdentifier, cluster.ClusterStatus);
  }

  marker = page.Marker;
} while (marker);
```

### Create a manual snapshot

```javascript
import {
  CreateClusterSnapshotCommand,
  RedshiftClient,
} from "@aws-sdk/client-redshift";

const redshift = new RedshiftClient({ region: "us-east-1" });

const response = await redshift.send(
  new CreateClusterSnapshotCommand({
    ClusterIdentifier: "analytics-prod",
    SnapshotIdentifier: "analytics-prod-manual-20260311",
  }),
);

console.log(response.Snapshot?.SnapshotIdentifier);
console.log(response.Snapshot?.Status);
```

### Delete a cluster and keep a final snapshot

If you do not want to lose the last cluster state, provide a final snapshot identifier.

```javascript
import {
  DeleteClusterCommand,
  RedshiftClient,
} from "@aws-sdk/client-redshift";

const redshift = new RedshiftClient({ region: "us-east-1" });

await redshift.send(
  new DeleteClusterCommand({
    ClusterIdentifier: "analytics-dev",
    SkipFinalClusterSnapshot: false,
    FinalClusterSnapshotIdentifier: "analytics-dev-final-20260311",
  }),
);
```

If you are intentionally deleting an ephemeral environment, you can set `SkipFinalClusterSnapshot: true` instead.

## Redshift-Specific Gotchas

- This package manages Redshift resources; it does not execute SQL. For SQL, use `@aws-sdk/client-redshift-data` or a direct database driver.
- Cluster operations are asynchronous. Poll `DescribeClusters` after create, restore, resize, pause, resume, or delete requests before assuming the next step can run.
- The cluster endpoint is not reliably usable until the cluster is fully available.
- `ClusterIdentifier` is the control-plane identifier, not the database name. Do not confuse it with `DBName` or the endpoint hostname.
- `DeleteCluster` requires either `SkipFinalClusterSnapshot: true` or a valid `FinalClusterSnapshotIdentifier` when you want to preserve the last state.
- Provisioned Redshift and Redshift Serverless are different surfaces. Use the serverless client for workgroups and namespaces.
- Browser-based admin flows are unusual; keep cluster-management calls on the server side unless you have a very deliberate IAM and credential design.
- Do not deep-import SDK internals from build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-redshift-data`: run SQL statements through the Redshift Data API.
- `@aws-sdk/client-redshift-serverless`: manage Redshift Serverless workgroups and namespaces instead of provisioned clusters.
- `pg`: connect directly to a Redshift endpoint using the PostgreSQL wire protocol.
- `@aws-sdk/credential-providers`: Cognito, STS assume-role flows, shared config helpers, and other credential helpers.

## Common Decision Point

Choose the package based on the job you need to do:

- provisioning, snapshots, IAM-role attachment, and cluster configuration: `@aws-sdk/client-redshift`
- submitting SQL without managing a raw TCP connection: `@aws-sdk/client-redshift-data`
- direct SQL connections over the PostgreSQL protocol: `pg`
- serverless namespace and workgroup administration: `@aws-sdk/client-redshift-serverless`

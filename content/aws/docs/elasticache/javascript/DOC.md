---
name: elasticache
description: "AWS SDK for JavaScript v3 client for Amazon ElastiCache control-plane operations."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,elasticache,javascript,nodejs,redis,memcached,valkey"
---

# `@aws-sdk/client-elasticache`

Use this package to manage Amazon ElastiCache infrastructure from JavaScript or TypeScript: cache clusters, replication groups, serverless caches, snapshots, subnet groups, parameter groups, service updates, and tags.

## What This Package Is For

- Control-plane operations only: create, modify, describe, tag, snapshot, and delete ElastiCache resources.
- Use it for Memcached clusters, Redis OSS or Valkey replication groups, and serverless cache administration.
- Do not use it for application cache reads and writes. Your app should connect to the cache endpoint with a Redis, Valkey, or Memcached protocol client.

## Install

```bash
npm install @aws-sdk/client-elasticache
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

Prefer `ElastiCacheClient` plus explicit command imports. The package also exposes an aggregated `ElastiCache` client, but command-based imports are the safer default for smaller bundles and clearer dependencies.

## Initialize the Client

### Minimal Node.js client

```javascript
import { ElastiCacheClient } from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { ElastiCacheClient } from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

## Credentials and Region

- In Node.js, the SDK can use the default credential provider chain, so environment variables, shared AWS config, ECS, EC2 instance metadata, or IAM Identity Center are often enough.
- Browser use requires explicit browser-safe credentials and is uncommon for ElastiCache administration flows.
- Set the region in the client constructor, through `AWS_REGION`, or through shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

```javascript
import {
  DescribeReplicationGroupsCommand,
  ElastiCacheClient,
} from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({ region: "us-east-1" });

const response = await elasticache.send(
  new DescribeReplicationGroupsCommand({
    ReplicationGroupId: "orders-cache",
  }),
);

const group = response.ReplicationGroups?.[0];

console.log(group?.Status);
console.log(
  group?.ConfigurationEndpoint?.Address ??
    group?.NodeGroups?.[0]?.PrimaryEndpoint?.Address,
);
```

## Common Operations

### Describe a cache cluster and inspect endpoints

Use `ShowCacheNodeInfo: true` when you need per-node connection data.

```javascript
import {
  DescribeCacheClustersCommand,
  ElastiCacheClient,
} from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({ region: "us-east-1" });

const response = await elasticache.send(
  new DescribeCacheClustersCommand({
    CacheClusterId: "session-cache",
    ShowCacheNodeInfo: true,
  }),
);

const cluster = response.CacheClusters?.[0];

console.log(cluster?.CacheClusterStatus);
console.log(cluster?.ConfigurationEndpoint?.Address);

for (const node of cluster?.CacheNodes ?? []) {
  console.log(node.CacheNodeId, node.Endpoint?.Address, node.Endpoint?.Port);
}
```

### Create a Redis OSS or Valkey replication group

Use replication groups for replicated Redis OSS or Valkey topologies and failover management.

```javascript
import {
  CreateReplicationGroupCommand,
  ElastiCacheClient,
} from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({ region: "us-east-1" });

const replicationGroupId = "orders-cache";

await elasticache.send(
  new CreateReplicationGroupCommand({
    ReplicationGroupId: replicationGroupId,
    ReplicationGroupDescription: "Cache for the orders service",
    Engine: "redis",
    CacheNodeType: process.env.ELASTICACHE_NODE_TYPE ?? "cache.r6g.large",
    NumCacheClusters: 2,
    AutomaticFailoverEnabled: true,
    TransitEncryptionEnabled: true,
    AtRestEncryptionEnabled: true,
    SnapshotRetentionLimit: 7,
  }),
);
```

### Wait until a replication group is available

Create, modify, and delete operations are asynchronous. Waiters are useful when your automation needs the resource to become usable before continuing.

```javascript
import {
  ElastiCacheClient,
  waitUntilReplicationGroupAvailable,
} from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({ region: "us-east-1" });

await waitUntilReplicationGroupAvailable(
  {
    client: elasticache,
    maxWaitTime: 900,
  },
  {
    ReplicationGroupId: "orders-cache",
  },
);
```

### Create a Memcached cluster

`CreateCacheCluster` is the classic cluster API and is the normal entry point for Memcached.

```javascript
import {
  CreateCacheClusterCommand,
  ElastiCacheClient,
} from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({ region: "us-east-1" });

await elasticache.send(
  new CreateCacheClusterCommand({
    CacheClusterId: "session-cache",
    Engine: "memcached",
    CacheNodeType: process.env.ELASTICACHE_NODE_TYPE ?? "cache.t4g.small",
    NumCacheNodes: 2,
    AZMode: "cross-az",
    Port: 11211,
  }),
);
```

### Create a serverless cache

Serverless caches have their own request and pagination model and can optionally cap storage and ECPU usage.

```javascript
import {
  CreateServerlessCacheCommand,
  ElastiCacheClient,
} from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({ region: "us-east-1" });

await elasticache.send(
  new CreateServerlessCacheCommand({
    ServerlessCacheName: "api-session-cache",
    Description: "Serverless cache for API sessions",
    Engine: "redis",
    SecurityGroupIds: ["sg-0123456789abcdef0"],
    SubnetIds: ["subnet-0123456789abcdef0", "subnet-abcdef01234567890"],
    CacheUsageLimits: {
      DataStorage: {
        Maximum: 10,
        Unit: "GB",
      },
      ECPUPerSecond: {
        Maximum: 5000,
      },
    },
    SnapshotRetentionLimit: 7,
    DailySnapshotTime: "03:00",
  }),
);
```

### List replication groups with pagination

```javascript
import {
  ElastiCacheClient,
  paginateDescribeReplicationGroups,
} from "@aws-sdk/client-elasticache";

const elasticache = new ElastiCacheClient({ region: "us-east-1" });

for await (const page of paginateDescribeReplicationGroups(
  { client: elasticache },
  { MaxRecords: 50 },
)) {
  for (const group of page.ReplicationGroups ?? []) {
    console.log(group.ReplicationGroupId, group.Status);
  }
}
```

## ElastiCache-Specific Gotchas

- This package manages ElastiCache resources, but it does not speak the Redis, Valkey, or Memcached wire protocols.
- `DescribeCacheClusters` returns richer per-node endpoint data only when `ShowCacheNodeInfo` is true.
- Memcached clusters expose a `ConfigurationEndpoint`; replication groups may expose a configuration endpoint or per-node-group primary and reader endpoints depending on topology.
- `CreateCacheCluster` is commonly used for Memcached or single-node legacy flows. For replicated Redis OSS or Valkey deployments, prefer `CreateReplicationGroup`.
- Classic list operations such as `DescribeCacheClusters` and `DescribeReplicationGroups` paginate with `Marker` and `MaxRecords`; serverless list operations use `NextToken` and `MaxResults`.
- Generated waiters exist for cache clusters and replication groups, which is helpful because create, modify, and delete operations are asynchronous.
- `DeleteServerlessCache` can create a final snapshot through `FinalSnapshotName`, and that path requires snapshot permissions.
- Import from `@aws-sdk/client-elasticache` only. Do not deep-import package internals.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: Cognito identity, shared config helpers, assume-role flows, and other credential setup.
- A Redis, Valkey, or Memcached client library: actual application data access after ElastiCache returns the endpoint.

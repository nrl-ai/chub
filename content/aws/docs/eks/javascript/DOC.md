---
name: eks
description: "AWS SDK for JavaScript v3 client for managing Amazon EKS clusters, node groups, and add-ons"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,eks,kubernetes,containers,cluster"
---

# AWS EKS SDK for JavaScript (v3)

Use `@aws-sdk/client-eks` to manage Amazon EKS control-plane resources from modern JavaScript and TypeScript code.

This package talks to the Amazon EKS management API. It does not replace `kubectl` or a Kubernetes client for working with Kubernetes objects inside the cluster.

## Golden Rule

- Install `@aws-sdk/client-eks`, not the legacy `aws-sdk` v2 package.
- This doc covers package version `3.1006.0`.
- Prefer `EKSClient` plus individual command imports.
- Treat create and update APIs as asynchronous workflows. Read status fields and follow up with `DescribeUpdateCommand` or a fresh `Describe*` call before assuming the resource is ready.
- `DescribeClusterCommand` returns the API server endpoint and certificate authority data you need for kubeconfig generation, but those fields are only available after the cluster reaches `ACTIVE`.
- Set `region` explicitly in code or through standard AWS config.

## Install

```bash
npm install @aws-sdk/client-eks
```

Common companion packages:

```bash
# Shared config, Cognito, STS, and other credential helpers
npm install @aws-sdk/credential-providers

# Kubernetes API access after you have cluster connection details
npm install @kubernetes/client-node
```

## Initialize the client

```javascript
import { EKSClient } from "@aws-sdk/client-eks";

const eks = new EKSClient({
  region: "us-east-1",
});
```

For browser runtimes, do not assume the Node.js default credential chain exists. Use explicit browser-safe credentials and verify that your IAM and network setup intentionally allows direct calls to EKS APIs.

## Credentials and Region

In Node.js, the default credential provider chain is usually enough if you already use environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

`DescribeClusterCommand` is the usual starting point when you need the cluster endpoint, certificate authority data, version, or status.

```javascript
import {
  DescribeClusterCommand,
  EKSClient,
} from "@aws-sdk/client-eks";

const eks = new EKSClient({ region: "us-east-1" });

const response = await eks.send(
  new DescribeClusterCommand({
    name: "prod-cluster",
  }),
);

const cluster = response.cluster;

if (!cluster) {
  throw new Error("Cluster not found");
}

console.log(cluster.status);
console.log(cluster.endpoint);
console.log(cluster.certificateAuthority?.data);
console.log(cluster.version);
```

`certificateAuthority.data` is base64-encoded certificate data. Keep it as-is if you are writing kubeconfig, or decode it if a downstream tool expects PEM text.

## Common Operations

### List clusters with pagination

```javascript
import {
  EKSClient,
  paginateListClusters,
} from "@aws-sdk/client-eks";

const eks = new EKSClient({ region: "us-east-1" });

for await (const page of paginateListClusters({ client: eks }, {})) {
  for (const clusterName of page.clusters ?? []) {
    console.log(clusterName);
  }
}
```

### Read installed add-ons for a cluster

```javascript
import {
  EKSClient,
  ListAddonsCommand,
} from "@aws-sdk/client-eks";

const eks = new EKSClient({ region: "us-east-1" });

const response = await eks.send(
  new ListAddonsCommand({
    clusterName: "prod-cluster",
  }),
);

console.log(response.addons ?? []);
```

## EKS-Specific Gotchas

- This client manages EKS resources such as clusters, managed node groups, add-ons, access entries, and updates. It does not manage Kubernetes resources like `Deployment` or `Service` objects.
- `DescribeClusterCommand` uses `{ name }`, while many other operations use `{ clusterName }`. Do not assume the input field names are identical across commands.
- The cluster endpoint and certificate authority are not available until the cluster is `ACTIVE`.
- Managed node group and add-on updates commonly return an update record first and complete later. Track the returned update ID with `DescribeUpdateCommand`.
- Add-on compatibility depends on the cluster's Kubernetes version. Use `DescribeAddonVersionsCommand` instead of hard-coding add-on versions.
- Cluster creation requires existing VPC subnets, security groups, and an IAM role ARN. This package does not provision those prerequisites for you.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: shared config helpers, Cognito, and STS-based credential flows.
- `@kubernetes/client-node`: Kubernetes API calls after you already have cluster endpoint, certificate authority data, and authentication.
- `@aws-sdk/client-iam`: create or inspect the IAM roles that EKS clusters and node groups need.
- `@aws-sdk/client-ec2`: inspect or provision the VPC, subnet, and security-group resources that EKS depends on.

## Common EKS Operations

### List managed node groups in a cluster

```javascript
import {
  EKSClient,
  ListNodegroupsCommand,
} from "@aws-sdk/client-eks";

const eks = new EKSClient({ region: "us-east-1" });

const response = await eks.send(
  new ListNodegroupsCommand({
    clusterName: "prod-cluster",
  }),
);

for (const nodegroupName of response.nodegroups ?? []) {
  console.log(nodegroupName);
}
```

Use `paginateListNodegroups` if you expect enough node groups to span multiple pages.

### Read add-on versions compatible with a Kubernetes version

```javascript
import {
  DescribeAddonVersionsCommand,
  EKSClient,
} from "@aws-sdk/client-eks";

const eks = new EKSClient({ region: "us-east-1" });

const response = await eks.send(
  new DescribeAddonVersionsCommand({
    addonName: "vpc-cni",
    kubernetesVersion: "1.31",
  }),
);

for (const addon of response.addons ?? []) {
  console.log(addon.addonName);

  for (const version of addon.addonVersions ?? []) {
    console.log(version.addonVersion, version.compatibilities);
  }
}
```

Use this before pinning an add-on version during upgrades.

### Update a managed node group's scaling config and labels

`UpdateNodegroupConfigCommand` starts an asynchronous update. The initial response includes an update record rather than fully updated node group state.

```javascript
import {
  EKSClient,
  UpdateNodegroupConfigCommand,
} from "@aws-sdk/client-eks";

const eks = new EKSClient({ region: "us-east-1" });

const response = await eks.send(
  new UpdateNodegroupConfigCommand({
    clusterName: "prod-cluster",
    nodegroupName: "workers-a",
    scalingConfig: {
      minSize: 2,
      maxSize: 6,
      desiredSize: 3,
    },
    labels: {
      addOrUpdateLabels: {
        workload: "general",
      },
    },
  }),
);

console.log(response.update?.id, response.update?.status);
```

### Poll an update until it finishes

```javascript
import {
  DescribeUpdateCommand,
  EKSClient,
} from "@aws-sdk/client-eks";

const eks = new EKSClient({ region: "us-east-1" });

async function waitForNodegroupUpdate({
  clusterName,
  nodegroupName,
  updateId,
}) {
  while (true) {
    const response = await eks.send(
      new DescribeUpdateCommand({
        name: clusterName,
        nodegroupName,
        updateId,
      }),
    );

    const status = response.update?.status;

    if (status === "Successful") {
      return response.update;
    }

    if (status === "Failed" || status === "Cancelled") {
      throw new Error(`EKS update failed with status: ${status}`);
    }

    await new Promise((resolve) => setTimeout(resolve, 15000));
  }
}
```

### Notes

- Many EKS operations are cluster-scoped, so keep `clusterName` available in your calling code instead of passing only ARNs around.
- Update APIs are eventually consistent. Re-read the cluster, node group, or add-on after a successful update before depending on new fields.
- EKS cluster creation needs existing VPC networking and IAM prerequisites; application code often combines this package with infrastructure tooling rather than creating clusters ad hoc.

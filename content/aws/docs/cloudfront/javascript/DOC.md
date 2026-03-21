---
name: cloudfront
description: "AWS SDK for JavaScript v3 CloudFront client for distributions, invalidations, and edge configuration APIs."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,cloudfront,javascript,nodejs,browser,cdn"
---

# `@aws-sdk/client-cloudfront`

Use this package for Amazon CloudFront control-plane APIs in AWS SDK for JavaScript v3: distributions, invalidations, cache policies, origin request policies, response headers policies, and related edge configuration resources.

## Install

```bash
npm install @aws-sdk/client-cloudfront
```

Prefer `CloudFrontClient` plus explicit command imports. The package also exposes an aggregated `CloudFront` client, but command-based imports are the safer default for smaller bundles and clearer dependency boundaries.

## Initialize the client

```javascript
import { CloudFrontClient } from "@aws-sdk/client-cloudfront";

const cloudfront = new CloudFrontClient({
  region: "us-east-1",
});
```

CloudFront is a global service, but the SDK still needs a region for configuration and request signing. In practice, JavaScript SDK examples commonly use `us-east-1` for CloudFront control-plane calls.

## Credentials and Region

- Node.js: the default credential provider chain usually works if AWS access is already configured through environment variables, shared config files, ECS, EC2, or IAM Identity Center.
- Browser runtimes: use an explicit credential provider such as Cognito identity; do not ship privileged CloudFront management credentials to the browser.
- Keep region setup simple and consistent. For CloudFront control-plane code, prefer `us-east-1` unless you have an explicit endpoint strategy.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

The v3 SDK uses client-plus-command calls:

```javascript
import {
  CloudFrontClient,
  ListDistributionsCommand,
} from "@aws-sdk/client-cloudfront";

const cloudfront = new CloudFrontClient({ region: "us-east-1" });

const response = await cloudfront.send(
  new ListDistributionsCommand({ MaxItems: 25 }),
);

for (const distribution of response.DistributionList?.Items ?? []) {
  console.log(distribution.Id, distribution.DomainName, distribution.Enabled);
}
```

## CloudFront-Specific Gotchas

- CloudFront configuration changes are asynchronous. A successful create or update call does not mean the new behavior is already deployed at edge locations.
- `UpdateDistribution` is a full-config replace flow. Read the current config first, keep required fields, and send `IfMatch` with the current `ETag`.
- `CreateInvalidation` requires a unique `CallerReference`. Reusing one can return an existing invalidation instead of creating a new request.
- Delete flows are multi-step: disable the distribution, wait for deployment, then delete with the latest `ETag`.
- Signed URLs and signed cookies are not the main job of this client. Use `@aws-sdk/cloudfront-signer` for signing helpers.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/cloudfront-signer`: create signed URLs and signed cookies for private content.
- `@aws-sdk/credential-providers`: Cognito, STS assume-role, shared config, and other credential helpers.

## Common CloudFront operations

### List distributions

```javascript
import {
  CloudFrontClient,
  ListDistributionsCommand,
} from "@aws-sdk/client-cloudfront";

const cloudfront = new CloudFrontClient({ region: "us-east-1" });

const { DistributionList } = await cloudfront.send(
  new ListDistributionsCommand({ MaxItems: 25 }),
);

for (const distribution of DistributionList?.Items ?? []) {
  console.log({
    id: distribution.Id,
    domainName: distribution.DomainName,
    enabled: distribution.Enabled,
    status: distribution.Status,
  });
}
```

### Create an invalidation

```javascript
import {
  CloudFrontClient,
  CreateInvalidationCommand,
} from "@aws-sdk/client-cloudfront";

const cloudfront = new CloudFrontClient({ region: "us-east-1" });

const response = await cloudfront.send(
  new CreateInvalidationCommand({
    DistributionId: "E1234567890ABC",
    InvalidationBatch: {
      CallerReference: `deploy-${Date.now()}`,
      Paths: {
        Quantity: 2,
        Items: ["/index.html", "/assets/*"],
      },
    },
  }),
);

console.log(response.Invalidation?.Id, response.Invalidation?.Status);
```

Keep `CallerReference` unique for each invalidation request you intend to create.

### Update a distribution

CloudFront updates use a read-modify-write flow. Fetch the full config and current `ETag` first, then send the updated config back with `IfMatch`.

```javascript
import {
  CloudFrontClient,
  GetDistributionConfigCommand,
  UpdateDistributionCommand,
} from "@aws-sdk/client-cloudfront";

const cloudfront = new CloudFrontClient({ region: "us-east-1" });
const id = "E1234567890ABC";

const { DistributionConfig, ETag } = await cloudfront.send(
  new GetDistributionConfigCommand({ Id: id }),
);

if (!DistributionConfig || !ETag) {
  throw new Error("Distribution config lookup failed.");
}

const response = await cloudfront.send(
  new UpdateDistributionCommand({
    Id: id,
    IfMatch: ETag,
    DistributionConfig: {
      ...DistributionConfig,
      Comment: "Updated from AWS SDK for JavaScript v3",
    },
  }),
);

console.log(response.Distribution?.Id, response.ETag);
```

If another process updates the distribution first, the `ETag` changes and your update can fail until you refetch the latest config.

### Disable before delete

CloudFront does not let you delete an enabled distribution. The usual sequence is:

1. Read the current config and `ETag`.
2. Update the distribution with `Enabled: false`.
3. Wait until the distribution finishes deploying.
4. Delete it with the newest `ETag`.

Treat deletes as an operational workflow, not a single call.

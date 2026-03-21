---
name: opensearch
description: "AWS SDK for JavaScript v3 OpenSearch client for Amazon OpenSearch Service domain administration and configuration."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,opensearch,javascript,nodejs,browser,search,managed-service,control-plane"
---

# `@aws-sdk/client-opensearch`

Use this package for the control-plane API of Amazon OpenSearch Service in AWS SDK for JavaScript v3. It is the client for creating, describing, updating, tagging, and deleting managed OpenSearch domains and related service resources.

This package is not the OpenSearch data-plane client for index, document, or query requests against a domain endpoint. Use it to manage the service itself, not to run searches.

## Install

```bash
npm install @aws-sdk/client-opensearch
```

Prefer `OpenSearchClient` plus explicit command imports. Like other AWS SDK v3 service packages, it also exposes an aggregated compatibility client, but the client-and-command style is the clearer default.

## Version And Runtime Notes

- This doc covers package version `3.1006.0`.
- Current AWS SDK for JavaScript v3 releases at and above `3.968.0` should be treated as Node.js 20+ in Node runtimes.
- The SDK also supports browser-oriented bundles, but OpenSearch domain administration typically belongs on trusted backend code rather than in browser applications.

## Client Setup

```javascript
import { OpenSearchClient } from "@aws-sdk/client-opensearch";

const opensearch = new OpenSearchClient({ region: "us-east-1" });
```

In Node.js, region plus the default credential provider chain is usually enough when credentials already come from environment variables, shared AWS config, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Control Plane Vs Data Plane

Use `@aws-sdk/client-opensearch` when you need to:

- create or delete domains
- inspect domain status and configuration
- update cluster, storage, networking, or security settings
- manage tags and other service-level metadata

Do not use this package for:

- indexing documents
- running search queries
- calling `/_search`, `/_bulk`, or other OpenSearch REST endpoints directly

Those data-plane requests go to the domain endpoint itself and usually use an OpenSearch HTTP client with AWS SigV4 signing.

## Core Usage Pattern

```javascript
import {
  DescribeDomainCommand,
  OpenSearchClient,
} from "@aws-sdk/client-opensearch";

const opensearch = new OpenSearchClient({ region: "us-east-1" });

const response = await opensearch.send(
  new DescribeDomainCommand({
    DomainName: "search-prod",
  }),
);

console.log(response.DomainStatus?.ARN);
console.log(response.DomainStatus?.Endpoint);
```

The standard v3 pattern is the same as other AWS packages: create a client once, then call `client.send(new SomeCommand(input))`.

## Common Operations

### List domains in an account and region

```javascript
import {
  ListDomainNamesCommand,
  OpenSearchClient,
} from "@aws-sdk/client-opensearch";

const opensearch = new OpenSearchClient({ region: "us-east-1" });

const response = await opensearch.send(
  new ListDomainNamesCommand({}),
);

for (const domain of response.DomainNames ?? []) {
  console.log(domain.DomainName);
}
```

This is the quickest way to discover the managed OpenSearch domains visible to the current credentials in the configured region.

### Inspect the current configuration for one domain

```javascript
import {
  DescribeDomainConfigCommand,
  OpenSearchClient,
} from "@aws-sdk/client-opensearch";

const opensearch = new OpenSearchClient({ region: "us-east-1" });

const response = await opensearch.send(
  new DescribeDomainConfigCommand({
    DomainName: "search-prod",
  }),
);

console.log(response.DomainConfig?.ClusterConfig);
console.log(response.DomainConfig?.EBSOptions);
```

Use `DescribeDomainConfig` when you need the configurable settings rather than only the summary status returned by `DescribeDomain`.

### Add tags to a domain

```javascript
import {
  AddTagsCommand,
  OpenSearchClient,
} from "@aws-sdk/client-opensearch";

const opensearch = new OpenSearchClient({ region: "us-east-1" });

await opensearch.send(
  new AddTagsCommand({
    ARN: "arn:aws:es:us-east-1:123456789012:domain/search-prod",
    TagList: [
      { Key: "Environment", Value: "prod" },
      { Key: "Owner", Value: "search-team" },
    ],
  }),
);
```

The tag APIs use the domain ARN, not the domain name. In practice, fetch the ARN from a prior `DescribeDomain` call or from infrastructure metadata.

### Delete a domain

```javascript
import {
  DeleteDomainCommand,
  OpenSearchClient,
} from "@aws-sdk/client-opensearch";

const opensearch = new OpenSearchClient({ region: "us-east-1" });

const response = await opensearch.send(
  new DeleteDomainCommand({
    DomainName: "search-dev",
  }),
);

console.log(response.DomainStatus?.Deleted);
```

Deletion is asynchronous. A successful API call means the service accepted the delete request, not that the domain and its endpoint have disappeared immediately.

## OpenSearch Service Gotchas

- This package manages Amazon OpenSearch Service resources. It does not replace an OpenSearch REST client for search and indexing traffic.
- Domain create, update, and delete workflows are asynchronous. Poll domain status before assuming configuration changes are active.
- Some domains expose VPC-only endpoints. The SDK can manage them from anywhere with AWS API access, but data-plane traffic still needs network access to the domain endpoint.
- OpenSearch Serverless uses a different service package: `@aws-sdk/client-opensearchserverless`.
- If your application needs document APIs, index templates, or raw query DSL calls, connect to the domain endpoint separately instead of trying to model those calls through this client.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, Cognito, and assume-role credential helpers.
- `@aws-sdk/client-opensearchserverless`: OpenSearch Serverless collections, policies, and related serverless control-plane operations.
- `@opensearch-project/opensearch`: data-plane indexing and search requests sent to an OpenSearch endpoint, typically combined with AWS SigV4 request signing.

## Practical Use Pattern

For most applications, the workflow is split across two layers:

1. Use `@aws-sdk/client-opensearch` in deployment or admin code to provision and configure the managed domain.
2. Use the domain endpoint with an OpenSearch client for runtime search and indexing traffic.

Keeping those roles separate avoids a common mistake: trying to use the AWS control-plane client as if it were the OpenSearch query client.

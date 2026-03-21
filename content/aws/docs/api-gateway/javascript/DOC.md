---
name: api-gateway
description: "AWS SDK for JavaScript v3 client for managing Amazon API Gateway REST APIs, resources, methods, deployments, stages, usage plans, and domains."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,api-gateway,javascript,nodejs,rest-api,openapi"
---

# `@aws-sdk/client-api-gateway`

Use this package for the API Gateway **REST API control plane** in AWS SDK for JavaScript v3. It manages APIs, resources, methods, integrations, deployments, stages, usage plans, API keys, and custom domains.

This package does **not** invoke your deployed API for application traffic. It is for provisioning and admin workflows.

## Install

```bash
npm install @aws-sdk/client-api-gateway
```

Prefer `APIGatewayClient` plus explicit command imports. The aggregated `APIGateway` client exists, but command-based imports are the safer default for clearer code and smaller bundles.

## Initialize the client

```javascript
import { APIGatewayClient } from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({
  region: "us-east-1",
});
```

In Node.js, the SDK can usually resolve credentials from the default credential provider chain, so setting the region is often enough.

## Credentials and Region

- Node.js: credentials usually come from environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.
- Browser runtimes: use explicit browser-safe credentials only. In practice, this control-plane client is usually better kept on the server because it manages high-privilege infrastructure.
- Region is required somewhere: pass it in code, set `AWS_REGION`, or configure it in shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## What This Client Covers

Use `@aws-sdk/client-api-gateway` for the original API Gateway REST API service:

- REST APIs and OpenAPI import/export
- resources, methods, and integrations
- deployments and stages
- usage plans and API keys
- custom domain names and base path mappings

If you need API Gateway HTTP APIs or WebSocket APIs, use the API Gateway v2 client instead of this package.

## Core Usage Pattern

The v3 SDK uses client-plus-command calls:

```javascript
import {
  APIGatewayClient,
  CreateRestApiCommand,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });

const response = await apiGateway.send(
  new CreateRestApiCommand({
    name: "orders-api",
    description: "REST API for order operations",
    endpointConfiguration: {
      types: ["REGIONAL"],
    },
  }),
);

console.log(response.id);
console.log(response.rootResourceId);
```

Most workflows follow the same sequence:

1. create or import a `RestApi`
2. create resources under the API root
3. attach methods and integrations
4. create a deployment
5. bind or update a stage

## Common Operations

### List REST APIs with pagination

```javascript
import {
  APIGatewayClient,
  paginateGetRestApis,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });

const paginator = paginateGetRestApis(
  { client: apiGateway },
  { limit: 25 },
);

for await (const page of paginator) {
  for (const api of page.items ?? []) {
    console.log(api.id, api.name);
  }
}
```

### Get resources for an API

```javascript
import {
  APIGatewayClient,
  GetResourcesCommand,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });

const { items = [] } = await apiGateway.send(
  new GetResourcesCommand({
    restApiId: "a1b2c3d4",
    limit: 500,
  }),
);

for (const resource of items) {
  console.log(resource.id, resource.path);
}
```

### Create a deployment

```javascript
import {
  APIGatewayClient,
  CreateDeploymentCommand,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });

const deployment = await apiGateway.send(
  new CreateDeploymentCommand({
    restApiId: "a1b2c3d4",
    stageName: "prod",
    description: "Deploy current API configuration",
  }),
);

console.log(deployment.id);
```

Passing `stageName` creates or updates that stage as part of the deployment flow.

## API Gateway REST API Gotchas

- This package manages **REST APIs**. Do not confuse it with the separate API Gateway v2 service for HTTP APIs and WebSocket APIs.
- Control-plane changes are not live until you create a deployment or point a stage at a new deployment.
- `PutMethod` and `PutIntegration` are separate steps. A method without an integration still will not serve backend traffic.
- `CreateDeployment` can create or update a stage when you pass `stageName`; otherwise deploy first and create a stage separately.
- `PutRestApi` can merge or overwrite an existing API definition. Be explicit about update mode during OpenAPI-based workflows.
- `Update*` operations use patch operations. Treat them like targeted infrastructure mutations rather than partial JSON merges.
- Custom domain names, usage plans, and API keys are in this client too, so you do not need separate service packages for those control-plane APIs.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-apigatewayv2`: API Gateway HTTP APIs and WebSocket APIs.
- `@aws-sdk/credential-providers`: Cognito, IAM Identity Center, shared config, assume-role flows, and other credential helpers.
- `@aws-sdk/signature-v4` or higher-level HTTP clients: signed runtime calls if you are invoking an API rather than administering it.

## Common API Gateway REST API Operations

### Import an API from an OpenAPI file

Use `ImportRestApiCommand` when the API definition already exists as OpenAPI or Swagger.

```javascript
import { readFile } from "node:fs/promises";
import {
  APIGatewayClient,
  ImportRestApiCommand,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });
const body = await readFile("./openapi.json");

const response = await apiGateway.send(
  new ImportRestApiCommand({
    failOnWarnings: true,
    body,
    parameters: {
      endpointConfigurationTypes: "REGIONAL",
    },
  }),
);

console.log(response.id, response.name);
```

Use `PutRestApiCommand` later if you want to merge or overwrite an existing API with a revised definition.

### Create a child resource under the API root

`CreateRestApi` returns `rootResourceId`. Use that as the parent when adding new paths.

```javascript
import {
  APIGatewayClient,
  CreateResourceCommand,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });

const resource = await apiGateway.send(
  new CreateResourceCommand({
    restApiId: "a1b2c3d4",
    parentId: "xyz987",
    pathPart: "orders",
  }),
);

console.log(resource.id, resource.path);
```

### Attach a method and HTTP proxy integration

`PutMethod` defines the method shape on the resource. `PutIntegration` connects that method to a backend.

```javascript
import {
  APIGatewayClient,
  PutIntegrationCommand,
  PutMethodCommand,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });
const restApiId = "a1b2c3d4";
const resourceId = "r123456";

await apiGateway.send(
  new PutMethodCommand({
    restApiId,
    resourceId,
    httpMethod: "GET",
    authorizationType: "NONE",
  }),
);

await apiGateway.send(
  new PutIntegrationCommand({
    restApiId,
    resourceId,
    httpMethod: "GET",
    type: "HTTP_PROXY",
    integrationHttpMethod: "GET",
    uri: "https://example.com/orders",
  }),
);
```

If you use `AWS` or `AWS_PROXY` integrations, the `uri` must be the service-specific AWS integration URI, and the target service usually needs its own IAM permissions and resource policy setup.

### Deploy a specific API revision to a stage

If you do not pass `stageName` to `CreateDeployment`, create the stage separately.

```javascript
import {
  APIGatewayClient,
  CreateDeploymentCommand,
  CreateStageCommand,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });

const deployment = await apiGateway.send(
  new CreateDeploymentCommand({
    restApiId: "a1b2c3d4",
    description: "Deploy after method and integration updates",
  }),
);

await apiGateway.send(
  new CreateStageCommand({
    restApiId: "a1b2c3d4",
    deploymentId: deployment.id,
    stageName: "prod",
    tracingEnabled: true,
    variables: {
      LOG_LEVEL: "info",
    },
  }),
);
```

Use `UpdateStageCommand` when you only need to change stage settings or move the stage to a new deployment.

### Paginate all resources for a large API

Large REST APIs may require multiple `GetResources` calls.

```javascript
import {
  APIGatewayClient,
  paginateGetResources,
} from "@aws-sdk/client-api-gateway";

const apiGateway = new APIGatewayClient({ region: "us-east-1" });

const paginator = paginateGetResources(
  { client: apiGateway },
  {
    restApiId: "a1b2c3d4",
    limit: 500,
  },
);

for await (const page of paginator) {
  for (const resource of page.items ?? []) {
    console.log(resource.path);
  }
}
```

### Notes

- For OpenAPI-first workflows, `ImportRestApi` and `PutRestApi` are usually simpler than manually building every resource and method.
- For imperative workflows, store `restApiId`, `rootResourceId`, `resourceId`, `deployment.id`, and `stageName` as soon as they are created.
- Changes to resources, methods, and integrations do not affect live traffic until a deployment is created and a stage points at it.
- API Gateway resource trees can become large, so use the paginators rather than assuming one page is enough.

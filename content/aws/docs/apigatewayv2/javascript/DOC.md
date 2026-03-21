---
name: apigatewayv2
description: "AWS SDK for JavaScript v3 client for Amazon API Gateway V2 HTTP APIs and WebSocket API control-plane workflows."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,apigatewayv2,api-gateway,http-api,websocket,javascript,nodejs"
---

# `@aws-sdk/client-apigatewayv2`

Use this package for Amazon API Gateway V2 control-plane APIs in AWS SDK for JavaScript v3. It manages HTTP APIs, WebSocket APIs, routes, integrations, stages, deployments, authorizers, custom domains, and API mappings.

This package is not the runtime client for posting messages back to connected WebSocket clients. For that, use `@aws-sdk/client-apigatewaymanagementapi`.

## Install

```bash
npm install @aws-sdk/client-apigatewayv2
```

Prefer `ApiGatewayV2Client` plus explicit command imports. The package also exposes an aggregated `ApiGatewayV2` client, but command-based imports are the safer default for smaller bundles and clearer dependencies.

## Initialize the client

```javascript
import { ApiGatewayV2Client } from "@aws-sdk/client-apigatewayv2";

const apiGatewayV2 = new ApiGatewayV2Client({
  region: "us-east-1",
});
```

## Credentials and Region

- Node.js: the default AWS credential provider chain usually works if credentials are already configured through environment variables, shared AWS config, ECS, EC2, or IAM Identity Center.
- Browser runtimes: this is a control-plane client for infrastructure and admin workflows, so it usually belongs on the server side.
- Keep the region explicit somewhere: in code, `AWS_REGION`, or shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## What This Client Covers

Use `@aws-sdk/client-apigatewayv2` for:

- HTTP APIs
- WebSocket APIs
- routes and route responses
- integrations and integration responses
- stages and deployments
- JWT and Lambda authorizers
- custom domains and API mappings

If you need the original API Gateway REST API service, use `@aws-sdk/client-api-gateway` instead.

## Core Usage Pattern

The v3 SDK uses `client.send(new Command(input))`.

```javascript
import {
  ApiGatewayV2Client,
  CreateApiCommand,
} from "@aws-sdk/client-apigatewayv2";

const apiGatewayV2 = new ApiGatewayV2Client({ region: "us-east-1" });

const response = await apiGatewayV2.send(
  new CreateApiCommand({
    Name: "orders-http-api",
    ProtocolType: "HTTP",
  }),
);

console.log(response.ApiId, response.ApiEndpoint);
```

For most HTTP API workflows, the sequence is:

1. create the API
2. create one or more integrations
3. create routes that point at those integrations
4. create a stage or rely on auto-deploy

## Common Operations

### List APIs with pagination

```javascript
import {
  ApiGatewayV2Client,
  GetApisCommand,
} from "@aws-sdk/client-apigatewayv2";

const apiGatewayV2 = new ApiGatewayV2Client({ region: "us-east-1" });

let nextToken;

do {
  const response = await apiGatewayV2.send(
    new GetApisCommand({
      MaxResults: "50",
      NextToken: nextToken,
    }),
  );

  for (const api of response.Items ?? []) {
    console.log(api.ApiId, api.Name, api.ProtocolType);
  }

  nextToken = response.NextToken;
} while (nextToken);
```

### Create an HTTP proxy integration

```javascript
import {
  ApiGatewayV2Client,
  CreateIntegrationCommand,
} from "@aws-sdk/client-apigatewayv2";

const apiGatewayV2 = new ApiGatewayV2Client({ region: "us-east-1" });

const integration = await apiGatewayV2.send(
  new CreateIntegrationCommand({
    ApiId: "a1b2c3d4",
    IntegrationType: "HTTP_PROXY",
    IntegrationMethod: "ANY",
    IntegrationUri: "https://example.com/orders",
    PayloadFormatVersion: "1.0",
  }),
);

console.log(integration.IntegrationId);
```

For Lambda proxy integrations, use the Lambda invoke ARN as the integration URI and make sure the function also has the correct invoke permission.

### Create an HTTP route

```javascript
import {
  ApiGatewayV2Client,
  CreateRouteCommand,
} from "@aws-sdk/client-apigatewayv2";

const apiGatewayV2 = new ApiGatewayV2Client({ region: "us-east-1" });

await apiGatewayV2.send(
  new CreateRouteCommand({
    ApiId: "a1b2c3d4",
    RouteKey: "GET /orders",
    Target: "integrations/abc123",
  }),
);
```

For HTTP APIs, route keys look like `METHOD /path`. For WebSocket APIs, route keys look like `$connect`, `$disconnect`, `$default`, or an application-defined route key.

### Create a stage with auto-deploy

```javascript
import {
  ApiGatewayV2Client,
  CreateStageCommand,
} from "@aws-sdk/client-apigatewayv2";

const apiGatewayV2 = new ApiGatewayV2Client({ region: "us-east-1" });

await apiGatewayV2.send(
  new CreateStageCommand({
    ApiId: "a1b2c3d4",
    StageName: "$default",
    AutoDeploy: true,
  }),
);
```

For HTTP APIs, a `$default` stage with `AutoDeploy: true` is a common simple setup.

### Create a deployment when auto-deploy is off

```javascript
import {
  ApiGatewayV2Client,
  CreateDeploymentCommand,
} from "@aws-sdk/client-apigatewayv2";

const apiGatewayV2 = new ApiGatewayV2Client({ region: "us-east-1" });

const deployment = await apiGatewayV2.send(
  new CreateDeploymentCommand({
    ApiId: "a1b2c3d4",
    Description: "Promote tested routes and integrations",
  }),
);

console.log(deployment.DeploymentId);
```

If the stage is not auto-deploying, point the stage at the new deployment after this step.

## API Gateway V2 Gotchas

- `ProtocolType` is required and changes the whole workflow: `HTTP` and `WEBSOCKET` APIs use different route-key patterns and capabilities.
- This package manages infrastructure, not runtime traffic. It creates APIs and routes; it does not invoke your deployed endpoints for application calls.
- `@aws-sdk/client-api-gateway` is for the original REST API service. Do not mix the two packages.
- A route usually needs an integration first, because the route `Target` points at `integrations/<integrationId>`.
- `$default` has special meaning for both routes and stages. Use it intentionally rather than as a placeholder string.
- Auto-deploy changes whether you need explicit deployment management. Know which stage mode you are using before assuming a route change is already live.
- WebSocket callback messages are handled through the API Gateway Management API package, not this package.

## When To Reach For Other Packages

- `@aws-sdk/client-api-gateway`: the original API Gateway REST API service.
- `@aws-sdk/client-apigatewaymanagementapi`: post messages to connected WebSocket clients at runtime.
- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, Cognito, and assume-role flows.

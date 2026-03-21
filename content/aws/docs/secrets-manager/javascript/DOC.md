---
name: secrets-manager
description: "AWS SDK for JavaScript v3 client for AWS Secrets Manager in Node.js and browser applications"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,secrets-manager,secrets,security,javascript"
---

# AWS Secrets Manager SDK for JavaScript (v3)

Use `@aws-sdk/client-secrets-manager` to read, create, and manage AWS Secrets Manager secrets from modern JavaScript and TypeScript code.

## Golden Rule

- Install `@aws-sdk/client-secrets-manager`, not the legacy `aws-sdk` v2 package.
- The package version covered here is `3.1006.0`.
- AWS SDK for JavaScript v2 reached end of support on September 8, 2025.
- Current v3 releases at and above `3.968.0` require Node.js 20+, so `3.1006.0` should be treated as Node 20+.
- Prefer server-side secret access. Do not expose broad Secrets Manager reads to untrusted browser clients.
- Import from the package root only. Do not deep-import from `dist-*` paths.

## Install

```bash
npm install @aws-sdk/client-secrets-manager
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Client Setup

### Minimal Node.js client

```javascript
import { SecretsManagerClient } from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { SecretsManagerClient } from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

### Browser usage warning

The package can run in browser builds, but browser code should not directly retrieve application secrets for end users. If you must run it outside Node.js, use explicit federated credentials and a tightly scoped IAM policy, and prefer a backend API over direct secret reads.

## Credentials and Region

In Node.js, the default credential provider chain is usually enough if you have already configured AWS access through environment variables, shared config files, ECS, EC2 instance metadata, or IAM Identity Center.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

If you rely on shared AWS config instead of environment variables, keep the client initialization simple and set only the region in code.

## Core Usage Pattern

The v3 SDK uses client-plus-command calls:

```javascript
import {
  GetSecretValueCommand,
  SecretsManagerClient,
} from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({ region: "us-east-1" });

const response = await secrets.send(
  new GetSecretValueCommand({
    SecretId: "prod/my-app",
  }),
);

const secretValue =
  response.SecretString ?? new TextDecoder().decode(response.SecretBinary);

console.log(secretValue);
```

Prefer `SecretsManagerClient` plus explicit commands for most application code.

## Common Operations

### Read a JSON secret

Many teams store structured JSON inside `SecretString`.

```javascript
import {
  GetSecretValueCommand,
  SecretsManagerClient,
} from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({ region: "us-east-1" });

const { SecretString } = await secrets.send(
  new GetSecretValueCommand({ SecretId: "prod/my-app" }),
);

const config = JSON.parse(SecretString);
console.log(config.username);
```

### Read a specific version stage

`AWSCURRENT` is the default. `AWSPREVIOUS` is useful for rollback checks.

```javascript
import {
  GetSecretValueCommand,
  SecretsManagerClient,
} from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({ region: "us-east-1" });

const previous = await secrets.send(
  new GetSecretValueCommand({
    SecretId: "prod/my-app",
    VersionStage: "AWSPREVIOUS",
  }),
);

console.log(previous.SecretString);
```

### Create a new secret

```javascript
import {
  CreateSecretCommand,
  SecretsManagerClient,
} from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({ region: "us-east-1" });

await secrets.send(
  new CreateSecretCommand({
    Name: "prod/my-app",
    Description: "Application credentials for my-app",
    SecretString: JSON.stringify({
      username: "app-user",
      password: "replace-me",
    }),
  }),
);
```

### Put a new secret value version

Use `PutSecretValue` when you want a fresh secret version instead of editing only metadata.

```javascript
import {
  PutSecretValueCommand,
  SecretsManagerClient,
} from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({ region: "us-east-1" });

await secrets.send(
  new PutSecretValueCommand({
    SecretId: "prod/my-app",
    SecretString: JSON.stringify({
      username: "app-user",
      password: process.env.NEXT_PASSWORD,
    }),
  }),
);
```

### Inspect rotation and versions

```javascript
import {
  DescribeSecretCommand,
  SecretsManagerClient,
} from "@aws-sdk/client-secrets-manager";

const secrets = new SecretsManagerClient({ region: "us-east-1" });

const details = await secrets.send(
  new DescribeSecretCommand({ SecretId: "prod/my-app" }),
);

console.log(details.RotationEnabled);
console.log(details.VersionIdsToStages);
```

## Secrets Manager-Specific Gotchas

- `SecretId` can be a friendly name or an ARN. Use the full ARN for cross-account access or when names may be ambiguous.
- `GetSecretValue` returns either `SecretString` or `SecretBinary`. Handle both paths explicitly.
- Parse `SecretString` defensively. Many secrets contain JSON, but the service does not enforce a schema.
- `PutSecretValue` creates a new secret version. Use it for rotation or value updates; reserve `UpdateSecret` for metadata or configuration changes.
- Version stages matter: `AWSCURRENT` is the default active version, `AWSPREVIOUS` is useful for rollback, and `AWSPENDING` commonly appears during rotation workflows.
- Do not log raw secret contents, copy them into frontend bundles, or keep them in long-lived error objects.
- If a secret is read frequently, cache the parsed value in your application layer and refresh it intentionally when rotation happens.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, Cognito, STS assume-role flows, and other credential helpers.


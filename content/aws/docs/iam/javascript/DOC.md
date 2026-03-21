---
name: iam
description: "AWS SDK for JavaScript v3 IAM client for users, roles, policies, and access key management."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,iam,javascript,nodejs,identity,access-management,policies,roles"
---

# `@aws-sdk/client-iam`

Use this package for AWS Identity and Access Management in AWS SDK for JavaScript v3. It covers account-level IAM resources such as users, groups, roles, managed policy attachments, inline policies, login profiles, and access keys.

## Install

```bash
npm install @aws-sdk/client-iam
```

Prefer `IAMClient` plus explicit command imports. The package also exposes an aggregated `IAM` client, but command-based imports are the safer default for smaller bundles and clearer call sites.

## Client Setup

```javascript
import { IAMClient } from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });
```

IAM is a global AWS service, but the SDK still expects normal client configuration. In practice, set a region such as `us-east-1` or rely on your standard AWS region configuration.

## Credentials and Runtime Notes

- In Node.js, the default credential provider chain is usually enough if credentials already come from environment variables, shared AWS config, ECS, EC2 instance metadata, or IAM Identity Center.
- IAM administration usually belongs on trusted backends, not in browser code.
- Avoid long-lived IAM user access keys when temporary role credentials will work.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

```javascript
import {
  GetRoleCommand,
  IAMClient,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

const response = await iam.send(
  new GetRoleCommand({
    RoleName: "app-worker-role",
  }),
);

console.log(response.Role?.Arn);
```

## Common Operations

### Create a role with a trust policy

`AssumeRolePolicyDocument` must be a JSON string, not a plain JavaScript object.

```javascript
import {
  CreateRoleCommand,
  IAMClient,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

const trustPolicy = JSON.stringify({
  Version: "2012-10-17",
  Statement: [
    {
      Effect: "Allow",
      Principal: { Service: "lambda.amazonaws.com" },
      Action: "sts:AssumeRole",
    },
  ],
});

const { Role } = await iam.send(
  new CreateRoleCommand({
    RoleName: "app-worker-role",
    AssumeRolePolicyDocument: trustPolicy,
    Description: "Execution role for background workers",
  }),
);

console.log(Role?.Arn);
```

### Attach an AWS managed policy to a role

```javascript
import {
  AttachRolePolicyCommand,
  IAMClient,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

await iam.send(
  new AttachRolePolicyCommand({
    RoleName: "app-worker-role",
    PolicyArn:
      "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
  }),
);
```

### List users with pagination

```javascript
import {
  IAMClient,
  paginateListUsers,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

for await (const page of paginateListUsers({ client: iam }, {})) {
  for (const user of page.Users ?? []) {
    console.log(user.UserName, user.Arn);
  }
}
```

## IAM-Specific Gotchas

- Trust policies and inline policy documents are JSON strings on write.
- Some IAM read APIs return policy documents in URL-encoded form. Decode them before `JSON.parse`.
- IAM is eventually consistent for many create, update, and delete flows, so follow-up calls may need retry/backoff.
- Managed policy attachment APIs require a full `PolicyArn`, not just a policy name.
- A role's trust policy controls who can assume it; attached or inline permission policies control what it can do after assumption.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-sts`: assume roles, inspect caller identity, and obtain temporary credentials.
- `@aws-sdk/credential-providers`: `fromIni`, IAM Identity Center, web identity, and other credential helpers.

## Common IAM Operations

### Create a user

```javascript
import {
  CreateUserCommand,
  IAMClient,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

const { User } = await iam.send(
  new CreateUserCommand({
    UserName: "ci-bot",
    Path: "/automation/",
  }),
);

console.log(User?.Arn);
```

### Get and decode a role trust policy

Some IAM policy documents come back URL-encoded.

```javascript
import {
  GetRoleCommand,
  IAMClient,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

const { Role } = await iam.send(
  new GetRoleCommand({
    RoleName: "app-worker-role",
  }),
);

const trustPolicy = Role?.AssumeRolePolicyDocument
  ? JSON.parse(decodeURIComponent(Role.AssumeRolePolicyDocument))
  : null;

console.log(trustPolicy);
```

### Put an inline role policy

`PolicyDocument` is a JSON string.

```javascript
import {
  IAMClient,
  PutRolePolicyCommand,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

const policyDocument = JSON.stringify({
  Version: "2012-10-17",
  Statement: [
    {
      Effect: "Allow",
      Action: ["s3:GetObject"],
      Resource: ["arn:aws:s3:::example-bucket/*"],
    },
  ],
});

await iam.send(
  new PutRolePolicyCommand({
    RoleName: "app-worker-role",
    PolicyName: "read-example-bucket",
    PolicyDocument: policyDocument,
  }),
);
```

### List attached role policies

```javascript
import {
  IAMClient,
  ListAttachedRolePoliciesCommand,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

const { AttachedPolicies = [] } = await iam.send(
  new ListAttachedRolePoliciesCommand({
    RoleName: "app-worker-role",
  }),
);

for (const policy of AttachedPolicies) {
  console.log(policy.PolicyName, policy.PolicyArn);
}
```

### List roles with pagination

```javascript
import {
  IAMClient,
  paginateListRoles,
} from "@aws-sdk/client-iam";

const iam = new IAMClient({ region: "us-east-1" });

for await (const page of paginateListRoles({ client: iam }, { PathPrefix: "/" })) {
  for (const role of page.Roles ?? []) {
    console.log(role.RoleName, role.Arn);
  }
}
```

If you need stronger read-after-write behavior after `CreateRole`, retry `GetRole` or the dependent downstream call with backoff.

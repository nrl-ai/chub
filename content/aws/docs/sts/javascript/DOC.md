---
name: sts
description: "AWS SDK for JavaScript v3 STS client for temporary credentials, role assumption, and caller identity inspection."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,sts,javascript,nodejs,iam,credentials,assume-role,temporary-credentials"
---

# `@aws-sdk/client-sts`

Use this package for AWS Security Token Service operations in AWS SDK for JavaScript v3. The STS client is how JavaScript code inspects the current AWS identity, assumes IAM roles, requests temporary session credentials, and decodes some authorization failure details.

Common operations include `AssumeRole`, `AssumeRoleWithWebIdentity`, `AssumeRoleWithSAML`, `GetSessionToken`, `GetFederationToken`, `GetCallerIdentity`, and `DecodeAuthorizationMessage`.

## Install

```bash
npm install @aws-sdk/client-sts
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

Prefer `STSClient` plus explicit command imports. The package also exposes an aggregated `STS` client, but command-based imports are the safer default for clearer call sites and smaller bundles.

## Client Setup

```javascript
import { STSClient } from "@aws-sdk/client-sts";

const sts = new STSClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

STS historically has a global endpoint, but the SDK still expects normal client configuration. In practice, set a region explicitly and keep it consistent with the workloads that will use the temporary credentials you request.

## Credentials and Runtime Notes

- In Node.js, the default credential provider chain is usually enough if AWS access is already configured through environment variables, shared AWS config files, ECS, EC2 instance metadata, or IAM Identity Center.
- Most STS calls are made from trusted backend code that already has some base AWS credentials and needs short-lived credentials for a narrower task.
- In browser and mobile code, do not ship long-lived AWS keys. Prefer federation flows such as Cognito or web identity plus tightly scoped IAM roles.
- If your app needs automatic refresh of temporary credentials, prefer `@aws-sdk/credential-providers` over hand-rolling STS refresh loops.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

`GetCallerIdentity` is the fastest STS sanity check. It confirms which principal the SDK is using without requiring you to guess whether the current credentials came from environment variables, a profile, ECS, EC2, or another provider.

```javascript
import {
  GetCallerIdentityCommand,
  STSClient,
} from "@aws-sdk/client-sts";

const sts = new STSClient({ region: "us-east-1" });

const response = await sts.send(new GetCallerIdentityCommand({}));

console.log(response.Account);
console.log(response.Arn);
console.log(response.UserId);
```

`GetCallerIdentity` is also unusual because STS returns this identity information even when an IAM policy explicitly denies `sts:GetCallerIdentity`.

## Common Operations

### Assume a role

Use `AssumeRole` for cross-account access, privilege narrowing, or role chaining from trusted backend code.

```javascript
import {
  AssumeRoleCommand,
  STSClient,
} from "@aws-sdk/client-sts";

const sts = new STSClient({ region: "us-east-1" });

const { Credentials, AssumedRoleUser } = await sts.send(
  new AssumeRoleCommand({
    RoleArn: "arn:aws:iam::123456789012:role/CrossAccountReadOnly",
    RoleSessionName: "billing-report-job",
    ExternalId: process.env.AWS_EXTERNAL_ID,
    DurationSeconds: 3600,
  }),
);

if (!Credentials) {
  throw new Error("AssumeRole did not return credentials.");
}

console.log(AssumedRoleUser?.Arn);
console.log(Credentials.Expiration);
```

The returned session is only usable if you pass all three credential parts onward: `AccessKeyId`, `SecretAccessKey`, and `SessionToken`.

### Use assumed credentials in another SDK client

After `AssumeRole`, pass the temporary credential set directly into the client that should run under that role.

```javascript
import {
  AssumeRoleCommand,
  STSClient,
} from "@aws-sdk/client-sts";
import { S3Client, ListBucketsCommand } from "@aws-sdk/client-s3";

const sts = new STSClient({ region: "us-east-1" });

const { Credentials } = await sts.send(
  new AssumeRoleCommand({
    RoleArn: "arn:aws:iam::123456789012:role/CrossAccountReadOnly",
    RoleSessionName: "list-buckets-job",
  }),
);

if (!Credentials?.AccessKeyId || !Credentials.SecretAccessKey || !Credentials.SessionToken) {
  throw new Error("Incomplete STS credentials returned.");
}

const s3 = new S3Client({
  region: "us-east-1",
  credentials: {
    accessKeyId: Credentials.AccessKeyId,
    secretAccessKey: Credentials.SecretAccessKey,
    sessionToken: Credentials.SessionToken,
  },
});

const { Buckets } = await s3.send(new ListBucketsCommand({}));

console.log(Buckets?.map((bucket) => bucket.Name));
```

If you need auto-refresh instead of one manual STS call, move this flow to `@aws-sdk/credential-providers`.

### Request an MFA-backed session token

Use `GetSessionToken` when an IAM user needs temporary credentials tied to MFA.

```javascript
import {
  GetSessionTokenCommand,
  STSClient,
} from "@aws-sdk/client-sts";

const sts = new STSClient({ region: "us-east-1" });

const { Credentials } = await sts.send(
  new GetSessionTokenCommand({
    SerialNumber: "arn:aws:iam::123456789012:mfa/alice",
    TokenCode: "123456",
    DurationSeconds: 3600,
  }),
);

console.log(Credentials?.AccessKeyId);
console.log(Credentials?.Expiration);
```

This does not switch you into a different IAM role. It issues temporary credentials for the calling IAM identity.

### Assume a role with a web identity token

Use `AssumeRoleWithWebIdentity` when an external identity provider gives you an OIDC token and IAM is configured to trust that provider.

```javascript
import { readFile } from "node:fs/promises";
import {
  AssumeRoleWithWebIdentityCommand,
  STSClient,
} from "@aws-sdk/client-sts";

const sts = new STSClient({ region: "us-east-1" });

const webIdentityToken = await readFile(
  "/var/run/secrets/eks.amazonaws.com/serviceaccount/token",
  "utf8",
);

const { Credentials, SubjectFromWebIdentityToken } = await sts.send(
  new AssumeRoleWithWebIdentityCommand({
    RoleArn: "arn:aws:iam::123456789012:role/oidc-workload-role",
    RoleSessionName: "web-identity-session",
    WebIdentityToken: webIdentityToken,
  }),
);

console.log(SubjectFromWebIdentityToken);
console.log(Credentials?.Expiration);
```

For EKS, local profile-based assume-role, or recurring browser-safe refresh, the higher-level providers in `@aws-sdk/credential-providers` are usually less error-prone than calling this command directly.

### Decode an encoded authorization failure

Some AWS API errors include an encoded authorization message. `DecodeAuthorizationMessage` turns that into a human-readable JSON string.

```javascript
import {
  DecodeAuthorizationMessageCommand,
  STSClient,
} from "@aws-sdk/client-sts";

const sts = new STSClient({ region: "us-east-1" });

const { DecodedMessage } = await sts.send(
  new DecodeAuthorizationMessageCommand({
    EncodedMessage: process.env.AWS_ENCODED_AUTH_MESSAGE,
  }),
);

if (DecodedMessage) {
  console.log(JSON.parse(DecodedMessage));
}
```

Only some AWS service errors include an encoded authorization message, so treat this as a troubleshooting tool rather than part of a normal request path.

## STS-Specific Gotchas

- Temporary STS credentials always include a session token. If you forget `SessionToken`, downstream AWS clients fail even if the access key and secret key look valid.
- `RoleSessionName` is required for role-assumption flows and shows up in AWS logs and ARNs. Use a descriptive, traceable value.
- `GetSessionToken` and `AssumeRole` solve different problems: one creates a temporary session for the current IAM identity, the other switches into another role.
- Session policies, managed policy ARNs, and session tags contribute to `PackedPolicySize`. If that value grows too large, role assumption can fail.
- Temporary credentials expire. Cache them with the expiration timestamp, not forever, and prefer auto-refreshing credential providers when possible.
- `GetCallerIdentity` is the safest first diagnostic when you are unsure which principal the SDK resolved.
- `DecodeAuthorizationMessage` requires an encoded message from another AWS error response; it is not a generic IAM simulator.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: `fromIni`, role chaining, IAM Identity Center, token-file and web-identity helpers, and auto-refreshing temporary credentials.
- `@aws-sdk/client-iam`: manage roles, trust policies, users, and policy attachments; STS consumes those identities and trust relationships but does not administer them.
- `@aws-sdk/client-cognito-identity`: browser and mobile federation patterns that should not embed long-lived AWS keys.

---
name: cognito-identity
description: "AWS SDK for JavaScript v3 Cognito Identity client for identity pools and temporary AWS credentials."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,cognito,identity-pools,javascript,nodejs,browser,credentials"
---

# `@aws-sdk/client-cognito-identity`

Use this package for Amazon Cognito **identity pools**: exchanging trusted identity-provider tokens for temporary AWS credentials, resolving Cognito identity IDs, and reading or managing identity-pool metadata. It is not the package for Cognito user-pool sign-in, sign-up, MFA, or admin APIs.

## Install

```bash
npm install @aws-sdk/client-cognito-identity
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Golden Rule

- Use `@aws-sdk/client-cognito-identity` for identity pools and federated AWS credentials.
- Use `@aws-sdk/client-cognito-identity-provider` for user-pool authentication and admin flows.
- Prefer `CognitoIdentityClient` plus explicit commands over the aggregated `CognitoIdentity` client.
- If you only need credentials for another AWS client, `fromCognitoIdentityPool()` is often simpler than manually calling `GetId` and `GetCredentialsForIdentity`.
- `IdentityPoolId` looks like `us-east-1:12345678-1234-1234-1234-123456789012`.
- Provider tokens go in the `Logins` map, keyed by provider identifier.

## Client Setup

```javascript
import { CognitoIdentityClient } from "@aws-sdk/client-cognito-identity";

const cognitoIdentity = new CognitoIdentityClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

In Node.js, the default credential provider chain is usually enough for calling the Cognito Identity service itself.

## Core Usage Pattern

The usual identity-pool flow is:

1. Exchange a trusted provider token for an `IdentityId` with `GetIdCommand`.
2. Exchange that `IdentityId` for temporary AWS credentials with `GetCredentialsForIdentityCommand`.

```javascript
import {
  CognitoIdentityClient,
  GetCredentialsForIdentityCommand,
  GetIdCommand,
} from "@aws-sdk/client-cognito-identity";

const cognitoIdentity = new CognitoIdentityClient({ region: "us-east-1" });

const identityPoolId = "us-east-1:12345678-1234-1234-1234-123456789012";
const logins = {
  "cognito-idp.us-east-1.amazonaws.com/us-east-1_Example": idToken,
};

const { IdentityId } = await cognitoIdentity.send(
  new GetIdCommand({
    IdentityPoolId: identityPoolId,
    Logins: logins,
  }),
);

if (!IdentityId) {
  throw new Error("No IdentityId returned");
}

const { Credentials } = await cognitoIdentity.send(
  new GetCredentialsForIdentityCommand({
    IdentityId,
    Logins: logins,
  }),
);

if (!Credentials?.AccessKeyId || !Credentials.SecretKey || !Credentials.SessionToken) {
  throw new Error("No AWS credentials returned");
}

console.log(Credentials.AccessKeyId);
console.log(Credentials.Expiration);
```

If you pass the returned credentials into another AWS SDK client manually, map `Credentials.SecretKey` to the SDK config field `secretAccessKey`.

## Credentials and Region

- Node.js server code can usually rely on the default AWS credential chain for the `CognitoIdentityClient` itself.
- Browser code should not embed long-lived AWS access keys. Use Cognito identity providers or `fromCognitoIdentityPool()`.
- The client `region`, the identity pool, and any provider-specific configuration should all line up.
- The `Logins` map keys must exactly match the provider names configured on the identity pool.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Cognito Identity Gotchas

- Identity pools and user pools are separate concepts. A user-pool token does not become AWS credentials until you exchange it through an identity pool.
- `GetIdCommand` returns an `IdentityId`, not credentials.
- `GetCredentialsForIdentityCommand` returns temporary credentials that expire.
- Calling `GetIdCommand` without `Logins` only works when the identity pool allows unauthenticated identities.
- `Logins` provider names are strict string identifiers; small mismatches cause hard-to-read authorization failures.
- For most browser and mobile app code, automatic credential refresh via `@aws-sdk/credential-providers` is safer than hand-rolling refresh logic.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: `fromCognitoIdentityPool()` and related helpers for automatic refresh and simpler app wiring.
- `@aws-sdk/client-cognito-identity-provider`: user-pool sign-in, token issuance, MFA, password flows, and admin APIs.
- Higher-level auth libraries such as Amplify Auth when you want a full browser or mobile auth flow instead of raw SDK commands.

## Common Cognito Identity Operations

### Use `fromCognitoIdentityPool()` for another AWS client

If your goal is simply to give another AWS SDK client temporary credentials, the credential-provider helper is often the cleanest integration point.

```javascript
import { S3Client } from "@aws-sdk/client-s3";
import { fromCognitoIdentityPool } from "@aws-sdk/credential-providers";

const s3 = new S3Client({
  region: "us-east-1",
  credentials: fromCognitoIdentityPool({
    clientConfig: { region: "us-east-1" },
    identityPoolId: "us-east-1:12345678-1234-1234-1234-123456789012",
    logins: {
      "cognito-idp.us-east-1.amazonaws.com/us-east-1_Example": idToken,
    },
  }),
});
```

This avoids manually caching `IdentityId` values and refreshing expired AWS credentials yourself.

### Get an unauthenticated identity

Unauthenticated access only works when the identity pool explicitly allows it.

```javascript
import {
  CognitoIdentityClient,
  GetIdCommand,
} from "@aws-sdk/client-cognito-identity";

const cognitoIdentity = new CognitoIdentityClient({ region: "us-east-1" });

const { IdentityId } = await cognitoIdentity.send(
  new GetIdCommand({
    IdentityPoolId: "us-east-1:12345678-1234-1234-1234-123456789012",
  }),
);

console.log(IdentityId);
```

### Get an OpenID token for an identity

`GetOpenIdTokenCommand` can produce an OpenID token for an already resolved Cognito identity.

```javascript
import {
  CognitoIdentityClient,
  GetOpenIdTokenCommand,
} from "@aws-sdk/client-cognito-identity";

const cognitoIdentity = new CognitoIdentityClient({ region: "us-east-1" });

const { Token } = await cognitoIdentity.send(
  new GetOpenIdTokenCommand({
    IdentityId: "us-east-1:example-identity-id",
    Logins: {
      "cognito-idp.us-east-1.amazonaws.com/us-east-1_Example": idToken,
    },
  }),
);

console.log(Token);
```

### Read identity-pool settings

```javascript
import {
  CognitoIdentityClient,
  DescribeIdentityPoolCommand,
} from "@aws-sdk/client-cognito-identity";

const cognitoIdentity = new CognitoIdentityClient({ region: "us-east-1" });

const pool = await cognitoIdentity.send(
  new DescribeIdentityPoolCommand({
    IdentityPoolId: "us-east-1:12345678-1234-1234-1234-123456789012",
  }),
);

console.log(pool.IdentityPoolName);
console.log(pool.AllowUnauthenticatedIdentities);
```

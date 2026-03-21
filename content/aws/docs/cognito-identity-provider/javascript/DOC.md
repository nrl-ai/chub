---
name: cognito-identity-provider
description: "AWS SDK for JavaScript v3 Cognito Identity Provider client for Amazon Cognito User Pools authentication and user administration."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,cognito,authentication,user-pools,javascript,nodejs,browser"
---

# `@aws-sdk/client-cognito-identity-provider`

Use this package for Amazon Cognito User Pools APIs in AWS SDK for JavaScript v3. It covers both app-client flows such as sign-up and sign-in, and IAM-backed admin APIs such as listing users or reading a user profile by username.

## Install

```bash
npm install @aws-sdk/client-cognito-identity-provider
```

Prefer `CognitoIdentityProviderClient` plus explicit command imports. The package also exposes an aggregated client, but command-based imports are the safer default for smaller bundles and clearer call sites.

## Initialize the client

```javascript
import { CognitoIdentityProviderClient } from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({
  region: "us-east-1",
});
```

The region must match the user pool region.

## Authentication Model

- Public user-pool app-client APIs such as `SignUp`, `ConfirmSignUp`, `InitiateAuth`, `RespondToAuthChallenge`, `ForgotPassword`, and `GetUser` are driven by app-client IDs or user tokens.
- Admin APIs such as `AdminGetUser`, `AdminCreateUser`, `AdminInitiateAuth`, and `ListUsers` require AWS credentials and IAM permissions.
- Browser code should use only public app clients and should never include an app client secret.
- If your app client has a secret, the relevant public commands need a `SecretHash`; compute that only on a trusted backend.

## Credentials and Region

- Server-side admin commands usually rely on the standard AWS credential provider chain.
- Public user-pool flows can be called without AWS credentials, but they still need the correct `region` and `ClientId`.
- Keep admin operations on the server side even if the same user pool also supports public client-side auth flows.

Typical server-side setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

Use `client.send(new Command(...))` for every operation.

```javascript
import {
  CognitoIdentityProviderClient,
  InitiateAuthCommand,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

const response = await cognito.send(
  new InitiateAuthCommand({
    AuthFlow: "USER_PASSWORD_AUTH",
    ClientId: process.env.COGNITO_APP_CLIENT_ID,
    AuthParameters: {
      USERNAME: "ada@example.com",
      PASSWORD: "correct horse battery staple",
    },
  }),
);

if (response.ChallengeName) {
  console.log("Additional challenge required:", response.ChallengeName);
} else {
  console.log(response.AuthenticationResult?.AccessToken);
}
```

`USER_PASSWORD_AUTH` must be enabled on the app client. If Cognito returns a `ChallengeName`, continue with `RespondToAuthChallengeCommand` instead of assuming token issuance is complete.

## Common Operations

### Sign up a user

```javascript
import {
  CognitoIdentityProviderClient,
  SignUpCommand,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

await cognito.send(
  new SignUpCommand({
    ClientId: process.env.COGNITO_APP_CLIENT_ID,
    Username: "ada@example.com",
    Password: "correct horse battery staple",
    UserAttributes: [
      { Name: "email", Value: "ada@example.com" },
    ],
  }),
);
```

### Read the current user from an access token

```javascript
import {
  CognitoIdentityProviderClient,
  GetUserCommand,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

const user = await cognito.send(
  new GetUserCommand({
    AccessToken: accessToken,
  }),
);

console.log(user.Username);
```

`GetUser` takes an access token, not an ID token.

### Read a user by username with admin permissions

```javascript
import {
  AdminGetUserCommand,
  CognitoIdentityProviderClient,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

const user = await cognito.send(
  new AdminGetUserCommand({
    UserPoolId: "us-east-1_example",
    Username: "ada@example.com",
  }),
);

console.log(user.UserStatus);
```

## Cognito-Specific Gotchas

- `ClientId` is the user-pool app client ID, not the `UserPoolId`.
- Public and admin commands have different auth models; do not assume a command that works in browser code also works for admin automation.
- App clients with secrets require `SecretHash` on several public operations, which means those flows belong on a trusted backend.
- `InitiateAuth` can return challenges such as MFA or password resets; handle them with `RespondToAuthChallengeCommand`.
- `GetUser` needs an access token from Cognito, not an AWS credential and not an ID token.
- Do not deep-import package internals from build directories.

## When To Reach For Other Packages

- `aws-amplify`: higher-level browser and mobile auth flows, hosted UI integration, token storage, and session management.
- `@aws-sdk/credential-providers`: server-side AWS credential resolution for admin Cognito operations.

## Common Cognito Identity Provider Operations

### Confirm a user after sign-up

```javascript
import {
  CognitoIdentityProviderClient,
  ConfirmSignUpCommand,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

await cognito.send(
  new ConfirmSignUpCommand({
    ClientId: process.env.COGNITO_APP_CLIENT_ID,
    Username: "ada@example.com",
    ConfirmationCode: codeFromEmail,
  }),
);
```

### Sign in and handle a follow-up challenge

Use `RespondToAuthChallengeCommand` whenever `InitiateAuth` does not return tokens immediately.

```javascript
import {
  CognitoIdentityProviderClient,
  InitiateAuthCommand,
  RespondToAuthChallengeCommand,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

const auth = await cognito.send(
  new InitiateAuthCommand({
    AuthFlow: "USER_PASSWORD_AUTH",
    ClientId: process.env.COGNITO_APP_CLIENT_ID,
    AuthParameters: {
      USERNAME: "ada@example.com",
      PASSWORD: password,
    },
  }),
);

if (auth.ChallengeName === "NEW_PASSWORD_REQUIRED") {
  const challenge = await cognito.send(
    new RespondToAuthChallengeCommand({
      ClientId: process.env.COGNITO_APP_CLIENT_ID,
      ChallengeName: auth.ChallengeName,
      Session: auth.Session,
      ChallengeResponses: {
        USERNAME: "ada@example.com",
        NEW_PASSWORD: newPassword,
      },
    }),
  );

  console.log(challenge.AuthenticationResult?.AccessToken);
}
```

### Start a forgot-password flow

```javascript
import {
  CognitoIdentityProviderClient,
  ForgotPasswordCommand,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

await cognito.send(
  new ForgotPasswordCommand({
    ClientId: process.env.COGNITO_APP_CLIENT_ID,
    Username: "ada@example.com",
  }),
);
```

### Complete a forgot-password flow

```javascript
import {
  CognitoIdentityProviderClient,
  ConfirmForgotPasswordCommand,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

await cognito.send(
  new ConfirmForgotPasswordCommand({
    ClientId: process.env.COGNITO_APP_CLIENT_ID,
    Username: "ada@example.com",
    ConfirmationCode: codeFromEmail,
    Password: newPassword,
  }),
);
```

### List users with pagination

`ListUsers` is an admin API, so run it only with AWS credentials that can access the target user pool.

```javascript
import {
  CognitoIdentityProviderClient,
  paginateListUsers,
} from "@aws-sdk/client-cognito-identity-provider";

const cognito = new CognitoIdentityProviderClient({ region: "us-east-1" });

const paginator = paginateListUsers(
  { client: cognito },
  {
    UserPoolId: "us-east-1_example",
    Limit: 60,
  },
);

for await (const page of paginator) {
  for (const user of page.Users ?? []) {
    console.log(user.Username);
  }
}
```

### Compute `SecretHash` on a trusted backend

If the app client has a secret, Cognito expects a base64-encoded HMAC-SHA256 over `username + clientId`.

```javascript
import { createHmac } from "node:crypto";

function makeSecretHash({ username, clientId, clientSecret }) {
  return createHmac("sha256", clientSecret)
    .update(`${username}${clientId}`)
    .digest("base64");
}
```

Do not ship the client secret to browser code.

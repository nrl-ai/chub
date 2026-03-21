---
name: ses
description: "AWS SDK for JavaScript v3 client for Amazon SES email sending, identity management, and template workflows."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,ses,email,javascript,nodejs,templating,delivery"
---

# `@aws-sdk/client-ses`

Use this package for the classic Amazon SES API in AWS SDK for JavaScript v3. It covers email sending, identity verification, template management, and related account-level SES operations.

This package is for the older SES `2010-12-01` API surface. If you need newer SES v2 features such as configuration sets, suppression-list operations, or contact-list features, use `@aws-sdk/client-sesv2` instead.

## Install

```bash
npm install @aws-sdk/client-ses
```

Prefer `SESClient` plus explicit command imports. The package also exposes an aggregated `SES` client, but command-based imports are the safer default for smaller bundles and clearer call sites.

## Initialize the client

```javascript
import { SESClient } from "@aws-sdk/client-ses";

const ses = new SESClient({
  region: "us-east-1",
});
```

SES is regional. Verified identities, sandbox restrictions, quotas, and deliverability settings are evaluated per region, so the client region must match the region where your SES resources are configured.

## Credentials and Region

- Node.js: the default AWS credential provider chain usually works if credentials are already configured through environment variables, shared config, ECS, EC2, or IAM Identity Center.
- Browser runtimes: this package can run in browser bundles, but direct browser-side email sending is usually the wrong design because it exposes privileged SES permissions.
- Keep the region explicit somewhere: in code, `AWS_REGION`, or shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## SES-Specific Rules

- In the SES sandbox, you can send only to verified identities unless your account has production access in that region.
- Sender identities must be verified in the same region where you send mail.
- Recipient limits, message-size limits, and sending quotas are service-side checks; a successful client call still depends on SES account state.
- `SendEmailCommand` is for structured subject/body content. Use `SendRawEmailCommand` when you need MIME control, custom headers, or attachments.

## Core Usage Pattern

The v3 SDK uses `client.send(new Command(input))`.

```javascript
import {
  SESClient,
  SendEmailCommand,
} from "@aws-sdk/client-ses";

const ses = new SESClient({ region: "us-east-1" });

await ses.send(
  new SendEmailCommand({
    Source: "noreply@example.com",
    Destination: {
      ToAddresses: ["ada@example.com"],
    },
    Message: {
      Subject: {
        Data: "Order received",
        Charset: "UTF-8",
      },
      Body: {
        Text: {
          Data: "Your order was received.",
          Charset: "UTF-8",
        },
      },
    },
  }),
);
```

Use a verified sender identity and keep the sender region aligned with the SES region in code.

## Common Operations

### Send an HTML email

```javascript
import {
  SESClient,
  SendEmailCommand,
} from "@aws-sdk/client-ses";

const ses = new SESClient({ region: "us-east-1" });

await ses.send(
  new SendEmailCommand({
    Source: "noreply@example.com",
    Destination: {
      ToAddresses: ["ada@example.com"],
      CcAddresses: ["ops@example.com"],
    },
    ReplyToAddresses: ["support@example.com"],
    Message: {
      Subject: {
        Data: "Welcome to Example Co",
        Charset: "UTF-8",
      },
      Body: {
        Html: {
          Data: "<h1>Welcome</h1><p>Your account is ready.</p>",
          Charset: "UTF-8",
        },
        Text: {
          Data: "Welcome. Your account is ready.",
          Charset: "UTF-8",
        },
      },
    },
  }),
);
```

Provide both HTML and text bodies when possible so clients that do not render HTML still receive a readable message.

### Send a templated email

```javascript
import {
  SESClient,
  SendTemplatedEmailCommand,
} from "@aws-sdk/client-ses";

const ses = new SESClient({ region: "us-east-1" });

await ses.send(
  new SendTemplatedEmailCommand({
    Source: "noreply@example.com",
    Destination: {
      ToAddresses: ["ada@example.com"],
    },
    Template: "order-created",
    TemplateData: JSON.stringify({
      firstName: "Ada",
      orderId: "12345",
    }),
  }),
);
```

`TemplateData` must be a JSON string, not a plain JavaScript object.

### Create an email template

```javascript
import {
  CreateTemplateCommand,
  SESClient,
} from "@aws-sdk/client-ses";

const ses = new SESClient({ region: "us-east-1" });

await ses.send(
  new CreateTemplateCommand({
    Template: {
      TemplateName: "order-created",
      SubjectPart: "Order {{orderId}} received",
      TextPart: "Hi {{firstName}}, your order {{orderId}} was received.",
      HtmlPart:
        "<h1>Thanks {{firstName}}</h1><p>Your order {{orderId}} was received.</p>",
    },
  }),
);
```

Template names are region-scoped SES resources, so keep naming stable across deploys.

### List verified identities

```javascript
import {
  ListIdentitiesCommand,
  SESClient,
} from "@aws-sdk/client-ses";

const ses = new SESClient({ region: "us-east-1" });

let nextToken;

do {
  const response = await ses.send(
    new ListIdentitiesCommand({
      IdentityType: "EmailAddress",
      MaxItems: 100,
      NextToken: nextToken,
    }),
  );

  for (const identity of response.Identities ?? []) {
    console.log(identity);
  }

  nextToken = response.NextToken;
} while (nextToken);
```

Use this for admin flows and environment checks rather than for every send path.

## SES Gotchas

- `@aws-sdk/client-ses` and `@aws-sdk/client-sesv2` are different service packages. Choose intentionally based on the API surface you need.
- SES sandbox restrictions are regional. Being out of the sandbox in one region does not imply the same state in another.
- Verified sender identity checks are also regional. A domain verified in one region is not automatically verified everywhere.
- `SendTemplatedEmailCommand` and related template APIs fail if template placeholders and `TemplateData` keys do not line up.
- `SendRawEmailCommand` is the correct path for attachments and full MIME control; `SendEmailCommand` does not accept arbitrary attachments.
- Keep email-sending permissions server-side. Direct browser calls with broad SES permissions are usually a bad default.

## When To Reach For Other Packages

- `@aws-sdk/client-sesv2`: newer SES v2 APIs such as suppression lists, configuration sets, and account-level deliverability features.
- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, Cognito, and assume-role flows.
- `nodemailer`: higher-level application mail composition when you still want to send through SES.

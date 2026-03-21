---
name: kms
description: "AWS SDK for JavaScript v3 client for AWS Key Management Service key management and cryptographic operations"
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,kms,key-management,encryption,security,javascript"
---

# AWS KMS SDK for JavaScript (v3)

Use `@aws-sdk/client-kms` to manage KMS keys, encrypt and decrypt small payloads, generate data keys for envelope encryption, and inspect aliases and key metadata from JavaScript or TypeScript code.

## Golden Rule

- Install `@aws-sdk/client-kms`, not the legacy `aws-sdk` v2 package.
- The package version covered here is `3.1006.0`.
- AWS SDK for JavaScript v2 reached end of support on September 8, 2025.
- Current v3 releases at and above `3.968.0` require Node.js 20+, so `3.1006.0` should be treated as Node 20+.
- Use KMS `Encrypt` and `Decrypt` for small payloads. For files, large objects, or application data at rest, use envelope encryption with `GenerateDataKey` and do the bulk encryption locally.
- Import from the package root only. Do not deep-import from `dist-*` paths.

## Install

```bash
npm install @aws-sdk/client-kms
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Client Setup

Prefer `KMSClient` plus explicit command imports. The package also exposes an aggregated `KMS` client, but command-based imports are the safer default for smaller bundles and clearer call sites.

### Minimal Node.js client

```javascript
import { KMSClient } from "@aws-sdk/client-kms";

const kms = new KMSClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

### Explicit credentials

```javascript
import { KMSClient } from "@aws-sdk/client-kms";

const kms = new KMSClient({
  region: "us-east-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});
```

### Browser usage warning

The package can run in browser builds, but direct browser access to KMS is usually the wrong architecture. Prefer calling KMS from a trusted backend or Lambda function. If you must use it in a browser, use tightly scoped temporary credentials and restrict permissions aggressively.

## Credentials and Region

- In Node.js, the default credential provider chain is usually enough if credentials already come from environment variables, shared AWS config, ECS, EC2 instance metadata, or IAM Identity Center.
- KMS keys are regional unless you intentionally work with multi-Region keys. Your client region must still match the key or alias you intend to use.
- KMS authorization is a combination of IAM permissions and key policy permissions. A principal can have valid AWS credentials and still be denied by the key policy.

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
  DescribeKeyCommand,
  KMSClient,
} from "@aws-sdk/client-kms";

const kms = new KMSClient({ region: "us-east-1" });

const response = await kms.send(
  new DescribeKeyCommand({
    KeyId: "alias/app-key",
  }),
);

console.log(response.KeyMetadata?.Arn);
```

Use aliases like `alias/app-key` where possible so application code does not hard-code raw key IDs.

## Common Operations

### Encrypt a small string

Use `TextEncoder` to turn text into bytes before calling KMS.

```javascript
import {
  EncryptCommand,
  KMSClient,
} from "@aws-sdk/client-kms";

const kms = new KMSClient({ region: "us-east-1" });
const plaintext = new TextEncoder().encode("hello from kms");

const { CiphertextBlob } = await kms.send(
  new EncryptCommand({
    KeyId: "alias/app-key",
    Plaintext: plaintext,
    EncryptionContext: {
      service: "billing",
      env: "prod",
    },
  }),
);

const ciphertextBase64 = Buffer.from(CiphertextBlob).toString("base64");
console.log(ciphertextBase64);
```

### Decrypt ciphertext

If you encrypted with an `EncryptionContext`, provide the same key-value pairs when decrypting.

```javascript
import {
  DecryptCommand,
  KMSClient,
} from "@aws-sdk/client-kms";

const kms = new KMSClient({ region: "us-east-1" });

const ciphertextBase64 = process.env.KMS_CIPHERTEXT;

const { Plaintext } = await kms.send(
  new DecryptCommand({
    CiphertextBlob: Buffer.from(ciphertextBase64, "base64"),
    EncryptionContext: {
      service: "billing",
      env: "prod",
    },
  }),
);

const value = new TextDecoder().decode(Plaintext);
console.log(value);
```

### Generate a data key for envelope encryption

For application payloads, ask KMS for a data key, use the plaintext key locally, and store only the encrypted data key beside the ciphertext.

```javascript
import {
  GenerateDataKeyCommand,
  KMSClient,
} from "@aws-sdk/client-kms";

const kms = new KMSClient({ region: "us-east-1" });

const { Plaintext, CiphertextBlob, KeyId } = await kms.send(
  new GenerateDataKeyCommand({
    KeyId: "alias/app-key",
    KeySpec: "AES_256",
    EncryptionContext: {
      service: "billing",
      env: "prod",
    },
  }),
);

const dataKey = Buffer.from(Plaintext);
const encryptedDataKey = Buffer.from(CiphertextBlob).toString("base64");

console.log({
  keyId: KeyId,
  dataKeyLength: dataKey.length,
  encryptedDataKey,
});
```

After using the plaintext data key, clear it from memory as early as your runtime and crypto flow allow.

### Create a symmetric encryption key and alias

```javascript
import {
  CreateAliasCommand,
  CreateKeyCommand,
  KMSClient,
} from "@aws-sdk/client-kms";

const kms = new KMSClient({ region: "us-east-1" });

const { KeyMetadata } = await kms.send(
  new CreateKeyCommand({
    Description: "Application envelope encryption key",
    KeyUsage: "ENCRYPT_DECRYPT",
    KeySpec: "SYMMETRIC_DEFAULT",
  }),
);

if (!KeyMetadata?.KeyId) {
  throw new Error("Missing key id");
}

await kms.send(
  new CreateAliasCommand({
    AliasName: "alias/app-key",
    TargetKeyId: KeyMetadata.KeyId,
  }),
);

console.log(KeyMetadata.KeyId);
```

Use aliases for application configuration and keep raw key IDs in infrastructure state or admin tooling, not in day-to-day app code.

### List aliases

```javascript
import {
  KMSClient,
  ListAliasesCommand,
} from "@aws-sdk/client-kms";

const kms = new KMSClient({ region: "us-east-1" });

const response = await kms.send(new ListAliasesCommand({}));

for (const alias of response.Aliases ?? []) {
  console.log(alias.AliasName, alias.TargetKeyId);
}
```

List operations can paginate. For large accounts, keep following `NextMarker` until `Truncated` is false.

## KMS-Specific Gotchas

- `Plaintext`, `CiphertextBlob`, and generated data keys are binary values. Use `Uint8Array` or `Buffer`, not plain strings.
- `Encrypt` and `Decrypt` are not for large payloads. Use envelope encryption for files, database fields at scale, or other bulk data.
- `EncryptionContext` is case-sensitive and exact-match. If you use it during encryption, keep it stable and supply the same context when decrypting.
- KMS key access depends on both IAM and key policies. A missing key-policy grant is a common cause of `AccessDeniedException`.
- Aliases are region-scoped. `alias/app-key` in one region is unrelated to the same alias name in another region.
- Keys that are disabled, pending deletion, or otherwise not usable will cause cryptographic operations to fail even if the key identifier resolves.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/credential-providers`: shared config, IAM Identity Center, assume-role, and other explicit credential flows.
- `@aws-sdk/client-secrets-manager`: secret storage and rotation workflows instead of storing ad hoc ciphertext blobs yourself.
- `@aws-sdk/client-ssm`: configuration and `SecureString` parameter workflows backed by KMS.

## Practical Notes

- Prefer aliases such as `alias/app-key` in application settings so you can rotate the backing key with less code churn.
- Keep cryptographic permissions narrow. Many workloads only need `kms:Decrypt` on one key, not broad KMS administration rights.
- Log key identifiers and alias names, but never log plaintext values, plaintext data keys, or raw decrypted secrets.
- If you need high-level client-side encryption workflows instead of raw KMS primitives, reach for an envelope-encryption library rather than rebuilding everything around `Encrypt` and `Decrypt` directly.

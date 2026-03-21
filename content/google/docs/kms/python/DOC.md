---
name: kms
description: "Google Cloud KMS Python client for key rings, crypto keys, IAM-aware setup, and symmetric encrypt/decrypt workflows"
metadata:
  languages: "python"
  versions: "3.11.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,kms,cloud-kms,encryption,gcp,adc"
---

# Google Cloud KMS Python Client

Use `google-cloud-kms` for Cloud KMS automation from Python. Install the PyPI package, authenticate with Application Default Credentials (ADC), import `kms_v1`, and work with fully qualified resource names for key rings, crypto keys, and crypto key versions.

```python
from google.cloud import kms_v1
```

## Version Note

This entry is written against package version `3.11.0`, which PyPI lists as the current visible release as of `2026-03-12`.

Official upstream pages do not fully agree on freshness:

- PyPI lists `3.11.0`
- the `KeyManagementServiceClient` reference page renders `3.11.0`
- the docs overview page and changelog under `latest` still render `3.7.0` as the latest visible version

Practical implication:

- the examples below use the current `kms_v1` client surface
- if you need a field or method added very recently, confirm it against the installed wheel and the class reference page, not only the overview page

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-kms==3.11.0"
```

Common alternatives:

```bash
uv add "google-cloud-kms==3.11.0"
poetry add "google-cloud-kms==3.11.0"
```

PyPI currently declares Python `>=3.7` for this package.

## Setup And Authentication

Before writing code:

1. Select the Google Cloud project that owns the KMS resources.
2. Enable the Cloud KMS API.
3. Authenticate with ADC.
4. Make sure the principal has the right IAM role on the project, key ring, or key.

Enable the API:

```bash
gcloud services enable cloudkms.googleapis.com
```

Local development with ADC:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
export GOOGLE_CLOUD_PROJECT="YOUR_PROJECT_ID"
export GOOGLE_CLOUD_LOCATION="us-central1"
```

If you must use a service account key file locally:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
export GOOGLE_CLOUD_PROJECT="YOUR_PROJECT_ID"
export GOOGLE_CLOUD_LOCATION="us-central1"
```

Typical permissions:

- symmetric encrypt/decrypt: `roles/cloudkms.cryptoKeyEncrypterDecrypter`
- metadata reads and listings: `roles/cloudkms.viewer`
- key and key ring administration: `roles/cloudkms.admin` or narrower resource-specific roles

## Client Creation

Default client with ADC:

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()
```

Async client:

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceAsyncClient()
```

Explicit service account file:

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient.from_service_account_file(
    "/absolute/path/service-account.json"
)
```

REST transport instead of gRPC:

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient(transport="rest")
```

Custom endpoint:

```python
from google.cloud import kms_v1
from google.api_core.client_options import ClientOptions

client = kms_v1.KeyManagementServiceClient(
    client_options=ClientOptions(api_endpoint="cloudkms.googleapis.com")
)
```

## Resource Names

Cloud KMS methods expect fully qualified resource names.

Common patterns:

- location parent: `projects/{project}/locations/{location}`
- key ring: `projects/{project}/locations/{location}/keyRings/{key_ring}`
- crypto key: `projects/{project}/locations/{location}/keyRings/{key_ring}/cryptoKeys/{crypto_key}`
- crypto key version: `projects/{project}/locations/{location}/keyRings/{key_ring}/cryptoKeys/{crypto_key}/cryptoKeyVersions/{version}`

Prefer the helper methods on the client instead of building deep paths manually:

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()

key_ring_name = client.key_ring_path("my-project", "us-central1", "app-secrets")
crypto_key_name = client.crypto_key_path(
    "my-project",
    "us-central1",
    "app-secrets",
    "primary",
)
```

## Core Usage

### List Key Rings In A Location

```python
import os
from google.cloud import kms_v1

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.environ.get("GOOGLE_CLOUD_LOCATION", "us-central1")

client = kms_v1.KeyManagementServiceClient()
parent = f"projects/{project_id}/locations/{location}"

for key_ring in client.list_key_rings(request={"parent": parent}):
    print(key_ring.name)
```

### Inspect One Crypto Key

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()
name = client.crypto_key_path("my-project", "us-central1", "app-secrets", "primary")

crypto_key = client.get_crypto_key(request={"name": name})

print(crypto_key.name)
print(crypto_key.primary.name if crypto_key.primary else "no primary version")
print(crypto_key.purpose.name)
```

`crypto_key.primary` tells you which key version will be used by operations such as symmetric encrypt unless you explicitly target a version elsewhere.

### List Crypto Key Versions

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()
parent = client.crypto_key_path("my-project", "us-central1", "app-secrets", "primary")

for version in client.list_crypto_key_versions(request={"parent": parent}):
    print(version.name, version.state.name)
```

Use this when troubleshooting disabled, pending-destruction, or destroyed versions.

### Encrypt Data With A Symmetric Key

`encrypt()` expects a symmetric key whose purpose is `ENCRYPT_DECRYPT`.

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()
name = client.crypto_key_path("my-project", "us-central1", "app-secrets", "primary")

plaintext = b"secret payload"
response = client.encrypt(
    request={
        "name": name,
        "plaintext": plaintext,
    }
)

ciphertext = response.ciphertext
print(len(ciphertext))
```

### Decrypt Data

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()
name = client.crypto_key_path("my-project", "us-central1", "app-secrets", "primary")

decrypt_response = client.decrypt(
    request={
        "name": name,
        "ciphertext": ciphertext,
    }
)

print(decrypt_response.plaintext.decode("utf-8"))
```

If you store ciphertext in JSON, a database text column, or another text-only channel, base64-encode it yourself before serialization and decode it back to `bytes` before calling `decrypt()`.

### Create A Key Ring

Infrastructure code often creates key rings once and reuses them for many keys:

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()
parent = "projects/my-project/locations/us-central1"

key_ring = client.create_key_ring(
    request={
        "parent": parent,
        "key_ring_id": "app-secrets",
        "key_ring": kms_v1.KeyRing(),
    }
)

print(key_ring.name)
```

### Create A Symmetric Crypto Key

```python
from google.cloud import kms_v1

client = kms_v1.KeyManagementServiceClient()
parent = client.key_ring_path("my-project", "us-central1", "app-secrets")

crypto_key = client.create_crypto_key(
    request={
        "parent": parent,
        "crypto_key_id": "primary",
        "crypto_key": kms_v1.CryptoKey(
            purpose=kms_v1.CryptoKey.CryptoKeyPurpose.ENCRYPT_DECRYPT,
            version_template=kms_v1.CryptoKeyVersionTemplate(
                algorithm=(
                    kms_v1.CryptoKeyVersion.CryptoKeyVersionAlgorithm
                    .GOOGLE_SYMMETRIC_ENCRYPTION
                )
            ),
        ),
    }
)

print(crypto_key.name)
```

For production keys, add rotation policy, labels, protection level, import settings, or HSM/EKM configuration deliberately instead of copying a minimal example unchanged.

## Configuration Notes

- Prefer ADC from the runtime environment over distributing service account JSON files.
- Pass transport or endpoint overrides through the client constructor rather than patching internals.
- If you need library logging, Google’s package docs document the `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` environment variable. Use a logger scope starting with `google`, such as `google.cloud.kms_v1`.
- The generated client also exposes path parser helpers such as `parse_crypto_key_path(...)` when you need to decompose a full resource name.

## Integrity Checks And Data Handling

The KMS API supports CRC32C integrity fields on encrypt and decrypt requests and responses, including:

- `plaintext_crc32c`
- `ciphertext_crc32c`
- `verified_plaintext_crc32c`
- `verified_ciphertext_crc32c`

Use those fields when request/response integrity matters, especially for high-assurance systems or when payloads move through intermediaries. The generated reference documents these fields on the encrypt and decrypt request/response types.

## Common Pitfalls

- The package name is `google-cloud-kms`, but the import surface for new code is `google.cloud.kms_v1`.
- `encrypt()` and `decrypt()` are for symmetric keys. Asymmetric keys use different methods such as `asymmetric_sign`, `asymmetric_decrypt`, and public-key retrieval.
- Most methods require full resource names, not short IDs.
- `PermissionDenied`, `NotFound`, and `FailedPrecondition` often mean IAM, API enablement, key state, or location mismatches rather than a bad client call.
- Disabled or destroyed key versions cannot be used for new crypto operations.
- Older examples may import `from google.cloud import kms`; prefer `kms_v1` for current code.
- The rolling docs overview currently lags PyPI for this package. Verify very recent behavior against the installed package and the class reference page.

## Version-Sensitive Notes

- The official changelog marks `3.0.0` as a breaking release to support `protobuf 5.x`.
- The same changelog notes async REST transport support in the 3.x line, so transport behavior can differ from older 2.x examples.
- On `2026-03-12`, PyPI and the class reference page show `3.11.0`, but the docs overview and changelog pages under `latest` still render `3.7.0`. Treat that as docs-site lag, not as evidence that `3.11.0` is unpublished.

## Official Sources Used

- `https://pypi.org/project/google-cloud-kms/`
- `https://cloud.google.com/python/docs/reference/cloudkms/latest`
- `https://docs.cloud.google.com/python/docs/reference/cloudkms/latest/google.cloud.kms_v1.services.key_management_service.KeyManagementServiceClient`
- `https://cloud.google.com/python/docs/reference/cloudkms/latest/changelog`
- `https://cloud.google.com/docs/authentication/provide-credentials-adc`
- `https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-kms`

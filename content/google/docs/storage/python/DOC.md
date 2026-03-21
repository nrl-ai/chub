---
name: storage
description: "Google Cloud Storage Python client library for buckets, blobs, transfers, and signed URLs"
metadata:
  languages: "python"
  versions: "3.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,cloud,storage,gcs,bucket,blob,signed-url"
---

# Google Cloud Storage Python Client

## Golden Rule

Use the official `google-cloud-storage` package with `from google.cloud import storage`, and prefer Application Default Credentials (ADC) for normal client setup. For write paths, add generation preconditions such as `if_generation_match=0` so retries stay safe and idempotent.

This package covers the main Cloud Storage Python client. Some newer control-plane features live in adjacent Google Cloud packages and product docs, so do not assume every Storage feature lands in `google-cloud-storage`.

## Install

Pin the version your project expects:

```bash
python -m pip install "google-cloud-storage==3.9.0"
```

Common alternatives:

```bash
uv add "google-cloud-storage==3.9.0"
poetry add "google-cloud-storage==3.9.0"
```

Useful extras from PyPI:

```bash
python -m pip install "google-cloud-storage[tracing]==3.9.0"
python -m pip install "google-cloud-storage[grpc]==3.9.0"
```

Use `tracing` when you want OpenTelemetry integration. Use `grpc` only when you actually need the gRPC-based async/client surfaces documented in the upstream changelog.

## Authentication And Setup

Enable the Cloud Storage API in the target Google Cloud project, then choose one of the standard auth flows.

### Local development with ADC

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
```

### Service account credentials

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
```

### Explicit credentials object

```python
from google.cloud import storage
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "service-account.json"
)

client = storage.Client(
    project="YOUR_PROJECT_ID",
    credentials=credentials,
)
```

### API key support

`storage.Client` accepts `api_key=...`, but that is not the normal path for authenticated bucket and object management. Treat ADC or service-account credentials as the default for production code, and do not mix `api_key` with `credentials`.

## Core Workflow

### Create a client, bucket, and blob handle

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-bucket")
blob = bucket.blob("reports/summary.json")
```

Creating a bucket:

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-new-bucket")
bucket.storage_class = "STANDARD"
new_bucket = client.create_bucket(bucket, location="US")

print(new_bucket.name)
```

### Upload data

Simple string upload:

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-bucket")
blob = bucket.blob("notes/hello.txt")

blob.upload_from_string("hello world", content_type="text/plain")
```

Create-only upload with a generation precondition:

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-bucket")
blob = bucket.blob("uploads/report.csv")

blob.upload_from_filename(
    "report.csv",
    if_generation_match=0,
    checksum="auto",
)
```

Use `if_generation_match=0` when the object must not already exist. That enables conditional retry behavior and prevents duplicate writes during retried uploads.

### Download data

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-bucket")
blob = bucket.blob("uploads/report.csv")

text = blob.download_as_text()
data = blob.download_as_bytes()
blob.download_to_filename("report.csv")
```

### List objects

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")

for blob in client.list_blobs("my-bucket", prefix="uploads/2026/"):
    print(blob.name, blob.size)
```

Pseudo-directory style listing:

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
iterator = client.list_blobs(
    "my-bucket",
    prefix="uploads/",
    delimiter="/",
)

for page in iterator.pages:
    for subdir in page.prefixes:
        print("prefix:", subdir)
    for blob in page:
        print("blob:", blob.name)
```

### Delete or replace safely

If you need optimistic concurrency, reload the blob first and pass a generation precondition:

```python
from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-bucket")
blob = bucket.blob("uploads/report.csv")
blob.reload()

blob.delete(if_generation_match=blob.generation)
```

The same pattern applies to overwrite flows: read the current generation, then write with `if_generation_match=<current_generation>`.

## Signed URLs

Generate a short-lived V4 signed URL for downloads:

```python
from datetime import timedelta

from google.cloud import storage

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-bucket")
blob = bucket.blob("private/report.csv")

url = blob.generate_signed_url(
    version="v4",
    expiration=timedelta(minutes=15),
    method="GET",
)

print(url)
```

Practical caveats:

- Signed URLs target Cloud Storage XML API style endpoints, even when the rest of your app uses the JSON API or the Python client library.
- URL signing needs signing-capable credentials. Many default runtime credentials can access Storage but cannot sign URLs directly.

## Parallel Transfers

For bulk uploads or downloads, use `transfer_manager` instead of writing your own thread/process orchestration:

```python
from pathlib import Path

from google.cloud import storage
from google.cloud.storage import transfer_manager

client = storage.Client(project="YOUR_PROJECT_ID")
bucket = client.bucket("my-bucket")

results = transfer_manager.upload_many_from_filenames(
    bucket,
    [str(path) for path in Path("dist").glob("*.json")],
    source_directory=".",
    blob_name_prefix="artifacts/",
    max_workers=8,
)

for result in results:
    if isinstance(result, Exception):
        raise result
```

Use this for many medium-sized files. For a single large file, a resumable upload through `upload_from_filename()` is usually enough.

## Custom Endpoints And Tests

If you point the client at a custom endpoint for tests or local emulation, be explicit about auth behavior:

```python
from google.cloud import storage

client = storage.Client(
    project="test-project",
    client_options={"api_endpoint": "http://localhost:4443"},
    use_auth_w_custom_endpoint=False,
)
```

Without `use_auth_w_custom_endpoint=False`, the client still assumes authentication on custom endpoints.

## Configuration Notes

- Set `project=` explicitly when your local ADC context and the target billing project are not the same.
- For Requester Pays buckets, set `bucket.user_project = "BILLING_PROJECT_ID"` before the operation so the request is billed correctly.
- Keep credentials out of source control. Use ADC locally and secret-managed service account credentials in CI or production.
- Prefer `checksum="auto"` unless you have a specific reason to disable checksums.

## Common Pitfalls

- Do not omit generation preconditions on important writes. Retrying a non-conditional upload can create correctness bugs.
- `download_as_text()` decodes content; use `download_as_bytes()` for binary data.
- Range downloads do not support checksum verification in the same way full-object downloads do. Validate integrity at the application layer if partial reads matter.
- `api_key` and `credentials` are mutually exclusive on `storage.Client`.
- Signed URL support and Storage data access are different capabilities. A credential that can read or write objects may still fail to sign.
- Some "folder", managed-folder, or storage-control features are documented elsewhere in Google Cloud and may require a different client package.
- Custom endpoints still try to authenticate unless you disable that behavior explicitly.

## Version-Sensitive Notes For 3.9.0

- PyPI and the latest reference docs both pointed to `3.9.0` on March 12, 2026.
- The `3.x` line changed several behaviors compared with `2.x`, especially around retries, checksum defaults, exception classes, and some upload/download edge cases. If you are migrating old code, read the changelog before assuming old retry or exception behavior still applies.
- In `3.x`, uploads and downloads moved toward safer defaults: checksum handling now defaults to `auto`, and conditional retry support matters more for safe writes.
- `3.1.0` added `api_key` support to `storage.Client`; if you see older examples claiming the client only accepts OAuth-style credentials, those examples are outdated.
- The `3.9.0` changelog includes async gRPC and zonal-bucket related additions. Those are advanced surfaces; standard sync `Client` / `Bucket` / `Blob` code remains the baseline for most agents.

## Official Sources

- Python reference root: `https://docs.cloud.google.com/python/docs/reference/storage/latest`
- Changelog: `https://docs.cloud.google.com/python/docs/reference/storage/latest/changelog`
- Client API: `https://docs.cloud.google.com/python/docs/reference/storage/latest/google.cloud.storage.client.Client`
- Blob API: `https://docs.cloud.google.com/python/docs/reference/storage/latest/google.cloud.storage.blob.Blob`
- Transfer manager API: `https://docs.cloud.google.com/python/docs/reference/storage/latest/google.cloud.storage.transfer_manager`
- Authentication guide: `https://docs.cloud.google.com/storage/docs/authentication`
- Request preconditions guide: `https://docs.cloud.google.com/storage/docs/request-preconditions`
- Signed URLs guide: `https://docs.cloud.google.com/storage/docs/access-control/signed-urls`
- PyPI package page: `https://pypi.org/project/google-cloud-storage/`
- GitHub repository: `https://github.com/googleapis/python-storage`

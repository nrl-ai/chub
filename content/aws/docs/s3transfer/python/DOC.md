---
name: s3transfer
description: "s3transfer package guide for Python managed S3 transfer orchestration"
metadata:
  languages: "python"
  versions: "0.16.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "s3transfer,aws,s3,botocore,boto3,transfers"
---

# s3transfer Python Package Guide

## Golden Rule

Prefer the stable S3 transfer helpers in `boto3` for ordinary application code.

Use direct `s3transfer` APIs only when you specifically need lower-level transfer orchestration, futures, or transfer-manager configuration. Upstream marks the direct package API as not currently GA, so pin the minor version if you depend on it directly.

## Installation

Install the direct package only if you need to work below the `boto3` wrapper:

```bash
pip install s3transfer==0.16.0
```

```bash
uv add s3transfer==0.16.0
```

```bash
poetry add s3transfer==0.16.0
```

Optional AWS Common Runtime support:

```bash
pip install "s3transfer[crt]==0.16.0"
```

## Setup And Authentication

`s3transfer` does not manage AWS credentials on its own. It runs transfer workflows on top of an S3 client from `botocore` or `boto3`, so auth and region come from the normal AWS client chain:

- `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`
- shared config in `~/.aws/config` and `~/.aws/credentials`
- IAM roles or workload identity when running in AWS

Minimal client setup with `botocore`:

```python
from botocore.session import Session

session = Session()
s3 = session.create_client("s3", region_name="us-east-1")
```

If you already use `boto3`, you can pass its S3 client instead:

```python
import boto3

s3 = boto3.client("s3", region_name="us-east-1")
```

For LocalStack, MinIO, or another S3-compatible endpoint, pass `endpoint_url=...` when creating the client.

## Core Usage

### Preferred Stable Path: `boto3`

The direct package is the engine behind `boto3`'s managed S3 transfer helpers. For most code, stay on the documented `boto3` surface:

```python
import boto3
from boto3.s3.transfer import TransferConfig

s3 = boto3.client("s3", region_name="us-east-1")
config = TransferConfig(
    multipart_threshold=8 * 1024 * 1024,
    multipart_chunksize=8 * 1024 * 1024,
    max_concurrency=10,
)

s3.upload_file(
    "dist/app.tar.gz",
    "my-bucket",
    "releases/app.tar.gz",
    Config=config,
    ExtraArgs={"ContentType": "application/gzip"},
)
```

This is the stable interface upstream recommends for production application code.

### Direct Usage: `TransferManager`

Use `TransferManager` when you need direct futures and lower-level control:

```python
from botocore.session import Session
from s3transfer.manager import TransferConfig, TransferManager

session = Session()
s3 = session.create_client("s3", region_name="us-east-1")

config = TransferConfig(
    multipart_threshold=8 * 1024 * 1024,
    multipart_chunksize=8 * 1024 * 1024,
    max_request_concurrency=10,
    max_io_queue_size=1000,
)

manager = TransferManager(s3, config=config)

try:
    future = manager.upload(
        "dist/app.tar.gz",
        "my-bucket",
        "releases/app.tar.gz",
        extra_args={"ContentType": "application/gzip"},
    )
    future.result()
finally:
    manager.shutdown()
```

Download example:

```python
from botocore.session import Session
from s3transfer.manager import TransferManager

session = Session()
s3 = session.create_client("s3", region_name="us-east-1")
manager = TransferManager(s3)

try:
    future = manager.download(
        "my-bucket",
        "releases/app.tar.gz",
        "downloads/app.tar.gz",
    )
    future.result()
finally:
    manager.shutdown()
```

Other direct operations exposed by `TransferManager` include `copy(...)` and `delete(...)`.

### Filename Wrapper: `S3Transfer`

If you want a thinner filename-based wrapper without using `boto3`:

```python
from botocore.session import Session
from s3transfer import S3Transfer

session = Session()
s3 = session.create_client("s3", region_name="us-east-1")
transfer = S3Transfer(s3)

transfer.upload_file(
    "dist/app.tar.gz",
    "my-bucket",
    "releases/app.tar.gz",
    extra_args={"ContentType": "application/gzip"},
)
```

## TransferConfig Notes

Core `s3transfer.manager.TransferConfig` knobs in `0.16.0`:

- `multipart_threshold=8388608`
- `multipart_chunksize=8388608`
- `max_request_concurrency=10`
- `max_submission_concurrency=5`
- `max_io_queue_size=1000`
- `io_chunksize=262144`
- `num_download_attempts=5`
- `max_bandwidth=None`

Important naming difference:

- `boto3.s3.transfer.TransferConfig` uses `max_concurrency`
- `s3transfer.manager.TransferConfig` uses `max_request_concurrency`

`boto3` documents this aliasing and recommends the injected client methods (`upload_file`, `download_file`, `upload_fileobj`, `download_fileobj`) as the stable public transfer surface.

## Optional CRT Support

Installing `s3transfer[crt]` enables the AWS Common Runtime transfer path through `botocore[crt]`.

Version-sensitive note for `0.16.0`:

- `CRTTransferManager` now supports `multipart_threshold`, `multipart_chunksize`, and `max_request_concurrency`

If you are upgrading from older `0.14.x` or `0.15.x` releases and rely on CRT-backed transfers, re-check your config behavior against this change.

## Common Pitfalls

### Do Not Assume The Direct Package API Is Stable

The upstream README says `s3transfer` is not currently GA and advises locking to a minor version if you use it directly in production.

### Prefer Filenames Over File-Like Objects When Possible

`TransferManager.upload()` and `download()` accept filenames or seekable file-like objects, but the upstream source notes that file-like objects can increase memory usage.

### Always Wait On The Future

`TransferManager` methods return a transfer future. Call `future.result()` or you may miss transfer failures and exit before the transfer completes.

### Always Shut Down The Manager

`TransferManager` owns executors for request submission, transfer work, and download I/O. Call `shutdown()` in a `finally` block.

### Do Not Copy `boto3` Config Names Into Direct `s3transfer` Code

Examples using `boto3.s3.transfer.TransferConfig(max_concurrency=...)` do not map directly to `s3transfer.manager.TransferConfig`, which expects `max_request_concurrency`.

### S3 Object Lambda Buckets Are Rejected

`TransferManager` explicitly blocks S3 Object Lambda resource ARNs and tells callers to use direct client calls instead.

### Credentials And Region Errors Come From The S3 Client

`NoCredentialsError`, bad region resolution, or endpoint mismatches are client-construction problems, not transfer-manager problems.

## Version-Sensitive Notes

- `0.16.0`: CRT transfer config gained support for `multipart_threshold`, `multipart_chunksize`, and `max_request_concurrency`
- `0.15.0`: multipart copy operations added stored-object ETag validation
- `0.14.0`: multipart download range validation was tightened
- `0.12.0`: Python 3.8 support ended, so `0.16.0` requires Python 3.9+

## Official Sources

- GitHub repository: https://github.com/boto/s3transfer
- PyPI package page: https://pypi.org/project/s3transfer/
- PyPI version metadata: https://pypi.org/pypi/s3transfer/0.16.0/json
- README: https://raw.githubusercontent.com/boto/s3transfer/develop/README.rst
- Changelog: https://github.com/boto/s3transfer/blob/develop/CHANGELOG.rst
- `TransferManager` source for `0.16.0`: https://raw.githubusercontent.com/boto/s3transfer/0.16.0/s3transfer/manager.py
- Stable wrapper reference: https://boto3.amazonaws.com/v1/documentation/api/latest/guide/s3.html
- Boto3 transfer customization reference: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/customizations/s3.html

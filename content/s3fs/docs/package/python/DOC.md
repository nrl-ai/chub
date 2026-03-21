---
name: package
description: "s3fs package guide for Python projects using Amazon S3 and S3-compatible object storage"
metadata:
  languages: "python"
  versions: "2026.2.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "s3fs,s3,fsspec,aws,storage,python"
---

# s3fs Python Package Guide

## What It Is

`s3fs` is the `fsspec` filesystem implementation for Amazon S3 and S3-compatible object stores. Use it when Python code needs filesystem-style operations such as listing objects, opening files, copying data, or passing `storage_options` into libraries like pandas.

## Installation

```bash
pip install s3fs==2026.2.0
```

```bash
conda install -c conda-forge s3fs
```

Project notes from the upstream package metadata and changelog:

- `2026.2.0` is the package version from PyPI.
- Python 3.9 support was dropped in `2025.12.0`; target Python 3.10+ for current releases.
- `s3fs` builds on `aiobotocore` under the hood, so async and credential behavior ultimately follows the botocore stack.

## Initialize A Filesystem

### Default AWS credential chain

If you do not pass credentials, `s3fs` uses the normal botocore credential resolution flow:

- environment variables such as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_SESSION_TOKEN`
- shared AWS config and credential files
- IAM roles or other ambient AWS credentials

```python
import s3fs

s3 = s3fs.S3FileSystem()
print(s3.ls("my-bucket"))
```

### Use a named AWS profile

```python
import s3fs

s3 = s3fs.S3FileSystem(profile="analytics")
```

### Pass explicit credentials

Use this for short-lived credentials in controlled code paths, not hard-coded long-term secrets.

```python
import s3fs

s3 = s3fs.S3FileSystem(
    key="AWS_ACCESS_KEY_ID",
    secret="AWS_SECRET_ACCESS_KEY",
    token="AWS_SESSION_TOKEN",
)
```

### Anonymous access for public buckets

```python
import s3fs

public_s3 = s3fs.S3FileSystem(anon=True)
print(public_s3.ls("landsat-pds"))
```

### S3-compatible endpoints such as MinIO

```python
import s3fs

s3 = s3fs.S3FileSystem(
    key="minioadmin",
    secret="minioadmin",
    endpoint_url="http://127.0.0.1:9000",
    client_kwargs={
        "region_name": "us-east-1",
    },
)
```

## Core Usage

### List, glob, and inspect objects

```python
import s3fs

s3 = s3fs.S3FileSystem()

print(s3.ls("my-bucket/prefix"))
print(s3.glob("my-bucket/prefix/*.parquet"))
print(s3.info("my-bucket/prefix/data.parquet"))
print(s3.exists("my-bucket/prefix/data.parquet"))
```

### Read and write files

Use binary modes with `s3.open()`.

```python
import json
import s3fs

s3 = s3fs.S3FileSystem()

record = {"id": 1, "status": "ok"}

with s3.open("my-bucket/output/result.json", "wb") as f:
    f.write(json.dumps(record).encode("utf-8"))

with s3.open("my-bucket/output/result.json", "rb") as f:
    loaded = json.loads(f.read().decode("utf-8"))

print(loaded)
```

### Transfer local files

```python
import s3fs

s3 = s3fs.S3FileSystem()

s3.put("local-report.csv", "my-bucket/reports/local-report.csv")
s3.get("my-bucket/reports/local-report.csv", "downloaded-report.csv")
s3.cp("my-bucket/reports/local-report.csv", "my-bucket/archive/local-report.csv")
s3.rm("my-bucket/archive/local-report.csv")
```

### Use `storage_options` with pandas

This is the common pattern when another library delegates I/O through `fsspec`.

```python
import pandas as pd

df = pd.read_parquet(
    "s3://my-bucket/datasets/events.parquet",
    storage_options={
        "profile": "analytics",
        "client_kwargs": {"region_name": "us-west-2"},
    },
)
```

## Configuration And Auth

`S3FileSystem` exposes several parameters that matter in real projects:

- `anon=True` for public data only.
- `profile="name"` to use a named AWS profile.
- `requester_pays=True` when reading Requester Pays buckets.
- `version_aware=True` only when you need S3 object versioning.
- `endpoint_url="..."` for S3-compatible stores such as MinIO or Ceph.
- `client_kwargs={...}` for botocore client options such as `region_name`.
- `config_kwargs={...}` for botocore `Config` values such as retries or signature behavior.
- `s3_additional_kwargs={...}` for per-request S3 options such as server-side encryption headers.

Example with region, retries, and SSE-S3:

```python
import s3fs

s3 = s3fs.S3FileSystem(
    profile="analytics",
    client_kwargs={"region_name": "us-west-2"},
    config_kwargs={"retries": {"max_attempts": 10, "mode": "standard"}},
    s3_additional_kwargs={"ServerSideEncryption": "AES256"},
)
```

If you need custom endpoints repeatedly, the upstream docs also support `FSSPEC_S3_ENDPOINT_URL`, `FSSPEC_S3_KEY`, and `FSSPEC_S3_SECRET` environment variables.

For distributed jobs, avoid shipping long-lived raw credentials between workers. The upstream docs point to `S3FileSystem.get_delegated_s3pars()` for temporary delegated credentials when you must fan work out across machines.

## Async Usage

`s3fs` is built on async infrastructure. For normal synchronous code, use the default API. For async applications, create the filesystem with `asynchronous=True`, initialize its session, and close it when finished.

```python
import asyncio
import s3fs

async def main() -> None:
    s3 = s3fs.S3FileSystem(asynchronous=True)
    await s3.set_session()
    try:
        listing = await s3._ls("my-bucket/prefix")
        print(listing)
    finally:
        await s3._session.close()

asyncio.run(main())
```

## Version-Sensitive Notes

- PyPI lists `2026.2.0` as the latest release, published on February 5, 2026.
- PyPI also shows the immediate prior releases `2026.1.0` on January 9, 2026 and `2025.12.0` on December 3, 2025.
- The published docs site currently renders with a `2025.12.0+dirty` banner, and its visible changelog navigation stops at `2025.12.0`. Before relying on behavior changes introduced in 2026, verify them against the PyPI release history or the upstream repository.
- Current PyPI metadata requires Python `>=3.10` and advertises classifiers through Python 3.14.

## Common Pitfalls

- `s3.open()` is primarily a binary file interface. Encode and decode text yourself, or let higher-level libraries like pandas handle it through `storage_options`.
- `version_aware=True` is not free. It switches some listings to version APIs, increases permission requirements, and can be much slower on large prefixes.
- On S3-compatible providers, failures are often endpoint or signature issues rather than path issues. Set `endpoint_url`, region, and any required signing config explicitly.
- `fork` is unsafe because of open sockets and the async background machinery used by `s3fs`. Use `spawn` or `forkserver` for multiprocessing.
- `anon=True` only works for genuinely public buckets and objects.
- If the bucket is Requester Pays, most read operations fail until you set `requester_pays=True`.

## Official Sources

- Docs: https://s3fs.readthedocs.io/en/latest/
- API reference: https://s3fs.readthedocs.io/en/latest/api.html
- PyPI: https://pypi.org/project/s3fs/
- Changelog source: https://raw.githubusercontent.com/fsspec/s3fs/main/docs/source/changelog.rst

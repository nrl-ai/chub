---
name: resumable-media
description: "Google low-level Python library for resumable uploads and media downloads over requests"
metadata:
  languages: "python"
  versions: "2.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,resumable-media,uploads,downloads,requests,google-auth,http,streaming"
---

# Google Resumable Media Python Library

## Golden Rule

Use `google-resumable-media` only when you need low-level control over upload or download HTTP flows. It is not a Google service client, it does not discover resources for you, and it does not build service-specific upload or media URLs.

For synchronous Python code, use the documented `google.resumable_media.requests` surface with a `google.auth.transport.requests.AuthorizedSession`.

One versioning caveat matters immediately: the official docs URL redirects to a rendered reference site that still labels `latest` as `2.4.1`, while PyPI and the official changelog show `2.8.0` as the current release. Use the rendered docs for class and method shape, then use PyPI and the changelog for post-`2.4.1` release facts.

## Install

For the documented sync transport:

```bash
python -m pip install "google-resumable-media[requests]==2.8.0" google-auth
```

If you are pinning without extras:

```bash
python -m pip install "google-resumable-media==2.8.0" requests google-auth
```

Common alternatives:

```bash
uv add "google-resumable-media==2.8.0"
poetry add "google-resumable-media==2.8.0"
```

PyPI also advertises an `aiohttp` extra:

```bash
python -m pip install "google-resumable-media[aiohttp]==2.8.0"
```

Treat async support as experimental and version-sensitive. The published reference pages in the official docs only document the `requests` transport.

## Authentication And Transport Setup

The official docs assume `requests` as the transport and `google-auth` for authenticated HTTP traffic. A normal Google API setup looks like this:

```python
from google.auth import default
from google.auth.transport.requests import AuthorizedSession

SCOPES = ("https://www.googleapis.com/auth/devstorage.read_write",)

credentials, _ = default(scopes=SCOPES)
transport = AuthorizedSession(credentials)
```

Practical rules:

- Choose scopes for the API you are actually calling.
- Build the upload URL or media URL for the target API yourself.
- Keep auth and URL construction in the calling code; this package only handles transfer mechanics.

## Core Usage

### Simple upload

Use `SimpleUpload` when the API accepts the full media body in a single request and there is no metadata part.

```python
from google.resumable_media.requests import SimpleUpload

upload_url = (
    "https://www.googleapis.com/upload/storage/v1/b/my-bucket/o"
    "?uploadType=media&name=hello.txt"
)

upload = SimpleUpload(upload_url)
response = upload.transmit(
    transport,
    b"hello world",
    "text/plain",
)
response.raise_for_status()
```

### Multipart upload

Use `MultipartUpload` when the API expects metadata plus content in one request.

The official class reference documents:

- `MultipartUpload(upload_url, headers=None, checksum=None)`
- `transmit(transport, data, metadata, content_type, timeout=(61, 60))`

```python
from google.resumable_media.requests import MultipartUpload

upload_url = "https://example.googleapis.com/upload/path?uploadType=multipart"
metadata = {"name": "example.json"}
data = b'{"hello": "world"}'

upload = MultipartUpload(upload_url)
response = upload.transmit(
    transport,
    data,
    metadata,
    "application/json",
)
response.raise_for_status()
```

### Resumable upload

Use `ResumableUpload` for large payloads, flaky networks, or cases where you need upload state to survive more than one HTTP request.

The official class reference documents:

- `ResumableUpload(upload_url, chunk_size, checksum=None, headers=None)`
- `initiate(transport, stream, metadata, content_type, total_bytes=None, stream_final=True, timeout=(61, 60))`
- `recover(transport)`
- `transmit_next_chunk(transport, timeout=(61, 60))`

```python
import io

from google.resumable_media.requests import ResumableUpload

payload = b"x" * (8 * 1024 * 1024)
stream = io.BytesIO(payload)
upload_url = (
    "https://www.googleapis.com/upload/storage/v1/b/my-bucket/o"
    "?uploadType=resumable&name=example.bin"
)

upload = ResumableUpload(upload_url, chunk_size=1024 * 1024)
response = upload.initiate(
    transport,
    stream,
    metadata={"name": "example.bin"},
    content_type="application/octet-stream",
    total_bytes=len(payload),
)
response.raise_for_status()

while not upload.finished:
    response = upload.transmit_next_chunk(transport)
    response.raise_for_status()
```

Call `recover(transport)` when the process boundary or network boundary is uncertain and you need to re-check the server-side upload state before sending more data.

### Chunked download

Use `ChunkedDownload` when the object is large, you do not want to buffer it all in memory, or you need ranged/chunked progress.

The official docs call out chunked downloads for very large objects or objects of unknown size and document:

- `ChunkedDownload(media_url, chunk_size, stream, start=0, end=None, headers=None)`
- `consume_next_chunk(transport, timeout=(61, 60))`

```python
from google.resumable_media.requests import ChunkedDownload

media_url = (
    "https://www.googleapis.com/download/storage/v1/b/"
    "my-bucket/o/example.bin?alt=media"
)

with open("example.bin", "wb") as fh:
    download = ChunkedDownload(
        media_url,
        chunk_size=50 * 1024 * 1024,
        stream=fh,
    )

    while not download.finished:
        response = download.consume_next_chunk(transport)
        response.raise_for_status()
```

### One-shot raw download

Use `RawDownload` when you want a single request and the raw `requests.Response` content path.

The official class reference documents:

- `RawDownload(media_url, stream=None, start=None, end=None, headers=None, checksum='md5')`
- `consume(transport, timeout=(61, 60))`

```python
from google.resumable_media.requests import RawDownload

download = RawDownload(
    "https://www.googleapis.com/download/storage/v1/b/"
    "my-bucket/o/example.bin?alt=media"
)
response = download.consume(transport)
response.raise_for_status()

payload = response.content
```

## Configuration Notes

### Timeouts

The published sync reference uses `timeout=(61, 60)` on `transmit`, `initiate`, `transmit_next_chunk`, `consume_next_chunk`, and `consume`. Pass `timeout=` explicitly when your environment needs different connect or read behavior.

### Checksums

The published constructors expose checksum controls on upload and raw-download helpers:

- `MultipartUpload(..., checksum=None)`
- `ResumableUpload(..., checksum=None)`
- `RawDownload(..., checksum='md5')`

The overview docs also say chunked downloads usually do not return a checksum, and even if they do, it is not validated during chunked transfer. If you need integrity validation for chunked downloads, validate after reassembling the full payload.

### URL construction

This package does not derive API-specific paths for you. Your caller code must provide the right URL shape, such as:

- `uploadType=media` for simple uploads
- `uploadType=multipart` for metadata + media uploads
- `uploadType=resumable` for resumable upload sessions
- `alt=media` for media downloads on APIs that use that query parameter

## Common Pitfalls

- Install name and import name differ. You install `google-resumable-media`, but the documented sync imports come from `google.resumable_media.requests`.
- This package is a transfer helper, not a product client. It will not list buckets, resolve object paths, or infer service endpoints.
- The official published reference is stale on version labeling. As of March 12, 2026, the rendered `latest` docs still show `2.4.1`, so use the changelog and PyPI for `2.7.x` and `2.8.0` release facts.
- The docs only clearly cover the `requests` surface. Do not assume sync examples translate directly to async code.
- Chunked downloads do not automatically give you end-to-end checksum validation for the whole object.

## Version-Sensitive Notes For 2.8.0

From the official changelog and PyPI metadata:

- `2.8.0` adds support for Python `3.13` and `3.14`.
- `2.7.2` fixes retry offset calculation for ranged reads.
- `2.7.1` adds a partial-response data check.
- `2.7.0` adds Python `3.12` support and brotli support.
- PyPI still lists the package requirement as Python `>=3.7`.

Practical implications:

- If you run Python `3.13` or `3.14`, use at least `2.8.0`.
- If you rely on ranged-read retries, pin at least `2.7.2`.
- If examples from the rendered docs conflict with newer behavior, prefer the changelog for post-`2.4.1` release notes and PyPI for release/version metadata.

## Official Sources Used

- Docs root from package metadata: https://googleapis.dev/python/google-resumable-media/latest/
- Canonical rendered docs root: https://docs.cloud.google.com/python/docs/reference/google-resumable-media/latest
- ResumableUpload reference: https://docs.cloud.google.com/python/docs/reference/google-resumable-media/latest/google.resumable_media.requests.ResumableUpload
- MultipartUpload reference: https://docs.cloud.google.com/python/docs/reference/google-resumable-media/latest/google.resumable_media.requests.MultipartUpload
- ChunkedDownload reference: https://docs.cloud.google.com/python/docs/reference/google-resumable-media/latest/google.resumable_media.requests.ChunkedDownload
- RawDownload reference: https://docs.cloud.google.com/python/docs/reference/google-resumable-media/latest/google.resumable_media.requests.RawDownload
- PyPI package page: https://pypi.org/project/google-resumable-media/
- Archived upstream repository: https://github.com/googleapis/google-resumable-media-python
- Upstream changelog: https://github.com/googleapis/google-resumable-media-python/blob/main/CHANGELOG.md

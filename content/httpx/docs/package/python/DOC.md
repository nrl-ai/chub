---
name: package
description: "HTTPX package guide for Python with sync and async clients, auth, transports, and 0.28.1 migration notes"
metadata:
  languages: "python"
  versions: "0.28.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "httpx,http,client,async,python,requests"
---

# HTTPX Python Package Guide

## When To Use HTTPX

Use `httpx` when Python code needs:

- a modern HTTP client with both sync and async APIs
- connection pooling, timeouts, streaming, redirects, cookies, and auth
- HTTP/2 support
- transport adapters for testing WSGI or ASGI apps in-process

`httpx` is the maintained successor-style choice when you want a `requests`-like API plus async support and more explicit client configuration.

## Install

```bash
pip install httpx==0.28.1
```

Common optional extras from the official package metadata:

```bash
pip install "httpx[http2]==0.28.1"
pip install "httpx[socks]==0.28.1"
pip install "httpx[brotli]==0.28.1"
pip install "httpx[zstd]==0.28.1"
```

Version-sensitive runtime note:

- PyPI metadata for `0.28.1` declares `Requires-Python >=3.8`.
- The current docs homepage now says Python `3.9+`.
- Treat the homepage as the latest upstream docs, not a strict guarantee for `0.28.1`. If you are pinned to `0.28.x`, prefer the package metadata and the `0.28.0` / `0.28.1` release notes for behavior changes.

## Golden Rules

- Prefer `httpx.Client()` or `httpx.AsyncClient()` for anything beyond a single one-off request.
- Reuse one client instead of creating a new client inside a hot loop.
- Call `response.raise_for_status()` when non-2xx responses should fail the workflow.
- Redirects are **not** followed unless you set `follow_redirects=True`.
- For `0.28.x`, use `proxy=` or `mounts=`. The old `proxies=` argument was removed.
- For app testing and in-process calls, use `WSGITransport` or `ASGITransport`. The old `app=` shortcut was removed.

## Basic Sync Usage

```python
import httpx

with httpx.Client(
    base_url="https://api.example.com",
    headers={"Accept": "application/json"},
    params={"api_version": "2026-01-01"},
    timeout=httpx.Timeout(10.0, connect=5.0),
    follow_redirects=True,
) as client:
    response = client.get("/items", params={"limit": 20})
    response.raise_for_status()
    items = response.json()
    print(items)
```

Useful client behavior:

- `base_url` lets you use relative paths like `"/items"`.
- Client-level headers, query params, and cookies merge with request-level values.
- A client keeps connections open and reuses them across requests.

## Sending JSON, Form Data, and Files

```python
import httpx

payload = {"name": "example", "enabled": True}

with httpx.Client() as client:
    response = client.post("https://api.example.com/items", json=payload)
    response.raise_for_status()
```

```python
import httpx

with open("report.csv", "rb") as f:
    files = {"file": ("report.csv", f, "text/csv")}
    response = httpx.post("https://api.example.com/upload", files=files)
    response.raise_for_status()
```

Use the request body arguments deliberately:

- `json=` for JSON bodies
- `data=` for HTML form data
- `files=` for multipart uploads
- `content=` for raw bytes or text when you need full control

## Async Usage

```python
import asyncio
import httpx

async def main() -> None:
    async with httpx.AsyncClient(
        base_url="https://api.example.com",
        timeout=httpx.Timeout(10.0, connect=5.0),
    ) as client:
        response = await client.get("/items")
        response.raise_for_status()
        print(response.json())

asyncio.run(main())
```

For streaming responses:

```python
import asyncio
import httpx

async def download(url: str, output_path: str) -> None:
    async with httpx.AsyncClient() as client:
        async with client.stream("GET", url) as response:
            response.raise_for_status()
            with open(output_path, "wb") as f:
                async for chunk in response.aiter_bytes():
                    f.write(chunk)

asyncio.run(download("https://example.com/large.bin", "large.bin"))
```

Async pitfalls:

- Keep one long-lived `AsyncClient` where possible.
- Do not instantiate clients repeatedly inside request handlers unless that is the intended lifecycle.
- If you use manual streaming mode, you must eventually call `Response.aclose()`.

## Authentication

Basic auth:

```python
import httpx

response = httpx.get(
    "https://api.example.com/me",
    auth=("username", "password"),
)
response.raise_for_status()
```

Client-scoped auth:

```python
import httpx

with httpx.Client(
    base_url="https://api.example.com",
    auth=("username", "password"),
) as client:
    response = client.get("/me")
    response.raise_for_status()
```

Upstream auth options to know about:

- Basic auth via a `(username, password)` tuple
- `httpx.DigestAuth(...)`
- `httpx.NetRCAuth()` for local `.netrc` credentials
- custom auth flows by subclassing `httpx.Auth`

For bearer tokens or API keys, set a header on the client:

```python
import httpx

token = "YOUR_TOKEN"

with httpx.Client(
    base_url="https://api.example.com",
    headers={"Authorization": f"Bearer {token}"},
) as client:
    response = client.get("/me")
    response.raise_for_status()
```

## Timeouts, Limits, and Retries

HTTPX enforces timeouts by default. Configure them explicitly for production code.

```python
import httpx

timeout = httpx.Timeout(10.0, connect=2.0, read=20.0, write=20.0, pool=5.0)
limits = httpx.Limits(max_connections=100, max_keepalive_connections=20)

with httpx.Client(timeout=timeout, limits=limits) as client:
    response = client.get("https://api.example.com/health")
    response.raise_for_status()
```

Notes:

- The docs describe HTTPX timeouts as inactivity timeouts, not a single total request deadline.
- Resource limits matter for high-concurrency code and async services.
- HTTPX does not provide the same built-in retry layer you may know from `urllib3`; implement retries at your application boundary or transport layer when needed.

## Proxies and Environment Variables

Explicit proxy:

```python
import httpx

with httpx.Client(proxy="http://localhost:8030") as client:
    response = client.get("https://example.com")
    response.raise_for_status()
```

Environment-driven proxy behavior is enabled by default via `trust_env=True`. Officially documented variables include:

- `HTTP_PROXY`
- `HTTPS_PROXY`
- `ALL_PROXY`
- `NO_PROXY`
- `SSL_CERT_FILE`
- `SSL_CERT_DIR`

Disable environment-derived proxy and certificate settings when you need deterministic behavior:

```python
import httpx

with httpx.Client(trust_env=False) as client:
    response = client.get("https://example.com")
    response.raise_for_status()
```

## SSL / TLS Configuration

For `0.28.x`, the release notes deprecate string-style `verify=` and the `cert=` shortcut. Prefer an explicit `ssl.SSLContext`.

```python
import ssl
import certifi
import httpx

ctx = ssl.create_default_context(cafile=certifi.where())

with httpx.Client(verify=ctx) as client:
    response = client.get("https://example.com")
    response.raise_for_status()
```

If you want system certificate stores on newer Python environments, the official SSL docs recommend `truststore`:

```python
import ssl
import httpx
import truststore

ctx = truststore.SSLContext(ssl.PROTOCOL_TLS_CLIENT)

with httpx.Client(verify=ctx) as client:
    response = client.get("https://example.com")
    response.raise_for_status()
```

## HTTP/2

Install the extra and enable it on the client:

```python
import httpx

with httpx.Client(http2=True) as client:
    response = client.get("https://example.com")
    print(response.http_version)
```

`http2=True` is not enough unless the `http2` extra is installed.

## Transports and In-Process App Calls

For tests and in-process service integration, use the transport APIs directly:

```python
import httpx
from starlette.applications import Starlette
from starlette.responses import JSONResponse
from starlette.routing import Route

async def homepage(request):
    return JSONResponse({"ok": True})

app = Starlette(routes=[Route("/", homepage)])
transport = httpx.ASGITransport(app=app)

async def main() -> None:
    async with httpx.AsyncClient(transport=transport, base_url="http://testserver") as client:
        response = await client.get("/")
        response.raise_for_status()
        print(response.json())
```

Use this pattern for `0.28.1`. The deprecated `app=` shortcut was removed in `0.28.0`.

## Error Handling

`httpx.HTTPError` is the broad top-level exception. The most useful subclasses in normal application code are:

- `httpx.HTTPStatusError` for `raise_for_status()` failures
- `httpx.RequestError` for transport-level failures
- `httpx.TimeoutException` and subclasses for timeout failures

```python
import httpx

try:
    response = httpx.get("https://api.example.com/items", timeout=5.0)
    response.raise_for_status()
except httpx.HTTPStatusError as exc:
    print("Bad status:", exc.response.status_code)
except httpx.TimeoutException:
    print("Request timed out")
except httpx.RequestError as exc:
    print("Transport error:", exc)
```

## Requests Compatibility Notes

HTTPX is intentionally similar to `requests`, but not identical:

- Redirects are off by default.
- `response.url` is an `httpx.URL` object, not a plain string.
- Cookies should normally live on the client, not be passed ad hoc on individual request calls.
- Streaming uses `Client.stream(...)` / `AsyncClient.stream(...)` instead of the exact `requests` interface.

If you are porting code from `requests`, check the compatibility guide before assuming every edge-case matches exactly.

## Version-Sensitive Notes For 0.28.1

- `0.28.0` removed the deprecated `proxies` argument. Use `proxy=` for a single proxy or `mounts=` for per-scheme routing.
- `0.28.0` removed the deprecated `app` argument. Use `WSGITransport` or `ASGITransport`.
- `0.28.0` deprecated string-style `verify=` and the `cert=` argument. Prefer `ssl.SSLContext`.
- `0.28.0` changed JSON request body serialization to a more compact representation.
- `0.28.1` fixed a bug involving `verify=False` together with client-side certificates.

## Official Sources

- Docs root: `https://www.python-httpx.org/`
- Quickstart: `https://www.python-httpx.org/quickstart/`
- Clients: `https://www.python-httpx.org/advanced/clients/`
- Authentication: `https://www.python-httpx.org/advanced/authentication/`
- Timeouts: `https://www.python-httpx.org/advanced/timeouts/`
- Resource limits: `https://www.python-httpx.org/advanced/resource-limits/`
- Proxies: `https://www.python-httpx.org/advanced/proxies/`
- SSL: `https://www.python-httpx.org/advanced/ssl/`
- Environment variables: `https://www.python-httpx.org/environment_variables/`
- Async support: `https://www.python-httpx.org/async/`
- HTTP/2: `https://www.python-httpx.org/http2/`
- Requests compatibility: `https://www.python-httpx.org/compatibility/`
- Exceptions: `https://www.python-httpx.org/exceptions/`
- Transports: `https://www.python-httpx.org/advanced/transports/`
- PyPI package metadata: `https://pypi.org/project/httpx/0.28.1/`
- Releases: `https://github.com/encode/httpx/releases/tag/0.28.0`
- Releases: `https://github.com/encode/httpx/releases/tag/0.28.1`

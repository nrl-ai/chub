---
name: package
description: "urllib3 package guide for Python: connection pooling, retries, TLS verification, proxies, and streaming HTTP"
metadata:
  languages: "python"
  versions: "2.6.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "urllib3,http,https,python,retries,tls,proxies"
---

# urllib3 Python Package Guide

## Golden Rule

Use a dedicated `urllib3.PoolManager(...)` in application code and reuse it. Use top-level `urllib3.request(...)` only for short scripts, CLIs, or REPL work.

`urllib3` 2.x is the low-level HTTP client layer focused on connection pooling, retries, TLS, proxies, and streaming. It is synchronous, thread-safe at the pool level, and a good fit when you want direct control over request behavior without a higher-level client wrapper.

## Version Snapshot

- Package: `urllib3`
- Ecosystem: `pypi`
- Version covered here: `2.6.3`
- Python requirement for 2.x: `3.9+`
- TLS baseline in 2.x: OpenSSL `1.1.1+`, TLS `1.2+` by default

## Installation

Install the package version you expect to run:

```bash
python -m pip install urllib3==2.6.3
```

Common optional extras:

```bash
python -m pip install "urllib3[socks]"
python -m pip install "urllib3[brotli]"
python -m pip install "urllib3[zstd]"
```

Use the `socks` extra only if you need SOCKS proxies. Prefer the official extras for compression support so you pick compatible decoder packages.

## Initialize a Reusable Client

Start with one long-lived `PoolManager`. It handles connection pooling and thread safety for you.

```python
import urllib3
from urllib3.util import Retry, Timeout

http = urllib3.PoolManager(
    num_pools=20,
    maxsize=10,
    block=True,
    timeout=Timeout(connect=2.0, read=10.0),
    retries=Retry(
        total=5,
        connect=3,
        read=2,
        status=3,
        backoff_factor=0.2,
        status_forcelist={429, 500, 502, 503, 504},
        allowed_methods={"DELETE", "GET", "HEAD", "OPTIONS", "PUT"},
        respect_retry_after_header=True,
    ),
    headers={
        "User-Agent": "my-service/1.0",
        "Accept": "application/json",
    },
)
```

Notes:

- `num_pools` controls how many host-specific pools are cached.
- `maxsize` controls how many reusable connections are kept per host.
- With `block=True`, at most `maxsize` connections are open per host; callers wait instead of creating unbounded extra connections.
- Pool-level `timeout` and `retries` become defaults and can still be overridden per request.

## Quick Requests

### GET with query parameters

For `GET`, `HEAD`, and `DELETE`, pass query params with `fields=...`:

```python
resp = http.request(
    "GET",
    "https://api.example.com/items",
    fields={"page": 1, "limit": 50},
)

print(resp.status)
data = resp.json()
```

### POST JSON

Use `json=...` instead of manually encoding JSON. `urllib3` sets `Content-Type: application/json` if you have not already set it.

```python
resp = http.request(
    "POST",
    "https://api.example.com/items",
    json={"name": "demo", "enabled": True},
)

payload = resp.json()
```

### Form-encoded POST

For `POST` or `PUT`, a dict in `fields=...` becomes form data:

```python
resp = http.request(
    "POST",
    "https://api.example.com/login",
    fields={"username": "alice", "password": "secret"},
)
```

### Raw bytes upload

```python
with open("report.pdf", "rb") as fp:
    resp = http.request(
        "PUT",
        "https://api.example.com/upload",
        body=fp.read(),
        headers={"Content-Type": "application/pdf"},
    )
```

## Response Handling

`request()` returns an `HTTPResponse`. The fields agents usually need are:

- `resp.status`
- `resp.headers`
- `resp.data`
- `resp.json()` for JSON responses

Example:

```python
resp = http.request("GET", "https://api.example.com/health")

if resp.status != 200:
    raise RuntimeError(f"unexpected status: {resp.status}")

print(resp.headers.get("Content-Type"))
print(resp.data)
```

Use `resp.json()` only when the response is actually JSON. Otherwise parse `resp.data` yourself.

## Timeouts and Retries

Always set explicit timeouts in production code.

```python
import urllib3

resp = http.request(
    "GET",
    "https://api.example.com/slow-endpoint",
    timeout=urllib3.Timeout(connect=1.0, read=5.0),
)
```

Retry guidance:

- `urllib3` retries idempotent methods by default.
- Passing an integer like `retries=3` mainly covers connection errors and redirects.
- Use `urllib3.util.Retry(...)` when you need status-code-based retry behavior, `Retry-After` support, or custom allowed methods.
- Do not blindly retry non-idempotent operations like `POST` unless the upstream API explicitly guarantees safety.

## Streaming Large Responses

Use `preload_content=False` for large or unknown-size responses. When you do this, you must release the connection back to the pool.

```python
resp = http.request(
    "GET",
    "https://api.example.com/export",
    preload_content=False,
)

try:
    with open("export.bin", "wb") as fp:
        while chunk := resp.read(64 * 1024):
            fp.write(chunk)
finally:
    resp.release_conn()
```

If you stop reading early, call `resp.drain_conn()` before `resp.release_conn()` so the connection can be safely reused.

## Headers and Basic Authentication

`urllib3` does not maintain a higher-level auth system. In most cases you send auth as headers.

Bearer token example:

```python
resp = http.request(
    "GET",
    "https://api.example.com/me",
    headers={"Authorization": f"Bearer {token}"},
)
```

Basic auth helper:

```python
import urllib3

headers = urllib3.util.make_headers(basic_auth="username:password")
resp = http.request("GET", "https://api.example.com/me", headers=headers)
```

## TLS Verification and Certificates

HTTPS certificate verification is on by default in modern urllib3. Leave it on unless you are debugging a trusted internal environment.

Cross-platform CA bundle example with `certifi`:

```python
import certifi
import urllib3

http = urllib3.PoolManager(
    cert_reqs="CERT_REQUIRED",
    ca_certs=certifi.where(),
)
```

Private CA bundle:

```python
http = urllib3.PoolManager(
    cert_reqs="CERT_REQUIRED",
    ca_certs="/path/to/ca-bundle.pem",
)
```

Client certificate / mutual TLS:

```python
http = urllib3.PoolManager(
    cert_file="/path/to/client-cert.pem",
    key_file="/path/to/client-key.pem",
    key_password="optional-password",
    cert_reqs="CERT_REQUIRED",
    ca_certs="/path/to/ca-bundle.pem",
)
```

If you must connect to an older server, you can lower the minimum TLS version explicitly, but treat that as a compatibility exception, not the default.

## Proxies

### HTTP or HTTPS proxy

```python
import urllib3

proxy = urllib3.ProxyManager("http://proxy.internal:8080")
resp = proxy.request("GET", "https://api.example.com/data")
```

If your proxy only speaks HTTP, the proxy URL must start with `http://`, not `https://`.

For HTTPS proxies, `use_forwarding_for_https=True` gives the proxy full visibility into HTTPS requests. Use it only with trusted corporate proxies.

### SOCKS proxy

Install the extra first:

```bash
python -m pip install "urllib3[socks]"
```

Then:

```python
from urllib3.contrib.socks import SOCKSProxyManager

proxy = SOCKSProxyManager("socks5h://proxy.example.com:1080")
resp = proxy.request("GET", "https://api.example.com/data")
```

Prefer `socks5h://` or `socks4a://` so DNS resolution happens on the proxy side instead of the client side.

## Common Pitfalls

- Do not build application code around top-level `urllib3.request(...)` if shared process state matters. It uses a module-global pool.
- Do not forget `resp.release_conn()` when using `preload_content=False`.
- For `POST` and `PUT`, query parameters are not auto-encoded from `fields`; encode them into the URL yourself if the API expects query params.
- Cookies sent via the `Cookie` header are stripped on redirects to a different host.
- Prefer `resp.headers` and `resp.headers.get(...)` over `getheaders()` / `getheader()`. Those methods were removed in `2.6.0` and restored in `2.6.1`.
- Do not disable certificate verification in production just to make TLS errors disappear. Fix CA bundles, hostnames, proxy config, or TLS versions instead.

## Version-Sensitive Notes for 2.x

- `urllib3` 2.x requires Python `3.9+`.
- `urllib3` 2.x requires OpenSSL `1.1.1+` and defaults to TLS `1.2+`.
- `urllib3.request(...)` and `HTTPResponse.json()` are available in 2.x and are useful for scripts, but `PoolManager` is still the safer default for services.
- `2.6.3` includes a security fix for redirects plus streamed compressed responses and caps `Retry-After` values above 6 hours by default.
- `2.6.0` tightened compressed-response decoding. If you use Brotli support, prefer the official `urllib3[brotli]` extra and keep Brotli libraries current.
- HTTP/2 support exists in 2.x but is still marked experimental upstream. Do not assume it is a drop-in replacement for mature HTTP/1.1 production paths.

## Migration Clues

If you are upgrading from 1.26.x and hit errors, these usually indicate a dependency or import-path problem rather than a broken urllib3 install:

- `ImportError: cannot import name 'DEFAULT_CIPHERS' from 'urllib3.util.ssl_'`
- `AttributeError: module 'urllib3.connectionpool' has no attribute 'VerifiedHTTPSConnection'`
- `AttributeError: 'HTTPResponse' object has no attribute 'strict'`

When these appear, check whether another dependency in the environment still expects urllib3 1.26.x or old private import paths.

## Official Sources

- Docs root: https://urllib3.readthedocs.io/en/stable/
- User guide: https://urllib3.readthedocs.io/en/stable/user-guide.html
- Advanced usage: https://urllib3.readthedocs.io/en/stable/advanced-usage.html
- API reference: https://urllib3.readthedocs.io/en/stable/reference/index.html
- v2 migration guide: https://urllib3.readthedocs.io/en/2.6.3/v2-migration-guide.html
- Changelog: https://urllib3.readthedocs.io/en/2.6.3/changelog.html
- PyPI: https://pypi.org/project/urllib3/

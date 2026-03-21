---
name: package
description: "Requests HTTP client for Python with practical guidance for sessions, auth, TLS, proxies, and common pitfalls"
metadata:
  languages: "python"
  versions: "2.32.5"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "requests,http,python,client,auth,tls,proxies"
---

# Requests Python Package Guide

## What It Is

`requests` is the standard synchronous HTTP client used in many Python projects. It is a blocking HTTP/1.1 client with:

- per-method helpers like `get`, `post`, `put`, and `delete`
- `Session` objects for shared headers, cookies, and connection pooling
- built-in Basic and Digest auth support
- TLS verification, proxies, multipart uploads, and streamed downloads

Use it when you need straightforward synchronous HTTP calls from Python code.

## Version Covered

- Package: `requests`
- Ecosystem: `pypi`
- Version: `2.32.5`
- Python support: `>=3.9`
- Registry: https://pypi.org/project/requests/
- Docs root used for this guide: https://requests.readthedocs.io/en/stable/

## Install

```bash
python -m pip install requests==2.32.5
```

Optional SOCKS proxy support:

```bash
python -m pip install "requests[socks]==2.32.5"
```

## Core Usage

Prefer the method helpers (`get`, `post`, `put`, `patch`, `delete`) and always set a timeout explicitly.

```python
import requests

response = requests.get(
    "https://api.github.com/events",
    params={"per_page": 10},
    timeout=(3.05, 30),
)
response.raise_for_status()

events = response.json()
print(events[0]["type"])
```

Notes:

- `timeout` can be a float or a `(connect_timeout, read_timeout)` tuple.
- `response.json()` can still succeed on an HTTP error response. Check `status_code` or call `raise_for_status()`.
- If you omit `timeout`, Requests will wait indefinitely.

### POST JSON

Use `json=` for JSON APIs instead of manually serializing `data=`.

```python
import requests

payload = {"name": "example", "enabled": True}

response = requests.post(
    "https://httpbin.org/post",
    json=payload,
    timeout=30,
)
response.raise_for_status()

print(response.json()["json"])
```

`json=` is ignored if you also pass `data=` or `files=`.

### Form Data And Files

```python
import requests

with open("report.csv", "rb") as fh:
    response = requests.post(
        "https://httpbin.org/post",
        data={"source": "daily-job"},
        files={"file": ("report.csv", fh, "text/csv")},
        timeout=30,
    )
    response.raise_for_status()
```

## Sessions And Shared Configuration

Use `Session` when you make multiple calls to the same service. A session persists cookies, reuses TCP connections through `urllib3`, and lets you centralize headers, auth, or TLS settings.

```python
import requests

with requests.Session() as session:
    session.headers.update(
        {
            "User-Agent": "my-service/1.0",
            "Accept": "application/json",
        }
    )
    session.params = {"api_version": "2025-01-01"}

    response = session.get(
        "https://httpbin.org/get",
        timeout=(3.05, 30),
    )
    response.raise_for_status()
```

Use a session for:

- shared headers or auth across many requests
- cookie persistence
- lower latency when repeatedly calling the same host

## Authentication

### Basic Auth

```python
import requests

response = requests.get(
    "https://httpbin.org/basic-auth/user/pass",
    auth=("user", "pass"),
    timeout=30,
)
response.raise_for_status()
```

### Digest Auth

```python
import requests
from requests.auth import HTTPDigestAuth

response = requests.get(
    "https://httpbin.org/digest-auth/auth/user/pass",
    auth=HTTPDigestAuth("user", "pass"),
    timeout=30,
)
response.raise_for_status()
```

### `netrc`

If you do not pass `auth=`, Requests can load Basic Auth credentials from `~/.netrc`, `~/_netrc`, or the file pointed to by `NETRC`.

Disable that behavior when you need predictable credentials or want to avoid environment-derived auth:

```python
import requests

with requests.Session() as session:
    session.trust_env = False
    response = session.get("https://example.com", timeout=30)
```

## TLS, Certificates, And Proxies

### TLS Verification

TLS verification is enabled by default and should stay enabled in normal code.

```python
import requests

response = requests.get(
    "https://example.com",
    verify="/path/to/ca-bundle.pem",
    timeout=30,
)
response.raise_for_status()
```

Useful options:

- `verify=True`: default behavior
- `verify="/path/to/ca-bundle.pem"`: trust a custom CA bundle
- `verify=False`: only for local testing; this disables certificate and hostname checks

Requests also honors:

- `REQUESTS_CA_BUNDLE`
- `CURL_CA_BUNDLE`

Requests uses `certifi` for trusted CA roots.

### Client Certificates

```python
import requests

response = requests.get(
    "https://example.com",
    cert=("/path/client.cert", "/path/client.key"),
    timeout=30,
)
response.raise_for_status()
```

The client private key must be unencrypted.

### Proxies

Per-request proxy configuration is safer than relying on `session.proxies`, because environment proxy settings can override session-level proxy values.

```python
import requests

proxies = {
    "http": "http://proxy.internal:3128",
    "https": "http://proxy.internal:3128",
}

response = requests.get(
    "https://example.com",
    proxies=proxies,
    timeout=30,
)
response.raise_for_status()
```

Requests also reads standard environment variables:

- `HTTP_PROXY`
- `HTTPS_PROXY`
- `ALL_PROXY`
- `NO_PROXY`

For SOCKS proxies, install `requests[socks]` and use `socks5://...` or `socks5h://...`. `socks5h` resolves DNS on the proxy side.

## Streaming Downloads And Responses

Use `stream=True` for large responses or streaming APIs.

```python
import requests

with requests.get("https://httpbin.org/stream/20", stream=True, timeout=30) as response:
    response.raise_for_status()

    if response.encoding is None:
        response.encoding = "utf-8"

    for line in response.iter_lines(decode_unicode=True):
        if line:
            print(line)
```

For file downloads:

```python
import requests

with requests.get("https://example.com/archive.tar.gz", stream=True, timeout=30) as response:
    response.raise_for_status()
    with open("archive.tar.gz", "wb") as fh:
        for chunk in response.iter_content(chunk_size=1024 * 1024):
            if chunk:
                fh.write(chunk)
```

Important details:

- `iter_content()` is preferred over `response.raw` for most code.
- With `stream=True`, consume the body or close the response, or the connection will not be returned to the pool.
- `iter_lines()` is not reentrant safe. Create the iterator once and share that iterator if multiple consumers are involved.

## Error Handling

Catch `RequestException` for broad transport errors and narrower subclasses when you need to branch.

```python
import requests
from requests.exceptions import HTTPError, JSONDecodeError, RequestException, Timeout

try:
    response = requests.get("https://api.github.com/events", timeout=(3.05, 30))
    response.raise_for_status()
    data = response.json()
except Timeout:
    print("The server was too slow to respond.")
except HTTPError as exc:
    print(f"HTTP error: {exc.response.status_code}")
except JSONDecodeError:
    print("The response body was not valid JSON.")
except RequestException as exc:
    print(f"Network or Requests error: {exc}")
```

Common exception families to remember:

- `RequestException`: base class for Requests errors
- `Timeout`: request took too long
- `ConnectionError`: DNS, TCP, or connection failures
- `HTTPError`: raised by `raise_for_status()`
- `JSONDecodeError`: invalid JSON in `response.json()`
- `SSLError`: TLS verification or certificate problems

## Common Pitfalls

- Always set `timeout`. Requests does not choose one for you.
- Do not treat `response.json()` as an HTTP success check.
- Do not use `verify=False` in production.
- Do not assume Requests is non-blocking; response reading is blocking even when using streaming APIs.
- Do not rely on `session.proxies` if environment proxies may be present.
- When you hand-build a `PreparedRequest`, merge environment settings if you still need env-based proxies or CA bundle behavior.
- If you partially read a streamed response and never close it, connection reuse suffers.
- Custom `Authorization` headers can be overridden by `.netrc` or `auth=`.

## Version-Sensitive Notes For 2.32.x

- `2.32.5` is the current PyPI release as of 2026-03-11. It reverted the SSLContext caching behavior introduced in `2.32.0`, added Python `3.14` support, and dropped Python `3.8`.
- `2.32.4` fixed `CVE-2024-47081`, where a malicious URL combined with trusted environment settings could pull `.netrc` credentials for the wrong host. If you are pinned below `2.32.4`, upgrade or set `session.trust_env = False` where appropriate.
- `2.32.0` and `2.32.1` were yanked on PyPI because of issues related to the `CVE-2024-35195` mitigation. Avoid pinning those versions.
- If you subclass `HTTPAdapter`, review the `2.32.2` change that introduced `get_connection_with_tls_context` and deprecated `get_connection` for adapters affected by the 2.32.0 TLS-related changes.
- Requests `2.30.0+` supports `urllib3 2.x`. If an older project is tightly pinned to `urllib3 1.x`, test upgrades carefully.

## Official Sources

- Stable docs: https://requests.readthedocs.io/en/stable/
- Quickstart: https://requests.readthedocs.io/en/stable/user/quickstart/
- Advanced usage: https://requests.readthedocs.io/en/stable/user/advanced/
- Authentication: https://requests.readthedocs.io/en/stable/user/authentication/
- API reference: https://requests.readthedocs.io/en/stable/api/
- PyPI: https://pypi.org/project/requests/
- Releases: https://github.com/psf/requests/releases

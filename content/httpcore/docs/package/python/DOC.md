---
name: package
description: "httpcore package guide for Python - low-level sync and async HTTP transport, pooling, streaming, and proxy support"
metadata:
  languages: "python"
  versions: "1.0.9"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "httpcore,http,http2,asyncio,trio,networking,proxy,transport"
---

# httpcore Python Package Guide

## What It Is

`httpcore` is Encode's low-level HTTP transport library. Use it when you need direct control over connection pooling, sync or async request execution, streaming bodies, proxy configuration, SSL contexts, or transport-level retries.

Do not reach for `httpcore` if you want a batteries-included client. It deliberately does not handle redirects, cookies, auth helpers, JSON helpers, multipart abstractions, or environment-based defaults. If you need those higher-level features, use `httpx` instead.

## Installation

Base install for HTTP/1.1:

```bash
python -m pip install "httpcore==1.0.9"
```

Optional extras from the maintainer docs:

```bash
# Async support on asyncio.
python -m pip install "httpcore[asyncio]==1.0.9"

# Async support on trio.
python -m pip install "httpcore[trio]==1.0.9"

# HTTP/2 support.
python -m pip install "httpcore[http2]==1.0.9"

# SOCKS proxy support.
python -m pip install "httpcore[socks]==1.0.9"
```

You can combine extras:

```bash
python -m pip install "httpcore[asyncio,http2,socks]==1.0.9"
```

## Choose The Right Entry Point

- `httpcore.request(...)`: one-shot synchronous request helper. Fine for scripts or tests.
- `httpcore.stream(...)`: one-shot synchronous streaming response helper.
- `httpcore.ConnectionPool(...)`: the default choice for reusable synchronous transport.
- `httpcore.AsyncConnectionPool(...)`: the async transport for `asyncio` or `trio`.

In real code, prefer a pool over repeated top-level `request()` calls. That is where `httpcore` actually gives you connection reuse and transport-level configuration.

## Quick Start

### One synchronous request

```python
import httpcore

response = httpcore.request("GET", "https://www.example.com/")

print(response)
print(response.status)
print(response.headers)
print(response.content)
```

`response.headers` is low-level header data, not a rich mapping object. Expect raw header pairs and decode or normalize them yourself if your code expects higher-level client behavior.

### Reuse a synchronous connection pool

```python
import httpcore

with httpcore.ConnectionPool() as pool:
    response = pool.request(
        "GET",
        "https://api.example.com/items",
        headers={"Accept": "application/json"},
    )
    print(response.status)
    print(response.content)
```

### Reuse an asynchronous connection pool

Install either `httpcore[asyncio]` or `httpcore[trio]` first.

```python
import asyncio
import httpcore

async def main() -> None:
    async with httpcore.AsyncConnectionPool() as pool:
        response = await pool.request(
            "GET",
            "https://api.example.com/items",
            headers={"Accept": "application/json"},
        )
        print(response.status)
        print(await response.aread())

asyncio.run(main())
```

## Request Model

The request surface is deliberately small:

- `method`: `str` or `bytes`
- `url`: `str`, `bytes`, or `httpcore.URL`
- `headers`: dict or list of header tuples
- `content`: `bytes` or an iterator of `bytes`
- `extensions`: a dict for transport-level options such as timeouts or tracing

Synchronous POST example:

```python
import json

import httpcore

payload = {"name": "example", "enabled": True}

with httpcore.ConnectionPool() as pool:
    response = pool.request(
        "POST",
        "https://api.example.com/items",
        headers={
            "Accept": "application/json",
            "Content-Type": "application/json",
            "Authorization": "Bearer TOKEN",
        },
        content=json.dumps(payload).encode("utf-8"),
    )
    print(response.status)
    print(response.content)
```

Header keys and values may be strings or bytes. If you pass strings, keep them ASCII-only. For non-ASCII values, encode explicitly and pass bytes.

For async requests with a body, pass either `bytes` or an async iterable yielding byte chunks.

## Streaming Responses

Use streaming when the response body may be large or when you want backpressure instead of buffering the whole body in memory.

### One-shot streaming

```python
import httpcore

with httpcore.stream("GET", "https://example.com/large-download") as response:
    with open("download.bin", "wb") as output:
        for chunk in response.iter_stream():
            output.write(chunk)
```

### Streaming through a pool

```python
import httpcore

with httpcore.ConnectionPool() as pool:
    with pool.stream("GET", "https://example.com/large-download") as response:
        with open("download.bin", "ab") as output:
            for chunk in response.iter_stream():
                output.write(chunk)
```

### Async streaming

```python
import asyncio
import httpcore
from pathlib import Path

async def main() -> None:
    path = Path("download.bin")
    async with httpcore.AsyncConnectionPool() as pool:
        async with pool.stream("GET", "https://example.com/large-download") as response:
            with path.open("ab") as output:
                async for chunk in response.aiter_stream():
                    output.write(chunk)

asyncio.run(main())
```

`stream()` is also the safest cleanup pattern because the context manager guarantees the response is closed even if your processing exits early.

## Connection Pool Configuration

The most important pool knobs are:

- `max_connections`: upper bound on concurrent open connections
- `max_keepalive_connections`: upper bound on idle reusable connections
- `keepalive_expiry`: idle keep-alive lifetime in seconds
- `http1` / `http2`: allowed protocol versions
- `retries`: connection-establishment retries
- `local_address`: bind to a local IP or force IPv4/IPv6 family
- `uds`: connect through a Unix domain socket instead of TCP
- `socket_options`: socket options applied when establishing TCP connections

Example:

```python
import httpcore

with httpcore.ConnectionPool(
    max_connections=50,
    max_keepalive_connections=20,
    keepalive_expiry=30.0,
    retries=2,
    http1=True,
    http2=False,
) as pool:
    response = pool.request("GET", "https://example.com/health")
    print(response.status)
```

Practical guidance:

- Start with the defaults unless you have a real concurrency bottleneck.
- Increase `max_connections` only when you expect actual concurrent work.
- If you see `PoolTimeout`, either your pool is too small or responses are not being closed quickly enough.
- Use `local_address="0.0.0.0"` or `"::"` when you need to force IPv4 or IPv6 source addressing.
- Use `uds="/path/to/socket.sock"` only for services that explicitly expose an HTTP-over-UDS endpoint.

## HTTP/2

HTTP/2 support is optional and disabled by default.

1. Install the extra:

```bash
python -m pip install "httpcore[http2]==1.0.9"
```

2. Enable it on the pool:

```python
import httpcore

with httpcore.ConnectionPool(http2=True) as pool:
    response = pool.request("GET", "https://example.com/")
    print(response.http_version)
```

Keep `http1=True` unless you specifically need to disable HTTP/1.1 fallback.

## Proxies

`httpcore` supports HTTP proxies, HTTPS proxies, and SOCKS proxies.

### HTTP proxy

```python
import httpcore

proxy = httpcore.Proxy("http://127.0.0.1:8080/")

with httpcore.ConnectionPool(proxy=proxy) as pool:
    response = pool.request("GET", "https://example.com/")
    print(response.status)
```

### Proxy authentication

```python
import httpcore

proxy = httpcore.Proxy(
    url="http://127.0.0.1:8080/",
    auth=("username", "password"),
)

with httpcore.ConnectionPool(proxy=proxy) as pool:
    response = pool.request("GET", "https://example.com/")
    print(response.status)
```

### HTTPS proxy with custom TLS configuration

```python
import ssl

import httpcore

proxy_ssl_context = ssl.create_default_context(cafile="/path/to/proxy-ca.pem")
proxy = httpcore.Proxy(
    url="https://proxy.internal:8443/",
    ssl_context=proxy_ssl_context,
)

with httpcore.ConnectionPool(proxy=proxy) as pool:
    response = pool.request("GET", "https://example.com/")
    print(response.status)
```

### SOCKS proxy

Install the extra first:

```bash
python -m pip install "httpcore[socks]==1.0.9"
```

Then configure the proxy URL:

```python
import httpcore

proxy = httpcore.Proxy("socks5://127.0.0.1:1080")

with httpcore.ConnectionPool(proxy=proxy) as pool:
    response = pool.request("GET", "https://example.com/")
    print(response.status)
```

## SSL, Certificates, And mTLS

Pass an `ssl.SSLContext` when you need custom CA bundles, stricter verification settings, or client certificates.

```python
import ssl

import httpcore

ssl_context = ssl.create_default_context(cafile="/path/to/internal-ca.pem")

with httpcore.ConnectionPool(ssl_context=ssl_context) as pool:
    response = pool.request("GET", "https://internal.example.com/")
    print(response.status)
```

Mutual TLS:

```python
import ssl

import httpcore

ssl_context = ssl.create_default_context()
ssl_context.load_cert_chain("client-cert.pem", "client-key.pem")

with httpcore.ConnectionPool(ssl_context=ssl_context) as pool:
    response = pool.request("GET", "https://mtls.example.com/")
    print(response.status)
```

## Auth And Headers

`httpcore` does not implement auth workflows for you. There are no session objects, auth plugins, cookie jars, or automatic token refresh hooks.

In practice you add headers directly:

```python
import httpcore

headers = {
    "Authorization": "Bearer TOKEN",
    "Accept": "application/json",
    "User-Agent": "my-service/1.0",
}

with httpcore.ConnectionPool() as pool:
    response = pool.request(
        "GET",
        "https://api.example.com/private",
        headers=headers,
    )
    print(response.status)
```

If your code needs auth choreography, cookie persistence, or redirect policy, `httpx` is usually the better layer.

## Timeouts And Other Extensions

The `extensions` dict is where `httpcore` exposes transport-level controls.

### Timeout extension

```python
import httpcore

timeouts = {
    "connect": 5.0,
    "read": 10.0,
    "write": 10.0,
    "pool": 5.0,
}

with httpcore.ConnectionPool() as pool:
    response = pool.request(
        "GET",
        "https://api.example.com/items",
        extensions={"timeout": timeouts},
    )
    print(response.status)
```

### Trace extension

Use tracing when you need to debug DNS, connect, TLS, or pool behavior.

```python
import httpcore

def trace(event_name, info) -> None:
    print(event_name, info)

response = httpcore.request(
    "GET",
    "https://www.example.com/",
    extensions={"trace": trace},
)
```

For broader debug output, the maintainer docs also show standard-library logging against the `httpcore` logger namespace.

## Exceptions You Will Actually Handle

The official exception tree is small and useful:

- `httpcore.TimeoutException`
  - `httpcore.PoolTimeout`
  - `httpcore.ConnectTimeout`
  - `httpcore.ReadTimeout`
  - `httpcore.WriteTimeout`
- `httpcore.NetworkError`
  - `httpcore.ConnectError`
  - `httpcore.ReadError`
  - `httpcore.WriteError`
- `httpcore.ProtocolError`
  - `httpcore.RemoteProtocolError`
  - `httpcore.LocalProtocolError`
- `httpcore.ProxyError`
- `httpcore.UnsupportedProtocol`

Typical handling:

```python
import httpcore

try:
    with httpcore.ConnectionPool() as pool:
        response = pool.request(
            "GET",
            "https://example.com/",
            extensions={
                "timeout": {
                    "connect": 2.0,
                    "read": 5.0,
                    "write": 5.0,
                    "pool": 1.0,
                }
            },
        )
        print(response.status)
except httpcore.ConnectTimeout:
    print("Connection attempt timed out")
except httpcore.PoolTimeout:
    print("Pool exhausted before a connection became available")
except httpcore.ReadTimeout:
    print("Response body was too slow")
except httpcore.RemoteProtocolError as exc:
    print(f"Upstream sent an invalid HTTP response: {exc}")
except httpcore.ProxyError as exc:
    print(f"Proxy negotiation failed: {exc}")
```

## Common Pitfalls

- Do not expect `requests` or `httpx` ergonomics. `httpcore` is a transport layer, not a general-purpose user client.
- Install extras explicitly. `asyncio`, `trio`, HTTP/2, and SOCKS support are not all available from the base install.
- Use `AsyncConnectionPool` only from async code and only after installing the right async extra.
- Close what you open. Long-lived pools should be context-managed or explicitly closed, and streaming responses must be consumed or closed.
- Expect low-level header handling. Response headers are raw tuples, not a case-insensitive mapping with convenience helpers.
- Remember that top-level `request()` is convenience only. Repeated calls lose the connection reuse that makes `httpcore` valuable.
- Set timeouts yourself. The extension-based timeout model is explicit; if you want higher-level timeout policy, wrap `httpcore` in your own helper.
- Use bytes for non-ASCII header values. String header values are ASCII-only.

## Version-Sensitive Notes For 1.0.9

- This guide targets `httpcore==1.0.9`, which is also the current latest PyPI release as of March 12, 2026.
- `1.0.9` shipped on April 24, 2025 and updated the `h11` dependency to address `GHSA-vqfr-h8mv-ghfj`.
- `1.0.8` fixed an `AttributeError` when importing on Python 3.14. If you are supporting Python 3.14, stay on at least `1.0.8`.
- `1.0.7` added direct `proxy=` support on `ConnectionPool()` and `AsyncConnectionPool()`. If you are pinned below `1.0.7`, re-check proxy examples before copying them.
- Since `1.0.0`, async support has been optional and must be enabled with `httpcore[asyncio]` or `httpcore[trio]`.
- The maintainer docs site is rolling `1.0.x` documentation rather than a version-pinned snapshot, so release notes are the authority for point-release-specific changes.

## Official Sources

- Docs root: https://www.encode.io/httpcore/
- Quickstart: https://www.encode.io/httpcore/quickstart/
- Connection pools: https://www.encode.io/httpcore/connection-pools/
- Async support: https://www.encode.io/httpcore/async/
- Proxies: https://www.encode.io/httpcore/proxies/
- HTTP/2: https://www.encode.io/httpcore/http2/
- Extensions: https://www.encode.io/httpcore/extensions/
- Exceptions: https://www.encode.io/httpcore/exceptions/
- Logging: https://www.encode.io/httpcore/logging/
- PyPI: https://pypi.org/project/httpcore/
- Source repository: https://github.com/encode/httpcore
- Releases: https://github.com/encode/httpcore/releases

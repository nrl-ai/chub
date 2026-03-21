---
name: package
description: "httptools package guide for Python HTTP/1.1 request parsing and URL parsing"
metadata:
  languages: "python"
  versions: "0.7.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "httptools,http,http1,url-parser,async,python"
---

# httptools Python Package Guide

## What It Is

`httptools` is a low-level Python binding around Node's `llhttp` parser. Use it when you need incremental HTTP/1.1 request parsing or fast URL parsing inside server, proxy, or framework code.

Use something higher-level if you need an HTTP client, ASGI/WSGI framework, TLS handling, retries, cookies, or authentication helpers. `httptools` only parses bytes and calls your callbacks.

## Installation

```bash
pip install httptools
```

Pin the documented version when behavior or wheel availability matters:

```bash
pip install "httptools==0.7.1"
```

With Poetry:

```bash
poetry add httptools
```

With uv:

```bash
uv add httptools
```

## Core Setup

`HttpRequestParser` takes a protocol-like object. The parser calls methods on that object as it sees request data. The official README shows these callback names:

- `on_message_begin()`
- `on_url(url: bytes)`
- `on_header(name: bytes, value: bytes)`
- `on_headers_complete()`
- `on_body(body: bytes)`
- `on_message_complete()`

Any of these methods can be omitted if you do not need them.

```python
import httptools

class RequestProtocol:
    def __init__(self) -> None:
        self.url = b""
        self.headers = []
        self.body_parts = []

    def on_url(self, url: bytes) -> None:
        self.url = url

    def on_header(self, name: bytes, value: bytes) -> None:
        self.headers.append((name, value))

    def on_body(self, body: bytes) -> None:
        self.body_parts.append(body)

    def on_message_complete(self) -> None:
        body = b"".join(self.body_parts)
        print("url =", self.url.decode("ascii", errors="replace"))
        print("headers =", [(k.decode(), v.decode()) for k, v in self.headers])
        print("body =", body.decode("utf-8", errors="replace"))

protocol = RequestProtocol()
parser = httptools.HttpRequestParser(protocol)

parser.feed_data(
    b"POST /users?id=1 HTTP/1.1\r\n"
    b"Host: example.com\r\n"
    b"Content-Type: application/json\r\n"
    b"Content-Length: 17\r\n"
    b"\r\n"
    b'{\"name\":\"alice\"}'
)
```

## Incremental Parsing

The parser is designed for chunked input. Feed it bytes exactly as you receive them from the socket or transport:

```python
import httptools

class Protocol:
    def on_message_begin(self) -> None:
        self.body = bytearray()

    def on_body(self, body: bytes) -> None:
        self.body.extend(body)

    def on_message_complete(self) -> None:
        print("complete body:", bytes(self.body))

parser = httptools.HttpRequestParser(Protocol())

for chunk in [
    b"POST /upload HTTP/1.1\r\nHost: example.com\r\nContent-L",
    b"ength: 11\r\n\r\nhello ",
    b"world",
]:
    parser.feed_data(chunk)
```

Wrap `feed_data()` in `try/except httptools.HttpParserError` if you need to reject malformed requests cleanly:

```python
import httptools

parser = httptools.HttpRequestParser(object())

try:
    parser.feed_data(b"broken request")
except httptools.HttpParserError as exc:
    print(f"parse failed: {exc}")
```

## URL Parsing

`parse_url()` is useful when you need the path, query string, host, or port without building your own splitter. It returns a parsed URL object whose string-like fields are bytes.

```python
import httptools

parsed = httptools.parse_url(b"/users/list?active=1")

print(parsed.path)   # b"/users/list"
print(parsed.query)  # b"active=1"
```

For absolute URLs:

```python
import httptools

parsed = httptools.parse_url(b"http://example.com:8080/users?id=1")

print(parsed.schema)  # b"http"
print(parsed.host)    # b"example.com"
print(parsed.port)    # 8080
print(parsed.path)    # b"/users"
print(parsed.query)   # b"id=1"
```

## Configuration And Auth

`httptools` has no authentication, session, retry, or transport configuration layer.

The practical configuration surface is:

- the callback methods you implement on the protocol object
- how you buffer partial request state between callbacks
- how your server or transport layer handles sockets, TLS, backpressure, and connection lifecycles

If you need request routing, middleware, header normalization, or response generation, pair `httptools` with a higher-level server stack instead of trying to add those concerns inside the parser callbacks.

## Common Pitfalls

- Pass `bytes` to `feed_data()`, not `str`.
- Callback arguments such as URL parts, header names, and header values are `bytes`; decode them explicitly when your application needs text.
- Treat parser callbacks as streaming events. Do not assume the request body arrives in one `on_body()` call.
- Keep request state on a per-connection or per-request object. Reset or replace it after `on_message_complete()`.
- Handle parser failures explicitly with `httptools.HttpParserError` if malformed input should become a `400` or connection close in your server.
- `httptools` is for HTTP/1.x parsing. It is not an HTTP client and it does not implement HTTP/2 or HTTP/3.

## Version-Sensitive Notes

- PyPI shows `0.7.1` as the current package version and requires Python `>=3.8`.
- The `v0.7.1` release added Python 3.14 wheels and fixed Python 3.13 free-threaded wheel builds.
- The `v0.7.0` release added wheels for Python 3.13 free-threaded, Windows ARM64, and macOS arm64.
- If your environment does not have a compatible wheel for the target runtime and platform, `pip` may need to build from source. Check the PyPI files list for the version you are deploying.

## Official Sources

- GitHub repository and README: https://github.com/MagicStack/httptools
- PyPI package page: https://pypi.org/project/httptools/
- GitHub release `v0.7.1`: https://github.com/MagicStack/httptools/releases/tag/v0.7.1
- GitHub release `v0.7.0`: https://github.com/MagicStack/httptools/releases/tag/v0.7.0

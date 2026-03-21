---
name: package
description: "h11 package guide for Python - sans-I/O HTTP/1.1 protocol state machine"
metadata:
  languages: "python"
  versions: "0.16.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "h11,http,http1.1,protocol,sans-io,networking"
---

# h11 Python Package Guide

## Golden Rule

Use `h11` when you need to parse or serialize raw HTTP/1.1 on top of your own transport layer.

`h11` is a low-level, bring-your-own-I/O state machine. It does not open sockets, perform TLS, retry requests, manage connection pools, follow redirects, or implement cookies and auth for you.

If you need a normal HTTP client or server framework, use something higher level and treat `h11` as an internal protocol layer.

## Install

```bash
pip install h11==0.16.0
```

```bash
poetry add h11==0.16.0
```

```bash
uv add h11==0.16.0
```

## Core Model

`h11` turns HTTP/1.1 bytes into typed events and turns typed events back into bytes.

```python
import h11

conn = h11.Connection(our_role=h11.CLIENT)
```

You drive it with four operations:

1. Read bytes from your socket, stream, or async transport.
2. Feed them into `conn.receive_data(...)`.
3. Pull parsed events with `conn.next_event()`.
4. Build outgoing events and serialize them with `conn.send(...)`.

Important event types:

- `Request`
- `Response`
- `InformationalResponse`
- `Data`
- `EndOfMessage`
- `ConnectionClosed`

Important sentinels and states:

- `NEED_DATA`: read more bytes before calling `next_event()` again
- `PAUSED`: stop reading until your application advances the connection cycle
- `IDLE`, `SEND_BODY`, `SEND_RESPONSE`, `DONE`, `MUST_CLOSE`

## Initialize And Configure

The main constructor accepts your role and an optional receive-buffer limit:

```python
import h11

conn = h11.Connection(
    our_role=h11.SERVER,
    max_incomplete_event_size=16 * 1024,
)
```

`max_incomplete_event_size` limits how much incomplete request-line/response-line and header data h11 buffers before failing. Increase it only if you expect unusually large headers, because larger values reduce protection against slowloris-style or otherwise malformed input.

## Minimal HTTPS Request

This follows the upstream "getting started" flow, but as a compact script.

```python
import socket
import ssl

import h11

ctx = ssl.create_default_context()
sock = ctx.wrap_socket(
    socket.create_connection(("httpbin.org", 443)),
    server_hostname="httpbin.org",
)

conn = h11.Connection(our_role=h11.CLIENT)

sock.sendall(
    conn.send(
        h11.Request(
            method="GET",
            target="/get",
            headers=[
                ("Host", "httpbin.org"),
                ("User-Agent", "context-hub-example/1.0"),
                ("Accept", "application/json"),
            ],
        )
    )
)
sock.sendall(conn.send(h11.EndOfMessage()))

status_code = None
headers = {}
body = bytearray()

while True:
    data = sock.recv(8192)
    conn.receive_data(data)

    while True:
        event = conn.next_event()

        if event is h11.NEED_DATA:
            break

        if isinstance(event, h11.Response):
            status_code = event.status_code
            headers = {
                name.decode("ascii"): value.decode("ascii")
                for name, value in event.headers
            }
        elif isinstance(event, h11.Data):
            body.extend(event.data)
        elif isinstance(event, h11.EndOfMessage):
            sock.close()
            print(status_code)
            print(headers["content-type"])
            print(body.decode("utf-8"))
            raise SystemExit
        elif isinstance(event, h11.ConnectionClosed):
            raise RuntimeError("peer closed before EndOfMessage")

    if data == b"":
        raise RuntimeError("unexpected EOF")
```

## Sending Request Or Response Bodies

When you send a body, emit one or more `Data(...)` events and always finish with `EndOfMessage()`.

```python
import h11

conn = h11.Connection(our_role=h11.SERVER)
payload = b"hello"

raw_bytes = b"".join(
    [
        conn.send(
            h11.Response(
                status_code=200,
                headers=[
                    ("Content-Length", str(len(payload))),
                    ("Content-Type", "text/plain; charset=utf-8"),
                ],
            )
        ),
        conn.send(h11.Data(data=payload)),
        conn.send(h11.EndOfMessage()),
    ]
)
```

If you forget framing headers such as `Content-Length` when they are required, h11 will raise a `LocalProtocolError` when you try to send body data.

## Keep-Alive And Connection Reuse

`h11` models one HTTP request/response exchange at a time. Reuse the same TCP connection only after both sides are done:

```python
if conn.our_state is h11.DONE and conn.their_state is h11.DONE:
    conn.start_next_cycle()
```

`start_next_cycle()` resets both sides to `IDLE`. If you call it before both sides reach `DONE`, h11 raises `LocalProtocolError`.

## Auth And Higher-Level Configuration

`h11` has no auth subsystem. Authentication is just headers you construct yourself:

```python
headers = [
    ("Host", "api.example.com"),
    ("Authorization", f"Bearer {token}"),
    ("Accept", "application/json"),
]
```

You are responsible for all higher-level behavior around h11:

- TCP or Unix socket setup
- TLS contexts and certificate verification
- timeouts and cancellation
- retries and reconnects
- connection pooling
- proxy handling
- redirects
- compression/decompression
- cookie jars and session state

## Common Pitfalls

### `h11` is not an HTTP client library

Do not expect `h11` to behave like `requests`, `httpx`, or `urllib3`. It only understands protocol events and framing.

### Always send `EndOfMessage()`

Even when h11 serializes it as `b""`, your application should still send the `EndOfMessage()` event so the state machine stays correct.

### Drain `next_event()` until `NEED_DATA` or `PAUSED`

One read from the network can yield multiple events. Do not assume a single `recv()` call maps to a single HTTP event.

### Call `receive_data(b"")` on EOF

EOF is meaningful input to h11. Feed `b""` when the transport closes so it can emit `ConnectionClosed` or finish parsing EOF-terminated cases correctly.

### HTTP/1.1 requests need exactly one `Host` header

Creating a `Request` with HTTP/1.1 and no `Host` header raises `LocalProtocolError`. Multiple `Host` headers are also invalid.

### Header names and values are normalized

When constructing events, h11 accepts ASCII strings or bytes-like values, normalizes header names to lowercase bytes internally, and validates whitespace and illegal characters. When reading events back, response/request headers are exposed as bytes pairs.

### Write failures must poison the state machine

If `conn.send(...)` produced bytes but your transport failed before they were actually sent, call `conn.send_failed()` and stop reusing that `Connection`.

### Upgrades and tunneling need a handoff

If you implement `Upgrade` or `CONNECT`, expect `PAUSED` and protocol-switch states such as `MIGHT_SWITCH_PROTOCOL`. At that point you must hand the transport and any trailing bytes to the next protocol layer yourself.

## Version-Sensitive Notes For `0.16.0`

- This entry targets `h11==0.16.0`.
- The docs URL points at `/latest/`, which currently renders `0.16.0+dev` docs. For package-accurate behavior, use `https://h11.readthedocs.io/en/v0.16.0/`.
- `0.16.0` added a security fix that rejects certain malformed `Transfer-Encoding: chunked` bodies that were previously accepted, closing a request-smuggling risk in some proxy or load-balancer deployments.
- `0.15.0`, which is included in the upgrade path to `0.16.0`, rejects absurdly large `Content-Length` values earlier instead of attempting full integer parsing.
- Current maintainer docs and PyPI metadata both describe `h11` as requiring Python `3.8+`.

## Official Sources Used

- `https://h11.readthedocs.io/en/latest/`
- `https://h11.readthedocs.io/en/v0.16.0/`
- `https://h11.readthedocs.io/en/stable/basic-usage.html`
- `https://h11.readthedocs.io/en/stable/api.html`
- `https://h11.readthedocs.io/en/v0.16.0/changes.html`
- `https://pypi.org/project/h11/`

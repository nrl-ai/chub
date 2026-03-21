---
name: package
description: "websocket-client package guide for Python synchronous WebSocket connections, callbacks, proxies, and reconnect handling"
metadata:
  languages: "python"
  versions: "1.9.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "websocket,python,realtime,networking,client"
---

# websocket-client Python Package Guide

## Golden Rule

Install `websocket-client`, but import `websocket` in code.

This library is for synchronous WebSocket clients:

- Use `create_connection()` or `websocket.WebSocket()` for short-lived request/response exchanges.
- Use `websocket.WebSocketApp` for long-lived, callback-driven connections.

If the project is built around `asyncio`, do not treat `websocket-client` as an asyncio-native client. Upstream explicitly documents compatibility limitations with `asyncio`.

## Install

```bash
pip install websocket-client==1.9.0
```

Optional extras from upstream:

```bash
pip install "websocket-client[optional]"
pip install rel
```

- `websocket-client[optional]` adds `python-socks` for proxy support and `wsaccel` for a small performance boost.
- `rel` is useful if you want `WebSocketApp.run_forever(..., reconnect=...)` with automatic reconnect dispatching.
- Upstream `1.9.0` removes Python `3.8` support. PyPI classifiers for this release cover Python `3.9` through `3.13`.

## Choose The Right API

### Short-lived connection

Use `create_connection()` when you need to connect, send one or a few messages, read responses, then close.

```python
import json
from websocket import create_connection

ws = create_connection(
    "wss://example.com/ws",
    timeout=10,
    header=[
        "Authorization: Bearer YOUR_TOKEN",
        "X-Client: my-app",
    ],
    subprotocols=["json"],
)

try:
    ws.send(json.dumps({"type": "ping"}))
    reply = ws.recv()
    print(reply)
finally:
    ws.close()
```

### Long-lived callback app

Use `WebSocketApp` when the server pushes events and you need callbacks for open, reconnect, messages, errors, and close.

```python
import json
import os

import websocket

def build_headers():
    return {
        "Authorization": f"Bearer {os.environ['WS_TOKEN']}",
        "X-Client": "my-app",
    }

def on_open(ws):
    ws.send(json.dumps({"type": "subscribe", "channel": "updates"}))

def on_reconnect(ws):
    ws.send(json.dumps({"type": "subscribe", "channel": "updates"}))

def on_message(ws, message):
    print("message:", message)

def on_error(ws, error):
    raise error

def on_close(ws, status_code, message):
    print("closed:", status_code, message)

app = websocket.WebSocketApp(
    "wss://example.com/ws",
    header=build_headers,
    subprotocols=["json"],
    on_open=on_open,
    on_reconnect=on_reconnect,
    on_message=on_message,
    on_error=on_error,
    on_close=on_close,
)

app.run_forever(
    ping_interval=30,
    ping_timeout=10,
)
```

`WebSocketApp.header` can be a callable in `1.9.0`, which is useful when auth headers depend on fresh state during reconnects.

## Core Operations

### Send and receive binary data

For higher-level usage, `WebSocket.recv()` gives you the next message as text or bytes. If you need explicit frame metadata, use `send_binary()` and `recv_data()`.

```python
import websocket

ws = websocket.create_connection("wss://example.com/ws")

try:
    ws.send_binary(b"\x01\x02\x03")
    opcode, data = ws.recv_data()
    print(opcode, data)
finally:
    ws.close()
```

### Set a global default timeout

`create_connection(..., timeout=...)` sets the socket timeout directly. `WebSocketApp.run_forever()` uses the global default timeout for connection setup, so set one with `websocket.setdefaulttimeout()` if needed.

```python
import websocket

websocket.setdefaulttimeout(5)

app = websocket.WebSocketApp("wss://example.com/ws")
app.run_forever()
```

## Connection Setup And Auth

Common knobs on `create_connection()` and `WebSocketApp.run_forever()`:

- `header` for custom handshake headers
- `cookie` for cookie-based sessions
- `subprotocols` for negotiated subprotocols
- `origin` or `suppress_origin` to control the Origin header
- `host` or `suppress_host` to control the Host header
- `timeout` for socket timeouts
- `sslopt` for TLS behavior
- `http_proxy_host`, `http_proxy_port`, `http_proxy_auth`, `http_no_proxy`, `proxy_type` for proxy routing
- `skip_utf8_validation=True` for performance-sensitive cases

### Header-based auth

```python
from websocket import create_connection

ws = create_connection(
    "wss://example.com/ws",
    header=[
        "Authorization: Bearer YOUR_TOKEN",
        "X-API-Key: YOUR_KEY",
    ],
)
```

### Cookie-based auth

```python
import websocket

app = websocket.WebSocketApp(
    "wss://example.com/ws",
    cookie="sessionid=abc123; csrftoken=def456",
)
```

### Subprotocol negotiation

```python
from websocket import create_connection

ws = create_connection(
    "wss://example.com/ws",
    subprotocols=["graphql-transport-ws"],
)
```

### Host and Origin overrides

Use `origin`, `suppress_origin`, `host`, or `suppress_host` only when the upstream server or proxy requires them.

```python
from websocket import create_connection

ws = create_connection(
    "wss://example.com/ws",
    origin="https://app.example.com",
    host="backend.example.internal",
)
```

## Reconnect, Ping, And Liveness

`WebSocketApp.run_forever()` is the main long-running loop.

Important behavior from upstream:

- `ping_interval=0` disables periodic pings.
- `ping_timeout` must be greater than `0`.
- If both are set, `ping_interval` must be greater than `ping_timeout`.
- `reconnect` sets the delay between reconnect attempts.
- Reconnect behavior is for unexpected disconnects, not graceful server closes.
- For practical reconnect loops, upstream recommends using a dispatcher such as `rel`.

Example:

```python
import rel
import websocket

def on_open(ws):
    ws.send("subscribe")

def on_reconnect(ws):
    ws.send("subscribe")

def on_message(ws, message):
    print(message)

app = websocket.WebSocketApp(
    "wss://example.com/ws",
    on_open=on_open,
    on_reconnect=on_reconnect,
    on_message=on_message,
)

app.run_forever(
    dispatcher=rel,
    reconnect=5,
    ping_interval=30,
    ping_timeout=10,
)
rel.signal(2, rel.abort)
rel.dispatch()
```

Reconnections create a fresh TCP and WebSocket session. Re-send subscriptions, auth, or resume tokens in `on_open` and `on_reconnect`.

## Proxies And TLS

### Proxy support

`websocket-client` supports:

- `http`
- `socks4`
- `socks4a`
- `socks5`
- `socks5h`

If you need DNS resolution through the proxy, use `socks4a` or `socks5h`. Keep `proxy_type` lowercase.

```python
import websocket

app = websocket.WebSocketApp("wss://example.com/ws")
app.run_forever(
    http_proxy_host="127.0.0.1",
    http_proxy_port=8080,
    proxy_type="http",
    http_proxy_auth=("user", "pass"),
    http_no_proxy=["localhost", "127.0.0.1"],
)
```

In `1.9.0`, upstream removed `localhost` and `127.0.0.1` from the default `NO_PROXY` list. If local endpoints should bypass the proxy, pass `http_no_proxy` explicitly or set matching environment variables.

### TLS options

For local development against self-signed certs, upstream documents:

```python
import ssl
from websocket import create_connection

ws = create_connection(
    "wss://localhost:8443/ws",
    sslopt={"cert_reqs": ssl.CERT_NONE},
)
```

To disable hostname verification:

```python
import websocket

ws = websocket.create_connection(
    "wss://localhost:8443/ws",
    sslopt={"check_hostname": False},
)
```

Treat both as development-only settings.

## Debugging And Error Handling

### Enable protocol tracing

```python
import websocket

websocket.enableTrace(True)
```

This is the fastest way to inspect the handshake, frames, close status, ping/pong traffic, and proxy behavior.

### Surface callback failures

```python
def on_error(ws, err):
    raise err
```

If you hit `WebSocketConnectionClosedException: Connection is already closed.`:

- Stop sending on that socket.
- Reconnect with `connect()` or `create_connection()`.
- For `WebSocketApp`, re-enter `run_forever()` or use the reconnect path.

## Common Pitfalls

- `pip install websocket-client` but `import websocket`
- This is a synchronous client library, not an asyncio-native one
- The package does not support the `permessage-deflate` compression extension (RFC 7692)
- `run_forever(reconnect=...)` is not enough by itself; for automatic reconnects you should also supply a dispatcher such as `rel`
- Reconnects start a fresh connection; they do not resume stream state unless the server has its own resume protocol
- `keep_running` on `WebSocketApp.__init__` is obsolete and ignored
- When you instantiate `websocket.WebSocket()` directly and use threads, keep `enable_multithread=True`; that has been the default since `1.1.0`, but older examples on the internet may assume otherwise
- Some `on_close` timing issues are easier to avoid by closing from a separate thread or timer instead of directly inside the message callback
- `skip_utf8_validation=True` can improve throughput, but only use it if you trust the server payloads or you are handling binary/text boundaries carefully

## Version-Sensitive Notes

- `1.9.0` removes Python `3.8` support and adds Python `3.13`
- `1.9.0` removes `localhost` and `127.0.0.1` from the default `NO_PROXY` handling, which can change local dev behavior behind proxies
- `1.9.0` documents `suppress_host` for both connection setup and `run_forever()`; do not assume that option exists in `1.8.0`
- `1.8.0` introduced `on_reconnect`; older callback examples may not use it even though it is the right hook for resubscription logic now

## Official Sources

- Docs root: `https://websocket-client.readthedocs.io/en/latest/`
- Installation: `https://websocket-client.readthedocs.io/en/latest/installation.html`
- Getting started: `https://websocket-client.readthedocs.io/en/latest/getting_started.html`
- Examples: `https://websocket-client.readthedocs.io/en/latest/examples.html`
- Threading: `https://websocket-client.readthedocs.io/en/latest/threading.html`
- FAQ: `https://websocket-client.readthedocs.io/en/latest/faq.html`
- `WebSocketApp` API: `https://websocket-client.readthedocs.io/en/latest/app.html`
- Core API: `https://websocket-client.readthedocs.io/en/latest/core.html`
- PyPI: `https://pypi.org/project/websocket-client/`
- PyPI JSON metadata: `https://pypi.org/pypi/websocket-client/json`
- Upstream changelog: `https://raw.githubusercontent.com/websocket-client/websocket-client/v1.9.0/ChangeLog`

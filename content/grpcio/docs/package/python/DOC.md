---
name: package
description: "gRPC Python runtime package for generated stubs, clients, servers, TLS credentials, metadata, retries, and AsyncIO APIs"
metadata:
  languages: "python"
  versions: "1.78.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "grpcio,grpc,rpc,protobuf,http2,asyncio"
---

# grpcio Python Package Guide

Use `grpcio` when you need to run Python gRPC clients or servers. The installed package is `grpcio`, but the runtime import is `grpc`.

## Golden Rule

- Install `grpcio` for the runtime and `grpcio-tools` only when you need to generate Python code from `.proto` files.
- Prefer `grpc.secure_channel(...)` in real deployments. Reserve `grpc.insecure_channel(...)` for local development, trusted internal networks, or tests.
- Keep the generated `_pb2.py` and `_pb2_grpc.py` files checked in or regenerated in CI. They are part of your application contract, not transient artifacts.

## Install

Runtime only:

```bash
python -m pip install grpcio==1.78.0
```

Runtime plus code generation:

```bash
python -m pip install grpcio==1.78.0 grpcio-tools
```

Verify the runtime version in code:

```python
import grpc

print(grpc.__version__)
```

## What `grpcio` Gives You

- Channel creation: `grpc.insecure_channel(...)`, `grpc.secure_channel(...)`
- Server creation: `grpc.server(...)`
- TLS and call credentials helpers such as `grpc.ssl_channel_credentials(...)` and `grpc.composite_channel_credentials(...)`
- Interceptors via `grpc.intercept_channel(...)`
- AsyncIO client and server APIs under `grpc.aio`

If you use protocol buffers, your usual workflow is:

1. Define the service in a `.proto`.
2. Generate Python files.
3. Build a server by subclassing the generated `*Servicer`.
4. Build clients from the generated `*Stub`.

## Generate Python Code From `.proto`

The Python basics guide uses `grpcio-tools` to generate the protobuf module and the gRPC module:

```bash
python -m grpc_tools.protoc \
  -I./protos \
  --python_out=. \
  --pyi_out=. \
  --grpc_python_out=. \
  ./protos/helloworld.proto
```

This produces:

- `helloworld_pb2.py`: protobuf messages and descriptors
- `helloworld_pb2_grpc.py`: gRPC-specific client and server helpers

The generated gRPC module imports the protobuf module, so both files need to stay importable together.

## Core Sync Usage

### Minimal server

```python
from concurrent import futures

import grpc

import helloworld_pb2
import helloworld_pb2_grpc

class Greeter(helloworld_pb2_grpc.GreeterServicer):
    def SayHello(self, request, context):
        return helloworld_pb2.HelloReply(message=f"Hello, {request.name}")

def serve() -> None:
    server = grpc.server(
        futures.ThreadPoolExecutor(max_workers=10),
        options=[
            ("grpc.keepalive_time_ms", 60_000),
            ("grpc.keepalive_timeout_ms", 20_000),
        ],
        maximum_concurrent_rpcs=1000,
    )
    helloworld_pb2_grpc.add_GreeterServicer_to_server(Greeter(), server)
    server.add_insecure_port("[::]:50051")
    server.start()
    server.wait_for_termination()

if __name__ == "__main__":
    serve()
```

### Minimal client

```python
import json

import grpc

import helloworld_pb2
import helloworld_pb2_grpc

service_config = json.dumps(
    {
        "methodConfig": [
            {
                "name": [{"service": "helloworld.Greeter"}],
                "timeout": "5s",
                "waitForReady": True,
            }
        ]
    }
)

channel = grpc.insecure_channel(
    "dns:///localhost:50051",
    options=[("grpc.service_config", service_config)],
)
stub = helloworld_pb2_grpc.GreeterStub(channel)

try:
    reply = stub.SayHello(
        helloworld_pb2.HelloRequest(name="Ada"),
        timeout=5,
        metadata=(("x-request-id", "req-123"),),
        wait_for_ready=True,
    )
    print(reply.message)
except grpc.RpcError as exc:
    print(exc.code(), exc.details())
```

## Streaming RPC Shapes

The Python basics tutorial maps the four RPC shapes like this:

- Unary-unary: return one response object.
- Unary-stream: `yield` zero or more response objects.
- Stream-unary: accept a request iterator and return one response object.
- Stream-stream: accept a request iterator and `yield` responses as they become available.

Example server-streaming handler:

```python
def ListFeatures(self, request, context):
    for feature in self._lookup(request):
        yield feature
```

Example client-streaming handler:

```python
def RecordRoute(self, request_iterator, context):
    count = sum(1 for _ in request_iterator)
    return route_pb2.RouteSummary(point_count=count)
```

## AsyncIO API

Use `grpc.aio` when the surrounding application is already built on AsyncIO. The official AsyncIO docs call this API stable, but also note an important caveat: gRPC AsyncIO objects may only be used on the thread where they were created.

### Async client

```python
import asyncio

import grpc

import helloworld_pb2
import helloworld_pb2_grpc

async def main() -> None:
    async with grpc.aio.insecure_channel("localhost:50051") as channel:
        stub = helloworld_pb2_grpc.GreeterStub(channel)

        try:
            reply = await stub.SayHello(
                helloworld_pb2.HelloRequest(name="Ada"),
                timeout=5,
                metadata=(("x-request-id", "req-123"),),
            )
            print(reply.message)
        except grpc.aio.AioRpcError as exc:
            print(exc.code(), exc.details())

asyncio.run(main())
```

### Async server

```python
import asyncio

import grpc

import helloworld_pb2
import helloworld_pb2_grpc

class Greeter(helloworld_pb2_grpc.GreeterServicer):
    async def SayHello(self, request, context):
        return helloworld_pb2.HelloReply(message=f"Hello, {request.name}")

async def serve() -> None:
    server = grpc.aio.server(
        options=[("grpc.keepalive_time_ms", 60_000)],
        maximum_concurrent_rpcs=1000,
    )
    helloworld_pb2_grpc.add_GreeterServicer_to_server(Greeter(), server)
    server.add_insecure_port("[::]:50051")
    await server.start()
    await server.wait_for_termination()

asyncio.run(serve())
```

Async streaming rules that matter in real code:

- Do not share `grpc.aio` channels, calls, or server objects across threads.
- On AsyncIO streaming calls, read operations must be serialized and write operations must be serialized.
- Do not mix the iterator API with explicit `read()` / `write()` calls on the same RPC.

## Auth, TLS, and Metadata

### TLS channel

```python
import grpc

with open("ca.pem", "rb") as fh:
    root_certs = fh.read()

channel_credentials = grpc.ssl_channel_credentials(root_certificates=root_certs)
channel = grpc.secure_channel("api.example.com:443", channel_credentials)
```

### TLS plus bearer token call credentials

```python
import grpc

channel_credentials = grpc.ssl_channel_credentials()
call_credentials = grpc.access_token_call_credentials("YOUR_ACCESS_TOKEN")
credentials = grpc.composite_channel_credentials(
    channel_credentials,
    call_credentials,
)

channel = grpc.secure_channel("api.example.com:443", credentials)
```

Other credential helpers worth knowing:

- `grpc.metadata_call_credentials(...)`: build call credentials from a metadata plugin
- `grpc.composite_call_credentials(...)`: combine multiple call credentials
- `grpc.ssl_server_credentials(...)`: TLS on the server side
- `grpc.local_channel_credentials(...)` and `grpc.local_server_credentials(...)`: local-only credentials for localhost or UDS testing

Important auth notes:

- The auth guide warns that Google credentials should only be sent to Google services.
- Local channel credentials are useful for tests because they allow call credentials without `insecure_channel`, but local TCP is still not encrypted. UDS is the safer local transport.
- Metadata keys must be ASCII and must not start with `grpc-`.
- Metadata values can be ASCII or binary. Servers may cap request headers; the metadata guide suggests assuming about 8 KiB by default.

Per-call metadata example:

```python
reply = stub.SayHello(
    helloworld_pb2.HelloRequest(name="Ada"),
    metadata=(
        ("authorization", "Bearer YOUR_ACCESS_TOKEN"),
        ("x-request-id", "req-123"),
    ),
)
```

## Service Config, Retries, and Keepalive

`grpcio` exposes channel and server configuration through `options=[("key", value), ...]`. The Python API docs point to gRPC core channel arguments, and the service-config guide documents the JSON model used for client-side behavior.

Useful channel args:

- `grpc.service_config`: default service config as JSON
- `grpc.keepalive_time_ms`
- `grpc.keepalive_timeout_ms`
- `grpc.keepalive_permit_without_calls`
- `grpc.max_receive_message_length`
- `grpc.max_send_message_length`

Example with service config:

```python
import json
import grpc

service_config = json.dumps(
    {
        "loadBalancingConfig": [{"round_robin": {}}],
        "methodConfig": [
            {
                "name": [{"service": "helloworld.Greeter"}],
                "timeout": "5s",
                "waitForReady": True,
                "retryPolicy": {
                    "maxAttempts": 4,
                    "initialBackoff": "0.1s",
                    "maxBackoff": "1s",
                    "backoffMultiplier": 2,
                    "retryableStatusCodes": ["UNAVAILABLE"],
                },
            }
        ],
    }
)

channel = grpc.insecure_channel(
    "dns:///greeter.internal:50051",
    options=[("grpc.service_config", service_config)],
)
```

Behavior to remember:

- Service config can control load balancing, wait-for-ready, timeouts, retries, hedging, and health checking.
- Retries are enabled by default, but there is no default retry policy. Without one, you only get transparent retries for narrowly defined low-level failures.
- Once response headers arrive, the RPC is committed and gRPC will not retry it.
- Keepalive should be coordinated with the service owner. The keepalive guide warns against enabling keepalive without active calls and against setting client keepalive much below one minute.
- If a server dislikes your keepalive settings, it can send `GOAWAY` with debug data `too_many_pings`.

## Interceptors

Use interceptors when you need a reusable cross-cutting layer for auth headers, tracing, logging, or client-side retries that sit above the generated stubs.

Sync client interceptors wrap a channel:

```python
channel = grpc.intercept_channel(base_channel, my_interceptor)
stub = helloworld_pb2_grpc.GreeterStub(channel)
```

For AsyncIO, pass interceptors directly to `grpc.aio.insecure_channel(...)` or `grpc.aio.secure_channel(...)`.

## Error Handling

For sync code, non-OK RPCs surface as `grpc.RpcError`.

```python
try:
    reply = stub.SayHello(helloworld_pb2.HelloRequest(name="Ada"), timeout=5)
except grpc.RpcError as exc:
    if exc.code() is grpc.StatusCode.UNAVAILABLE:
        ...
```

For AsyncIO, await calls can raise `grpc.aio.AioRpcError`.

On the server side, use the `context` object to read deadlines and set status details, codes, and trailing metadata when you need to return a structured failure.

## Common Pitfalls

- `pip install grpcio`, but `import grpc`.
- `grpcio` does not generate code by itself. Use `grpcio-tools` and commit or regenerate the `_pb2.py` / `_pb2_grpc.py` pair.
- Generated import paths matter. If your project needs package-prefixed generated imports, use the custom `-Ipackage=...` form shown in the basics tutorial.
- Do not send OAuth or Google-issued tokens to arbitrary non-Google endpoints.
- Do not assume `local_channel_credentials()` means encrypted localhost traffic. Local TCP is checked for locality, not encrypted.
- Do not overuse keepalive on short unary RPCs.
- Do not share AsyncIO gRPC objects across threads.
- Do not mix `async for call` with manual `call.read()` / `call.write()` on the same async streaming RPC.

## Version-Sensitive Notes

- This entry is pinned to PyPI package version `1.78.0`, released on February 6, 2026.
- The docs URL currently redirects to the canonical Python API site serving `gRPC Python Docs v1.78.1`.
- The PyPI project notes that gRPC patch versions are not guaranteed to stay aligned across all languages in the monorepo, so do not assume the latest GitHub patch tag exists on PyPI for Python.
- If you are debugging generated-code mismatches, compare the checked-in generated files, the installed `grpc.__version__`, and the docs version shown on `grpc.github.io`.

## Official Sources

- Python API landing page: https://grpc.io/docs/languages/python/api/
- Canonical Python API docs: https://grpc.github.io/grpc/python/
- Python basics tutorial: https://grpc.io/docs/languages/python/basics/
- Python generated-code reference: https://grpc.io/docs/languages/python/generated-code/
- Auth guide: https://grpc.io/docs/guides/auth/
- Metadata guide: https://grpc.io/docs/guides/metadata/
- Retry guide: https://grpc.io/docs/guides/retry/
- Keepalive guide: https://grpc.io/docs/guides/keepalive/
- Service config guide: https://grpc.io/docs/guides/service-config/
- PyPI package page: https://pypi.org/project/grpcio/

---
name: aiobotocore
description: "aiobotocore package guide for async AWS clients with botocore-compatible APIs, aiohttp transport, and AioConfig"
metadata:
  languages: "python"
  versions: "3.2.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "aiobotocore,aws,botocore,aiohttp,asyncio,python,sdk"
---

# aiobotocore Python Package Guide

## What It Is

`aiobotocore` is the aio-libs asynchronous wrapper around `botocore`. You keep botocore-style AWS service names, method names, and request kwargs, but run them under `asyncio`, create clients with async context managers, and `await` service operations.

If you already know the `boto3` or `botocore` client model, the practical translation is usually:

- create the same service client
- call the same operation names
- `await` the calls
- close clients and streaming response bodies correctly

## Version-Sensitive Notes

- This entry is pinned to the version used here `3.2.1`.
- On March 12, 2026, the current stable docs and the current PyPI release are both `3.2.1`.
- `3.2.1` widened the supported `botocore` window to `>=1.42.53,<1.42.62`. Do not upgrade `botocore` independently past that range.
- Since `3.0.0`, `aiobotocore` forbids leaving a loose underlying `ClientSession` around after an `AioBaseClient` exits. Keep client lifetime inside `async with` or manage it with `AsyncExitStack`.
- Since `3.0.0`, the old package extras `aiobotocore[awscli]` and `aiobotocore[boto3]` are gone. Install `awscli` or `boto3` separately alongside `aiobotocore` if you need them.
- Since `2.23.0`, an optional `httpx` backend exists, but upstream still describes it as experimental and not fully feature-parity with `aiohttp`.
- Since `3.1.0`, `AioConfig.connector_args` can include `socket_factory`.

## Install

Pin the package if you need reproducible behavior:

```bash
python -m pip install "aiobotocore==3.2.1"
```

If you want to try the optional `httpx` transport:

```bash
python -m pip install "aiobotocore[httpx]==3.2.1"
```

If you also need `boto3` or `awscli`, install them separately:

```bash
python -m pip install "aiobotocore==3.2.1" "boto3"
python -m pip install "aiobotocore==3.2.1" "awscli"
```

## Initialize A Session

Use `get_session()` for the common case. Create service clients with `async with` so sockets and credential-refresh resources close deterministically.

```python
import asyncio

from aiobotocore.session import get_session

async def main() -> None:
    session = get_session()

    async with session.create_client("sts", region_name="us-east-1") as sts:
        identity = await sts.get_caller_identity()
        print(identity["Account"])
        print(identity["Arn"])

asyncio.run(main())
```

## Credentials And Region Configuration

`aiobotocore` uses botocore-compatible credential and region behavior. In practice, prefer the same AWS setup you would use for `botocore` or `boto3`:

- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_SESSION_TOKEN`
- `AWS_DEFAULT_REGION`
- `AWS_PROFILE`
- shared config in `~/.aws/config` and `~/.aws/credentials`
- IAM roles in deployed AWS environments

For local development, letting botocore resolve a profile is usually cleaner than hardcoding credentials:

```bash
export AWS_PROFILE=dev
export AWS_DEFAULT_REGION=us-west-2
```

```python
from aiobotocore.session import get_session

session = get_session()

async def list_queues() -> None:
    async with session.create_client("sqs") as sqs:
        response = await sqs.list_queues()
        print(response.get("QueueUrls", []))
```

If you already have short-lived credentials from STS or another bootstrap step, pass them directly to `create_client(...)`:

```python
from aiobotocore.session import get_session

session = get_session()

async def head_bucket() -> None:
    async with session.create_client(
        "s3",
        region_name="us-west-2",
        aws_access_key_id="...",
        aws_secret_access_key="...",
        aws_session_token="...",
    ) as s3:
        await s3.head_bucket(Bucket="my-bucket")
```

## Core Usage Patterns

### Use Botocore-Style Operation Names

AWS operations map to async client methods with the same keyword arguments you would use in botocore-style code.

```python
import asyncio

from aiobotocore.session import get_session

async def main() -> None:
    session = get_session()

    async with session.create_client("s3", region_name="us-west-2") as s3:
        await s3.put_object(Bucket="my-bucket", Key="demo.txt", Body=b"hello")

        response = await s3.get_object(Bucket="my-bucket", Key="demo.txt")
        async with response["Body"] as stream:
            body = await stream.read()
            print(body)

asyncio.run(main())
```

### Always Consume Or Close Streaming Bodies

The upstream examples explicitly wrap `response["Body"]` in `async with` so the underlying connection can be reused. If you forget this, you can leak connections and reduce throughput under load.

```python
response = await s3.get_object(Bucket="my-bucket", Key="demo.txt")
async with response["Body"] as stream:
    payload = await stream.read()
```

### Use Async Paginators

Paginators are async iterables:

```python
from aiobotocore.session import get_session

async def list_keys(bucket: str, prefix: str) -> None:
    session = get_session()

    async with session.create_client("s3", region_name="us-west-2") as s3:
        paginator = s3.get_paginator("list_objects_v2")
        async for page in paginator.paginate(Bucket=bucket, Prefix=prefix):
            for item in page.get("Contents", []):
                print(item["Key"])
```

### Manage Long-Lived Clients With `AsyncExitStack`

For application components that need a client for a longer scope, keep cleanup explicit:

```python
from contextlib import AsyncExitStack

from aiobotocore.session import AioSession

class AwsManager:
    def __init__(self) -> None:
        self._exit_stack = AsyncExitStack()
        self.s3 = None

    async def __aenter__(self):
        session = AioSession()
        self.s3 = await self._exit_stack.enter_async_context(
            session.create_client("s3", region_name="us-west-2")
        )
        return self

    async def __aexit__(self, exc_type, exc, tb) -> None:
        await self._exit_stack.__aexit__(exc_type, exc, tb)
```

## `AioConfig` And Transport Configuration

Use `AioConfig` when you need retry configuration, connection-pool sizing, or async transport-specific options.

```python
from aiobotocore.config import AioConfig
from aiobotocore.session import get_session

config = AioConfig(
    retries={"mode": "standard", "max_attempts": 10},
    connect_timeout=5,
    read_timeout=60,
    max_pool_connections=50,
    connector_args={"ttl_dns_cache": 60},
)

session = get_session()

async def list_tables() -> None:
    async with session.create_client(
        "dynamodb",
        region_name="us-west-2",
        config=config,
    ) as dynamodb:
        print(await dynamodb.list_tables(Limit=10))
```

If you need the experimental `httpx` backend, activate it through `AioConfig` and install the extra first:

```python
from aiobotocore.config import AioConfig
from aiobotocore.httpxsession import HttpxSession
from aiobotocore.session import get_session

config = AioConfig(http_session_cls=HttpxSession)
session = get_session()
```

Use the `httpx` backend only when you have a concrete reason. Upstream still says some `aiohttp` features have not been ported and the backend is not fully tested.

## Tested Service Coverage

Upstream documents explicit test coverage for at least these services:

- S3
- DynamoDB
- SNS
- SQS
- CloudFormation
- Kinesis

The maintainers also note that many other botocore service clients work with the same pattern even if they are not listed individually. Treat that as a good starting assumption, but still smoke-test less common services against the exact operations you need.

## Type Checking

For service-specific type stubs and editor completion:

```bash
python -m pip install "types-aiobotocore[essential]"
```

If you need a lighter install, upstream also publishes `types-aiobotocore-lite`, but it does not provide `session.create_client(...)` overloads, so you usually need explicit client type annotations.

## Common Pitfalls

- Do not install or upgrade `botocore` independently without checking the exact supported range for your `aiobotocore` version.
- Do not keep using a client after its async context has exited.
- Do not ignore `response["Body"]`; close or consume it so connections return to the pool.
- Do not copy older install commands that use `aiobotocore[boto3]` or `aiobotocore[awscli]`; those extras were removed in `3.0.0`.
- Do not assume the experimental `httpx` backend behaves exactly like the default `aiohttp` backend.
- Prefer `asyncio.run(...)` in application entry points instead of older `get_event_loop().run_until_complete(...)` snippets from historical examples.

## Official Sources Used For This Entry

- Docs root: `https://aiobotocore.aio-libs.org/en/stable/`
- Getting started: `https://aiobotocore.aio-libs.org/en/stable/tutorial.html`
- S3 example: `https://aiobotocore.aio-libs.org/en/stable/examples/s3/basic_usage.html`
- PyPI project page: `https://pypi.org/project/aiobotocore/`
- PyPI release metadata for `3.2.1`: `https://pypi.org/pypi/aiobotocore/3.2.1/json`
- Maintainer repository: `https://github.com/aio-libs/aiobotocore`

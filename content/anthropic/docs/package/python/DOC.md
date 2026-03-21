---
name: package
description: "Anthropic Python package guide for the Claude SDK: install, auth, messages, streaming, async, tools, and runtime configuration"
metadata:
  languages: "python"
  versions: "0.84.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "anthropic,claude,llm,api,sdk,python"
---

# anthropic Python Package Guide

## What This Covers

This entry is for the PyPI package [`anthropic`](https://pypi.org/project/anthropic/) version `0.84.0`, which Anthropic documents as the Claude SDK for Python.

Use it when you need to:

- call the Messages API from Python
- stream tokens or structured events
- use the async client
- configure retries, timeouts, or custom HTTP transports
- switch to Anthropic-hosted, Bedrock, or Vertex clients without changing the overall SDK shape

## Install

```bash
pip install anthropic==0.84.0
```

Optional extras documented by Anthropic:

```bash
pip install "anthropic[aiohttp]"
pip install "anthropic[bedrock]"
pip install "anthropic[vertex]"
pip install "anthropic[mcp]"
```

Notes:

- `anthropic[aiohttp]` lets the async client use `aiohttp` instead of the default `httpx`.
- `anthropic[bedrock]` and `anthropic[vertex]` install cloud-specific integrations.
- `anthropic[mcp]` installs Model Context Protocol helpers. Anthropic documents that this extra requires Python `3.10+`.

## Authentication And Client Setup

For Anthropic-hosted API usage, set `ANTHROPIC_API_KEY` and let the client pick it up automatically:

```bash
export ANTHROPIC_API_KEY="your-api-key"
```

```python
from anthropic import Anthropic

client = Anthropic()
```

You can also pass the key directly:

```python
from anthropic import Anthropic

client = Anthropic(api_key="your-api-key")
```

Cloud-specific clients use different classes:

```python
from anthropic import AnthropicBedrock, AnthropicVertex

bedrock = AnthropicBedrock(aws_region="us-east-1")
vertex = AnthropicVertex(project_id="my-gcp-project", region="us-east5")
```

## Core Usage

### Basic message request

```python
from anthropic import Anthropic

client = Anthropic()

message = client.messages.create(
    model="claude-opus-4-6",
    max_tokens=1024,
    messages=[
        {"role": "user", "content": "Write a haiku about clean APIs."}
    ],
)

text = "".join(
    block.text for block in message.content if block.type == "text"
)
print(text)
```

Practical notes:

- `max_tokens` is required in normal `messages.create` calls.
- Response `content` is block-based. Do not assume the SDK returns a single plain string.
- Model IDs change independently of SDK releases. If a copied example uses an outdated model, check Anthropic's current model docs or `client.models.list()`.

### Count tokens before sending

```python
from anthropic import Anthropic

client = Anthropic()

count = client.messages.count_tokens(
    model="claude-opus-4-1",
    messages=[
        {"role": "user", "content": "Hello, Claude"}
    ],
)

print(count.input_tokens)
```

### Async client

```python
import asyncio
from anthropic import AsyncAnthropic

client = AsyncAnthropic()

async def main() -> None:
    message = await client.messages.create(
        model="claude-opus-4-6",
        max_tokens=512,
        messages=[
            {"role": "user", "content": "Summarize this repository layout."}
        ],
    )
    text = "".join(
        block.text for block in message.content if block.type == "text"
    )
    print(text)

asyncio.run(main())
```

If you want the async client to use `aiohttp`:

```python
from anthropic import AsyncAnthropic, DefaultAioHttpClient

client = AsyncAnthropic(http_client=DefaultAioHttpClient())
```

### Streaming

Anthropic documents two patterns. Prefer the streaming helper when you want incremental text and the final assembled message.

```python
from anthropic import Anthropic

client = Anthropic()

with client.messages.stream(
    model="claude-opus-4-6",
    max_tokens=1024,
    messages=[
        {"role": "user", "content": "Stream a short explanation of retries."}
    ],
) as stream:
    for text in stream.text_stream:
        print(text, end="", flush=True)

    final_message = stream.get_final_message()
```

Use `.with_streaming_response` when you want direct access to the raw streaming response object and headers:

```python
from anthropic import Anthropic

client = Anthropic()

with client.messages.with_streaming_response.create(
    model="claude-opus-4-6",
    max_tokens=1024,
    messages=[
        {"role": "user", "content": "List three HTTP status codes."}
    ],
) as response:
    print(response.headers.get("request-id"))
    for line in response.iter_lines():
        print(line)
```

### Tool use

The direct Messages API accepts tool definitions inline:

```python
from anthropic import Anthropic

client = Anthropic()

message = client.messages.create(
    model="claude-opus-4-6",
    max_tokens=1024,
    messages=[
        {"role": "user", "content": "What is the weather in SF?"}
    ],
    tools=[
        {
            "name": "get_weather",
            "description": "Get the weather for a city",
            "input_schema": {
                "type": "object",
                "properties": {
                    "city": {"type": "string"}
                },
                "required": ["city"],
            },
        }
    ],
)

for block in message.content:
    print(block)
```

For local Python functions, Anthropic also ships beta helpers such as `@beta_tool`, `client.beta.messages.tool_runner(...)`, and MCP adapters in `anthropic[mcp]`.

## Configuration

### Retries

Anthropic documents automatic retries by default for:

- connection errors
- HTTP `408`
- HTTP `409`
- HTTP `429`
- HTTP `>=500`

Default retry count is `2`.

```python
from anthropic import Anthropic

client = Anthropic(max_retries=5)

# Per-request override
message = client.with_options(max_retries=0).messages.create(
    model="claude-opus-4-6",
    max_tokens=256,
    messages=[{"role": "user", "content": "No automatic retries for this call."}],
)
```

### Timeouts

Anthropic documents a default timeout of `10 minutes`.

```python
import httpx
from anthropic import Anthropic

client = Anthropic(timeout=20.0)

custom_client = Anthropic(
    timeout=httpx.Timeout(60.0, connect=5.0),
)
```

### Custom HTTP client

Use a custom transport when you need proxies, custom connection limits, or local test endpoints:

```python
import httpx
from anthropic import Anthropic, DefaultHttpxClient

client = Anthropic(
    http_client=DefaultHttpxClient(
        proxy="http://proxy.internal:8080",
        transport=httpx.HTTPTransport(local_address="0.0.0.0"),
    )
)
```

Anthropic also documents per-request overrides through `client.with_options(...)`.

## Common Pitfalls

- The package name and import root are both `anthropic`, but Anthropic's current docs brand the library as the Claude SDK for Python.
- Use `client.messages.create(...)` for new code. Anthropic's modern SDK documentation is centered on the Messages API, not legacy text-completions patterns.
- Do not treat `message.content` as a single string. It is a list of content blocks.
- Streaming has two different APIs: `client.messages.stream(...)` for helper-driven iteration and `client.messages.with_streaming_response.create(...)` for lower-level raw access.
- Beta helpers live under `client.beta...` and may require explicit `betas=[...]` flags. Do not copy beta examples into stable code paths without checking the named beta header requirement.
- Bedrock and Vertex use dedicated client classes and platform credentials. `ANTHROPIC_API_KEY` is for Anthropic-hosted API usage, not for those cloud provider flows.

## Version-Sensitive Notes For 0.84.0

- PyPI lists `0.84.0` as the current release published on `2026-02-25`.
- PyPI lists Python support as `>=3.8`.
- The optional MCP helper package path (`anthropic[mcp]`) is documented separately and requires Python `3.10+`.
- Anthropic's current SDK docs live under `platform.claude.com`. The older docs URL `https://docs.anthropic.com/en/api/client-sdks` redirects there.

## Official Sources Used

- Anthropic SDK docs: `https://platform.claude.com/docs/en/api/sdks/python`
- Anthropic client SDK landing page: `https://platform.claude.com/docs/en/api/client-sdks`
- Anthropic Python SDK repository: `https://github.com/anthropics/anthropic-sdk-python`
- Anthropic SDK helper docs: `https://github.com/anthropics/anthropic-sdk-python/blob/main/helpers.md`
- PyPI package page: `https://pypi.org/project/anthropic/`

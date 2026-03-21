---
name: package
description: "litellm package guide for Python with multi-provider LLM calls, auth, routing, responses API, and proxy setup"
metadata:
  languages: "python"
  versions: "1.82.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "litellm,llm,openai,anthropic,routing,proxy,python"
---

# litellm Python Package Guide

## What It Is

`litellm` is a Python package for calling multiple LLM providers through a mostly OpenAI-shaped interface. Use it when you need one client layer across providers, want to swap models without rewriting the whole call path, or need routing and fallback logic on top of provider SDKs.

The package can be used in two distinct modes:

- Direct Python SDK calls such as `completion()`, `acompletion()`, and `responses()`
- Optional proxy or gateway mode if you install the `proxy` extra

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.82.1`.
- The official docs site is unversioned. Use the docs for current API shape and provider behavior, but use the versioned PyPI release page when you need an exact package pin.
- The PyPI release page for `1.82.1` was current on 2026-03-12 and lists `proxy` as a published extra.
- Provider coverage changes quickly. Before using embeddings, responses, rerank, image generation, or other non-chat endpoints, verify support on the LiteLLM provider page for that provider.

## Install

Base SDK install:

```bash
python -m pip install "litellm==1.82.1"
```

Install the proxy extra only if you need LiteLLM's proxy or gateway features:

```bash
python -m pip install "litellm[proxy]==1.82.1"
```

## Authentication And Provider Setup

LiteLLM supports both explicit credentials and provider environment variables.

- For deterministic scripts and tests, pass `api_key=` directly.
- For local development and deployed apps, provider environment variables are usually cleaner.
- For OpenAI-compatible endpoints or gateways, pass `base_url=` explicitly.

Common provider environment variables from the official docs:

- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

Minimal shell setup:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

If you are targeting a provider-specific model family, verify the exact model name and any provider prefix on that provider's LiteLLM page before writing code.

## Core Usage

### Synchronous chat completion

```python
import os

from litellm import completion

response = completion(
    model="gpt-4o",
    messages=[
        {"role": "user", "content": "Reply with one short sentence about LiteLLM."}
    ],
    api_key=os.environ["OPENAI_API_KEY"],
)

print(response.choices[0].message.content)
```

LiteLLM follows the OpenAI-style `messages=[...]` request shape for chat-style calls. In most provider-backed completion flows, the generated text is available at `response.choices[0].message.content`.

### Async completion

```python
import asyncio
import os

from litellm import acompletion

async def main() -> None:
    response = await acompletion(
        model="claude-sonnet-4-20250514",
        messages=[{"role": "user", "content": "Say hello from async LiteLLM."}],
        api_key=os.environ["ANTHROPIC_API_KEY"],
    )
    print(response.choices[0].message.content)

asyncio.run(main())
```

Use `acompletion()` in async servers, workers, or batch jobs. Do not call it from synchronous code without an event loop boundary.

### Responses API

The docs also expose a `responses()` helper for the OpenAI Responses-style API:

```python
import os

from litellm import responses

response = responses(
    model="gpt-5",
    input="What is the capital of France?",
    api_key=os.environ["OPENAI_API_KEY"],
)

print(response.output_text)
```

Use `responses()` when your application already targets the newer Responses-style surface. Use `completion()` when your codebase is already structured around chat completions and `messages`.

## Routing, Fallbacks, And Multi-Model Setup

LiteLLM's `Router` is the right entry point when you need load balancing, fallback routing, or a shared model alias across providers.

```python
import os

from litellm import Router

router = Router(
    model_list=[
        {
            "model_name": "fast-chat",
            "litellm_params": {
                "model": "gpt-4o-mini",
                "api_key": os.environ["OPENAI_API_KEY"],
            },
        },
        {
            "model_name": "reliable-chat",
            "litellm_params": {
                "model": "claude-sonnet-4-20250514",
                "api_key": os.environ["ANTHROPIC_API_KEY"],
            },
        },
    ]
)

response = router.completion(
    model="fast-chat",
    messages=[{"role": "user", "content": "Answer in five words."}],
)

print(response.choices[0].message.content)
```

Practical guidance:

- Use `model_name` as your app-facing alias and keep provider-specific model strings inside `litellm_params`.
- Centralize retries, fallbacks, and load balancing in the router config instead of scattering provider-switching logic across your code.
- Keep provider keys outside source control and inject them through environment variables or secret stores.

## Optional Proxy Mode

Install `litellm[proxy]` if you need the proxy server or gateway features. The direct SDK install is enough for normal in-process Python usage.

Use proxy mode when you want:

- one internal OpenAI-compatible endpoint in front of multiple providers
- centralized auth, routing, logging, or budgeting
- app code that talks to a single gateway instead of every provider directly

The proxy documentation lives under the LiteLLM docs root and should be treated as a separate operational surface from the Python SDK.

## Error Handling And Common Pitfalls

LiteLLM documents OpenAI-compatible exception mapping. In practice, the most important families are authentication failures, bad request errors, rate limits, and unavailable upstream providers.

Common pitfalls:

- Do not assume every provider supports every OpenAI parameter. LiteLLM documents `drop_params=True` for cases where you intentionally want unsupported OpenAI params omitted instead of raising.
- Do not copy model names across providers blindly. Check the provider page for the exact supported model string and required auth variable.
- Do not treat the unversioned docs site as proof of package pinning. Use the versioned PyPI page for `1.82.1` when you need reproducible installs.
- Do not install `litellm[proxy]` unless you need proxy mode. The base package is enough for direct SDK calls.
- If you switch between `completion()` and `responses()`, keep the response shape straight. `completion()` is typically read from `choices`, while `responses()` exposes `output_text`.
- If a provider rejects an OpenAI-style field, confirm whether LiteLLM documents that field for the target provider before adding compatibility flags.

## Recommended Workflow For Agents

1. Pin `litellm==1.82.1` when reproducing bugs or generating project changes that need a stable dependency version.
2. Start with `completion()` unless the codebase already uses the Responses API or a LiteLLM router.
3. Set auth with explicit env vars or `api_key=` and add `base_url=` only when targeting an OpenAI-compatible gateway or custom endpoint.
4. Check the provider page and supported-endpoints page before adding embeddings, responses, rerank, or image features.
5. Move to `Router` only when the task actually needs multi-model routing, fallback, or aliasing.

## Official Sources

- Docs root: `https://docs.litellm.ai/docs/`
- Completion quick start and async usage: `https://docs.litellm.ai/docs/completion/input`
- Responses API: `https://docs.litellm.ai/docs/response_api`
- Router and load balancing: `https://docs.litellm.ai/docs/routing-load-balancing`
- Supported endpoints and provider coverage: `https://docs.litellm.ai/docs/providers`
- OpenAI provider auth and examples: `https://docs.litellm.ai/docs/providers/openai`
- Exception mapping: `https://docs.litellm.ai/docs/exception_mapping`
- PyPI release page for this version: `https://pypi.org/project/litellm/1.82.1/`

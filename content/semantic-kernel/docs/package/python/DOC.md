---
name: package
description: "semantic-kernel package guide for Python - Microsoft Semantic Kernel SDK"
metadata:
  languages: "python"
  versions: "1.40.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "semantic-kernel,ai,agents,llm,openai,azure"
---

# semantic-kernel Python Package Guide

## What It Is

`semantic-kernel` is Microsoft's orchestration SDK for building LLM-powered Python applications with chat-completion services, prompt invocation, plugins, vector memory connectors, and higher-level agent patterns.

- Package name: `semantic-kernel`
- Import root: `semantic_kernel`
- Language: `python`
- Version covered: `1.40.0`
- Python requirement: `>=3.10`

## Install

Install the pinned package version:

```bash
pip install "semantic-kernel==1.40.0"
```

If you need optional integrations, install the matching PyPI extra or use the broad setup:

```bash
pip install "semantic-kernel[all]==1.40.0"
```

## Choose A Chat Service

The Python SDK centers on registering one or more AI services with a `Kernel`. The common starting points are:

- `OpenAIChatCompletion` for OpenAI-hosted models
- `AzureChatCompletion` for Azure OpenAI deployments

The maintainer docs and samples assume you create `Kernel()` directly and then add services and plugins. There is no separate builder step for Python.

## Configuration And Auth

### OpenAI

Common environment variables:

```env
OPENAI_API_KEY=...
OPENAI_CHAT_MODEL_ID=gpt-4o-mini
```

Optional variables supported by the SDK include `OPENAI_BASE_URL`, `OPENAI_API_VERSION`, and `OPENAI_ORG_ID`.

### Azure OpenAI

Common environment variables:

```env
AZURE_OPENAI_API_KEY=...
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com/
AZURE_OPENAI_CHAT_DEPLOYMENT_NAME=your-chat-deployment
AZURE_OPENAI_API_VERSION=2024-10-21
```

The SDK also supports loading settings from a `.env` file via connector constructor arguments such as `env_file_path`.

## Minimal Setup

### OpenAI-backed kernel

```python
from semantic_kernel import Kernel
from semantic_kernel.connectors.ai.open_ai import OpenAIChatCompletion

kernel = Kernel()
kernel.add_service(OpenAIChatCompletion(service_id="chat"))
```

If you set `OPENAI_API_KEY` and `OPENAI_CHAT_MODEL_ID`, the connector can read them from the environment. You can also pass `api_key=...`, `ai_model_id=...`, and related options directly.

### Azure OpenAI-backed kernel

```python
from semantic_kernel import Kernel
from semantic_kernel.connectors.ai.open_ai import AzureChatCompletion

kernel = Kernel()
kernel.add_service(AzureChatCompletion(service_id="chat"))
```

If you rely on environment loading, make sure the Azure-specific variables match the deployment you actually created. Azure uses deployment names, not raw OpenAI model ids, for chat completion routing.

## Core Usage

### 1. Invoke a prompt directly

```python
import asyncio

from semantic_kernel import Kernel
from semantic_kernel.connectors.ai.open_ai import OpenAIChatCompletion

async def main() -> None:
    kernel = Kernel()
    kernel.add_service(OpenAIChatCompletion(service_id="chat"))

    result = await kernel.invoke_prompt(
        "Write a three-bullet release note for a Python SDK update."
    )
    print(result)

asyncio.run(main())
```

Use `service_id` when you register multiple chat services and need to control which connector executes a prompt.

### 2. Add a plugin for tool/function calling

```python
import asyncio

from semantic_kernel import Kernel
from semantic_kernel.connectors.ai.function_choice_behavior import FunctionChoiceBehavior
from semantic_kernel.connectors.ai.open_ai import (
    OpenAIChatCompletion,
    OpenAIChatPromptExecutionSettings,
)
from semantic_kernel.functions import KernelArguments, kernel_function

class MathPlugin:
    @kernel_function(description="Add two integers and return the sum.")
    def add(self, a: int, b: int) -> int:
        return a + b

async def main() -> None:
    kernel = Kernel()
    kernel.add_service(OpenAIChatCompletion(service_id="chat"))
    kernel.add_plugin(MathPlugin(), plugin_name="math")

    settings = OpenAIChatPromptExecutionSettings()
    settings.function_choice_behavior = FunctionChoiceBehavior.Auto()

    result = await kernel.invoke_prompt(
        "What is 18 + 27? Use the math plugin.",
        arguments=KernelArguments(settings=settings),
    )
    print(result)

asyncio.run(main())
```

For function calling to work reliably:

- decorate callable methods with `@kernel_function`
- give functions and parameters clear names and descriptions
- use execution settings that enable automatic function choice

### 3. Move to agents when you need orchestration

The current maintainer docs position agents as a layer on top of services, prompts, plugins, and memory. Start with a working `Kernel` and service registration first, then add agent abstractions once the base prompt flow is stable.

## Version-Sensitive Notes For `1.40.0`

- The version used here `1.40.0` matches the official Python release published on March 2, 2026.
- The official Learn documentation is a moving target and may describe features added after `1.40.0`; use the release notes when you need to confirm whether a capability existed at this exact version.
- The Python `1.40.0` release specifically called out support for the OpenAI realtime API with GPT-4o Realtime and GPT-4o mini Realtime audio models.

## Common Pitfalls

- Install/import mismatch: install `semantic-kernel`, but import from `semantic_kernel`.
- Wrong Python version: the package requires Python 3.10 or newer.
- Connector env mismatch: `OpenAIChatCompletion` and `AzureChatCompletion` use different environment variable sets.
- Azure deployment confusion: Azure chat connectors need your deployment name, not just a model family name.
- Async execution: most useful examples are `async`, so use `asyncio.run(...)` or integrate into the app's existing event loop.
- Direct service calls need matching settings: if you invoke a service without registering it on a `Kernel`, create the service-specific prompt execution settings explicitly.
- Function calling depends on metadata: vague plugin descriptions and poorly typed parameters reduce tool selection quality.
- Latest-doc drift: Microsoft Learn pages can outpace the exact pinned package version; prefer release notes when a feature looks new.

## Recommended Workflow For Agents

1. Pin `semantic-kernel==1.40.0` unless the project already uses a newer vetted version.
2. Confirm the provider first: OpenAI, Azure OpenAI, or another supported connector.
3. Set provider-specific environment variables before writing orchestration code.
4. Start with `Kernel()` plus one chat-completion service and a direct `invoke_prompt(...)` call.
5. Add plugins only after the base prompt path works.
6. Reach for agents after prompts and plugins are already stable in the target app.

## Official Sources

- Overview: https://learn.microsoft.com/en-us/semantic-kernel/overview/
- Python getting started and quickstarts: https://learn.microsoft.com/en-us/semantic-kernel/get-started/quick-start-guide?pivots=programming-language-python
- Python README and package examples: https://github.com/microsoft/semantic-kernel/blob/main/python/README.md
- Python settings reference: https://raw.githubusercontent.com/microsoft/semantic-kernel/main/python/samples/concepts/setup/ALL_SETTINGS.env.example
- Release notes for `1.40.0`: https://github.com/microsoft/semantic-kernel/releases/tag/python-1.40.0
- PyPI package page: https://pypi.org/project/semantic-kernel/

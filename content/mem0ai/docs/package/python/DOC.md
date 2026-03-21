---
name: package
description: "mem0ai package guide for Python with local Memory, hosted MemoryClient, and v1.0.5 notes"
metadata:
  languages: "python"
  versions: "1.0.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "mem0ai,mem0,python,memory,rag,agents,llm"
---

# mem0ai Python Package Guide

## What It Is

`mem0ai` adds long-term memory to AI applications. In Python, the package has two main surfaces:

- `Memory` / `AsyncMemory` for self-managed open-source memory pipelines
- `MemoryClient` / `AsyncMemoryClient` for the hosted Mem0 Platform API

Pick the surface first. Most integration mistakes come from mixing the local `Memory` setup flow with the hosted `MemoryClient` flow.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.0.5`.
- Official PyPI shows `mem0ai 1.0.5` published on `2026-03-03`.
- The current docs root is a v1 docs site. Older v0 examples and blog posts may use different constructors and payload shapes.
- In the current maintainer docs and README, self-managed usage starts with `Memory.from_config(...)` or `AsyncMemory.from_config(...)`.
- Hosted usage starts with `MemoryClient(...)` or `AsyncMemoryClient(...)`, not the local `Memory(...)` surface.
- Graph memory is optional. Install the `graph` extra if you need it.

## Install

Pin the package when you need reproducible behavior:

```bash
python -m pip install "mem0ai==1.0.5"
```

If you plan to use graph memory:

```bash
python -m pip install "mem0ai[graph]==1.0.5"
```

## Golden Rules

- Use `MemoryClient` for Mem0 Platform and `Memory` for self-managed memory. Do not treat them as interchangeable constructors.
- Keep API keys in environment variables, not source code.
- Pass stable scope identifiers such as `user_id`, `agent_id`, or `run_id`; retrieval only works if writes and reads use the same scope.
- For local memory, confirm the provider config for your embedder, vector store, and LLM before copying an example from another backend.

## Hosted Setup With `MemoryClient`

Use the hosted client when your application should talk to Mem0's managed API.

```bash
export MEM0_API_KEY="m0-..."
```

```python
import os

from mem0 import MemoryClient

client = MemoryClient(api_key=os.environ["MEM0_API_KEY"])

messages = [
    {"role": "user", "content": "I prefer aisle seats on long flights."},
    {"role": "assistant", "content": "I'll remember that preference."},
]

client.add(messages, user_id="alice")

results = client.search(
    "What seating preference does Alice have?",
    user_id="alice",
)

print(results)
```

Use the async client in async services:

```python
import asyncio
import os

from mem0 import AsyncMemoryClient

async def main() -> None:
    client = AsyncMemoryClient(api_key=os.environ["MEM0_API_KEY"])
    await client.add(
        [{"role": "user", "content": "My favorite editor is Neovim."}],
        user_id="alice",
    )
    results = await client.search("What editor does Alice use?", user_id="alice")
    print(results)

asyncio.run(main())
```

## Local Open-Source Setup With `Memory`

Use the open-source surface when you want to choose and operate the vector store, embedder, and LLM yourself.

The official quickstart shows a config-driven setup. Provider-specific auth belongs inside that config, usually from environment variables such as `OPENAI_API_KEY`.

```bash
export OPENAI_API_KEY="sk-..."
```

```python
import os

from mem0 import Memory

config = {
    "vector_store": {
        "provider": "qdrant",
        "config": {
            "host": "localhost",
            "port": 6333,
        },
    },
    "llm": {
        "provider": "openai",
        "config": {
            "api_key": os.environ["OPENAI_API_KEY"],
            "model": "gpt-4.1-mini",
        },
    },
    "embedder": {
        "provider": "openai",
        "config": {
            "api_key": os.environ["OPENAI_API_KEY"],
            "model": "text-embedding-3-small",
        },
    },
}

memory = Memory.from_config(config_dict=config)

memory.add("Alice prefers aisle seats on long flights.", user_id="alice")

results = memory.search(
    "What seating preference does Alice have?",
    user_id="alice",
)

print(results)
```

Practical setup notes:

- The sample above expects a reachable Qdrant instance.
- Switching providers means changing the nested config shape, not just the model name.
- Use the provider pages under the official docs root before swapping to OpenAI-compatible, Anthropic, Azure OpenAI, or other backends.

## Core Operations

The current v1 Python API reference documents these day-to-day operations:

- `add(...)` to write memory from text or chat messages
- `search(...)` to retrieve semantically relevant memories for a scope
- `get_all(...)` to list stored memories for a scope
- `update(...)` and `delete(...)` to maintain specific memories
- `delete_all(...)`, `history(...)`, and `reset()` for cleanup and debugging workflows

For agent systems, the common loop is:

1. Add important user or conversation facts with a stable `user_id`.
2. Search memories at the start of the next interaction.
3. Update or delete memories when a preference changes instead of blindly appending duplicates.

## Config And Auth Checklist

Hosted Mem0 Platform:

- `MEM0_API_KEY` authenticates `MemoryClient` and `AsyncMemoryClient`.

Self-managed open-source usage:

- Provider credentials such as `OPENAI_API_KEY` live inside the config for the chosen embedder or LLM provider.
- The vector store backend may also need its own host, port, URL, or token settings.

Shared advice:

- Keep secrets in environment variables or a secret manager.
- Reuse one configured client or memory instance per process or request scope when possible.
- Keep your scope IDs explicit. `user_id="alice"` and `user_id="Alice"` are different scopes.

## Common Pitfalls

- Mixing the hosted and local setup models. `MemoryClient(api_key=...)` is for Mem0 Platform; `Memory.from_config(...)` is for self-managed memory.
- Copying older pre-v1 examples that instantiate different classes or skip the config factory.
- Forgetting backend prerequisites for local mode. A vector database like Qdrant must be running and reachable if your config points to it.
- Writing memories under one scope and searching another. Missing or inconsistent `user_id` values make retrieval look broken.
- Installing plain `mem0ai` and then enabling graph features without the `graph` extra.

## Official Sources

- Docs root: `https://docs.mem0.ai/`
- Open-source Python quickstart: `https://docs.mem0.ai/open-source/python-quickstart`
- Memory v1 quickstart: `https://docs.mem0.ai/v1.0/components/memory/quickstart`
- Memory API reference: `https://docs.mem0.ai/v1.0/components/memory/api-reference/python/add`
- Migration guide: `https://docs.mem0.ai/migration-guide`
- PyPI package page: `https://pypi.org/project/mem0ai/`
- Maintainer repository: `https://github.com/mem0ai/mem0`

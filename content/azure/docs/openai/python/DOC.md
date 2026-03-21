---
name: openai
description: "Azure OpenAI for Python - practical guide for current Azure OpenAI usage and legacy azure-openai package context"
metadata:
  languages: "python"
  versions: "1.0.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "azure,openai,llm,python,azure-openai"
---

# Azure OpenAI Python Package Guide

## Golden Rule

Treat `azure-openai==1.0.0` as legacy package metadata, not the default install target for new code.

For current Azure OpenAI Python work, Microsoft Learn documents the official `openai` client pointed at your Azure endpoint:

```bash
pip install openai
pip install azure-identity
```

Use `azure-openai` only if you are maintaining an already-pinned legacy environment and have verified the exact package source internally.

## What This Entry Covers

- Ecosystem: `pypi`
- Version used here: `1.0.0`
- Language: `python`
- Current official Azure Python guidance: use the `openai` package with Azure OpenAI `v1` endpoints

## Setup

You need all of the following before code will work:

1. An Azure OpenAI resource endpoint such as `https://YOUR-RESOURCE-NAME.openai.azure.com/`
2. A deployed model in that resource
3. The deployment name for that model
4. Either an API key or Microsoft Entra credentials

Recommended environment variables:

```bash
export AZURE_OPENAI_ENDPOINT="https://YOUR-RESOURCE-NAME.openai.azure.com"
export AZURE_OPENAI_DEPLOYMENT="gpt-4.1-mini"
export AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT="text-embedding-3-large"
export AZURE_OPENAI_API_KEY="..."
```

For keyless auth, keep `AZURE_OPENAI_ENDPOINT` and `AZURE_OPENAI_DEPLOYMENT` and authenticate with `DefaultAzureCredential`.

## Initialize With API Key

Use the standard `OpenAI` client and include the Azure `v1` base URL.

```python
import os
from openai import OpenAI

client = OpenAI(
    api_key=os.environ["AZURE_OPENAI_API_KEY"],
    base_url=f"{os.environ['AZURE_OPENAI_ENDPOINT'].rstrip('/')}/openai/v1/",
)

response = client.responses.create(
    model=os.environ["AZURE_OPENAI_DEPLOYMENT"],
    input="Write a haiku about distributed systems.",
)

print(response.output_text)
```

## Initialize With Microsoft Entra ID

Microsoft recommends keyless auth when possible.

```python
import os
from azure.identity import DefaultAzureCredential, get_bearer_token_provider
from openai import OpenAI

token_provider = get_bearer_token_provider(
    DefaultAzureCredential(),
    "https://cognitiveservices.azure.com/.default",
)

client = OpenAI(
    base_url=f"{os.environ['AZURE_OPENAI_ENDPOINT'].rstrip('/')}/openai/v1/",
    api_key=token_provider,
)

response = client.responses.create(
    model=os.environ["AZURE_OPENAI_DEPLOYMENT"],
    input="Summarize the difference between a queue and a topic.",
)

print(response.output_text)
```

## Core Usage Patterns

### Responses API

Use `client.responses.create(...)` for new text-generation work.

```python
from openai import OpenAI
import os

client = OpenAI(
    api_key=os.environ["AZURE_OPENAI_API_KEY"],
    base_url=f"{os.environ['AZURE_OPENAI_ENDPOINT'].rstrip('/')}/openai/v1/",
)

response = client.responses.create(
    model=os.environ["AZURE_OPENAI_DEPLOYMENT"],
    instructions="You are a precise Python code reviewer.",
    input="Review this function for obvious bugs: def add(a, b): return a - b",
)

print(response.output_text)
```

### Streaming

```python
from openai import OpenAI
import os

client = OpenAI(
    api_key=os.environ["AZURE_OPENAI_API_KEY"],
    base_url=f"{os.environ['AZURE_OPENAI_ENDPOINT'].rstrip('/')}/openai/v1/",
)

stream = client.responses.create(
    model=os.environ["AZURE_OPENAI_DEPLOYMENT"],
    input="List three deployment risks for a schema migration.",
    stream=True,
)

for event in stream:
    if getattr(event, "type", "") == "response.output_text.delta":
        print(event.delta, end="")
```

### Embeddings

Azure OpenAI embeddings on the `v1` API currently require API key auth.

```python
from openai import OpenAI
import os

client = OpenAI(
    api_key=os.environ["AZURE_OPENAI_API_KEY"],
    base_url=f"{os.environ['AZURE_OPENAI_ENDPOINT'].rstrip('/')}/openai/v1/",
)

embedding = client.embeddings.create(
    model=os.environ["AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT"],
    input=[
        "Package docs support internal search.",
        "Agents should prefer official SDK examples.",
    ],
)

print(len(embedding.data))
print(len(embedding.data[0].embedding))
```

For batch embedding requests, Azure documents a maximum array size of `2048` inputs and an `8192` token request limit for the latest embeddings models.

## Configuration And Auth Notes

- `base_url` must end with `/openai/v1/`
- The `model` argument must be your Azure deployment name, not the raw foundation model name
- Keep secrets in environment variables, Azure Key Vault, or Entra ID, not in source code
- For local development, `python-dotenv` is fine, but production code should use your platform secret store

## Version-Sensitive Notes

- Current Azure OpenAI `v1` guidance uses `from openai import OpenAI`; older examples often use `AzureOpenAI`
- The `v1` API removes the need to keep supplying `api-version` for GA usage
- Older blog posts and sample repos may still show `AzureOpenAI(...)`, `azure_endpoint=...`, and explicit `api_version=...`; treat those as legacy patterns unless your codebase is already pinned to them
- Microsoft Learn now documents Azure OpenAI Python usage through `openai`, not through a current `azure-openai` installation flow

## Common Pitfalls

- Using the model name instead of the deployment name in `model=...`
- Forgetting the `/openai/v1/` suffix in `base_url`, which typically causes `404` errors
- Assuming Entra ID works for embeddings on the `v1` API; current Microsoft docs say it does not
- Mixing OpenAI and Azure environment variables in the same process without being explicit about the client configuration
- Copying stale `azure-openai` install instructions from old posts instead of following the current Microsoft Learn guidance

## Official Sources

- Azure OpenAI responses quickstart: https://learn.microsoft.com/en-us/azure/ai-foundry/openai/how-to/responses
- Switching between OpenAI and Azure OpenAI endpoints: https://learn.microsoft.com/en-us/azure/ai-foundry/openai/how-to/switching-endpoints
- Azure OpenAI embeddings: https://learn.microsoft.com/en-us/azure/ai-foundry/openai/how-to/embeddings
- Docs URL: https://learn.microsoft.com/en-us/python/api/azure-openai/

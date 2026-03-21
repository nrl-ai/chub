---
name: package
description: "guidance package guide for Python - constrained generation and structured outputs across local and hosted LLM backends"
metadata:
  languages: "python"
  versions: "0.3.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "guidance,llm,prompting,structured-output,openai,transformers,llamacpp"
---

# guidance Python Package Guide

## What It Is

`guidance` is a Python library for constrained generation with LLMs. It gives you a common interface for:

- local Hugging Face models through `guidance.models.Transformers`
- local GGUF models through `guidance.models.LlamaCpp`
- hosted models through `guidance.models.OpenAI`

Use it when you need structured output, token-level constraints, selectable branches, or reusable prompt programs instead of plain string prompting.

## Version Covered

- Ecosystem: `pypi`
- Package: `guidance`
- Version: `0.3.1`
- Python requirement: `>=3.10`
- Docs root used for coding guidance: `https://guidance.readthedocs.io/en/latest/`

## Install

Install the base package:

```bash
pip install guidance==0.3.1
```

Install the backend extras you actually use:

```bash
pip install "guidance[transformers]"
pip install "guidance[llamacpp]"
pip install "guidance[openai]"
```

If you want everything the project exposes as optional extras:

```bash
pip install "guidance[all]"
```

## Backend Setup

Choose one backend first. Most runtime issues come from mixing examples from different backends.

### Transformers

Use this for local or self-hosted Hugging Face causal language models.

```python
from guidance.models import Transformers

lm = Transformers("microsoft/Phi-4-mini-instruct")
```

The constructor also accepts loaded `transformers` model and tokenizer objects when you need custom loading behavior.

### LlamaCpp

Use this for local GGUF models through `llama.cpp`.

```python
from guidance.models import LlamaCpp

lm = LlamaCpp(model="models/phi-4-mini-instruct-q4.gguf")
```

### OpenAI

Use this when you want hosted OpenAI models.

```python
import os
from guidance.models import OpenAI

lm = OpenAI(
    "gpt-4o-mini",
    api_key=os.environ["OPENAI_API_KEY"],
)
```

If you need a proxy or compatible endpoint, pass `base_url`. If you need org scoping, pass `organization`.

```python
import os
from guidance.models import OpenAI

lm = OpenAI(
    "gpt-4o-mini",
    api_key=os.environ["OPENAI_API_KEY"],
    base_url=os.getenv("OPENAI_BASE_URL"),
    organization=os.getenv("OPENAI_ORGANIZATION"),
)
```

## Core Usage

### Minimal chat with a constrained choice

```python
from guidance import assistant, select, system, user
from guidance.models import Transformers

lm = Transformers("microsoft/Phi-4-mini-instruct")

result = (
    lm
    + system()
    + "You are a classifier. Reply with the selected label only."
    + user()
    + "Classify the sentiment of: guidance makes structured output easier."
    + assistant()
    + select(["positive", "negative"], name="sentiment")
)

print(result["sentiment"])
```

Use `select()` when the output must be one of a fixed set of strings.

### Free-form generation with capture

```python
from guidance import gen
from guidance.models import Transformers

lm = Transformers("microsoft/Phi-4-mini-instruct")
result = lm + "Write a short release note:\n" + gen(name="note", max_tokens=60)

print(result["note"])
```

Use `gen(name=...)` when you want to capture generated text into a named field.

### Structured JSON generation

```python
from guidance import json as gen_json
from guidance.models import OpenAI

schema = {
    "type": "object",
    "properties": {
        "language": {"type": "string"},
        "score": {"type": "integer", "minimum": 0, "maximum": 10},
    },
    "required": ["language", "score"],
}

lm = OpenAI("gpt-4o-mini", api_key="...")
result = lm + "Return a rating for Python as JSON.\n" + gen_json(
    name="rating",
    schema=schema,
)

print(result["rating"])
```

Use `guidance.json()` when you need schema-constrained JSON instead of post-processing raw text.

### Reusable prompt programs

```python
import guidance
from guidance import gen
from guidance.models import Transformers

@guidance(stateless=True)
def summarize(lm, text):
    lm += f"Summarize this in one sentence:\n{text}\nSummary: "
    lm += gen(name="summary", max_tokens=50)
    return lm

lm = Transformers("microsoft/Phi-4-mini-instruct")
result = lm + summarize("Guidance composes prompts as immutable model states.")

print(result["summary"])
```

Use `@guidance` functions when you want reusable prompt fragments or higher-level prompt programs.

## Config And Auth

### OpenAI credentials

The common setup is:

```bash
export OPENAI_API_KEY=your_api_key
```

Optional settings:

```bash
export OPENAI_BASE_URL=https://your-proxy-or-compatible-endpoint/v1
export OPENAI_ORGANIZATION=org_xxx
```

### Local model files

- `Transformers` needs a compatible Hugging Face model id or loaded model/tokenizer objects.
- `LlamaCpp` needs a local GGUF model file path.
- Local backends avoid API keys but still need model weights present on disk.

## Common Pitfalls

- Install the right extra. `pip install guidance` alone does not guarantee the backend integration dependencies you need.
- Backend capabilities differ. The project docs explicitly note that `LlamaCpp` and `Transformers` support the full set of Guidance features, while `OpenAI` support is currently limited to JSON generation.
- Treat results as returned model state. Capture the returned object and then read named variables from it, for example `result["sentiment"]` or `result["rating"]`.
- `guidance.json()` only supports a subset of JSON Schema keywords. The documented supported set includes `enum`, `format`, `maximum`, `minimum`, `maxItems`, `minItems`, `properties`, `required`, `type`, and related object/array structure.
- When using `gen(..., tools=...)`, the API docs require `max_tokens`, and `regex`/`stop` constraints are not supported in the same call.
- The docs site contains some older notebook material. Prefer the generated API pages plus the current intro tutorial when notebook examples disagree.

## Version-Sensitive Notes For 0.3.1

- PyPI confirms `0.3.1` as the package version covered here.
- The upstream docs site is published as `latest`, not a pinned `0.3.1` doc set, so verify signatures against the current API pages if you are debugging version-specific behavior.
- The GitHub releases page exposes `0.3.0` notes publicly, but a matching `0.3.1` release note was not surfaced on that date. For exact behavioral deltas, confirm against the package source or release history before assuming a new API surface.
- PyPI metadata for `0.3.1` requires Python `>=3.10`.

## Official Sources

- GitHub repository: https://github.com/guidance-ai/guidance
- ReadTheDocs docs root: https://guidance.readthedocs.io/en/latest/
- PyPI package page: https://pypi.org/project/guidance/

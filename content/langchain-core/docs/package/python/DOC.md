---
name: package
description: "langchain-core package guide for Python covering runnables, prompt templates, tools, configuration, and integration boundaries"
metadata:
  languages: "python"
  versions: "1.2.18"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "langchain-core,langchain,python,llm,runnables,prompts,tools,langsmith"
---

# langchain-core Python Package Guide

## What This Package Provides

`langchain-core` is the base abstractions package for LangChain Python applications.

Use it for:

- prompt templates and message objects
- runnable composition with the LangChain Expression Language (LCEL)
- tools and tool schemas
- output parsers, documents, callbacks, and shared config plumbing

Important boundary: `langchain-core` does not include provider clients, embeddings, or vector store integrations. For actual model calls, pair it with an integration package such as `langchain-openai`.

Common imports:

- `langchain_core.prompts`
- `langchain_core.messages`
- `langchain_core.runnables`
- `langchain_core.tools`
- `langchain_core.output_parsers`

## Version Covered

- Package: `langchain-core`
- Version: `1.2.18`
- Python requirement on PyPI: `>=3.10,<4.0`
- Registry: `https://pypi.org/project/langchain-core/`
- Canonical reference root: `https://reference.langchain.com/python/langchain_core/`

The maintainer docs URL `https://python.langchain.com/api_reference/core/` redirects to the current reference site under `reference.langchain.com`.

## Install

Use a virtual environment and pin the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "langchain-core==1.2.18"
```

If you plan to call a hosted model, install the matching provider package too:

```bash
python -m pip install "langchain-core==1.2.18" "langchain-openai"
```

## Core Workflow

### Build chat prompts

`ChatPromptTemplate` turns message templates into a prompt value that downstream runnables can consume.

```python
from langchain_core.prompts import ChatPromptTemplate

prompt = ChatPromptTemplate(
    [
        ("system", "Reply in one short sentence."),
        ("human", "Explain {topic}."),
    ]
)

prompt_value = prompt.invoke({"topic": "Python context managers"})

for message in prompt_value.to_messages():
    print(type(message).__name__, message.content)
```

`prompt.invoke(...)` returns a prompt value with message objects, not a plain string.

### Compose runnables with LCEL

`RunnableLambda` wraps a normal Python callable so it can participate in LCEL pipelines.

```python
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.runnables import RunnableLambda

prompt = ChatPromptTemplate(
    [
        ("system", "Be concise."),
        ("human", "Summarize {topic} in one sentence."),
    ]
)

def mock_model(prompt_value) -> str:
    last_message = prompt_value.to_messages()[-1].content
    return f"Stub answer for: {last_message}"

chain = prompt | RunnableLambda(mock_model)

print(chain.invoke({"topic": "LCEL"}))
```

Interfaces to remember:

- `invoke(...)` for one input
- `ainvoke(...)` for async use
- `batch([...])` for multiple inputs
- `stream(...)` and `astream(...)` when the runnable supports streaming

The reference docs note that `RunnableLambda` is best for logic that does not need token-by-token streaming.

### Fan out one input into parallel branches

Use `RunnableParallel` when the same input should feed multiple computations.

```python
from langchain_core.runnables import RunnableLambda, RunnableParallel

analysis = RunnableParallel(
    original=RunnableLambda(lambda x: x["text"]),
    upper=RunnableLambda(lambda x: x["text"].upper()),
    word_count=RunnableLambda(lambda x: len(x["text"].split())),
)

result = analysis.invoke({"text": "langchain core keeps abstractions reusable"})
print(result)
```

### Define tools

Use `@tool` when you need a callable with a name, description, and argument schema that an agent or model integration can consume.

```python
from langchain_core.tools import tool

@tool
def lookup_order(order_id: str) -> str:
    """Return a stub shipping status for an order."""
    return f"{order_id}: shipped"

print(lookup_order.name)
print(lookup_order.invoke({"order_id": "A-100"}))
```

Use type hints and a useful docstring so the generated tool schema is accurate.

### Attach run metadata and callbacks

Use `with_config(...)` to add metadata, tags, callbacks, or concurrency settings without rewriting the runnable.

```python
from langchain_core.runnables import RunnableLambda

uppercase = RunnableLambda(lambda text: text.upper()).with_config(
    run_name="uppercase_demo",
    tags=["demo", "langchain-core"],
    metadata={"package": "langchain-core"},
)

print(uppercase.invoke("hello"))
```

Common config keys:

- `run_name`
- `tags`
- `metadata`
- `callbacks`
- `max_concurrency`
- `recursion_limit`
- `configurable`

## Auth And Initialization

### `langchain-core` has no provider auth of its own

`langchain-core` is intentionally provider-agnostic. It does not define a package-specific API key for model access.

For real model calls:

1. install the provider integration package
2. set that provider's environment variables
3. compose the provider client with your `langchain_core` prompt or runnable pipeline

Example with `langchain-openai`:

```bash
python -m pip install "langchain-openai"
export OPENAI_API_KEY="sk-..."
```

```python
from langchain_core.prompts import ChatPromptTemplate
from langchain_openai import ChatOpenAI

prompt = ChatPromptTemplate(
    [
        ("system", "Reply tersely."),
        ("human", "Summarize {topic} in one sentence."),
    ]
)

model = ChatOpenAI(model="gpt-4.1-mini")
chain = prompt | model

print(chain.invoke({"topic": "LangChain"}).content)
```

### Optional LangSmith tracing

LangSmith tracing is optional, but the official LangChain docs document these environment variables:

```bash
export LANGSMITH_TRACING=true
export LANGSMITH_API_KEY="lsv2_..."
export LANGSMITH_PROJECT="my-project"
```

You can use those settings when you want traces for prompt execution and runnable graphs. Pure `langchain-core` composition does not require LangSmith.

## Practical Pitfalls

- `langchain-core` is not the full application stack. Install separate provider or storage integrations when your workflow needs them.
- `ChatPromptTemplate.invoke(...)` returns a prompt value with messages, not a plain string.
- `RunnableLambda` is convenient for Python-only logic, but it is not the right wrapper when you need token streaming behavior from the callable itself.
- Tool schemas depend on the Python signature and docstring. Untyped or poorly described functions produce weaker tool metadata.
- Many LangChain examples on the web mix `langchain-core` with `langchain`, `langgraph`, LangSmith, or provider packages. Check imports before copying a snippet into a project pinned to `langchain-core` only.

## Official Sources

- PyPI package page: `https://pypi.org/project/langchain-core/`
- Install docs: `https://docs.langchain.com/oss/python/langchain/install`
- Canonical API reference root: `https://reference.langchain.com/python/langchain_core/`
- `RunnableLambda` reference: `https://reference.langchain.com/python/langchain_core/runnables/#langchain_core.runnables.base.RunnableLambda`
- `ChatPromptTemplate` reference: `https://reference.langchain.com/python/langchain_core/prompts/#langchain_core.prompts.chat.ChatPromptTemplate`
- LangSmith observability quickstart: `https://docs.langchain.com/langsmith/observability-quickstart`

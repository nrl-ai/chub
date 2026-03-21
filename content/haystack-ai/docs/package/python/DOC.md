---
name: package
description: "Haystack AI Python framework for modular RAG pipelines, agents, custom components, and integrations"
metadata:
  languages: "python"
  versions: "2.25.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "haystack,rag,agents,llm,pipelines,retrieval"
---

# Haystack AI Python Package Guide

## Golden Rule

Use `haystack-ai` 2.x for new Haystack work, not the legacy `farm-haystack` package. Build applications as explicit pipelines of components, keep secrets in environment variables via Haystack `Secret`s, and treat the core package as the orchestration layer while adding integration packages only when a component or backend actually requires them.

## Install

Pin the package version your project expects:

```bash
python -m pip install "haystack-ai==2.25.2"
```

Common alternatives:

```bash
uv pip install "haystack-ai==2.25.2"
uv add "haystack-ai==2.25.2"
conda config --add channels conda-forge/label/haystack-ai_rc
conda install haystack-ai
```

Important install notes:

- Do not keep `farm-haystack` and `haystack-ai` in the same environment. The official installation guide says mixed installs can fail in obscure ways.
- Base installation is intentionally lightweight. Some components need optional Python dependencies, and Haystack will raise an `ImportError` telling you what to install.
- Many external providers and backends are documented as Haystack integrations. The API reference separates the core `haystack-ai` package from integrations distributed as separate packages.

If you need to clean up a mixed 1.x and 2.x environment:

```bash
python -m pip uninstall -y farm-haystack haystack-ai
python -m pip install "haystack-ai==2.25.2"
```

## Core Concepts

### Pipelines

Haystack pipelines are directed multigraphs of components. The practical flow is:

1. Create a `Pipeline()`
2. Add components with `add_component()`
3. Wire them with `connect()`
4. Execute with `run()`

Connections are validated before runtime, which helps catch type and socket mismatches early.

### Documents and Retrievers

`Document` is the basic unit of retrieved context. For local experiments, the simplest setup is `InMemoryDocumentStore` plus `InMemoryBM25Retriever`.

### ChatMessage

For chat-oriented generators and agents, use `ChatMessage.from_user(...)`, `from_system(...)`, `from_assistant(...)`, and `from_tool(...)`. Current docs treat those factory methods as the standard way to create messages.

### Secrets

Use `from haystack.utils import Secret` and prefer `Secret.from_env_var(...)` for API keys. Environment-variable secrets serialize safely because Haystack stores the variable name, not the raw secret value.

## Minimal RAG Pipeline

This is the current official-style "first app" pattern for Haystack 2.25: load documents, retrieve them, build a chat prompt, and send the messages to a chat generator.

```python
from haystack import Document, Pipeline
from haystack.components.builders.chat_prompt_builder import ChatPromptBuilder
from haystack.components.generators.chat import OpenAIChatGenerator
from haystack.components.retrievers.in_memory import InMemoryBM25Retriever
from haystack.dataclasses import ChatMessage
from haystack.document_stores.in_memory import InMemoryDocumentStore
from haystack.utils import Secret

document_store = InMemoryDocumentStore()
document_store.write_documents(
    [
        Document(content="My name is Jean and I live in Paris."),
        Document(content="My name is Mark and I live in Berlin."),
        Document(content="My name is Giorgio and I live in Rome."),
    ]
)

prompt_template = [
    ChatMessage.from_system("You are a helpful assistant."),
    ChatMessage.from_user(
        "Given these documents, answer the question.\n"
        "Documents:\n{% for doc in documents %}{{ doc.content }}\n{% endfor %}\n"
        "Question: {{question}}\n"
        "Answer:"
    ),
]

prompt_builder = ChatPromptBuilder(
    template=prompt_template,
    required_variables={"documents", "question"},
)
retriever = InMemoryBM25Retriever(document_store=document_store)
llm = OpenAIChatGenerator(api_key=Secret.from_env_var("OPENAI_API_KEY"))

rag_pipeline = Pipeline()
rag_pipeline.add_component("retriever", retriever)
rag_pipeline.add_component("prompt_builder", prompt_builder)
rag_pipeline.add_component("llm", llm)

rag_pipeline.connect("retriever", "prompt_builder.documents")
rag_pipeline.connect("prompt_builder", "llm.messages")

question = "Who lives in Paris?"
result = rag_pipeline.run(
    {
        "retriever": {"query": question},
        "prompt_builder": {"question": question},
    }
)

print(result["llm"]["replies"][0].text)
```

Practical notes:

- `InMemoryDocumentStore` is recommended for experiments, not production.
- `InMemoryBM25Retriever` is keyword-based. For semantic retrieval, switch to an embedding-based path and the corresponding dependencies or integrations.
- `OpenAIChatGenerator` consumes `messages`, not raw prompt strings.

## Configuration And Auth

### Environment variables

For OpenAI-backed examples, export the API key before running your script:

```bash
export OPENAI_API_KEY="sk-..."
```

Then construct the component with a Haystack secret:

```python
from haystack.components.generators.chat import OpenAIChatGenerator
from haystack.utils import Secret

llm = OpenAIChatGenerator(api_key=Secret.from_env_var("OPENAI_API_KEY"))
```

### When to use `Secret.from_token(...)`

Use `Secret.from_token(...)` only for short-lived experiments. Token-based secrets cannot be serialized, so any pipeline containing them cannot be safely dumped to YAML.

### Serialization

Pipelines are serializable, and current Haystack docs describe YAML as the built-in format.

```python
yaml_text = rag_pipeline.dumps()

# Later
restored = Pipeline.loads(yaml_text)
```

If you expect to save and reload a pipeline, prefer environment-variable secrets and keep non-serializable runtime objects out of component init parameters.

## Custom Components

Use `@component` for custom logic you want to insert into a pipeline. Output names must match the dictionary returned by `run()`.

```python
from haystack import component

@component
class WelcomeTextGenerator:
    @component.output_types(welcome_text=str, note=str)
    def run(self, name: str):
        return {
            "welcome_text": f"Hello {name}, welcome to Haystack!".upper(),
            "note": "welcome message is ready",
        }
```

This is the standard extension point when the built-in components do not exactly match your workflow.

## Agents And Tools

Current Haystack 2.25 docs treat agents as compositions of:

- a chat generator that supports tool calling
- tools such as `Tool`, `ComponentTool`, or `PipelineTool`
- optionally `ToolInvoker` when you want tool execution as part of a pipeline graph

For simple tool-calling agents, the `Agent` component is the highest-level entry point. For explicit orchestration, use a normal `Pipeline` and wire a generator, router, tool invoker, and state-handling components yourself.

## Common Pitfalls

- Mixing `farm-haystack` and `haystack-ai` in one environment is a real source of broken imports and obscure runtime behavior.
- `InMemoryDocumentStore` is not a production document store. Replace it before shipping anything that needs persistence, scale, or multi-process access.
- Missing optional dependencies are expected with the minimal install. Read the `ImportError` and install the named dependency or integration package instead of debugging the component itself.
- Chat-based components work with `ChatMessage` objects. If you pass raw strings where `messages` are expected, the pipeline wiring will not match the component inputs.
- Use `ChatMessage.from_user(...)` and the other factory methods rather than inventing your own message dict format.
- Secret handling affects serialization. `Secret.from_token(...)` is convenient, but it prevents safe pipeline dumping.
- If tracing libraries such as OpenTelemetry or Datadog are present and you do not want auto tracing, set `HAYSTACK_AUTO_TRACE_ENABLED=false` before import.

## Version-Sensitive Notes For 2.25.2

- PyPI currently lists `haystack-ai 2.25.2` as the latest stable release, released on March 5, 2026.
- The official API reference root currently defaults to version `2.25` and also shows a separate `2.26-unstable` track. Keep examples aligned to the 2.25 docs when working against `2.25.2`.
- The docs explicitly separate the core `haystack-ai` API from integrations distributed as separate packages. If you find older examples assuming everything ships in one install, re-check the current integration docs.
- Current tooling docs include `PipelineTool` as the direct way to wrap a pipeline for tool calling. Prefer current tool APIs over older blog posts that describe more manual wrapping patterns.

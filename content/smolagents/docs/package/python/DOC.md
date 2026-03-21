---
name: package
description: "smolagents package guide for Python agents with CodeAgent, ToolCallingAgent, models, tools, and secure execution"
metadata:
  languages: "python"
  versions: "1.24.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "smolagents,python,agents,llm,huggingface,tool-calling,code-agents"
---

# smolagents Python Package Guide

## What It Is

`smolagents` is Hugging Face's lightweight agent framework for Python. It gives you:

- agent classes such as `CodeAgent` and `ToolCallingAgent`
- model wrappers for Hugging Face Inference, OpenAI-compatible APIs, LiteLLM, and local Transformers
- a `@tool` decorator and `Tool` base class for exposing callable tools to agents
- optional secure execution backends for code-generating agents

Use it when you need a Python agent loop that can call tools, generate code, or orchestrate other agents without pulling in a large framework.

## Version-Sensitive Notes

- This entry is pinned to `smolagents==1.24.0`.
- The current Hugging Face docs "stable" version and the current PyPI release both resolve to `1.24.0`, so the version used here matches upstream as of March 12, 2026.
- The upstream docs describe `CodeAgent` as the code-writing agent and `ToolCallingAgent` as the JSON/tool-calling variant; choose deliberately because they execute tasks differently.
- `CodeAgent` can execute model-generated Python. For untrusted prompts or networked tasks, review the secure execution guide before running it on your machine.
- Agent loops are multi-step by default. The docs show `max_steps=20` as the default, so tune that down for bounded tasks or up for more complex workflows.

## Install

Basic install:

```bash
python -m pip install "smolagents==1.24.0"
```

The official docs also show the plain install form:

```bash
pip install smolagents
```

## Core Mental Model

`smolagents` is built around three pieces:

1. A model wrapper object such as `InferenceClientModel` or `OpenAIServerModel`
2. An agent such as `CodeAgent` or `ToolCallingAgent`
3. Zero or more tools the agent is allowed to call

Do not pass just a model name string to an agent. Instantiate a model object first, then pass it as `model=...`.

## Minimal Setup

This is the smallest useful pattern from the official docs: create a model object, create an agent, then call `run(...)`.

```python
from smolagents import CodeAgent, InferenceClientModel

model = InferenceClientModel(
    model_id="Qwen/Qwen2.5-Coder-32B-Instruct",
)

agent = CodeAgent(
    tools=[],
    model=model,
    max_steps=20,
)

result = agent.run("Write a Python function that returns the first 10 Fibonacci numbers.")
print(result)
```

Use `CodeAgent` when you want the model to solve tasks by writing Python snippets during the loop.

## Choosing A Model Backend

The model guide documents several wrappers. The ones you are most likely to reach for are:

- `InferenceClientModel`: Hugging Face Inference providers and hosted inference endpoints
- `OpenAIServerModel`: OpenAI-compatible APIs such as OpenAI or other providers that expose an OpenAI-style endpoint
- `LiteLLMModel`: when you already standardize provider access through LiteLLM
- `TransformersModel`: local model inference through `transformers`

### OpenAI-Compatible Models

Use `OpenAIServerModel` when your provider exposes an OpenAI-style API.

```python
import os

from smolagents import CodeAgent, OpenAIServerModel

model = OpenAIServerModel(
    model_id="gpt-4.1-mini",
    api_base="https://api.openai.com/v1",
    api_key=os.environ["OPENAI_API_KEY"],
)

agent = CodeAgent(tools=[], model=model)
print(agent.run("Summarize what a Python context manager does in two sentences."))
```

The official model guide also shows `OpenAIServerModel` as the path for other OpenAI-compatible backends such as Ollama and third-party hosted providers. When you switch providers, keep the same pattern and change the base URL, model id, and credentials to match that provider.

### Hugging Face Inference

Use `InferenceClientModel` when you want Hugging Face-hosted or provider-routed inference.

```python
from smolagents import CodeAgent, InferenceClientModel

model = InferenceClientModel(
    model_id="Qwen/Qwen2.5-Coder-32B-Instruct",
    provider="together",
)

agent = CodeAgent(tools=[], model=model)
print(agent.run("Give me a 3-item Python code review checklist."))
```

Auth for this path depends on the inference provider you choose. Configure the provider credentials before running the agent, then instantiate the model once and reuse it.

## Building Tools

For most projects, define tools with `@tool`. The tool docs require:

- typed function parameters
- a return type annotation
- a descriptive docstring so the agent can understand when to call it

```python
from smolagents import ToolCallingAgent, InferenceClientModel, tool

@tool
def get_weather(city: str) -> str:
    """
    Return the current weather summary for a city.

    Args:
        city: The city name to look up.
    """
    return f"The weather in {city} is sunny."

model = InferenceClientModel(model_id="Qwen/Qwen2.5-Coder-32B-Instruct")
agent = ToolCallingAgent(tools=[get_weather], model=model)

print(agent.run("What is the weather in Paris?"))
```

Use `ToolCallingAgent` when the model/provider has reliable tool-calling support and you want structured tool invocations instead of code execution.

## Secure Code Execution

`CodeAgent` can run generated Python code. The secure execution guide matters if prompts, tools, or retrieved content are not fully trusted.

Practical guidance:

- local execution is convenient for trusted internal tasks, prototypes, and tests
- for untrusted tasks, prefer an isolated executor rather than running generated code on your workstation or CI host
- the official guide documents isolated execution options including E2B, Docker, Modal, and a Pyodide+Deno WebAssembly path

If the task only needs tool calls and not arbitrary generated Python, consider `ToolCallingAgent` instead of `CodeAgent`.

## Common Usage Patterns

### Constrain The Loop

Set `max_steps` explicitly for tasks that must stop quickly:

```python
from smolagents import CodeAgent, InferenceClientModel

agent = CodeAgent(
    tools=[],
    model=InferenceClientModel(model_id="Qwen/Qwen2.5-Coder-32B-Instruct"),
    max_steps=6,
)
```

This avoids long agent loops when the task should be answerable in a few reasoning/tool steps.

### Reuse Long-Lived Objects

Instantiate the model and agent once for a workflow instead of rebuilding them for every prompt:

```python
from smolagents import CodeAgent, InferenceClientModel

model = InferenceClientModel(model_id="Qwen/Qwen2.5-Coder-32B-Instruct")
agent = CodeAgent(tools=[], model=model)

for task in [
    "Generate a regex for ISO dates.",
    "Explain why raw strings help with regexes in Python.",
]:
    print(agent.run(task))
```

## Authentication And Configuration Notes

- `smolagents` itself is the agent framework. Authentication usually comes from the model backend you choose.
- For OpenAI-compatible APIs, the official docs show passing `api_key=` and `api_base=` to `OpenAIServerModel`.
- For Hugging Face inference, configure the chosen provider's credentials first, then create `InferenceClientModel(...)`.
- Keep provider secrets in environment variables or your platform secret manager, not inline in source files.
- If a model does not support the behavior you need, switch model backends before changing agent logic. Tool calling, vision, and provider-specific capabilities come from the selected underlying model.

## Common Pitfalls

- Passing `model="gpt-4.1-mini"` directly into an agent. `smolagents` expects a model wrapper instance, not a bare string.
- Using `CodeAgent` for untrusted work without isolation. It may execute model-generated Python locally.
- Decorating a tool without full type hints or without a useful docstring. The agent uses that schema to decide whether the tool is callable.
- Debugging the wrong layer. Provider auth failures, unsupported tool calling, or missing vision support usually come from the selected model backend, not from the agent class itself.
- Letting loops run too long. Set `max_steps` intentionally for bounded tasks.

## Recommended Starting Points

Pick one path and keep it simple:

- If you want Python-code actions: start with `CodeAgent` plus a single trusted model backend.
- If you want structured tool calls: start with `ToolCallingAgent` plus one or two well-typed tools.
- If you are exposing external systems: define tools first, then let the agent orchestrate them.
- If prompts are untrusted: choose a sandboxed execution strategy before using `CodeAgent`.

## Official URLs Used For This Guide

- Docs home: `https://huggingface.co/docs/smolagents/`
- Versioned docs index: `https://huggingface.co/docs/smolagents/v1.24.0/en/index`
- Agents guide: `https://huggingface.co/docs/smolagents/v1.24.0/en/conceptual_guides/intro_agents`
- Models guide: `https://huggingface.co/docs/smolagents/v1.24.0/en/reference/models`
- Tools guide: `https://huggingface.co/docs/smolagents/v1.24.0/en/tutorials/tools`
- Secure code execution guide: `https://huggingface.co/docs/smolagents/v1.24.0/en/tutorials/secure_code_execution`
- PyPI: `https://pypi.org/project/smolagents/1.24.0/`
- GitHub repository: `https://github.com/huggingface/smolagents`
- `v1.24.0` release notes: `https://github.com/huggingface/smolagents/releases/tag/v1.24.0`

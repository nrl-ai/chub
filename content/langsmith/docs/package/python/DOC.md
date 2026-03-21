---
name: package
description: "LangSmith 0.7.16 package guide for tracing, evaluation, and testing in Python"
metadata:
  languages: "python"
  versions: "0.7.16"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "langsmith,python,llm,observability,tracing,evaluation,testing"
---

# LangSmith Python Package Guide

## Install

Base install:

```bash
python -m venv .venv
source .venv/bin/activate
pip install "langsmith==0.7.16"
```

Useful extras:

```bash
pip install "langsmith[pytest]==0.7.16"
pip install "langsmith[otel]==0.7.16"
```

- `pytest` adds richer test output and request-caching helpers for the LangSmith pytest integration.
- `otel` lets LangSmith receive OpenTelemetry spans from already-instrumented apps.

## Auth And Configuration

Minimal environment setup for the hosted service:

```bash
export LANGSMITH_API_KEY="YOUR_API_KEY"
export LANGSMITH_TRACING="true"
export LANGSMITH_PROJECT="my-app"
```

Additional environment variables:

```bash
# Set this only when your API key can access multiple workspaces.
export LANGSMITH_WORKSPACE_ID="YOUR_WORKSPACE_ID"

# Set this for the EU SaaS region or a self-hosted deployment.
export LANGSMITH_ENDPOINT="https://api.smith.langchain.com"
```

Notes:

- `LANGSMITH_TRACING=true` is what actually turns automatic tracing on for decorators and wrapped model clients.
- If `LANGSMITH_PROJECT` is omitted, traces go to the default project.
- For EU accounts, use the EU API endpoint instead of the US default.
- Avoid hardcoding the API key into source code. Use environment variables or your app's secret manager.

If environment variables are not practical, configure a client directly:

```python
from langsmith import Client

client = Client(
    api_key="YOUR_API_KEY",
    api_url="https://api.smith.langchain.com",
    workspace_id="YOUR_WORKSPACE_ID",  # omit when not needed
)
```

## Core Tracing Patterns

### Trace Plain Python Functions

Use `@traceable` for normal Python functions. This is the simplest way to get spans for custom application logic.

```python
from langsmith import traceable

@traceable(run_type="chain", name="answer_question")
def answer_question(question: str) -> dict[str, str]:
    answer = question.upper()
    return {"answer": answer}

result = answer_question("Where is the trace?")
print(result)
```

Important:

- Decorated functions are only sent to LangSmith when tracing is enabled with env vars or `tracing_context`.
- Use `run_type` to make traces easier to read in the UI (`chain`, `tool`, `llm`, and so on).

### Wrap Supported Model SDKs

For provider SDKs that LangSmith knows how to wrap, wrap the client once and keep the rest of your code normal.

```python
from openai import OpenAI

from langsmith import traceable
from langsmith.wrappers import wrap_openai

openai_client = wrap_openai(OpenAI())

@traceable(name="chat_pipeline")
def chat_pipeline(question: str) -> str:
    response = openai_client.chat.completions.create(
        model="gpt-4.1-mini",
        messages=[{"role": "user", "content": question}],
    )
    return response.choices[0].message.content or ""

print(chat_pipeline("Summarize LangSmith in one sentence."))
```

Pitfall: wrapping the client does not bypass configuration. You still need `LANGSMITH_TRACING=true` or a `tracing_context(enabled=True)` block.

### Enable Tracing Without Environment Variables

Use `tracing_context` when you want explicit per-block control or cannot rely on process-wide environment variables.

```python
from langsmith import Client, traceable, tracing_context

client = Client(api_key="YOUR_API_KEY")

@traceable(run_type="tool")
def normalize(text: str) -> str:
    return text.strip().lower()

with tracing_context(
    client=client,
    project_name="manual-config",
    enabled=True,
    tags=["local-dev"],
):
    print(normalize("  LangSmith  "))
```

This pattern is useful in notebooks, unit tests, local scripts, and serverless code where environment setup is awkward.

### Manual Span Control With `RunTree`

Use `RunTree` when you need explicit parent/child span control, need to trace code that cannot use decorators, or need to post and patch runs manually.

```python
from langsmith.run_trees import RunTree

run = RunTree(
    name="fetch-context",
    run_type="tool",
    inputs={"query": "langsmith"},
)
run.post()

try:
    output = {"documents": 3}
    run.end(outputs=output)
finally:
    run.patch()
```

For nested spans, create child runs from the parent `RunTree` instead of creating unrelated top-level runs.

## Datasets And Evaluation

LangSmith datasets are the basis for offline evaluation, regression tracking, and reproducible test suites.

Create a dataset and seed examples programmatically:

```python
from langsmith import Client

client = Client()

dataset = client.create_dataset(
    dataset_name="qa-smoke",
    description="Small regression set for answer generation",
)

client.create_examples(
    dataset_id=dataset.id,
    examples=[
        {
            "inputs": {"question": "What is the capital of France?"},
            "outputs": {"answer": "Paris"},
            "metadata": {"source": "seed"},
        },
        {
            "inputs": {"question": "2 + 2"},
            "outputs": {"answer": "4"},
            "metadata": {"source": "seed"},
        },
    ],
)
```

Recommended workflow:

1. Create or sync a dataset with `Client`.
2. Keep example `inputs` and reference `outputs` small and deterministic.
3. Run `evaluate()` or `aevaluate()` against that dataset with your application callable and evaluator functions.
4. Inspect the experiment in LangSmith before promoting model or prompt changes.

Practical note: dataset changes create a new dataset version. If you care about experiment reproducibility, avoid mutating a shared dataset casually.

## Testing Integration

The SDK includes a pytest plugin by default. The `pytest` extra adds richer output and request caching.

Minimal test instrumentation:

```python
import pytest
from langsmith import testing as t

@pytest.mark.langsmith
def test_translation():
    t.log_inputs({"text": "hola"})
    t.log_reference_outputs({"translation": "hello"})
    t.log_outputs({"translation": "hello"})
    assert True
```

Run with a named suite:

```bash
export LANGSMITH_TEST_SUITE="smoke"
pytest tests/
```

Useful test settings:

- `LANGSMITH_TEST_TRACKING=false` disables LangSmith reporting while leaving the test code intact.
- `LANGSMITH_TEST_CACHE=tests/cassettes` enables request caching for slower or non-deterministic external calls.
- Use `pytest --langsmith-output` for rich console output. The older `--output=langsmith` form only applies to much older SDK versions.

## Sensitive Data Controls

If traces may contain secrets or user content, use the SDK's masking controls instead of disabling tracing entirely.

Available options include:

- Environment-based hiding for inputs and outputs.
- `Client(...)` masking hooks such as `hide_inputs`, `hide_outputs`, and `hide_metadata`.
- Selective tracing scopes with `tracing_context(enabled=False)` around code that should never emit traces.

Prefer masking over manual string scrubbing inside business logic.

## Async And OpenTelemetry

- Use `AsyncClient` if your app already interacts with the LangSmith API from async code and you want to avoid wrapping sync client calls in executors.
- If your app already emits OpenTelemetry spans, install `langsmith[otel]` and enable LangSmith's OTEL bridge instead of duplicating instrumentation.
- The OTEL setup docs require `langsmith>=0.3.18` and recommend `>=0.4.25`; `0.7.16` satisfies that guidance.

## Common Pitfalls

- `Client()` configures access to the LangSmith API. It does not replace model providers such as OpenAI or Anthropic.
- Wrapped clients still require tracing to be enabled. A wrapped client alone does not send traces.
- The web app URL (`smith.langchain.com`) and the API endpoint (`api.smith.langchain.com`) are different surfaces.
- `LANGSMITH_WORKSPACE_ID` is only needed when one API key can access more than one workspace.
- The hosted docs are not version-pinned per release. For `0.7.16`, prefer features already reflected in the current tracing, testing, and evaluation guides and avoid copying stale third-party snippets.

## Version-Sensitive Notes For `0.7.16`

- `0.7.16` is the current PyPI release reflected in the upstream package metadata as of 2026-03-12.
- Python 3.9 is not supported. Use Python 3.10 or newer.
- The pytest plugin is bundled with the SDK, but richer pytest output and caching helpers come from the `pytest` extra.
- OpenTelemetry support guidance in the official docs applies to this version because `0.7.16` is newer than the documented minimums.

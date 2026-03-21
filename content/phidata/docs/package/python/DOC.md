---
name: package
description: "phidata Python package guide for building agents, tools, knowledge bases, and playground apps"
metadata:
  languages: "python"
  versions: "2.7.10"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "phidata,agents,llm,rag,playground,python"
---

# phidata Python Package Guide

## What This Covers

This guide is for the PyPI package `phidata` version `2.7.10`.

- Install name: `phidata`
- Primary import namespace: `phi`
- Docs root: `https://docs.phidata.com/`
- Package registry: `https://pypi.org/project/phidata/`

`phidata` is the pre-Agno package line for building agents, teams, tools, knowledge-backed assistants, and local playground apps. The official repo now points new users to Agno, but projects pinned to `phidata==2.7.10` should still use the `phidata` package and `phi.*` imports documented here.

## Install

Install the pinned package plus the model provider you plan to call:

```bash
pip install phidata==2.7.10 openai
```

Common optional dependencies from the official examples:

```bash
pip install duckduckgo-search
pip install lancedb tantivy pypdf sqlalchemy
pip install "fastapi[standard]"
```

PyPI also exposes extras such as `aws`, `docker`, `k8s`, `server`, and `all` if you want a broader integration bundle, but most projects should install only the provider and tool packages they actually use.

## Minimal Agent

Set your model provider credentials first:

```bash
export OPENAI_API_KEY=your-key
```

Basic synchronous agent:

```python
from phi.agent import Agent
from phi.model.openai import OpenAIChat

agent = Agent(
    model=OpenAIChat(id="gpt-4o"),
    markdown=True,
)

agent.print_response("Explain how retries work in HTTP clients.", stream=True)
```

What matters:

- Use `Agent`, not the older `Assistant` class.
- Pass `model=...`, not the older `llm=...` argument.
- The package does not ship model-provider SDKs for you; install `openai`, `anthropic`, or other provider packages separately.

## Add Tools

Official examples use tool objects that the agent can call during a run:

```python
from phi.agent import Agent
from phi.model.openai import OpenAIChat
from phi.tools.duckduckgo import DuckDuckGo

agent = Agent(
    model=OpenAIChat(id="gpt-4o"),
    tools=[DuckDuckGo()],
    instructions=["Cite sources in plain language."],
    show_tool_calls=True,
    markdown=True,
)

agent.print_response("What changed in Python 3.12?", stream=True)
```

Practical notes:

- If a tool import fails, you are usually missing a separate dependency such as `duckduckgo-search`.
- `show_tool_calls=True` is useful while iterating because it makes tool execution visible in logs and terminal output.
- Keep instructions short; the framework already handles tool orchestration.

## Add Knowledge / RAG

The docs show knowledge bases backed by a vector database. A common setup uses `PDFUrlKnowledgeBase` with LanceDB:

```python
from phi.agent import Agent
from phi.knowledge.pdf import PDFUrlKnowledgeBase
from phi.model.openai import OpenAIChat
from phi.vectordb.lancedb import LanceDb, SearchType

knowledge_base = PDFUrlKnowledgeBase(
    urls=["https://phi-public.s3.amazonaws.com/recipes/ThaiRecipes.pdf"],
    vector_db=LanceDb(
        table_name="recipes",
        uri="tmp/lancedb",
        search_type=SearchType.vector,
    ),
)

knowledge_base.load(upsert=True)

agent = Agent(
    model=OpenAIChat(id="gpt-4o"),
    knowledge=knowledge_base,
    search_knowledge=True,
    markdown=True,
)

agent.print_response("How do I make green curry?", stream=True)
```

Use this pattern when you need retrieval over PDFs or other indexed data instead of stuffing raw documents into the prompt.

## Persist Sessions And Memory

Use storage when you want resumable sessions or chat history across runs:

```python
from phi.agent import Agent
from phi.model.openai import OpenAIChat
from phi.storage.agent.sqlite import SqlAgentStorage

storage = SqlAgentStorage(
    table_name="agent_sessions",
    db_file="tmp/agents.db",
)

agent = Agent(
    model=OpenAIChat(id="gpt-4o"),
    storage=storage,
    add_history_to_messages=True,
    markdown=True,
)
```

This is the simplest official persistence path for local development. SQLite is fine for local apps; switch storage backends only when you need shared or remote state.

## Run A Local Playground App

For a browser UI around one or more agents:

```python
from phi.agent import Agent
from phi.model.openai import OpenAIChat
from phi.playground import Playground, serve_playground_app

agent = Agent(
    name="assistant",
    model=OpenAIChat(id="gpt-4o"),
    markdown=True,
)

app = Playground(agents=[agent]).get_app()

if __name__ == "__main__":
    serve_playground_app("playground:app", reload=True)
```

Install the serving dependencies first:

```bash
pip install "fastapi[standard]" sqlalchemy
```

## Config And Auth

There are two auth layers to keep straight:

1. Model/provider auth.
   Set the environment variables required by the provider you instantiate, such as `OPENAI_API_KEY`.

2. Phidata platform auth.
   `phi auth` and `PHI_API_KEY` are only needed for Phidata-hosted features such as the hosted playground or monitoring. They are not required for a plain local `Agent(...)` that only calls a model provider.

If code works in a local script but the playground or monitoring flow fails, you are usually missing the Phidata platform login rather than the model key.

## Common Pitfalls

- Install/import mismatch: the package is `phidata`, but most imports start with `phi.`.
- Missing provider package: `phidata` does not replace `openai`, `anthropic`, or other model SDKs.
- Missing tool dependencies: toolkit examples often require extra packages that are not installed automatically.
- Old examples from blogs: many older snippets use `Assistant`, `llm=`, or `knowledge_base=`. For `2.7.10`, prefer `Agent`, `model=`, and `knowledge=`.
- Assuming the docs rename means the package renamed too: the repo now says "Phidata is now Agno", but the pinned PyPI package for this entry is still `phidata==2.7.10`.

## Version-Sensitive Notes For 2.7.10

The official migration guide for `2.5.0` is still relevant to `2.7.10` because the newer package line keeps those renamed APIs:

- `Assistant` was renamed to `Agent`.
- `llm` was renamed to `model`.
- `knowledge_base` was renamed to `knowledge`.
- `run()` examples were updated toward `print_response()` for terminal-style usage.

When working on an older codebase, scan for those names first before assuming the code and docs are aligned.

## Official Sources

- Docs root: https://docs.phidata.com/
- Agent docs: https://docs.phidata.com/agents
- Knowledge docs: https://docs.phidata.com/agents/knowledge
- Playground docs: https://docs.phidata.com/playground/introduction
- Migration guide: https://docs.phidata.com/migration/2-5-0
- PyPI package page: https://pypi.org/project/phidata/
- Official repository: https://github.com/bz-e/phidata

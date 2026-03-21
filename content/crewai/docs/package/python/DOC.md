---
name: package
description: "CrewAI Python package guide for building agent crews, tasks, and flows"
metadata:
  languages: "python"
  versions: "1.10.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "crewai,agents,llm,workflow,automation,python"
---

# CrewAI Python Package Guide

## What It Is

`crewai` is the core Python package for building multi-agent workflows with:

- `Agent`, `Task`, and `Crew` for agent orchestration
- `Flow` for event-driven stateful workflows
- `LLM` for explicit model/provider configuration

Use it when you need Python-native orchestration around LLM-backed agents rather than a hosted SaaS API.

## Installation

Install the core package:

```bash
pip install crewai==1.10.1
```

Install optional bundled tools support if you need imports from `crewai_tools`:

```bash
pip install "crewai[tools]==1.10.1"
```

If you want the CLI as a tool-managed binary, the official install page also shows:

```bash
uv tool install crewai
```

## LLM Setup and Authentication

CrewAI routes model calls through LiteLLM. In practice, that means you should set provider credentials in the environment and preferably set the model explicitly instead of relying on defaults.

OpenAI example:

```bash
export OPENAI_API_KEY="your-api-key"
export MODEL="openai/gpt-4o-mini"
```

Explicit Python setup:

```python
import os
from crewai import LLM

llm = LLM(
    model=os.environ.get("MODEL", "openai/gpt-4o-mini"),
    api_key=os.environ["OPENAI_API_KEY"],
)
```

Notes:

- The docs show provider/model strings such as `openai/gpt-4o-mini`.
- Different providers need their own environment variables or credentials.
- If you use hierarchical crews, configure `manager_llm` explicitly.

## Fastest Way to Start a Project

The official quickstart uses the CLI to scaffold a crew project:

```bash
crewai create crew latest_ai_development
cd latest_ai_development
crewai install
```

That scaffold gives you:

- YAML config files such as `config/agents.yaml` and `config/tasks.yaml`
- a `@CrewBase` class with `@agent`, `@task`, and `@crew` decorators
- a `crew().kickoff(inputs=...)` entry point

Use the scaffold when you want a structured app layout. Use the direct Python API when you want to embed CrewAI into an existing codebase.

## Core Usage: Agent + Task + Crew

Minimal direct API example:

```python
import os
from crewai import Agent, Crew, LLM, Process, Task

llm = LLM(
    model=os.environ.get("MODEL", "openai/gpt-4o-mini"),
    api_key=os.environ["OPENAI_API_KEY"],
)

researcher = Agent(
    role="Research Analyst",
    goal="Find the most relevant facts about {topic}",
    backstory="You turn raw information into concise briefings.",
    llm=llm,
    verbose=True,
)

research_task = Task(
    description="Research {topic} and collect the most important facts.",
    expected_output="A short bullet list with the key findings.",
    agent=researcher,
)

crew = Crew(
    agents=[researcher],
    tasks=[research_task],
    process=Process.sequential,
    verbose=True,
)

result = crew.kickoff(inputs={"topic": "CrewAI flows"})
print(result)
```

Important defaults:

- `Process.sequential` is the safest starting point.
- Pass `inputs={...}` to fill `{variables}` referenced by agents/tasks/YAML config.
- Set `llm` explicitly on agents to avoid model-default ambiguity.

## Multi-Task Context Passing

Task context is task-to-task, not free-form text. Pass earlier `Task` objects in `context=[...]` so downstream tasks can use upstream results.

```python
from crewai import Agent, Task

writer = Agent(
    role="Technical Writer",
    goal="Turn research into a concise memo.",
    backstory="You write clear summaries for engineering teams.",
    llm=llm,
)

research_task = Task(
    description="Research the latest Python agent frameworks.",
    expected_output="A fact list with citations or source names.",
    agent=researcher,
)

writing_task = Task(
    description="Turn the research into a concise recommendation memo.",
    expected_output="A short memo with a final recommendation.",
    agent=writer,
    context=[research_task],
)
```

## Hierarchical Crews

Use hierarchical orchestration when you want a manager model to delegate work across agents. Configure the manager model directly:

```python
import os
from crewai import Crew, LLM, Process

manager_llm = LLM(model="openai/gpt-4o-mini", api_key=os.environ["OPENAI_API_KEY"])

crew = Crew(
    agents=[researcher, writer],
    tasks=[research_task, writing_task],
    process=Process.hierarchical,
    manager_llm=manager_llm,
)
```

If `process=Process.hierarchical` is set without a valid manager model, expect runtime issues.

## Tools

CrewAI docs and examples often import tools from `crewai_tools`, not from `crewai` itself. Install the optional tools extra when needed, then attach tool instances to agents:

```python
from crewai import Agent
from crewai_tools import SerperDevTool

search_tool = SerperDevTool()

researcher = Agent(
    role="Research Analyst",
    goal="Find relevant information on demand.",
    backstory="You search first, then summarize.",
    llm=llm,
    tools=[search_tool],
)
```

Many tools also require provider-specific API keys. Check the matching tool page before assuming a tool is ready after package installation alone.

## Flows

Use `Flow` when your workflow is stateful and event-driven rather than a single crew execution. The official docs model flows with decorators such as `@start()` and `@listen(...)`.

```python
from pydantic import BaseModel, Field
from crewai.flow.flow import Flow, listen, start

class ReportState(BaseModel):
    topic: str = "CrewAI"
    notes: list[str] = Field(default_factory=list)

class ResearchFlow(Flow[ReportState]):
    @start()
    def fetch(self):
        self.state.notes.append(f"Researching {self.state.topic}")

    @listen(fetch)
    def summarize(self):
        return "\n".join(self.state.notes)

result = ResearchFlow().kickoff()
print(result)
```

Use flows when you need branching, routing, or persisted workflow state. Use a plain crew when a single agent pipeline is enough.

## Generated Project Pattern

If you use the official scaffold, expect code shaped like this:

```python
from crewai import Agent, Crew, Process, Task
from crewai.project import CrewBase, agent, crew, task

@CrewBase
class LatestAiDevelopment:
    agents_config = "config/agents.yaml"
    tasks_config = "config/tasks.yaml"

    @agent
    def researcher(self) -> Agent:
        return Agent(config=self.agents_config["researcher"])

    @task
    def research_task(self) -> Task:
        return Task(config=self.tasks_config["research_task"])

    @crew
    def crew(self) -> Crew:
        return Crew(
            agents=self.agents,
            tasks=self.tasks,
            process=Process.sequential,
            verbose=True,
        )
```

This is the repo-friendly pattern to use if you want YAML-driven configuration and generated structure instead of hand-built Python objects.

## Common Pitfalls

- Do not assume the model default is stable. The docs currently reference both `MODEL=...` and older OpenAI-default language. Set `llm` explicitly.
- Do not forget `expected_output` on tasks. Official task examples include it consistently.
- Do not pass raw strings into `context`. Pass upstream `Task` objects.
- Do not import bundled tools from `crewai` if the docs example imports from `crewai_tools`.
- Do not start with hierarchical crews unless you actually need a manager model.
- Do not mix scaffolded YAML names and Python placeholders carelessly. Inputs passed to `kickoff(inputs=...)` must match the placeholders used by your config.

## Version-Sensitive Notes for 1.10.1

- PyPI shows `1.10.1` as the current release for `crewai`, released on `2025-10-27`.
- PyPI also marks `1.10.0` as yanked, with the note `Issue with wheel file not installing`.
- The docs site header is on `v1.10.1`, so the docs root and PyPI currently agree on the package version.
- The docs changelog page currently documents releases through `1.9.0`, so use PyPI release metadata for exact `1.10.1` release confirmation.
- The installation page still contains a legacy note mentioning `CrewAI 0.175.0` in its OpenAI SDK section. Treat that as stale page content, not as the package version.

## Official Source URLs

- Docs root: https://docs.crewai.com/
- Installation: https://docs.crewai.com/en/installation
- Quickstart: https://docs.crewai.com/quickstart
- Agents: https://docs.crewai.com/en/concepts/agents
- Tasks: https://docs.crewai.com/en/concepts/tasks
- Crews: https://docs.crewai.com/en/concepts/crews
- Flows: https://docs.crewai.com/en/concepts/flows
- LLMs: https://docs.crewai.com/en/learn/llms
- Changelog: https://docs.crewai.com/en/changelog
- PyPI project: https://pypi.org/project/crewai/
- PyPI JSON metadata: https://pypi.org/pypi/crewai/json

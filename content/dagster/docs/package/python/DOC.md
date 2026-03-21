---
name: package
description: "Dagster Python package guide for asset-based orchestration, local development, jobs, schedules, and structured config"
metadata:
  languages: "python"
  versions: "1.12.18"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dagster,python,data-orchestration,assets,pipelines,schedules,sensors"
---

# Dagster Python Package Guide

## Golden Rule

Use `dagster` as the core programming model, expose a top-level `defs = dg.Definitions(...)` object in a loadable module, and build around software-defined assets unless you have a clear reason to stay in the older op/graph model.

For a local developer experience that matches the official docs, install the companion CLI and webserver packages alongside `dagster`.

## Install

If you only need the core library in an existing project:

```bash
python -m pip install "dagster==1.12.18"
```

For the full local OSS workflow from the official quickstart, install the core package plus the local UI and `dg` CLI:

```bash
python -m pip install \
  "dagster==1.12.18" \
  "dagster-webserver==1.12.18" \
  "dagster-dg-cli==1.12.18"
```

Equivalent `uv` install:

```bash
uv add dagster dagster-webserver dagster-dg-cli
```

## Initialize A Project

The official scaffolder creates a modern project layout with `src/<project_name>/definitions.py`, a `defs/` package, `tests/`, and `pyproject.toml`:

```bash
create-dagster project my_dagster_project
cd my_dagster_project
dg dev
```

If you already have a Python package and just want to add Dagster manually, the important part is a loadable module with a top-level `Definitions` instance:

```text
src/
  my_project/
    __init__.py
    definitions.py
```

```python
# src/my_project/definitions.py
import dagster as dg

@dg.asset
def hello() -> str:
    return "hello"

defs = dg.Definitions(assets=[hello])
```

Then run the local instance by pointing Dagster at that module:

```bash
dg dev -m my_project.definitions
```

## Core Usage

### Minimal asset-based project

This is the main Dagster shape to remember:

```python
import dagster as dg

class ApiConfig(dg.Config):
    base_url: str
    dataset: str = "events"

class ApiResource(dg.ConfigurableResource):
    token: str

    def fetch(self, dataset: str) -> list[dict]:
        # Replace with real HTTP or SDK logic.
        return [{"dataset": dataset, "status": "ok"}]

@dg.asset
def raw_events(config: ApiConfig, api: ApiResource) -> list[dict]:
    return api.fetch(config.dataset)

@dg.asset
def event_count(raw_events: list[dict]) -> int:
    return len(raw_events)

daily_assets = dg.define_asset_job("daily_assets", selection=[raw_events, event_count])

@dg.schedule(
    cron_schedule="0 6 * * *",
    target=daily_assets,
    execution_timezone="America/Los_Angeles",
)
def daily_schedule():
    return dg.RunRequest()

defs = dg.Definitions(
    assets=[raw_events, event_count],
    jobs=[daily_assets],
    schedules=[daily_schedule],
    resources={
        "api": ApiResource(token=dg.EnvVar("API_TOKEN")),
    },
)
```

Key behaviors that matter in practice:

- `@dg.asset` defines a persistent data asset, and Dagster infers upstream asset dependencies from function arguments.
- `dg.Definitions(...)` is the object Dagster tools load. If the module does not expose a top-level `Definitions` instance, `dg dev` and other tooling will not find your code.
- `ConfigurableResource` is the structured resource API for external systems.
- `Config` is the structured config API for assets and ops.
- `define_asset_job(...)` is the usual way to turn an asset selection into a runnable job.
- `@dg.schedule` automates runs with cron and can target a job or asset selection.

### Validate before starting the UI

Use the CLI to catch loadability and dependency problems early:

```bash
dg check defs
```

Useful local inspection commands:

```bash
dg list defs
dg list envs
```

### Test definitions in Python

For unit tests, validate that your definitions can load cleanly:

```python
from my_project.definitions import defs

def test_defs_loadable() -> None:
    defs.validate_loadable()
```

## Configuration And Secrets

Dagster OSS itself does not require a package-level auth step. Authentication usually belongs to the systems your assets talk to, such as databases, cloud APIs, warehouses, or Dagster+.

Use Dagster config and resource APIs for runtime settings:

- `dg.Config` for structured asset or op config
- `dg.ConfigurableResource` for typed resources
- `dg.EnvVar("NAME")` for secrets in resources or config
- `StringSource`, `IntSource`, and similar legacy config types when you are still on legacy config schemas

Example:

```python
import dagster as dg

class WarehouseResource(dg.ConfigurableResource):
    username: str
    password: str

defs = dg.Definitions(
    resources={
        "warehouse": WarehouseResource(
            username=dg.EnvVar("WAREHOUSE_USER"),
            password=dg.EnvVar("WAREHOUSE_PASSWORD"),
        )
    }
)
```

Important secret-handling rule: job config dictionaries and `configured(...)` values can be visible in the Dagster UI. Do not hardcode secrets into run config, schedules, or constant resource configuration.

## Common Pitfalls

- A loadable Dagster module must expose a top-level `Definitions` instance. A helper function that returns `Definitions` is not enough unless the CLI is pointed at it appropriately.
- Assets infer dependencies from function parameters. Renaming a parameter changes the upstream asset key relationship unless you configure it explicitly.
- Jobs created with `@job` do not respect the `resources=` argument on `Definitions`. Bind resources at job creation time for op jobs, or prefer asset jobs when possible.
- `dg check defs` should be part of local validation. It is faster to catch missing resources, conflicting asset keys, or unresolved asset jobs there than in the UI.
- Use environment variables or Dagster config sources for secrets. Values supplied as constant job config can show up in the UI.
- `execute_in_process()` is useful for tests, but it swaps execution to the in-process executor and changes the default IO manager behavior to in-memory. Do not assume it behaves exactly like your production execution environment.
- Dagster schedules are cron-based and timezone-aware. Set `execution_timezone` explicitly when local time matters.

## Version-Sensitive Notes For 1.12.18

- PyPI currently lists `dagster 1.12.18` released on March 5, 2026.
- Dagster's docs are not perfectly version-aligned right now: several API pages still show `Latest (1.12.8)` in the site chrome, while the CLI reference and GitHub releases reflect `1.12.18`. Trust PyPI and the release feed for the actual package version.
- The official 1.12.18 release notes mention a fix for nested resource attributes annotated with `dagster.ResourceDependency` during parent resource setup. If you use nested resources, stay on at least `1.12.18`.
- `run_request_for_partition()` is marked deprecated in the jobs API and scheduled for removal in `2.0.0`; use `dg.RunRequest(partition_key=...)` directly instead.

## Official Sources Used

- Docs root: https://docs.dagster.io/api
- Definitions API: https://docs.dagster.io/api/dagster/definitions
- Assets API: https://docs.dagster.io/api/dagster/assets
- Config API: https://docs.dagster.io/api/dagster/config
- Resources API: https://docs.dagster.io/api/dagster/resources
- Schedules and sensors API: https://docs.dagster.io/api/dagster/schedules-sensors
- CLI reference: https://docs.dagster.io/api/clis
- `dg` CLI reference: https://docs.dagster.io/api/clis/dg-cli/dg-cli-reference
- `create-dagster` CLI reference: https://docs.dagster.io/api/clis/create-dagster
- PyPI package page: https://pypi.org/project/dagster/
- GitHub releases: https://github.com/dagster-io/dagster/releases

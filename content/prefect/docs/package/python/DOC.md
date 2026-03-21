---
name: package
description: "Prefect Python workflow orchestration package for flows, tasks, scheduling, deployments, workers, and configuration"
metadata:
  languages: "python"
  versions: "3.6.21"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "prefect,python,workflow,orchestration,pipelines,scheduling,automation"
---

# Prefect Python Package Guide

## Golden Rule

Use the full `prefect` package for Python workflow orchestration, define workflows with `@flow` and `@task`, and connect the runtime to a real Prefect API when you need deployments, workers, schedules, blocks, variables, or the UI. Running a flow function directly works locally, but orchestration features depend on either Prefect Cloud or a self-hosted Prefect server.

## Install

Pin the version your project expects:

```bash
python -m pip install "prefect==3.6.21"
```

Common alternatives:

```bash
uv add "prefect==3.6.21"
poetry add "prefect==3.6.21"
```

Sanity-check the install:

```bash
prefect version
python -c "import prefect; print(prefect.__version__)"
```

Notes:

- Install `prefect`, not just `prefect-client`, when you need decorators, CLI commands, local orchestration features, workers, or server integration.
- Provider integrations are separate packages. If your flow needs AWS, GCP, Databricks, dbt, Docker, Kubernetes, or similar integrations, install the matching `prefect-*` package instead of assuming core `prefect` includes every block or worker type.

## Initialize And Connect To An API

### Local development with a self-hosted server

Start the local API and UI:

```bash
prefect server start
```

Point your current profile at that API:

```bash
prefect config set PREFECT_API_URL="http://127.0.0.1:4200/api"
prefect config view --show-defaults
```

### Prefect Cloud

The simplest path is the login flow from the official docs:

```bash
prefect cloud login
```

If you are scripting setup instead of using the interactive login, configure the workspace API URL and API key through environment variables or profile settings:

```bash
export PREFECT_API_URL="https://api.prefect.cloud/api/accounts/<account-id>/workspaces/<workspace-id>"
export PREFECT_API_KEY="pnu_..."
```

### Profiles And Settings Files

Useful profile commands:

```bash
prefect profile ls
prefect profile create local-server
prefect profile use local-server
prefect config view
```

Practical configuration guidance:

- Use profiles to keep local, CI, staging, and production settings separate.
- Use environment variables for secrets and CI overrides.
- Prefect can also read project configuration from `.env`, `prefect.toml`, or `[tool.prefect]` in `pyproject.toml`.
- Settings precedence is: environment variables, `.env`, `prefect.toml`, `pyproject.toml`, active profile, then defaults.
- When something behaves unexpectedly, inspect the effective settings with `prefect config view` before changing code.

## Core Usage

### Define tasks and flows

```python
from datetime import timedelta

from prefect import flow, task
from prefect.cache_policies import INPUTS

@task(
    retries=3,
    retry_delay_seconds=5,
    cache_policy=INPUTS,
    cache_expiration=timedelta(minutes=30),
    persist_result=True,
)
def fetch_customer(customer_id: str) -> dict[str, str]:
    print(f"Fetching {customer_id}")
    return {"customer_id": customer_id, "status": "ok"}

@flow(name="sync-customers", log_prints=True)
def sync_customers(customer_ids: list[str]) -> list[dict[str, str]]:
    return [fetch_customer(customer_id) for customer_id in customer_ids]

if __name__ == "__main__":
    print(sync_customers(["cust-1", "cust-2"]))
```

Why this pattern matters:

- Put retry policy on tasks, not only on the surrounding flow.
- Use `persist_result=True` when you expect caching or downstream result reuse to matter.
- `log_prints=True` is useful during development because plain `print()` output is attached to the flow run logs.

### Serve a flow from a long-running process

Use `.serve()` when the same Python environment will stay alive and register scheduled or ad hoc runs itself:

```python
if __name__ == "__main__":
    sync_customers.serve(
        name="sync-customers-local",
        cron="0 * * * *",
    )
```

`serve()` is good for a developer machine, a VM, or a simple service process. Keep that process running or the schedule will stop being polled.

### Deploy to a work pool for remote execution

Use deployments and workers when execution should happen in managed infrastructure instead of the process that created the schedule:

```python
if __name__ == "__main__":
    sync_customers.deploy(
        name="hourly",
        work_pool_name="docker-pool",
        image="ghcr.io/acme/prefect-sync:latest",
        push=False,
        cron="0 * * * *",
    )
```

Then start a worker for that pool:

```bash
prefect worker start --pool docker-pool
```

Trigger the deployment manually when needed:

```bash
prefect deployment run "sync-customers/hourly"
```

Rule of thumb:

- `flow()` call: just run Python now
- `.serve()`: schedule from the current long-lived process
- `.deploy()`: register remote execution for workers in a work pool

## Configuration, Variables, And Secrets

### Non-secret runtime configuration

Use Variables for lightweight, JSON-serializable runtime values that may change without a code redeploy:

```python
from prefect.variables import Variable

api_base_url = Variable.get("customer-api-base-url", default="https://api.example.com")
```

Variables are not the right place for credentials.

### Secrets and reusable connection objects

Use blocks for sensitive values and reusable infrastructure or credential configuration:

```python
from prefect.blocks.system import Secret

token = Secret.load("customer-api-token").get()
```

Blocks are better than hard-coded secrets because they can be updated in Prefect without editing the flow source.

## Common Pitfalls

- Flows can run without an API, but deployments, workers, blocks, variables, automations, and the UI require `PREFECT_API_URL` to point at a real server or Cloud workspace.
- A served flow does nothing after registration unless the serving process keeps running.
- A deployment does nothing unless a worker is running for the target work pool.
- Do not store credentials in Variables. Use Secret blocks or environment variables.
- Caching often disappoints when agents forget `persist_result=True` or expect local result storage to be shared across machines.
- Fresh installs start on the `ephemeral` profile unless you configure another one. Set an explicit `PREFECT_API_URL` when you want persistent orchestration state.
- Old Prefect 2 blog posts still mention agents. For Prefect 3, prefer current work-pool and worker guidance from the v3 docs.
- Installing only `prefect-client` is a common mistake when the project actually needs the orchestration package and CLI.
- Integration-specific blocks or workers will not import until the related `prefect-*` integration package is installed.

## Version-Sensitive Notes For Prefect 3.6.21

- PyPI lists `prefect 3.6.21` as the current package version covered here.
- The docs URL used `https://docs.prefect.io/latest/`; for authoring, the stable canonical docs root is `https://docs.prefect.io/v3/`.
- Some official docs examples still show older sample output such as `prefect-3.4.24` during installation. Treat that as example output, not the current package version.
- Project-level config in `prefect.toml` and `[tool.prefect]` in `pyproject.toml` requires Prefect `>=3.1`.
- Loading config from `.env` requires Prefect `>=3.0.5`.
- When you find examples built around Prefect 2 agents or older deployment flows, translate them to Prefect 3 work pools and workers before copying them into production code.

## Official Sources Used

- Docs root: `https://docs.prefect.io/latest/`
- Installation: `https://docs.prefect.io/v3/get-started/install`
- Quickstart: `https://docs.prefect.io/v3/get-started/quickstart`
- Connect to Prefect Cloud: `https://docs.prefect.io/v3/manage/cloud/connect-to-cloud`
- Settings and profiles: `https://docs.prefect.io/v3/concepts/settings-and-profiles`
- Blocks: `https://docs.prefect.io/v3/concepts/blocks`
- Variables: `https://docs.prefect.io/v3/concepts/variables`
- PyPI package page: `https://pypi.org/project/prefect/`

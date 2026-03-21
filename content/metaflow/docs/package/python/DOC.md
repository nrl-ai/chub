---
name: package
description: "Metaflow package guide for Python flows, local iteration, remote execution, and production orchestration"
metadata:
  languages: "python"
  versions: "2.19.20"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "metaflow,python,ml,workflow,orchestration,batch,kubernetes"
---

# Metaflow Python Package Guide

## Golden Rule

Use Metaflow's own flow model and CLI instead of wrapping it in an unrelated job runner. A Metaflow program is a `FlowSpec` subclass with `@step` methods, run with commands like `python myflow.py run`.

As of March 12, 2026, the version used here `2.19.20` matches the current PyPI release.

## Install

Use a virtual environment or project manager and pin the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "metaflow==2.19.20"
```

Alternatives supported by upstream:

```bash
conda install -c conda-forge metaflow
uv add metaflow
```

Metaflow can run locally without any cloud setup. Remote compute, shared metadata, and production scheduling require additional infrastructure configuration.

## First Flow

Metaflow flows are Python classes inheriting from `FlowSpec`. Each workflow step is a method decorated with `@step`, and transitions are declared with `self.next(...)`.

```python
from metaflow import FlowSpec, Parameter, current, step

class HelloFlow(FlowSpec):
    name = Parameter("name", default="world", help="Who to greet")

    @step
    def start(self):
        self.message = f"hello, {self.name}"
        self.next(self.end)

    @step
    def end(self):
        print(self.message)
        print(current.pathspec)

if __name__ == "__main__":
    HelloFlow()
```

Common local commands:

```bash
python hello_flow.py show
python hello_flow.py run
python hello_flow.py run --name metaflow
python hello_flow.py resume
```

Rules that matter:

- Every flow needs a `start` step and an `end` step.
- `self.next(...)` should be the last statement in every non-terminal step.
- Values assigned to `self.<name>` become artifacts available in downstream steps.

## Parameters, Files, And Branching

Use `Parameter` for CLI-provided inputs and `IncludeFile` when the input should be read from a local file and stored as an artifact.

```python
from metaflow import FlowSpec, IncludeFile, Parameter, step

class TrainFlow(FlowSpec):
    sample = Parameter("sample", default=100, type=int)
    config_text = IncludeFile("config", default="config.json")

    @step
    def start(self):
        self.rows = list(range(self.sample))
        self.next(self.end)

    @step
    def end(self):
        print(len(self.rows), self.config_text[:40])

if __name__ == "__main__":
    TrainFlow()
```

Useful flow patterns from the API:

- Straight line: `self.next(self.train)`
- Static fan-out: `self.next(self.a, self.b)`
- Foreach: `self.next(self.worker, foreach="items")`
- Conditional routing: `self.next({"small": self.fast, "large": self.slow}, condition="route")`

## Dependencies And Environments

For step-specific dependencies, use `@pypi` or `@conda`. Upstream documents them as interchangeable patterns for isolated execution environments; in practice, choose one approach per step.

```python
from metaflow import FlowSpec, pypi, step

class PandasFlow(FlowSpec):
    @pypi(
        python="3.11.9",
        packages={
            "pandas": "2.2.3",
            "pyarrow": "18.1.0",
        },
    )
    @step
    def start(self):
        import pandas as pd

        self.rows = len(pd.DataFrame({"x": [1, 2, 3]}))
        self.next(self.end)

    @step
    def end(self):
        print(self.rows)

if __name__ == "__main__":
    PandasFlow()
```

If your project already uses `uv`, Metaflow supports it directly:

```bash
uv init myflow
cd myflow
uv add metaflow pandas
uv run flow.py run
uv run flow.py --environment=uv run
```

To avoid repeating the flag:

```bash
export METAFLOW_ENVIRONMENT=uv
```

Use `uv` when you want project-level dependency management. Use `@pypi` or `@conda` when you want Metaflow-managed step environments with stronger reproducibility for remote tasks.

## Working With Data And Run Context

Artifacts are persisted automatically when assigned to `self`. Inside a running step, inspect the active run with `current`.

```python
from metaflow import FlowSpec, current, step

class ContextFlow(FlowSpec):
    @step
    def start(self):
        self.values = [1, 2, 3]
        print(current.flow_name, current.run_id, current.step_name)
        print(current.pathspec)
        self.next(self.end)

    @step
    def end(self):
        print(sum(self.values))

if __name__ == "__main__":
    ContextFlow()
```

`current` is only available during flow execution. Use it for run IDs, task IDs, retry counts, cards, and project metadata.

For large datasets already in S3, use Metaflow's S3 utilities instead of pulling everything through ordinary local temp files. For normal step-to-step data, prefer persisted artifacts on `self`.

## Inspect Past Runs

Metaflow ships a client API for querying runs, steps, tasks, artifacts, and tags. The object hierarchy is centered on pathspecs such as `FlowName/12/start/1`.

```python
from metaflow import Flow, Run

flow = Flow("HelloFlow")
run = flow.latest_successful_run
print(run.pathspec)
print(run.tags)

same_run = Run(run.pathspec)
print(same_run.created_at)
```

Important behavior:

- Client API visibility is filtered by the current namespace.
- The client uses the currently configured metadata provider.
- If you "cannot find" runs that exist, check namespace and metadata configuration before assuming the run is missing.

## Run Flows Programmatically

Use `Runner` when another Python process needs to launch or resume flows.

```python
from metaflow import Runner

with Runner("hello_flow.py", profile="default") as runner:
    execution = runner.run(name="metaflow")
    print(execution.run.pathspec)
```

Notes:

- `Runner` and `ExecutingRun` are designed to be used as context managers.
- If you do not use a context manager, call `cleanup()` yourself.
- `NBRunner` is the notebook-oriented equivalent.

## Configuration And Profiles

Metaflow supports structured flow configuration through `Config`, CLI flags, and environment variables.

```python
import time
from metaflow import Config, FlowSpec, step, timeout

class TimeoutConfigFlow(FlowSpec):
    config = Config("config", default="myconfig.json")

    @timeout(seconds=config.timeout)
    @step
    def start(self):
        time.sleep(1)
        self.next(self.end)

    @step
    def end(self):
        print(self.config)

if __name__ == "__main__":
    TimeoutConfigFlow()
```

Pass config at runtime:

```bash
python timeout_config.py --config-value config '{"timeout": 3}' run
```

Or through environment variables:

```bash
export METAFLOW_FLOW_CONFIG_VALUE='{"config": {"timeout": 3}}'
python timeout_config.py run
```

For environment-level switching, Metaflow uses configuration profiles. The official APIs and runner docs refer to `METAFLOW_PROFILE` for selecting the active profile, and `Runner(..., profile="...")` when launching programmatically.

Metadata configuration matters for both the client API and orchestration:

- Local mode stores metadata in the local `.metaflow` directory.
- Remote mode uses Metaflow Service.
- You can switch providers in Python with `metadata(...)`, but the usual pattern is configuring the provider in a profile and selecting it with `METAFLOW_PROFILE`.

## Secrets And Authentication

Do not pass credentials as ordinary Metaflow parameters if they are real secrets. Use `@secrets` and a secrets manager.

```python
import os
from metaflow import FlowSpec, secrets, step

class SecretFlow(FlowSpec):
    @secrets(sources=["my-db-credentials"])
    @step
    def start(self):
        print(os.environ["DB_USER"])
        self.next(self.end)

    @step
    def end(self):
        pass

if __name__ == "__main__":
    SecretFlow()
```

Officially documented secret backends:

- AWS Secrets Manager via `METAFLOW_DEFAULT_SECRETS_BACKEND_TYPE = "aws-secrets-manager"`
- Azure Key Vault via `METAFLOW_DEFAULT_SECRETS_BACKEND_TYPE = "az-key-vault"`
- GCP Secret Manager with the documented GCP secret path and prefix settings

Operational notes:

- `@secrets` exposes secret values through environment variables for the step.
- Access control is handled by the cloud identity in the execution environment.
- For AWS, you can supply a specific IAM role in `@secrets(..., role="arn:...")` when the default role should not read the secret directly.
- For local prototyping, plain environment variables may still be acceptable until you move to shared or production environments.

## Remote Compute And Production

Remote compute is where Metaflow becomes more than a local workflow helper. Cloud execution is available through decorators like `@batch` and `@kubernetes`, but these features require a deployed Metaflow stack first. Upstream recommends Metaflow Sandbox when you want to try the remote model before setting up infrastructure.

High-level AWS model from the official docs:

- Datastore: local directory in local mode, S3 in AWS mode
- Compute: local processes in local mode, AWS Batch in AWS mode
- Metadata: local directory or Metaflow Service
- Scheduling: Step Functions plus EventBridge

For production deployments, use the CLI or `Deployer` API to create scheduler-managed flows.

## Common Pitfalls

- Forgetting `@step`, `start`, or `end` produces broken flows quickly.
- Putting logic after `self.next(...)` is wrong; treat it as the end of the step.
- `spin` is for rapid local step iteration, not for durable runs. It does not persist metadata or artifacts by default, and it does not support remote execution.
- Client API results depend on namespace and metadata provider. Wrong profile is a common cause of "missing" runs.
- Remote `@batch` and `@kubernetes` execution do not work in a plain local-only setup; deploy the stack first.
- Do not rely on ad hoc system packages for remote tasks. Declare dependencies explicitly with `@pypi`, `@conda`, or `--environment=uv`.
- If you launch flows through `Runner` without a context manager, temp files accumulate unless you call `cleanup()`.
- Keep secrets in a secrets manager for shared or production runs; ordinary parameters and checked-in config files are not appropriate for credentials.

## Version-Sensitive Notes For 2.19.20

- PyPI currently lists `2.19.20`, released on February 26, 2026.
- The 2.19 line introduced the `spin` command for iterative single-step development.
- The 2.19.x line also includes recent fixes around dependency packaging and cross-platform PyPI resolution, so avoid copying older blog posts that assume pre-2.19 behavior.
- If you adopt `uv`, remember that remote execution still requires `--environment=uv` or `METAFLOW_ENVIRONMENT=uv`; local `uv run` alone is not enough for cloud tasks.

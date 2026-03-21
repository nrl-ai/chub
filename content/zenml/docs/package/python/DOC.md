---
name: package
description: "ZenML package guide for Python projects using pipelines, stacks, and local or remote ZenML servers"
metadata:
  languages: "python"
  versions: "0.94.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "zenml,mlops,pipelines,orchestration,artifacts"
---

# ZenML Python Package Guide

## What It Is

`zenml` is the Python SDK and CLI for defining ML pipelines with `@step` and `@pipeline`, tracking runs and artifacts, and executing them against a configurable ZenML stack.

For `0.94.0`, treat the package as a client-plus-CLI entry point. Add the right extras depending on whether you want only the client, a local development stack, or a self-hosted server.

## Version Snapshot

- Ecosystem: `pypi`
- Package: `zenml`
- Version covered: `0.94.0`
- Python requirement on PyPI: `>=3.10,<3.14`
- Docs root: `https://docs.zenml.io/`
- Reference landing page: `https://docs.zenml.io/reference`
- Registry page: `https://pypi.org/project/zenml/`

## Install

Install the exact package version used by the project:

```bash
pip install "zenml==0.94.0"
```

For the normal local development experience, install the local extra instead of the minimal base package:

```bash
pip install "zenml[local]==0.94.0"
```

If you need to run a self-hosted ZenML server from the package, use the server extra:

```bash
pip install "zenml[server]==0.94.0"
```

## Initialize A Project

Initialize ZenML once at the repository root:

```bash
zenml init
```

This creates a `.zen` directory and marks the project root as the default source root. That matters because ZenML needs importable pipeline code when it materializes steps, snapshots, and remote runs.

For local development, start the local ZenML dashboard and API server:

```bash
zenml up
```

The local dashboard defaults to `http://127.0.0.1:8237`.

## Minimal Pipeline

```python
from zenml import pipeline, step

@step
def load_values() -> list[int]:
    return [1, 2, 3]

@step
def train(values: list[int]) -> int:
    return sum(values)

@pipeline
def training_pipeline() -> None:
    train(load_values())

if __name__ == "__main__":
    training_pipeline()
```

Run the pipeline with normal Python execution:

```bash
python run_pipeline.py
```

ZenML will register the run, materialize step outputs as artifacts, and show the run in the dashboard if a server is active.

## Core Workflow

1. Install `zenml[local]` for local development unless the project intentionally uses the minimal client-only install.
2. Run `zenml init` from the repo root once.
3. Define functions as `@step` and compose them inside a `@pipeline`.
4. Execute the pipeline from importable project code, not from ad hoc notebook cells that ZenML cannot resolve later.
5. Use the dashboard or CLI to inspect runs, artifacts, stack configuration, and failures.

## Stacks And Execution

ZenML runs pipelines against an active stack. A stack combines components such as an orchestrator and artifact store.

To switch to a different configured stack:

```bash
zenml stack set <STACK_NAME>
```

Use local stack components for local experimentation. For deployed or team workflows, expect to configure non-local components such as a remote artifact store and orchestrator.

## Auth And Server Configuration

### Local server

For local-only work, `zenml up` is the normal entry point. It starts the local ZenML services and opens or serves the dashboard.

### Remote ZenML server

To connect the CLI and SDK to a remote ZenML server:

```bash
zenml login https://<your-zenml-server>
```

For service-account or headless authentication, ZenML also supports API-key login:

```bash
zenml login https://<your-zenml-server> --api-key <YOUR_API_KEY>
```

If the project uses ZenML Pro or a team deployment, verify the target server URL, workspace/project, and active stack before launching runs.

## Source Root And Imports

ZenML needs stable imports for pipeline code. The safest pattern is:

1. Keep pipeline code inside the repository you initialized with `zenml init`.
2. Run entry scripts from that repository root.
3. Avoid relative-import hacks or moving execution to a different working directory after initialization.

If the project structure is unusual, set or correct the source root explicitly in ZenML instead of relying on whatever current working directory happens to be active.

## Common Pitfalls

- Installing only `zenml` and expecting the full local experience. The base package is intentionally slim; use `zenml[local]` when you need local execution and the local dashboard.
- Forgetting `zenml init`. Without the `.zen` directory and source-root registration, imports and run materialization often fail later.
- Treating arbitrary Python inside a pipeline function as a step. Only decorated `@step` functions become first-class ZenML steps with tracked inputs and outputs.
- Writing step code with unstable or non-serializable outputs. Prefer explicit Python types and deterministic return values so ZenML can materialize artifacts cleanly.
- Switching to remote orchestration without checking the active stack. Remote runs usually need more than the default local stack, especially a non-local artifact store.
- Assuming old blog posts still match the current packaging model. The docs and PyPI page now distinguish the minimal base package from extras such as `[local]` and `[server]`.
- On Apple Silicon, local Docker-based workflows can require `OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES` when using `zenml up`.

## Version-Sensitive Notes For 0.94.0

- `0.94.0` is the current ZenML package version on PyPI as of 2026-03-12.
- The official docs site is mostly versionless. Use `0.94.0` in install commands and verify behavior against the installed package if you are copying examples from older issues, blogs, or screenshots.
- The slimmer package model introduced in recent ZenML releases still applies here: plain `zenml` is not the same as `zenml[local]`.
- The docs URL points at the reference area, but the practical setup flow starts from the main docs root and the installation guide.

## Official Sources

- Docs root: https://docs.zenml.io/
- Installation guide: https://docs.zenml.io/getting-started/installation
- Pipeline guide: https://docs.zenml.io/user-guides/starter-guide/create-an-ml-pipeline
- Source root and imports: https://docs.zenml.io/concepts/source-code-management/source-root-and-imports
- Stacks and components: https://docs.zenml.io/stacks/stack-components
- Remote login and API keys: https://docs.zenml.io/how-to/manage-zenml-pro/connect-to-zenml-pro
- Changelog: https://docs.zenml.io/changelog
- PyPI package: https://pypi.org/project/zenml/
- GitHub repository: https://github.com/zenml-io/zenml

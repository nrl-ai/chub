---
name: package
description: "wandb package guide for Python experiment tracking, model logging, and W&B run management"
metadata:
  languages: "python"
  versions: "0.25.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "wandb,weights-and-biases,python,ml,experiment-tracking,artifacts,training"
---

# wandb Python Package Guide

## What It Is

`wandb` is the Python SDK for Weights & Biases. Use it to:

- create and manage experiment runs
- log scalar metrics, media, tables, and config
- store model checkpoints and datasets as artifacts
- query or automate runs later through the W&B API

For most coding tasks, the core flow is: authenticate, `wandb.init(...)`, `run.log(...)`, optionally `run.log_artifact(...)`, then `run.finish()`.

## Installation

Install the exact version covered here:

```bash
python -m pip install wandb==0.25.1
```

Common package managers:

```bash
uv add wandb==0.25.1
poetry add wandb==0.25.1
```

If the project already pins `wandb`, match that version instead of forcing `0.25.1`.

## Authentication And Setup

W&B can authenticate interactively or from an API key.

### Typical environment variables

```bash
export WANDB_API_KEY="..."
export WANDB_ENTITY="your-team-or-user"
export WANDB_PROJECT="my-project"
```

In Python:

```python
import os
import wandb

wandb.login(key=os.environ["WANDB_API_KEY"])
```

For a self-hosted W&B instance, set the base URL before logging in:

```bash
export WANDB_BASE_URL="https://wandb.example.com"
```

```python
import os
import wandb

wandb.login(
    key=os.environ["WANDB_API_KEY"],
    host=os.environ["WANDB_BASE_URL"],
)
```

Useful runtime configuration:

- `WANDB_PROJECT`: default project name
- `WANDB_ENTITY`: default team or user namespace
- `WANDB_DIR`: local directory for generated run files
- `WANDB_MODE`: runtime mode such as `online`, `offline`, or `disabled`
- `WANDB_BASE_URL`: API host for self-managed deployments

## Initialize A Run

Prefer keeping the returned run object instead of relying on the module-global current run.

```python
import os
import wandb

wandb.login(key=os.environ["WANDB_API_KEY"])

run = wandb.init(
    project=os.getenv("WANDB_PROJECT", "demo-project"),
    entity=os.getenv("WANDB_ENTITY"),
    config={
        "learning_rate": 1e-3,
        "batch_size": 32,
        "epochs": 5,
    },
    tags=["baseline"],
    mode=os.getenv("WANDB_MODE", "online"),
)
```

Important `wandb.init()` parameters for agents:

- `project`: logical destination for the run
- `entity`: user or team namespace
- `config`: hyperparameters and other structured metadata
- `dir`: where local run data should be written
- `tags`: labels for filtering in the UI
- `mode`: `online`, `offline`, or `disabled`
- `id` plus `resume`: use when resuming a known run instead of creating a new one
- `reinit`: relevant when the same Python process intentionally creates multiple runs

## Core Usage

### Log Metrics

Use `run.log()` or `wandb.log()` only after a run exists. If you control the run handle, prefer `run.log()`.

```python
import random
import wandb

run = wandb.init(project="demo-project", config={"epochs": 3})

for step in range(run.config["epochs"]):
    train_loss = 1.0 / (step + 1) + random.random() * 0.01
    val_accuracy = 0.7 + step * 0.1

    run.log(
        {
            "train/loss": train_loss,
            "val/accuracy": val_accuracy,
            "epoch": step,
        },
        step=step,
    )

run.finish()
```

The reference docs call out that custom `step` values must be monotonically increasing. If you omit `step`, W&B manages the internal step counter for you.

### Update Run Config

Use the run config for values you will want to filter or compare later.

```python
import wandb

run = wandb.init(project="demo-project", config={"optimizer": "adam"})
run.config.update({"dropout": 0.2, "seed": 42})
run.finish()
```

### Log Artifacts

Artifacts are the right abstraction for model files, evaluation outputs, and reusable datasets.

```python
from pathlib import Path
import wandb

run = wandb.init(project="demo-project")

checkpoint = Path("checkpoints/model.pt")

artifact = wandb.Artifact(
    name="mnist-model",
    type="model",
    metadata={"framework": "pytorch"},
)
artifact.add_file(checkpoint)

run.log_artifact(artifact)
run.finish()
```

For directories, use `artifact.add_dir(...)`. For later consumption, use the Artifact APIs rather than treating saved files as anonymous blobs.

### Offline Or Disabled Modes

For disconnected environments:

```python
import wandb

run = wandb.init(project="demo-project", mode="offline")
run.log({"loss": 0.42})
run.finish()
```

Use `mode="offline"` when you still want local run data and plan to sync later. Use `mode="disabled"` for tests or code paths where W&B should become a no-op.

## Config And Auth Guidance

- Prefer environment variables for secrets and defaults instead of hard-coding API keys, project names, or team names.
- In CI, set `WANDB_API_KEY`, `WANDB_ENTITY`, and `WANDB_PROJECT` explicitly.
- In containers or ephemeral jobs, set `WANDB_DIR` to a writable persistent location if you need local run files after process exit.
- For self-hosted W&B, keep `WANDB_BASE_URL` and the `host=` argument consistent across login and later API use.
- If you resume runs, keep the run `id` stable and use the documented `resume` mode intentionally; otherwise W&B may create a new run instead of continuing the old one.

## Common Pitfalls

- Docs version mismatch: the reference landing page still shows `0.25.0`, but PyPI is `0.25.1`. Use PyPI for package-version truth and the current reference pages for API shapes.
- Logging before initialization: call `wandb.init()` first. Keep the returned `run` object and log through it.
- Multiple runs in one process: finish the current run before starting another. If the program intentionally creates multiple runs, configure `reinit` instead of relying on implicit behavior.
- Non-monotonic steps: if you pass `step=...`, it must only move forward.
- Wrong runtime mode: `offline` still writes local run data, while `disabled` suppresses tracking. Pick the mode that matches the environment.
- Missing local write permissions: W&B writes local run metadata and artifact staging files. Set `WANDB_DIR` when the default working directory is not appropriate.
- Self-hosted auth drift: if the API key belongs to a different W&B host than `WANDB_BASE_URL`, login and later API calls will fail in confusing ways.

## Version-Sensitive Notes

- This doc targets `wandb` `0.25.1` from PyPI.
- PyPI currently lists `Requires: Python >=3.9` for `0.25.1`.
- The docs URL `https://docs.wandb.ai/ref/python/` is still useful as a landing page, but current detailed reference pages are under `https://docs.wandb.ai/models/ref/python/`.
- Because the landing page version badge still shows `0.25.0`, avoid using the docs site title as the package-version source of truth during automation.

## Practical Guidance For Agents

1. Start with explicit environment configuration: `WANDB_API_KEY`, `WANDB_PROJECT`, and optionally `WANDB_ENTITY`.
2. Keep `run = wandb.init(...)` in a variable and use `run.log(...)` and `run.log_artifact(...)` rather than relying on implicit global state.
3. Put hyperparameters and fixed metadata in `config`, and stream time-series values through `run.log(...)`.
4. Use artifacts for checkpoints, model bundles, datasets, and evaluation outputs you need to version or reuse later.
5. In tests, use `mode="disabled"`. In disconnected training jobs, use `mode="offline"` and sync later if needed.

## Official Sources

- PyPI package page: `https://pypi.org/project/wandb/`
- PyPI JSON metadata: `https://pypi.org/pypi/wandb/json`
- Python reference landing page: `https://docs.wandb.ai/ref/python/`
- `wandb.login()` reference: `https://docs.wandb.ai/models/ref/python/functions/login`
- `wandb.init()` reference: `https://docs.wandb.ai/models/ref/python/functions/init`
- `Run` reference: `https://docs.wandb.ai/models/ref/python/experiments/run`
- `Artifact` reference: `https://docs.wandb.ai/models/ref/python/experiments/artifact`
- Environment variables: `https://docs.wandb.ai/models/track/environment-variables`

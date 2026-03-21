---
name: package
description: "ClearML Python SDK guide for experiment tracking, configuration, and dataset workflows"
metadata:
  languages: "python"
  versions: "2.1.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "clearml,python,mlops,experiments,datasets,tracking"
---

# ClearML Python Package Guide

## What This Package Is For

`clearml` is the Python SDK for ClearML experiment tracking and data/model management. The core SDK surface is centered on `Task` for run tracking and `Dataset` for versioned data.

Use this doc for the package version used here, `2.1.3`, with the official docs root at `https://clear.ml/docs/latest/docs/`.

## Install

Base install:

```bash
pip install clearml==2.1.3
```

If you need cloud storage integrations for artifacts or datasets, install the matching extra:

```bash
pip install "clearml[s3]==2.1.3"
pip install "clearml[gs]==2.1.3"
pip install "clearml[azure]==2.1.3"
```

Import paths are package-native:

```python
from clearml import Dataset, Task
```

## First-Time Setup And Auth

The standard setup flow is:

1. Create or sign in to a ClearML server or `app.clear.ml` workspace.
2. Open **Settings > Workspace > Create new credentials** in the web app.
3. Run `clearml-init` locally and paste the web host, API host, files host, access key, and secret key.

```bash
clearml-init
```

Important behavior:

- `clearml-init` does **not** overwrite an existing `clearml.conf`.
- If you are switching servers or workspaces, remove or replace the stale config yourself.
- For CI, containers, or ephemeral jobs, prefer environment variables or `Task.set_credentials(...)`.

Useful environment variables:

```bash
export CLEARML_API_HOST="https://api.clear.ml"
export CLEARML_WEB_HOST="https://app.clear.ml"
export CLEARML_FILES_HOST="https://files.clear.ml"
export CLEARML_API_ACCESS_KEY="..."
export CLEARML_API_SECRET_KEY="..."
export CLEARML_CONFIG_FILE="/abs/path/to/clearml.conf"
```

If you set credentials in code, do it before `Task.init()`:

```python
from clearml import Task

Task.set_credentials(
    api_host="https://api.clear.ml",
    web_host="https://app.clear.ml",
    files_host="https://files.clear.ml",
    key="CLEARML_ACCESS_KEY",
    secret="CLEARML_SECRET_KEY",
)
```

## Minimal Experiment Tracking

Initialize a task early, connect your config, and log metrics through the ClearML logger:

```python
from clearml import Task

task = Task.init(
    project_name="forecasting",
    task_name="xgboost-baseline",
    task_type=Task.TaskTypes.training,
    reuse_last_task_id=False,
)

params = {
    "learning_rate": 0.1,
    "max_depth": 6,
    "n_estimators": 200,
}
task.connect(params)

logger = task.get_logger()

for epoch, loss in enumerate([0.61, 0.47, 0.38], start=1):
    logger.report_scalar("loss", "train", iteration=epoch, value=loss)

task.close()
```

Practical notes:

- `Task.init()` is the normal entry point for tracked runs.
- `task.connect(...)` is the simplest way to persist a dict of hyperparameters or runtime config.
- Set `reuse_last_task_id=False` when you want each local run to create a fresh task instead of reusing the previous development task.
- Close the task explicitly if the same process may create another task later.

## Dataset Workflow

Use `Dataset` when you want versioned, reproducible data instead of ad hoc local paths.

Create and publish a dataset version:

```python
from clearml import Dataset

dataset = Dataset.create(
    dataset_project="forecasting-data",
    dataset_name="raw-training-data",
)
dataset.add_files(path="/data/raw")
dataset.upload()
dataset.finalize()
```

Resolve a dataset version back into a local working directory:

```python
from clearml import Dataset

local_path = Dataset.get(
    dataset_project="forecasting-data",
    dataset_name="raw-training-data",
).get_local_copy()
```

If datasets or artifacts are stored in S3, GCS, or Azure, install the matching package extra and make sure the storage credentials are configured in ClearML before upload or download.

## Configuration Patterns

`clearml` supports three practical configuration paths:

- `clearml.conf` on disk for developer machines and long-lived environments
- environment variables for CI, containers, and secret injection
- `Task.set_credentials(...)` for explicit bootstrap code

Use `CLEARML_CONFIG_FILE` when your config file is not in the default location. This is useful in Docker images and managed jobs where the config must live in a mounted secret or generated path.

## Common Pitfalls

- `Task.init()` requires valid `project_name` and `task_name` values when it creates a new task. The docs note a minimum name length of 3 characters.
- Call `Task.set_credentials(...)` before `Task.init()`, not after.
- `clearml-init` will keep an old config file in place, so switching between self-hosted and hosted ClearML can silently point jobs at the wrong server.
- Remote storage support is not automatic. Install the matching extra such as `clearml[s3]` before using S3-backed datasets or artifact output.
- Finalize datasets after upload. Leaving them unfinished can produce draft-like states that are awkward to reuse from later runs.

## Version-Sensitive Notes

- This entry is pinned to PyPI version `2.1.3` because that is the version used here for this package.
- Official docs are served from a latest-tracking root, `https://clear.ml/docs/latest/docs/`, and several pages still use the label "ClearML 2.0 Documentation". Treat that site label as product-doc branding, not as a precise pip package version.
- PyPI shows `2.1.3` as the current stable release and also lists a newer prerelease, `2.1.4rc0`. If you need features added after `2.1.3`, verify them against the installed package and upstream release metadata before copying examples verbatim.

## Official Sources

- Docs root: https://clear.ml/docs/latest/docs/
- SDK task guide: https://clear.ml/docs/latest/docs/fundamentals/task/
- `Task.init()` reference: https://clear.ml/docs/latest/docs/references/sdk/task/#taskinit
- Dataset guide: https://clear.ml/docs/latest/docs/fundamentals/artifacts/data_management/
- Setup and configuration: https://clear.ml/docs/latest/docs/configs/clearml_conf/
- Environment variables: https://clear.ml/docs/latest/docs/configs/env_vars/
- `clearml-init`: https://clear.ml/docs/latest/docs/clearml_sdk/clearml_sdk_setup/
- PyPI package: https://pypi.org/project/clearml/

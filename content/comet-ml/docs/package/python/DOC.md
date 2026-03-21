---
name: package
description: "Comet ML Python SDK for experiment tracking, autologging, and experiment queries"
metadata:
  languages: "python"
  versions: "3.57.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "comet-ml,comet,python,mlops,experiment-tracking,observability"
---

# comet-ml Python Package Guide

## Installation

Use the package name from PyPI:

```bash
pip install comet-ml==3.57.2
```

Common alternatives:

```bash
uv add comet-ml
poetry add comet-ml
```

## Authentication And Basic Configuration

For normal online logging, Comet needs an API key and usually a workspace and project name.

Set them with environment variables:

```bash
export COMET_API_KEY="YOUR_API_KEY"
export COMET_WORKSPACE="YOUR_WORKSPACE"
export COMET_PROJECT_NAME="YOUR_PROJECT"
```

Or persist them in `~/.comet.config`:

```ini
[comet]
api_key=YOUR_API_KEY
workspace=YOUR_WORKSPACE
project_name=YOUR_PROJECT
```

Environment variables are the safer default for CI, containers, and ephemeral dev environments.

## Recommended Initialization For New Code

The current quickstart uses `comet_ml.start()` as the main entry point for new code:

```python
import os
import comet_ml

experiment = comet_ml.start(
    api_key=os.environ["COMET_API_KEY"],
    workspace=os.environ["COMET_WORKSPACE"],
    project_name=os.environ.get("COMET_PROJECT_NAME", "sandbox"),
)

experiment.set_name("baseline-run")
experiment.log_parameter("learning_rate", 1e-3)
experiment.log_parameter("batch_size", 64)

for step, loss in enumerate([0.82, 0.57, 0.41], start=1):
    experiment.log_metric("loss", loss, step=step)

experiment.log_metrics({"accuracy": 0.93, "f1": 0.91}, step=3)
```

Comet also automatically logs source code, installed packages, command-line details, Git metadata, CPU and GPU usage, and network activity unless you change the default configuration.

## Explicit Experiment Classes

Use the explicit classes when you need tighter lifecycle control:

- `Experiment` for normal online runs
- `ExistingExperiment` to resume an already-created experiment
- `OfflineExperiment` when the training environment cannot reach Comet

Example offline run:

```python
from comet_ml import OfflineExperiment

experiment = OfflineExperiment(
    offline_directory="./comet-offline",
    project_name="sandbox",
)

experiment.log_parameter("model", "xgboost")
experiment.log_metric("auc", 0.94)
```

The offline workflow produces experiment zip files. Upload them later with the documented `comet upload <path-to-experiment-zip>` flow from the offline experiments guide.

## Reading Existing Experiments

If the task is to read experiment history, models, or assets instead of logging a new run, initialize the SDK's API client:

```python
import os
from comet_ml import API

api = API(api_key=os.environ["COMET_API_KEY"])
```

Use the Python SDK API reference for project, experiment, model, and asset retrieval methods after creating the client.

## Common Pitfalls

- Install with `pip install comet-ml`, but import it as `comet_ml`.
- Prefer `comet_ml.start()` for new code. Many older blog posts and snippets still construct `Experiment(...)` directly.
- Do not rely on interactive local configuration in CI. Set `COMET_API_KEY`, `COMET_WORKSPACE`, and `COMET_PROJECT_NAME` explicitly.
- Default autologging may capture code, package lists, Git metadata, and system information. Review the configuration reference before running against sensitive repositories or restricted environments.
- The docs URL path uses `/docs/v2/`, but that is the documentation site versioning, not the PyPI package version. Package compatibility should be checked against the installed `comet-ml` release.

## Version-Sensitive Notes For `3.57.2`

- PyPI currently lists `3.57.2` as the latest `comet-ml` release, published on 2026-03-09.
- The current official quickstart and SDK docs are aligned around `comet_ml.start()` plus the Python SDK reference pages under `docs/v2`.
- When copying older Comet examples, verify whether they use deprecated initialization style or older class names before reusing them unchanged.

## Official Sources

- PyPI package: `https://pypi.org/project/comet-ml/`
- Docs root: `https://www.comet.com/docs/v2/`
- Python quickstart: `https://www.comet.com/docs/v2/guides/getting-started/quickstart/`
- Python configuration reference: `https://www.comet.com/docs/v2/guides/experiment-management/configure-sdk/`
- Offline experiments guide: `https://www.comet.com/docs/v2/guides/experiment-management/offline-experiments/`
- Python SDK API reference: `https://www.comet.com/docs/v2/api-and-sdk/python-sdk/reference/API/`

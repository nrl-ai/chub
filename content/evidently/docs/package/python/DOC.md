---
name: package
description: "evidently package guide for Python data quality, drift, performance, and evaluation reports"
metadata:
  languages: "python"
  versions: "0.7.21"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "evidently,python,ml,monitoring,data-quality,data-drift,testing"
---

# evidently Python Package Guide

## Golden Rule

**Use `evidently` to generate a snapshot first, then decide how you want to persist it: save HTML/JSON for one-off analysis or push it into a workspace for history and dashboards.**

For most coding tasks, start with a `Report` or `TestSuite` over pandas data, confirm the output locally, and only then wire it into a local or cloud workspace.

## Installation

Pin the version if you need behavior to match this guide:

```bash
python -m pip install "evidently==0.7.21"
```

Optional extras are published for feature-specific integrations:

```bash
python -m pip install "evidently[llm]==0.7.21"
python -m pip install "evidently[s3,gcs,fsspec]==0.7.21"
python -m pip install "evidently[sql,spark]==0.7.21"
```

Use extras only when you need them:

- `llm` for LLM evaluation features
- `s3`, `gcs`, `fsspec` for object-storage or filesystem integrations
- `sql` for SQL connectors
- `spark` for Spark-based workflows

## Minimal Setup

The current README and library docs both use pandas data as the normal starting point.

```python
import pandas as pd

from evidently import Report
from evidently.presets import DataDriftPreset

reference_df = pd.read_parquet("reference.parquet")
current_df = pd.read_parquet("current.parquet")

report = Report([DataDriftPreset()])
snapshot = report.run(current_data=current_df, reference_data=reference_df)

snapshot.save_html("evidently-report.html")
snapshot.save_json("evidently-report.json")
```

Practical rules:

- Use `reference_data` as the baseline and `current_data` as the batch or slice you are evaluating now.
- Persist the returned snapshot with `save_html()` or `save_json()` if you only need a file artifact.
- If your columns are ambiguous, typed incorrectly, or include timestamps/targets/predictions/text, move to the `Dataset` plus `DataDefinition` workflow from the official quickstart instead of relying on inference.

## Core Usage Patterns

### Reports for drift, quality, and model summaries

Reports are the main entry point for analysis snapshots. The official examples use presets from `evidently.presets`.

```python
from evidently import Report
from evidently.presets import DataSummaryPreset, RegressionPreset

summary_snapshot = Report([DataSummaryPreset()]).run(current_data=current_df)

regression_snapshot = Report([RegressionPreset()]).run(
    current_data=current_df,
    reference_data=reference_df,
)
```

Use current-only reports for point-in-time summaries. Use both current and reference datasets for drift and performance comparisons.

### Test suites for pass/fail checks

Use a `TestSuite` when you need gate-style evaluation rather than an exploratory report.

```python
from evidently import TestSuite
from evidently.presets import NoTargetPerformanceTestPreset

test_suite = TestSuite(
    tests=[
        NoTargetPerformanceTestPreset(stattest="ks", threshold=0.9),
    ]
)

suite_snapshot = test_suite.run(
    current_data=current_df,
    reference_data=reference_df,
)
```

This pattern is a better fit for CI, scheduled checks, or pipeline thresholds than manually inspecting charts.

### Local workspace for persisted history

Use a workspace when you want project history instead of standalone files. The local workspace API uses a `file://` URI.

```python
from evidently import Report
from evidently.presets import DataSummaryPreset
from evidently.ui.workspace import Workspace

ws = Workspace.create("file:///tmp/evidently-workspace")
project = ws.create_project("batch-monitoring")

snapshot = Report([DataSummaryPreset()]).run(current_data=current_df)
ws.add_run(project.id, snapshot, include_data=False)
```

Common local pattern:

- Create one workspace path per environment or project
- Create one Evidently project per monitored model or dataset family
- Add snapshots on every batch, training run, or scheduled job

### Evidently Cloud workspace

Cloud usage adds authentication and a hosted URL, but the report/test creation flow stays the same.

```python
from evidently.ui.workspace.cloud import CloudWorkspace

ws = CloudWorkspace(
    token="YOUR_API_KEY",
    url="https://app.evidently.cloud",
)

project = ws.get_project("PROJECT_ID")
ws.add_run(project.id, snapshot)
```

Use cloud mode when you need shared access, hosted dashboards, or centralized run history.

## Configuration And Auth

`evidently` itself is a local Python library. There is no auth step for local reports, JSON/HTML output, or local workspaces.

Authentication matters only when you connect to Evidently Cloud:

- initialize `CloudWorkspace(...)` with a valid API token
- use the correct cloud base URL for your environment
- fetch or create the target project before adding runs

Configuration choices that matter in code:

- Decide whether you are running against pandas data directly or explicit `Dataset` objects.
- Decide whether the result should be a file artifact, a local workspace run, or a cloud workspace run.
- Install the matching optional extra before you depend on Spark, SQL, storage, or LLM features.

## Common Pitfalls

- Do not swap `reference_data` and `current_data`. Drift and performance results depend on that direction.
- Do not expect a `Report` object by itself to create files or dashboards. You need the returned snapshot and then `save_*()` or `add_run(...)`.
- Do not skip explicit schema definition when your dataframe has timestamps, text columns, targets, predictions, or object-typed numeric columns.
- Do not forget the `file://` prefix when creating a local workspace path.
- Do not wire cloud auth into local-only jobs. Local reports and local workspaces do not need a token.
- Do not assume all presets work without reference data. Summary-style outputs can, but drift and many performance checks need a baseline dataset.
- Do not forget feature extras. Storage, SQL, Spark, and LLM flows may need an extra install beyond base `evidently`.

## Version-Sensitive Notes For 0.7.21

- This guide is pinned to `evidently 0.7.21` on PyPI.
- The published package metadata for `0.7.21` requires Python `>=3.9`.
- The maintained docs and README for the current `0.7.x` line center on `Report`, `TestSuite`, workspace APIs, and presets from `evidently.presets`.
- The Evidently Cloud docs state that cloud supports library versions `0.7.11` and above, so `0.7.21` is within the supported range.
- If you are copying older examples from blogs or past internal code, re-check imports and workflow against the current docs before reusing them. The official `0.7.x` examples emphasize snapshots plus workspaces rather than older ad hoc patterns.

## Official Sources Used

- PyPI project page: `https://pypi.org/project/evidently/`
- Repository: `https://github.com/evidentlyai/evidently`
- README: `https://raw.githubusercontent.com/evidentlyai/evidently/main/README.md`
- Package metadata: `https://raw.githubusercontent.com/evidentlyai/evidently/main/python/pyproject.toml`
- Library quickstart: `https://docs.evidentlyai.com/docs/library/get_started`
- Reports docs: `https://docs.evidentlyai.com/docs/library/report`
- Workspaces docs: `https://docs.evidentlyai.com/docs/library/workspace`
- Cloud docs: `https://docs.evidentlyai.com/docs/platform/cloud`
- Cloud quickstart: `https://docs.evidentlyai.com/docs/platform/get_started_cloud`

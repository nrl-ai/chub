---
name: package
description: "Neptune package guide for Python experiment tracking with runs, auth, offline mode, and 2.x versus Neptune Scale docs"
metadata:
  languages: "python"
  versions: "1.14.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "neptune,python,experiment-tracking,mlops,metadata,logging,runs"
---

# Neptune Python Package Guide

## Golden Rule

- For `neptune==1.14.0`, use the legacy Neptune 2.x client API: `import neptune` and `neptune.init_run(...)`.
- Do not mix this package with Neptune 3.x / `neptune-scale` examples from the main docs landing pages.
- Set `NEPTUNE_API_TOKEN` and `NEPTUNE_PROJECT` before running code unless you pass `api_token=` and `project=` explicitly.

## Version-Sensitive Notes

- This entry is pinned to the version used here `1.14.0`.
- The docs URL now points at Neptune's current docs experience, which includes Neptune 3.x and `neptune-scale`. For `neptune==1.14.0`, the official package docs are under `docs-legacy.neptune.ai`.
- Neptune's official GitHub repository is still named `neptune-client`, but the maintained PyPI package is `neptune`.
- Older blog posts and snippets may still say `pip install neptune-client`. For new installs matching this entry, use `pip install neptune==1.14.0`.
- `neptune.init_run()` supports multiple connection modes. The practical ones are `async` (default), `sync`, `offline`, `read-only`, and `debug`.

## Install

Install the pinned version:

```bash
python -m pip install "neptune==1.14.0"
```

Verify the import and version:

```bash
python - <<'PY'
import neptune
print(neptune.__version__)
PY
```

## Authentication And Project Setup

The official docs use two main environment variables:

- `NEPTUNE_API_TOKEN`: your user or service-account token
- `NEPTUNE_PROJECT`: project name in `workspace-name/project-name` form

Set them in the shell:

```bash
export NEPTUNE_API_TOKEN="your-api-token"
export NEPTUNE_PROJECT="workspace-name/project-name"
```

Then initialize without hardcoding secrets in code:

```python
import neptune

run = neptune.init_run()
```

If you need explicit configuration:

```python
import neptune

run = neptune.init_run(
    project="workspace-name/project-name",
    api_token="your-api-token",
)
```

Use the special token `ANONYMOUS` only for public projects where anonymous access is enabled.

## Initialize A Run

Start with a single run object and close it when the script is done:

```python
import neptune

run = neptune.init_run()

run["parameters"] = {
    "learning_rate": 1e-3,
    "batch_size": 64,
    "optimizer": "adamw",
}

run["train/loss"].append(0.842)
run["train/loss"].append(0.611)
run["train/accuracy"] = 0.91
run["sys/tags"].add(["baseline", "resnet"])

run.stop()
```

Use `run.stop()` in scripts, tests, and batch jobs so buffered metadata is flushed before process exit.

## Core Logging Patterns

### Log scalar values and time series

Use assignment for single values and `append(...)` for series:

```python
run["eval/f1"] = 0.88
run["train/loss"].append(0.42)
run["train/loss"].append(0.31)
```

### Log files and artifacts

```python
run["artifacts/model"].upload("checkpoints/model.pt")
run["artifacts/config"].upload("configs/train.yaml")
```

### Organize metadata by namespace

Neptune fields are path-like keys. Use stable namespaces so downstream dashboards and queries stay consistent:

```python
run["config/model/name"] = "resnet50"
run["metrics/val/loss"].append(0.27)
run["metrics/val/accuracy"].append(0.93)
```

## Resume Or Reattach To Existing Runs

Use `with_id=` when you already know the Neptune run ID and want to reopen that run:

```python
import neptune

run = neptune.init_run(with_id="CLS-123")
run["train/loss"].append(0.22)
run.stop()
```

Use `custom_run_id=` when your application has its own stable external identifier for a run:

```python
import neptune

run = neptune.init_run(custom_run_id="train-job-2026-03-12-001")
```

Practical distinction:

- `with_id=`: reopen a specific Neptune run that already exists
- `custom_run_id=`: attach your own stable identifier to a run so distributed or restarted jobs can refer to it consistently

## Useful Modes

### Default: `async`

`async` is the default mode. Use it for normal training or application code where background logging is fine.

```python
run = neptune.init_run(mode="async")
```

### `sync`

Use `sync` when you want logging calls to block until data is sent. This is useful in short-lived scripts or tests where you want simpler timing behavior.

```python
run = neptune.init_run(mode="sync")
```

### `offline`

Use `offline` when the job cannot reach Neptune during execution. The legacy docs describe syncing offline data later with the `neptune sync` CLI.

```python
run = neptune.init_run(mode="offline")
```

### `read-only`

Use `read-only` for code that only fetches metadata from existing objects and must not create new logs.

```python
run = neptune.init_run(with_id="CLS-123", mode="read-only")
```

### `debug`

Use `debug` in tests when you want Neptune calls to be effectively disabled without touching production code paths.

```python
run = neptune.init_run(mode="debug")
```

## Reading Existing Metadata

For inspection code, fetch existing fields instead of re-logging them:

```python
import neptune

run = neptune.init_run(with_id="CLS-123", mode="read-only")

best_score = run["eval/best_f1"].fetch()
print(best_score)

run.stop()
```

Use read-only access in analysis scripts, dashboards, or CI verification steps so they cannot accidentally mutate tracking state.

## Common Pitfalls

### Mixing the wrong Neptune product docs

The biggest failure mode is using current Neptune Scale docs for this package. If the example imports `neptune_scale`, talks about Neptune 3.x tables as the primary API, or assumes the new docs navigation, it is not the right source for `neptune==1.14.0`.

### Forgetting `run.stop()`

In notebooks this is less visible, but in short-lived scripts or worker jobs, failing to stop the run can leave data unsent.

### Wrong project format

`project=` is not a URL. Use `workspace-name/project-name`.

### Confusing `with_id` and `custom_run_id`

`with_id` reopens a Neptune run by Neptune's own identifier. `custom_run_id` is your application-level identifier. Using the wrong one leads to resume bugs and duplicate tracking records.

### Assuming anonymous auth works everywhere

The `ANONYMOUS` token only works for public projects configured to allow it. For private workspaces, use a real API token.

### Ignoring offline and test modes

For air-gapped jobs, CI, or unit tests, choose `offline`, `read-only`, or `debug` deliberately instead of letting default networked logging fail at runtime.

## Recommended Workflow For Agents

1. Confirm the project is using the legacy `neptune` package, not `neptune-scale`.
2. Pin `neptune==1.14.0` if reproducibility matters.
3. Set `NEPTUNE_API_TOKEN` and `NEPTUNE_PROJECT`.
4. Start with `neptune.init_run()`, log parameters and metrics, and call `run.stop()`.
5. Reach for `with_id=...`, `custom_run_id=...`, or `mode=...` only when the task needs resume, offline execution, or read-only access.

## Official Source URLs

- Main docs landing page: `https://docs.neptune.ai/`
- Migration note that points `neptune` users to legacy docs: `https://docs.neptune.ai/3.20250901/migration_guide/`
- Legacy docs root for this package: `https://docs-legacy.neptune.ai/`
- Legacy quickstart: `https://docs-legacy.neptune.ai/quickstart/`
- Legacy auth docs: `https://docs-legacy.neptune.ai/setup/neptune-api-token/`
- Legacy `init_run()` reference: `https://docs-legacy.neptune.ai/api/neptune/#init_run`
- PyPI project page: `https://pypi.org/project/neptune/`
- Official repository: `https://github.com/neptune-ai/neptune-client`

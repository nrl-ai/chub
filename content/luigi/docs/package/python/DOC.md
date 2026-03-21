---
name: package
description: "Luigi package guide for Python batch pipelines, task dependencies, and scheduler configuration"
metadata:
  languages: "python"
  versions: "3.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "luigi,python,pipeline,workflow,scheduler,batch,orchestration"
---

# Luigi Python Package Guide

## Golden Rule

Use `luigi` to model batch pipelines as Python `Task` classes with explicit `requires()`, `output()`, and `run()` methods. Start with `--local-scheduler` for development, but use `luigid` in shared environments so Luigi can prevent duplicate task runs and give you the scheduler UI.

## Install

Pin the package version your project expects:

```bash
python -m pip install "luigi==3.8.0"
```

Common alternatives:

```bash
uv add "luigi==3.8.0"
poetry add "luigi==3.8.0"
```

Useful extras from PyPI:

```bash
python -m pip install "luigi[toml]==3.8.0"
python -m pip install "luigi[prometheus]==3.8.0"
python -m pip install "luigi[jsonschema]==3.8.0"
```

Notes:

- `luigi[toml]` adds TOML-based config support.
- `luigi[prometheus]` is useful when you want scheduler or worker metrics integration.
- `luigi[jsonschema]` is only needed if your code or config relies on that optional support.

## Core Model

Luigi task graphs are built from Python classes:

- `requires()`: upstream task dependencies
- `output()`: the `Target` that marks completion
- `run()`: the work to execute
- `complete()`: usually inherited; a task is complete when its outputs exist

Luigi treats `output()` as the contract for completion, so your `run()` method must fully create the declared target before downstream tasks can run safely.

## Minimal Working Pipeline

This is the common local-file pattern for simple ETL or reporting tasks:

```python
from __future__ import annotations

import json
from pathlib import Path

import luigi

class FetchNumbers(luigi.Task):
    date = luigi.DateParameter()

    def output(self) -> luigi.LocalTarget:
        return luigi.LocalTarget(f"data/raw/{self.date}.json")

    def run(self) -> None:
        Path("data/raw").mkdir(parents=True, exist_ok=True)
        rows = [{"value": 2}, {"value": 3}, {"value": 5}]
        with self.output().open("w") as f:
            json.dump(rows, f)

class SumNumbers(luigi.Task):
    date = luigi.DateParameter()

    def requires(self) -> FetchNumbers:
        return FetchNumbers(date=self.date)

    def output(self) -> luigi.LocalTarget:
        return luigi.LocalTarget(f"data/processed/{self.date}.txt")

    def run(self) -> None:
        Path("data/processed").mkdir(parents=True, exist_ok=True)
        with self.input().open("r") as f:
            rows = json.load(f)

        total = sum(row["value"] for row in rows)
        with self.output().open("w") as f:
            f.write(f"{total}\n")
```

Run it from the CLI:

```bash
python -m luigi --module my_pipeline SumNumbers --date 2026-03-12 --local-scheduler
```

Notes:

- Task parameter names become CLI flags after the task family name.
- Parameters with `_` in Python use `-` on the CLI, for example `my_parameter` becomes `--my-parameter`.
- `self.input()` maps the `requires()` graph into the corresponding `Target` objects.

## Running From Python

Use `luigi.build()` when your entrypoint already has Python logic for argument parsing, configuration loading, or selecting tasks dynamically:

```python
import datetime as dt

import luigi

from my_pipeline import SumNumbers

if __name__ == "__main__":
    ok = luigi.build(
        [SumNumbers(date=dt.date(2026, 3, 12))],
        workers=4,
        local_scheduler=True,
    )
    if not ok:
        raise SystemExit("Luigi build failed")
```

This is often easier than shelling out to the CLI from another Python program.

## Configuration And Scheduler Setup

Luigi supports both config files and task parameters, but the docs note that parameters have largely superseded config for many application-level settings. Use config mainly for scheduler, worker, logging, and shared infrastructure defaults.

### Config file discovery

Default config lookup for the `cfg` parser, from lower to higher priority:

- `/etc/luigi/client.cfg` and `client.cfg` are deprecated old paths
- `/etc/luigi/luigi.cfg`
- `luigi.cfg`
- `LUIGI_CONFIG_PATH`

For TOML configs:

- `/etc/luigi/luigi.toml`
- `luigi.toml`
- `LUIGI_CONFIG_PATH`

Set `LUIGI_CONFIG_PARSER=toml` if you want Luigi to read TOML instead of `cfg`.

### Typical local config

```ini
[core]
default-scheduler-host=localhost
default-scheduler-port=8082
no_configure_logging=false

[worker]
keep_alive=true
check_unfulfilled_deps=true
check_complete_on_run=true
cache_task_completion=true
```

`check_complete_on_run=true` is a useful safeguard when agents generate tasks that might forget to materialize outputs cleanly.

### Local scheduler vs central scheduler

Use `--local-scheduler` while iterating on task code:

```bash
python -m luigi --module my_pipeline SumNumbers --date 2026-03-12 --local-scheduler
```

For shared or production-style runs, start the central scheduler:

```bash
luigid
```

Then run tasks without `--local-scheduler`. By default Luigi connects to `http://localhost:8082/`.

Use the central scheduler when you need:

- duplicate-run protection across workers or hosts
- the web UI for dependency graphs and task state
- task history or centralized scheduling behavior

## Targets And Storage

`luigi.LocalTarget` is the default file target for local pipelines. Luigi writes atomically, which is one reason targets are central to the framework.

When using multiple workers or machines, your outputs must live on storage visible to all workers. If one worker writes only to local disk and another worker cannot see that path, Luigi may recompute the same task because completion is target-based.

If you need an externally created dependency, wrap it as `luigi.ExternalTask` instead of returning a bare `Target` from `requires()`.

```python
import luigi

class UpstreamExport(luigi.ExternalTask):
    date = luigi.DateParameter()

    def output(self) -> luigi.LocalTarget:
        return luigi.LocalTarget(f"incoming/{self.date}.csv")
```

## Common Patterns

### Dynamic dependencies

If you only know a dependency at runtime, `yield` another task from `run()`. Luigi will suspend the current task, run the yielded task, then re-enter `run()`.

That means `run()` must be idempotent. Do not append blindly to files or mutate external state before the yielded task finishes unless you can safely repeat the work.

### Parameterized recurring tasks

Luigi is good at date-partitioned and backfill-style pipelines:

```python
class DailyReport(luigi.Task):
    date = luigi.DateParameter()
```

Prefer modeling partitions as parameters instead of hard-coding paths or dates inside `run()`.

## Authentication And Network Notes

Luigi itself is not a cloud auth SDK. Authentication depends on the systems your tasks talk to, such as databases, object stores, Hadoop, or APIs.

For the scheduler itself:

- local development usually talks to `localhost:8082`
- remote scheduler access is plain HTTP by default
- if you need TLS, terminate it with an HTTP proxy and point Luigi at the resulting URL

Keep secrets for downstream systems out of task IDs and scheduler-visible significant parameters. Luigi parameters are part of task identity unless marked otherwise.

## Common Pitfalls

- Do not return a `Target` directly from `requires()`. Luigi expects `Task` objects there; use `ExternalTask` for external data.
- Do not declare multiple outputs unless you can preserve atomicity yourself. A single output target is the safest default.
- Do not assume `run()` success means the task is complete. Completion is output-driven.
- Do not use local disk outputs for multi-worker production jobs unless every worker shares that filesystem.
- Do not forget `--local-scheduler` during early development. Otherwise Luigi will try to connect to a scheduler on `localhost:8082`.
- Do not expose secrets as normal significant parameters. They can leak into task identity and scheduler UI state.
- If you use dynamic dependencies with `yield`, make sure `run()` is repeatable because Luigi resumes it from scratch.
- If you write binary files, use `format=Nop` with `LocalTarget`; Luigi strips the normal binary flag handling for its atomic file semantics.

## Version-Sensitive Notes For Luigi 3.8.0

- PyPI shows `luigi 3.8.0` released on March 6, 2026, and the verified Python requirement is now `>=3.10,<3.14`.
- The top-level Read the Docs landing page also shows `3.8.0`, but several deeper pages still display older subpage version banners such as `3.7.x` or `3.6.0`. Treat those pages as authoritative for behavior only when the content still matches the current package release.
- The docs now describe Python 3.10 through 3.13 as the tested range. Older blog posts and examples for Python 3.8 or 3.9 are stale for new environments.
- `client.cfg` paths remain documented but deprecated. Prefer `luigi.cfg` or `luigi.toml` for new setups.

## Official Sources

- PyPI project page: https://pypi.org/project/luigi/
- Repository: https://github.com/spotify/luigi
- Docs root: https://luigi.readthedocs.io/
- Tasks guide: https://luigi.readthedocs.io/en/latest/tasks.html
- Running Luigi: https://luigi.readthedocs.io/en/stable/running_luigi.html
- Central scheduler guide: https://luigi.readthedocs.io/en/stable/central_scheduler.html
- Configuration guide: https://luigi.readthedocs.io/en/stable/configuration.html

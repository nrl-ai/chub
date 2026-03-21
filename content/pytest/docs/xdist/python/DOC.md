---
name: xdist
description: "pytest-xdist package guide for parallel pytest runs, worker-aware fixtures, and distributed test scheduling"
metadata:
  languages: "python"
  versions: "3.8.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pytest,pytest-xdist,testing,parallelism,ci,python"
---

# pytest-xdist Python Package Guide

## Golden Rule

Use `pytest-xdist` only to parallelize a suite that already passes reliably under plain `pytest`. When a failure is hard to understand, rerun with `-n 0` or without `-n` first. The plugin changes scheduling and process boundaries; it does not make nondeterministic tests safe.

## Install

Pin the plugin to the version your project expects:

```bash
python -m pip install "pytest-xdist==3.8.0"
```

Common alternatives:

```bash
uv add --dev "pytest-xdist==3.8.0"
poetry add --group dev "pytest-xdist==3.8.0"
```

Optional extras published on PyPI:

```bash
python -m pip install "pytest-xdist[psutil]==3.8.0"
python -m pip install "pytest-xdist[setproctitle]==3.8.0"
```

Use the `psutil` extra if you want better logical CPU detection for `-n logical`. Use the `setproctitle` extra if process titles in `ps` or `top` are useful while diagnosing stuck workers.

## Minimal Setup

The plugin auto-registers with pytest after installation. There is no separate initialization step.

Run a suite in parallel:

```bash
pytest -n auto
```

Useful variants:

```bash
pytest -n 4
pytest -n logical
pytest -n 0
```

- `-n auto` uses the machine's physical CPU cores.
- `-n logical` uses logical cores when `psutil` can determine them; otherwise it falls back to `auto` behavior.
- `-n 0` disables xdist and runs everything in the main process.

## Project Configuration

If parallel execution is your default in CI, keep it explicit in pytest config rather than hiding it in shell scripts.

`pyproject.toml`:

```toml
[tool.pytest.ini_options]
addopts = "-ra -n auto --dist loadscope"
```

Equivalent `pytest.ini`:

```ini
[pytest]
addopts = -ra -n auto --dist loadscope
```

If worker count should depend on the environment, set the documented environment variable:

```bash
export PYTEST_XDIST_AUTO_NUM_WORKERS=6
pytest -n auto
```

You can also override auto-detection in code with the `pytest_xdist_auto_num_workers(config)` hook in `conftest.py`.

## Core Usage Patterns

### Basic parallel run

```bash
pytest -n auto
```

This is the default entry point for speeding up a CPU-bound or I/O-heavy suite.

### Choose an explicit scheduler

```bash
pytest -n auto --dist load
pytest -n auto --dist loadscope
pytest -n auto --dist loadfile
pytest -n auto --dist loadgroup
pytest -n auto --dist worksteal
```

Distribution modes that matter in practice:

- `load`: default. Best general-purpose throughput, but test order is not guaranteed.
- `loadscope`: keeps tests from the same module or class in the same worker. Good when module/class fixtures are expensive or stateful.
- `loadfile`: keeps each file on one worker. Good when file-level setup or ordering matters.
- `loadgroup`: keeps tests with the same `xdist_group` mark on one worker. Good for browser sessions, shared containers, or stateful service fixtures.
- `worksteal`: starts with an even split and then steals queued tests from busy workers. Good when individual test durations vary a lot, but still expect some scheduling churn.

### Keep related tests on the same worker

```python
import pytest

@pytest.mark.xdist_group("chrome")
def test_login_in_chrome():
    ...

@pytest.mark.xdist_group("chrome")
def test_checkout_in_chrome():
    ...
```

Run with:

```bash
pytest -n auto --dist loadgroup
```

### Control `loadscope` ordering in 3.8.0

`pytest-xdist 3.8.0` added these options:

```bash
pytest -n auto --dist loadscope --no-loadscope-reorder
pytest -n auto --dist loadscope --loadscope-reorder
```

Use `--no-loadscope-reorder` when relative file ordering matters and you still want `loadscope` grouping.

## Worker-Aware Tests And Fixtures

### Detect the current worker in a fixture

```python
import pytest

@pytest.fixture
def user_account(worker_id: str) -> str:
    return f"account_{worker_id}"
```

When xdist is disabled, `worker_id` returns `"master"`. This makes it safe to share the same fixture implementation between local debugging and parallel CI runs.

### Use worker environment variables

Workers expose these environment variables:

- `PYTEST_XDIST_WORKER`: current worker name such as `gw0`
- `PYTEST_XDIST_WORKER_COUNT`: total number of workers
- `PYTEST_XDIST_TESTRUNUID`: unique id for the whole distributed run

Example:

```python
import os

def test_worker_env():
    worker_name = os.getenv("PYTEST_XDIST_WORKER", "master")
    worker_count = int(os.getenv("PYTEST_XDIST_WORKER_COUNT", "1"))
    assert worker_count >= 1
    assert worker_name
```

### Generate one resource per test run

Use `testrun_uid` when multiple workers must share one logical namespace:

```python
import pytest

@pytest.fixture(scope="session")
def database_name(testrun_uid: str) -> str:
    return f"test_db_{testrun_uid}"
```

This is the right pattern for temporary databases, buckets, queues, or directories that should be unique per CI run rather than per worker.

### Session fixtures run once per worker, not once globally

This is one of the most important xdist behaviors: each worker performs collection and may execute high-scope fixtures separately. If a `session` fixture must happen exactly once, add your own inter-process locking.

Example with `filelock`:

```python
import json
from pathlib import Path

import pytest
from filelock import FileLock

def produce_expensive_data() -> dict[str, str]:
    return {"token": "generated-once"}

@pytest.fixture(scope="session")
def session_data(tmp_path_factory: pytest.TempPathFactory, worker_id: str) -> dict[str, str]:
    if worker_id == "master":
        return produce_expensive_data()

    root_tmp_dir = tmp_path_factory.getbasetemp().parent
    data_file = Path(root_tmp_dir) / "session-data.json"

    with FileLock(str(data_file) + ".lock"):
        if data_file.exists():
            return json.loads(data_file.read_text())

        data = produce_expensive_data()
        data_file.write_text(json.dumps(data))
        return data
```

Install `filelock` yourself if you use this pattern; it is not part of the default runtime dependency set.

## Logging And Diagnostics

If a worker crashes, xdist can restart it automatically. Cap restarts in CI when you want crashes to fail fast instead of being masked by repeated retries:

```bash
pytest -n auto --max-worker-restart 2
pytest -n auto --max-worker-restart 0
```

To emit one log file per worker, use `PYTEST_XDIST_WORKER` during `pytest_configure`:

```python
import logging
import os

def pytest_configure(config):
    worker_id = os.environ.get("PYTEST_XDIST_WORKER")
    if worker_id:
        logging.basicConfig(
            filename=f"tests_{worker_id}.log",
            level=config.getini("log_file_level"),
            format=config.getini("log_file_format"),
        )
```

This is a practical replacement for expecting interleaved stdout from workers in the terminal.

## Common Pitfalls

- `-s` and `--capture=no` do not work with `pytest-xdist`. Worker stdout and stderr are not streamed back live through `execnet`.
- `--pdb` is disabled during distributed runs. Reproduce the failure with `-n 0` or without `-n` before interactive debugging.
- Test collection order must be identical across workers. Do not parametrize from unordered sets, generators with unstable iteration, or environment-dependent discovery.
- `session` fixtures are not global across all workers. If a resource must be created once, add locking or centralize creation outside pytest.
- Shared temp files, ports, and database names must be made worker-aware or run-aware. `worker_id` and `testrun_uid` exist for this.
- `load` maximizes throughput but can increase fixture churn. Switch to `loadscope`, `loadfile`, or `loadgroup` when setup cost or state affinity matters.
- Keep flaky tests and race-sensitive assertions out of parallel mode until they are deterministic in single-process runs.

## Version-Sensitive Notes For 3.8.0

- `pytest-xdist 3.8.0` was released on July 1, 2025 on PyPI.
- The 3.8.0 changelog adds `--no-loadscope-reorder` and `--loadscope-reorder` for `loadscope` scheduling. Use them if relative ordering matters inside loadscope groups.
- The 3.7.0 release added Python 3.13 support and dropped Python 3.8 support. For 3.8.0, treat Python 3.9+ as the supported baseline.
- The 3.6.1 release line already required `pytest>=7.0.0` and `execnet>=2.1.0`; that remains the practical dependency floor for 3.8.0.
- Remote rsync mode and `--looponfail` are still documented but were deprecated earlier and are planned for removal in xdist 4.0. Avoid building new automation around them.
- `--boxed` is already removed from modern xdist; use `pytest-forked` if you specifically need fork isolation behavior.

## Recommended Agent Workflow

1. Run `pytest` without xdist first to confirm the suite is already healthy.
2. Enable parallelism with `pytest -n auto`.
3. If fixtures or stateful tests churn too much, switch to `--dist loadscope`, `loadfile`, or `loadgroup`.
4. If failures only happen in parallel mode, add worker-aware resource names and rerun with a smaller fixed worker count such as `-n 2`.
5. For debugging, rerun the same subset with `-n 0` and then use `--pdb` or your normal debugger.

## Official Sources

- Stable docs: `https://pytest-xdist.readthedocs.io/en/stable/`
- Distribution guide: `https://pytest-xdist.readthedocs.io/en/stable/distribution.html`
- How-tos: `https://pytest-xdist.readthedocs.io/en/stable/how-to.html`
- Known limitations: `https://pytest-xdist.readthedocs.io/en/stable/known-limitations.html`
- Changelog: `https://pytest-xdist.readthedocs.io/en/stable/changelog.html`
- PyPI package page: `https://pypi.org/project/pytest-xdist/`

---
name: package
description: "pytest-xdist package guide for parallel pytest runs, worker-aware fixtures, and distributed scheduling"
metadata:
  languages: "python"
  versions: "3.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,pytest-xdist,python,testing,parallelism,ci"
---

# pytest-xdist Python Package Guide

## Golden Rule

Use `pytest-xdist` only to parallelize a suite that already passes reliably under plain `pytest`. The plugin changes scheduling and process boundaries; it does not make flaky or stateful tests safe.

`pytest-xdist` is a pytest plugin, not an application runtime dependency. Install it in the same environment as `pytest` and invoke it through the pytest CLI.

## Install

`pytest-xdist 3.8.0` supports Python `>=3.9` and depends on `pytest>=7.0.0` plus `execnet>=2.1.0`.

Install it into your test environment:

```bash
python -m pip install "pytest-xdist==3.8.0"
```

Common project tool equivalents:

```bash
uv add --dev "pytest-xdist==3.8.0"
poetry add --group dev "pytest-xdist==3.8.0"
```

Optional extras published on PyPI:

```bash
python -m pip install "pytest-xdist[psutil]==3.8.0"
python -m pip install "pytest-xdist[setproctitle]==3.8.0"
```

- `psutil` improves CPU detection for `-n logical`.
- `setproctitle` gives workers clearer process names in tools like `ps` and `top`.

## Initialization And Configuration

There is no auth setup and no client object to initialize. After installation, the plugin auto-registers with pytest.

Minimal parallel run:

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
- `-n 0` disables xdist and runs in the main process.

If parallel execution is your default, keep it in pytest config instead of a shell wrapper.

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

To override how many workers `-n auto` should use, set the documented environment variable:

```bash
export PYTEST_XDIST_AUTO_NUM_WORKERS=6
pytest -n auto
```

You can also implement the `pytest_xdist_auto_num_workers(config)` hook in `conftest.py`.

## Common Workflows

### Speed up a suite

```bash
pytest -n auto tests
```

This is the standard starting point for a suite that is already stable in single-process mode.

### Choose a scheduler deliberately

```bash
pytest -n auto --dist load
pytest -n auto --dist loadscope
pytest -n auto --dist loadfile
pytest -n auto --dist loadgroup
pytest -n auto --dist worksteal
```

Use these modes when behavior matters more than raw throughput:

- `load`: default scheduler; good general throughput.
- `loadscope`: keeps tests from the same module or class on the same worker.
- `loadfile`: keeps each file on one worker.
- `loadgroup`: keeps tests sharing an `xdist_group` mark on the same worker.
- `worksteal`: redistributes queued tests from busy workers to idle ones.

### Keep related tests on one worker

When tests need to share a browser session, service container, or other stateful fixture, group them explicitly:

```python
import pytest


@pytest.mark.xdist_group("chrome")
def test_login_in_chrome() -> None:
    ...


@pytest.mark.xdist_group("chrome")
def test_checkout_in_chrome() -> None:
    ...
```

Run with:

```bash
pytest -n auto --dist loadgroup
```

### Control `loadscope` ordering in 3.8.0

`pytest-xdist 3.8.0` adds explicit `loadscope` reordering controls:

```bash
pytest -n auto --dist loadscope --no-loadscope-reorder
pytest -n auto --dist loadscope --loadscope-reorder
```

Use `--no-loadscope-reorder` when relative file ordering still matters inside `loadscope` groups.

## Worker-Aware Tests And Fixtures

### Detect the current worker

Use the built-in `worker_id` fixture when a test resource must be unique per worker:

```python
import pytest


@pytest.fixture
def user_account(worker_id: str) -> str:
    return f"account_{worker_id}"
```

When xdist is disabled, `worker_id` returns `"master"`, so the same fixture works for both local debugging and parallel CI.

### Read worker environment variables

Workers expose these environment variables:

- `PYTEST_XDIST_WORKER`
- `PYTEST_XDIST_WORKER_COUNT`
- `PYTEST_XDIST_TESTRUNUID`

Example:

```python
import os


def test_worker_env() -> None:
    worker_name = os.getenv("PYTEST_XDIST_WORKER", "master")
    worker_count = int(os.getenv("PYTEST_XDIST_WORKER_COUNT", "1"))
    assert worker_name
    assert worker_count >= 1
```

### Create one namespace per distributed run

Use `testrun_uid` when all workers should share one logical identifier for the entire run:

```python
import pytest


@pytest.fixture(scope="session")
def database_name(testrun_uid: str) -> str:
    return f"test_db_{testrun_uid}"
```

This is the right pattern for temporary databases, object stores, or directory prefixes that must be unique per CI run rather than per worker.

### Handle session fixtures correctly

High-scope fixtures are not global across all workers. A `session` fixture can run once per worker.

If setup must happen exactly once, add your own inter-process lock:

```python
import json
from pathlib import Path

import pytest
from filelock import FileLock


def produce_expensive_data() -> dict[str, str]:
    return {"token": "generated-once"}


@pytest.fixture(scope="session")
def session_data(
    tmp_path_factory: pytest.TempPathFactory,
    worker_id: str,
) -> dict[str, str]:
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

Install `filelock` yourself if you use this pattern.

## Diagnostics And Failure Handling

Cap automatic worker restarts when you want crashes to fail fast in CI:

```bash
pytest -n auto --max-worker-restart 2
pytest -n auto --max-worker-restart 0
```

For one log file per worker, use `PYTEST_XDIST_WORKER` during pytest setup:

```python
import logging
import os


def pytest_configure(config) -> None:
    worker_id = os.environ.get("PYTEST_XDIST_WORKER")
    if worker_id:
        logging.basicConfig(
            filename=f"tests_{worker_id}.log",
            level=config.getini("log_file_level"),
            format=config.getini("log_file_format"),
        )
```

## Important Pitfalls

- `-s` and `--capture=no` do not stream live worker stdout or stderr back through xdist.
- `--pdb` is disabled for distributed runs; rerun with `-n 0` or without `-n` before interactive debugging.
- Test collection order must match across workers. Avoid parametrizing from unordered sets or other unstable sources.
- `session` fixtures are not singletons across the whole run.
- Shared temp files, ports, and database names must be worker-aware or run-aware.
- `load` gives the highest throughput, but it can increase fixture churn compared with `loadscope`, `loadfile`, or `loadgroup`.
- Keep flaky tests out of parallel mode until they are deterministic in single-process runs.

## Version Notes For 3.8.0

- `3.8.0` adds `--loadscope-reorder` and `--no-loadscope-reorder`.
- Python `3.8` support was dropped in `3.7.0`, so `3.9+` is the practical baseline for `3.8.0`.
- Remote rsync mode and `--looponfail` are still documented but were deprecated earlier and are planned for removal in xdist `4.0`.
- `--boxed` is already removed from modern xdist; use `pytest-forked` if you specifically need fork isolation.

## Recommended Workflow

1. Run `pytest` without xdist first.
2. Enable parallelism with `pytest -n auto`.
3. If stateful tests churn too much, switch to `--dist loadscope`, `loadfile`, or `loadgroup`.
4. If failures only happen in parallel mode, make resource names worker-aware and rerun with a smaller fixed worker count such as `-n 2`.
5. For debugging, rerun the same subset with `-n 0` and then use your normal debugger or `--pdb`.

## Official Sources

- Stable docs: `https://pytest-xdist.readthedocs.io/en/stable/`
- Distribution guide: `https://pytest-xdist.readthedocs.io/en/stable/distribution.html`
- How-to guide: `https://pytest-xdist.readthedocs.io/en/stable/how-to.html`
- Known limitations: `https://pytest-xdist.readthedocs.io/en/stable/known-limitations.html`
- Changelog: `https://pytest-xdist.readthedocs.io/en/stable/changelog.html`
- PyPI package page: `https://pypi.org/project/pytest-xdist/`

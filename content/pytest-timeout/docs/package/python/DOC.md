---
name: package
description: "pytest-timeout plugin guide for adding per-test and session time limits to pytest suites"
metadata:
  languages: "python"
  versions: "2.4.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,testing,timeout,python,ci"
---

# pytest-timeout Python Package Guide

## What It Is

`pytest-timeout` is a pytest plugin for failing tests that hang longer than an allowed limit. Use it to keep deadlocks, infinite loops, or stuck network calls from blocking the whole test run.

Use timeouts as a safety net, not as a benchmark. If a suite needs a whole-run cap, pair `--session-timeout` with a normal per-test timeout because session timeouts are only checked between tests.

## Install

`pytest-timeout 2.4.0` requires Python `>=3.7`, and the `2.3.x` line raised the minimum supported `pytest` version to `7.0.0`.

Install it with pytest in the same environment:

```bash
python -m pip install "pytest>=7" "pytest-timeout==2.4.0"
```

There is no runtime initialization code. After install, pytest loads the plugin automatically and exposes timeout-related CLI options, config keys, and the `timeout` marker.

## Quick Start

Set a default timeout for every test in the current run:

```bash
pytest --timeout=30
```

Add a whole-suite cap as a second guardrail:

```bash
pytest --timeout=30 --session-timeout=900
```

Set a timeout on one test only:

```python
import pytest


@pytest.mark.timeout(5)
def test_fast_path() -> None:
    ...
```

Override the timeout method for one test:

```python
import pytest


@pytest.mark.timeout(10, method="thread")
def test_might_deadlock() -> None:
    ...
```

Disable a global timeout for one test:

```python
import pytest


@pytest.mark.timeout(0)
def test_intentionally_long() -> None:
    ...
```

## Configure Defaults

Timeout settings apply from lowest to highest priority:

1. `timeout` in pytest config
2. `PYTEST_TIMEOUT` environment variable
3. `--timeout` on the command line
4. `@pytest.mark.timeout(...)` on an individual test

Example `pytest.ini`:

```ini
[pytest]
timeout = 30
timeout_method = signal
session_timeout = 900
timeout_func_only = false
```

Equivalent `pyproject.toml` config:

```toml
[tool.pytest.ini_options]
timeout = 30
timeout_method = "signal"
session_timeout = 900
timeout_func_only = false
```

You can also set the default from the environment:

```bash
export PYTEST_TIMEOUT=30
pytest
```

## Choose A Timeout Method

`pytest-timeout` supports two timeout mechanisms:

- `signal`: preferred on POSIX systems where `SIGALRM` is available; pytest can usually fail the timed-out test and keep running normally
- `thread`: fallback mode that uses a timer thread and terminates the whole process when the timeout expires

Use the hyphenated CLI spelling in new commands:

```bash
pytest --timeout=30 --timeout-method=thread
```

The config key stays `timeout_method`, and older docs may still show the old CLI form `--timeout_method`.

Choose `thread` when:

- the code under test already uses signals or `SIGALRM`
- the signal method is unavailable on the platform
- hard-stop reliability matters more than preserving normal teardown and reporting

Choose `signal` when:

- you are on a POSIX platform with `SIGALRM`
- keeping normal pytest teardown and reporting is more important than a hard process exit

## Fixtures, Setup, And Teardown

By default, the timeout covers the full test lifecycle:

- fixture setup
- test body
- fixture teardown and finalizers

If fixture time should not count, enable `timeout_func_only` globally or pass `func_only=True` on the marker:

```python
import pytest


@pytest.mark.timeout(10, func_only=True)
def test_only_the_function_body_is_timed() -> None:
    ...
```

Be careful with this setting. If setup or teardown is what hangs, `func_only=True` will not protect that phase.

## Session Timeout

Use `session_timeout` or `--session-timeout` to cap the full pytest invocation:

```bash
pytest --timeout=15 --session-timeout=600
```

Important behavior:

- session timeout is checked between tests, not during a still-running test
- a stuck test still needs its own timeout to be interrupted
- combining per-test and session timeouts is the safer setup

## Debugger Behavior

The plugin tries to avoid firing while a debugger is attached. That helps local debugging, but it can also hide timeouts if a debugger stays active unexpectedly.

If you need to disable debugger detection, use the config setting:

```ini
[pytest]
timeout_disable_debugger_detection = true
```

## Common Pitfalls

- Do not use timeout failures as a substitute for making tests fast or deterministic.
- `thread` mode can end the process with `os._exit()`, which can skip teardown, truncate JUnit XML, and prevent other plugins from cleaning up cleanly.
- `signal` mode can conflict with code that already depends on `SIGALRM`.
- Fixture setup and teardown count toward the timeout unless you opt into `timeout_func_only`.
- A test-level marker overrides a global `timeout_func_only = true` unless that marker also passes `func_only=True`.
- Session timeout alone will not rescue one permanently hung test.
- Older docs may mention timeout diagnostics on `stderr`; since `2.3.0`, reporting goes through pytest's `TerminalReporter`, so the output is written through normal pytest terminal reporting.

## Version-Sensitive Notes

- `2.4.0` improves debugger detection for debuggers registered with `sys.monitoring`.
- `2.3.0` raised the minimum supported pytest version to `7.0.0`.
- `2.3.0` added `--session-timeout` and the `session_timeout` config setting.
- The preferred CLI spelling is `--timeout-method`; older examples may still use `--timeout_method`.
- PyPI metadata for `2.4.0` requires Python `>=3.7`.

## Official Source URLs

- GitHub repository: `https://github.com/pytest-dev/pytest-timeout`
- PyPI package page for `2.4.0`: `https://pypi.org/project/pytest-timeout/2.4.0/`
- PyPI latest page: `https://pypi.org/project/pytest-timeout/`
- pytest configuration reference: `https://docs.pytest.org/en/stable/reference/customize.html`
- pytest marker reference: `https://docs.pytest.org/en/stable/how-to/mark.html`

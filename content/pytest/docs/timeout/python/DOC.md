---
name: timeout
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

## Golden Rule

Use `pytest-timeout` to stop hung or deadlocked test runs, not to measure performance. Pair any whole-suite timeout with a per-test timeout, because session timeouts are cooperative and do not interrupt a single stuck test by themselves.

## Install

Install it alongside pytest in your test dependencies:

```bash
python -m pip install "pytest>=7" "pytest-timeout==2.4.0"
```

Common alternatives:

```bash
uv add --dev "pytest-timeout==2.4.0"
poetry add --group dev "pytest-timeout==2.4.0"
```

There is no runtime initialization code. After install, the plugin registers pytest options, ini settings, and the `timeout` marker automatically.

Check that pytest sees the plugin:

```bash
pytest --help | rg timeout
```

## Quick Start

Set a global timeout for the current run:

```bash
pytest --timeout=30
```

Add a session timeout as a second guardrail:

```bash
pytest --timeout=30 --session-timeout=900
```

Set a timeout on one test only:

```python
import pytest

@pytest.mark.timeout(5)
def test_fast_path():
    ...
```

Override the method for one test:

```python
import pytest

@pytest.mark.timeout(10, method="thread")
def test_might_deadlock():
    ...
```

Disable a global timeout for one test:

```python
import pytest

@pytest.mark.timeout(0)
def test_intentionally_long():
    ...
```

## Configuration

Timeout precedence is:

1. `timeout` value in pytest config
2. `PYTEST_TIMEOUT` environment variable
3. `--timeout` CLI option
4. `@pytest.mark.timeout(...)` on a specific test

Example `pytest.ini`:

```ini
[pytest]
timeout = 30
timeout_method = signal
session_timeout = 900
timeout_func_only = false
```

Equivalent `pyproject.toml` using pytest's standard ini-options table:

```toml
[tool.pytest.ini_options]
timeout = 30
timeout_method = "signal"
session_timeout = 900
timeout_func_only = false
```

You can also set a default timeout from the environment:

```bash
export PYTEST_TIMEOUT=30
pytest
```

## Choosing A Timeout Method

`pytest-timeout` supports two timeout mechanisms:

- `signal`: preferred on POSIX systems with `SIGALRM`; the test run can continue and pytest can fail the timed-out test normally
- `thread`: the safest fallback; a timer thread terminates the whole process when the timeout expires

Use the hyphenated CLI spelling in new scripts:

```bash
pytest --timeout=30 --timeout-method=thread
```

Older docs and examples may still show `--timeout_method`. The config key remains `timeout_method`.

Choose `thread` when:

- your code under test uses signals or `SIGALRM`
- the signal method is unavailable on the platform
- you need the most reliable interruption path for a truly wedged test

Choose `signal` when:

- you are on a POSIX platform with `SIGALRM`
- preserving normal pytest teardown/reporting is more important than hard-stop reliability

## Fixtures, Setup, And Teardown

By default, the timeout covers the whole test lifecycle:

- fixture setup
- test body
- fixture teardown/finalizers

If fixture time should not count toward the limit, set `timeout_func_only = true` globally or per test:

```python
import pytest

@pytest.mark.timeout(10, func_only=True)
def test_only_the_function_body_is_timed():
    ...
```

Be careful here. If a fixture is what hangs, `func_only=True` will not protect you from that setup/teardown stall.

## Session Timeout

Use `session_timeout` or `--session-timeout` to cap the full pytest invocation:

```bash
pytest --timeout=15 --session-timeout=600
```

Important behavior:

- session timeout is checked between tests, not during a still-running test
- a stuck test still needs a per-test timeout to be interrupted

## Debugger Behavior

The plugin tries to avoid firing while a debugger is attached. This is convenient for local debugging, but it can hide timeouts if you forget the debugger is active.

If debugger detection is getting in the way, use the config setting:

```ini
[pytest]
timeout_disable_debugger_detection = true
```

## Common Pitfalls

- Do not treat timeout failures as a substitute for making tests fast. The maintainers explicitly frame timeouts as a last resort.
- `thread` mode can terminate the whole process with `os._exit()`. Expect lost teardown, truncated JUnit XML, and incomplete plugin cleanup in that failure path.
- `signal` mode can conflict with code that already uses `SIGALRM`.
- Fixture setup and teardown count toward the timeout unless you opt into `timeout_func_only`.
- A decorator overrides `timeout_func_only = true` unless you pass `func_only=True` again on the marker.
- Session timeout alone will not rescue a single permanently hung test.
- Older docs may say timeout diagnostics are written to `stderr`; since `2.3.0`, pytest-timeout reports through pytest's `TerminalReporter`, which moves that output to `stdout`.

## Version-Sensitive Notes For 2.4.0

- `2.4.0` improves debugger detection for debuggers registered with `sys.monitoring`, which matters on newer Python runtimes and debugger integrations.
- `2.3.0` raised the minimum supported pytest version to `7.0.0`. If you are pinned below pytest 7, do not assume modern `pytest-timeout` examples apply.
- `2.3.0` added `--session-timeout` and the `session_timeout` config setting. Older versions do not have this whole-suite timeout feature.
- The changelog changed the preferred CLI spelling from `--timeout_method` to `--timeout-method` long ago; use the hyphenated form in new commands even though old examples still exist.
- PyPI verified metadata for `2.4.0` requires Python `>=3.7`. The project description still contains older prose saying "Python 3.6 and higher"; prefer the verified metadata.

## Official Sources

- GitHub repository: https://github.com/pytest-dev/pytest-timeout
- PyPI package page for `2.4.0`: https://pypi.org/project/pytest-timeout/2.4.0/
- PyPI latest page: https://pypi.org/project/pytest-timeout/
- pytest configuration reference: https://docs.pytest.org/en/stable/reference/customize.html
- pytest marker reference: https://docs.pytest.org/en/stable/how-to/mark.html

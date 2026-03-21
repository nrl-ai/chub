---
name: mock
description: "pytest-mock package guide for Python tests using the mocker fixture with pytest"
metadata:
  languages: "python"
  versions: "3.15.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pytest,pytest-mock,mock,testing,fixtures"
---

# pytest-mock Python Package Guide

## Golden Rule

Use `pytest-mock` when you want `unittest.mock`-style patching integrated with pytest fixture lifecycles. Install it in the same environment as pytest, use the `mocker` fixture instead of nested `with patch(...)` blocks, and patch names where the code under test looks them up.

`pytest-mock 3.15.1` is the current PyPI release listed for this package, and the upstream docs for this version live at `https://pytest-mock.readthedocs.io/en/latest/`.

## Install

Install `pytest-mock` into the same environment that runs your tests:

```bash
python -m pip install "pytest>=6.2.5" "pytest-mock==3.15.1"
```

Common alternatives:

```bash
uv add --dev "pytest>=6.2.5" "pytest-mock==3.15.1"
poetry add --group test "pytest>=6.2.5" "pytest-mock==3.15.1"
```

The plugin is auto-discovered by pytest through the `pytest11` entry point, so a normal install is enough. You usually do not need to add anything to `conftest.py` or `pytest_plugins`.

## Initialize And Verify Setup

Create a smoke test to confirm the fixture is available:

```python
import os

def test_smoke(mocker):
    mocked = mocker.patch("os.getcwd", return_value="/tmp/project")
    assert mocked() == "/tmp/project"
```

Run:

```bash
pytest
```

If pytest reports `fixture 'mocker' not found`, the package is either not installed in the active environment or pytest is running under a different interpreter than the one where you installed it.

## Core Usage

### Basic patching

`mocker` mirrors the `unittest.mock.patch` API, so standard `patch`, `patch.object`, `patch.multiple`, and `patch.dict` patterns still apply:

```python
import os

def rm(path: str) -> None:
    os.remove(path)

def test_rm_calls_os_remove(mocker) -> None:
    remove = mocker.patch("os.remove")

    rm("file.txt")

    remove.assert_called_once_with("file.txt")
```

### Prefer `autospec=True` when patching callables

`autospec=True` catches wrong signatures earlier and is usually a better default for function or method patching:

```python
from pathlib import Path

def test_listdir(mocker) -> None:
    listdir = mocker.patch.object(Path, "iterdir", autospec=True, return_value=[])

    assert list(Path(".").iterdir()) == []
    listdir.assert_called_once()
```

### Spy on real behavior

Use `mocker.spy` when the real function should still run, but you want call assertions and access to return values:

```python
class Calculator:
    def double(self, value: int) -> int:
        return value * 2

def test_spy(mocker) -> None:
    calc = Calculator()
    spy = mocker.spy(calc, "double")

    assert calc.double(21) == 42
    spy.assert_called_once_with(21)
    assert spy.spy_return == 42
```

### Stub callbacks

`mocker.stub()` is useful when your code accepts a callback and you only care that it was invoked with the right arguments:

```python
def emit(handler):
    handler("event", ok=True)

def test_stub_callback(mocker) -> None:
    handler = mocker.stub(name="handler")

    emit(handler)

    handler.assert_called_once_with("event", ok=True)
```

### Async support

`mocker` exposes `AsyncMock`, and `mocker.spy` works with `async def` functions. You still need an async test runner such as `pytest-asyncio` for `@pytest.mark.asyncio` tests:

```python
import pytest

async def fetch_name() -> str:
    return "context-hub"

@pytest.mark.asyncio
async def test_async_patch(mocker) -> None:
    mocked = mocker.patch(__name__ + ".fetch_name", new_callable=mocker.AsyncMock)
    mocked.return_value = "patched"

    assert await fetch_name() == "patched"
    mocked.assert_awaited_once()
```

### Reset or stop selected patches

Use `mocker.resetall()` to clear call history on mocks created so far. Use `mocker.stop(mock_or_spy)` to stop a specific patch or spy before the test ends.

```python
class Greeter:
    def hello(self) -> str:
        return "hello"

def test_stop_spy(mocker) -> None:
    greeter = Greeter()
    spy = mocker.spy(greeter, "hello")

    assert greeter.hello() == "hello"
    assert spy.call_count == 1

    mocker.stop(spy)
    assert greeter.hello() == "hello"
    assert spy.call_count == 1
```

### Other fixture scopes

If a test class, module, package, or session needs a wider-lived mock fixture, use:

- `class_mocker`
- `module_mocker`
- `package_mocker`
- `session_mocker`

Use wider scopes carefully. They make state leakage between tests easier if you mutate mocks and forget to reset them.

## Configuration

Most projects need no plugin-specific configuration. Install the package and use `mocker`.

There is no external authentication model for this package. Setup is local to your pytest environment.

If you specifically want the separate `mock` package instead of `unittest.mock`, configure pytest with:

```ini
# pytest.ini
[pytest]
mock_use_standalone_module = true
```

That option forces `pytest-mock` to import `mock` from PyPI instead of the standard-library `unittest.mock`.

## Typing

`pytest-mock` is fully type annotated. For annotated tests, import `MockerFixture` from `pytest_mock`:

```python
from pytest_mock import MockerFixture

def test_typed(mocker: MockerFixture) -> None:
    mocked = mocker.patch("os.getcwd", return_value="/tmp")
    assert mocked() == "/tmp"
```

For newer versions, `MockType` and `AsyncMockType` are also available from `pytest_mock` if you need explicit annotations for mock objects.

## Common Pitfalls

- Patch where the code under test looks up the symbol, not where it was originally defined. `mocker.patch("module.symbol")` still follows the same lookup rules as `unittest.mock.patch`.
- Do not use `mocker.patch(...)` or `mocker.patch.object(...)` as a context manager or decorator through the fixture. The plugin warns about that usage because the fixture already manages cleanup. If you really need to patch a real context manager object, use `mocker.patch.context_manager(...)`.
- Install `pytest-mock` in the test environment itself. A globally installed pytest plus a separate project virtualenv is a common source of `fixture 'mocker' not found`.
- Prefer `autospec=True` for function and method patches when possible. Bare mocks can hide signature mistakes.
- `mocker.resetall()` resets existing mocks; it does not replace a stopped patch. Use `mocker.stop(...)` or let fixture teardown clean up at test end.
- Wider-scoped fixtures such as `session_mocker` are convenient but can leak state between tests if reused carelessly.

## Version-Sensitive Notes For 3.15.1

- `3.15.1` is the latest PyPI release as of March 11, 2026.
- Starting in `3.15.0`, Python 3.8 is no longer supported. Stay on Python 3.9+ for this package line.
- `3.15.0` added `spy_return_iter` for `mocker.spy` when the spied function returns an iterator.
- `3.15.1` changes iterator duplication behavior: pass `duplicate_iterators=True` to `mocker.spy(...)` if you need a duplicated iterator in `spy_return_iter`.
- `spy_return_list` was added in `3.13` and can be useful when you need the full history of return values from a spy.
- `mocker.stop(...)` has been available since `3.10`; older examples that manually rebuild spies or rely on teardown-only cleanup can usually be simplified.

## Official Sources

- Documentation: `https://pytest-mock.readthedocs.io/en/latest/`
- Usage guide: `https://pytest-mock.readthedocs.io/en/latest/usage.html`
- Configuration: `https://pytest-mock.readthedocs.io/en/latest/configuration.html`
- Remarks and typing: `https://pytest-mock.readthedocs.io/en/latest/remarks.html`
- Changelog: `https://pytest-mock.readthedocs.io/en/latest/changelog.html`
- Package metadata: `https://pypi.org/project/pytest-mock/`
- Project metadata and dependencies: `https://github.com/pytest-dev/pytest-mock/blob/main/pyproject.toml`

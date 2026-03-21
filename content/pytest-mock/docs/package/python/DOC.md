---
name: package
description: "pytest-mock plugin for pytest with a mocker fixture for patching, spying, and stubbing"
metadata:
  languages: "python"
  versions: "3.15.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,pytest-mock,python,testing,mocking"
---

# pytest-mock Python Package Guide

## Golden Rule

Install `pytest-mock` in the same environment as `pytest`, then use the `mocker` fixture for patches, spies, and stubs instead of managing `unittest.mock.patch` cleanup manually. The plugin integrates with pytest and automatically undoes its mocks at the end of the fixture scope.

## Install

`pytest-mock 3.15.1` requires Python `>=3.9` and `pytest>=6.2.5`.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "pytest>=6.2.5" "pytest-mock==3.15.1"
```

Common alternatives:

```bash
uv add "pytest-mock==3.15.1"
poetry add "pytest-mock==3.15.1"
```

If you want to confirm the plugin is available in the active environment, run:

```bash
pytest --fixtures
```

Look for `mocker` and the broader-scope variants such as `module_mocker`.

## How The Plugin Loads

No authentication, environment variables, or runtime initialization are required. Once `pytest-mock` is installed, pytest discovers it as a plugin and makes these fixtures available:

- `mocker` for function scope
- `class_mocker` for class scope
- `module_mocker` for module scope
- `package_mocker` for package scope
- `session_mocker` for session scope

Typed test signature:

```python
from pytest_mock import MockerFixture

def test_example(mocker: MockerFixture) -> None:
    ...
```

## Patch Functions And Methods

Use `mocker.patch(...)` when your code looks up a symbol by import path, and `mocker.patch.object(...)` when you already have the object instance or class.

```python
# app.py
from pathlib import Path


def config_path() -> Path:
    return Path.home() / ".myapp" / "config.toml"
```

```python
# test_app.py
from pathlib import Path

from pytest_mock import MockerFixture

import app


def test_config_path(mocker: MockerFixture) -> None:
    mocker.patch("app.Path.home", return_value=Path("/tmp/test-home"))

    assert app.config_path() == Path("/tmp/test-home/.myapp/config.toml")
```

```python
# notifier.py
class Mailer:
    def send(self, recipient: str) -> bool:
        raise NotImplementedError


def notify(mailer: Mailer, recipient: str) -> bool:
    return mailer.send(recipient)
```

```python
# test_notifier.py
from pytest_mock import MockerFixture

from notifier import Mailer, notify


def test_notify(mocker: MockerFixture) -> None:
    mailer = Mailer()
    mocked_send = mocker.patch.object(mailer, "send", return_value=True)

    assert notify(mailer, "dev@example.com") is True
    mocked_send.assert_called_once_with("dev@example.com")
```

## Patch Dicts And Environment Variables

Use `mocker.patch.dict(...)` for dictionaries and `os.environ`.

```python
# settings.py
import os


def feature_enabled() -> bool:
    return os.environ.get("FEATURE_FLAG") == "1"
```

```python
# test_settings.py
import os

from pytest_mock import MockerFixture

import settings


def test_feature_enabled(mocker: MockerFixture) -> None:
    mocker.patch.dict(os.environ, {"FEATURE_FLAG": "1"})

    assert settings.feature_enabled() is True
```

## Spy On Real Calls

Use `mocker.spy(...)` when you want the real implementation to run but still want call assertions.

```python
# greeter.py
def normalize(name: str) -> str:
    return name.strip().lower()


def greet(name: str) -> str:
    return f"hi {normalize(name)}"
```

```python
# test_greeter.py
from pytest_mock import MockerFixture

import greeter


def test_greet_uses_normalize(mocker: MockerFixture) -> None:
    spy = mocker.spy(greeter, "normalize")

    assert greeter.greet(" Alice ") == "hi alice"

    spy.assert_called_once_with(" Alice ")
    assert spy.spy_return == "alice"
```

If you only want to observe the first part of a test, stop the spy explicitly:

```python
from pytest_mock import MockerFixture

import greeter


def test_stop_spy(mocker: MockerFixture) -> None:
    spy = mocker.spy(greeter, "normalize")

    greeter.greet(" Alice ")
    mocker.stop(spy)
    greeter.greet(" Bob ")

    assert spy.call_count == 1
```

## Use Stubs For Callbacks

Use `mocker.stub()` when the test only needs a callable placeholder and assertion target.

```python
# worker.py
def run_job(on_complete) -> None:
    on_complete("job-123")
```

```python
# test_worker.py
from pytest_mock import MockerFixture

import worker


def test_run_job_calls_callback(mocker: MockerFixture) -> None:
    on_complete = mocker.stub(name="on_complete")

    worker.run_job(on_complete)

    on_complete.assert_called_once_with("job-123")
```

## Broader Fixture Scopes

`mocker` is function-scoped. When a patch must live across more than one test, request the fixture with the matching pytest scope instead of carrying state manually.

```python
# conftest.py
import pytest


@pytest.fixture(scope="module", autouse=True)
def patch_api_root(module_mocker) -> None:
    module_mocker.patch("app.API_ROOT", "https://example.test")
```

Use the narrowest scope that matches the test lifetime you actually need.

## Configuration

In the normal case, no plugin configuration is required.

By default, `pytest-mock` uses the standard library `unittest.mock` module. If your project explicitly depends on the standalone `mock` package instead, enable it in `pytest.ini`:

```ini
[pytest]
mock_use_standalone_module = true
```

Install the standalone package only if your project needs it:

```bash
python -m pip install mock
```

## Typing

Import `MockerFixture` from `pytest_mock` when you want explicit type annotations in tests.

```python
from pytest_mock import MockerFixture


def test_compute(mocker: MockerFixture) -> None:
    mocked = mocker.patch("app.compute", return_value=3)

    assert mocked.return_value == 3
```

## Common Pitfalls

- Patch the name your code actually uses. If `app.py` imported `Path` directly, patch `app.Path.home`, not `pathlib.Path.home`.
- Use `mocker.patch(...)` directly. The fixture already handles teardown, so you usually do not need patch decorators or context-manager cleanup.
- Keep fixture scope in mind. `mocker` resets after each test; use `class_mocker`, `module_mocker`, `package_mocker`, or `session_mocker` only when the broader lifetime is intentional.
- Install both `pytest` and `pytest-mock`. The plugin does not replace pytest itself.
- If a project still has to run on Python 3.8, it cannot use `pytest-mock 3.15.x`.

## Version-Sensitive Notes For 3.15.x

- This guide targets `pytest-mock 3.15.1`.
- `3.15.0` dropped Python 3.8 support, so `3.15.x` requires Python 3.9 or newer.
- `3.15.1` changed `mocker.spy(..., duplicate_iterators=True)` behavior for iterator duplication. Check the 3.15.1 changelog before reusing older spy assertions for iterator-returning functions.

## Official Sources

- Docs root: https://pytest-mock.readthedocs.io/en/latest/
- Usage: https://pytest-mock.readthedocs.io/en/latest/usage.html
- Configuration: https://pytest-mock.readthedocs.io/en/latest/configuration.html
- Remarks: https://pytest-mock.readthedocs.io/en/latest/remarks.html
- Changelog: https://pytest-mock.readthedocs.io/en/latest/changelog.html
- PyPI package page: https://pypi.org/project/pytest-mock/
- Project metadata: https://github.com/pytest-dev/pytest-mock/blob/main/pyproject.toml

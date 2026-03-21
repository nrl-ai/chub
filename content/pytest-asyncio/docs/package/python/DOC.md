---
name: package
description: "pytest-asyncio guide for writing and configuring asyncio tests with pytest"
metadata:
  languages: "python"
  versions: "1.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,pytest-asyncio,python,asyncio,testing"
---

# pytest-asyncio Python Package Guide

## Golden Rule

`pytest-asyncio` is a pytest plugin for `asyncio` tests and fixtures. After installation, pytest loads the plugin automatically. There is no client object to initialize and no package-specific environment variable to set.

Choose one discovery mode up front:

- `strict` is the default and is the safest choice when your test suite mixes multiple async backends or plugins
- `auto` is simpler for asyncio-only projects because async tests and fixtures are handled automatically

## Install

Version `1.3.0` requires Python 3.10 or newer.

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "pytest" "pytest-asyncio==1.3.0"
```

Common alternatives:

```bash
uv add "pytest-asyncio==1.3.0"
poetry add "pytest-asyncio==1.3.0"
```

## Configure pytest

Set the asyncio mode explicitly in your pytest config. Also set `asyncio_default_fixture_loop_scope` explicitly, because upstream warns that the implicit default will change in a future release.

`pyproject.toml`:

```toml
[tool.pytest.ini_options]
asyncio_mode = "strict"
asyncio_default_fixture_loop_scope = "function"
```

For an asyncio-only project, switch to `auto` instead:

```toml
[tool.pytest.ini_options]
asyncio_mode = "auto"
asyncio_default_fixture_loop_scope = "function"
```

## Write Async Tests

In `strict` mode, mark async tests with `@pytest.mark.asyncio`.

```python
import asyncio
import pytest


async def get_value() -> str:
    await asyncio.sleep(0)
    return "ok"


@pytest.mark.asyncio
async def test_get_value() -> None:
    assert await get_value() == "ok"
```

Run the test suite with normal pytest commands:

```bash
pytest
pytest tests/test_async_code.py -q
```

If you configure `asyncio_mode = "auto"`, async test functions do not need the explicit `@pytest.mark.asyncio` marker.

## Write Async Fixtures

In `strict` mode, async fixtures should use `@pytest_asyncio.fixture`.

```python
import pytest
import pytest_asyncio


class FakeClient:
    def __init__(self) -> None:
        self.closed = False

    async def fetch_status(self) -> str:
        return "ok"

    async def aclose(self) -> None:
        self.closed = True


@pytest_asyncio.fixture
async def client() -> FakeClient:
    resource = FakeClient()
    yield resource
    await resource.aclose()


@pytest.mark.asyncio
async def test_client_fixture(client: FakeClient) -> None:
    assert await client.fetch_status() == "ok"
```

If you switch to `auto` mode, pytest-asyncio can also manage async fixtures declared with the normal `@pytest.fixture` decorator.

## Control Event Loop Scope

Use `loop_scope` when a test should run in a loop that lives longer than one function call.

```python
import asyncio
import pytest


@pytest.mark.asyncio(loop_scope="module")
async def test_uses_module_loop() -> None:
    loop = asyncio.get_running_loop()
    assert loop.is_running()
```

Use wider scopes deliberately and keep related tests on the same scope when they share loop-sensitive state.

## Replace The Removed `event_loop` Fixture

`pytest-asyncio 1.x` no longer provides the deprecated `event_loop` fixture. Inside async tests or fixtures, get the running loop directly:

```python
import asyncio
import pytest


@pytest.mark.asyncio
async def test_current_loop() -> None:
    loop = asyncio.get_running_loop()
    assert loop.is_running()
```

If a group of tests or fixtures must share a longer-lived loop, configure `loop_scope` or the relevant asyncio loop-scope setting instead of reintroducing `event_loop`.

## Common Pitfalls

- Async tests are skipped or treated as normal callables in `strict` mode when `@pytest.mark.asyncio` is missing
- Async fixtures use `@pytest.fixture` in `strict` mode instead of `@pytest_asyncio.fixture`
- `asyncio_default_fixture_loop_scope` is left unset, which can make future upgrades noisier when the upstream default changes
- Legacy examples still reference the removed `event_loop` fixture; use `asyncio.get_running_loop()` in async code instead

## Version Notes

- `1.0.0` removes the deprecated `event_loop` fixture
- `1.3.0` adds support for `pytest 9` and drops Python 3.9, so plan on Python 3.10+

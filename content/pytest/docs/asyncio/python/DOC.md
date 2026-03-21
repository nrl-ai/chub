---
name: asyncio
description: "pytest-asyncio package guide for asyncio-based pytest test suites in Python"
metadata:
  languages: "python"
  versions: "1.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-asyncio,pytest,asyncio,testing,python"
---

# pytest-asyncio Python Package Guide

## Golden Rule

Use the `stable` docs for `pytest-asyncio 1.3.0`, not the previous `en/latest` docs root. As of March 12, 2026, `https://pytest-asyncio.readthedocs.io/en/latest/` is serving `0.1.dev1` content, while `https://pytest-asyncio.readthedocs.io/en/stable/` matches the current `1.3.0` release.

## What It Does

`pytest-asyncio` is a `pytest` plugin for running `asyncio`-based tests and fixtures. It gives async test functions an event loop, supports async fixtures, and lets you control loop scope per test, fixture, class, module, package, or session.

It is for `asyncio`. If the project mixes multiple async backends such as Trio and asyncio, be deliberate about plugin mode and fixture decorators.

## Install

Install it as a test dependency alongside `pytest`:

```bash
python -m pip install "pytest>=8.2,<10" "pytest-asyncio==1.3.0"
```

Common alternatives:

```bash
uv add --dev "pytest>=8.2,<10" "pytest-asyncio==1.3.0"
poetry add --group test "pytest>=8.2,<10" "pytest-asyncio==1.3.0"
```

`pytest` auto-loads installed plugins, so no manual plugin registration is normally needed.

## Minimal Setup

Strict mode is the default. In strict mode, async tests need the `asyncio` marker and async fixtures should use `@pytest_asyncio.fixture`.

```python
import asyncio

import pytest

@pytest.mark.asyncio
async def test_sleep():
    await asyncio.sleep(0)
    assert True
```

Run tests normally:

```bash
pytest
```

## Recommended Project Configuration

If the project uses only `asyncio`, set auto mode in `pyproject.toml` so agents do not have to remember `@pytest.mark.asyncio` on every async test:

```toml
[tool.pytest.ini_options]
asyncio_mode = "auto"
asyncio_default_fixture_loop_scope = "function"
asyncio_default_test_loop_scope = "function"
```

Why set both loop-scope defaults explicitly:

- `asyncio_mode` defaults to `strict` if unset.
- `asyncio_default_test_loop_scope` already defaults to `function`.
- `asyncio_default_fixture_loop_scope` currently defaults to the fixture scope when unset, but upstream says a future version will change the unset default to `function`.

Set `asyncio_default_fixture_loop_scope` explicitly to avoid future behavior drift.

## Core Usage

### Async tests in strict mode

```python
import pytest

@pytest.mark.asyncio
async def test_fetch(client):
    response = await client.get("/health")
    assert response.status_code == 200
```

### Async tests in auto mode

With `asyncio_mode = "auto"`, plain async test functions are managed automatically:

```python
async def test_fetch(client):
    response = await client.get("/health")
    assert response.status_code == 200
```

### Async fixtures

Use `@pytest_asyncio.fixture` when you need an async fixture, especially in strict mode:

```python
import pytest_asyncio

@pytest_asyncio.fixture
async def seeded_db():
    db = await create_test_db()
    await db.seed()
    try:
        yield db
    finally:
        await db.close()
```

Then use it from async tests:

```python
import pytest

@pytest.mark.asyncio
async def test_query(seeded_db):
    rows = await seeded_db.fetch_all()
    assert rows
```

## Event Loop Scope

By default, tests run with function-scoped event loops for maximum isolation. If nearby tests must share the same loop, use `loop_scope`.

Module-scoped loop:

```python
import asyncio

import pytest

pytestmark = pytest.mark.asyncio(loop_scope="module")

async def test_remember_loop():
    loop = asyncio.get_running_loop()
    assert loop.is_running()

async def test_same_loop_again():
    assert asyncio.get_running_loop().is_running()
```

Fixture loop scope:

```python
import asyncio

import pytest
import pytest_asyncio

@pytest_asyncio.fixture(loop_scope="module")
async def current_loop():
    return asyncio.get_running_loop()

@pytest.mark.asyncio(loop_scope="module")
async def test_uses_module_loop(current_loop):
    assert current_loop is asyncio.get_running_loop()
```

Keep neighboring tests on the same loop scope when they logically share setup. Mixing loop scopes in the same module or class makes failures harder to reason about.

## Configuration Reference

Supported config knobs for most projects:

- `asyncio_mode = "strict" | "auto"`
- `asyncio_default_fixture_loop_scope = "function" | "class" | "module" | "package" | "session"`
- `asyncio_default_test_loop_scope = "function" | "class" | "module" | "package" | "session"`
- `asyncio_debug = true | false`

CLI overrides:

```bash
pytest --asyncio-mode=strict
pytest --asyncio-debug
```

There is no auth setup for this package. Configuration is entirely through pytest config files and CLI flags.

## Common Pitfalls

- Do not rely on `en/latest` docs for this package until upstream fixes the Read the Docs mapping. Use `en/stable` or versioned pages instead.
- Default mode is `strict`, not `auto`. Unmarked async tests can be collected without being handled the way you expect.
- In strict mode, async fixtures should use `@pytest_asyncio.fixture`, not plain `@pytest.fixture`.
- Async tests still run sequentially. `pytest-asyncio` does not make the test suite concurrent by default.
- `unittest.TestCase` subclasses are not supported. Use pytest-style tests or `unittest.IsolatedAsyncioTestCase`.
- Avoid carrying forward old examples that request the deprecated `event_loop` fixture inside async tests. Use `asyncio.get_running_loop()` instead.
- If you need a different loop implementation, prefer changing the event loop policy instead of re-implementing the old `event_loop` fixture pattern.

## Version-Sensitive Notes

- `1.3.0` adds support for `pytest 9` and removes Python `3.9` support.
- `1.2.0` adds `--asyncio-debug` and `asyncio_debug`, plus validation for invalid loop-scope config values.
- `1.0.0` removed the deprecated `event_loop` fixture. If you are upgrading from `0.21.x` or `0.23.x`, migrate async tests and async fixtures to `asyncio.get_running_loop()` and `loop_scope`.
- The `0.23.x` series had known event-loop/fixture-scope issues that upstream explicitly called out. Prefer `1.x` for new work unless a project is pinned for compatibility reasons.

## Migration Notes For Older Suites

If a test suite still has pre-`1.0` patterns:

1. Remove async test arguments named `event_loop`.
2. Replace them with `loop = asyncio.get_running_loop()` inside the coroutine.
3. Replace old fixture redefinitions with `@pytest_asyncio.fixture(...)`.
4. Set `loop_scope` explicitly where tests or fixtures need shared loops.
5. Set `asyncio_default_fixture_loop_scope = "function"` unless the suite intentionally shares broader-scoped loops.

## Official Links

- Stable docs: https://pytest-asyncio.readthedocs.io/en/stable/
- Concepts: https://pytest-asyncio.readthedocs.io/en/stable/concepts.html
- Configuration: https://pytest-asyncio.readthedocs.io/en/stable/reference/configuration.html
- Decorators: https://pytest-asyncio.readthedocs.io/en/stable/reference/decorators/
- Changelog: https://pytest-asyncio.readthedocs.io/en/stable/reference/changelog.html
- Migration from 0.21: https://pytest-asyncio.readthedocs.io/en/stable/how-to-guides/migrate_from_0_21.html
- Migration from 0.23: https://pytest-asyncio.readthedocs.io/en/stable/how-to-guides/migrate_from_0_23.html
- PyPI: https://pypi.org/project/pytest-asyncio/

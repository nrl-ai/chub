---
name: lazy-fixture
description: "pytest-lazy-fixture package guide for parametrizing pytest tests with fixture values"
metadata:
  languages: "python"
  versions: "0.6.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "pytest,testing,fixtures,parametrize,plugin"
---

# pytest-lazy-fixture Python Package Guide

## What It Is

`pytest-lazy-fixture` is a small pytest plugin that lets you reference fixtures inside `@pytest.mark.parametrize(...)` and fixture `params=[...]` lists.

Use it when you want parametrized test cases to mix:

- literal values
- fixture-backed values
- fixtures that are already parametrized themselves

The plugin is install-only under normal pytest plugin autoload. It registers a `pytest11` entry point and exposes both `from pytest_lazyfixture import lazy_fixture` and `pytest.lazy_fixture(...)`.

## Install

For a new project, pin both the plugin and pytest explicitly. Upstream has an open issue for `pytest==8.0.0`, so `pytest<8` is the safe choice for this package version.

```bash
python -m pip install "pytest-lazy-fixture==0.6.3" "pytest<8"
```

With `uv`:

```bash
uv add "pytest-lazy-fixture==0.6.3" "pytest<8"
```

If your environment already manages pytest separately:

```bash
python -m pip install "pytest-lazy-fixture==0.6.3"
```

## Initialize And Setup

There is no auth or remote configuration.

Minimal setup:

1. Install the package into the same environment as pytest.
2. Import `lazy_fixture` in test modules, or use `pytest.lazy_fixture(...)`.
3. Run tests normally with `pytest`.

Typical import pattern:

```python
import pytest
from pytest_lazyfixture import lazy_fixture
```

## Core Usage

### Use a fixture inside `@pytest.mark.parametrize`

```python
import pytest
from pytest_lazyfixture import lazy_fixture

@pytest.fixture
def one():
    return 1

@pytest.mark.parametrize(
    ("label", "value"),
    [
        ("literal", 0),
        ("fixture", lazy_fixture("one")),
    ],
)
def test_value(label, value):
    assert value in {0, 1}
```

### Use a parametrized fixture as a single param source

```python
import pytest
from pytest_lazyfixture import lazy_fixture

@pytest.fixture(params=[1, 2])
def one(request):
    return request.param

@pytest.mark.parametrize("value", [lazy_fixture("one")])
def test_parametrized_fixture(value):
    assert value in {1, 2}
```

This expands as you would expect: pytest generates one test per fixture parameter value.

### Use lazy fixtures inside another fixture's `params`

```python
import pytest
from pytest_lazyfixture import lazy_fixture

@pytest.fixture
def sqlite_url():
    return "sqlite:///test.db"

@pytest.fixture
def postgres_url():
    return "postgresql://localhost/test"

@pytest.fixture(params=lazy_fixture(["sqlite_url", "postgres_url"]))
def database_url(request):
    return request.param

def test_database_url(database_url):
    assert "://" in database_url
```

`lazy_fixture()` accepts either one fixture name or a list of fixture names.

### `pytest.lazy_fixture(...)` also works

The plugin assigns `pytest.lazy_fixture = lazy_fixture` during pytest startup, so these are equivalent:

```python
from pytest_lazyfixture import lazy_fixture

lazy_fixture("user")
```

```python
import pytest

pytest.lazy_fixture("user")
```

Prefer the direct import when you want explicit module dependencies and better static-analysis clarity.

## Configuration Notes

This package has no dedicated config file and no runtime options of its own.

The only setup that usually matters is pytest version selection:

- pin `pytest-lazy-fixture==0.6.3` if you need reproducible behavior
- pin `pytest<8` unless you have locally verified a patched fork or workaround
- keep the plugin in the same virtual environment as the `pytest` executable you run

## Common Pitfalls

### `pytest==8.x` breakage

Upstream issue `#65` reports:

- `pytest-lazy-fixture` does not work with `pytest==8.0.0`
- the observed error is `AttributeError: 'CallSpec2' object has no attribute 'funcargs'`

If you hit that failure, the first fix is to downgrade pytest:

```bash
python -m pip install "pytest<8" --upgrade
```

### Do not treat it as normal fixture injection

`lazy_fixture(...)` is only for places where pytest expects parameter values during collection, such as:

- `@pytest.mark.parametrize(...)`
- fixture `params=[...]`

For ordinary test dependencies, use regular fixture arguments:

```python
def test_user(user):
    assert user.id
```

### This plugin is sensitive to pytest internals

The implementation reaches into pytest internals such as `item.funcargs`, `metafunc._calls`, and `FixtureManager.getfixtureclosure()`. In practice, that makes the plugin version-sensitive when pytest changes collection internals.

Treat this package as a compatibility shim, not a future-proof abstraction.

## Version-Sensitive Notes

- PyPI currently lists `0.6.3` as the latest release, published on February 1, 2020.
- The published package metadata still declares Python classifiers only through Python 3.7.
- The repository's GitHub releases UI is older than PyPI and shows `0.6.0` as the latest GitHub release entry, so use PyPI as the canonical package-version source.
- The plugin source contains compatibility branches for older pytest series, including comments for `<3.6`, `3.6.x`, and `>=5.3`, which is another signal that behavior is tightly coupled to pytest internals.

## Official Sources

- GitHub repository: `https://github.com/TvoroG/pytest-lazy-fixture`
- README / maintainer docs: `https://github.com/TvoroG/pytest-lazy-fixture`
- Source file: `https://github.com/TvoroG/pytest-lazy-fixture/blob/master/pytest_lazyfixture.py`
- Packaging metadata: `https://github.com/TvoroG/pytest-lazy-fixture/blob/master/setup.py`
- PyPI registry page: `https://pypi.org/project/pytest-lazy-fixture/`
- Pytest 8 compatibility issue: `https://github.com/TvoroG/pytest-lazy-fixture/issues/65`

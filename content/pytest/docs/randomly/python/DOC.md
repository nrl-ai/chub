---
name: randomly
description: "pytest-randomly plugin guide for pytest test ordering and reproducible random seeds in Python"
metadata:
  languages: "python"
  versions: "4.0.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,pytest-randomly,testing,python,random,seeding"
---

# pytest-randomly Python Package Guide

## Golden Rule

Use `pytest-randomly` as an automatically loaded pytest plugin, and treat the printed seed as the first debugging artifact when order-dependent failures appear.

As of March 12, 2026, the version used here `4.0.1` still matches PyPI. The current maintainer README on the `main` branch describes the same core workflow, but it also says current branch support is Python 3.10 to 3.14, while the PyPI release metadata for `4.0.1` still declares `Requires-Python >=3.9`. That likely reflects main-branch support moving ahead of the last release.

## Install

Install it into the same environment as `pytest`:

```bash
python -m pip install "pytest-randomly==4.0.1"
```

Common alternatives:

```bash
uv add --dev "pytest-randomly==4.0.1"
poetry add --group test "pytest-randomly==4.0.1"
```

If you just need the latest compatible release:

```bash
python -m pip install pytest-randomly
```

## How It Works

`pytest-randomly` is a pytest plugin, not a library you normally import in tests. Once installed, pytest auto-discovers it and applies these behaviors by default:

- Randomizes test order by module, then class, then function.
- Prints a base seed at the start of the run.
- Resets Python's global `random.seed()` before test setup, run, and teardown.
- Resets supported libraries' random state when they are installed, including `factory_boy`, `Faker`, `Model Bakery`, and NumPy's legacy random state.
- Works with `pytest-xdist`.

## Basic Usage

Run pytest normally:

```bash
pytest
```

You should see a line like:

```text
Using --randomly-seed=1553614239
```

Reuse that seed to reproduce an order-dependent or random-data-dependent failure:

```bash
pytest --randomly-seed=1553614239
```

You can also reuse the last stored seed:

```bash
pytest --randomly-seed=last
```

`last` depends on pytest's cache provider. If the `cacheprovider` plugin is disabled, use an explicit integer seed instead.

## Project Setup

Persist your preferred behavior in pytest config so local runs and CI use the same defaults.

`pyproject.toml`:

```toml
[tool.pytest.ini_options]
addopts = "--randomly-seed=last"
```

Equivalent `pytest.ini`:

```ini
[pytest]
addopts = --randomly-seed=last
```

For CI, prefer one of these approaches:

- Use `--randomly-seed=last` when the cache is preserved between runs.
- Use an explicit seed in reruns when reproducing a specific failure.
- Let normal runs randomize freely so hidden inter-test dependencies keep surfacing.

## Useful Flags

Keep the default behavior unless you have a concrete reason to narrow it.

Disable only the per-test reseeding:

```bash
pytest --randomly-dont-reset-seed
```

Disable only test reordering:

```bash
pytest --randomly-dont-reorganize
```

Disable the plugin entirely:

```bash
pytest -p no:randomly
```

## Core Debugging Workflow

When a failure appears only under a specific order:

1. Re-run with the printed seed.
2. Narrow the target to the failing module or class while keeping the same seed.
3. Look for leaked global state, shared fixtures, database residue, monkeypatches not undone, or random test data that was never pinned.

Example:

```bash
pytest --randomly-seed=1234 tests/test_orders.py
pytest --randomly-seed=1234 tests/test_orders.py::TestCheckoutFlow
```

Because ordering is module -> class -> function, narrowing the scope while keeping the same seed is often the fastest way to isolate the dependency.

## Integration With Random-Using Libraries

`pytest-randomly` can reset more than the stdlib RNG:

- `factory_boy`
- `Faker`
- `Model Bakery`
- NumPy legacy random state

This is useful when tests generate data through factories or fake data providers and still need deterministic reproduction from one seed.

If your own package has another random generator, you can register a setuptools entry point so `pytest-randomly` reseeds it for every test.

`setup.cfg`:

```ini
[options.entry_points]
pytest_randomly.random_seeder =
    mypackage = mypackage.reseed
```

Implement the callable to accept the new integer seed:

```python
def reseed(new_seed: int) -> None:
    ...
```

## Config And Auth

There is no auth or service configuration. The only setup is pytest/plugin configuration.

Configuration surface to care about in practice:

- CLI flags such as `--randomly-seed`, `--randomly-dont-reset-seed`, and `--randomly-dont-reorganize`
- shared pytest config in `pyproject.toml` or `pytest.ini`
- optional pytest plugin loading control with `-p no:randomly`
- optional `pytest_randomly.random_seeder` entry points for custom RNGs

## Common Pitfalls

- Do not try to `import pytest_randomly` in normal test code just to "activate" it. Installation is enough.
- Do not ignore the printed seed when a flaky failure appears. Without it, reproducing the same order is much harder.
- `--randomly-seed=last` only works when pytest cache is available.
- If tests implicitly rely on process-global RNG state, installing this plugin can surface failures immediately. That is usually the point, not a plugin bug.
- NumPy support only covers the legacy random state. If your test suite uses `numpy.random.default_rng()`, verify your own reproducibility strategy.
- If you disable reordering to make failures disappear, you may hide real inter-test dependencies instead of fixing them.
- When combining with `pytest-order`, read that plugin's docs for interaction rules instead of assuming both plugins will preserve your intended order automatically.

## Version-Sensitive Notes For 4.0.1

- PyPI shows `pytest-randomly 4.0.1` released on September 12, 2025.
- PyPI release metadata for `4.0.1` declares `Requires-Python >=3.9`.
- The current GitHub README says the `main` branch supports Python 3.10 to 3.14. Treat that as newer maintainer guidance for current development, not necessarily a strict statement about the already-published `4.0.1` wheel.
- The docs URL points to the GitHub repository, which is the canonical maintainer doc surface for this package; there is no separate hosted documentation site to prefer here.

## Official Sources

- Maintainer docs and usage guide: https://github.com/pytest-dev/pytest-randomly
- Package registry and release metadata: https://pypi.org/project/pytest-randomly/

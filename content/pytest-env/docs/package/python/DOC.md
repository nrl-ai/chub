---
name: package
description: "pytest-env plugin guide for declaring test-only environment variables in pytest configuration"
metadata:
  languages: "python"
  versions: "1.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest-env,pytest,python,testing,environment-variables,toml"
---

# pytest-env Python Package Guide

## What It Does

`pytest-env` is a pytest plugin for setting environment variables during pytest runs from project configuration.

Use it when your tests need stable, test-only values such as:

- `APP_ENV=test`
- `DATABASE_URL` pointing at a test database
- feature flags or service endpoints that should differ from local development defaults

There is no runtime client to initialize and no auth flow. Once `pytest-env` is installed in the test environment, pytest discovers the plugin automatically.

## Install

Current maintainer metadata documents `Python >=3.10` and `pytest >=9.0.2` for the current package line. Install the plugin into your development or CI test environment:

```bash
python -m pip install "pytest==9.0.2" "pytest-env==1.5.0"
```

Common alternatives:

```bash
uv add --dev "pytest==9.0.2" "pytest-env==1.5.0"
poetry add --group dev "pytest==9.0.2" "pytest-env==1.5.0"
```

Verify that pytest is available on the active environment before debugging config issues:

```bash
pytest --version
```

## Minimal Setup With `pyproject.toml`

Current maintainer guidance is centered on native TOML configuration. For a project using `pyproject.toml`, keep normal pytest options under `[tool.pytest]` and plugin-managed environment variables under `[tool.pytest_env]`.

`pyproject.toml`:

```toml
[tool.pytest]
testpaths = ["tests"]
addopts = ["-q"]

[tool.pytest_env]
APP_ENV = "test"
API_BASE_URL = "https://api.example.test"
FEATURE_X_ENABLED = "0"
```

Test code reads those values from the normal process environment:

```python
import os


def test_env_is_available() -> None:
    assert os.environ["APP_ENV"] == "test"
    assert os.environ["API_BASE_URL"] == "https://api.example.test"
    assert os.environ["FEATURE_X_ENABLED"] == "0"
```

Run the suite normally:

```bash
pytest
```

`pytest-env` sets values for the pytest process, so application code should continue to use `os.environ` or `os.getenv()` as usual.

## `pytest.toml` Layout

If the project uses `pytest.toml` instead of `pyproject.toml`, place plugin configuration under `[pytest_env]` rather than `[tool.pytest_env]`.

Minimal plugin section:

```toml
[pytest_env]
APP_ENV = "test"
```

This matches the current pytest 9 native TOML layout described by the maintainer docs.

## Common Workflows

### Keep Shared Test Defaults In VCS

Use checked-in config for non-secret defaults that every developer and CI runner should share:

```toml
[tool.pytest_env]
APP_ENV = "test"
LOG_LEVEL = "warning"
PAYMENTS_ENABLED = "0"
```

This is usually a better fit than relying on every shell session to export the same values manually.

### Read Settings In Application Code Normally

`pytest-env` is only responsible for preparing the environment. Your code should keep using standard environment access patterns:

```python
import os


def load_settings() -> dict[str, str]:
    return {
        "app_env": os.environ.get("APP_ENV", "dev"),
        "api_base_url": os.environ["API_BASE_URL"],
    }
```

That keeps production, local development, and pytest runs aligned around the same configuration interface.

### Use `.env` Or One-Off Overrides When Values Should Not Be Checked In

The current maintainer README also documents `.env`-based loading and runtime override workflows. Prefer those patterns when:

- values differ by machine
- CI injects secrets dynamically
- you need a one-off override for a single test run

For those cases, follow the current README for the exact option names and file-based workflow instead of inventing custom bootstrap code in `conftest.py`.

## Config And Auth

There is no auth or service initialization step.

In practice, the main configuration surface is:

- pytest config in `pyproject.toml` or `pytest.toml`
- environment variable reads in your application code via `os.environ` or `os.getenv()`
- optional `.env` or CLI override features documented by the maintainer README

## Common Pitfalls

- Put plugin config in the correct table: `[tool.pytest_env]` for `pyproject.toml` and `[pytest_env]` for `pytest.toml`.
- Do not try to activate the plugin with `import pytest_env` in tests or `conftest.py`; installation is enough.
- Remember that `pytest-env` affects pytest runs, not your interactive shell and not standalone `python script.py` execution.
- Do not commit production credentials into pytest config. Checked-in values should be test defaults only.
- If a repository already uses pytest 9 TOML config, avoid mixing in older blog-post examples without checking the current README first.

## Version-Sensitive Notes For `1.5.0`

- This guide targets `pytest-env 1.5.0`.
- The canonical maintainer documentation is the GitHub repository README; there is no separate hosted docs site to prefer here.
- Current maintainer metadata documents `Python >=3.10`, `pytest >=9.0.2`, and `python-dotenv >=1.2.1` for the current package line.
- Current maintainer guidance is aligned with pytest 9 native TOML configuration. That matters because older examples built around legacy pytest config files may not match current usage.

## Official Sources

- Maintainer docs: `https://github.com/pytest-dev/pytest-env`
- Maintainer package metadata: `https://github.com/pytest-dev/pytest-env/blob/main/pyproject.toml`
- PyPI package page: `https://pypi.org/project/pytest-env/`
- PyPI JSON metadata: `https://pypi.org/pypi/pytest-env/json`
- Pytest configuration docs: `https://docs.pytest.org/en/latest/customize.html`

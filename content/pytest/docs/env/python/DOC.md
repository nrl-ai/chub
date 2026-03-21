---
name: env
description: "pytest-env package guide for configuring test environment variables in pytest"
metadata:
  languages: "python"
  versions: "1.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pytest,testing,environment,configuration,python"
---

# pytest-env Python Package Guide

## Golden Rule

Use `pytest-env` as a pytest plugin, not as an importable runtime library. Put test-specific variables in native TOML config under `[tool.pytest_env]` or `[pytest_env]`, use `.env` files only for bulk values, and use `--pytest-env-verbose` any time precedence is unclear.

As of March 12, 2026, PyPI lists `pytest-env 1.5.0` as the latest release. The maintainer metadata currently documents `Python >=3.10`, and the current upstream project metadata also declares `pytest>=9.0.2`.

## Install

Install the plugin into the same environment where `pytest` runs:

```bash
python -m pip install "pytest-env==1.5.0"
```

Common alternatives:

```bash
uv add --dev "pytest-env==1.5.0"
poetry add --group test "pytest-env==1.5.0"
```

Quick sanity check that the plugin is active:

```bash
pytest --help | rg 'envfile|pytest-env-verbose'
```

You normally do not import `pytest_env` in test code. Pytest discovers the plugin from the installed package.

## Initialize With Native TOML

The maintainer README treats native TOML as the main configuration format.

`pyproject.toml`:

```toml
[tool.pytest_env]
DATABASE_URL = "postgresql://localhost/test_db"
DEBUG = "true"
```

Or use a dedicated pytest TOML file:

```toml
# pytest.toml or .pytest.toml
[pytest_env]
DATABASE_URL = "postgresql://localhost/test_db"
DEBUG = "true"
```

A test reads these values through `os.environ`:

```python
import os

def test_database_settings() -> None:
    assert os.environ["DATABASE_URL"] == "postgresql://localhost/test_db"
    assert os.environ["DEBUG"] == "true"
```

Debug what the plugin actually applied:

```bash
pytest --pytest-env-verbose
```

## Core Configuration Patterns

### Plain values

Plain values are converted to strings:

```toml
[tool.pytest_env]
API_URL = "http://localhost:8000"
PORT = 8000
```

### Transform placeholders explicitly in TOML

In TOML mode, expansion is off unless `transform = true` is set:

```toml
[tool.pytest_env]
RUN_PATH = { value = "{HOME}/tmp/test-data", transform = true }
```

### Preserve existing CI or shell values

Use `skip_if_set = true` when a CI job or shell export should win:

```toml
[tool.pytest_env]
DATABASE_URL = { value = "postgresql://localhost/test_db", skip_if_set = true }
```

### Unset variables

Use `unset = true` when a variable must be removed instead of set to an empty string:

```toml
[tool.pytest_env]
HTTP_PROXY = { unset = true }
```

## Load Variables From `.env` Files

Use `env_files` when you have many variables or want to share the same file with local tooling:

```toml
[tool.pytest_env]
env_files = [".env", ".env.test"]
```

Example `.env` file:

```dotenv
DATABASE_URL=postgres://localhost/mydb
export SECRET_KEY='my-secret-key'
DEBUG="true"
API_KEY=${FALLBACK_KEY:-default_key}
```

Runtime overrides:

```bash
pytest --envfile .env.local
pytest --envfile +.env.override
```

Behavior that matters:

- Configured `env_files` are loaded before inline variables, so inline values override `.env` values.
- Missing files listed in config are skipped silently.
- A CLI `--envfile` path must exist; missing files raise `FileNotFoundError`.
- Paths are resolved relative to the project root.

## INI-Style Configuration

INI syntax is still supported and is useful for simple `KEY=VALUE` pairs:

```ini
# pytest.ini
[pytest]
env =
    HOME=~/tmp
    RUN_ENV=test
    D:CONDITIONAL=value
    R:RAW_VALUE={USER}
    U:REMOVED_VAR
```

You can also place the same `env` list in `pyproject.toml` with pytest's native TOML config:

```toml
[tool.pytest]
env = [
  "HOME=~/tmp",
  "RUN_ENV=test",
]
```

Flag behavior in INI mode:

- `D:` only sets the variable if it is missing
- `R:` disables `{VAR}` expansion
- `U:` unsets the variable

Important difference: INI mode expands `{VAR}` placeholders by default. Native TOML does not unless `transform = true`.

## Precedence And Discovery

When the same variable appears in multiple places, the maintainer README documents this order:

1. Inline variables in config files
2. Variables loaded from `.env` files
3. Variables already present in the environment, when preserved by `skip_if_set = true` or `D:`

Format and file selection also matter:

- Native TOML config takes precedence over INI-style `env` config.
- For native TOML, the plugin checks `pytest.toml`, then `.pytest.toml`, then `pyproject.toml`, and stops at the first file containing a `pytest_env` section.
- The plugin walks upward from pytest's resolved configuration directory, so a subdirectory config can intentionally override parent settings for an integration-test subtree.

## Common Pitfalls

- Do not try to `import pytest_env` in application code. This is a pytest plugin, not an app runtime dependency.
- In TOML mode, `{HOME}` or `{USER}` placeholders do nothing unless `transform = true`.
- In INI mode, expansion is already on; add `R:` if you need a literal `{VAR}` string.
- `skip_if_set = true` and `D:` do not make the variable highest priority. They only preserve an existing value if one is already present.
- `unset = true` and `U:` remove the variable completely. They are not equivalent to setting it to `""`.
- If nothing seems to happen, check which config file pytest selected and run with `--pytest-env-verbose`.
- If your repository has nested pytest configs, remember that the first matching native TOML file wins.

## Version-Sensitive Notes

- PyPI currently shows `1.5.0` as the latest `pytest-env` release, published on February 17, 2026.
- `pytest.toml` and native `[tool.pytest]` config are pytest 9 features. Older pytest docs often use `[tool.pytest.ini_options]`; that is a different configuration path.
- The current maintainer metadata documents `pytest>=9.0.2` and `python-dotenv>=1.2.1`. If your project is pinned to older pytest versions, verify compatibility before copying 1.5.0 examples directly.
- The plugin documentation is primarily the GitHub README rather than a separate docs site, so README updates can materially change recommended config shapes.

## Official Sources

- Maintainer docs and reference: `https://github.com/pytest-dev/pytest-env`
- PyPI package page: `https://pypi.org/project/pytest-env/`
- PyPI metadata JSON: `https://pypi.org/pypi/pytest-env/json`
- Pytest configuration reference for pytest 9 TOML behavior: `https://docs.pytest.org/en/latest/customize.html`

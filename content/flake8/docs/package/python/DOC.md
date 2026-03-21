---
name: package
description: "flake8 command-line linter for Python style, pyflakes, pycodestyle, and McCabe checks"
metadata:
  languages: "python"
  versions: "7.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flake8,python,linting,pyflakes,pycodestyle,mccabe,ci"
---

# flake8 7.3.0 Python Package Guide

## Golden Rule

Run `flake8` from the same virtualenv as the project, invoke it as `python -m flake8` in automation when you need interpreter certainty, and keep configuration in one of the supported INI-style files: `.flake8`, `setup.cfg`, or `tox.ini`.

`flake8` is primarily a CLI tool. There is no auth flow to configure.

## What Flake8 Checks

Flake8 combines several check families:

- `pyflakes` findings such as unused imports and undefined names
- `pycodestyle` findings such as whitespace and formatting rules
- `mccabe` complexity checks when `max-complexity` is configured

Violation code families you will usually see:

- `F...`: Pyflakes
- `E...` and `W...`: pycodestyle
- `C901`: McCabe complexity

## Install

Pin the version your project expects:

```bash
python -m pip install "flake8==7.3.0"
```

Common project-manager equivalents:

```bash
uv add --dev "flake8==7.3.0"
poetry add --group dev "flake8==7.3.0"
```

Verify what is installed, including active plugins:

```bash
python -m flake8 --version
```

## Core Usage

Lint the current project:

```bash
python -m flake8 .
```

Lint specific source roots:

```bash
python -m flake8 src tests
```

Add source context and summary counts when debugging CI failures:

```bash
python -m flake8 src tests --show-source --statistics
```

Treat the command as a non-zero exit gate in CI:

```bash
python -m flake8 src tests
```

If your tooling shells into multiple Python environments, prefer `python -m flake8` over a bare `flake8` executable so the interpreter and installed plugins match the current environment.

## Configuration

Flake8 reads configuration from:

- `.flake8`
- `setup.cfg`
- `tox.ini`

Use a single project-level config unless you have a strong reason to layer more.

Minimal example:

```ini
[flake8]
max-line-length = 88
max-complexity = 10
exclude =
    .git,
    __pycache__,
    .venv,
    build,
    dist
per-file-ignores =
    __init__.py:F401
```

Useful knobs:

- `exclude`: skip generated, vendored, or environment directories
- `per-file-ignores`: allow targeted exceptions such as package re-export files
- `max-line-length`: set a project-wide line length
- `max-complexity`: enable McCabe complexity reporting
- `select`, `ignore`, `extend-select`, `extend-ignore`: tune which code families are enforced

Configuration parsing pitfalls:

- Keep comments on their own lines. Flake8 does not support inline comments for option values.
- Use INI syntax, not TOML. Flake8 7.3.0 does not read `pyproject.toml` directly.
- If you pass file paths explicitly on the command line, excludes behave differently than a pure directory walk. Re-check CI and pre-commit wiring if excluded files still get linted.

## Common Workflows

### Lint a single file while iterating

```bash
python -m flake8 path/to/module.py --show-source
```

### Gate complexity on selected code paths

```ini
[flake8]
max-complexity = 12
```

Without `max-complexity`, you will not get `C901` complexity findings.

### Keep `__init__.py` re-exports intentional

```ini
[flake8]
per-file-ignores =
    package/__init__.py:F401
```

Prefer targeted per-file ignores over globally ignoring the code.

## Plugins

Flake8 loads plugins installed in the same environment through entry points. In practice:

1. Install the plugin package into the same environment as `flake8`
2. Run `python -m flake8 --version`
3. Confirm the plugin appears in the version output before relying on its codes in CI

If a repo config references plugin-specific codes but the plugin is missing, local results and CI results will drift.

## Python API

Flake8 exposes a Python API under `flake8.api.legacy`, but it is intentionally the legacy API surface. For editor integration, CI, and agent automation, prefer the CLI unless you have a specific need to call the API directly.

Minimal example:

```python
from flake8.api import legacy as flake8

style_guide = flake8.get_style_guide(ignore=["E24", "W504"])
report = style_guide.check_files(["src", "tests"])

if report.total_errors:
    raise SystemExit(1)
```

Use this sparingly. Shelling out to `python -m flake8` is usually easier to reason about and keeps plugin loading behavior aligned with normal command-line runs.

## Common Pitfalls

- Do not assume `pyproject.toml` support. For Flake8 7.3.0, the documented config files are still `.flake8`, `setup.cfg`, and `tox.ini`.
- Do not rely on a globally installed `flake8` if the project also needs plugins. Install both `flake8` and plugins in the project environment.
- Do not expect inline comments inside multi-line config values to work.
- Do not enable `ignore = ...` broadly when `per-file-ignores` or `extend-ignore` would keep the rule set tighter.
- Do not expect complexity warnings unless `max-complexity` is set.
- If older tooling still passes doctest-related Flake8 flags, re-check it against 7.x; some old doctest CLI options were removed in 7.0.0.

## Version-Sensitive Notes

For `7.3.0` specifically:

- PyPI requires Python `>=3.9`.
- The 7.3.0 release notes mention support for Python `3.14`.
- The 7.3.0 release also updated bundled checker dependencies to `pycodestyle >= 2.14.0, < 2.15.0` and `Pyflakes >= 3.4.0, < 3.5.0`.

Relevant 7.x migration note:

- Flake8 7.0.0 dropped Python 3.8 support and removed older doctest-related options such as `--include-in-doctest` and `--exclude-from-doctest`.

## Official Source URLs

- Docs root: `https://flake8.pycqa.org/en/7.3.0/`
- User guide index: `https://flake8.pycqa.org/en/7.3.0/user/index.html`
- Invoking Flake8: `https://flake8.pycqa.org/en/7.3.0/user/invocation.html`
- Configuration: `https://flake8.pycqa.org/en/7.3.0/user/configuration.html`
- Python API: `https://flake8.pycqa.org/en/7.3.0/user/python-api.html`
- Release notes: `https://flake8.pycqa.org/en/7.3.0/release-notes/7.3.0.html`
- PyPI release page: `https://pypi.org/project/flake8/7.3.0/`

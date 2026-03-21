---
name: package
description: "Nox package guide for Python task automation across multiple interpreters"
metadata:
  languages: "python"
  versions: "2026.2.9"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "nox,python,testing,automation,ci,virtualenv"
---

# Nox Python Package Guide

## Golden Rule

Use `nox` for Python automation only when the project already expects a Python-defined task runner. Start from a checked-in `noxfile.py`, keep each session explicit, and prefer `session.install(...)` plus `session.run(...)` over shell-heavy wrappers.

## Install

For a user-level CLI, install with `pipx`:

```bash
pipx install nox
```

Project or CI install:

```bash
python -m pip install "nox==2026.2.9"
```

Useful extras:

```bash
python -m pip install "nox[uv]==2026.2.9"
python -m pip install "nox[pbs]==2026.2.9"
```

- `nox[uv]`: enables the `uv` venv backend if `uv` is available
- `nox[pbs]`: enables Python downloads from python-build-standalone when `uv` is not the backend

## Initialize A Project

Create a `noxfile.py` at the repo root:

```python
import nox

@nox.session
def lint(session):
    session.install("ruff")
    session.run("ruff", "check", ".")

@nox.session(python=["3.11", "3.12"])
def tests(session):
    session.install("-e", ".[test]")
    session.run("pytest", *session.posargs)
```

Basic commands:

```bash
nox -l
nox
nox -s lint
nox -s tests-3.12 -- tests/unit/test_cli.py -q
```

What happens:

- `nox -l` lists sessions, including parametrized interpreter variants like `tests-3.11`
- `nox` runs all default sessions in file order
- `-s` or `--session` targets one or more sessions
- arguments after `--` become `session.posargs`

## Core Usage

### Session model

Each `@nox.session` function receives a `session` object. The core workflow is:

1. install tools or your package into the session environment
2. run commands inside that environment
3. optionally branch on interpreter, tags, or passed arguments

```python
import nox

@nox.session
def tests(session):
    session.install("-e", ".[test]")
    session.run("pytest", *session.posargs)
```

### Multiple Python versions

```python
import nox

@nox.session(python=["3.10", "3.11", "3.12"])
def tests(session):
    session.install("-e", ".[test]")
    session.run("pytest")
```

Nox creates a separate session for each interpreter, such as `tests-3.10` and `tests-3.12`.

### Parametrized sessions

Use `@nox.parametrize` when the matrix is not just Python versions:

```python
import nox

@nox.session
@nox.parametrize("dependency", ["pydantic<2", "pydantic>=2"])
def tests(session, dependency):
    session.install("-e", ".", dependency, "pytest")
    session.run("pytest")
```

### Reusing environments locally

Nox recreates environments by default. For faster local iteration:

```bash
nox --reuse-venv=yes -s tests-3.12
nox --reuse-venv=yes --no-install -s tests-3.12
```

`--reuse-existing-virtualenvs` still works, but current docs prefer `--reuse-venv=yes|no|always|never`.

### Selecting and filtering sessions

Useful CLI patterns:

```bash
nox --session lint --session tests-3.12
nox --tags quick
nox --keywords "tests and not slow"
nox --force-python 3.12 -s lint
```

- `--tags` and `--keywords` are useful once a `noxfile.py` grows into a larger matrix

## `pyproject.toml` Integration

`nox.project` helpers are useful when the project already defines dependency groups or supported Python versions in `pyproject.toml`.

```python
import nox

PYPROJECT = nox.project.load_toml("pyproject.toml")

@nox.session
def tests(session):
    session.install(*nox.project.dependency_groups(PYPROJECT, "test"))
    session.run("pytest")
```

You can also derive session Python versions from project metadata:

```python
import nox

PYPROJECT = nox.project.load_toml("pyproject.toml")
PYTHON_VERSIONS = nox.project.python_versions(PYPROJECT)

@nox.session(python=PYTHON_VERSIONS)
def tests(session):
    session.install("-e", ".[test]")
    session.run("pytest")
```

This reduces drift between packaging metadata and CI task definitions.

## Environment And Configuration

There is no built-in auth model. `nox` just runs local commands, so any credentials come from your normal environment, `.env` loading, CI secrets, or the tools invoked inside a session.

Common Noxfile-level options:

```python
import nox

nox.options.sessions = ["lint", "tests"]
nox.options.default_venv_backend = "uv|virtualenv"
nox.options.reuse_venv = "yes"
nox.options.error_on_missing_interpreters = True
nox.options.error_on_external_run = True
```

Notes:

- command-line flags override `nox.options.*`
- `default_venv_backend` can use fallbacks such as `uv|virtualenv`
- `error_on_missing_interpreters` is useful in CI so missing Python versions fail loudly
- `error_on_external_run` tightens execution and forces explicit `external=True` for tools outside the session env

If you need to pass or block environment variables per command:

```python
@nox.session
def integration(session):
    session.install("-e", ".[test]")
    session.run("pytest", env={"API_TOKEN": session.env["API_TOKEN"]})
```

Use explicit `env={...}` when a session depends on secrets or environment-sensitive behavior.

## CI And Interpreter Management

By default, Nox can download interpreters when a requested Python is missing:

```bash
nox --download-python auto
nox --download-python never
nox --download-python always
```

Current behavior to remember:

- `auto` is the documented default
- on CI, missing interpreters are treated as errors by default when `CI` is set
- if you are not using the `uv` backend, Python downloads require the `pbs` extra

## Common Pitfalls

### Running tools outside the session environment

If a command is not installed in the session, Nox may warn and still run the external program unless you enable `--error-on-external-run`. This can hide undeclared dependencies.

Prefer:

```python
session.install("pytest")
session.run("pytest")
```

instead of assuming `pytest` already exists on the host machine.

### Reused envs with stale dependencies

`--reuse-venv=yes --no-install` is fast but only safe when dependencies did not change. If a lockfile, extras, or editable install changed, rerun without `--no-install`.

### Missing interpreters silently skipping locally

Outside CI, missing interpreters may skip sessions unless you opt into:

```bash
nox --error-on-missing-interpreters
```

This matters for multi-Python matrices where a skipped interpreter would otherwise look like success.

### `uv` backend surprises

The `uv` backend is fast, but it does not install `pip` by default. If a session or debug workflow expects `pip`, install it explicitly:

```python
session.install("pip")
```

### Overusing shell wrappers

Keep `noxfile.py` readable Python. If a session mostly shells out to a long bash script, agents lose the benefit of clear dependency installation, argument passing, and interpreter control.

## Version-Sensitive Notes For `2026.2.9`

- PyPI and the stable docs both currently resolve to `2026.2.9`.
- The stable docs root is `nox.thea.codes`; PyPI metadata also points there, while some older project text still references `nox.readthedocs.io`.
- Current docs prefer `--reuse-venv=...` and `nox.options.reuse_venv` over the older `reuse_existing_virtualenvs` naming.
- Current docs treat `--download-python auto` as the default, with `uv` or `pbs` determining how interpreter downloads happen.
- The 2026.2.9 changelog notes support for `uv 0.10`.

## Official Sources

- Stable docs: `https://nox.thea.codes/en/stable/`
- Configuration and API: `https://nox.thea.codes/en/stable/config.html`
- Command-line usage: `https://nox.thea.codes/en/stable/usage.html`
- Changelog: `https://nox.thea.codes/en/latest/CHANGELOG.html`
- PyPI: `https://pypi.org/project/nox/`

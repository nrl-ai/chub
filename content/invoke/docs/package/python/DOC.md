---
name: package
description: "Invoke Python task execution library for defining CLI tasks and running local shell commands"
metadata:
  languages: "python"
  versions: "2.2.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "invoke,python,cli,tasks,automation,shell"
---

# Invoke Python Package Guide

## Golden Rule

Use `invoke` as a Python-native task runner: define tasks with `@task`, let `inv` discover the collection, and run subprocesses through the provided `Context`. If a task shells out, prefer `c.run(...)` over hand-rolled `subprocess` code unless you need behavior Invoke does not expose.

## Install

Pin the version your project expects:

```bash
python -m pip install "invoke==2.2.1"
```

Common alternatives:

```bash
uv add "invoke==2.2.1"
poetry add "invoke==2.2.1"
```

If you only need the CLI for local automation, a standard virtualenv install is enough:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "invoke==2.2.1"
```

## Initialize A Task Collection

Invoke looks for a task collection starting from the current working directory and walking upward. The usual entry point is a `tasks.py` file.

Minimal example:

```python
from invoke import task

@task
def test(c):
    c.run("pytest -q")

@task
def lint(c):
    c.run("ruff check .")
```

Useful first commands:

```bash
inv --list
inv test
inv lint
```

If `inv --list` shows no tasks, check that:

- you are running from the intended project directory
- the collection file is named `tasks.py` or explicitly configured
- the module imports cleanly without import-time errors

## Define Parameterized Tasks

Task arguments become CLI flags automatically. Keep the first parameter as the Invoke context object.

```python
from invoke import task

@task(help={"name": "Person or environment name"})
def greet(c, name="world", loud=False):
    message = f"hello, {name}"
    if loud:
        message = message.upper()
    print(message)
```

Run it like this:

```bash
inv greet
inv greet --name codex
inv greet --name staging --loud
```

Practical notes:

- Write task docstrings and `help={...}` metadata if you want `inv --help <task>` to be useful.
- Boolean parameters become flags; use explicit parameter names instead of overloaded positional parsing.
- Keep task bodies small and compose shared logic in normal Python functions.

## Run Shell Commands From Tasks

`Context.run` is the core API for local command execution. By default, a non-zero exit code raises `UnexpectedExit`.

```python
from invoke import task

@task
def check(c):
    result = c.run("pytest -q", hide=True, warn=True)
    print("ok:", result.ok)
    print("exit:", result.exited)
    print(result.stdout)
```

Common options agents usually need:

- `warn=True`: return a `Result` instead of raising on non-zero exit
- `hide=True`: capture output instead of echoing directly
- `pty=True`: allocate a pseudo-terminal for interactive or color-sensitive commands
- `env={...}`: add or override subprocess environment variables

Example with environment overrides:

```python
from invoke import task

@task
def integration(c):
    c.run(
        "pytest tests/integration -q",
        env={"APP_ENV": "test", "CHUB_TELEMETRY": "0"},
    )
```

Use `pty=True` for commands that behave differently without a TTY:

```python
from invoke import task

@task
def deploy(c):
    c.run("python manage.py migrate", pty=True)
```

## Organize Larger Collections

When `tasks.py` grows, move tasks into modules and expose a `Collection`.

```python
from invoke import Collection, task

@task
def test(c):
    c.run("pytest -q")

@task
def docs(c):
    c.run("mkdocs build")

ns = Collection(test, docs)
```

This keeps your task surface stable while letting you split implementation details across files.

## Configuration And Environment

Invoke supports configuration files in multiple formats, including:

- `invoke.yaml`
- `invoke.yml`
- `invoke.json`
- `invoke.py`
- `pyproject.toml` under `[tool.invoke]` in Invoke `2.1+`

`pyproject.toml` example:

```toml
[tool.invoke.run]
echo = true
warn = true

[tool.invoke.tasks]
dedupe = true
```

Environment variables can override config values with the `INVOKE_` prefix. For example:

```bash
export INVOKE_RUN_ECHO=1
export INVOKE_RUN_WARN=1
```

Use config for task-runner behavior, not service credentials. Invoke itself does not manage API authentication; if a task calls tools like `aws`, `gcloud`, or `docker`, those tools still need their own credentials and environment setup.

## Sudo, Prompts, And Interactive Commands

Invoke also exposes `c.sudo(...)` for privilege escalation. In non-interactive automation, password prompts are a common failure point. If a command expects interactive input, prefer one of these approaches:

- run in an environment where passwordless sudo is already configured for the needed command
- use `pty=True` when the target program requires a terminal
- use Invoke watchers when you must respond to interactive prompts programmatically

Do not hard-code real credentials into task files or checked-in config.

## Common Pitfalls

- The first argument to a task must be the context object. `@task def build(): ...` is wrong; use `def build(c): ...`.
- `c.run(...)` raises on command failure unless you set `warn=True`.
- `pty=True` changes I/O behavior. It is often necessary for interactive tools, but it can make output capture different from non-PTY execution.
- Task discovery depends on the working directory. Running `inv` from the wrong directory is a frequent reason for "No idea what task" style errors.
- `pyproject.toml` configuration only works in Invoke `2.1+`. Older 2.0.x setups need one of the legacy config file formats.
- Invoke is for local or subprocess-driven automation. It is not Fabric; do not assume remote SSH execution APIs exist just because older blog posts mix the projects together.
- Avoid putting secrets in `invoke.yaml` or `pyproject.toml`. Pass them via the environment or a secret manager and let downstream tools read them.

## Version-Sensitive Notes

- The version used here `2.2.1` matches the current PyPI release as of `2026-03-12`.
- Invoke `2.0.0` dropped support for Python versions earlier than `3.6`. If you are stuck on Python `2.7` or `3.5`, you need the older `1.x` line instead of `2.x`.
- Invoke `2.1.0` added support for reading configuration from `pyproject.toml`, which matters if you are migrating from ad hoc `invoke.yaml` files to modern project metadata.
- Invoke `2.2.1` includes a fix for Python `3.14` compatibility around deprecated argument-inspection APIs. That matters if tasks or task loading fail on very new Python runtimes.

## Official Sources

- Project site: `https://www.pyinvoke.org/`
- Documentation root: `https://docs.pyinvoke.org/en/latest/`
- Getting started: `https://docs.pyinvoke.org/en/latest/getting-started.html`
- Configuration: `https://docs.pyinvoke.org/en/latest/concepts/configuration.html`
- Command execution: `https://docs.pyinvoke.org/en/latest/concepts/invoking-tasks.html`
- Changelog: `https://www.pyinvoke.org/changelog.html`
- PyPI: `https://pypi.org/project/invoke/`

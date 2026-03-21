---
name: package
description: "pip-tools package guide for Python projects using pip-compile and pip-sync to produce reproducible dependency locks"
metadata:
  languages: "python"
  versions: "7.5.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pip-tools,pip-compile,pip-sync,packaging,dependencies,python"
---

# pip-tools Python Package Guide

## Golden Rule

Use `pip-tools` as a command-line workflow, not as an application library. Treat `pip-compile` as the lockfile generator, treat `pip-sync` as the environment reconciler, and run both from the same Python version and platform you are targeting because the resolved output can vary across environments.

## Install

Install `pip-tools` into the same virtual environment you use for dependency management:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "pip-tools==7.5.3"
```

Common alternatives:

```bash
uv tool install "pip-tools==7.5.3"
pipx install "pip-tools==7.5.3"
```

If you use `uv tool install` or `pipx`, keep in mind that the tool still resolves packages for the interpreter and platform it runs under. Compile inside the target project environment when the output must match deploy-time markers or wheel availability.

## Initialize A Lockfile Workflow

### Recommended: compile from `pyproject.toml`

`pip-tools` can read dependencies directly from `pyproject.toml`:

```toml
[project]
name = "my-app"
version = "0.1.0"
dependencies = [
  "django>=5.2,<5.3",
  "httpx>=0.28,<0.29",
]

[project.optional-dependencies]
dev = [
  "pytest>=8.3,<9",
  "ruff>=0.11,<0.12",
]
```

Compile the default dependency set:

```bash
python -m piptools compile -o requirements.txt pyproject.toml
```

Compile an extra into a separate lock file:

```bash
python -m piptools compile --extra dev -o dev-requirements.txt pyproject.toml
```

### Alternative: compile from `requirements.in`

If the project does not use PEP 621 metadata yet, keep direct requirements in a small input file and compile it:

```txt
# requirements.in
django>=5.2,<5.3
httpx>=0.28,<0.29
```

```bash
python -m piptools compile requirements.in
```

The default output is `requirements.txt` next to the input file.

## Core Commands

### Generate a lock file

Basic compile:

```bash
python -m piptools compile -o requirements.txt pyproject.toml
```

Generate hashes for repeatable installs:

```bash
python -m piptools compile --generate-hashes -o requirements.txt pyproject.toml
```

Re-resolve everything from scratch:

```bash
python -m piptools compile --rebuild -o requirements.txt pyproject.toml
```

Upgrade all packages:

```bash
python -m piptools compile --upgrade -o requirements.txt pyproject.toml
```

Upgrade only one or two packages while leaving the rest pinned:

```bash
python -m piptools compile \
  --upgrade-package django \
  --upgrade-package httpx \
  -o requirements.txt \
  pyproject.toml
```

Important behavior: if an output file already exists, `pip-compile` uses its existing pins as constraints unless you explicitly ask for upgrades or delete the file.

### Sync an environment to the lock file

Apply the compiled requirements to the current environment:

```bash
python -m piptools sync requirements.txt
```

Sync multiple lock files together:

```bash
python -m piptools sync requirements.txt dev-requirements.txt
```

`pip-sync` installs missing packages, updates mismatched ones, and uninstalls packages that are not present in the compiled files. The docs note one important exception: it does not upgrade or uninstall packaging tools such as `pip`, `setuptools`, or `pip-tools` itself.

### Layer production and development requirements

Use a compiled base file as a constraint for a dev file:

```txt
# requirements.in
django>=5.2,<5.3
httpx>=0.28,<0.29
```

```txt
# dev-requirements.in
-c requirements.txt
pytest>=8.3,<9
ruff>=0.11,<0.12
```

```bash
python -m piptools compile requirements.in
python -m piptools compile dev-requirements.in
python -m piptools sync requirements.txt dev-requirements.txt
```

This keeps development-only tools from changing the production dependency set.

## Configuration And Package Indexes

Both `pip-compile` and `pip-sync` can read defaults from `.pip-tools.toml` or `pyproject.toml`. The docs show these lookup rules:

- `.pip-tools.toml` is checked first
- `pyproject.toml` is checked next
- command-line flags still override config values

Example `pyproject.toml` configuration:

```toml
[tool.pip-tools]
generate_hashes = true
allow_unsafe = false

[tool.pip-tools.compile]
resolver = "backtracking"
dry_run = false
extra_index_url = ["https://pypi.mycompany.example/simple"]
```

Use list values in TOML for options that can be repeated on the CLI, such as `extra_index_url`, `find_links`, or `upgrade_package`.

### Private indexes and credentials

`pip-tools` follows pip's index configuration model. In practice, prefer one of these:

- `PIP_INDEX_URL` and `PIP_EXTRA_INDEX_URL` in the environment
- pip config files managed by `pip config`
- a secrets manager that injects index credentials at runtime

Example:

```bash
export PIP_INDEX_URL="https://__token__:${PYPI_TOKEN}@pypi.mycompany.example/simple"
python -m piptools compile --generate-hashes -o requirements.txt pyproject.toml
```

Be careful with `--emit-index-url` or config that writes index URLs into generated output. If your index URL contains credentials, do not commit those lock files without sanitizing the emitted URL.

## Common Pitfalls

- `pip-tools` lock files are environment-sensitive. Compile separately for different Python versions, operating systems, or platforms when markers and wheel availability differ.
- A stale `requirements.txt` can silently constrain future resolves. Use `--upgrade`, `--upgrade-package`, or `--rebuild` when you actually want new versions.
- `pip-sync` is for compiled outputs, not loose input files. Syncing from hand-edited requirement sets defeats the point of the workflow.
- Compiling with hashes against private or extra indexes is exactly where misconfigured index URLs surface. Validate the generated file and a fresh install path, not just the compile step.
- `piptools` has private API documentation, which is a signal that internals are not a stable integration surface. For automation, prefer shelling out to `pip-compile` and `pip-sync` instead of importing `piptools.*`.
- Agents often forget that `pip-sync` mutates the current environment aggressively. Do not point it at a shared global interpreter.

## Version-Sensitive Notes For 7.5.3

- PyPI lists `7.5.3` as the current release, and the versioned stable docs match that release.
- `7.5.3` requires Python `>=3.9`. The changelog records Python 3.8 support as dropped in this release.
- The `7.5.3` changelog also calls out compatibility fixes for `pip 26.0`.
- If you use `--generate-hashes` together with private or extra package indexes, `7.5.3` matters: the changelog notes a fix so index URL options are preserved while looking up hashes.
- The changelog also notes normalization for `--unsafe-package`; use canonical package names in automation and avoid depending on older casing behavior.

## Official Sources

- Stable docs root: `https://pip-tools.readthedocs.io/en/stable/`
- `pip-compile` reference: `https://pip-tools.readthedocs.io/en/stable/cli/pip-compile/`
- `pip-sync` reference: `https://pip-tools.readthedocs.io/en/stable/cli/pip-sync/`
- Changelog: `https://pip-tools.readthedocs.io/en/stable/changelog/`
- PyPI release page: `https://pypi.org/project/pip-tools/7.5.3/`

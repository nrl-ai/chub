---
name: package
description: "PDM package guide for Python projects using pyproject.toml, lockfiles, scripts, and publishing workflows"
metadata:
  languages: "python"
  versions: "2.26.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pdm,python,packaging,dependency-management,pyproject,lockfile,publishing"
---

# PDM Python Package Guide

## Golden Rule

Use `pdm` as a project tool, not as a runtime dependency of your application. Prefer the default virtualenv workflow unless you intentionally want PEP 582, and treat `pdm.lock` as the source of truth for installs. Commit `pdm.lock` for applications and most service repos; decide deliberately for pure libraries if you want CI to mimic end-user installs instead.

## Install PDM

Prefer installing PDM as a user-level tool instead of adding it to a project virtualenv.

Recommended isolated installs:

```bash
uv tool install pdm
pipx install pdm
```

Official installer scripts:

```bash
curl -sSL https://pdm-project.org/install.sh | bash
curl -sSL https://pdm-project.org/install.sh | bash -s -- -v 2.26.6
```

If you need a plain Python install:

```bash
python -m pip install --user "pdm==2.26.6"
```

Check the installed version:

```bash
pdm --version
```

Update an existing installation:

```bash
pdm self update
```

## Initialize A Project

For a new project, use `pdm new`:

```bash
pdm new my-project
cd my-project
```

For an existing directory, generate `pyproject.toml` with `pdm init`:

```bash
cd existing-project
pdm init
```

Important setup details:

- PDM stores the selected interpreter path in `.pdm-python`.
- `PDM_PYTHON=/path/to/python` overrides the saved interpreter.
- If `.python-version` exists, PDM can use that version hint.
- In `2.26.6`, virtualenv mode is the default. On first install for a new project, PDM usually creates `.venv` unless you configure otherwise.

Pick the project type carefully:

- Application: usually no build backend is needed.
- Library: needs package metadata and a build backend because you expect to build and publish wheels/sdists.

For library behavior, PDM uses `distribution = true` under `[tool.pdm]`.

## Core Workflow

### Add And Manage Dependencies

Add production dependencies:

```bash
pdm add requests httpx
```

Add an optional dependency group that becomes `[project.optional-dependencies]`:

```bash
pdm add -G postgres psycopg[binary]
```

Add development dependencies under `[dependency-groups]`:

```bash
pdm add -dG test pytest pytest-cov
pdm add -dG lint ruff mypy
```

Remove or update dependencies:

```bash
pdm remove httpx
pdm update requests
pdm outdated
```

Notes:

- `pdm add requests` saves a minimum version specifier by default, such as `>=x.y.z`.
- Use `--save-compatible` or `--save-exact` when you need tighter constraints.
- Editable installs are allowed only in development groups:

```bash
pdm add -d -e ./shared-lib
```

### Install And Sync From The Lockfile

```bash
pdm install
pdm sync
pdm sync --clean
```

Use group selection when needed:

```bash
pdm install --prod
pdm install -G postgres
pdm install -G test
pdm install -G:all
```

Key behavior:

- `pdm.lock` is the install source of truth.
- `pdm install` and `pdm add` automatically create or update `pdm.lock`.
- `pdm sync --clean` removes packages that are not in the selected locked groups.

Check or refresh the lockfile:

```bash
pdm lock --check
pdm lock --refresh
```

### Minimal `pyproject.toml` Shape

Application-oriented example:

```toml
[project]
requires-python = ">=3.12"
dependencies = [
  "fastapi>=0.118.0",
  "uvicorn>=0.37.0",
]

[dependency-groups]
test = ["pytest>=8.3.0"]
lint = ["ruff>=0.11.0"]

[tool.pdm]
distribution = false
```

Library-oriented example:

```toml
[project]
name = "example-lib"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = ["httpx>=0.28.0"]

[build-system]
requires = ["pdm-backend"]
build-backend = "pdm.backend"

[tool.pdm]
distribution = true
```

## Virtual Environments And Python Selection

Default behavior in current docs:

- Virtualenvs are preferred over PEP 582 because ecosystem tooling and IDE support are better.
- First-time `pdm install` on a new project usually creates `.venv`.
- `pdm use` can switch interpreters, and with `python.use_venv = true` it will create a virtualenv for the chosen interpreter.

Common commands:

```bash
pdm info
pdm use 3.12
pdm venv create 3.12
pdm use -f /path/to/venv
```

Useful config:

```bash
pdm config python.use_venv true
pdm config venv.in_project true
pdm config venv.backend virtualenv
```

If you disable in-project environments, named virtualenvs go under `venv.location` instead of `.venv`.

## Lockfiles And Reproducibility

PDM creates cross-platform lockfiles by default. That is usually what you want for shared repos, but it can cause trouble when packages publish incomplete wheel sets.

Useful commands:

```bash
pdm lock
pdm lock --strategy no_cross_platform
pdm lock --exclude-newer 2026-01-01
```

Relevant current behavior:

- `inherit_metadata` is enabled by default and speeds up installs.
- Since `2.25.0`, PDM also supports the experimental `pylock.toml` format from PEP 751.
- In `2.26.6`, the default lock format is still `pdm.lock`, not `pylock.toml`.

To switch formats explicitly:

```bash
pdm config lock.format pylock
```

## Private Indexes, Auth, And Configuration

Use `[[tool.pdm.source]]` when the index definition should be shared with the project:

```toml
[[tool.pdm.source]]
name = "private"
url = "https://private.example.com/simple"
verify_ssl = true
```

Use `pdm config` for machine-local index settings:

```bash
pdm config pypi.url "https://test.pypi.org/simple"
pdm config pypi.extra.url "https://extra.example.com/simple"
```

Important distinction:

- Indexes (`pypi.*`, `tool.pdm.source`) are for resolving and locking.
- Repositories (`repository.*`) are for publishing.

For publish credentials:

```bash
pdm config repository.pypi.username "__token__"
pdm config repository.pypi.password "pypi-..."
```

Or use environment variables:

```bash
export PDM_PUBLISH_USERNAME="__token__"
export PDM_PUBLISH_PASSWORD="pypi-..."
```

For source credentials, prefer environment-variable expansion instead of committing secrets:

```toml
[[tool.pdm.source]]
name = "private"
url = "https://${PRIVATE_PYPI_USERNAME}:${PRIVATE_PYPI_PASSWORD}@private.example.com/simple"
```

When keyring support is available, PDM can store index and repository passwords there instead of writing them into config files.

If teams must use only the indexes committed in the repo:

```bash
pdm config pypi.ignore_stored_index true
```

If source priority matters:

```toml
[tool.pdm.resolution]
respect-source-order = true
```

## Scripts And Task Running

PDM can act as a lightweight task runner through `[tool.pdm.scripts]`.

Example:

```toml
[tool.pdm.scripts]
test = "pytest"
serve.cmd = "uvicorn app.main:app --reload"
serve.env_file = ".env"
check.composite = ["test", "ruff check ."]
```

Run scripts with:

```bash
pdm run test
pdm run serve
pdm run check
pdm run --list
```

Practical notes:

- `env_file` loads dotenv-style variables for the script.
- `composite` runs multiple named tasks in order.
- `pdm run` is usually the right entrypoint for local dev commands because it uses the project environment and dependency graph.

## Build And Publish

For libraries:

```bash
pdm build
pdm publish --no-build
```

Or build and upload in one step:

```bash
pdm publish
```

Current docs note:

- `pdm publish` builds both wheel and sdist before upload.
- PyPI uploads use `__token__` as the username and the API token as the password.
- Trusted publishing is supported for CI and is preferable to storing long-lived PyPI tokens in GitHub Actions or GitLab CI.

## Version Control

Usually commit these files:

- `pyproject.toml`
- `pdm.lock` for applications and most deployable repos
- `pdm.toml` when you want to share project-wide PDM config

Sometimes skip `pdm.lock` for pure libraries if your CI intentionally tests resolution against current ecosystem packages instead of a pinned graph.

Do not commit:

- `.pdm-python`

## Common Pitfalls

- Do not treat `pdm` like an importable library. It is primarily a CLI tool for managing the project.
- Do not share one existing virtualenv across multiple projects and then run `pdm sync --clean`; PDM can remove packages that are not declared in the current project.
- Do not confuse optional dependency groups with development groups. Optional groups go into package metadata; dev groups do not.
- Editable installs only work in dev dependency groups.
- `--prod` cannot be combined with dev groups.
- Do not put private credentials directly in committed `pyproject.toml`; use environment variables, local config, or keyring.
- Publishing repositories and install indexes are separate config systems. `repository.*` does not affect locking, and `pypi.*` does not affect publishing.
- Set `requires-python` carefully. PDM resolves against the whole declared range, so an overly broad range can produce `ResolutionImpossible` failures.

## Version-Sensitive Notes For `2.26.6`

- PyPI lists `pdm 2.26.6` as the latest release on January 22, 2026.
- `2.26.6` is a patch release that fixes compatibility with `packaging==26.0`; if you see version-comparison issues on older PDM releases, upgrade before debugging the resolver.
- `pdm new` is relatively recent, added in `2.24.0`. Older guides may tell you to use `pdm init` for everything.
- Support for `pylock.toml` was added in `2.25.0`, but `pdm.lock` is still the default format in `2.26.6`.
- PDM itself requires Python `>=3.9` to run. Current docs also note that application projects without a build backend can still target older Python versions, but library builds remain constrained by the selected backend. The default `pdm-backend` supports Python `>=3.7`.

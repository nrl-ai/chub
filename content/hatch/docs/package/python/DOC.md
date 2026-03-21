---
name: package
description: "Hatch Python project manager for environments, builds, publishing, and versioning"
metadata:
  languages: "python"
  versions: "1.16.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "hatch,python,packaging,pyproject,build,publish,virtualenv"
---

# Hatch Python Package Guide

## Golden Rule

Use `hatch` as a project-level CLI for Python packaging workflows, but use `hatchling` as the build backend in `pyproject.toml`. Most project configuration lives under `tool.hatch` in `pyproject.toml`; if you also create `hatch.toml`, its top-level settings override the `pyproject.toml` equivalents.

## Install

Pin the CLI version if your project or automation depends on specific Hatch behavior:

```bash
python -m pip install "hatch==1.16.5"
```

Common tool-install alternatives:

```bash
pipx install "hatch==1.16.5"
uv tool install "hatch==1.16.5"
```

Verify the installed CLI:

```bash
hatch --version
```

The official install page for the `1.16` docs stream still shows `1.16.4` in its sample verification output. For this guide, treat PyPI `1.16.5` as the package version of record.

## Initialize Or Adopt A Project

Create a new project from Hatch's template:

```bash
hatch new awesome-app
cd awesome-app
```

Initialize Hatch metadata in an existing project instead:

```bash
hatch new --init
```

For package builds, make sure your `pyproject.toml` uses Hatchling:

```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
```

Minimal project setup with dynamic versioning and a default dev environment:

```toml
[project]
name = "awesome-app"
dynamic = ["version"]
dependencies = [
  "httpx>=0.28",
]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.version]
path = "src/awesome_app/__about__.py"

[tool.hatch.envs.default]
python = "3.12"
dependencies = [
  "pytest",
  "ruff",
]

[tool.hatch.envs.default.scripts]
test = "pytest"
lint = "ruff check ."
fmt = "ruff format ."
```

Version file example:

```python
# src/awesome_app/__about__.py
__version__ = "0.1.0"
```

## Core Usage

### Work with environments

If you define no environments, Hatch automatically provides a `default` environment. Create or refresh environments explicitly when you want deterministic setup in CI or local tooling:

```bash
hatch env create
hatch env show
```

Run commands inside the default environment:

```bash
hatch run python -V
hatch run test
hatch run lint
```

Target a named environment when you have more than one:

```bash
hatch run lint:check
hatch run test.py311:pytest
```

Open a shell in the default environment:

```bash
hatch shell
```

### Use tool-only or matrix environments

Detached environments are useful for linting, docs, or other tooling that should not install your package:

```toml
[tool.hatch.envs.lint]
detached = true
dependencies = [
  "ruff",
]

[tool.hatch.envs.lint.scripts]
check = "ruff check ."
```

If you want an environment to exist without installing the project itself, set:

```toml
skip-install = true
```

For multi-version testing, define a matrix:

```toml
[tool.hatch.envs.test]
dependencies = ["pytest"]

[[tool.hatch.envs.test.matrix]]
python = ["3.10", "3.11", "3.12"]
```

Then run a specific matrix environment such as `hatch run test.py311:pytest`.

### Build artifacts

Build both wheel and source distribution:

```bash
hatch build
```

Build only one target:

```bash
hatch build -t wheel
hatch build -t sdist
```

Artifacts are written to `dist/` by default.

### Manage versions

Show the current project version:

```bash
hatch version
```

Bump semver-style versions:

```bash
hatch version patch
hatch version minor
hatch version major
```

Set an explicit version:

```bash
hatch version 1.2.3
```

By default, Hatch's code version source expects a `__version__ = "..."` assignment. If your version lives elsewhere or uses a different format, configure a different source or regex under `tool.hatch.version`.

## Config And Auth

### Where configuration lives

Preferred project config location:

- `pyproject.toml` under `tool.hatch`

Optional override location:

- `hatch.toml`

`hatch.toml` uses top-level keys rather than the `tool.hatch` prefix and takes precedence over equivalent `pyproject.toml` settings. Keep one source of truth when possible; mixing both files is a common cause of confusion.

### Choose the environment installer

Hatch supports `pip` and `uv` for environment installation. To make `uv` the project default:

```toml
[tool.hatch.envs.default]
installer = "uv"
```

Use this only when your team machines or CI images have a compatible `uv` available.

### Configure publishing targets

Repository configuration belongs in Hatch's user config, not in committed project metadata. Example:

```toml
# ~/.config/hatch/config.toml
[publish.index.repos.test]
url = "https://test.pypi.org/legacy/"
```

Publish to TestPyPI with environment variables:

```bash
export HATCH_INDEX_REPO=test
export HATCH_INDEX_USER=__token__
export HATCH_INDEX_AUTH=pypi-xxxxxxxxxxxxxxxx

hatch build
hatch publish
```

Useful publishing flags and environment variables:

- `--repo` / `HATCH_INDEX_REPO`
- `--user` / `HATCH_INDEX_USER`
- `--auth` / `HATCH_INDEX_AUTH`
- `--ca-cert` / `HATCH_INDEX_CA_CERT`
- `--client-cert` / `HATCH_INDEX_CLIENT_CERT`
- `--client-key` / `HATCH_INDEX_CLIENT_KEY`

`hatch publish` uploads built `.whl` and `.tar.gz` artifacts from `dist/` by default. Build first so the upload step is explicit and reproducible.

## Common Pitfalls

- Installing `hatch` does not replace the need for `hatchling` in `[build-system]`. The CLI package and the build backend serve different roles.
- If both `pyproject.toml` and `hatch.toml` define the same Hatch settings, `hatch.toml` wins. This can make local behavior differ from CI unexpectedly.
- `hatch version` only works cleanly when the configured version source matches the file layout in your repository.
- `detached = true` and `skip-install = true` are useful for tooling environments, but they also mean your package code is not installed there. Do not run package-import tests in those envs unless that is intentional.
- `hatch publish` does not infer alternate repositories unless you pass `--repo` or set `HATCH_INDEX_REPO`.
- Matrix environment names are generated from the matrix values. Check `hatch env show` before hard-coding names in scripts or CI.

## Version-Sensitive Notes For 1.16.x

- The official docs are versioned by `MAJOR.MINOR` rather than exact patch number. For `hatch 1.16.5`, the matching docs family is `https://hatch.pypa.io/1.16/`, while `latest` currently points at the same `1.16` stream.
- The `1.16.0` release added workspaces, dependency groups, and SBOM support, and dropped Python 3.9 support. If you are upgrading from `1.15.x` or earlier, confirm your Python baseline and project layout assumptions before copying old config.
- The official install page currently includes stale `1.16.4` verification text even though PyPI lists `1.16.5`. Prefer PyPI for the exact package version and the docs site for behavior and configuration.

## Official Links

- Docs root: https://hatch.pypa.io/latest/
- Versioned docs: https://hatch.pypa.io/1.16/
- Install guide: https://hatch.pypa.io/1.16/install/
- Environment overview: https://hatch.pypa.io/1.16/config/environment/overview/
- Build configuration: https://hatch.pypa.io/1.16/config/build/
- Versioning: https://hatch.pypa.io/1.16/version/
- Publishing: https://hatch.pypa.io/1.16/publish/
- PyPI package: https://pypi.org/project/hatch/1.16.5/

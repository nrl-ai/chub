---
name: package
description: "PyPA build frontend for creating Python sdists, wheels, and build metadata from pyproject.toml projects"
metadata:
  languages: "python"
  versions: "1.4.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,packaging,pypa,build,wheel,sdist,pyproject"
---

# build Python Package Guide

## Golden Rule

Use `build` as a local PEP 517 build frontend for projects that already define how they should be built in `pyproject.toml`. `build` creates artifacts such as source distributions and wheels; it does not manage dependencies and it does not publish releases.

## Install

Pin the package version your workflow expects:

```bash
python -m pip install "build==1.4.0"
```

Common alternatives:

```bash
uv add "build==1.4.0"
poetry add "build==1.4.0"
```

Useful extras from PyPI:

```bash
python -m pip install "build[virtualenv]==1.4.0"
python -m pip install "build[uv]==1.4.0"
```

Notes:

- `build[virtualenv]` makes `build` use `virtualenv` for isolation instead of `venv`.
- `build[uv]` is useful when you want `--installer uv` and do not already have `uv` installed separately.
- On conda-forge, the package name is `python-build`.

## Initialize A Buildable Project

`build` expects a Python project root with a valid `pyproject.toml`. Keep the backend explicit instead of relying on fallback behavior.

Minimal example:

```toml
[build-system]
requires = ["setuptools>=69", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "example-pkg"
version = "0.1.0"
description = "Example package"
requires-python = ">=3.9"
```

Project layout:

```text
example-pkg/
  pyproject.toml
  src/
    example_pkg/
      __init__.py
```

From the project root:

```bash
python -m build
```

By default, `build` creates an sdist from the source tree and then builds a wheel from that sdist into `dist/`.

## Core CLI Usage

### Build both artifacts

```bash
python -m build
```

This is the normal release check before upload. If the wheel build fails here but works with ad hoc local commands, the sdist is probably missing files that your backend needs.

### Build only one artifact

```bash
python -m build --sdist
python -m build --wheel
```

Use this when CI or a release pipeline needs only one output kind.

### Build another source directory or output directory

```bash
python -m build ../my-package --outdir /tmp/artifacts
```

### Print wheel metadata as JSON

```bash
python -m build --metadata
```

`--metadata` is for inspection only. It cannot be combined with `--sdist` or `--wheel`.

### Pass backend-specific config settings

```bash
python -m build -C--global-option=--some-flag
python -m build --config-json '{"--global-option": ["--some-flag"]}'
```

Use `--config-json` when the backend expects nested or repeated values. If a config key begins with `-`, pass it with `=` so the shell and CLI do not misread it as a top-level `build` option.

### Control isolation

```bash
python -m build --installer uv
python -m build --no-isolation
python -m build --skip-dependency-check
```

Guidance:

- Default isolated builds are safest for release automation.
- `--no-isolation` is only for environments where build dependencies are already installed and intentionally controlled.
- `--skip-dependency-check` is mainly for bootstrapping or specialized workflows; it will not fix a genuinely missing backend dependency.

## Programmatic API

Use the Python API when a tool needs to inspect requirements, prepare metadata, or build artifacts without shelling out to the CLI.

### Build a wheel with `ProjectBuilder`

```python
from pathlib import Path

from build import ProjectBuilder

project_dir = Path(".").resolve()
dist_dir = project_dir / "dist"
dist_dir.mkdir(exist_ok=True)

builder = ProjectBuilder(project_dir)
wheel_path = builder.build("wheel", dist_dir)

print(wheel_path)
```

### Check unmet build dependencies before building

```python
from build import ProjectBuilder

builder = ProjectBuilder(".")
missing = builder.check_dependencies("wheel")

if missing:
    raise RuntimeError(f"Missing build dependencies: {sorted(missing)}")
```

### Read prepared metadata

```python
from pathlib import Path

from build import ProjectBuilder

builder = ProjectBuilder(".")
metadata_dir = builder.metadata_path(Path("build-metadata"))

print(metadata_dir)
```

`metadata_path()` will use the backend metadata hook when available and otherwise fall back to building a wheel and extracting metadata from it.

### Create and reuse an isolated environment

```python
from pathlib import Path

from build import ProjectBuilder
from build.env import DefaultIsolatedEnv

project_dir = Path(".").resolve()

with DefaultIsolatedEnv(installer="pip") as env:
    builder = ProjectBuilder.from_isolated_env(env, project_dir)
    env.install(builder.build_system_requires)
    env.install(builder.get_requires_for_build("wheel"))
    wheel_path = builder.build("wheel", project_dir / "dist")

print(wheel_path)
```

This is the right level if you are writing your own packaging automation around `build`.

## Configuration And Environment Notes

- There is no service authentication layer. Configuration is local: CLI flags, backend config settings, and the build backend declared in `pyproject.toml`.
- `build` reads the project from `srcdir` or the current working directory. Run it from the project root unless you pass an explicit path.
- Prefer an explicit `[build-system]` table. If no backend is specified, `build` falls back to `setuptools.build_meta:__legacy__`, which is a compatibility path, not a modern packaging default to depend on.
- For isolated builds, `build` uses `pip` by default. Use `--installer uv` when that is part of your toolchain.
- The `pyproject-build` script is equivalent to `python -m build` and is convenient in `pipx`-style environments.

## Common Pitfalls

- `build` does not upload artifacts. Use Twine, trusted publishing, or your release platform separately.
- `build` does not resolve runtime dependencies for your app. It only installs the build requirements needed by the backend when isolation is enabled.
- A missing or invalid `[build-system]` table causes build errors. Do not treat the legacy setuptools fallback as a substitute for proper project metadata.
- The default wheel-from-sdist flow catches packaging mistakes. If files exist in your checkout but not in the sdist, the wheel step can fail or produce an incomplete artifact.
- `--metadata` cannot be combined with `--sdist` or `--wheel`.
- If backend config keys begin with `-`, pass them as `--config-setting=...` or with `-C...`; a space-separated form can be parsed as a `build` CLI option instead.
- `--no-isolation` can hide missing build requirements by leaking packages from the current environment into the build.
- Editable builds are part of the Python API surface, but they still depend on backend support for editable installs.

## Version-Sensitive Notes For 1.4.0

- `1.4.0` adds `--quiet`, `--metadata`, and support for the `UV` environment variable.
- `1.3.0` added `--config-json` and dropped Python 3.8 support, so older automation snippets may be outdated.
- The stable installation page lists verified compatibility for Python `3.9` through `3.13` plus `PyPy3`, while the PyPI classifiers already include Python `3.14`. If Python `3.14` support matters for a release pipeline, check the current changelog and live package metadata instead of assuming older blog posts are accurate.

## Official Source URLs

- Documentation: https://build.pypa.io/en/stable/
- Installation: https://build.pypa.io/en/stable/installation.html
- API documentation: https://build.pypa.io/en/stable/api.html
- Differences from other tools: https://build.pypa.io/en/stable/differences.html
- Changelog: https://build.pypa.io/en/stable/changelog.html
- PyPI registry: https://pypi.org/project/build/

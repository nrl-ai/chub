---
name: package
description: "setuptools packaging backend and build configuration guide for Python projects"
metadata:
  languages: "python"
  versions: "82.0.1"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "setuptools,python,packaging,build,pyproject,wheel,sdist"
---

# `setuptools`

Use `setuptools` as the packaging backend for building Python source distributions and wheels. For new projects, prefer declarative configuration in `pyproject.toml` with `setuptools.build_meta`; keep `setup.py` only when you truly need imperative build logic.

## Install

For local packaging work:

```bash
python -m pip install --upgrade setuptools
```

For building distributions in a project:

```bash
python -m pip install --upgrade build setuptools
python -m build
```

When `setuptools` is your build backend, declare it in `pyproject.toml` so isolated builds install the right version automatically:

```toml
[build-system]
requires = ["setuptools>=82.0.1"]
build-backend = "setuptools.build_meta"
```

## Golden Rule

- Put standard package metadata in `[project]`.
- Put `setuptools`-specific behavior in `[tool.setuptools]`.
- Prefer a `src/` layout unless you have a strong reason not to.
- Do not introduce new runtime dependencies on `pkg_resources`; use `importlib.metadata` or `importlib.resources` instead.

## Minimal Modern Setup

This is the most useful starting point for a new package:

```toml
[build-system]
requires = ["setuptools>=82.0.1"]
build-backend = "setuptools.build_meta"

[project]
name = "example-package"
version = "0.1.0"
description = "Example package"
readme = "README.md"
requires-python = ">=3.9"
dependencies = [
  "requests>=2.32",
]

[tool.setuptools]
package-dir = {"" = "src"}

[tool.setuptools.packages.find]
where = ["src"]
```

Recommended layout:

```text
pyproject.toml
README.md
src/
  example_package/
    __init__.py
    cli.py
tests/
```

Why this shape matters:

- `src/` layout reduces accidental imports from the repository root during development.
- `package-dir = {"" = "src"}` keeps package discovery aligned with that layout.
- `setuptools.build_meta` is the modern PEP 517 backend entry point.

## Core Usage

### Build an sdist and wheel

```bash
python -m build
```

This reads `pyproject.toml`, creates an isolated build environment, and asks `setuptools.build_meta` to produce `dist/*.tar.gz` and `dist/*.whl`.

### Install your project in editable mode

```bash
python -m pip install -e .
```

If you want editable installs to behave more like a regular wheel and catch missing files earlier, use strict editable mode:

```bash
python -m pip install -e . --config-settings editable_mode=strict
```

### Add a console script

Use standard project metadata for CLI entry points:

```toml
[project.scripts]
example-cli = "example_package.cli:main"
```

With:

```python
def main() -> int:
    print("hello")
    return 0
```

After installation, `example-cli` is available on the command line.

### Add plugin entry points

For plugin ecosystems, use named entry point groups:

```toml
[project.entry-points."pytest11"]
example_plugin = "example_package.pytest_plugin"
```

This is the standard way to register plugins for tools that discover entry points.

## Configuration

`setuptools` still supports configuration in `setup.py` and `setup.cfg`, but new work should center on `pyproject.toml`.

Preferred split:

- `[project]`: name, version, dependencies, optional dependencies, scripts, entry points, classifiers
- `[tool.setuptools]`: package discovery, package data, namespace package behavior, dynamic metadata helpers
- `setup.py`: only for edge cases where values must be computed in Python

If you must keep a `setup.py`, keep it minimal:

```python
from setuptools import setup

setup()
```

That lets `pyproject.toml` remain the source of truth.

## Package Discovery

Two safe patterns:

1. Use `src/` layout with discovery scoped to `src`.
2. In flat layouts, explicitly control what gets included.

For a `src/` layout:

```toml
[tool.setuptools]
package-dir = {"" = "src"}

[tool.setuptools.packages.find]
where = ["src"]
include = ["example_package*"]
```

Why this matters:

- Automatic discovery in a flat repository can pick up unintended packages.
- `src/` layout is the less error-prone default for published libraries.
- If your package name and import package differ, discovery rules need to reflect the import package, not just the distribution name.

## Package Data

If your wheel needs non-Python files, configure them explicitly. A common pattern is:

```toml
[tool.setuptools.package-data]
example_package = ["py.typed", "templates/*.html"]
```

Treat `MANIFEST.in` as sdist-focused. Do not assume that adding files there alone guarantees they land in the wheel you install from.

## No Auth Layer

`setuptools` is a local build backend, not a hosted API client. There is no package-specific authentication flow.

The practical "configuration surface" is:

- `pyproject.toml`
- optionally `setup.cfg`
- optionally `setup.py`
- environment/tooling around the build, such as `pip`, `build`, and publishing tools like `twine`

## Common Pitfalls

- Missing `[build-system]`: build isolation can fail or use a different backend path than you expect.
- Relying on flat-layout auto-discovery: extra directories can be packaged accidentally.
- Using `pkg_resources` in runtime code: it is deprecated for new usage and often slower than stdlib replacements.
- Keeping important wheel files only in `MANIFEST.in`: source archives and wheels do not include files the same way.
- Treating `setuptools` as an installer: use `pip` to install packages and `build` to create artifacts; `setuptools` is the backend underneath.

## Version-Sensitive Notes For `82.0.1`

- The target version for this doc is `82.0.1`, but PyPI had already published `82.0.2` by 2026-03-08. The upstream docs root uses `/latest/`, so examples there may reflect the newer patch release.
- `easy_install` and `package_index` were removed in `82.0.0`. Do not use legacy `easy_install` workflows or `setup.py develop`; use `pip install -e .` instead.
- Modern `setuptools` guidance prefers `importlib`-based runtime APIs over `pkg_resources`. If older examples rely on `pkg_resources`, treat them as migration candidates rather than new patterns to copy.

## Practical Decision Rules For Agents

- If you are packaging a normal library or CLI, start with `pyproject.toml` and `setuptools.build_meta`.
- If package discovery behaves strangely, switch to `src/` layout before adding more custom exclusions.
- If a project mixes `setup.py`, `setup.cfg`, and `pyproject.toml`, treat `pyproject.toml` as the target end state and minimize imperative `setup.py`.
- If docs or blog posts mention `easy_install`, `pkg_resources`, or direct `setup.py` commands, assume they may be stale until confirmed against current upstream docs.

## Official Sources

- Docs root: https://setuptools.pypa.io/en/latest/
- Quickstart: https://setuptools.pypa.io/en/latest/userguide/quickstart.html
- Package discovery: https://setuptools.pypa.io/en/latest/userguide/package_discovery.html
- Entry points: https://setuptools.pypa.io/en/latest/userguide/entry_point.html
- Build backend: https://setuptools.pypa.io/en/latest/build_meta.html
- Supported interfaces and deprecations: https://setuptools.pypa.io/en/latest/userguide/interfaces.html
- Changelog: https://setuptools.pypa.io/en/latest/history.html
- PyPI registry: https://pypi.org/project/setuptools/

---
name: package
description: "flit-core Python build backend for packaging projects with pyproject.toml and PEP 517"
metadata:
  languages: "python"
  versions: "3.12.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flit-core,python,packaging,pyproject,pep517,build-backend,pypi"
---

# flit-core Python Package Guide

## Golden Rule

`flit-core` is the build backend, not the full `flit` CLI. Configure it in `pyproject.toml` with `build-backend = "flit_core.buildapi"` and let a frontend such as `pip` or `python -m build` call it. If you need commands like `flit init`, `flit build`, or `flit publish`, install `flit`, not only `flit-core`.

## Install

For a locked environment:

```bash
python -m pip install "flit-core==3.12.0"
```

Most projects should declare it in `pyproject.toml` as a build dependency instead of importing it directly at runtime:

```toml
[build-system]
requires = ["flit_core >=3.11,<4"]
build-backend = "flit_core.buildapi"
```

Use an exact pin like `flit_core==3.12.0` only when your project or CI needs full reproducibility.

## Minimal Setup

For new projects on Flit 3.x, prefer PEP 621 metadata in `[project]`:

```toml
[build-system]
requires = ["flit_core >=3.11,<4"]
build-backend = "flit_core.buildapi"

[project]
name = "demo-pkg"
authors = [{name = "Example Maintainer", email = "dev@example.com"}]
readme = "README.md"
license = {file = "LICENSE"}
requires-python = ">=3.9"
dynamic = ["version", "description"]

[project.urls]
Home = "https://example.com/demo-pkg"
```

Then expose the version and module docstring from your import package:

```python
"""Short package description used by Flit."""

__version__ = "0.1.0"
```

By default, Flit derives the importable module name from the project name, replacing hyphens with underscores.

## Core Usage

### Build a wheel and sdist

After the project is configured, build artifacts with a frontend:

```bash
python -m pip install build
python -m build
```

This uses `flit_core.buildapi` from your `pyproject.toml`.

### Package layout

Flit supports a simple top-level module or a package directory. A common layout is:

```text
pyproject.toml
README.md
LICENSE
src/
  demo_pkg/
    __init__.py
```

If your import package name does not match the normalized project name, set it explicitly:

```toml
[tool.flit.module]
name = "demo_pkg"
```

### Console scripts

Declare CLI entry points with standard `[project.scripts]` metadata:

```toml
[project.scripts]
demo-pkg = "demo_pkg.cli:main"
```

### Source distributions

If you need extra files in the sdist or want to exclude generated files, configure them explicitly:

```toml
[tool.flit.sdist]
include = ["tests/", "docs/"]
exclude = ["docs/_build/"]
```

This matters because `flit-core` itself is just the backend. Some workflows that use the `flit` CLI add VCS-tracked files by default, but direct backend builds do not.

## Configuration Notes

- Prefer `[project]` metadata for new work. Flit 3.x still recognizes the older `[tool.flit.metadata]` style, but it is legacy.
- `dynamic = ["version", "description"]` reads `__version__` and the module docstring from your package.
- `requires-python`, dependencies, optional dependencies, entry points, and URLs belong in standard PEP 621 metadata under `[project]`.
- Reproducible build timestamps can be controlled with `SOURCE_DATE_EPOCH`.
- `flit-core` has no authentication flow of its own. Package publishing credentials apply to tools such as `flit publish` or `twine`, not to the backend configuration shown here.

## Common Pitfalls

- Installing `flit-core` does not install the `flit` CLI.
- Use `build-backend = "flit_core.buildapi"`. The older `flit.buildapi` backend name is deprecated in the 3.x line.
- Do not assume the project name and import package are identical. Hyphens are normalized to underscores, and some projects need an explicit `[tool.flit.module]`.
- If you rely on non-code files in source distributions, configure `[tool.flit.sdist]` instead of assuming they will be discovered automatically.
- Old blog posts often show `[tool.flit.metadata]` examples. They still work in `flit-core` 3.x, but they are the wrong starting point for new projects.
- `flit-core` is a build-time dependency. It usually should not appear in your application imports or runtime requirements.

## Version-Sensitive Notes For 3.12.0

- The stable Flit docs are versioned for `3.12.0`, so `https://flit.pypa.io/en/stable/` is the correct canonical doc root for this package version.
- Flit `3.12` adds support for SPDX license expressions with `AND` and `OR`, and it accepts an annotated assignment such as `__version__: str = "0.1.0"` for dynamic version loading.
- Flit `3.11` added support for PEP 639 license expressions and `license-files`, so examples using those fields require at least `flit_core >=3.11`.
- Flit `3.10` deprecated the old `flit.buildapi` backend path. For 3.12 projects, use `flit_core.buildapi` only.
- The old metadata format is still accepted in the 3.x series, but the docs state it is removed in `flit_core` 4.0. If you want an easy future upgrade path, stay on `[project]` now.

## Bootstrap And Advanced Packaging Notes

- If you are bootstrapping packaging tooling itself, Flit documents `python -m flit_core.wheel` as the low-level way to build a wheel for Flit without having `flit` installed already.
- For normal project packaging, prefer standard build frontends and keep `flit-core` behind the PEP 517 interface.

## Official Sources

- Flit stable docs: `https://flit.pypa.io/en/stable/`
- Flit `pyproject.toml` reference: `https://flit.pypa.io/en/stable/pyproject_toml.html`
- Flit bootstrap notes: `https://flit.pypa.io/en/stable/bootstrap.html`
- Flit history: `https://flit.pypa.io/en/stable/history.html`
- Flit reproducible builds: `https://flit.pypa.io/en/stable/reproducible.html`
- PyPI package page: `https://pypi.org/project/flit-core/`

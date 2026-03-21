---
name: package
description: "Hatchling build backend for Python packages, covering pyproject.toml setup, build targets, version sources, and hooks"
metadata:
  languages: "python"
  versions: "1.29.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,packaging,build-backend,pep517,pyproject,hatchling"
---

# Hatchling Python Package Guide

## Golden Rule

Use `hatchling` as your PEP 517 build backend in `pyproject.toml`, not as a runtime dependency. Put project metadata in `[project]`, put build behavior in `[tool.hatch.build...]`, and let a frontend like `pip`, `python -m build`, or `hatch build` drive the build.

## Install

In most projects, do not manually import or run `hatchling`. Declare it in `[build-system]` and let the build frontend install it in an isolated build environment.

Minimal setup:

```toml
[build-system]
requires = ["hatchling>=1.29.0"]
build-backend = "hatchling.build"

[project]
name = "acme-widget"
version = "0.1.0"
description = "Example package"
readme = "README.md"
requires-python = ">=3.10"
dependencies = [
  "httpx>=0.27",
]
```

Useful local tools:

```bash
python -m pip install --upgrade build
python -m pip install "hatchling==1.29.0"
```

Use `build` when you want `python -m build`; the frontend is separate from Hatchling itself.

## Initialize A Package

A practical `src/` layout looks like this:

```text
pyproject.toml
README.md
src/
  acme_widget/
    __init__.py
```

Minimal `pyproject.toml` for a `src/` layout:

```toml
[build-system]
requires = ["hatchling>=1.29.0"]
build-backend = "hatchling.build"

[project]
name = "acme-widget"
version = "0.1.0"
description = "Example package"
readme = "README.md"
requires-python = ">=3.10"

[tool.hatch.build.targets.wheel]
packages = ["src/acme_widget"]
```

`packages = ["src/acme_widget"]` tells the wheel target what package to ship and rewrites the installed path to `acme_widget`.

## Core Usage

### Build distributions

Hatchling is a backend. Common frontend commands are:

```bash
# Build sdist + wheel
python -m build

# Build only a wheel
python -m build --wheel

# Build only an sdist
python -m build --sdist

# Install the local project through pip
python -m pip install .

# If you use Hatch as the frontend
hatch build
```

### Use standard project metadata

Hatchling uses PEP 621 project metadata from `[project]`:

```toml
[project]
name = "acme-widget"
version = "0.1.0"
description = "Example package"
readme = "README.md"
requires-python = ">=3.10"
dependencies = ["httpx>=0.27"]

[project.optional-dependencies]
dev = ["pytest>=8", "ruff>=0.11"]

[project.scripts]
acme-widget = "acme_widget.cli:main"
```

Keep package metadata here unless the field is explicitly dynamic.

### Control what goes into the wheel and sdist

Global build settings live under `[tool.hatch.build]`. Target-specific settings live under `[tool.hatch.build.targets.<target>]`.

Example:

```toml
[tool.hatch.build]
include = [
  "/src",
  "/README.md",
  "/LICENSE",
]
artifacts = [
  "build/*.json",
]

[tool.hatch.build.targets.wheel]
packages = ["src/acme_widget"]
only-packages = true

[tool.hatch.build.targets.sdist]
include = [
  "/src",
  "/tests",
  "/README.md",
  "/LICENSE",
  "/pyproject.toml",
]
```

Practical rules:

- `wheel` should usually contain only importable package contents plus explicitly selected data.
- `sdist` should usually contain source, tests, docs needed for build, and packaging metadata.
- `only-packages = true` removes non-package files from the wheel unless you force-include them.
- `artifacts` are for files generated during the build or otherwise included outside normal VCS-style selection.

### Dynamic versioning

If you do not want a static version in `[project]`, declare it as dynamic and configure `[tool.hatch.version]`.

Regex source, using the default `__version__` pattern:

```toml
[project]
name = "acme-widget"
dynamic = ["version"]

[tool.hatch.version]
path = "src/acme_widget/__about__.py"
```

```python
# src/acme_widget/__about__.py
__version__ = "0.1.0"
```

Other built-in sources worth knowing:

- `source = "code"` loads a Python file and evaluates an expression, defaulting to `__version__`
- `source = "env"` reads the version from an environment variable
- `hatch-vcs` is the main third-party option when versioning should come from Git tags

If you use `source = "code"` with a `src/` layout and the file imports your own package modules, you may need absolute imports plus `search-paths = ["src"]`.

### Build hooks

Build hooks run around each selected build target and can modify build data before or after the build.

Custom hooks normally live in `hatch_build.py`:

```toml
[tool.hatch.build.targets.wheel.hooks.custom]
```

```python
from hatchling.builders.hooks.plugin.interface import BuildHookInterface

class CustomBuildHook(BuildHookInterface):
    def initialize(self, version, build_data):
        build_data["artifacts"].append("build/generated/schema.json")
```

Useful facts:

- Global build hooks run before target-specific hooks.
- Hooks can influence build data such as `artifacts` and forced inclusions.
- If you need a hook dependency or a third-party hook plugin, add it to `[build-system].requires`.

### Metadata hooks

Metadata hooks modify project metadata after Hatchling loads it. This is useful when fields like dependencies or the readme must be rewritten at build time.

Custom metadata hooks also default to `hatch_build.py`:

```toml
[tool.hatch.metadata.hooks.custom]
```

```python
from hatchling.metadata.plugin.interface import MetadataHookInterface

class CustomMetadataHook(MetadataHookInterface):
    def update(self, metadata):
        metadata["keywords"] = ["acme", "widget"]
```

### Write the version into a file during builds

If your runtime code needs a version file generated at build time, use the built-in version build hook:

```toml
[project]
name = "acme-widget"
dynamic = ["version"]

[tool.hatch.version]
path = "src/acme_widget/__about__.py"

[tool.hatch.build.hooks.version]
path = "src/acme_widget/_version.py"
```

That hook writes the resolved project version into the target file during the build.

## Configuration And Environment

There is no authentication model for Hatchling itself.

Configuration lives in `pyproject.toml`. Environment-sensitive behavior mainly appears in these cases:

- `source = "env"` versioning reads the version from an environment variable
- build frontends create isolated environments from `[build-system].requires`
- `HATCH_METADATA_CLASSIFIERS_NO_VERIFY` disables trove-classifier verification if you need to bypass that check

If a build depends on local plugins, dynamic metadata, or custom hooks, treat `[build-system].requires` as the complete build-time dependency set.

## Common Pitfalls

- Do not confuse Hatchling with the full `hatch` CLI. `hatchling` is the backend; `hatch` is an optional frontend and project manager.
- Do not put package metadata like `dependencies`, entry points, or `requires-python` under `[tool.hatch.build]`; those belong under `[project]`.
- If you set `dynamic = ["version"]`, you must also configure a version source. Otherwise the build will not know how to resolve the version.
- If Hatchling cannot determine what the wheel should include, define `packages`, `only-include`, `include`, or `force-include` explicitly. Recent Hatchling versions are stricter here and fail instead of guessing silently.
- `only-packages = true` is convenient, but it will drop top-level files that are not part of a Python package. Add explicit inclusion for package data or generated assets you still need.
- `artifacts` are not affected by `exclude`. If you need to exclude a subset of artifact matches, use more precise patterns or explicit negation.
- `skip-excluded-dirs = true` can speed up builds, but it may prevent discovery under directories you later expected Hatchling to inspect. Use it carefully in repos with generated files or unusual layouts.
- If you use the `code` version source, relative imports inside the loaded file are a common failure mode. Prefer absolute imports and set `search-paths` when needed.
- Hook plugins and metadata plugins are build-time dependencies. If they are missing from `[build-system].requires`, the build can fail only in isolated CI builds while appearing to work locally.

## Version-Sensitive Notes For 1.29.0

- As of March 12, 2026, PyPI lists `hatchling 1.29.0` and requires Python `>=3.10`.
- `1.28.0` dropped Python 3.9 support and added the `sbom-files` option plus `sbom_files` build data for the `wheel` target. If your build still assumes Python 3.9, you must stay on an older release.
- `1.27.0` updated the default core metadata version to `2.4`.
- `1.26.0` changed `license-files` to the newer array-of-glob-patterns form and added `HATCH_METADATA_CLASSIFIERS_NO_VERIFY`.
- PyPI marks `1.26.0`, `1.26.1`, and `1.26.2` as yanked. Do not copy pins to those versions from stale lockfiles or blog posts; use `1.26.3+`.

## Official Sources

- Docs root: `https://hatch.pypa.io/latest/`
- Build configuration: `https://hatch.pypa.io/latest/config/build/`
- Project metadata: `https://hatch.pypa.io/latest/config/metadata/`
- Wheel builder: `https://hatch.pypa.io/latest/plugins/builder/wheel/`
- Source distribution builder: `https://hatch.pypa.io/latest/plugins/builder/sdist/`
- Versioning: `https://hatch.pypa.io/latest/version/`
- Version source plugins: `https://hatch.pypa.io/latest/plugins/version-source/reference/`
- Build hook plugins: `https://hatch.pypa.io/latest/plugins/build-hook/reference/`
- Metadata hook plugins: `https://hatch.pypa.io/latest/plugins/metadata-hook/reference/`
- Hatchling release history: `https://hatch.pypa.io/dev/history/hatchling/`
- PyPI package page: `https://pypi.org/project/hatchling/`

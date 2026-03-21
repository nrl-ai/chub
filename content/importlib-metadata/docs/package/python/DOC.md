---
name: package
description: "Backport of importlib.metadata for reading installed distribution metadata and entry points in Python"
metadata:
  languages: "python"
  versions: "8.7.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python,packaging,metadata,entry-points,importlib"
---

# importlib-metadata Python Package Guide

Use `importlib-metadata` when you need the backport of `importlib.metadata`, want a consistent metadata API across Python versions, or are migrating older `pkg_resources` metadata code.

## Golden Rule

- The PyPI package name is `importlib-metadata`, but the import name is `importlib_metadata`.
- The library works on installed distribution metadata, not arbitrary import names.
- On Python versions where the standard library is sufficient, prefer `from importlib import metadata`.
- Install the backport when you need features or behavior newer than the stdlib copy bundled with the interpreter.
- Query distribution names such as `importlib-metadata`, `wheel`, or `PyYAML`, not import package names such as `importlib_metadata`, `wheel`, or `yaml`.

## Version Scope

- Package covered here: `importlib-metadata==8.7.1`
- Version used here: `8.7.1`
- Release date for `8.7.1`: `2025-12-21`
- Python requirement on PyPI: `>=3.9`
- Docs drift note: the source URL `https://importlib-metadata.readthedocs.io/en/latest/` is a moving target and rendered `8.7.2.dev1+gd8a7576de` on `2026-03-12`; use the official `stable` pages for `8.7.1`-anchored behavior.

The examples below stay on APIs available in `8.7.1`.

## Install

```bash
pip install importlib-metadata==8.7.1
```

If you only need the backport on interpreters where the stdlib copy is older than the behavior you want, use an environment marker in project metadata:

```toml
dependencies = [
  "importlib-metadata>=8.7.1; python_version < '3.10'",
]
```

If you need one pinned behavior across all supported interpreters, install it unconditionally and use the compatibility import shown below.

## Setup and Import Pattern

There is no auth, network setup, or environment-variable configuration. The package reads metadata from distributions that are installed in the current Python environment and discoverable on `sys.path`.

### Backport-only import

```python
from importlib_metadata import (
    PackageNotFoundError,
    distribution,
    entry_points,
    files,
    metadata,
    packages_distributions,
    requires,
    version,
)
```

### Cross-version compatibility import

Use this when the same codebase may run with either the stdlib module or the backport:

```python
try:
    from importlib import metadata as importlib_metadata
except ImportError:
    import importlib_metadata
```

Then call everything through `importlib_metadata`:

```python
wheel_version = importlib_metadata.version("wheel")
```

This follows the upstream recommendation: start with the stdlib docs for usage, but substitute `importlib_metadata` for `importlib.metadata` when using the backport.

## Core Usage

### Get the installed version of a distribution

```python
from importlib_metadata import PackageNotFoundError, version

def get_dist_version(dist_name: str) -> str | None:
    try:
        return version(dist_name)
    except PackageNotFoundError:
        return None

print(get_dist_version("pip"))
```

Use the distribution name from packaging metadata. That is not always the same as the import name used in Python code.

### Read package metadata

```python
from importlib_metadata import metadata, requires

dist_meta = metadata("setuptools")

print(dist_meta["Name"])
print(dist_meta["Summary"])
print(dist_meta.json["requires_python"])
print(requires("setuptools") or [])
```

`metadata()` returns message-style package metadata, and `.json` exposes a JSON-compatible view of the same data. `requires()` returns dependency requirement strings or `None`.

### Discover entry points

Modern code should use the selectable `entry_points()` API:

```python
from importlib_metadata import entry_points

console_scripts = entry_points(group="console_scripts")

for ep in console_scripts:
    if ep.name == "uvicorn":
        print(ep.value)
```

You can also filter by both group and name:

```python
from importlib_metadata import entry_points

matches = entry_points(group="pytest11", name="pytest_cov")

for ep in matches:
    plugin = ep.load()
    print(plugin)
```

Do not write new code assuming `entry_points()` returns a dict keyed by group. In modern `importlib_metadata`, it returns an `EntryPoints` collection.

### Inspect installed files

```python
from importlib_metadata import files

dist_files = files("wheel")
if dist_files is not None:
    for path in dist_files:
        if path.name == "METADATA":
            print(path.locate())
```

Use this when you need the installed file list or an on-disk path for a distribution artifact.

### Work with a `Distribution` object

```python
from importlib_metadata import distribution

dist = distribution("wheel")

print(dist.version)
print(dist.metadata["Name"])

if getattr(dist, "origin", None) is not None:
    print(dist.origin.url)
```

Use `distribution()` when you want one object that exposes `version`, `metadata`, `files`, and related properties together.

### Map import packages to owning distributions

```python
from importlib_metadata import packages_distributions

mapping = packages_distributions()

print(mapping.get("yaml"))
print(mapping.get("google"))
```

This is the official way to answer "which installed distribution provides this import package?".

## Environment and Runtime Notes

- The package only sees distributions installed in the interpreter environment that is currently running your code.
- In virtualenv-heavy tooling, run the lookup inside the target environment instead of the system interpreter.
- It works with discoverable `dist-info` or `egg-info` metadata and primarily targets packages installed by standard PyPA tooling.
- By default it reads from file system and zip-based distributions visible on `sys.path`.
- There are no environment variables, credentials, or service endpoints to configure.

## Common Pitfalls

- Package name and import name differ: install `importlib-metadata`, import `importlib_metadata`.
- Distribution names and import package names are not 1:1. Use `packages_distributions()` when you only know the import name.
- `version()`, `metadata()`, `files()`, and `requires()` raise `PackageNotFoundError` when the distribution is not installed.
- `files()` can return `None` if the installed distribution metadata does not expose a file list. Guard before iterating.
- Namespace packages may map to multiple distributions, and some editable installs do not supply complete top-level name metadata.
- `importlib_metadata` is not a `pkg_resources` working-set manager. It does not support multi-version installs or `require()` semantics.
- Entry points are not automatically validated the way `pkg_resources.iter_entry_points()` behaved. Access a property such as `ep.name` or call `ep.load()` to force real use.
- `EntryPoint.load()` does not verify that declared extras are installed.
- Do not write new code against the old dict-like `entry_points()` behavior. Use `entry_points(group=...)` or `.select(...)`.

## Version-Sensitive Notes

- `8.7.1` includes fixes for FastPath behavior under fork-multiprocessing. If metadata discovery behaves strangely only after a fork, do not copy pre-`8.7.1` workarounds blindly.
- `8.7.0` changed `.metadata()` and `Distribution.metadata` so they can return `None` when a metadata directory exists but no metadata file is present. Guard if you inspect broken or partially installed distributions.
- `importlib_metadata 7.0` roughly corresponds to the copy merged into Python `3.13`, `6.5` to `3.12`, `4.13` to `3.11`, `4.6` to `3.10`, and `1.4` to `3.8`. If you are debugging version drift between stdlib and backport behavior, check that mapping first.
- The backport can be ahead of the stdlib copy that ships with a given interpreter. If you need one consistent feature set across Python versions, standardize on the compatibility import pattern above.
- The official docs homepage points readers at the Python standard library documentation for general usage. That is correct, but remember the import path changes between stdlib (`importlib.metadata`) and the backport (`importlib_metadata`).

## Recommended Agent Workflow

1. Confirm whether the target project should use the stdlib module or the backport.
2. Use distribution names for lookups unless you explicitly map from imports with `packages_distributions()`.
3. Wrap metadata lookups in `PackageNotFoundError` handling when the target distribution may be optional.
4. Guard `files()` and, for `8.7.0+`, be prepared for `metadata()` to be absent on malformed installs.
5. Prefer `entry_points(group=..., name=...)` selection style instead of older compatibility patterns.
6. If you are migrating from `pkg_resources`, plan replacement for working-set behavior separately. `importlib_metadata` only covers metadata discovery.

## Official Sources

- Stable docs root: https://importlib-metadata.readthedocs.io/en/stable/
- Migration guide: https://importlib-metadata.readthedocs.io/en/stable/migration.html
- Release history: https://importlib-metadata.readthedocs.io/en/stable/history.html
- Python stdlib reference: https://docs.python.org/3/library/importlib.metadata.html
- PyPI project page: https://pypi.org/project/importlib-metadata/8.7.1/

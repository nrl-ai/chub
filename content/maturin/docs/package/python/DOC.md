---
name: package
description: "maturin guide for building, testing, and publishing Rust-backed Python packages"
metadata:
  languages: "python"
  versions: "1.12.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "maturin,rust,python,packaging,wheels,pyo3,cffi,uniffi"
---

# maturin Python Package Guide

## Golden Rule

Use `maturin` when you need to build or publish a Python package backed by Rust. Keep packaging metadata and `maturin` settings in `pyproject.toml`, use `maturin develop` for fast local iteration inside a virtualenv, and use `maturin build -r` for the wheel you intend to ship.

## Install The Tool

The upstream docs prefer installing `maturin` as a tool instead of as a project dependency:

```bash
pipx install maturin
uv tool install maturin
```

`pip install maturin` also works:

```bash
python -m pip install "maturin==1.12.6"
```

If you need platform-repair or cross-compilation helpers, the published extras are:

```bash
python -m pip install "maturin[patchelf]==1.12.6"
python -m pip install "maturin[zig]==1.12.6"
```

If you already have Rust via `rustup` and want a Rust-native install path:

```bash
cargo install --locked maturin
```

## Initialize A Project

For a new package, `maturin new` is the quickest path:

```bash
maturin new --bindings pyo3 --mixed hello-maturin
cd hello-maturin
python -m venv .venv
source .venv/bin/activate
maturin develop
```

Useful `maturin new` options from the user guide:

- `--bindings pyo3|cffi|uniffi|bin`
- `--mixed` for a mixed Rust/Python layout
- `--src` for a Python-first source layout under `src/`

If you already have a Rust crate, add a `pyproject.toml` next to `Cargo.toml`:

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "hello-maturin"
version = "0.1.0"
requires-python = ">=3.8"

[tool.maturin]
bindings = "pyo3"
compatibility = "pypi"
```

For older `pyo3` projects, add the extension feature explicitly:

```toml
[tool.maturin]
features = ["pyo3/extension-module"]
```

You only need that for `pyo3 0.26` and earlier. The user guide says `pyo3 0.27` handles it automatically.

## Recommended Layouts

### Pure Rust package

For a pure Rust project, keep `Cargo.toml`, `pyproject.toml`, and `src/` at the project root. `maturin` will generate the package `__init__.py` during wheel build.

### Mixed Rust/Python package

For most real packages, the mixed layout is safer and more flexible:

```text
hello-maturin/
├── Cargo.toml
├── pyproject.toml
├── python/
│   └── hello_maturin/
│       └── __init__.py
└── src/
    └── lib.rs
```

Configure it like this:

```toml
[tool.maturin]
bindings = "pyo3"
python-source = "python"
module-name = "hello_maturin._core"
compatibility = "pypi"
auditwheel = "repair"
```

This layout avoids the common import-path collision the upstream docs call out. If you set `module-name = "hello_maturin._core"`, make the Rust module name match `_core` in your `#[pymodule]` declaration.

## Core Commands

### Fast local development

Run this inside an active virtualenv:

```bash
maturin develop
```

Common variants:

```bash
maturin develop -r
maturin develop --features pyo3/abi3-py38
maturin develop --skip-install
```

Notes:

- `develop` builds in debug mode by default.
- `--skip-install` only makes sense for mixed Rust/Python layouts.
- `develop` is faster than the wheel-install path, but the upstream docs explicitly note it does not support every feature that `pip install` after `maturin build` supports.

### Build wheels

```bash
maturin build -r
```

By default, wheels go to `target/wheels/`.

Useful variants:

```bash
maturin build -r --compatibility pypi
maturin build -r --bindings cffi
maturin build -r --target x86_64-unknown-linux-gnu
```

For Linux wheels intended for broad PyPI use, keep `compatibility = "pypi"` and use a manylinux container or Zig-based cross compilation when needed.

### Build an sdist or use the PEP 517 backend

If the project has the `[build-system]` block shown above, you can use standard Python packaging flows:

```bash
pip install .
pip install . --config-settings="build-args=--features pyo3/abi3-py38"
python -m pip wheel .
maturin sdist
```

`maturin` also honors:

```bash
export MATURIN_PEP517_ARGS="--features pyo3/abi3-py38 --profile release"
```

But `pip --config-settings` takes priority over `MATURIN_PEP517_ARGS`.

## Binding Modes You Should Pick Explicitly

`maturin` supports these binding types:

- `pyo3`
- `pyo3-ffi`
- `cffi`
- `uniffi`
- `bin`

Practical guidance:

- `pyo3` is the default choice for Python extension modules.
- `cffi` and `uniffi` can produce wheels usable across Python versions, but they have extra setup requirements.
- `bin` packages a Rust executable as a Python-installed script on the user's `PATH`.

Important detection rule:

- `cffi` and `bin` projects are the ones most worth declaring explicitly in `[tool.maturin]` or on the CLI, especially when auto-detection is ambiguous.

Example:

```toml
[tool.maturin]
bindings = "cffi"
compatibility = "linux"
```

## Configuration That Matters Most

The main `maturin` options live under `[tool.maturin]` in `pyproject.toml`.

```toml
[tool.maturin]
bindings = "pyo3"
compatibility = "pypi"
auditwheel = "repair"
python-source = "python"
include = ["hello_maturin/data/*.json"]
strip = true
use-base-python = false
```

What these settings do in practice:

- `bindings`: selects `pyo3`, `cffi`, `uniffi`, or `bin`
- `compatibility`: controls the wheel platform tag and PyPI compatibility checks
- `auditwheel`: `repair`, `check`, or `skip` for Linux shared-library handling
- `python-source`: points at your Python package tree for mixed layouts
- `include`: adds extra files to `sdist` and/or wheel outputs
- `strip`: reduces native artifact size
- `use-base-python`: can avoid repeated rebuilds in PEP 517 flows, but upstream warns not to use it when the sdist build needs packages installed inside the venv

## Publishing And Auth

The README now recommends building with `maturin` and publishing with `uv`:

```bash
maturin build -r
uv publish target/wheels/*
```

If you still rely on `maturin` upload credentials, the documented environment variables are:

```bash
export MATURIN_PYPI_TOKEN="pypi-..."
export MATURIN_REPOSITORY="pypi"
export MATURIN_REPOSITORY_URL="https://test.pypi.org/legacy/"
export MATURIN_USERNAME="__token__"
export MATURIN_PASSWORD="..."
export MATURIN_NON_INTERACTIVE=1
```

Use `MATURIN_REPOSITORY_URL` for custom indexes or TestPyPI. Prefer token auth over username/password when publishing to PyPI.

## Cross-Compilation And Platform Notes

- The upstream install docs expose a `zig` extra specifically for easier cross compilation and manylinux compliance.
- The `patchelf` extra is for repairing Linux wheels that link additional shared libraries.
- On Linux, `auditwheel = "repair"` is the default path for fixing wheel portability when possible.
- For macOS universal2 or multi-arch builds, `ARCHFLAGS="-arch x86_64 -arch arm64"` is the documented environment-variable path.
- For PyO3 cross compilation, `PYO3_CROSS_PYTHON_VERSION`, `PYO3_CROSS_LIB_DIR`, and `PYO3_CONFIG_FILE` are the main env vars the user guide documents.

## Common Pitfalls

- Do not keep old `maturin` metadata in `Cargo.toml`. Current configuration belongs in `[tool.maturin]` inside `pyproject.toml`.
- Do not assume `maturin develop` is identical to `pip install .`; it is an editable-development shortcut, not the full packaging path.
- For mixed layouts, avoid giving the native extension the same import name as the top-level Python package unless you want the nested import style.
- If you use `python-source`, make sure the Python package actually exists there; the `1.12` line tightened checks around missing modules.
- For `cffi` and `uniffi`, budget for extra tooling and packaging steps instead of assuming the `pyo3` examples map over directly.
- For Linux wheel publishing, a local build on a modern distro often produces a too-specific `linux` tag unless you use manylinux or Zig correctly.
- `maturin` builds Rust artifacts; if `cargo` is unavailable during a PEP 517 build, the tool may try to install Rust unless you set `MATURIN_NO_INSTALL_RUST`.

## Version-Sensitive Notes

- The version used here `1.12.6` matches the current PyPI latest release as of March 12, 2026. PyPI shows it was released on March 1, 2026.
- When using `maturin` as a build backend, the upstream tutorial still recommends `requires = ["maturin>=1.0,<2.0"]` rather than pinning an exact patch release in `pyproject.toml`.
- The migration guide says the old `[package.metadata.maturin]` path was removed in `0.15`; modern projects should keep config in `[tool.maturin]`.
- The changelog marks `publish` and `upload` as deprecated since `1.11.0`, which is why the current README recommends `uv publish`.
- The `1.12.0` line added `MATURIN_STRIP` support, improved include handling, and hardened `python-source` validation. If you are upgrading from older `1.x` releases, re-check custom include globs and mixed-layout packaging.

## Official Links

- User guide: https://www.maturin.rs/
- Installation: https://www.maturin.rs/installation.html
- Tutorial: https://www.maturin.rs/tutorial.html
- Project layout: https://www.maturin.rs/project_layout.html
- Bindings: https://www.maturin.rs/bindings.html
- Configuration: https://www.maturin.rs/config.html
- Environment variables: https://www.maturin.rs/environment-variables.html
- Migration guide: https://www.maturin.rs/migration.html
- Changelog: https://www.maturin.rs/changelog.html
- PyPI: https://pypi.org/project/maturin/

---
name: package
description: "wheel package guide for Python wheel-file tooling and legacy build compatibility"
metadata:
  languages: "python"
  versions: "0.46.3"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "wheel,python,packaging,build,pypi"
---

# wheel Python Package Guide

## What It Is

`wheel` is the reference implementation of the Python wheel format (PEP 427) and the official command-line tool for manipulating `.whl` files.

For current Python packaging workflows, treat `wheel` primarily as a CLI utility:

- Use `build` plus `setuptools` to produce wheels for modern projects.
- Install `wheel` when you need CLI commands such as `convert`, `unpack`, `pack`, or `tags`.
- Keep `wheel` in legacy build environments only when an older `setuptools`-based `bdist_wheel` flow still depends on it.

## Install

Install the package when you need the `wheel` command-line tool:

```bash
python -m pip install wheel==0.46.3
```

The upstream docs also note that many system package managers expose it as `python-wheel` or `python3-wheel`.

## Setup

There is no auth, account setup, or remote service configuration. The main setup decision is whether your project actually needs `wheel` installed.

### Modern project build setup

For normal builds, use `build` and a current setuptools backend:

```toml
[build-system]
requires = ["setuptools>=70.1"]
build-backend = "setuptools.build_meta"
```

```bash
python -m pip install build
python -m build --wheel
```

This produces `dist/<project>-<version>-<tags>.whl`.

### Legacy setup for old `setuptools`

If you are forced to keep `setuptools` older than `70.1` and still rely on `bdist_wheel`, add `wheel` to the build environment explicitly:

```toml
[build-system]
requires = ["setuptools<70.1", "wheel==0.46.3"]
build-backend = "setuptools.build_meta"
```

That should be treated as compatibility-only. Upstream advises against installing `wheel` just to build wheels in modern projects.

## Core Usage

### Build a wheel for your project

Use `build`, not the `wheel` CLI, for standard package builds:

```bash
python -m pip install build
python -m build --wheel
```

### Convert a legacy egg or wininst artifact

```bash
wheel convert legacy_package-1.2.3-py3.11.egg
wheel convert old_installer.exe
wheel convert --dest-dir dist legacy_package-1.2.3-py3.11.egg
```

Use this only for migration from old packaging artifacts. `wheel convert` does not build from source.

### Unpack a wheel safely

```bash
wheel unpack dist/example_pkg-1.0.0-py3-none-any.whl
wheel unpack -d /tmp/unpacked dist/example_pkg-1.0.0-py3-none-any.whl
```

`wheel unpack` validates hashes and file sizes against `RECORD`, so it is safer than a plain `unzip` when you need to inspect or modify an existing wheel.

### Repack an unpacked wheel

```bash
wheel pack example_pkg-1.0.0
wheel pack --build-number 2 example_pkg-1.0.0
wheel pack --dest-dir dist example_pkg-1.0.0
```

This regenerates `RECORD` for the new archive.

### Adjust compatibility tags on an existing wheel

```bash
wheel tags \
  --python-tag=py3 \
  --abi-tag=none \
  --platform-tag=any \
  dist/example_pkg-1.0.0-cp311-cp311-manylinux_2_28_x86_64.whl
```

Use `+tag` to append tags and `-tag` to remove tags. `wheel tags` writes a new wheel unless you also pass `--remove`.

## Configuration

### `setup.cfg` options

For legacy setuptools-based projects, `wheel` still documents a few common config points in `setup.cfg`.

Include license files in the generated wheel:

```ini
[metadata]
license_files =
    LICENSE
    NOTICE
    licenses/*.txt
```

Mark a package as universal only for the old pure-Python Python 2/3 compatibility case:

```ini
[bdist_wheel]
universal = 1
```

Do not cargo-cult `universal = 1` into Python 3-only or extension-bearing packages.

## Common Pitfalls

- Do not add `wheel` as a normal runtime dependency. It is packaging tooling, not an application library.
- Do not keep `wheel` in `build-system.requires` by default for current setuptools projects. Build with `build` plus `setuptools>=70.1` instead.
- Do not import `wheel.cli`, `wheel.metadata`, or `wheel.bdist_wheel` in new code. Upstream made `wheel.cli` private in `0.46.0`, made `wheel.metadata` private, and deprecated the old `wheel.bdist_wheel` path.
- Do not use `wheel tags` to claim compatibility you have not actually tested. Retagging changes metadata; it does not rebuild binaries.
- Do not use `wheel convert` for source distributions. It is for old `.egg` files, `.egg` directories, and `bdist_wininst` installers.
- Prefer `wheel unpack` over `unzip` when auditing a wheel, because it verifies `RECORD` hashes and sizes.

## Version-Sensitive Notes

- `0.46.0` dropped Python 3.8 support.
- `0.46.0` removed wheel’s own `bdist_wheel` implementation and turned `wheel.bdist_wheel` into a deprecated alias to the setuptools implementation.
- `0.46.0` also made `wheel.cli` private and `wheel.metadata` private, so code that imported wheel internals is now on a weaker path.
- `0.46.2` restored `bdist_wheel` command compatibility for `setuptools` older than `70.1` and fixed a `wheel unpack` directory-traversal style issue tracked as `CVE-2026-24049`.
- `0.46.3` fixed an `ImportError` that could occur with older setuptools during `bdist_wheel`.

## Quick Decision Guide

- Need to build a wheel from source in a modern project: install `build`, not `wheel`.
- Need to inspect, retag, convert, or repack existing wheel artifacts: install `wheel`.
- Need `bdist_wheel` with an older setuptools stack: pin `wheel` for compatibility and plan a setuptools upgrade.

## Official Sources

- Documentation: `https://wheel.readthedocs.io/en/stable/`
- Installation: `https://wheel.readthedocs.io/en/stable/installing.html`
- User guide: `https://wheel.readthedocs.io/en/stable/user_guide.html`
- Reference guide: `https://wheel.readthedocs.io/en/stable/reference/index.html`
- Release notes: `https://wheel.readthedocs.io/en/stable/news.html`
- PyPI project: `https://pypi.org/project/wheel/`

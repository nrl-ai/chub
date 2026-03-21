---
name: package
description: "mypy package guide for Python projects using the official mypy 1.19.1 docs"
metadata:
  languages: "python"
  versions: "1.19.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "mypy,python,typing,type-checking,static-analysis,pep-561"
---

# mypy Python Package Guide

## Golden Rule

Use `mypy` as the type checker, configure it in a single repo-level config file, and check the packages your project actually imports instead of relying on blanket global ignores. `mypy` itself requires Python `>=3.9` to run, but it can type-check code that targets older Python versions when you set `python_version` in config.

## Install

Pin the version your project expects:

```bash
python -m pip install "mypy==1.19.1"
```

Common alternatives:

```bash
uv add --dev "mypy==1.19.1"
poetry add --group dev "mypy==1.19.1"
```

Optional extra for faster cache serialization:

```bash
python -m pip install "mypy[faster-cache]==1.19.1"
```

If you run `mypy` in CI, install it in the same environment as the project dependencies and stubs it needs to inspect.

## Initialize In An Existing Project

Start by checking a narrow target instead of the whole repository:

```bash
python -m mypy src/
python -m mypy path/to/module.py
```

Then add a repo-level config file. `mypy` looks for configuration in this order:

1. `mypy.ini`
2. `.mypy.ini`
3. `pyproject.toml`
4. `setup.cfg`

There is no config merging. Once `mypy` finds one file, that is the file it uses.

Recommended `pyproject.toml` starter:

```toml
[tool.mypy]
python_version = "3.12"
files = ["src", "tests"]
warn_unused_configs = true
strict = true
```

Use `python_version` for the code you are checking, not necessarily the Python version used to run `mypy`.

If `strict = true` is too noisy for an existing codebase, start with a narrower baseline:

```toml
[tool.mypy]
python_version = "3.12"
files = ["src"]
warn_unused_configs = true
disallow_untyped_defs = true
no_implicit_optional = true
warn_return_any = true
warn_redundant_casts = true
warn_unused_ignores = true
```

## Core Usage

### Check files, packages, or modules

```bash
python -m mypy src/
python -m mypy -p your_package
python -m mypy -m your_package.module
```

Useful patterns:

- Use `-p` to type-check an importable package.
- Use `-m` for a specific module.
- Use directory paths when you want the check to follow the on-disk tree directly.

### Use in CI

```bash
python -m mypy --config-file pyproject.toml
```

If the config already sets `files = [...]`, you can omit paths on the command line and let the config control the scope.

### Use per-module overrides instead of global ignores

Prefer targeted overrides in config:

```toml
[tool.mypy]
python_version = "3.12"
warn_unused_configs = true

[[tool.mypy.overrides]]
module = ["legacy_package.*"]
ignore_errors = true

[[tool.mypy.overrides]]
module = ["untyped_vendor_lib"]
ignore_missing_imports = true
```

This keeps the rest of the repo checked normally.

### Install missing stub packages

If a third-party library ships no inline types, `mypy` may suggest a PEP 561 stub package such as `types-requests`.

Two options:

1. Install the suggested stub package directly in your environment.
2. Let `mypy` install available stubs:

```bash
python -m mypy --install-types --non-interactive
```

`--install-types` can be convenient, but it effectively adds an install step and then reruns the type check, so it is slower and less deterministic than pinning stub packages yourself.

### Speed up repeated runs with `dmypy`

For local development on larger codebases, use the daemon:

```bash
dmypy run -- src/
dmypy status
dmypy restart -- src/
```

`dmypy` keeps state across runs and is often much faster than cold-starting `mypy` every time.

## Import Discovery And Project Layout

`mypy` needs to resolve your imports the same way your project does.

Common setups:

- Standard package layout: run `mypy` from the repo root and check `src/` or the importable package.
- `src/` layout: keep `files = ["src", "tests"]` and run from the repo root.
- Additional local stub or source directories: set `MYPYPATH` or `mypy_path`.

Example:

```toml
[tool.mypy]
python_version = "3.12"
mypy_path = ["src", "stubs"]
```

If you use namespace packages or a nonstandard layout, read the import-discovery flags carefully before papering over errors with `ignore_missing_imports`.

## Config Notes

- `warn_unused_configs = true` is worth enabling immediately; it catches misspelled sections and ineffective overrides.
- Prefer one config file at the repo root. Do not split settings across `mypy.ini` and `pyproject.toml`.
- `files = [...]` makes CI and editor behavior more predictable than relying on ad hoc command-line paths.
- Use per-module overrides for legacy code, generated code, or untyped vendors.
- Use error-code-specific ignores when possible, for example `# type: ignore[attr-defined]`, instead of bare `# type: ignore`.

## Common Pitfalls

- Running `mypy` with one Python interpreter while the code and dependencies live in a different virtual environment. This produces missing-import noise and wrong platform assumptions.
- Using `--ignore-missing-imports` globally. It can hide real integration problems across the whole project.
- Forgetting that `mypy` stops at the first config file it finds. There is no merging between `mypy.ini`, `.mypy.ini`, `pyproject.toml`, and `setup.cfg`.
- Checking the wrong target. `python -m mypy src/` and `python -m mypy -p your_package` can behave differently if the project layout is off.
- Assuming every dependency ships types. Many still need separately installed stub packages.
- Leaving broad `# type: ignore` comments in place after upgrading `mypy`; newer releases may support narrower and more accurate fixes.
- Treating `strict = true` as all-or-nothing. On existing code, it is usually better to enable specific strictness flags and tighten them over time.

## Version-Sensitive Notes For 1.19.1

- As of March 12, 2026, PyPI and the stable docs both point to `mypy 1.19.1`.
- `mypy 1.19.1` still requires Python `>=3.9` to run, even if you are type-checking code that targets an older Python version.
- The stable changelog for the `1.19` line includes support work for newer Python versions such as Python 3.14. If your project is on a new interpreter, prefer current `1.19.x` docs over older blog posts.
- The `faster-cache` extra is available on PyPI in `1.19.1`; use it when cache performance matters and you control the development environment.
- Error codes, strictness defaults, and typeshed coverage keep evolving. Revalidate old `type: ignore` comments and per-module overrides when upgrading across `mypy` releases.

## Official Sources Used

- Stable docs root: `https://mypy.readthedocs.io/en/stable/`
- Getting started: `https://mypy.readthedocs.io/en/stable/getting_started.html`
- Config file docs: `https://mypy.readthedocs.io/en/stable/config_file.html`
- Running and managing imports: `https://mypy.readthedocs.io/en/stable/running_mypy.html`
- mypy daemon: `https://mypy.readthedocs.io/en/stable/mypy_daemon.html`
- Changelog: `https://mypy.readthedocs.io/en/stable/changelog.html`
- PyPI package page: `https://pypi.org/project/mypy/`

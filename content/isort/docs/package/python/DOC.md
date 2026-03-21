---
name: package
description: "isort Python package guide for sorting imports from the CLI, pre-commit, editors, and Python code"
metadata:
  languages: "python"
  versions: "8.0.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "isort,python,imports,formatting,linter,pre-commit"
---

# isort Python Package Guide

## Golden Rule

Use `isort` to normalize Python import blocks, keep the configuration in the repository root, and make the config explicit enough that first-party modules are classified correctly. For `8.0.1`, trust PyPI metadata for runtime support and release versioning over older snippets on the docs site: PyPI says `isort` now requires Python `>=3.10`.

## Install

Pin the version your project expects:

```bash
python -m pip install "isort==8.0.1"
```

Common alternatives:

```bash
uv add --dev "isort==8.0.1"
poetry add --group dev "isort==8.0.1"
```

PyPI metadata for `8.0.1` lists only the `colors` extra:

```bash
python -m pip install "isort[colors]==8.0.1"
```

Do not rely on old docs examples that mention extras such as `requirements_deprecated_finder` or `pipfile_deprecated_finder`; those examples are from older documentation snapshots and do not match current PyPI metadata.

## Setup And Configuration

`isort` does not need authentication. The main setup task is making import classification deterministic.

Supported config locations, in search order:

1. `.isort.cfg`
2. `pyproject.toml`
3. `setup.cfg`
4. `tox.ini`
5. `.editorconfig`

Important behavior:

- `isort` walks upward until it finds the nearest supported config file.
- It stops at the first config file it finds.
- It does not merge config files.
- It does not leave a Git or Mercurial repository when searching.
- Use `--settings-path` to force a config file or root.
- Use `--show-config` when imports are being classified unexpectedly.

For most repos, prefer `pyproject.toml`:

```toml
[tool.isort]
profile = "black"
py_version = 311
src_paths = ["src", "tests"]
known_first_party = ["my_package"]
skip_gitignore = true
filter_files = true
```

Why these settings matter:

- `profile = "black"` keeps wrapping compatible with Black.
- `py_version` avoids stdlib classification drift across Python versions.
- `src_paths` marks modules under those directories as first-party.
- `known_first_party` is the escape hatch for packages not covered by `src_paths`.
- `skip_gitignore = true` makes `isort` honor `.gitignore`, but it requires `git` to be installed.
- `filter_files = true` matters when files are passed explicitly, which is common in pre-commit.

## Core CLI Usage

Sort one or more files in place:

```bash
isort app.py package/__init__.py
```

Sort a tree recursively:

```bash
isort .
```

Preview changes without writing:

```bash
isort . --diff
```

Fail CI if imports would change:

```bash
isort . --check-only --diff
```

Use `--atomic` if you want `isort` to avoid writing changes that would introduce syntax errors:

```bash
isort --atomic .
```

Useful debugging commands:

```bash
isort . --show-config
isort . --show-files
isort . --verbose
```

## Python API Usage

`isort` exposes a direct Python API when you need formatting inside generators, codemods, or editor tooling.

Sort code in memory:

```python
import isort

source = """
import requests
import os
from my_package import api
"""

result = isort.code(source, profile="black")
print(result)
```

Check code in memory without rewriting:

```python
import isort

if not isort.check_code(source, profile="black"):
    raise ValueError("Imports are not sorted")
```

Sort a file on disk:

```python
import isort

isort.file("app.py", profile="black")
```

Debug import placement:

```python
import isort

section = isort.place_module("my_package", known_first_party=["my_package"])
print(section)
```

The quick-start docs use convenience names like `isort.code`, `isort.check_code`, `isort.file`, and `isort.place_module`. The API reference also documents the underlying `sort_code_string`, `check_code_string`, `sort_file`, and related functions.

## Pre-commit And Formatter Integration

If a repo uses Black, set the profile in config instead of only passing CLI flags. That keeps editor integrations, pre-commit, and local CLI runs aligned.

Example `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/PyCQA/isort
    rev: 8.0.1
    hooks:
      - id: isort
        args: ["--filter-files"]
```

Notes:

- The official pre-commit docs page still shows older `5.x` revisions; pin the hook to the current tag your repo has standardized on.
- The old `pre-commit/mirrors-isort` repository is deprecated. Use `PyCQA/isort` directly.
- If your repo still uses `seed-isort-config`, remove it unless you have a very specific legacy reason. The official docs say modern `isort` placement logic made that helper unnecessary starting in `isort 5`.

## Common Pitfalls

### First-party imports are grouped as third-party

Usually this means `src_paths`, `known_first_party`, `virtual_env`, or `conda_env` is not configured for the project layout. Check the resolved config first:

```bash
isort path/to/file.py --show-config
```

### Running from the wrong directory changes the chosen config

Config lookup is relative to the current directory for `isort .` and relative to the first explicit path when multiple paths are passed. If results vary between CI and local runs, compare working directories and use `--settings-path` if needed.

### `.gitignore` is not respected

`skip_gitignore` is off by default. Turn it on with `skip_gitignore = true` or `--skip-gitignore`, and make sure `git` is available in the environment.

### Pre-commit still rewrites files you expected it to skip

When pre-commit passes explicit file paths, add `filter_files = true` in config or `--filter-files` in hook args.

### Standard library classification is wrong for the project target

Set `py_version` to the project target instead of relying on the default union of Python 3 stdlib modules.

### isort and Black keep fighting

Set `profile = "black"` once in repo config. Do not try to hand-tune wrap settings unless the project has intentionally diverged from Black-compatible formatting.

## Version-Sensitive Notes For 8.0.1

- PyPI lists `8.0.1` as the latest release on February 28, 2026.
- PyPI metadata for `8.0.1` requires Python `>=3.10`.
- The docs home page still says `Python 3.7+`, which is stale for current releases.
- The docs site still includes older pre-commit snippets such as `rev: 5.11.2` and older Black compatibility examples with `rev: 5.6.4`; update those revisions when copying examples.
- The docs site still includes older optional-install examples that do not match current extras metadata on PyPI.
- GitHub’s public release page appears behind the current PyPI version in some crawled views, so use PyPI as the canonical source for the package version and Python requirement.

## Official Sources

- Docs: `https://pycqa.github.io/isort/`
- Config files: `https://pycqa.github.io/isort/docs/configuration/config_files.html`
- Options: `https://pycqa.github.io/isort/docs/configuration/options.html`
- Black compatibility: `https://pycqa.github.io/isort/docs/configuration/black_compatibility.html`
- Pre-commit: `https://pycqa.github.io/isort/docs/configuration/pre-commit.html`
- Python API quick start: `https://pycqa.github.io/isort/docs/quick_start/3.-api.html`
- Python API reference: `https://pycqa.github.io/isort/reference/isort/api.html`
- PyPI project page: `https://pypi.org/project/isort/`
- PyPI JSON metadata: `https://pypi.org/pypi/isort/json`

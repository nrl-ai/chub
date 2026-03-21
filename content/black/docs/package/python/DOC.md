---
name: package
description: "Black Python formatter guide for install, pyproject.toml configuration, CLI usage, and pre-commit integration"
metadata:
  languages: "python"
  versions: "26.3.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "black,python,formatter,code-style,pre-commit,pyproject"
---

# Black Python Package Guide

## Golden Rule

Use `black` as an opinionated formatter, not as a style linter you tune line by line. Put the configuration in `pyproject.toml`, run it from the CLI or `pre-commit`, and let Black own formatting decisions. Black's stable style changes are intentionally limited; larger formatting changes are behind `--preview` or annual stable-style updates.

## Install

Pin the exact version if your project or formatter checks depend on stable output:

```bash
python -m pip install "black==26.3.0"
```

Install notebook support when you need to format `.ipynb` files:

```bash
python -m pip install "black[jupyter]==26.3.0"
```

If you want the latest compatible patch in the same monthly line instead of the exact version used here:

```bash
python -m pip install "black~=26.3"
```

## Initialize And Configure

Black does not need API keys, credentials, or service setup. The only real setup step is project configuration.

Create `pyproject.toml` at the repository root:

```toml
[tool.black]
line-length = 88
target-version = ["py311", "py312"]
include = '\.pyi?$'
extend-exclude = '''
/(
  migrations
  | build
  | dist
)/
'''
required-version = "26.3.0"
```

Important config behavior:

- Black looks for `pyproject.toml` starting from the common base directory of the files you pass.
- It uses one configuration file per run.
- The search stops at the first `pyproject.toml`, a `.git` or `.hg` directory, or the filesystem root.
- CLI option names map directly to config keys in `[tool.black]` using the long flag name without leading dashes.

## Core Usage

Format one file:

```bash
black app.py
```

Format common project paths:

```bash
black src tests
```

Check formatting in CI without rewriting files:

```bash
black --check --diff .
```

Read from standard input:

```bash
python -m black -
```

Format a notebook explicitly:

```bash
black --ipynb notebook.ipynb
```

Useful flags agents commonly need:

- `--check`: fail instead of rewriting files
- `--diff`: print the diff that would be applied
- `--line-length <n>`: override the default `88`
- `--target-version py312`: constrain formatting to a Python syntax target
- `--required-version <version>`: fail when the installed Black version does not match expectations
- `--preview`: opt into style changes that are not in the current stable style yet

## Pre-commit Integration

Black is commonly run through `pre-commit` so formatting stays consistent across machines and CI:

```yaml
repos:
  - repo: https://github.com/psf/black
    rev: 26.3.0
    hooks:
      - id: black
      - id: black-jupyter
```

Use `black-jupyter` only if the repository actually formats notebooks.

## Common Workflows

### CI formatting gate

```bash
black --check .
```

Pair it with a pinned version so local runs and CI produce the same output.

### Editor integration

Prefer editor integrations that shell out to the installed `black` binary from your environment. That keeps editor formatting aligned with your pinned project version and `pyproject.toml`.

### Notebooks

Notebook formatting requires the `jupyter` extra or a workflow that explicitly supports notebook cells. Plain `black .` is not enough if your repo depends on `.ipynb` formatting.

## Common Pitfalls

- Black only uses one `pyproject.toml` per run. Nested package configs are not merged.
- If files are unexpectedly skipped, check `.gitignore`, `include`, `exclude`, and `extend-exclude` rules before assuming Black is broken.
- `--target-version` matters. It changes which syntax Black is allowed to emit.
- `--fast` disables the post-format safety check. Use the default safe mode unless you specifically need the speed tradeoff.
- `blackd` does not use `pyproject.toml`, so do not assume daemon-based formatting will honor repository config automatically.
- Formatting notebooks needs `black[jupyter]` or notebook-aware hooks.
- `--preview` and unstable features can change formatting output between releases. Do not enable them casually in shared CI unless the team intends to absorb churn.

## Version-Sensitive Notes

- Black's style is deliberately stable. The docs state that formatting changes for the stable style are only made in January releases, except for bug fixes and support for new Python syntax.
- `26.3.0` includes preview-style changes for long typed function parameter lists and for forcing parentheses around conditional expressions in typed lambdas.
- `26.3.1` is a patch release that fixes corruption for non-UTF-8 files and a lambda-with-comment formatting bug. If you are pinned to exact `26.3.0`, review diffs before moving to `26.3.1`.

## Official Sources

- Stable docs: `https://black.readthedocs.io/en/stable/`
- Usage and configuration: `https://black.readthedocs.io/en/stable/usage_and_configuration/the_basics.html`
- File collection and exclude behavior: `https://black.readthedocs.io/en/stable/usage_and_configuration/file_collection_and_discovery.html`
- Getting started and `pre-commit`: `https://black.readthedocs.io/en/stable/getting_started.html`
- Change log: `https://black.readthedocs.io/en/stable/change_log.html`
- PyPI: `https://pypi.org/project/black/`
- Repository: `https://github.com/psf/black`

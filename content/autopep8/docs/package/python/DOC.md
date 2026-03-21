---
name: package
description: "autopep8 package guide for formatting Python code to PEP 8 with the maintainer's CLI and module API"
metadata:
  languages: "python"
  versions: "2.3.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "autopep8,python,pep8,formatter,pycodestyle,pre-commit"
---

# autopep8 Python Package Guide

## Golden Rule

Use `autopep8` when you specifically want a `pycodestyle`-driven formatter for Python code. It is a formatter, not a linter, and some non-whitespace rewrites only happen when you opt into `--aggressive`.

## Install

Pin the version if your project or toolchain expects a specific formatter behavior:

```bash
python -m pip install "autopep8==2.3.2"
```

Upgrade in place when you want the latest published release:

```bash
python -m pip install --upgrade autopep8
```

`autopep8` requires `pycodestyle`; pip will install the dependency for you.

## Initialize And Setup

`autopep8` has no project bootstrap step and no authentication requirements. Setup usually means one of these:

1. Install the package into the environment that will run the formatter.
2. Add a project config file so repeated runs use the same options.
3. Optionally wire it into `pre-commit` or your editor.

Check the installed CLI:

```bash
autopep8 --version
autopep8 --help
```

## Core CLI Usage

Preview changes without modifying files:

```bash
autopep8 --diff path/to/file.py
```

Rewrite a file in place:

```bash
autopep8 --in-place path/to/file.py
```

Format a tree recursively. `--recursive` must be combined with `--in-place` or `--diff`:

```bash
autopep8 --recursive --in-place src/
```

Read from stdin by passing `-` as the file:

```bash
cat bad.py | autopep8 -
```

Limit fixes to specific codes:

```bash
autopep8 --select=E1,W1 --in-place path/to/file.py
```

Restrict formatting to a line range:

```bash
autopep8 --line-range 1 120 --in-place path/to/file.py
```

List fix codes supported by the tool:

```bash
autopep8 --list-fixes
```

## Aggressive And Experimental Modes

By default, `autopep8` only makes whitespace-safe changes. Important consequences from the maintainer docs:

- `E711` and `E712` are not fixed unless you pass `--aggressive`
- `E712` needs aggressiveness level 2, so pass `--aggressive --aggressive`
- deprecated-code `W6` fixes also require aggressive mode
- `--experimental` enables additional line-shortening behavior

Typical aggressive run:

```bash
autopep8 --in-place --aggressive --aggressive path/to/file.py
```

Use aggressive mode carefully on code with overloaded equality or code where boolean-comparison rewrites could be semantically sensitive.

## Use As A Module

The simplest programmatic API is `fix_code()`:

```python
import autopep8

fixed = autopep8.fix_code("x=       123\n")
print(fixed)
```

Pass options when you need behavior closer to the CLI:

```python
import autopep8

fixed = autopep8.fix_code(
    "print( 123 )\n",
    options={"ignore": ["E"]},
)
print(fixed)
```

This is useful when you want formatting inside another Python tool instead of shelling out to the CLI.

## Configuration

`autopep8` can read configuration from several places:

- global config: `$HOME/.config/pycodestyle` on Unix-like systems
- local project config: `setup.cfg`, `tox.ini`, `.pep8`, `.flake8`
- `pyproject.toml`

Supported INI sections are `pep8`, `pycodestyle`, and `flake8`.

If you use `pyproject.toml`, the section must be `[tool.autopep8]`, and the maintainer docs state that `pyproject.toml` takes precedence over the other config files.

Example:

```toml
[tool.autopep8]
max_line_length = 100
ignore = ["E501"]
in-place = true
recursive = true
aggressive = 2
```

You can also point the CLI at a specific global config file:

```bash
autopep8 --global-config /path/to/pep8.ini --in-place app.py
```

## pre-commit Integration

The maintainer repo publishes a `pre-commit` hook:

```yaml
repos:
  - repo: https://github.com/hhatto/autopep8
    rev: v2.3.2
    hooks:
      - id: autopep8
```

After adding it to `.pre-commit-config.yaml`, run:

```bash
pre-commit run --all-files
```

Pin `rev` to the formatter version you want reviewed and reproduced in CI.

## Common Pitfalls

- `autopep8` is not the same tool as `black`, `ruff format`, or `yapf`. Do not assume those tools' defaults or style rules apply here.
- `--recursive` does nothing useful on its own; pair it with `--in-place` or `--diff`.
- Default behavior is conservative. If a rewrite did not happen, check whether it needs `--aggressive`.
- The tool uses `pycodestyle` findings as its basis, so config choices like `max_line_length`, `ignore`, and `select` materially change what gets rewritten.
- `pyproject.toml` wins over `setup.cfg`, `tox.ini`, `.pep8`, and `.flake8`. Conflicting config files are a common source of surprise.
- `# autopep8: off` / `# autopep8: on` and `# fmt: off` / `# fmt: on` can disable formatting for sections that should not be touched.
- If you need CI to fail when formatting would change files, use `--diff --exit-code` and treat exit code `2` as "differences found".
- If an older environment raises `pkg_resources.DistributionNotFound`, the maintainer docs recommend upgrading `setuptools`.

## Version-Sensitive Notes For 2.3.2

- The version used here `2.3.2` matches the current PyPI release page for `autopep8` as of March 12, 2026.
- PyPI lists `Requires: Python >=3.9`, so do not plan new usage around Python 3.8 or older.
- The official maintainer documentation is the repository README rendered on GitHub and mirrored on PyPI, not a separate docs site.
- The project supports `pyproject.toml` configuration, and that precedence rule is important when migrating older repos from `setup.cfg` or `.flake8`.

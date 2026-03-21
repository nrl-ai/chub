---
name: package
description: "YAPF Python formatter guide for configuring styles, formatting files, and using the library API"
metadata:
  languages: "python"
  versions: "0.43.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "yapf,formatter,python,code-style,linting,clang-format"
---

# YAPF Python Package Guide

## Golden Rule

Use `yapf` when the repository already standardizes on YAPF or needs YAPF-compatible formatting behavior. Put the style in checked-in project config, use diff mode in CI, and do not assume YAPF fully supports newer Python 3.12 syntax.

## Install

Pin the version your project expects:

```bash
python -m pip install "yapf==0.43.0"
```

Common alternatives:

```bash
uv add "yapf==0.43.0"
poetry add "yapf==0.43.0"
```

If you only need the formatter as a checked-in dev tool, keep it in your dev dependency group rather than your runtime dependencies.

## Setup

YAPF reads style settings from these locations in order:

1. `--style` on the command line
2. `[style]` in `.style.yapf`
3. `[yapf]` in `setup.cfg`
4. `[tool.yapf]` in `pyproject.toml`
5. `[style]` in `~/.config/yapf/style`

If none of those exist, it falls back to the built-in `pep8` style.

Recommended `pyproject.toml` setup:

```toml
[tool.yapf]
based_on_style = "pep8"
column_limit = 88
indent_width = 4
split_before_logical_operator = true

[tool.yapfignore]
ignore_patterns = [
  "build/**/*.py",
  ".venv/**/*.py",
]
```

Equivalent `.style.yapf` setup:

```ini
[style]
based_on_style = pep8
column_limit = 88
indent_width = 4
split_before_logical_operator = true
```

Predefined styles are `pep8`, `google`, `yapf`, and `facebook`.

## Core CLI Usage

Print formatted output to stdout:

```bash
yapf path/to/file.py
```

Rewrite a file in place:

```bash
yapf -i path/to/file.py
```

Format a directory recursively and print diffs instead of editing files:

```bash
yapf -d -r src tests
```

Format multiple files in parallel and write changes in place:

```bash
yapf -i -r -p src tests
```

Format only part of a file. Line numbers are 1-based:

```bash
yapf -i -l 10-40 app/views.py
```

Run a CI-style check:

```bash
yapf -d -r .
```

With `--diff`, YAPF returns zero only when no reformatting is needed. That makes it suitable for CI enforcement.

## Format Only Changed Lines

Use `yapf-diff` when you want to reformat lines touched by a patch instead of whole files:

```bash
git diff -U0 --no-color --relative HEAD^ | yapf-diff -i
```

If the diff paths are not relative to your current working directory, use `-p` or run the command from the repository root so the filenames resolve correctly.

## Library API

Format a code string:

```python
from yapf.yapflib.yapf_api import FormatCode

source = "def add ( a, b ):\n    return a+b\n"
formatted, changed = FormatCode(source, style_config="pep8")

print(formatted)
print(changed)
```

Format only selected line ranges:

```python
from yapf.yapflib.yapf_api import FormatCode

source = "def f( ):\n a=1\n b = 2\n return a==b\n"
formatted, changed = FormatCode(source, lines=[(1, 1), (2, 3)])
```

Format a file:

```python
from yapf.yapflib.yapf_api import FormatFile

formatted, encoding, changed = FormatFile("app.py", style_config="pyproject.toml")
```

Write changes back to the file:

```python
from yapf.yapflib.yapf_api import FormatFile

_, encoding, changed = FormatFile(
    "app.py",
    style_config="pyproject.toml",
    in_place=True,
)
```

Use `print_diff=True` with `FormatCode(...)` when you need a unified diff instead of the rewritten source.

## Config And Auth

YAPF is a local formatter. It does not require an API key, account, or network auth.

The main configuration choices agents need to get right are:

- whether the repo already has a checked-in YAPF style file
- whether to use a named base style such as `pep8` or `google`
- which files to exclude through `.yapfignore` or `[tool.yapfignore]`
- whether the task should print diffs, edit files in place, or format only selected lines

## Common Pitfalls

- Without `-i`, the `yapf` CLI prints formatted code to stdout and does not modify the file.
- The config section names are different by file type: `[style]` in `.style.yapf`, `[yapf]` in `setup.cfg`, and `[tool.yapf]` in `pyproject.toml`.
- Exclude patterns in `pyproject.toml` belong under `[tool.yapfignore]`, not `[tool.yapf]`.
- The default style is `pep8` if no local config is found, so agents should not assume Black-like output or project-specific settings.
- `yapf-diff` trusts the filenames in the incoming diff. If paths are wrong relative to the current directory, it will not update the intended files.
- Line ranges passed through `--lines` or `lines=[...]` are 1-based.
- YAPF 0.43.0 still documents unsupported Python 3.12 features for PEP 695 type parameter syntax and PEP 701 f-string syntax.

## Version-Sensitive Notes

- As of March 12, 2026, PyPI still lists `0.43.0` as the latest release. It was uploaded on November 14, 2024.
- The upstream changelog in the GitHub repo is stale relative to PyPI release history. It documents changes through `0.40.2` and still labels `0.41.0` as unreleased, so confirm release status from PyPI when pinning versions.
- `0.40.2` removed the verification module and removed the public `verify` parameter from the API. Older examples using `verify=` are outdated.
- `0.40.0` and `0.33.0` were yanked on PyPI. Do not pin those versions.
- `0.33.0` added default `pyproject.toml` support, and `0.40.2` switched the build to `pyproject.toml`, so current `0.43.0` workflows should prefer `pyproject.toml` over legacy config files when the repo already uses it.

## Official Sources

- Maintainer docs and usage reference: `https://github.com/google/yapf`
- Maintainer changelog: `https://raw.githubusercontent.com/google/yapf/main/CHANGELOG.md`
- Package registry and release history: `https://pypi.org/project/yapf/`

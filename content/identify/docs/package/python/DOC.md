---
name: package
description: "identify package guide for classifying files into tags in Python tooling"
metadata:
  languages: "python"
  versions: "2.6.17"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "identify,python,files,pre-commit,automation"
---

# identify Python Package Guide

`identify` is a small utility package for classifying files into tags such as language, text or binary, executable state, and filesystem type. Use it when your tooling needs `pre-commit`-style file typing inside Python code.

It is a local helper library, not a network client:

- Environment variables: none
- Authentication: none
- Client initialization: none

## Install

Pin the package version your project expects:

```bash
python -m pip install "identify==2.6.17"
```

Common alternatives:

```bash
uv add "identify==2.6.17"
poetry add "identify==2.6.17"
```

Import the package like this:

```python
from identify import identify
```

## Core API

The two helpers you will usually need are:

- `identify.tags_from_path(path)` for a real filesystem path
- `identify.tags_from_filename(filename)` when you only know the name

Both return a Python `set[str]` of tags.

## Tag A Real File Or Directory

Use `tags_from_path()` when the path already exists on disk.

```python
from identify import identify

tags = identify.tags_from_path("src/app.py")

if {"file", "text", "python"} <= tags:
    print("run Python-specific checks")
```

This is the right helper when file mode, symlinks, directories, or shebang-based detection matter.

## Tag A Filename Before Writing It

Use `tags_from_filename()` when you only have a file name, such as for generated outputs or archive entries.

```python
from identify import identify

tags = identify.tags_from_filename("generated_config.py")

if "python" in tags:
    print("generate Python-oriented content")
```

This only uses the filename. It cannot inspect file contents, executable bits, or symlink state.

## Filter Files In A Repository Walk

`identify` works well as a lightweight filter before linting, formatting, or custom analysis.

```python
from pathlib import Path

from identify import identify


python_files: list[Path] = []

for path in Path(".").rglob("*"):
    tags = identify.tags_from_path(str(path))
    if {"file", "text", "python"} <= tags:
        python_files.append(path)

for path in python_files:
    print(path)
```

If your tool only wants regular files, check for the `"file"` tag explicitly instead of assuming everything returned from a directory walk is a normal file.

## Use Tags As Capability Checks

Treat the returned set as a collection of capabilities or attributes.

```python
from identify import identify

tags = identify.tags_from_path("scripts/release")

if "executable" in tags:
    print("this path can be run directly")

if "text" in tags:
    print("safe to process as text")
```

Membership tests such as `"python" in tags` or subset checks such as `{"file", "text"} <= tags` are the practical way to consume the API.

## Common Pitfalls

- Use `tags_from_path()` for files that already exist and `tags_from_filename()` for names only.
- Do not expect `tags_from_filename()` to infer executable or shebang-derived tags.
- The return value is a set. Do not rely on tag ordering.
- When working with `pathlib.Path`, pass `str(path)` if the surrounding code expects plain string paths.
- Check for `"file"` when your logic should exclude directories, symlinks, or other filesystem entries.

## When To Reach For identify

Use `identify` when you need file classification as a library dependency inside Python automation. If you only need a fixed glob like `*.py`, plain path matching is simpler. `identify` becomes useful when filename rules are not enough and your tool needs the same kind of path tagging that `pre-commit` uses.

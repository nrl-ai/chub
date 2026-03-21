---
name: package
description: "tabulate Python package for formatting iterables and tabular data as plain-text, Markdown, HTML, and LaTeX tables"
metadata:
  languages: "python"
  versions: "0.10.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tabulate,tables,markdown,html,cli,formatting"
---

# tabulate Python Package Guide

## Golden Rule

## Install

Pin the version your project expects:

```bash
python -m pip install "tabulate==0.10.0"
```

Common alternatives:

```bash
uv add "tabulate==0.10.0"
poetry add "tabulate==0.10.0"
```

If the output includes full-width characters, install the optional wide-character support:

```bash
python -m pip install "tabulate[widechars]==0.10.0"
```

On Unix-like systems and Windows, the upstream docs also describe a library-only install path that skips the CLI wrapper:

```bash
TABULATE_INSTALL=lib-only python -m pip install "tabulate==0.10.0"
```

## Initialize And Basic Setup

There is no client object, environment configuration, or authentication step.

```python
from tabulate import tabulate
```

The main API is:

```python
tabulate(tabular_data, headers=(), tablefmt="simple", **formatting_options)
```

`tabular_data` can be a list of rows, a list of dictionaries, a dictionary of iterables, a dataclass sequence, a NumPy array, or a pandas `DataFrame`.

## Core Usage

### Format a list of rows

```python
from tabulate import tabulate

rows = [
    ["Ada", 36, "admin"],
    ["Linus", 55, "maintainer"],
]

print(tabulate(rows, headers=["name", "age", "role"], tablefmt="github"))
```

Use an explicit format such as `github`, `pipe`, `grid`, `rounded_grid`, `html`, or `latex` when your downstream consumer expects a specific shape.

### Format dictionaries by key

```python
from tabulate import tabulate

rows = [
    {"name": "Ada", "age": 36, "active": True},
    {"name": "Linus", "age": 55, "active": False},
]

print(tabulate(rows, headers="keys", tablefmt="github"))
```

`headers="keys"` is the simplest path for list-of-dicts input. If you need a stable column order across varying dictionaries, normalize the keys before calling `tabulate`.

### Control numeric formatting and alignment

By default, `tabulate` tries to detect and align numbers. Override that when strings should stay strings.

```python
from tabulate import tabulate

rows = [
    ["00123", 3.14159, 12000],
    ["00007", 9.5, 42],
]

print(
    tabulate(
        rows,
        headers=["code", "ratio", "count"],
        tablefmt="github",
        disable_numparse=[0],
        floatfmt=".2f",
        intfmt=",",
        numalign="right",
        stralign="left",
    )
)
```

Useful options:

- `disable_numparse=True` disables numeric parsing for every column
- `disable_numparse=[0, 2]` disables parsing for specific columns
- `floatfmt` controls float precision
- `intfmt` adds integer separators such as `,`
- `numalign` and `stralign` control alignment
- `colalign` sets per-column alignment overrides

### Handle missing values and long text

```python
from tabulate import tabulate

rows = [
    ["ok", "short note"],
    [None, "A much longer note that should wrap into multiple lines for narrow output."],
]

print(
    tabulate(
        rows,
        headers=["status", "note"],
        tablefmt="grid",
        missingval="",
        maxcolwidths=[None, 24],
    )
)
```

Use `missingval` to control how `None` renders. Use `maxcolwidths` when you need predictable wrapping instead of one very wide cell.

### Use with pandas DataFrames

`tabulate` accepts a pandas `DataFrame` directly:

```python
from tabulate import tabulate
import pandas as pd

df = pd.DataFrame(
    [
        {"name": "Ada", "score": 9.75},
        {"name": "Linus", "score": 8.25},
    ]
)

print(tabulate(df, headers="keys", tablefmt="psql", showindex=False))
```

Set `showindex=True` or a custom iterable when the index needs to appear in the rendered table.

### Command-line usage

The package installs a `tabulate` command that reads tabular input files and prints a formatted table. Example:

```bash
tabulate -1 -f github data.tsv
```

This is useful for quick local conversions, but most agent workflows should call the Python API so formatting is explicit in code.

## Configuration Notes

- No auth: `tabulate` is a pure formatting library and does not use API keys, credentials, or network configuration.
- No global initialization is required for normal usage.
- Install `tabulate[widechars]` when tables include CJK full-width characters and column width must stay visually correct.
- Choose `tablefmt` explicitly for generated Markdown, HTML, LaTeX, or terminal output. Relying on the default `simple` format can cause avoidable diffs across tools.

## Common Pitfalls

- Numeric-looking strings such as ZIP codes, IDs, or zero-padded codes may be auto-parsed and right-aligned. Use `disable_numparse` for those columns.
- `headers="keys"` follows the input structure. If your dictionaries do not all share the same keys or order, normalize them before rendering.
- DataFrame index handling is easy to miss. Use `showindex=False` when you do not want the index column in the output.
- Wide-character alignment is not automatic unless the optional `wcwidth` dependency is installed through the `widechars` extra.
- `tabulate` renders tables; it does not paginate, truncate huge datasets efficiently, or stream rows incrementally. Slice or aggregate large results before formatting.
- Plain-text table formats are for display, not round-trip parsing. If another tool needs machine-readable output, emit CSV/JSON separately and use `tabulate` only for presentation.

## Version-Sensitive Notes For 0.10.0

- PyPI currently lists `0.10.0` as the latest release, published on March 4, 2026.
- The published `0.10.0` package metadata on PyPI says `Requires-Python >=3.9`.
- The current `master` branch `pyproject.toml` in the maintainer repo already says `>=3.10`. Treat that as repo-head drift, not as the requirement for the published `0.10.0` wheel and sdist.
- The maintainer changelog for `0.10.0` calls out new alignment controls (`headersglobalalign`, `headersalign`, `colglobalalign`) and the `colon_grid` table format.
- The same `0.10.0` changelog entry notes dropped support for Python 3.7 and 3.8, so older runtime targets need an earlier package version.

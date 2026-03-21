---
name: package
description: "openpyxl package guide for Python Excel workbooks: install, load/save flags, streaming modes, formulas, styles, and common pitfalls"
metadata:
  languages: "python"
  versions: "3.1.5"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
---

# openpyxl Python Package Guide

## Golden Rule

Use `openpyxl` for Office Open XML Excel workbooks in Python, typically `.xlsx` and `.xlsm`. Treat it as a workbook editor, not an Excel calculation engine: write formulas as strings, read cached results with `data_only=True` only when the file has already been recalculated by Excel or LibreOffice, and use the optimized read/write modes for large files.

## Install

Pin the version your project expects:

```bash
python -m pip install "openpyxl==3.1.5"
```

Common alternatives:

```bash
uv add "openpyxl==3.1.5"
poetry add "openpyxl==3.1.5"
```

Optional companion packages called out by the official sources:

```bash
python -m pip install lxml pillow defusedxml
```

Use them only when needed:

- `lxml`: faster XML writing, especially for large-file creation workloads
- `pillow`: required if you need image support
- `defusedxml`: recommended when processing untrusted workbooks because the project does not guard against quadratic blowup or billion laughs XML attacks by default

## Initialize And Mental Model

`openpyxl` revolves around three core objects:

- `Workbook`: the Excel file
- `Worksheet`: a sheet inside the workbook
- `Cell`: an addressed value such as `A1`

Basic workbook creation:

```python
from openpyxl import Workbook

wb = Workbook()
ws = wb.active
ws.title = "Report"

ws["A1"] = "region"
ws["B1"] = "sales"
ws.append(["west", 120])
ws.append(["east", 95])

wb.save("report.xlsx")
```

Important save behavior: saving to an existing filename overwrites the file without warning.

## Core Usage

### Load an existing workbook

Use `load_workbook()` when you need to inspect or modify an existing file:

```python
from openpyxl import load_workbook

wb = load_workbook("report.xlsx")
ws = wb["Report"]

print(ws["A2"].value)
```

Useful flags:

```python
from openpyxl import load_workbook

wb = load_workbook(
    "report.xlsm",
    read_only=False,
    data_only=False,
    keep_vba=True,
    keep_links=True,
    rich_text=False,
)
```

Use these flags deliberately:

- `read_only=True`: stream rows from large workbooks with low memory usage
- `data_only=True`: read cached formula results instead of formula strings
- `keep_vba=True`: preserve existing VBA content in macro-enabled files when saving
- `keep_links=False`: drop external-link preservation when you do not need it
- `rich_text=True`: preserve rich text runs when that formatting matters

### Iterate through rows and columns

```python
from openpyxl import load_workbook

wb = load_workbook("report.xlsx")
ws = wb.active

for row in ws.iter_rows(min_row=2, values_only=True):
    region, sales = row
    print(region, sales)
```

`values_only=True` is usually the right default when you only need Python values rather than `Cell` objects.

### Write tabular data

```python
from openpyxl import Workbook

rows = [
    ("name", "team", "score"),
    ("Ava", "blue", 10),
    ("Noah", "green", 14),
]

wb = Workbook()
ws = wb.active

for row in rows:
    ws.append(row)

wb.save("scores.xlsx")
```

Use `append()` for row-oriented writes. Use direct cell assignment such as `ws["C2"] = 42` when you need random access updates.

### Read formulas vs cached values

Write formulas as strings:

```python
from openpyxl import Workbook

wb = Workbook()
ws = wb.active

ws["A1"] = 10
ws["A2"] = 20
ws["A3"] = "=SUM(A1:A2)"

wb.save("formula.xlsx")
```

If you reopen the file with `data_only=False`, you read the formula text. If you reopen with `data_only=True`, you get the last cached value stored by spreadsheet software. `openpyxl` itself does not calculate formulas.

The official docs also note that function names must use the English Excel names and argument separators must use commas.

### Apply simple styles

```python
from openpyxl import Workbook
from openpyxl.styles import Font, PatternFill

wb = Workbook()
ws = wb.active
ws.append(["name", "team", "score"])

header_font = Font(bold=True)
header_fill = PatternFill("solid", fgColor="D9EAF7")

for cell in ws[1]:
    cell.font = header_font
    cell.fill = header_fill

wb.save("styled.xlsx")
```

Style objects are immutable once assigned. Create a new style object rather than mutating one already attached to cells.

### Add an image

Image support requires `Pillow`:

```python
from openpyxl import Workbook
from openpyxl.drawing.image import Image

wb = Workbook()
ws = wb.active

img = Image("logo.png")
ws.add_image(img, "D2")

wb.save("with-image.xlsx")
```

## Large Files And Optimized Modes

### Read-only mode

Use `read_only=True` when loading large workbooks:

```python
from openpyxl import load_workbook

wb = load_workbook("large.xlsx", read_only=True)
ws = wb.active

for row in ws.rows:
    values = [cell.value for cell in row]
    print(values)

wb.close()
```

Notes from the official optimized-modes docs:

- read-only worksheets use lazy loading and return `ReadOnlyCell` objects
- the workbook must be explicitly closed
- some worksheet methods are unavailable in read-only mode
- if a producer writes incorrect worksheet dimensions, check `ws.calculate_dimension()` and call `ws.reset_dimensions()` when needed

### Write-only mode

Use `Workbook(write_only=True)` to stream output for very large exports:

```python
from openpyxl import Workbook

wb = Workbook(write_only=True)
ws = wb.create_sheet(title="Export")

for i in range(1, 100001):
    ws.append([i, f"row-{i}"])

wb.save("export.xlsx")
```

Write-only mode is append-only, does not expose random cell access, and the official docs warn that a write-only workbook can be saved only once.

## Configuration And File-Type Notes

`openpyxl` has no package-level authentication or environment configuration. Almost all "configuration" is per-call behavior:

- load-time flags such as `read_only`, `data_only`, `keep_vba`, `keep_links`, and `rich_text`
- workbook mode such as normal vs `write_only=True`
- file extension and whether the workbook contains macros or templates

Macro/template caveats from the official tutorial:

- keep macros with `keep_vba=True` and save with a macro-capable extension such as `.xlsm`
- use `wb.template = True` when intentionally producing a template and save with the matching template extension
- do not mix workbook content and file extension types; Excel may refuse to open the file

## Common Pitfalls

- `openpyxl` does not calculate formulas. If you need computed results, recalculate the workbook in Excel or LibreOffice before reading it with `data_only=True`.
- `data_only=True` can return `None` or stale values when the workbook has never been recalculated since the formula changed.
- Saving overwrites existing files without warning.
- `copy_worksheet()` does not copy images or charts, and you cannot copy a worksheet between different workbooks.
- VBA preservation is not the same as VBA editing. `keep_vba=True` is for preserving existing macro content while manipulating the workbook.
- Read-only mode is not a drop-in replacement for the normal API. Some iteration helpers and random-access operations are intentionally unavailable.
- Write-only mode is append-only and one-shot. Build rows in order and save once.
- Be careful with untrusted files. The project documentation explicitly recommends installing `defusedxml`.

## Version-Sensitive Notes

- The official docs build is behind the PyPI release: `stable` is `3.1.3`, the `3.1` branch is `3.1.4`, and PyPI is `3.1.5`.
- When examples disagree across community posts, prefer the official 3.1 docs series plus PyPI metadata over older blog posts.
- If you depend on behavior around macros, rich text preservation, or optimized modes, verify against the official tutorial and optimized-modes pages rather than generic snippets.

## Official Sources

- PyPI package page: `https://pypi.org/project/openpyxl/`
- Official docs root (`3.1`): `https://openpyxl.readthedocs.io/en/3.1/`
- Official stable docs root: `https://openpyxl.readthedocs.io/en/stable/`
- Tutorial: `https://openpyxl.readthedocs.io/en/3.1/tutorial.html`
- Optimized modes: `https://openpyxl.readthedocs.io/en/3.1/optimized.html`
- Formula reference: `https://openpyxl.readthedocs.io/en/3.1/simple_formulae.html`

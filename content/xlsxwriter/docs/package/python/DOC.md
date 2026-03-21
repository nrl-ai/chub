---
name: package
description: "XlsxWriter Python package guide for creating Excel .xlsx files with worksheets, formats, charts, and dataframe exports"
metadata:
  languages: "python"
  versions: "3.2.9"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
---

# XlsxWriter Python Package Guide

## Golden Rule

Use `xlsxwriter` when you need to generate new Excel `.xlsx` files from Python. It is a write-focused library: create workbooks, worksheets, formats, charts, tables, and dataframe exports, then close the workbook exactly once. It does not read or modify existing Excel files.

## Install

Pin the version your project expects:

```bash
python -m pip install "XlsxWriter==3.2.9"
```

Common alternatives:

```bash
uv add "XlsxWriter==3.2.9"
poetry add "XlsxWriter==3.2.9"
```

## Basic Workbook Lifecycle

Create a workbook, add one or more worksheets, write data, then call `close()` to finalize the file:

```python
import xlsxwriter

workbook = xlsxwriter.Workbook("hello.xlsx")

try:
    worksheet = workbook.add_worksheet("Summary")
    worksheet.write("A1", "Hello")
    worksheet.write("B1", 123)
finally:
    workbook.close()
```

Notes:

- `close()` writes the final ZIP container and can raise file-creation errors if the destination is open in Excel or the path is invalid.
- XlsxWriter supports both zero-based row/column indexes and Excel `A1` notation.
- Use `.xlsx` output paths. XlsxWriter is not an `.xls` writer.

## Core Usage

### Write typed values

`write()` auto-detects many Python values, but the typed methods are clearer when type handling matters:

```python
from datetime import datetime
import xlsxwriter

workbook = xlsxwriter.Workbook(
    "report.xlsx",
    {
        "default_date_format": "yyyy-mm-dd",
        "remove_timezone": True,
    },
)

try:
    worksheet = workbook.add_worksheet("Summary")
    header = workbook.add_format({"bold": True, "bg_color": "#D9E2F3"})
    money = workbook.add_format({"num_format": "$#,##0.00"})
    date_fmt = workbook.add_format({"num_format": "yyyy-mm-dd hh:mm"})

    worksheet.write_row("A1", ["Item", "Amount", "Created At"], header)

    rows = [
        ("Books", 42.50, datetime(2026, 3, 12, 9, 30)),
        ("Games", 19.99, datetime(2026, 3, 12, 11, 15)),
        ("Snacks", 8.25, datetime(2026, 3, 12, 13, 5)),
    ]

    for row_idx, (item, amount, created_at) in enumerate(rows, start=1):
        worksheet.write_string(row_idx, 0, item)
        worksheet.write_number(row_idx, 1, amount, money)
        worksheet.write_datetime(row_idx, 2, created_at, date_fmt)

    worksheet.write_formula("B5", "=SUM(B2:B4)", money)
    worksheet.autofit()
finally:
    workbook.close()
```

### Add formatting, charts, and tables

```python
import xlsxwriter

workbook = xlsxwriter.Workbook("sales.xlsx")

try:
    worksheet = workbook.add_worksheet("Sales")

    worksheet.write_row("A1", ["Month", "Revenue"])
    worksheet.write_column("A2", ["Jan", "Feb", "Mar"])
    worksheet.write_column("B2", [12000, 15000, 17000])

    worksheet.add_table(
        "A1:B4",
        {
            "style": "Table Style Medium 2",
            "columns": [
                {"header": "Month"},
                {"header": "Revenue"},
            ],
        },
    )

    chart = workbook.add_chart({"type": "column"})
    chart.add_series(
        {
            "name": "Revenue",
            "categories": "=Sales!$A$2:$A$4",
            "values": "=Sales!$B$2:$B$4",
        }
    )
    chart.set_title({"name": "Quarter Revenue"})

    worksheet.insert_chart("D2", chart)
finally:
    workbook.close()
```

Useful worksheet APIs agents commonly need:

- `set_column()` and `set_row()` for sizing and default formats
- `freeze_panes()` for sticky headers
- `autofilter()` and `add_table()` for structured sheet output
- `write_url()`, `insert_image()`, `data_validation()`, and `conditional_format()` for richer reports

## DataFrame Exports

### Pandas

Use the `xlsxwriter` engine when you want pandas output plus workbook-level formatting:

```python
import pandas as pd

df = pd.DataFrame(
    [
        {"name": "Ada", "score": 98},
        {"name": "Linus", "score": 91},
    ]
)

with pd.ExcelWriter(
    "scores.xlsx",
    engine="xlsxwriter",
    engine_kwargs={"options": {"strings_to_numbers": True}},
) as writer:
    df.to_excel(writer, sheet_name="Scores", index=False)

    workbook = writer.book
    worksheet = writer.sheets["Scores"]
    score_fmt = workbook.add_format({"num_format": "0"})

    worksheet.set_column("A:A", 18)
    worksheet.set_column("B:B", 10, score_fmt)
    worksheet.autofilter(0, 0, len(df), len(df.columns) - 1)
```

### Polars

The upstream docs also include dedicated Polars integration guidance through `DataFrame.write_excel()`. Use that when the project already depends on Polars and you want table-style exports without converting through pandas first.

## Configuration Notes

XlsxWriter has no network auth or API credentials. Configuration is local to `Workbook(...)` options and worksheet/workbook methods.

Constructor options that matter in real projects:

- `constant_memory=True`: reduce memory usage for large exports by flushing rows as you go; write in row order and expect some features to be more limited than in normal mode.
- `in_memory=True`: build the file in memory instead of temp files.
- `tmpdir="/path"`: control where temporary files are written if the default temp directory is unsuitable.
- `default_date_format="yyyy-mm-dd"`: set a default Excel number format for date writes.
- `remove_timezone=True`: strip timezone info from datetimes before writing, which avoids timezone-related Excel write issues.
- `strings_to_numbers`, `strings_to_formulas`, `strings_to_urls`: control automatic coercion when writing plain strings.

## Exceptions And File Handling

XlsxWriter surfaces library-specific exceptions for workbook creation and integrity problems. The ones agents most often need to handle are:

- `FileCreateError`: output file cannot be created, often because the file is already open in Excel
- `DuplicateWorksheetName`: duplicate worksheet title in a workbook
- `InvalidWorksheetName`: worksheet title exceeds Excel rules
- `OverlappingRange`: merged ranges or tables overlap

Example:

```python
import xlsxwriter
from xlsxwriter.exceptions import FileCreateError

workbook = xlsxwriter.Workbook("report.xlsx")

try:
    worksheet = workbook.add_worksheet()
    worksheet.write("A1", "ok")
    workbook.close()
except FileCreateError as exc:
    raise RuntimeError(
        "Could not write report.xlsx. Close the file in Excel and try again."
    ) from exc
```

## Common Pitfalls

- XlsxWriter creates new files; it does not load or edit existing workbooks. Use another library if you must modify an existing `.xlsx`.
- Forgetting `close()` leaves the workbook incomplete or corrupted.
- Excel stores dates as numbers. Use `write_datetime()` plus a date format instead of writing raw datetime strings if you want Excel date behavior.
- Worksheet names must be unique and must follow Excel naming rules.
- Large exports can consume significant memory in default mode; switch to `constant_memory` or a streaming dataframe workflow when row counts are high.
- Formula strings beginning with `=` are treated as formulas. If a value should stay plain text, write it explicitly with `write_string()`.
- If an output file already exists and is open in Excel, finalization commonly fails on `close()`, not at workbook creation time.

## Version-Sensitive Notes For 3.2.9

- PyPI lists `XlsxWriter 3.2.9` as the current package version as of 2026-03-12.
- The upstream change log for `3.2.9` notes a typing-related packaging change: `py.typed` was removed because the package uses `.pyi` stubs. If editor or CI typing behavior changes after upgrading, verify the tool reads the installed stubs correctly.
- The current PyPI metadata requires Python `>=3.8`. If you are curating or generating code for older runtimes, this version is out of range.

## Official Sources

- Docs root: https://xlsxwriter.readthedocs.io/
- Tutorial and examples: https://xlsxwriter.readthedocs.io/tutorial01.html
- Workbook and constructor options: https://xlsxwriter.readthedocs.io/workbook.html
- Exceptions: https://xlsxwriter.readthedocs.io/exceptions.html
- Pandas integration: https://xlsxwriter.readthedocs.io/working_with_pandas.html
- Polars integration: https://xlsxwriter.readthedocs.io/working_with_polars.html
- Change log: https://xlsxwriter.readthedocs.io/changes.html
- PyPI metadata: https://pypi.org/project/XlsxWriter/

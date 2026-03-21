---
name: package
description: "Camelot Python package for extracting tables from text-based PDFs into pandas DataFrames"
metadata:
  languages: "python"
  versions: "1.0.9"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "camelot,pdf,table-extraction,pandas,dataframe,etl"
---

# Camelot Python Package Guide

## Golden Rule

Use `camelot-py` only for text-based PDFs, not scanned image PDFs. Pick the parsing flavor deliberately: `lattice` for tables with visible ruling lines, `stream` for whitespace-aligned tables, and `network` or `hybrid` only when the default approaches need more control.

## Install

Pin the package version your project expects:

```bash
python -m pip install "camelot-py==1.0.9"
```

Common alternatives:

```bash
uv add "camelot-py==1.0.9"
poetry add "camelot-py==1.0.9"
conda install -c conda-forge camelot-py
```

Optional extras from upstream package metadata:

```bash
python -m pip install "camelot-py[plot]==1.0.9"
python -m pip install "camelot-py[ghostscript]==1.0.9"
```

Install notes:

- `1.0.x` uses `pdfium` via `pypdfium2` as the default image backend for `lattice`, so plain `pip install camelot-py` is usually enough for the default path.
- The Read the Docs install page still shows `camelot-py[base]`; for `1.0.9`, the published package metadata already includes the core runtime dependencies directly.
- If you switch to `backend="ghostscript"`, install Ghostscript itself with your system package manager and the Python `ghostscript` extra.

## Setup And Initialization

Camelot is a local parsing library. There is no API key or service authentication step.

Typical import and first read:

```python
import camelot

tables = camelot.read_pdf("report.pdf")
print(tables.n)
```

Important defaults from the official API:

- `pages="1"` by default, so it will only parse the first page unless you ask for more
- `flavor="lattice"` by default
- `parallel=False` by default
- `filepath` can be a local path, a `Path`, a file-like object, or a URL

## Core Usage

### Extract a ruled table with the default `lattice` flavor

```python
import camelot

tables = camelot.read_pdf(
    "report.pdf",
    pages="1,2",
    flavor="lattice",
)

table = tables[0]
print(table.parsing_report)
print(table.df.head())
table.df.to_csv("report-page-1-table-1.csv", index=False)
```

Use `table.parsing_report` as a quick quality check before trusting the output.

### Extract whitespace-aligned tables with `stream`

```python
import camelot

tables = camelot.read_pdf(
    "financials.pdf",
    pages="all",
    flavor="stream",
    table_areas=["43,535,555,120"],
    columns=["72,95,209,327,442,529"],
    split_text=True,
)

df = tables[0].df
```

`table_areas`, `columns`, `split_text`, `edge_tol`, `row_tol`, and `column_tol` are the knobs you will use most when `stream` merges or splits cells incorrectly.

### Export all tables in one pass

```python
import camelot

tables = camelot.read_pdf("report.pdf", pages="1-5")
tables.export("tables.csv", f="csv", compress=True)
```

Camelot can export `csv`, `json`, `excel`, `html`, `markdown`, and `sqlite`.

### Read encrypted PDFs

```python
import camelot

tables = camelot.read_pdf(
    "protected.pdf",
    password="userpass",
    pages="all",
)
```

If decryption fails, verify the password first before changing extraction settings.

### Use visual debugging when extraction is off

Install the plotting extra first:

```bash
python -m pip install "camelot-py[plot]==1.0.9"
```

Then inspect how Camelot sees the page:

```python
import camelot

tables = camelot.read_pdf("edge_tol.pdf", flavor="stream")
camelot.plot(tables[0], kind="contour").show()
```

This is often the fastest way to tune `table_areas`, `columns`, `edge_tol`, or `process_background`.

## Configuration Notes

- There is no global config file that agents need to manage. Most behavior is controlled per call to `camelot.read_pdf(...)`.
- `layout_kwargs` passes `pdfminer.six` `LAParams` options through to the text layout stage. Use it when PDF text grouping is wrong.
- For `lattice`, `backend` defaults to `"pdfium"` and `use_fallback=True`. You can switch to `"ghostscript"` or provide a custom conversion backend object if the image conversion step is the problem.
- `parallel=True` uses all available CPU cores for page processing. It can speed up long runs, but it also increases memory pressure.

## Command-Line Interface

The package installs a `camelot` CLI alongside the Python API. Start with:

```bash
camelot --help
```

The CLI exposes flavor-specific subcommands such as `lattice`, `stream`, `network`, and `hybrid`. Use it when you just need extracted files and do not need to inspect `Table.df` or `parsing_report` programmatically.

## Common Pitfalls

- Camelot does not work on scanned or image-only PDFs. If you cannot select text in the PDF viewer, use OCR first or use a different tool.
- Agents often forget that only page 1 is parsed by default. Set `pages="all"` or an explicit range.
- `lattice` is not a universal default. For borderless tables, switch to `flavor="stream"` early instead of over-tuning `lattice`.
- `stream` and `network` parsing are sensitive to `columns`, `table_areas`, and text layout. Use `camelot.plot()` to debug rather than guessing coordinates blindly.
- Large multi-page PDFs can consume a lot of RAM. The project FAQ recommends chunking page ranges and exporting each chunk incrementally.
- When using `ghostscript` on macOS, the docs call out that the `libgs` library may need to be symlinked into `~/lib` if discovery fails.
- `tables.export("out.csv", f="csv")` writes page and table suffixes into filenames. If you expect one single file artifact, set `compress=True` or export individual tables yourself.

## Version-Sensitive Notes For 1.0.9

- `camelot-py 1.0.9` supports Python `3.8` through `3.13` in the upstream project metadata.
- Since `v1.0.0`, `pdfium` replaced Ghostscript as the default image conversion backend. Older blog posts that assume Ghostscript is mandatory are stale.
- The official docs root `https://camelot-py.readthedocs.io/` and the source URL `https://camelot-py.readthedocs.io/en/latest/` currently surface `v1.0.0` pages in some places, while the current `1.0.9` docs are under `https://camelot-py.readthedocs.io/en/master/`. Use the `master` docs when checking `1.0.9` behavior.
- The current parser flavors are `lattice`, `stream`, `network`, and `hybrid`. Older examples that mention only `lattice` and `stream` are incomplete for the current release.

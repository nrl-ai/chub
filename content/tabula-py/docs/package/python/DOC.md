---
name: package
description: "tabula-py Python package for extracting tabular data from PDFs into pandas DataFrames or CSV/TSV/JSON outputs"
metadata:
  languages: "python"
  versions: "2.10.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "tabula-py,pdf,tables,pandas,java,csv,json"
---

# tabula-py Python Package Guide

## Golden Rule

Use `tabula-py` only for text-based PDFs and make the Java runtime requirement explicit in setup. Import it as `import tabula`, expect `read_pdf()` to return table data derived from `tabula-java`, and treat JVM setup, `pages`, and extraction mode (`lattice` vs `stream`) as the first things to verify when results look wrong.

## Install

Pin the package version your project expects:

```bash
python -m pip install "tabula-py==2.10.0"
```

Common alternatives:

```bash
uv add "tabula-py==2.10.0"
poetry add "tabula-py==2.10.0"
```

If you want the faster `jpype` execution path and your Python version supports it:

```bash
python -m pip install "tabula-py[jpype]==2.10.0"
```

Before installing or running the package, make sure `java` is available on `PATH`:

```bash
java -version
```

The upstream docs explicitly recommend `pip` for current releases. The maintainer notes that the `conda-forge` recipe is not maintained by them.

## Initialize And Basic Setup

`tabula-py` has no client object and no authentication step. The main setup requirement is a working Java runtime.

```python
import tabula
```

Useful environment-level controls:

- `TABULA_JAR`: point at a custom `tabula-java` JAR if you need to override the bundled/default JAR choice
- `PATH`: must include the Java runtime so `tabula-py` can launch `java`

Quick health check when setup is failing:

```python
import tabula

print(tabula.environment_info())
```

Use that before debugging extraction options. The upstream FAQ recommends it when `tabula-py` cannot find or launch Java correctly.

## Core Usage

### Extract tables from a local PDF

`read_pdf()` defaults to `pages=1` and `multiple_tables=True`, so the common result is a list of DataFrames.

```python
import tabula

tables = tabula.read_pdf("reports/sales.pdf", pages=1)

if tables:
    df = tables[0]
    print(df.head())
```

If the PDF uses all pages:

```python
tables = tabula.read_pdf("reports/sales.pdf", pages="all")
```

### Extract from a remote PDF URL

`input_path` can be a local path, file-like object, or URL.

```python
import tabula

tables = tabula.read_pdf(
    "https://example.com/report.pdf",
    pages="all",
    user_agent="my-agent/1.0",
)
```

### Choose `lattice` or `stream`

Use `lattice=True` when the PDF has visible ruling lines. Use `stream=True` when tables are aligned by whitespace instead of borders.

```python
import tabula

tables = tabula.read_pdf(
    "reports/statement.pdf",
    pages=2,
    lattice=True,
)
```

```python
import tabula

tables = tabula.read_pdf(
    "reports/statement.pdf",
    pages=2,
    stream=True,
)
```

`guess=True` is the default. The upstream API docs note that `guess` can be combined with `lattice` or `stream`.

### Restrict extraction to a known page area

When the automatic detector picks up headers, footers, or adjacent text blocks, pass a fixed extraction area.

```python
import tabula

tables = tabula.read_pdf(
    "reports/sales.pdf",
    pages=1,
    area=[80, 30, 560, 560],
    multiple_tables=False,
)
```

Use `columns=[...]` when you know vertical split positions. Use `relative_area=True` or `relative_columns=True` only when you intentionally want percentage-based coordinates instead of absolute PDF points.

### Export directly to CSV, TSV, or JSON

Use `convert_into()` when you want a file on disk instead of DataFrames in memory.

```python
import tabula

tabula.convert_into(
    "reports/sales.pdf",
    "reports/sales.csv",
    output_format="csv",
    pages="all",
)
```

Batch conversion across a directory:

```python
import tabula

tabula.convert_into_by_batch(
    "incoming-pdfs",
    output_format="json",
    pages="all",
)
```

### Use a saved Tabula App template

If you already tuned extraction in the Tabula desktop/web app, reuse the exported template JSON instead of re-encoding the area and column settings by hand.

```python
import tabula

tables = tabula.read_pdf_with_template(
    "reports/sales.pdf",
    "templates/sales.tabula-template.json",
    pages="all",
)
```

This is the most stable route when the same PDF layout repeats regularly.

## Configuration Notes

- No auth: `tabula-py` does not use API keys, tokens, or service credentials.
- JVM options: use `java_options` for heap size, file encoding, or headless mode, for example `["-Xmx1g"]` or `["-Djava.awt.headless=true"]`.
- Sticky JVM startup: the FAQ notes that `java_options` are applied when the VM starts. If you need different Java options on later calls in the same Python process, use `force_subprocess=True` or run a fresh process.
- `multiple_tables=True` is the default in modern releases. The API docs warn that this changes how `pandas_options` is applied: with `multiple_tables=True`, options go to `pandas.DataFrame`; otherwise they go to `pandas.read_csv()`.
- Remote input: pass `user_agent=` when a remote PDF host rejects default urllib user agents.
- Encoding: `encoding="utf-8"` is the default. Windows-specific encoding problems may still require matching terminal and JVM encoding settings.

## Common Pitfalls

- Image-only PDFs do not work. `tabula-py` extracts text-based tables through `tabula-java`; scanned PDFs usually need OCR first.
- `pages` defaults to the first page. Agents often forget `pages="all"` and then think extraction failed.
- Installing `tabula` instead of `tabula-py` breaks `from tabula import ...` imports because the package names conflict. The FAQ recommends uninstalling `tabula` and reinstalling `tabula-py`.
- Empty or malformed DataFrames usually mean the wrong extraction mode, wrong area, or a PDF layout that is not actually tabular.
- `ParserError` or `CSVParseError` often happens when multiple tables with different shapes are parsed as one table. The FAQ recommends `multiple_tables=True` for that case.
- On macOS, Java UI focus stealing can happen. The FAQ suggests `java_options=["-Djava.awt.headless=true"]`.
- On Windows, file paths with spaces, terminal encoding, and Java `PATH` setup are common failure points.
- If you change JVM flags after the first extraction call and nothing happens, you are likely hitting the one-time VM initialization behavior rather than a bad option value.

## Version-Sensitive Notes For 2.10.0

- PyPI still lists `2.10.0` as the latest published release as of March 12, 2026.
- The maintainer release note for `v2.10.0` says this version adds Python 3.13 support and drops Python 3.8.
- That same `v2.10.0` release note warns that the optional `jpype` path does not support Python 3.13 yet. On Python 3.13, prefer the base package install and expect subprocess-backed execution unless upstream `jpype` support has caught up.
- The `v2.9.0` release made `jpype` optional so newer Python versions can still use `tabula-py` even when `jpype` lags behind CPython releases.
- If older examples assume pre-2.0 behavior, re-check `multiple_tables` handling. The docs note that `read_pdf()` has defaulted to `multiple_tables=True` since `2.0.0`.

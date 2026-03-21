---
name: package
description: "pdfplumber package guide for extracting text, tables, and layout data from PDFs in Python"
metadata:
  languages: "python"
  versions: "0.11.9"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pdfplumber,pdf,pdfminer,text-extraction,tables,python"
---

# pdfplumber Python Package Guide

## Golden Rule

Use `pdfplumber` for reading and extracting structured data from machine-generated PDFs. It is not a PDF editor and it does not perform OCR, so scanned image PDFs usually need an OCR step before `pdfplumber` can extract meaningful text.

## Install

Pin the package version your project expects:

```bash
python -m pip install "pdfplumber==0.11.9"
```

Common alternatives:

```bash
uv add "pdfplumber==0.11.9"
poetry add "pdfplumber==0.11.9"
```

Published `0.11.9` metadata depends on `pdfminer.six==20251230`, `Pillow>=9.1`, and `pypdfium2>=4.18.0`.

## Initialization And Setup

`pdfplumber` has no auth layer. The main setup work is choosing how to open the PDF and which parsing options you need.

Open a local file and ensure handles and page caches are released with a context manager:

```python
import pdfplumber

with pdfplumber.open("statement.pdf") as pdf:
    first_page = pdf.pages[0]
    text = first_page.extract_text()
    print(text)
```

Open encrypted PDFs with `password=`:

```python
import pdfplumber

with pdfplumber.open("protected.pdf", password="secret") as pdf:
    print(len(pdf.pages))
```

Open from bytes or a file-like object when the PDF comes from cloud storage or an HTTP response:

```python
from io import BytesIO

import pdfplumber
import requests

response = requests.get("https://example.com/report.pdf", timeout=30)
response.raise_for_status()

with pdfplumber.open(BytesIO(response.content)) as pdf:
    print(pdf.pages[0].extract_text())
```

Useful open-time options:

- `laparams={...}` passes layout-analysis settings through to `pdfminer.six`
- `unicode_norm="NFC" | "NFKC" | "NFD" | "NFKD"` normalizes Unicode before extraction
- `strict_metadata=True` turns invalid metadata values into exceptions instead of warnings
- `password="..."` opens encrypted PDFs

## Core Usage

### Extract plain text

```python
import pdfplumber

with pdfplumber.open("report.pdf") as pdf:
    page = pdf.pages[0]
    text = page.extract_text()
    print(text)
```

If spacing or line grouping matters, try the layout-aware path:

```python
text = page.extract_text(layout=True)
```

### Extract words with coordinates

`extract_words()` is often better than raw text when you need positions for downstream parsing:

```python
import pdfplumber

with pdfplumber.open("invoice.pdf") as pdf:
    words = pdf.pages[0].extract_words()

for word in words[:5]:
    print(word["text"], word["x0"], word["top"], word["x1"], word["bottom"])
```

This is a common starting point for invoice, statement, and form parsers.

### Search for text on a page

`search()` returns matched text with positions, which is useful when you need anchors before cropping nearby content:

```python
import re

import pdfplumber

with pdfplumber.open("invoice.pdf") as pdf:
    matches = pdf.pages[0].search(re.compile(r"Invoice Number"))

for match in matches:
    print(match["text"], match["x0"], match["top"])
```

### Crop to a region before extracting

Cropping usually improves reliability when the page has headers, footers, or multiple columns:

```python
import pdfplumber

with pdfplumber.open("report.pdf") as pdf:
    page = pdf.pages[0]
    body = page.crop((40, 80, page.width - 40, page.height - 60))
    print(body.extract_text())
```

The bounding box format is `(x0, top, x1, bottom)`.

### Remove duplicate characters

Some PDFs contain overlapping character layers, which causes repeated text. `dedupe_chars()` helps before text extraction:

```python
import pdfplumber

with pdfplumber.open("overlay.pdf") as pdf:
    page = pdf.pages[0].dedupe_chars()
    print(page.extract_text())
```

### Extract tables

Start with the defaults:

```python
import pdfplumber

with pdfplumber.open("table.pdf") as pdf:
    table = pdf.pages[0].extract_table()

for row in table or []:
    print(row)
```

If the defaults miss rows or columns, pass `table_settings` explicitly:

```python
import pdfplumber

table_settings = {
    "vertical_strategy": "lines",
    "horizontal_strategy": "text",
}

with pdfplumber.open("table.pdf") as pdf:
    tables = pdf.pages[0].extract_tables(table_settings=table_settings)
```

For hard PDFs, the usual workflow is:

1. crop to the table region
2. inspect with `debug_tablefinder(...)`
3. adjust `vertical_strategy`, `horizontal_strategy`, tolerances, and edge filtering

### Visual debugging

Use page images to debug table detection and geometry:

```python
import pdfplumber

with pdfplumber.open("table.pdf") as pdf:
    page = pdf.pages[0]
    image = page.to_image(resolution=150)
    image.debug_tablefinder()
    image.save("table-debug.png")
```

This is especially useful in notebooks because `PageImage` objects render inline.

### Command-line usage

The package installs a `pdfplumber` CLI. Typical examples:

```bash
pdfplumber input.pdf --format text
pdfplumber input.pdf --format csv --pages 1,2
pdfplumber input.pdf --types char rect line
```

Use the CLI for quick inspection and switch to Python when you need cropping, custom matching logic, or post-processing.

## Configuration Notes

There is no service configuration or API credential model. The main configuration surfaces are:

- `laparams` for layout analysis behavior inherited from `pdfminer.six`
- `table_settings` for table extraction behavior
- page cropping and filtering for scoping extraction
- password handling for encrypted PDFs

If you fetch PDFs over the network, handle authentication yourself with `requests`, `httpx`, cloud SDKs, or your storage client, then pass a path or file-like object into `pdfplumber`.

## Common Pitfalls

- `pdfplumber` works best on digitally generated PDFs. If the source is scanned images, text extraction may return little or nothing until OCR is applied.
- Coordinate math is easy to get wrong. Most page objects expose `x0`, `x1`, `top`, and `bottom`, and crop boxes use `(x0, top, x1, bottom)`.
- Large PDFs can hold substantial cached layout data. Prefer `with pdfplumber.open(...)` and call `page.close()` if you need to aggressively flush cached page data mid-run.
- `Page.filter(...)` changes extraction results but does not affect the current `to_image(...)` rendering, so visual debug output can differ from filtered extraction output.
- Table extraction is sensitive to page noise and layout. Crop first and tune strategies instead of assuming `extract_table()` will work globally on the full page.
- `search()` discards zero-width and all-whitespace matches because they do not have meaningful page positions.
- Image objects in a PDF expose metadata, but `pdfplumber` is not a full image reconstruction pipeline. Use a dedicated PDF or imaging tool if you need to recover embedded image bytes exactly.

## Version-Sensitive Notes

- `0.11.9` updates the `pdfminer.six` dependency to `20251230`. If text extraction behavior changes after an upgrade, check whether `pdfminer.six` changes are the cause.
- `0.11.8` added `edge_min_length_prefilter` to `table_settings`, which can help reduce short-edge noise in table detection.
- `0.11.7` removed `stroking_pattern` and `non_stroking_pattern` attributes after upstream parser changes. Avoid depending on those fields in object dictionaries.
- `0.11.5` added CLI `--format text` support and the `raise_unicode_errors` option on `open(...)`.
- The table extraction redesign landed in `v0.5.0`; older blog posts and examples written for pre-`0.5` releases are often incompatible with current settings and result shapes.
- PyPI `0.11.9` metadata declares `Python >=3.8`. If the maintainer branch README mentions newer tested interpreter ranges, treat the published package metadata as authoritative for this pinned release.

## Official Sources

- Maintainer repository: `https://github.com/jsvine/pdfplumber`
- Project README: `https://raw.githubusercontent.com/jsvine/pdfplumber/stable/README.md`
- Changelog: `https://raw.githubusercontent.com/jsvine/pdfplumber/stable/CHANGELOG.md`
- Tagged release requirements: `https://raw.githubusercontent.com/jsvine/pdfplumber/v0.11.9/requirements.txt`
- PyPI package page: `https://pypi.org/project/pdfplumber/`
- PyPI JSON metadata: `https://pypi.org/pypi/pdfplumber/0.11.9/json`

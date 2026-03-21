---
name: package
description: "PyMuPDF Python package guide for opening, inspecting, extracting, rendering, annotating, and saving PDF and document files"
metadata:
  languages: "python"
  versions: "1.27.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pymupdf,pdf,documents,ocr,text-extraction,rendering,annotations"
---

# PyMuPDF Python Package Guide

## Golden Rule

Use `pymupdf` for document access and manipulation, keep document I/O explicit, and save changes deliberately. The official docs for `1.27.2` use `import pymupdf`, `pymupdf.open(...)`, `page.get_text(...)`, `page.get_pixmap()`, and `doc.save(...)` as the core workflow.

## Install

Pin the version your project expects:

```bash
python -m pip install "PyMuPDF==1.27.2"
```

Common alternatives:

```bash
uv add "PyMuPDF==1.27.2"
poetry add "PyMuPDF==1.27.2"
```

Notes:

- PyPI lists `Requires: Python >=3.10`.
- Do not install `PyMuPDFb` directly unless you are debugging packaging internals. It is an implementation dependency used by current wheels.
- OCR support requires a separate Tesseract installation and language data; PyMuPDF does not bundle Tesseract itself.

## Initialize And Open Documents

### Open a local document

Use a context manager so the file handle closes cleanly:

```python
import pymupdf

with pymupdf.open("example.pdf") as doc:
    print(doc.page_count)
    print(doc.metadata)
```

### Create a new empty PDF

```python
import pymupdf

doc = pymupdf.open()
page = doc.new_page()
page.insert_text((72, 72), "Hello, PyMuPDF")
doc.save("created.pdf")
doc.close()
```

### Open from bytes or a network response

For in-memory data, pass `stream=`. If the format is not obvious from the bytes, also pass `filetype=`.

```python
import pymupdf
import requests

response = requests.get("https://example.com/report.pdf", timeout=30)
response.raise_for_status()

doc = pymupdf.open(stream=response.content)
print(doc.page_count)
doc.close()
```

For non-PDF streams:

```python
import pymupdf

with open("report.xps", "rb") as f:
    data = f.read()

doc = pymupdf.open(stream=data, filetype="xps")
doc.close()
```

### Open an encrypted PDF

Authentication is document-level. If a PDF is password-protected, check `needs_pass` and call `authenticate(...)` before reading pages.

```python
import os
import pymupdf

doc = pymupdf.open("secret.pdf")

if doc.needs_pass:
    ok = doc.authenticate(os.environ["PDF_PASSWORD"])
    if not ok:
        raise RuntimeError("Failed to authenticate PDF")

print(doc.page_count)
doc.close()
```

## Core Usage

### Extract text

The baseline extraction path is `page.get_text()`. For agent workflows, structured modes are often more useful than plain text.

```python
import pymupdf

with pymupdf.open("example.pdf") as doc:
    page = doc[0]

    plain_text = page.get_text()
    blocks = page.get_text("blocks")
    words = page.get_text("words")
    markdown = page.get_text("markdown")

    print(plain_text[:500])
    print(blocks[:2])
    print(words[:10])
    print(markdown[:500])
```

Use:

- default text when you just need readable content
- `"blocks"` or `"words"` when reading order matters or you need coordinates
- `"markdown"` when feeding extracted content into another LLM step

### Render a page to an image

```python
import pymupdf

with pymupdf.open("example.pdf") as doc:
    page = doc[0]
    pix = page.get_pixmap()
    pix.save("page-1.png")
```

If the default render is too low-resolution for OCR or UI previews, render with a higher DPI or a scaling matrix.

### Add annotations

```python
import pymupdf

doc = pymupdf.open("example.pdf")
page = doc[0]

rect = pymupdf.Rect(72, 72, 220, 120)
annot = page.add_freetext_annot(rect, "Review this section")
annot.set_colors(stroke=(1, 0, 0))
annot.update()

doc.save("annotated.pdf")
doc.close()
```

PyMuPDF also supports highlight, underline, strikeout, squiggle, shape, text, and stamp annotations. Save after updates or the changes stay in memory only.

### Save changes

Use a new output path unless you explicitly want in-place incremental updates.

```python
import pymupdf

doc = pymupdf.open("example.pdf")
doc.set_metadata({})
doc.save("clean-copy.pdf", garbage=3, deflate=True)
doc.close()
```

Incremental save is only appropriate when writing back to the original file and when the document still allows it:

```python
import pymupdf

doc = pymupdf.open("example.pdf")

if doc.can_save_incrementally():
    doc.save(doc.name, incremental=True, encryption=pymupdf.PDF_ENCRYPT_KEEP)

doc.close()
```

## OCR Setup And Usage

PyMuPDF can OCR page images through Tesseract, but you must install and configure Tesseract separately.

Typical setup:

```bash
# macOS
brew install tesseract

# Debian / Ubuntu
sudo apt-get install tesseract-ocr

export TESSDATA_PREFIX="/path/to/tessdata"
```

OCR a page once, then reuse the generated `TextPage` for extraction:

```python
import pymupdf

with pymupdf.open("scanned.pdf") as doc:
    page = doc[0]
    textpage = page.get_textpage_ocr(language="eng")
    text = page.get_text(textpage=textpage)
    print(text[:1000])
```

Guidance:

- OCR is much slower than normal text extraction, so do it only for scanned or image-only pages.
- Reuse the `TextPage` instead of calling OCR repeatedly on the same page.
- Install the Tesseract language packs that match the page language.

## Configuration Notes

PyMuPDF has no package-level network auth, cloud auth, or global configuration file. Configuration is mostly about how you open files and where supporting binaries live.

- Remote auth happens before PyMuPDF sees the file. Fetch bytes with `requests`, `httpx`, S3 clients, or another transport, then pass the bytes to `pymupdf.open(stream=...)`.
- Password-protected PDFs use `doc.authenticate(...)`, not a global environment variable consumed by PyMuPDF.
- OCR setup depends on the Tesseract installation and `TESSDATA_PREFIX`.
- When opening in-memory documents that are not PDFs, pass `filetype=` explicitly.

## Common Pitfalls

- Prefer `import pymupdf` and current official examples. Many older snippets use the older naming style and are more likely to drift.
- Page access is zero-based: `doc[0]` is the first page.
- `page.get_text()` is convenient, but plain text order may not match the human reading order on complex layouts. Use `"blocks"`, `"words"`, or `"markdown"` when structure matters.
- Non-PDF byte streams often need `filetype=`. Without it, open may fail or mis-detect the format.
- OCR is not automatic. If a scanned PDF returns little or no text, switch to `get_textpage_ocr(...)`.
- `doc.save()` writes a new file by default. Incremental save only works for specific cases and should be gated with `doc.can_save_incrementally()`.
- Page and annotation objects depend on the underlying document. Do not keep using them after `doc.close()`.

## Version-Sensitive Notes For 1.27.2

- PyPI currently requires Python `>=3.10`. Treat that as the packaging floor even if some docs pages still mention test coverage for older interpreters.
- The `1.27.x` docs explicitly document `page.get_text("markdown")`, OCR via `get_textpage_ocr()`, and incremental-save checks via `doc.can_save_incrementally()`. If you are pinned to an older project version, verify those helpers before assuming they exist unchanged.

## Official Sources

- PyMuPDF docs root: `https://pymupdf.readthedocs.io/en/latest/`
- Installation: `https://pymupdf.readthedocs.io/en/latest/installation.html`
- Tutorial: `https://pymupdf.readthedocs.io/en/latest/tutorial.html`
- Opening files: `https://pymupdf.readthedocs.io/en/latest/how-to-open-a-file.html`
- Text recipes: `https://pymupdf.readthedocs.io/en/latest/recipes-text.html`
- Image recipes: `https://pymupdf.readthedocs.io/en/latest/recipes-images.html`
- Annotation recipes: `https://pymupdf.readthedocs.io/en/latest/recipes-annotations.html`
- OCR recipes: `https://pymupdf.readthedocs.io/en/latest/recipes-ocr.html`
- Document API: `https://pymupdf.readthedocs.io/en/latest/document.html`
- Changelog: `https://pymupdf.readthedocs.io/en/latest/changes.html`
- PyPI package page: `https://pypi.org/project/PyMuPDF/`

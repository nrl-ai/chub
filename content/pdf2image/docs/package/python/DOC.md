---
name: package
description: "pdf2image Python package guide for converting PDFs to Pillow images with Poppler"
metadata:
  languages: "python"
  versions: "1.17.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pdf2image,pdf,poppler,pillow,image,conversion"
---

# pdf2image Python Package Guide

## Golden Rule

`pdf2image` is not a standalone PDF renderer. It is a Python wrapper around Poppler command-line tools such as `pdftoppm`, `pdftocairo`, and `pdfinfo`. If Poppler is missing or too old, your code will fail even if `pip install pdf2image` succeeded.

## Install

Pin the package version your project expects:

```bash
python -m pip install "pdf2image==1.17.0"
```

Common alternatives:

```bash
uv add "pdf2image==1.17.0"
poetry add "pdf2image==1.17.0"
```

Install Poppler separately.

Ubuntu or Debian:

```bash
sudo apt-get install poppler-utils
```

Arch Linux:

```bash
sudo pacman -S poppler
```

macOS:

```bash
brew install poppler
```

Windows:

1. Install `pdf2image` with `pip`.
2. Download a Poppler build for Windows from the upstream-recommended `oschwartz10612/poppler-windows` releases.
3. Add the extracted `bin` directory to `PATH`, or pass that directory as `poppler_path=...`.

Verify Poppler before debugging Python code:

```bash
pdftoppm -h
pdfinfo -h
```

## Setup And Initialization

There is no service authentication layer. Configuration is local:

- Poppler executable discovery through `PATH`
- optional `poppler_path` override
- optional PDF password via `userpw`

Basic imports:

```python
from pdf2image import convert_from_bytes, convert_from_path
from pdf2image.exceptions import (
    PDFInfoNotInstalledError,
    PDFPageCountError,
    PDFPopplerTimeoutError,
    PDFSyntaxError,
)
```

Using an explicit Poppler location is common on Windows or inside custom containers:

```python
from pdf2image import convert_from_path

images = convert_from_path(
    "document.pdf",
    poppler_path=r"C:\poppler\Library\bin",
)
```

## Core Usage

### Convert a PDF file from disk

```python
from pdf2image import convert_from_path

images = convert_from_path(
    "report.pdf",
    dpi=200,
    fmt="jpeg",
)

for i, image in enumerate(images, start=1):
    image.save(f"page-{i}.jpg", "JPEG")
```

`convert_from_path()` returns a list of Pillow `Image` objects, one per page.

### Convert from bytes

Use this when the PDF already lives in memory, for example after downloading it from object storage or a database.

```python
from pdf2image import convert_from_bytes

with open("report.pdf", "rb") as f:
    pdf_bytes = f.read()

images = convert_from_bytes(pdf_bytes, dpi=200, fmt="png")
```

### Limit work to a page range

```python
from pdf2image import convert_from_path

images = convert_from_path(
    "report.pdf",
    first_page=1,
    last_page=3,
    fmt="png",
)
```

### Password-protected PDFs

`pdf2image` supports the user password for encrypted PDFs:

```python
from pdf2image import convert_from_path

images = convert_from_path(
    "protected.pdf",
    userpw="secret-password",
)
```

### Large PDFs: write to disk instead of keeping everything in memory

This is the main operational pattern for real workloads.

```python
import tempfile

from pdf2image import convert_from_path

with tempfile.TemporaryDirectory() as tmpdir:
    images = convert_from_path(
        "large.pdf",
        output_folder=tmpdir,
        fmt="jpeg",
        paths_only=True,
    )

    for image_path in images:
        print(image_path)
```

Use `paths_only=True` when your next step works on file paths and you want to avoid loading every page into memory.

### Inspect page metadata first

Use `pdfinfo_from_path()` when you want the page count before deciding what to render:

```python
from pdf2image import pdfinfo_from_path

info = pdfinfo_from_path("report.pdf")
page_count = info["Pages"]
print(page_count)
```

## Important Options

- `dpi`: output resolution. Higher DPI improves quality but increases CPU, memory, and file size.
- `fmt`: common values are `ppm`, `jpeg`, `png`, and `tiff`.
- `first_page` / `last_page`: limit rendering to the pages you need.
- `thread_count`: parallelizes Poppler work; upstream recommends staying conservative because I/O becomes the bottleneck quickly.
- `output_folder`: strongly recommended for big PDFs.
- `paths_only`: return file paths instead of Pillow objects; requires `output_folder`.
- `use_pdftocairo`: can improve performance for some documents.
- `timeout`: raises `PDFPopplerTimeoutError` if Poppler takes too long.
- `strict=True`: raises `PDFSyntaxError` on syntax errors instead of swallowing them.
- `size`: resizes output pages after rendering.
- `grayscale=True`: render grayscale images.

## Common Pitfalls

- Missing Poppler is the first thing to check. The failure often shows up as `PDFInfoNotInstalledError` or `PDFPageCountError`, not as a Python import error.
- `pdf2image` can exhaust memory on large PDFs if you render many pages to in-memory Pillow objects. Prefer `output_folder` and `paths_only=True`.
- `paths_only=True` only makes sense with `output_folder`. Without an output directory, there are no files to point at.
- Default `ppm` output is large and uncompressed. Use `fmt="jpeg"` for faster I/O and smaller files when JPEG is acceptable.
- PNG is slower because of compression. Use it when you need lossless output or transparency, not by default.
- Old Poppler versions can fail on some PDFs, especially the DocuSign-style broken-xref case documented upstream. If you see `Unable to get page count` with syntax errors, update Poppler before changing Python code.
- More threads are not automatically better. Upstream explicitly warns that high `thread_count` values usually become I/O-bound; keep it modest.

## Version-Sensitive Notes

- The version used here `1.17.0` still matches the latest PyPI release as of March 12, 2026.
- The official docs site currently renders `1.16.1`, so prefer the GitHub README and PyPI project page for package-level setup and behavior when you need `1.17.0` context.
- The `1.17.0` GitHub release notes call out fixes around public exports, `pdfinfo` page-range support, and a `single_file` plus `thread_count` bug.
- Upstream package text says Python `3.7+`, but the PyPI classifiers shown on March 12, 2026 stop at Python `3.10`. Treat newer Python versions as plausible but verify them in your own CI instead of assuming classifier coverage means active testing.
- The current `master` branch source exposes parameters that are not described consistently across the README and docs site. For agent work pinned to `1.17.0`, avoid relying on `master`-only signatures without checking the installed package.

## Official Sources

- GitHub repository: `https://github.com/Belval/pdf2image`
- GitHub releases: `https://github.com/Belval/pdf2image/releases`
- Official docs site: `https://belval.github.io/pdf2image/`
- PyPI project page: `https://pypi.org/project/pdf2image/`

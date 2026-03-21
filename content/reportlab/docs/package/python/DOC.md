---
name: package
description: "ReportLab PDF toolkit for generating PDFs, Platypus documents, graphics, and charts in Python"
metadata:
  languages: "python"
  versions: "4.4.10"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "reportlab,pdf,documents,platypus,graphics,charts,python"
---

# ReportLab Python Package Guide

## Golden Rule

Use the open-source `reportlab` package from PyPI, import from `reportlab.*`, and choose the API level deliberately:

- Use `reportlab.pdfgen.canvas.Canvas` when you need low-level drawing control.
- Use Platypus (`reportlab.platypus`) when you need layouted documents built from paragraphs, tables, images, and page templates.

The docs site also contains ReportLab PLUS and RML material. Do not mix those products into code meant for the open-source `reportlab` package.

## Install

Pin the package version your project expects:

```bash
python -m pip install "reportlab==4.4.10"
```

Common alternatives:

```bash
uv add "reportlab==4.4.10"
poetry add "reportlab==4.4.10"
```

Upstream notes that matter during setup:

- The open-source install docs describe `reportlab` as a Python library for creating PDFs and graphics.
- The upstream install page says the open-source package supports Python `3.7` through `3.13`.
- The user guide notes that `Pillow` is needed if you want to import images beyond the native JPEG support.

## Choose The Right API

### Low-level PDF generation with `pdfgen`

Use `canvas.Canvas` when you want absolute positioning and full control over pages:

```python
from io import BytesIO

from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas

buffer = BytesIO()
pdf = canvas.Canvas(buffer, pagesize=letter)

pdf.setTitle("hello-reportlab")
pdf.drawString(72, 720, "Hello from ReportLab")
pdf.drawString(72, 700, "Use pdfgen when exact coordinates matter.")
pdf.showPage()
pdf.save()

pdf_bytes = buffer.getvalue()
```

Use this style for invoices, labels, overlays, and other fixed-layout output.

### Structured documents with Platypus

Use Platypus when you want flow-based layout instead of manual coordinates:

```python
from pathlib import Path

from reportlab.lib.styles import getSampleStyleSheet
from reportlab.platypus import Paragraph, SimpleDocTemplate, Spacer, Table

output_path = Path("reportlab-guide.pdf")
styles = getSampleStyleSheet()

story = [
    Paragraph("ReportLab Platypus Example", styles["Title"]),
    Spacer(1, 12),
    Paragraph(
        "Platypus builds a document from flowables such as paragraphs, tables, "
        "images, and page breaks.",
        styles["BodyText"],
    ),
    Spacer(1, 12),
    Table(
        [["Package", "Version"], ["reportlab", "4.4.10"]],
        hAlign="LEFT",
    ),
]

doc = SimpleDocTemplate(output_path)
doc.build(story)
```

The Platypus chapter describes documents as a sequence of flowables, assembled into a `Story`, then built into the final PDF.

### Vector graphics and charts

The user guide also documents `reportlab.graphics` for drawings and chart primitives. Use that layer when you need reusable vector graphics embedded in PDFs instead of manually drawing every shape on a canvas.

## Core Patterns

### Generate into memory for web responses

`reportlab` writes binary PDF output cleanly to file-like objects, so `BytesIO` is the usual choice in web apps and background jobs:

```python
from io import BytesIO

from reportlab.pdfgen import canvas

def build_pdf_bytes() -> bytes:
    buffer = BytesIO()
    pdf = canvas.Canvas(buffer)
    pdf.drawString(72, 720, "Generated in memory")
    pdf.showPage()
    pdf.save()
    return buffer.getvalue()
```

### Register a TrueType font before using it

Base 14 PDF fonts work without extra setup, but non-default fonts should be registered explicitly:

```python
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.pdfgen import canvas

pdfmetrics.registerFont(TTFont("Inter", "/absolute/path/Inter-Regular.ttf"))

pdf = canvas.Canvas("font-demo.pdf")
pdf.setFont("Inter", 12)
pdf.drawString(72, 720, "Custom font example")
pdf.showPage()
pdf.save()
```

This avoids fragile assumptions about what fonts are already known to the runtime.

## Configuration And Asset Paths

The user guide documents `reportlab.rl_config` for package-level defaults. Useful settings include:

- `defaultPageSize`
- `defaultEncoding`
- `T1SearchPath`
- `TTFSearchPath`
- `CMapSearchPath`
- `showBoundary`
- `shapeChecking`

Example:

```python
from reportlab import rl_config

rl_config.defaultPageSize = (595.27, 841.89)  # A4 in points
rl_config.showBoundary = 0
rl_config.shapeChecking = 1
rl_config.TTFSearchPath.append("/app/fonts")
```

Prefer explicit configuration in application startup instead of mutating these values ad hoc in random helper functions.

## Common Pitfalls

- The docs site includes commercial ReportLab products alongside the open-source toolkit. Stay within the open-source install guide and user guide when writing code for `pip install reportlab`.
- `canvas.Canvas` output is incomplete until you call `showPage()` for the current page and `save()` for the document.
- Platypus is flowable-based. Do not treat it like a thin wrapper over `canvas.drawString()`; build a `Story` from flowables and let the document template place them.
- Complex fonts are not automatic. Register TrueType fonts yourself and verify glyph coverage before assuming Unicode text will render correctly.
- Imported images beyond JPEG support rely on Pillow being available, so image-heavy code should be tested in the same environment it will run in.
- Global `rl_config` changes affect the process. Keep environment-specific font paths and debug flags centralized.

## Version-Sensitive Notes For `4.4.10`

- The version used here `4.4.10` matches the current PyPI release for `reportlab` as of March 12, 2026.
- ReportLab `4.x` uses modern packaging metadata (`pyproject.toml` was introduced in the 4.0 line), so older installation advice aimed at much earlier releases is not a good default.
- The upstream `4.4.0` release notes mention experimental right-to-left and shaping support with `pyfribidi`. Treat complex-script output as version-sensitive and verify it visually instead of assuming older examples are still the best reference.

## Official Sources

- Docs root: https://docs.reportlab.com/
- Open-source install guide: https://docs.reportlab.com/reportlab/install/open_source_package/
- User guide: https://docs.reportlab.com/reportlab/userguide/
- 4.4.0 release notes: https://docs.reportlab.com/releases/notes/whats-new-440/
- PyPI: https://pypi.org/project/reportlab/

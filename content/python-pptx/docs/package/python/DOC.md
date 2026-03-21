---
name: package
description: "python-pptx package guide for creating and editing PowerPoint .pptx files in Python"
metadata:
  languages: "python"
  versions: "1.0.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "python-pptx,powerpoint,pptx,presentation,office,document-generation"
---

# python-pptx Python Package Guide

## Golden Rule

Use `python-pptx` with `from pptx import Presentation` to create or edit `.pptx` files, and start from a real template deck whenever layout, theme, or slide size matters. The Read the Docs site still shows `1.0.0` in its page chrome, so check PyPI and the changelog for `1.0.2`-level compatibility notes before assuming the docs site reflects the newest patch release.

## Install

Pin the version your project expects:

```bash
python -m pip install "python-pptx==1.0.2"
```

Common alternatives:

```bash
uv add "python-pptx==1.0.2"
poetry add "python-pptx==1.0.2"
```

`python-pptx` is a local file-format library. It does not call a remote API and does not require Microsoft PowerPoint to be installed.

## Initialize And Save Presentations

Create a new deck from the built-in default template:

```python
from pptx import Presentation

prs = Presentation()
print(len(prs.slide_layouts))
```

Open an existing deck or template:

```python
from pptx import Presentation

prs = Presentation("template.pptx")
```

You can also read from or save to a file-like object:

```python
from io import BytesIO
from pptx import Presentation

buffer = BytesIO()
prs = Presentation()
prs.save(buffer)
buffer.seek(0)

same_prs = Presentation(buffer)
```

## Core Usage

### Add a slide and fill placeholders

Use a layout from the template, then populate its placeholders:

```python
from pptx import Presentation

prs = Presentation("template.pptx")
slide = prs.slides.add_slide(prs.slide_layouts[1])  # often "Title and Content"

slide.shapes.title.text = "Quarterly Review"

text_frame = slide.placeholders[1].text_frame
text_frame.text = "Highlights"

for bullet in ["Revenue up 12%", "Churn down 2%", "Opened EU region"]:
    paragraph = text_frame.add_paragraph()
    paragraph.text = bullet
    paragraph.level = 1

prs.save("quarterly-review.pptx")
```

### Add an image with explicit sizing

Use helper units instead of raw integers:

```python
from pptx import Presentation
from pptx.util import Inches

prs = Presentation("template.pptx")
slide = prs.slides.add_slide(prs.slide_layouts[6])  # often blank

slide.shapes.add_picture(
    "chart.png",
    Inches(1.0),
    Inches(1.0),
    width=Inches(8.0),
)

prs.save("with-image.pptx")
```

### Add a chart

`add_chart()` returns a `GraphicFrame`; get the chart from `.chart`:

```python
from pptx import Presentation
from pptx.chart.data import CategoryChartData
from pptx.enum.chart import XL_CHART_TYPE
from pptx.util import Inches

prs = Presentation()
slide = prs.slides.add_slide(prs.slide_layouts[5])

chart_data = CategoryChartData()
chart_data.categories = ["Q1", "Q2", "Q3"]
chart_data.add_series("Sales", (19.2, 21.4, 24.7))

graphic_frame = slide.shapes.add_chart(
    XL_CHART_TYPE.COLUMN_CLUSTERED,
    Inches(1.0),
    Inches(1.5),
    Inches(8.0),
    Inches(4.5),
    chart_data,
)

chart = graphic_frame.chart
chart.has_legend = False

prs.save("with-chart.pptx")
```

### Add a table

`add_table()` also returns a `GraphicFrame`; use `.table` to reach cells:

```python
from pptx import Presentation
from pptx.util import Inches

prs = Presentation()
slide = prs.slides.add_slide(prs.slide_layouts[5])

graphic_frame = slide.shapes.add_table(
    rows=3,
    cols=2,
    left=Inches(1.0),
    top=Inches(1.5),
    width=Inches(6.0),
    height=Inches(1.5),
)

table = graphic_frame.table
table.cell(0, 0).text = "Region"
table.cell(0, 1).text = "Revenue"
table.cell(1, 0).text = "NA"
table.cell(1, 1).text = "$1.2M"
table.cell(2, 0).text = "EU"
table.cell(2, 1).text = "$0.9M"

prs.save("with-table.pptx")
```

### Read an existing deck

```python
from pptx import Presentation

prs = Presentation("existing-deck.pptx")

for slide in prs.slides:
    for shape in slide.shapes:
        if hasattr(shape, "text"):
            print(shape.text)
```

## Configuration And Auth

- Auth: none. `python-pptx` manipulates local `.pptx` files and does not need API keys, OAuth, or cloud credentials.
- Template choice is the main configuration surface. Use `Presentation("template.pptx")` when you need your organization's theme, widescreen layout, custom masters, or predefined placeholders.
- Use `pptx.util` helpers like `Inches`, `Cm`, and `Pt` for positions and font sizes instead of raw EMU values.
- Prefer file-like objects when integrating with web uploads, object storage downloads, or in-memory attachments.

## Common Pitfalls

- `Presentation()` with no argument uses the package's default template, not your project's branded deck.
- Placeholder access is keyed by placeholder `idx`, not by visual order. A placeholder index that works in one layout can be missing in another.
- `slide.shapes.add_chart()` and `slide.shapes.add_table()` return `GraphicFrame` objects. Access `.chart` or `.table` before trying to edit content.
- Slide layout indexes are template-specific. `slide_layouts[1]` is often "Title and Content", but do not hard-code that assumption across arbitrary templates without checking.
- Coordinates and sizes are stored internally in EMUs. Use `Inches`, `Cm`, or `Pt` helpers or your layout math will be unreadable and easy to break.
- When editing an existing deck, open that `.pptx` first and save back out from it. Rebuilding from a blank presentation will not preserve the original theme, master slides, or placeholder structure.

## Version-Sensitive Notes For 1.0.2

- PyPI lists `python-pptx 1.0.2` with a release date of `2024-08-07`; the version used here matched the live registry.
- The Read the Docs `latest` site still displays `python-pptx 1.0.0 documentation`, so treat it as the main usage guide but verify patch-level compatibility against PyPI and the changelog.
- `1.0.1` dropped Python 3.7 support, added Python 3.12 support, and fixed several chart and image-handling issues.
- `1.0.2` adds Python 3.13 support and includes bug fixes around hyperlinks, line spacing, font handling, and chart date-axis behavior.

## Official Sources

- Docs root: `https://python-pptx.readthedocs.io/en/latest/`
- Quickstart: `https://python-pptx.readthedocs.io/en/latest/user/quickstart.html`
- Working with presentations: `https://python-pptx.readthedocs.io/en/latest/user/presentations.html`
- Working with placeholders: `https://python-pptx.readthedocs.io/en/latest/user/placeholders-using.html`
- PyPI package page: `https://pypi.org/project/python-pptx/`
- Repository README: `https://raw.githubusercontent.com/scanny/python-pptx/master/README.rst`
- Changelog: `https://raw.githubusercontent.com/scanny/python-pptx/master/HISTORY.rst`

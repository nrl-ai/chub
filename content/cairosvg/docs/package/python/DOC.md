---
name: package
description: "CairoSVG Python package guide for converting SVG inputs to PNG, PDF, PS, and SVG outputs"
metadata:
  languages: "python"
  versions: "2.8.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "cairosvg,svg,graphics,rendering,png,pdf,postscript,cli"
---

# CairoSVG Python Package Guide

## Golden Rule

Use `cairosvg` when you need a pure-Python entry point for converting static SVG content into `png`, `pdf`, `ps`, or `svg` outputs. Treat it as a renderer for trusted, mostly static SVG/CSS content, not as a full browser engine: JavaScript, animation, advanced filters, and webfont behavior are limited.

## Install

Pin the package version your project expects:

```bash
python -m pip install "CairoSVG==2.8.2"
```

Common alternatives:

```bash
uv add "CairoSVG==2.8.2"
poetry add "CairoSVG==2.8.2"
```

Native dependency note:

- CairoSVG depends on the Cairo graphics library plus `libffi`.
- Linux/macOS environments usually need system packages for Cairo available before import-time rendering works.
- The official install page calls out `cairo`, `libffi`, and a C compiler toolchain as the usual prerequisites.

If installation succeeds but conversion fails at import time, check that the runtime can load the native Cairo library.

## Initialize And Convert

The public API is module-level conversion helpers:

- `cairosvg.svg2png(...)`
- `cairosvg.svg2pdf(...)`
- `cairosvg.svg2ps(...)`
- `cairosvg.svg2svg(...)`

Each helper accepts one input source at a time:

- `url=...` for a local path or remote URL
- `file_obj=...` for an open file-like object
- `bytestring=...` for SVG bytes or a Unicode string in `2.8.2`

Each helper returns output bytes unless you pass `write_to=...`.

### Convert a local SVG file to PNG bytes

```python
import cairosvg

png_bytes = cairosvg.svg2png(url="diagram.svg")

with open("diagram.png", "wb") as f:
    f.write(png_bytes)
```

### Convert a string or bytes payload directly

```python
import cairosvg

svg = """
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="40">
  <rect width="120" height="40" fill="#0b7285" />
  <text x="12" y="26" fill="white">Hello</text>
</svg>
"""

cairosvg.svg2pdf(bytestring=svg, write_to="hello.pdf")
```

### Convert to a file-like object

```python
from io import BytesIO
import cairosvg

output = BytesIO()
cairosvg.svg2png(url="diagram.svg", write_to=output)

png_bytes = output.getvalue()
```

## Core Usage

### Control output size and resolution

The most useful rendering parameters are:

- `dpi`: pixel density used for CSS absolute units
- `scale`: multiply the rendered output size
- `parent_width` and `parent_height`: resolve percentage-based SVG dimensions
- `output_width` and `output_height`: force final output dimensions

```python
import cairosvg

cairosvg.svg2png(
    url="chart.svg",
    write_to="chart@2x.png",
    dpi=192,
    output_width=1600,
    output_height=900,
)
```

Use `parent_width` or `parent_height` when the SVG relies on percentages and otherwise renders at an unexpected size.

### Convert directly from a URL

```python
import cairosvg

cairosvg.svg2pdf(
    url="https://example.com/assets/report.svg",
    write_to="report.pdf",
)
```

This works for remote assets, but any linked images, stylesheets, or nested SVG resources still need to be reachable from the runtime environment.

### Use the CLI for shell pipelines

```bash
cairosvg input.svg -f png -o output.png
cairosvg input.svg -f pdf -o output.pdf
cat input.svg | cairosvg -f png -o output.png
```

Common CLI flags from the official docs:

- `-f, --format`: `pdf`, `png`, `ps`, or `svg`
- `-o, --output`: output filename
- `-d, --dpi`: DPI for CSS unit conversion
- `-W, --width` and `-H, --height`: parent container size
- `-s, --scale`: scale factor
- `--output-width` and `--output-height`: final dimensions
- `-u, --unsafe`: resolve XML entities and allow very large files

## Configuration And Environment

CairoSVG has no service authentication or persistent client configuration. Most configuration is per-call or per-command:

- install the right package version
- ensure Cairo and font libraries are available on the host
- choose whether to read from local files, file-like objects, raw SVG content, or remote URLs
- pass explicit sizing parameters when the source SVG uses percentages or relies on CSS units

Font behavior is environment-sensitive. CairoSVG uses the host font stack through Cairo, so missing fonts on the machine usually change output rendering.

## Common Pitfalls

### `unsafe=True` is for trusted input only

The `unsafe` flag enables XML entity resolution and very large files. The docs warn that this is vulnerable to XXE-style issues and denial-of-service conditions. Do not enable it for untrusted SVG content.

### It does not implement the full browser SVG model

The support docs explicitly call out important limits:

- animations are not supported
- no DOM, no JavaScript, and no interactive SVG behavior
- only a small subset of filter effects is implemented
- `@font-face` is not supported in the documented feature set

If an SVG depends on browser-only behavior, headless browser rendering is often a better fit than CairoSVG.

### Fonts and CSS can render differently across machines

If text layout or icon rendering changes between environments:

- verify the needed fonts are installed on the host
- avoid assuming browser webfont loading behavior
- test the exact runtime image or container used in production

### Percentage-based SVG sizes often need a parent size

An SVG that relies on percentage width or height may render too small, too large, or fail to match browser output unless you pass `parent_width` and `parent_height` or the CLI `-W` and `-H` flags.

### Output format naming is slightly inconsistent across official sources

The current docs page documents `png`, `pdf`, `ps`, and `svg`. The PyPI page and repository README also mention EPS output, but the docs page does not currently document a public `svg2eps` API or `-f eps` CLI flag. For agent-generated code, stick to the four formats documented on the official user docs unless you verify EPS behavior against the installed version.

## Version-Sensitive Notes

- Version used here `2.8.2` matches the current PyPI release as of March 12, 2026.
- The CairoSVG website still contains stale version signals, including references to Python 3.6+ and a homepage note pointing at `CairoSVG 2.7.1`. Prefer PyPI and the GitHub releases page for current package-version and Python-support facts.
- GitHub release notes for `2.8.1` and `2.8.2` mention broader input compatibility for `file_obj` and `bytestring`. If you are supporting older `2.8.0` or earlier environments, be more conservative and normalize SVG input to bytes.

## Official Sources

- Documentation: https://cairosvg.org/documentation/
- SVG support notes: https://cairosvg.org/documentation/index.html#how-good-is-cairosvg-at-following-the-specification
- Installation notes and current release metadata: https://pypi.org/project/CairoSVG/
- Repository README: https://github.com/Kozea/CairoSVG
- Releases: https://github.com/Kozea/CairoSVG/releases

---
name: package
description: "plotnine grammar-of-graphics plotting library for Python with pandas and polars DataFrame support"
metadata:
  languages: "python"
  versions: "0.15.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "plotnine,python,visualization,ggplot,data-viz,pandas,polars,matplotlib"
---

# plotnine Python Package Guide

## Golden Rule

Use `plotnine` when you want ggplot2-style grammar-of-graphics plots in Python. Put dataframe column names in `aes(...)`, set constant styles outside `aes(...)`, use `.show()` in scripts and terminals, and use `.save(...)` for deterministic output files.

## Install

Pin the version your project expects:

```bash
python -m pip install "plotnine==0.15.3"
```

Common alternatives:

```bash
uv add "plotnine==0.15.3"
poetry add "plotnine==0.15.3"
```

Install optional extras when you need them:

```bash
python -m pip install "plotnine[extra]==0.15.3"
```

`plotnine[extra]` pulls in commonly needed optional packages such as:

- `adjustText` for automatic text-label adjustment
- `geopandas` for geographic data helpers
- `scikit-learn` for Gaussian-process smoothing
- `scikit-misc` for LOESS smoothing

## Initialize And Render A First Plot

The standard workflow is:

1. Start with a dataframe.
2. Create a `ggplot(data, aes(...))` object.
3. Add layers with `+`.
4. Render with `.show()` or save with `.save(...)`.

```python
from plotnine import aes, geom_point, geom_smooth, ggplot, labs, theme_minimal
from plotnine.data import penguins

p = (
    ggplot(
        penguins.dropna(subset=["bill_length_mm", "bill_depth_mm", "species"]),
        aes("bill_length_mm", "bill_depth_mm", color="species"),
    )
    + geom_point(size=2.5, alpha=0.7)
    + geom_smooth(method="lm", se=False)
    + labs(
        title="Palmer Penguins",
        x="Bill length (mm)",
        y="Bill depth (mm)",
    )
    + theme_minimal()
)

p.show()
```

Notes:

- In notebooks, returning the plot as the last expression is usually enough to render it.
- In scripts, REPL sessions, and agent-driven terminal runs, call `.show()` explicitly.
- The built-in example datasets live under `plotnine.data`.

## Core Usage

### Map data in `aes`, set constants on the geom

Mapped aesthetics create scales and legends. Constant aesthetics should be set directly on the layer:

```python
from plotnine import aes, geom_point, ggplot

p = (
    ggplot(df, aes("x", "y", color="group"))
    + geom_point(size=3, alpha=0.8)
)
```

Do not do this for a constant color:

```python
ggplot(df, aes("x", "y", color="blue"))  # wrong: "blue" is treated as mapped data
```

Instead:

```python
ggplot(df, aes("x", "y")) + geom_point(color="blue")
```

### Facet and label plots

```python
from plotnine import aes, facet_wrap, geom_point, ggplot, labs

p = (
    ggplot(df, aes("x", "y", color="group"))
    + geom_point()
    + facet_wrap("~group")
    + labs(title="Grouped scatter plot")
)
```

### Control scales vs coordinate zoom

If you want to zoom without dropping data that affects statistics, use a coordinate system such as `coord_cartesian(...)` instead of scale limits.

```python
from plotnine import aes, coord_cartesian, geom_point, ggplot

p = (
    ggplot(df, aes("x", "y"))
    + geom_point()
    + coord_cartesian(xlim=(0, 10), ylim=(0, 100))
)
```

Use `xlim(...)`, `ylim(...)`, or scale limits only when you intentionally want rows outside the range removed before stats are computed.

### Save reproducible output

`plotnine` uses inches for physical size and `dpi` for raster resolution:

```python
p.save("reports/penguins-scatter.png", width=8, height=5, dpi=150)
```

Useful save-time options:

- `format="svg"` or a `.svg` filename for vector output
- `verbose=False` to suppress save logging
- `limitsize=False` only when you intentionally need images larger than the default 25x25 inch safety limit

## Configuration

`plotnine` does not use API credentials or service authentication. The configuration surface is mainly rendering and theme behavior.

### Set a default figure size

```python
from plotnine.options import set_option

set_option("figure_size", (8, 5))
```

Per-plot sizing is often clearer when different plots need different output dimensions:

```python
from plotnine import theme

p = p + theme(figure_size=(8, 5))
```

### SVG font portability

If you save SVGs for environments that may not have the same fonts installed, disable font embedding assumptions:

```python
from plotnine import theme

p = p + theme(svg_usefonts=False)
```

## Common Pitfalls

- `aes(...)` expects variable names or expressions, usually as strings. `aes("colname")` is correct for dataframe columns.
- Constant style values such as `color="red"` or `size=3` belong on the geom or stat, not inside `aes(...)`.
- `print(p)` and `repr(p)` are not the modern rendering path. Use `p.show()` in scripts.
- `save(width=..., height=...)` uses inches, not pixels. Control raster sharpness with `dpi`.
- `xlim(...)`, `ylim(...)`, and scale limits can remove rows before stats run. Use `coord_cartesian(...)` when you only want to zoom.
- Text repulsion and some smoothing methods need optional dependencies. If labels or LOESS examples fail, install `plotnine[extra]`.
- `plotnine` sits on top of `matplotlib`, so missing fonts, backend quirks, and environment-specific rendering issues usually come from the `matplotlib` side.

## Version-Sensitive Notes

- This doc is for stable `0.15.3`, which PyPI lists as the latest stable release as of March 12, 2026. PyPI also shows `0.16.0a*` prereleases; do not copy prerelease behavior into stable projects unless the project is explicitly pinned there.
- `0.15.0` added plot composition support, so expressions like `(plot_a | plot_b)` and `(plot_a / plot_b)` are valid in the `0.15.x` line.
- The changelog for `0.15.3` notes compatibility cleanup for pandas `3.0.0` copy-on-write warnings, so older blog posts that warn about those warnings may be stale for `0.15.3`.
- If you are migrating from much older `plotnine` releases, check the upstream changelog before relying on `print(plot)` examples or composition behavior from pre-`0.15` content.

## Official Sources

- Docs home: `https://plotnine.org/`
- Reference: `https://plotnine.org/reference/`
- Installation guide: `https://plotnine.org/#installation`
- Guide index: `https://plotnine.org/guide/`
- Changelog: `https://plotnine.org/changelog.html`
- PyPI: `https://pypi.org/project/plotnine/`

---
name: package
description: "HoloViews package guide for Python projects using the official HoloViews docs and reference gallery"
metadata:
  languages: "python"
  versions: "1.22.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "holoviews,holoviz,python,visualization,plotting,jupyter,bokeh,matplotlib,plotly"
---

# HoloViews Python Package Guide

## Golden Rule

Use `holoviews` as a backend-agnostic plotting layer, activate a backend explicitly with `hv.extension(...)`, and prefer the `recommended` extra unless you intentionally want a minimal install. For `1.22.1`, treat `https://holoviews.org/` as the docs root and `https://holoviews.org/reference/` as the fastest way to look up concrete element and backend examples.

## Install

The official install guide distinguishes between a minimal install and the recommended install with plotting dependencies.

Recommended for most agent work:

```bash
python -m pip install "holoviews[recommended]==1.22.1"
```

Minimal package only:

```bash
python -m pip install "holoviews==1.22.1"
```

Conda also has first-party guidance:

```bash
conda install -c conda-forge "holoviews=1.22.1"
```

Use the `recommended` extra when you expect notebook rendering, widgets, or the common Bokeh/Matplotlib/Plotly plotting stack to work without additional package chasing.

## Initialize And Choose A Backend

HoloViews objects are not displayable until a plotting extension is loaded. In notebooks, make backend selection explicit instead of relying on defaults:

```python
import numpy as np
import holoviews as hv

hv.extension("bokeh")

xs = np.linspace(0, 2 * np.pi, 200)
curve = hv.Curve((xs, np.sin(xs)), kdims="x", vdims="y")
curve
```

Supported backends in the official docs are `bokeh`, `matplotlib`, and `plotly`:

```python
hv.extension("matplotlib")
hv.extension("plotly")
```

Practical guidance:

- Use `bokeh` for interactive notebook exploration and most examples in the docs.
- Use `matplotlib` when you need static image output or tighter Matplotlib ecosystem integration.
- Use `plotly` when you need Plotly-native interactivity, but verify backend-specific options against the reference gallery.

## Core Usage

### Build elements from arrays or dataframes

```python
import pandas as pd
import holoviews as hv

hv.extension("bokeh")

df = pd.DataFrame(
    {
        "x": [1, 2, 3, 4],
        "y": [2.5, 3.0, 2.2, 4.1],
        "label": ["a", "b", "c", "d"],
    }
)

points = hv.Points(df, kdims=["x", "y"], vdims=["label"])
points
```

Core object families agents will use most often:

- Elements such as `Curve`, `Scatter`, `Points`, `Image`, `Bars`, and `HeatMap`
- Containers such as `Overlay`, `Layout`, `HoloMap`, and `DynamicMap`
- `Dataset` as the common wrapper when you want consistent dimension metadata and conversion between element types

### Compose plots declaratively

Use `*` for overlays and `+` for layouts:

```python
line = hv.Curve(df, kdims="x", vdims="y").opts(line_width=3, color="black")
markers = hv.Scatter(df, kdims="x", vdims="y").opts(size=8, color="tomato")

overlay = line * markers
layout = overlay + points
```

### Customize with `.opts(...)`

The user guide recommends the `.opts(...)` method for per-object customization:

```python
from holoviews import opts

styled = overlay.opts(
    opts.Curve(width=700, height=350, tools=["hover"]),
    opts.Scatter(size=10),
)
```

If you want shared defaults for a notebook or module, set them once:

```python
opts.defaults(
    opts.Curve(width=700, height=350),
    opts.Scatter(size=8),
)
```

### Use `DynamicMap` for callable, parameter-driven plots

`DynamicMap` is the core HoloViews abstraction for plots generated on demand:

```python
import numpy as np
import holoviews as hv

hv.extension("bokeh")

def sine_curve(phase):
    xs = np.linspace(0, 2 * np.pi, 200)
    return hv.Curve((xs, np.sin(xs + phase)))

dmap = hv.DynamicMap(sine_curve, kdims=["phase"]).redim.values(
    phase=[0.0, 0.5, 1.0, 1.5]
)
```

Use `DynamicMap` when the plot should respond to widgets, streams, or deferred computation. If you only need a finite set of precomputed states, a `HoloMap` is usually simpler to save and debug.

## Export And Embedding

Use `hv.save(...)` for files:

```python
hv.save(overlay, "plot.html")
hv.save(overlay, "plot.png", backend="matplotlib")
```

Practical export rules:

- HTML export is the most reliable default for interactive Bokeh output.
- For static PNG/SVG workflows, prefer the `matplotlib` backend unless you have already validated the target format with another backend.
- Saving uses the active or specified backend, so mismatched backend assumptions are a common cause of export failures.

## Configuration And Runtime Notes

- Authentication: none. `holoviews` is a local plotting library, not a remote API client.
- Notebook setup matters more than credentials: install the plotting dependencies you actually need and load a backend before display.
- The official docs note that `DynamicMap` relies on a running Python process, so fully interactive behavior is available in live notebook or app contexts, not in static HTML alone.
- If you are building a richer dashboard or widget-driven app, HoloViews often pairs with Panel. Keep that as a separate dependency decision instead of assuming `holoviews` alone provides app serving.

## Common Pitfalls

- Installing plain `holoviews` and then expecting rich plotting backends to work. Use `holoviews[recommended]` unless you intentionally manage dependencies yourself.
- Forgetting `hv.extension("bokeh")` or another explicit backend before display.
- Assuming `.opts(...)` settings are backend-neutral. Many options are shared, but some tools, hooks, and styling flags are backend-specific.
- Treating `DynamicMap` like a static object. Interactive updates depend on a live kernel or app session.
- Exporting with the wrong backend. If `hv.save(...)` fails or produces the wrong format, specify `backend=...` explicitly.
- Copying older examples that rely on implicit Matplotlib defaults, notebook magics, or old streamz integrations without checking the release notes first.

## Version-Sensitive Notes For 1.22.1

- PyPI lists `1.22.1` as the current stable release for `holoviews`.
- The `1.22` release line adds Narwhals-based dataframe compatibility, which broadens support for Polars and DuckDB-backed workflows. That matters when agents are adapting examples written for pandas-only inputs.
- The `1.21` release raised the minimum supported Python version to `3.10`.
- The `1.21` release also deprecated the `streamz` interface, autoloading the RC file, IPython magic, and relying on Matplotlib as the default backend for `hv.extension`; the release notes say these are planned for removal in `1.23.0`.

## Official Sources Used

- Docs root: `https://holoviews.org/`
- Install guide: `https://holoviews.org/install.html`
- Getting started: `https://holoviews.org/getting_started/Introduction.html`
- Customizing plots: `https://holoviews.org/user_guide/Customizing_Plots.html`
- Live data: `https://holoviews.org/user_guide/Live_Data.html`
- Exporting: `https://holoviews.org/user_guide/Exporting_and_Archiving.html`
- Release notes: `https://holoviews.org/releases.html`
- Reference gallery: `https://holoviews.org/reference/`
- PyPI: `https://pypi.org/project/holoviews/`

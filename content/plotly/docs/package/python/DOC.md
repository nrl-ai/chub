---
name: package
description: "Plotly Python graphing library for interactive figures, notebook display, HTML export, and static image generation"
metadata:
  languages: "python"
  versions: "6.6.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "plotly,python,visualization,charts,dash,jupyter"
---

# Plotly Python Package Guide

## Golden Rule

Use `plotly` for interactive visualization in Python, start with `plotly.express` for most charts, and drop to `plotly.graph_objects` when you need precise trace or layout control. In Plotly 6.x, assume notebook rendering, image export, and map traces are the areas most likely to break if you copy older examples without checking the current docs.

## Install

Pin the version your project expects:

```bash
python -m pip install "plotly==6.6.0"
```

Useful variants:

```bash
python -m pip install "plotly[express]==6.6.0"
python -m pip install "plotly==6.6.0" "kaleido>=1"
uv add "plotly==6.6.0"
poetry add "plotly==6.6.0"
```

Notes:

- `plotly[express]` installs dependencies used by Plotly Express; you still need a supported dataframe library for dataframe-centric workflows.
- Install `kaleido` if you need `write_image()` or other static export helpers.
- Install `plotly-geo==1.0.0` only for older geo features that rely on the separate shape bundle, such as the county choropleth figure factory.

## Initialize And Display Figures

Minimal Plotly Express example:

```python
import plotly.express as px

fig = px.line(
    x=["2026-03-10", "2026-03-11", "2026-03-12"],
    y=[12, 18, 15],
    title="Requests per day",
    markers=True,
)

fig.update_layout(xaxis_title="Date", yaxis_title="Requests")
fig.show()
```

Minimal Graph Objects example:

```python
import plotly.graph_objects as go

fig = go.Figure()
fig.add_trace(go.Bar(x=["ok", "warn", "error"], y=[42, 7, 2], name="Jobs"))
fig.update_layout(title="Job Status", bargap=0.2)
fig.show()
```

Practical rule:

- Start with `px.*` for fast figure creation.
- Use `update_layout()`, `update_traces()`, and `add_trace()` to customize.
- Switch to `go.Figure(...)` when the chart is assembled incrementally or needs trace types that are easier to control directly.

## Notebook And Renderer Setup

Plotly usually auto-detects a renderer, but notebook environments are the most common source of confusion.

For JupyterLab:

```bash
python -m pip install jupyterlab anywidget
```

For classic Jupyter Notebook:

```bash
python -m pip install "notebook>=7.0" "anywidget>=0.9.13"
```

To force browser rendering from a local Python process:

```python
import plotly.io as pio

pio.renderers.default = "browser"
```

You can also set the renderer through the environment:

```bash
export PLOTLY_RENDERER=browser
```

Use `renderer="browser"` or another renderer name on `fig.show(...)` when you need a one-off override.

## Core Usage Patterns

### Plotly Express returns normal `Figure` objects

Every Plotly Express call returns a `plotly.graph_objects.Figure`, so you can mix high-level and low-level APIs:

```python
import plotly.express as px

fig = px.scatter(
    x=[1, 2, 3, 4],
    y=[10, 14, 13, 17],
    color=["a", "a", "b", "b"],
    title="Experiment results",
)

fig.update_traces(marker_size=12)
fig.update_layout(template="plotly_white")
fig.add_hline(y=15, line_dash="dash")
fig.show()
```

### Build multi-trace figures explicitly

```python
import plotly.graph_objects as go
from plotly.subplots import make_subplots

fig = make_subplots(rows=1, cols=2, subplot_titles=["Latency", "Errors"])
fig.add_trace(go.Scatter(x=[1, 2, 3], y=[120, 90, 140], mode="lines+markers"), row=1, col=1)
fig.add_trace(go.Bar(x=["a", "b", "c"], y=[1, 0, 2]), row=1, col=2)
fig.update_layout(height=450, width=900, title="Service health")
fig.show()
```

### Save interactive HTML

Use HTML export when the recipient needs hover, zoom, legend toggles, or offline sharing:

```python
import plotly.express as px

fig = px.bar(x=["A", "B", "C"], y=[1, 3, 2], title="Interactive export")
fig.write_html("report.html", include_plotlyjs="cdn")
```

Use `fig.to_html(full_html=False)` when embedding a figure into a larger HTML template or web response.

### Save static images

```python
import plotly.express as px

fig = px.bar(x=["A", "B", "C"], y=[1, 3, 2])
fig.write_image("chart.png", width=900, height=500, scale=2)
```

If static export fails, check both `kaleido` and the local Chrome requirement before debugging your figure code.

## Configuration And Auth Notes

`plotly` is a local visualization library, so it does not require API keys for normal figure generation, notebook display, HTML export, or static image export.

Auth-related edge cases:

- Dash apps that display Plotly figures have their own server and deployment configuration, but not Plotly-package auth.
- External tile providers or custom map styles can introduce their own credentials outside Plotly itself.
- Plotly-provided map styles no longer require Mapbox API keys when you use the newer MapLibre-backed map traces.

Useful configuration points:

- `pio.renderers.default` sets the default display backend for the current session.
- `PLOTLY_RENDERER` sets the default renderer at process startup.
- `template` in `update_layout()` controls theme defaults.
- `include_plotlyjs` in `write_html()` trades file size against offline portability.

## Common Pitfalls

- Old notebook instructions are often wrong for Plotly 6.x. Use Notebook 7+ and `anywidget`, not legacy extension setup.
- `go.FigureWidget` now depends on `anywidget`. If a widget example errors in Jupyter, check that package first.
- `fig.show(renderer="browser")` only works when Python runs locally on the same machine as the browser.
- `include_plotlyjs="cdn"` makes exported HTML much smaller, but the file will not render offline.
- `write_image()` needs `kaleido`; with Kaleido v1+, missing Chrome can also cause failures.
- VS Code notebooks can lag the newest Plotly.js features because rendering depends on the editor extension's bundled frontend.
- Plotly Express accepts many dataframe-like inputs, but old examples may assume pandas-specific behavior that changed with Narwhals-backed support in v6.

## Version-Sensitive Notes For Plotly 6.x

- Plotly 6 no longer supports Jupyter Notebook versions earlier than `7`.
- `go.FigureWidget` uses `anywidget` in v6, so older `ipywidgets`-only setup instructions are stale.
- Plotly Express in v6 uses Narwhals, which improves native support for pandas, Polars, and PyArrow inputs.
- Mapbox-based traces are deprecated. For new code, prefer `px.scatter_map`, `px.line_map`, `px.choropleth_map`, `px.density_map`, and the `go.*map` trace types instead of `*_mapbox`.
- When migrating from older map code, update `layout.mapbox` to `layout.map` and `mapbox_style` to `map_style`.
- Plotly's static export docs still mention Orca in older places, but Orca support was scheduled for removal after September 2025. Use Kaleido in new code.
- Plotly 6.2.0 deprecated `engine=` for image export and the old `plotly.io.kaleido.scope` path in favor of `plotly.io.defaults`, so prefer current examples when configuring export defaults.

## Official Sources

- Python API reference: `https://plotly.com/python-api-reference/`
- Getting started: `https://plotly.com/python/getting-started/`
- Plotly Express overview: `https://plotly.com/python/plotly-express/`
- Renderers and display: `https://plotly.com/python/renderers/`
- Interactive HTML export: `https://plotly.com/python/interactive-html-export/`
- Static image export: `https://plotly.com/python/static-image-export/`
- Plotly 6 migration guide: `https://plotly.com/python/v6-migration/`
- MapLibre migration guide: `https://plotly.com/python/mapbox-to-maplibre/`
- PyPI project page: `https://pypi.org/project/plotly/`
- PyPI JSON metadata: `https://pypi.org/pypi/plotly/json`

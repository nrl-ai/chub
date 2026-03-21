---
name: package
description: "Bokeh interactive visualization library for Python plots, dashboards, notebook output, and Bokeh server apps"
metadata:
  languages: "python"
  versions: "3.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "bokeh,python,visualization,plotting,dashboard,jupyter,server"
---

# Bokeh Python Package Guide

## Golden Rule

Use `bokeh` for browser-based interactive plots and dashboards in Python, and choose the output mode first:

- standalone HTML with `show()` or `save()`
- notebook output with `output_notebook()`
- embedded JSON or script/div snippets with `bokeh.embed`
- live apps with `bokeh serve`

Most agent mistakes come from mixing standalone plots and Bokeh server patterns in the same code path.

## Version Drift To Know About

As of `2026-03-12`, the Bokeh docs site and release notes already publish `3.9.0`, but the live PyPI project page still shows `3.8.2` as the latest stable release and lists `3.9.0.dev*` prereleases. Use this doc for the `3.9.0` API surface, but verify what your package index actually exposes before pinning.

## Install

Use a virtual environment and pin the version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "bokeh==3.9.0"
```

If your index still only exposes the current PyPI stable line on `2026-03-12`, use:

```bash
python -m pip install "bokeh==3.8.2"
```

Common alternatives:

```bash
uv add "bokeh==3.9.0"
poetry add "bokeh==3.9.0"
conda install bokeh
```

Notes:

- Bokeh `3.9.0` no longer installs `pandas` as a required dependency. Install `pandas` yourself if your code passes DataFrames into Bokeh helpers.
- Static image export is an extra deployment concern. `export_png()` and `export_svg()` need Selenium plus a compatible browser and driver.
- JupyterLab usage may require the `jupyter_bokeh` extension package depending on your notebook environment.

## Initialize A Standalone Plot

This is the safest default for scripts, CLIs, tests, and generated reports:

```python
from bokeh.io import output_file, show
from bokeh.plotting import figure

p = figure(
    title="Request latency",
    x_axis_label="minute",
    y_axis_label="latency_ms",
    width=800,
    height=400,
    toolbar_location="above",
)

p.line([1, 2, 3, 4], [120, 98, 143, 110], line_width=2, legend_label="p95")
p.circle([1, 2, 3, 4], [120, 98, 143, 110], size=8)

output_file("latency.html", title="Latency Report")
show(p)
```

Use `show()` when you want Bokeh to open a browser tab during local execution. Use `save(p)` when you only need the generated HTML file.

## Core Usage

### Use `ColumnDataSource` for structured data and updates

`ColumnDataSource` is the normal bridge between Python data and glyphs:

```python
from bokeh.models import ColumnDataSource, HoverTool
from bokeh.plotting import figure, show

source = ColumnDataSource(
    data={
        "x": [1, 2, 3],
        "y": [4, 7, 5],
        "service": ["api", "worker", "db"],
    }
)

p = figure(title="Service metrics", width=700, height=350, tools="pan,wheel_zoom,reset,save")
p.scatter("x", "y", source=source, size=12)
p.add_tools(HoverTool(tooltips=[("service", "@service"), ("x", "@x"), ("y", "@y")]))

show(p)
```

For streaming or incremental updates:

```python
source.stream({"x": [4], "y": [8], "service": ["cache"]}, rollover=200)
```

Use `source.patch(...)` when you need partial in-place updates instead of replacing all columns.

### Layout multiple plots and controls

```python
from bokeh.layouts import column, row
from bokeh.models import Div
from bokeh.plotting import figure, show

p1 = figure(width=350, height=250, title="Throughput")
p1.vbar(x=["a", "b", "c"], top=[10, 15, 12], width=0.7)

p2 = figure(width=350, height=250, title="Errors")
p2.line([1, 2, 3], [0, 1, 0], line_width=2)

layout = column(Div(text="<h2>System Summary</h2>"), row(p1, p2))
show(layout)
```

### Notebook output

For notebooks, initialize notebook resources once before calling `show()`:

```python
from bokeh.io import output_notebook, show
from bokeh.plotting import figure

output_notebook()

p = figure(title="Notebook plot", width=600, height=300)
p.line([1, 2, 3], [1, 4, 9], line_width=2)

show(p)
```

If plots render as blank areas in JupyterLab, verify `jupyter_bokeh` and the frontend extension state before changing plotting code.

## Embedding In Web Apps

Use standalone embedding when you already have a Flask, Django, or FastAPI app and do not need live Python callbacks.

### Script and div fragments

```python
from bokeh.embed import components
from bokeh.plotting import figure

p = figure(title="Embedded chart", width=500, height=300)
p.line([1, 2, 3], [3, 2, 4], line_width=2)

script, div = components(p)
```

Render `script` and `div` in your template.

### JSON embedding

`json_item()` is useful when your frontend fetches chart payloads over HTTP:

```python
import json

from bokeh.embed import json_item
from bokeh.plotting import figure

p = figure(title="API chart", width=500, height=300)
p.line([1, 2, 3], [3, 2, 4], line_width=2)

payload = json.dumps(json_item(p, "bokeh-target"))
```

On the client side, hand that payload to BokehJS and mount it into the element with id `bokeh-target`.

## Bokeh Server Apps

Use the server only when the browser must talk back to live Python code. If the chart is static after render, standalone embedding is simpler and more reliable.

Minimal server app:

```python
from bokeh.io import curdoc
from bokeh.layouts import column
from bokeh.models import ColumnDataSource, Slider
from bokeh.plotting import figure

x = [1, 2, 3, 4, 5]
source = ColumnDataSource(data={"x": x, "y": [i * i for i in x]})

p = figure(title="Scaled squares", width=700, height=350)
p.line("x", "y", source=source, line_width=2)

slider = Slider(start=1, end=10, value=1, step=1, title="Scale")

def update(attr, old, new):
    scale = slider.value
    source.data = {"x": x, "y": [scale * i * i for i in x]}

slider.on_change("value", update)

curdoc().add_root(column(slider, p))
curdoc().title = "Bokeh Server Demo"
```

Run it with:

```bash
bokeh serve --show app.py
```

Deployment notes:

- Public deployments must set explicit WebSocket origins with `--allow-websocket-origin` or equivalent server config.
- Keep origin settings narrow. Using `*` is only acceptable for local development.
- Server apps are stateful. Avoid mutating shared global objects in ways that leak data across sessions.
- If you need application auth, session management, or signed embedding, treat that as part of the surrounding web app deployment, not as a plotting concern.

## Configuration Notes

- Standalone HTML output does not require credentials or auth.
- Use `output_file()` when the deliverable is an HTML artifact.
- Use `output_notebook()` only in notebook runtimes.
- Use `components()` or `json_item()` when another web framework owns the page shell.
- Use a Bokeh server app only when Python-side interactivity must persist after initial render.
- For air-gapped or restricted environments, verify how BokehJS resources are served before assuming CDN access is allowed.

## Common Pitfalls

- Do not mix server callbacks like `curdoc()` and `on_change()` with a plain standalone script unless you are actually running under `bokeh serve`.
- `ColumnDataSource` columns must stay the same length. Mismatched arrays produce hard-to-debug rendering errors.
- If you pass pandas objects in `3.9.0`, install `pandas` explicitly. It is no longer a guaranteed transitive dependency.
- `show()` opens a browser during local execution. In CI or headless jobs, prefer `save()` or explicit export workflows.
- Static image export failures usually come from missing Selenium, Firefox or Chromium, or the matching browser driver, not from plot code.
- Blank notebook output usually means a frontend resource issue, not a bad glyph definition.
- Bokeh server apps exposed behind a reverse proxy often fail because WebSocket origin settings were not updated.
- Copying old Bokeh 2.x blog posts is risky. The modern `3.x` docs use newer APIs, stricter browser support, and different dependency assumptions.

## Version-Sensitive Notes

- `3.9.0` release notes call out Python `3.14` support and compatibility with `pandas 3`.
- `3.9.0` removes `pandas` as a required dependency, which matters for agent-generated examples that assume DataFrame support is always present.
- `3.9.0` adds support for dataclasses in `ColumnDataSource` and improves map plotting with new `figure` methods for Cartopy coordinates.
- `3.8.2` contains an important Bokeh server security fix for incomplete origin validation. If you deploy server apps, do not stay below `3.8.2`.
- Because the docs site is ahead of the stable PyPI page on `2026-03-12`, verify whether your environment is on `3.8.x`, a `3.9.0.dev*` prerelease, or the final `3.9.0` release before copying examples blindly.

## Official Links

- Docs root: `https://docs.bokeh.org/en/latest/`
- Reference: `https://docs.bokeh.org/en/latest/docs/reference.html`
- Installation: `https://docs.bokeh.org/en/latest/docs/first_steps/installation.html`
- First steps: `https://docs.bokeh.org/en/latest/docs/first_steps/first_steps.html`
- Embedding: `https://docs.bokeh.org/en/latest/docs/user_guide/output/embed.html`
- Jupyter output: `https://docs.bokeh.org/en/latest/docs/user_guide/output/jupyter.html`
- Server guide: `https://docs.bokeh.org/en/latest/docs/user_guide/server.html`
- Releases: `https://docs.bokeh.org/en/latest/docs/releases.html`
- PyPI: `https://pypi.org/project/bokeh/`

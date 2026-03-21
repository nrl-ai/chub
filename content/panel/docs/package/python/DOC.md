---
name: package
description: "panel package guide for Python dashboards and apps with widgets, templates, notebooks, and server deployment"
metadata:
  languages: "python"
  versions: "1.8.7"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "panel,holoviz,bokeh,dashboard,widgets,jupyter,python"
---

# panel Python Package Guide

## Golden Rule

Use `panel` as `import panel as pn`, call `pn.extension()` before rendering widgets or templates, and choose one execution model deliberately: notebook output, `panel serve`, or `pn.serve(...)`. For most app logic, prefer Panel's reactive APIs such as `pn.bind`, `pn.rx`, and `@pn.depends` instead of hand-wiring lower-level callbacks.

## Version-Sensitive Notes

- As of March 12, 2026, the official Panel release docs and PyPI package page point to `1.8.7`, not the older version reference `1.8.9`.
- The docs site currently mixes version labels across pages. Some pages or assets show `v1.8.9`, while the release page still advertises `1.8.7`. Use PyPI and the release notes as the source of truth for exact package pinning.
- The installation docs say Panel supports Python `3.9+`. Confirm the exact interpreter support from the version you pin in your environment and lockfile before changing CI or deployment images.
- The docs URL points at the API reference. For package-level work, the installation, app-building, reactive API, serving, and authentication guides are more useful than starting from `/api/`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "panel==1.8.7"
```

Conda-based installs are also first-class in the official docs:

```bash
conda install -c conda-forge panel
mamba install -c conda-forge panel
```

Basic verification:

```bash
python - <<'PY'
import panel as pn
print(pn.__version__)
PY
```

## Initialization And Setup

Panel can run inside notebooks or as a served app. In both cases, initialize the frontend first:

```python
import panel as pn

pn.extension(sizing_mode="stretch_width")
```

Notes:

- Call `pn.extension()` once near process startup or near the top of a notebook.
- Pass optional frontend extensions such as `"tabulator"` or `"plotly"` if you use components that need extra JavaScript resources.
- Keep Panel installed in the same environment as your Jupyter kernel and JupyterLab server to avoid frontend-extension confusion.

## Core Usage

### Minimal served app

Use `.servable()` in files that will be launched with `panel serve`:

```python
import panel as pn

pn.extension(sizing_mode="stretch_width")

name = pn.widgets.TextInput(name="Name", value="Panel")
greeting = pn.bind(lambda value: f"Hello {value}", name)

app = pn.Column(name, greeting)
app.servable()
```

Run it locally:

```bash
panel serve app.py --show --dev
```

Use `--dev` during local development for autoreload-style iteration. For production, serve the app without dev flags and put it behind your normal process manager or reverse proxy.

### Notebook usage

In notebooks, you usually display Panel objects directly after calling `pn.extension()`:

```python
import panel as pn

pn.extension()

slider = pn.widgets.IntSlider(name="Value", start=0, end=10, value=3)
view = pn.bind(lambda n: f"Current value: {n}", slider)

pn.Column(slider, view)
```

### Use `pn.bind` for simple reactive flows

`pn.bind` is usually the fastest path from widgets to working app logic:

```python
import panel as pn

pn.extension()

species = pn.widgets.Select(name="Species", options=["cat", "dog", "otter"])

def describe(choice: str) -> str:
    return f"You selected {choice}"

pn.Row(species, pn.bind(describe, species))
```

Use this when you want a plain Python function to react to widget or parameter values.

### Use `@pn.depends` when composing with Param or reusable functions

```python
import panel as pn

pn.extension()

slider = pn.widgets.FloatSlider(name="Scale", start=0, end=2, step=0.1, value=1.0)

@pn.depends(slider)
def scaled_message(scale):
    return f"Scale is {scale:.1f}"

pn.Column(slider, scaled_message)
```

This pattern stays useful when you are working with Param-based classes or methods.

### Use templates for full-page apps

For multi-section apps, start with a template instead of hand-building a page shell:

```python
import panel as pn

pn.extension(sizing_mode="stretch_width")

template = pn.template.FastListTemplate(title="Panel Demo")
template.main.append(pn.pane.Markdown("## Hello from Panel"))
template.main.append(pn.widgets.IntSlider(name="Rows", start=10, end=100, value=25))
template.servable()
```

### Use `pn.serve(...)` when embedding Panel in a Python process

If you need to launch apps programmatically:

```python
import panel as pn

pn.extension()

def make_app():
    slider = pn.widgets.IntSlider(name="Value", start=0, end=10, value=5)
    return pn.Column(slider, pn.bind(lambda v: v * 2, slider))

pn.serve({"app": make_app}, show=False, port=5006)
```

This is useful when integrating Panel with a larger Python application or a custom launch script.

## Configuration And Authentication

There is no package-level API key or service auth requirement for local Panel use. Configuration is mostly about frontend initialization, server options, and optional app protection for deployed dashboards.

Practical defaults:

- Use `pn.extension(...)` for frontend setup and optional component extensions.
- Use `pn.config` and server flags when you need global behavior changes instead of scattering per-widget overrides.
- Treat public serving as a deployment concern, not just a Python import concern.

For protected apps, the official auth docs cover:

- basic auth via `panel serve` for simple internal deployments
- OAuth providers configured with command-line flags or `PANEL_...` environment variables
- cookie-secret and encryption-key settings for secure session handling

Common OAuth-related environment variables from the auth docs include:

- `PANEL_OAUTH_KEY`
- `PANEL_OAUTH_SECRET`
- `PANEL_COOKIE_SECRET`

If you expose a Panel app publicly, configure auth explicitly instead of assuming the default dev server setup is safe enough.

## Common Pitfalls

- Forgetting `pn.extension()` before rendering widgets, templates, or panes.
- Using `.show()` in code that should be served with `panel serve`; prefer `.servable()` for served apps.
- Installing Panel in one environment while Jupyter runs from another.
- Forgetting to load required frontend extensions for components such as Tabulator or Plotly-backed panes.
- Reaching for low-level callback wiring when `pn.bind` or `@pn.depends` is enough.
- Treating `--dev` as a production flag; it is for local iteration.
- Publishing a dashboard without auth, cookie-secret, or reverse-proxy planning.
- Trusting page-level version banners over PyPI when the docs site shows mixed version labels.

## Official Source URLs

- `https://panel.holoviz.org/`
- `https://panel.holoviz.org/getting_started/installation.html`
- `https://panel.holoviz.org/getting_started/build_app.html`
- `https://panel.holoviz.org/explanation/api/reactive.html`
- `https://panel.holoviz.org/tutorials/basic/serve.html`
- `https://panel.holoviz.org/how_to/server/commandline.html`
- `https://panel.holoviz.org/how_to/authentication/configuration.html`
- `https://panel.holoviz.org/about/releases.html`
- `https://pypi.org/project/panel/`
- `https://github.com/holoviz/panel`

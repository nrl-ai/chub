---
name: package
description: "ipywidgets package guide for Python notebooks and JupyterLab with install, display, events, layouts, and ipywidgets 8 migration notes"
metadata:
  languages: "python"
  versions: "8.1.8"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "ipywidgets,jupyter,notebook,jupyterlab,widgets,ui,traitlets"
---

# ipywidgets Python Package Guide

## Golden Rule

Use `ipywidgets` in the kernel environment, then make sure the matching frontend support is installed where Jupyter itself runs. If a widget renders as plain text like `IntSlider(value=0)`, the Python package is present but the browser-side widget manager is missing, still loading, or enabled in the wrong environment.

## Install

Most projects only need the package in the active kernel environment:

```bash
python -m pip install "ipywidgets==8.1.8"
```

Common alternatives:

```bash
uv add "ipywidgets==8.1.8"
poetry add "ipywidgets==8.1.8"
conda install -c conda-forge ipywidgets
```

### JupyterLab setup

If JupyterLab and the kernel use the same environment, installing `ipywidgets` is usually enough.

If they are split across environments:

1. Install `jupyterlab_widgets` where JupyterLab runs.
2. Install `ipywidgets` in each kernel environment that should display widgets.

Example:

```bash
conda install -n base -c conda-forge jupyterlab_widgets
conda install -n pyenv -c conda-forge ipywidgets
```

Official `ipywidgets` install docs still call out JupyterLab 3.x, but the current `jupyterlab_widgets` docs say the extension enables support in JupyterLab 3.x or 4.x.

### Classic Notebook setup

If Notebook and the kernel use the same environment, `ipywidgets` usually pulls in the needed frontend pieces automatically.

If they are split:

1. Install `widgetsnbextension` in the Notebook server environment.
2. Install `ipywidgets` in each kernel environment.

Example:

```bash
conda install -n base -c conda-forge widgetsnbextension
conda install -n pyenv -c conda-forge ipywidgets
```

For Notebook `5.2` or earlier, enable the extension manually:

```bash
jupyter nbextension enable --py widgetsnbextension --sys-prefix
```

### Legacy JupyterLab 1 or 2

For JupyterLab 1.x or 2.x, you still need the lab extension and Node.js:

```bash
jupyter labextension install @jupyter-widgets/jupyterlab-manager
```

Do not use this path for JupyterLab 3 or 4 unless you are doing extension development.

## Quick Start

```python
import ipywidgets as widgets
from IPython.display import display

slider = widgets.IntSlider(
    value=5,
    min=0,
    max=10,
    step=1,
    description="Count",
    continuous_update=False,
)

display(slider)
```

Widgets display automatically when they are the last value in a cell, but `display(...)` is the safer pattern when composing multiple widgets.

## Core Usage

### Read and set widget state

All widgets expose synchronized trait values such as `value`, `description`, `disabled`, `layout`, and `style`.

```python
import ipywidgets as widgets
from IPython.display import display

slider = widgets.IntSlider(value=0, min=-5, max=5, description="X")
display(slider)

print(slider.value)   # 0
slider.value = 3
```

### Observe changes with traitlets

Use `.observe(...)` for value-driven UI logic. The callback receives a `change` mapping with keys like `owner`, `old`, `new`, and `name`.

```python
import ipywidgets as widgets
from IPython.display import display

slider = widgets.IntSlider(description="Threshold", continuous_update=False)
label = widgets.Label(value="Threshold is 0")

def handle_change(change):
    label.value = f"Threshold is {change['new']}"

slider.observe(handle_change, names="value")
display(widgets.VBox([slider, label]))
```

For expensive callbacks, set `continuous_update=False` on sliders or use `interact_manual` so you do not rerun heavy work on every drag event.

### Button events and debug output

In JupyterLab, `print(...)` from widget callbacks is easy to lose. Route callback output through an `Output` widget.

```python
import ipywidgets as widgets
from IPython.display import display

button = widgets.Button(description="Run", button_style="primary", icon="play")
out = widgets.Output(layout={"border": "1px solid #ddd"})

@out.capture(clear_output=True)
def run_clicked(_):
    print("Button clicked")

button.on_click(run_clicked)
display(button, out)
```

### Build simple controls with `interact`

`interact` is the quickest way to turn function parameters into widgets.

```python
from ipywidgets import interact, FloatSlider

def area(radius):
    return 3.14159 * radius ** 2

interact(area, radius=FloatSlider(min=0, max=10, step=0.5, continuous_update=False))
```

Use `interact_manual(...)` when the callback is slow and should only run on demand.

### Compose layouts and linked controls

Widgets are regular objects, so build layouts explicitly with `VBox`, `HBox`, `GridBox`, `Tab`, `Accordion`, and `Stack`.

```python
import ipywidgets as widgets
from IPython.display import display

source = widgets.IntSlider(description="Source")
target = widgets.IntText(description="Target")
widgets.link((source, "value"), (target, "value"))

ui = widgets.VBox([
    widgets.HTML("<b>Linked controls</b>"),
    widgets.HBox([source, target]),
])

display(ui)
```

Use `widgets.jslink(...)` when you want the sync to happen in the browser without a kernel round-trip.

### Separate controls from rendered output

`interactive_output(...)` is useful when you want a custom layout instead of the default `interact` UI.

```python
import ipywidgets as widgets
from IPython.display import display

x = widgets.IntSlider(description="a")
y = widgets.IntSlider(description="b")

def render(a, b):
    print(f"{a} + {b} = {a + b}")

out = widgets.interactive_output(render, {"a": x, "b": y})
display(widgets.HBox([widgets.VBox([x, y]), out]))
```

### File uploads in ipywidgets 8

`FileUpload` in `8.x` returns uploaded files through `.value` as file objects with fields such as `name`, `type`, `size`, `last_modified`, and `content`.

```python
import io
import ipywidgets as widgets
import pandas as pd
from IPython.display import display

uploader = widgets.FileUpload(accept=".csv", multiple=False)
display(uploader)

def handle_upload(change):
    if not uploader.value:
        return
    uploaded = uploader.value[0]
    df = pd.read_csv(io.BytesIO(uploaded.content))
    print(uploaded.name, df.head())

uploader.observe(handle_upload, names="value")
```

To convert back to the old name-to-bytes shape:

```python
files = {f.name: f.content.tobytes() for f in uploader.value}
```

## Configuration Notes

- There is no API key or auth flow.
- The important configuration boundary is environment placement:
  - kernel environment needs `ipywidgets`
  - JupyterLab environment may need `jupyterlab_widgets`
  - classic Notebook environment may need `widgetsnbextension`
- Browser support matters for some widgets. For example, date and time pickers rely on browser support for the underlying HTML inputs.
- Displaying the same widget object multiple times creates multiple synchronized views of the same backend state.

## Common Pitfalls

- Seeing `IntSlider(value=0)` instead of a rendered control usually means the widget frontend is missing, disabled, or installed in the wrong environment.
- Callback output often disappears unless you route it through `widgets.Output()` or write directly into other widgets like `HTML` or `Label`.
- `FileUpload` stores uploaded content in memory and widget state can end up saved into the notebook. Avoid this for large or sensitive files.
- `Accordion`, `Tab`, and `Stack` use `selected_index`, not `value`.
- JupyterLab 1.x and 2.x still need a manual labextension install plus Node.js.
- `continuous_update=True` can flood the kernel with events during slider drags. Turn it off for expensive work.

## Version-Sensitive Notes For 8.1.8

- `8.1.8` itself is a small release. The upstream release notes call out an update to `jupyterlab_widgets` metadata to indicate compatibility with JupyterLab 4.
- `8.1.7` removed Python 3.8 in the release notes. Combined with the current PyPI classifiers, that makes Python `3.9+` the practical target for new work even though PyPI metadata still says `>=3.7`. This is an inference from the official sources.
- In `8.x`, `FileUpload` changed substantially:
  - `.value` is no longer the old name-to-content mapping
  - `.data` and `.metadata` were removed
- `description_tooltip` was deprecated in favor of `tooltip`.
- Widget descriptions are sanitized on the client side. If you need HTML in `description`, set `description_allow_html=True` only for trusted content.
- `Layout.overflow_x` and `Layout.overflow_y` were removed; use `overflow`.
- `ipywidgets` no longer pulls in `notebook` transitively through `widgetsnbextension`. If your workflow needs classic Notebook, install `notebook` explicitly.

## Official Sources

- Stable docs: `https://ipywidgets.readthedocs.io/en/stable/`
- Installation: `https://ipywidgets.readthedocs.io/en/stable/user_install.html`
- Widget basics: `https://ipywidgets.readthedocs.io/en/stable/examples/Widget%20Basics.html`
- Widget list: `https://ipywidgets.readthedocs.io/en/8.1.4/examples/Widget%20List.html`
- Migration guide: `https://ipywidgets.readthedocs.io/en/8.1.4/user_migration_guides.html`
- JupyterLab manager docs: `https://ipywidgets.readthedocs.io/en/latest/_static/typedoc/modules/_jupyter_widgets_jupyterlab_manager.html`
- PyPI package page: `https://pypi.org/project/ipywidgets/`
- GitHub releases: `https://github.com/jupyter-widgets/ipywidgets/releases`

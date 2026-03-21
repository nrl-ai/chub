---
name: package
description: "Altair Python package guide for declarative charts with Vega-Lite 6"
metadata:
  languages: "python"
  versions: "6.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "altair,vega-lite,visualization,charts,jupyter,data-viz"
---

# Altair Python Package Guide

## Golden Rule

Use `altair` to build Vega-Lite chart specifications in Python, not to draw pixels directly. Pass Altair tabular data, annotate encoding types when the dataframe metadata is not enough, and expect rendering or export features to depend on the frontend renderer plus optional `vl-convert-python`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "altair==6.0.0"
```

Common alternatives:

```bash
uv add "altair==6.0.0"
poetry add "altair==6.0.0"
```

Useful extras from the official install guide:

```bash
python -m pip install "altair[all]==6.0.0"
python -m pip install "altair[save]==6.0.0"
```

Use cases:

- `altair[all]`: installs Altair with all optional dependencies, which is the easiest choice for notebooks and examples.
- `altair[save]`: installs the dependencies needed for offline HTML export and PNG/SVG/PDF export.

If you need image export or offline HTML without extras, install `vl-convert-python` directly:

```bash
python -m pip install vl-convert-python
```

## Initialize And Render A First Chart

Altair works best with tabular data. A pandas `DataFrame` is the smoothest path because Altair can infer many encoding types from pandas dtypes.

```python
import altair as alt
import pandas as pd

source = pd.DataFrame(
    [
        {"month": "Jan", "revenue": 12, "region": "east"},
        {"month": "Feb", "revenue": 18, "region": "east"},
        {"month": "Jan", "revenue": 10, "region": "west"},
        {"month": "Feb", "revenue": 15, "region": "west"},
    ]
)

chart = (
    alt.Chart(source)
    .mark_line(point=True)
    .encode(
        x="month:O",
        y="revenue:Q",
        color="region:N",
        tooltip=["month:O", "region:N", "revenue:Q"],
    )
    .properties(width=500, title="Monthly revenue")
)

chart
```

Notes:

- `:O`, `:Q`, `:N`, and `:T` mean ordinal, quantitative, nominal, and temporal.
- With pandas input you can sometimes omit the type suffixes, but keeping them explicit avoids ambiguous output when code is reused.
- In notebooks, the chart usually renders when it is the final expression in the cell.

## Core Usage

### Use built-in datasets for examples

Altair 6 includes the `altair.datasets` module for convenient access to Vega datasets:

```python
import altair as alt
from altair.datasets import data

cars = data.cars()

chart = (
    alt.Chart(cars)
    .mark_point()
    .encode(
        x="Horsepower:Q",
        y="Miles_per_Gallon:Q",
        color="Origin:N",
        tooltip=["Name:N", "Origin:N", "Horsepower:Q", "Miles_per_Gallon:Q"],
    )
)
```

### Compose charts with transformations and views

Altair charts are immutable-like objects. Build them up with method chaining:

```python
base = alt.Chart(cars).properties(width=280, height=220)

points = base.mark_point().encode(
    x="Horsepower:Q",
    y="Miles_per_Gallon:Q",
    color="Origin:N",
)

bars = base.mark_bar().encode(
    x="count()",
    y="Origin:N",
    color="Origin:N",
)

dashboard = points | bars
```

Use `|` for horizontal concatenation, `&` for vertical concatenation, and `+` for layering.

### Add interactivity with parameters and selections

In Altair 6, the practical interaction model is still parameters plus `add_params()`. Prefer `alt.when(...)` over the older `alt.condition(...)` style.

```python
import altair as alt
from altair.datasets import data

cars = data.cars()
brush = alt.selection_interval()

points = (
    alt.Chart(cars)
    .mark_point()
    .encode(
        x="Horsepower:Q",
        y="Miles_per_Gallon:Q",
        color=alt.when(brush).then("Origin:N").otherwise(alt.value("lightgray")),
    )
    .add_params(brush)
)

bars = (
    alt.Chart(cars)
    .mark_bar()
    .encode(
        x="count()",
        y="Origin:N",
        color="Origin:N",
    )
    .transform_filter(brush)
)

interactive = points & bars
```

### Save or serialize charts

```python
chart.save("chart.json")
chart.save("chart.html")
chart.save("chart.html", inline=True)
chart.save("chart.png")

spec = chart.to_dict()
vega_url = chart.to_url()
```

Behavior to remember:

- `.json` and standard `.html` output work from `Chart.save()`.
- `inline=True` makes HTML self-contained for offline viewing, but requires `vl-convert-python`.
- PNG, SVG, and PDF export also require `vl-convert-python`.
- `chart.to_url()` is useful for opening the chart in the online Vega editor during debugging.

## Configuration And Environment

Altair itself does not use API keys or service authentication. The main setup questions are renderer choice, data handling, and export dependencies.

### Pick a renderer that matches the runtime

```python
import altair as alt

alt.renderers.enable("html")                # default in many environments
alt.renderers.enable("jupyter")             # widget-based rendering in Jupyter frontends
alt.renderers.enable("jupyter", offline=True)
alt.renderers.enable("browser")             # useful from IPython or local scripts
```

Guidance:

- Default HTML rendering usually works in JupyterLab, Notebook, VS Code, and other notebook-like frontends.
- HTML and Jupyter renderers load JavaScript from a CDN unless you choose an offline mode that uses `vl-convert-python`.
- The `"browser"` renderer is useful outside notebooks, but it is not a fit for remote notebook environments.
- In a plain Python script or REPL, `chart.show()` is often clearer than relying on implicit display.

### Handle larger datasets intentionally

Altair raises `MaxRowsError` when it would embed more than 5000 rows directly in a spec. Prefer one of these approaches:

1. Pre-aggregate or filter in pandas before passing data to Altair.
2. Enable VegaFusion for larger transformed datasets.
3. Pass data by URL if the runtime and deployment model support it.
4. Disable the max row check only when you explicitly want to embed the full dataset.

Install VegaFusion first:

```bash
python -m pip install vegafusion vl-convert-python
```

```python
import altair as alt

alt.data_transformers.enable("vegafusion")
```

Important details:

- With the VegaFusion data transformer enabled, charts created afterward can work with datasets up to 100,000 rows after supported transformations are evaluated.
- When VegaFusion is active, converting to JSON or dict should use `format="vega"` rather than the default Vega-Lite format.

```python
spec = chart.to_dict(format="vega")
```

## Common Pitfalls

### Non-pandas data needs explicit encoding types

For plain dict records, URLs, or DataFrame-interchange inputs, declare the type suffixes explicitly:

```python
alt.Chart(records).mark_bar().encode(
    x="category:N",
    y="value:Q",
)
```

### DataFrame indices are not chart columns

If the index matters, move it into a column first:

```python
chart = alt.Chart(df.reset_index()).mark_line().encode(
    x="index:T",
    y="value:Q",
)
```

### Wide data is often the wrong shape

Altair’s grammar is usually easier with long-form data. Use pandas `melt()` or an Altair fold transform when multiple series are stored as separate columns.

### Rendering problems are often frontend problems

If a chart object prints but does not render:

- check that the notebook or IDE supports the chosen renderer
- confirm the environment can load required JavaScript assets
- switch to `alt.renderers.enable("jupyter")` or `alt.renderers.enable("browser")`
- export to `chart.html` to isolate frontend issues from chart-spec issues

### Export failures usually mean a missing optional dependency

If PNG, SVG, PDF, or offline HTML export fails, install `vl-convert-python`. Do not reach for `altair_saver`; the official docs treat it as an old Altair 4-era path that has been superseded.

## Version-Sensitive Notes For 6.0.0

- Altair `6.0.0` targets Vega-Lite 6.x.
- PyPI lists Python `>=3.9`, and the 6.0.0 release notes call out Python 3.14 support.
- Altair 6 introduces the `altair.datasets` module for lazy access to Vega datasets.
- The release notes also call out thread-safety improvements, so chart specs should be more stable after reruns.
- The interaction docs still describe major parameter-system changes from Altair 5, and the current guidance is to prefer `alt.when()` over `alt.condition()`.

## Official Sources

- Docs root: https://altair-viz.github.io/
- Installation: https://altair-viz.github.io/getting_started/installation.html
- Data guide: https://altair-viz.github.io/user_guide/data.html
- Display/renderers: https://altair-viz.github.io/user_guide/display_frontends.html
- Large datasets: https://altair-viz.github.io/user_guide/large_datasets.html
- Interactions: https://altair-viz.github.io/user_guide/interactions/parameters.html
- Saving charts: https://altair-viz.github.io/user_guide/saving_charts.html
- PyPI package page: https://pypi.org/project/altair/
- GitHub releases: https://github.com/vega/altair/releases

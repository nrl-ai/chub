---
name: package
description: "hvPlot plotting API for pandas, xarray, polars, DuckDB, and related Python data objects"
metadata:
  languages: "python"
  versions: "0.12.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "hvplot,holoviz,plotting,pandas,xarray,polars,bokeh,panel"
---

# hvPlot Python Package Guide

## Golden Rule

Use `hvplot` as a high-level plotting layer on top of HoloViews, and import the accessor module for the data type you actually use before calling `.hvplot(...)`. The default rendering backend is Bokeh; if you need Matplotlib or Plotly, set it explicitly with `hvplot.extension(...)` near process startup.

## Install

Pin the version your project expects:

```bash
python -m pip install "hvplot==0.12.2"
```

Common alternatives:

```bash
uv add "hvplot==0.12.2"
poetry add "hvplot==0.12.2"
```

Notes:

- `hvplot` is the plotting layer. Install the data-library packages you actually plot, such as `pandas`, `xarray`, `polars`, `duckdb`, `ibis`, or `intake`.
- Basic Bokeh-backed plotting works with the package install above. If you switch to a different backend or build richer apps, make sure the rest of that stack is installed and initialized.

## Initialize

### Pandas or Polars accessors

Import the adapter module once, then use the `.hvplot` accessor on the object:

```python
import pandas as pd
import hvplot.pandas

df = pd.DataFrame(
    {
        "x": [1, 2, 3, 4],
        "y": [3, 1, 4, 2],
        "group": ["a", "a", "b", "b"],
    }
)

plot = df.hvplot.line(x="x", y="y", by="group", title="Series by group")
```

```python
import polars as pl
import hvplot.polars

df = pl.DataFrame({"x": [1, 2, 3], "y": [2, 4, 1]})
plot = df.hvplot.scatter(x="x", y="y")
```

### Xarray accessor

```python
import xarray as xr
import hvplot.xarray

da = xr.DataArray(
    [[1, 2, 3], [4, 5, 6]],
    dims=("row", "col"),
    coords={"row": ["a", "b"], "col": [0, 1, 2]},
)

plot = da.hvplot.image(x="col", y="row")
```

### Explicit object API

Use `hvPlot(...)` when you want the plotting helper without relying on an accessor import:

```python
import pandas as pd
from hvplot import hvPlot

df = pd.DataFrame({"x": [1, 2, 3], "y": [3, 1, 2]})

plot = hvPlot(df).scatter(x="x", y="y")
```

## Backend And Display Setup

The default backend is Bokeh. Set a different backend before generating plots if your environment expects Matplotlib or Plotly:

```python
import hvplot

hvplot.extension("bokeh")
# hvplot.extension("matplotlib")
# hvplot.extension("plotly")
```

For notebook and app-style workflows, initialize Panel early when you expect widgets or servable views:

```python
import panel as pn
import hvplot

pn.extension()
hvplot.extension("bokeh")
```

If you want pandas-style `.plot(...)` calls to route through hvPlot, configure the pandas plotting backend:

```python
import pandas as pd
import hvplot.pandas

pd.options.plotting.backend = "hvplot"
```

Use that mode for common plots, not as a guarantee of full pandas plotting API compatibility.

## Core Usage

### Common chart types

```python
import pandas as pd
import hvplot.pandas

df = pd.DataFrame(
    {
        "time": [1, 2, 3, 4],
        "value": [5, 3, 6, 8],
        "category": ["a", "a", "b", "b"],
    }
)

line = df.hvplot.line(x="time", y="value")
scatter = df.hvplot.scatter(x="time", y="value", by="category")
hist = df.hvplot.hist(y="value", bins=10)
```

### Grouping, faceting, and layout

```python
import pandas as pd
import hvplot.pandas

df = pd.DataFrame(
    {
        "x": [1, 2, 3, 1, 2, 3],
        "y": [3, 1, 4, 5, 2, 6],
        "species": ["adelie", "adelie", "adelie", "chinstrap", "chinstrap", "chinstrap"],
        "island": ["Torgersen", "Torgersen", "Biscoe", "Dream", "Dream", "Dream"],
    }
)

plot = df.hvplot.scatter(
    x="x",
    y="y",
    groupby="species",
    by="island",
    responsive=True,
)
```

### Interactive pipeline

`interactive` lets you build reactive data pipelines that stay linked to widgets:

```python
import pandas as pd
import panel as pn
import hvplot.pandas

df = pd.DataFrame(
    {
        "x": [1, 2, 3, 4],
        "y": [3, 2, 4, 5],
        "kind": ["a", "a", "b", "b"],
    }
)

selector = pn.widgets.Select(name="Kind", options=["a", "b"], value="a")
idf = df.interactive()

view = idf[idf.kind == selector].hvplot.line(x="x", y="y")
```

This is most useful in a live notebook or a Panel app, not in static markdown output.

### Explorer UI

Use `explorer(...)` when you want a quick visual exploration surface instead of hand-writing the plot config:

```python
import pandas as pd
from hvplot import explorer

df = pd.DataFrame(
    {
        "x": [1, 2, 3, 4],
        "y": [4, 1, 3, 2],
        "group": ["a", "a", "b", "b"],
    }
)

ui = explorer(df)
```

If you are running a script or app, serve it explicitly:

```python
ui.servable()
```

### Save or show outside a notebook

For script-driven output, prefer saving HTML artifacts:

```python
import pandas as pd
import hvplot
import hvplot.pandas

df = pd.DataFrame({"x": [1, 2, 3], "y": [3, 1, 2]})
plot = df.hvplot.line(x="x", y="y")

hvplot.save(plot, "plot.html")
```

Use `hvplot.show(plot)` or a Panel server only when you need a live rendered view.

## Configuration Notes

- `hvplot` itself does not handle service authentication. Credentials belong to the data-source layer you are plotting from, such as DuckDB connections, Ibis backends, cloud warehouse clients, or remote filesystems.
- Backend selection is process-global enough that it should be treated as startup configuration. Set it once near import time instead of switching backends mid-request.
- For reusable dashboards, prefer a Panel app or saved HTML over relying on implicit notebook display hooks.

## Common Pitfalls

- `import hvplot` alone does not register `.hvplot` on pandas, xarray, polars, DuckDB, or other supported objects. Import the matching adapter module first.
- The pandas plotting backend mode is intentionally not a full emulation of pandas plotting. It covers common cases, but not every pandas `.plot(...)` feature or edge case.
- `interactive()` and `explorer()` create live widget-driven objects. They will not behave like plain static images in logs, CI output, or markdown rendering.
- If a notebook cell shows nothing useful, check that you initialized the frontend extensions and that the backend you selected is available in that environment.
- `hvplot` can plot many data libraries, but the data adapter and the underlying package still need to be installed. Support is not magical just because `hvplot` itself imports cleanly.

## Version-Sensitive Notes For 0.12.2

- `0.12.2` is the current PyPI release and requires Python `>=3.10`.
- The `0.12.x` line dropped Python 3.9 support starting with `0.12.0`, so upgrade your runtime before pinning this version.
- `0.12.2` includes fixes for DuckDB support and Pandas 3.0 compatibility. If your project mixes `hvplot` with newer pandas or DuckDB releases, prefer `0.12.2` or later in the `0.12.x` line.

## Official Sources

- Docs root: https://hvplot.holoviz.org/
- User guide: https://hvplot.holoviz.org/getting_started/index.html
- Supported data libraries: https://hvplot.holoviz.org/ref/data_libraries.html
- Plotting extensions and backends: https://hvplot.holoviz.org/ref/plotting_extensions/index.html
- Interactive guide: https://hvplot.holoviz.org/user_guide/Interactive.html
- Explorer guide: https://hvplot.holoviz.org/user_guide/Explorer.html
- Viewing and exporting: https://hvplot.holoviz.org/user_guide/Viewing.html
- Release notes: https://hvplot.holoviz.org/releases.html
- PyPI: https://pypi.org/project/hvplot/
- GitHub releases: https://github.com/holoviz/hvplot/releases

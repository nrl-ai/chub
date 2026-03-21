---
name: package
description: "Datashader package guide for Python rasterization of large tabular and gridded datasets"
metadata:
  languages: "python"
  versions: "0.18.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "datashader,python,visualization,rasterization,pandas,dask,xarray,holoviz"
---

# Datashader Python Package Guide

## Golden Rule

Use `datashader` to rasterize data into a fixed-size aggregate first, then colorize that aggregate. The core workflow is:

1. Build a `Canvas`
2. Aggregate with a glyph method such as `points()`, `line()`, `raster()`, or `quadmesh()`
3. Convert the aggregate to an image with `datashader.transfer_functions`

If you try to treat Datashader as a normal plotting library, you will usually miss the point. It is the rasterization engine underneath a visualization pipeline.

## Install

Pin the version your project expects:

```bash
python -m pip install "datashader==0.18.2"
```

Common alternatives:

```bash
uv add "datashader==0.18.2"
poetry add "datashader==0.18.2"
```

Datashader works with multiple data containers, but some of the surrounding libraries are optional. Install them separately when your workflow needs them:

- `pandas` for in-memory tabular data
- `dask` for partitioned tabular data
- `xarray` for gridded arrays
- `spatialpandas` for large geospatial vector data
- `pillow` if you want `img.to_pil()` image export helpers
- `holoviews` or `hvplot` if you want interactive plotting wrappers around Datashader

## Initialization And Mental Model

There is no authentication or service setup. The important setup is choosing a canvas size and coordinate ranges that match your task.

```python
import datashader as ds

canvas = ds.Canvas(
    plot_width=800,
    plot_height=500,
    x_range=(-10, 10),
    y_range=(-5, 5),
)
```

Important knobs:

- `plot_width` and `plot_height` define output resolution
- `x_range` and `y_range` define the data window being rasterized
- `x_axis_type` and `y_axis_type` matter for log axes

If you omit ranges, Datashader infers them from the current data. That is convenient for exploration, but it makes outputs harder to compare across runs.

## Core Usage

### Rasterize tabular points

```python
import pandas as pd
import datashader as ds
from datashader import transfer_functions as tf

df = pd.DataFrame(
    {
        "x": [0.1, 0.2, 0.2, 0.9, 1.2],
        "y": [0.0, 0.1, 0.4, 0.8, 1.3],
    }
)

canvas = ds.Canvas(
    plot_width=600,
    plot_height=400,
    x_range=(0, 2),
    y_range=(0, 2),
)

agg = canvas.points(df, "x", "y", agg=ds.count())
img = tf.shade(agg, cmap=["#deebf7", "#08519c"], how="log")
img = tf.set_background(img, "white")
```

Use `ds.count()` when you want density. Swap in reducers such as `ds.sum("value")`, `ds.mean("value")`, `ds.max("value")`, or `ds.count_cat("category")` when you need value-aware aggregation.

### Rasterize line data

Use `Canvas.line()` for ordered trajectories or time-series paths rather than approximating them as points:

```python
agg = canvas.line(df, "x", "y", agg=ds.count())
img = tf.shade(agg, cmap=["#f7fbff", "#08306b"])
```

### Work with Dask DataFrames

The API is intentionally similar to pandas:

```python
import dask.dataframe as dd
import datashader as ds
from datashader import transfer_functions as tf

ddf = dd.read_parquet("trips/*.parquet")
canvas = ds.Canvas(plot_width=900, plot_height=500)

agg = canvas.points(ddf, "pickup_x", "pickup_y", agg=ds.count())
img = tf.shade(agg, how="eq_hist")
```

Since `0.17.0`, Dask is no longer installed automatically with Datashader. Install `dask` yourself if you want this workflow.

### Work with xarray rasters

For gridded data, use raster-aware methods instead of forcing the data into a tabular glyph:

```python
import xarray as xr
import datashader as ds
from datashader import transfer_functions as tf

da = xr.DataArray(
    [[1, 2, 3], [4, 5, 6]],
    dims=("y", "x"),
    coords={"x": [0, 1, 2], "y": [0, 1]},
)

canvas = ds.Canvas(plot_width=600, plot_height=300)
agg = canvas.raster(da)
img = tf.shade(agg, cmap=["#f7fbff", "#08306b"])
```

Use `Canvas.quadmesh()` for curvilinear or nonuniform meshes. This matters because `0.18.2` includes a fix for antialiasing nonuniform `quadmesh` edges.

### Export Or Display The Result

Datashader returns an image object after `tf.shade(...)`. Common next steps:

```python
img = tf.dynspread(img, threshold=0.5, max_px=4)
img.to_pil().save("output.png")
```

Notes:

- `dynspread()` is useful when sparse points disappear at normal zoom levels
- `to_pil()` requires Pillow to be installed
- In notebooks, HoloViews and hvPlot can display Datashader-backed images directly
- For Matplotlib integration, use `datashader.mpl_ext.dsshow`

## Configuration Notes

There are no credentials or environment variables to manage. The main configuration choices are data-shaping and rendering choices:

- Aggregation reducer: `ds.count()`, `ds.sum("col")`, `ds.mean("col")`, `ds.max("col")`, `ds.count_cat("col")`
- Shading transform: `how="linear"`, `how="log"`, or `how="eq_hist"`
- Color map: pass a list of colors or a colormap object
- Canvas size: larger canvases preserve more detail but cost more CPU and memory
- Coordinate ranges: set them explicitly for reproducible comparisons and tiled rendering

For geospatial work, Datashader does not perform arbitrary reprojection for you. Make sure the coordinates you aggregate are already in the coordinate system your downstream plotting stack expects.

## Common Pitfalls

- Do not skip the aggregate step. `tf.shade()` expects an aggregate array, not your raw DataFrame.
- Do not assume bigger input data automatically means a bigger image. Output size is controlled by the canvas, not by row count.
- If repeated renders look inconsistent, check whether `x_range` and `y_range` are being auto-inferred each time.
- If `img.to_pil()` or Dask-backed examples fail with import errors, install the optional companion packages explicitly. Since `0.17.0`, Dask and Pillow are no longer hard dependencies.
- Use raster-specific methods such as `raster()` and `quadmesh()` for gridded data. Point and line glyphs are for tabular coordinates.
- Datashader does not replace your whole plotting stack. Use HoloViews, hvPlot, Bokeh, or Matplotlib when you need axes, layout, widgets, or interactive composition.

## Version-Sensitive Notes

- PyPI currently lists `0.18.2`, released on September 8, 2025.
- `0.18.2` fixes antialiasing for nonuniform `quadmesh` edges and includes edge-bundling inspector typing fixes. If you are debugging mesh rendering artifacts, confirm you are not on an older `0.18.x` patch.
- `0.17.0` dropped Python 3.9 support and changed Dask and Pillow from hard dependencies to optional ones. Older blog posts often assume those packages are installed automatically.
- The source URL pointed directly at the API reference. For actual coding work, use the docs root and getting-started guide for workflow, then the API page for signatures and exact method names.

## Official Source URLs

- `https://datashader.org/`
- `https://datashader.org/getting_started/Installation.html`
- `https://datashader.org/getting_started/Introduction.html`
- `https://datashader.org/api.html`
- `https://datashader.org/user_guide/Performance.html`
- `https://datashader.org/releases.html`
- `https://pypi.org/project/datashader/`
- `https://github.com/holoviz/datashader`

---
name: package
description: "Cartopy Python package guide for cartographic plotting, CRS transforms, and geospatial data overlays with Matplotlib"
metadata:
  languages: "python"
  versions: "0.25.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "cartopy,python,gis,maps,matplotlib,projection,crs,geospatial"
---

# Cartopy Python Package Guide

## Golden Rule

Use `cartopy` together with Matplotlib, choose the map `projection` for the axes, and explicitly pass `transform=` for any plotted data unless the data are already in the target projection. Most bad Cartopy plots come from mixing up those two concepts.

## What Cartopy Is Good At

Cartopy is a map projection and geospatial plotting library layered on top of Matplotlib. Use it when you need:

- map axes with real geographic projections
- automatic reprojection of points, lines, polygons, rasters, and vectors
- quick access to Natural Earth features like coastlines, borders, land, and lakes
- shapefile readers and tiled or remote map sources for scientific or GIS-style plots

If you only need tabular CRS transforms with no map rendering, `pyproj` is usually the smaller dependency. If you need GIS data manipulation before plotting, combine Cartopy with GeoPandas, Shapely, Rasterio, or xarray rather than expecting Cartopy to replace them.

## Install

For current releases, Cartopy publishes binary wheels for major operating systems, so `pip install` is the default path.

```bash
python -m pip install "cartopy==0.25.0"
```

Common alternatives:

```bash
uv add "cartopy==0.25.0"
poetry add "cartopy==0.25.0"
conda install -c conda-forge cartopy
```

Useful optional extras from PyPI:

```bash
python -m pip install "cartopy[plotting,ows,srtm]==0.25.0"
```

Notes:

- PyPI lists extras including `plotting`, `ows`, `srtm`, `speedups`, `doc`, and `test`.
- If a source build happens instead of installing a wheel, the docs list native dependency requirements such as PROJ plus supported versions of Matplotlib, Shapely, pyproj, and pyshp.
- The stable docs are published as `0.25.0.post2`, while the PyPI package release is `0.25.0`. Use the stable docs for behavior, but pin the package with the PyPI version.

## Initialize A Map

The simplest working map is a GeoAxes with a projection:

```python
import cartopy.crs as ccrs
import matplotlib.pyplot as plt

fig = plt.figure(figsize=(8, 4))
ax = plt.axes(projection=ccrs.PlateCarree())
ax.set_global()
ax.coastlines()

plt.show()
```

What matters:

- `projection=` controls how the map is drawn
- `GeoAxes` methods such as `coastlines()`, `set_extent()`, and `gridlines()` become available after creating the projected axes
- save figures before `plt.show()` if you need file output in batch code

## Plot Data Correctly

Always tell Cartopy the CRS of the input data.

```python
import numpy as np
import cartopy.crs as ccrs
import matplotlib.pyplot as plt

lon = np.linspace(-130, -60, 50)
lat = np.linspace(20, 55, 40)
lon2d, lat2d = np.meshgrid(lon, lat)
data = np.sin(np.deg2rad(lat2d)) + np.cos(np.deg2rad(lon2d))

data_crs = ccrs.PlateCarree()

fig = plt.figure(figsize=(9, 4))
ax = plt.axes(projection=ccrs.LambertConformal())
ax.set_extent([-130, -60, 20, 55], crs=data_crs)
ax.coastlines()

mesh = ax.contourf(lon, lat, data, transform=data_crs, cmap="viridis")
plt.colorbar(mesh, ax=ax, shrink=0.7)
plt.show()
```

Rule of thumb:

- `projection=` is for the destination map
- `transform=` is for the CRS your input coordinates are already in
- if your longitude and latitude arrays are ordinary geographic coordinates, `ccrs.PlateCarree()` is usually the right `transform`

## Common GeoAxes Workflow

Add built-in features and constrain the view:

```python
import cartopy.crs as ccrs
import cartopy.feature as cfeature
import matplotlib.pyplot as plt

fig = plt.figure(figsize=(10, 5))
ax = plt.axes(projection=ccrs.Mercator())
ax.set_extent([-125, -66.5, 24, 50], crs=ccrs.PlateCarree())

ax.add_feature(cfeature.LAND, facecolor="#f5f1e8")
ax.add_feature(cfeature.OCEAN, facecolor="#d9ebff")
ax.add_feature(cfeature.BORDERS, linewidth=0.5)
ax.add_feature(cfeature.STATES, linewidth=0.3)
ax.coastlines(resolution="50m", linewidth=0.7)

gridliner = ax.gridlines(draw_labels=True, linewidth=0.4, alpha=0.5)
gridliner.top_labels = False
gridliner.right_labels = False

plt.show()
```

Useful feature constants include `LAND`, `OCEAN`, `COASTLINE`, `BORDERS`, `LAKES`, `RIVERS`, and `STATES`.

## Working With Shapefiles And Natural Earth Data

Cartopy can fetch and read standard datasets for you:

```python
from cartopy.io import shapereader

filename = shapereader.natural_earth(
    resolution="110m",
    category="cultural",
    name="admin_0_countries",
)

reader = shapereader.Reader(filename)

for record in reader.records():
    print(record.attributes["NAME_LONG"])
    break
```

This is convenient, but it has operational implications:

- first use may download Natural Earth or other external data into Cartopy's data directory
- a fresh CI runner or offline container can fail or hang if the dataset is not already cached
- if you need reproducible offline execution, pre-download required features instead of depending on first-run network access

Since Cartopy 0.23, the built-in feature downloader can be invoked as `python -m cartopy.feature.download` or with the installed `cartopy-feature-download` CLI.

## Configuration And Data Paths

Cartopy's top-level `cartopy.config` dictionary controls where packaged and downloaded data live.

Relevant keys:

- `pre_existing_data_dir`: read-only directory to check first for standard datasets
- `data_dir`: writable directory where Cartopy stores downloaded datasets
- `repo_data_dir`: repository-shipped data path, mainly relevant to packagers

Example:

```python
import cartopy

cartopy.config["data_dir"] = "/tmp/cartopy-data"
cartopy.config["pre_existing_data_dir"] = "/opt/cartopy-data"
```

Practical guidance:

- set these before code that triggers downloads
- in production, prefer a pre-populated shared data directory over ad hoc downloads
- in containers, mount a writable cache or bake the data into the image

## OGC, Raster, And Tile Sources

Cartopy includes interfaces under `cartopy.io` for:

- shapefiles
- raster reprojection utilities
- image tiles
- OGC clients
- SRTM elevation sources

Use these when you want Cartopy to stay responsible for map reprojection. If your raster pipeline is already built around Rasterio, xarray, or rioxarray, it is often cleaner to prepare the data there and use Cartopy mainly for the final plotted projection.

## Performance Notes

- Large shapefiles and dense coastlines can dominate render time. Use smaller Natural Earth resolutions like `110m` or `50m` unless you truly need `10m`.
- Reprojecting high-resolution rasters on every draw is expensive. Reuse transformed outputs when plotting the same dataset repeatedly.
- The docs note that you can set `PYPROJ_GLOBAL_CONTEXT=ON` for faster projection calculations if thread safety is not a concern in your application.
- For static reporting jobs, save the figure once rather than repeatedly redrawing interactive windows.

## Common Pitfalls

### Forgetting `transform=`

This is the most common mistake. A plot can appear correct in `PlateCarree` and then become wrong after switching projections because Cartopy assumes the source data CRS matches the axes projection when `transform` is omitted.

### Assuming all first-run environments can download data

Feature and shapefile helpers may need network access the first time they run. Cache or prefetch the data for CI, containers, and air-gapped systems.

### Mixing Shapely or Matplotlib examples from older releases

Cartopy `0.25` deprecated older path conversion helpers:

- `path_to_geos` and `geos_to_path` are deprecated in favor of `path_to_shapely` and `shapely_to_path`
- `cartopy.mpl.clip_path` is deprecated without replacement
- `path_segments` is deprecated

If copied code uses those names, update it before adding more logic around it.

### Using old gridliner label attributes

Older examples may use `xlabels_top`, `xlabels_bottom`, `ylabels_left`, or `ylabels_right`. Current Cartopy examples use `top_labels`, `bottom_labels`, `left_labels`, and `right_labels`.

### Treating Cartopy as a full GIS stack

Cartopy is excellent at projected plotting and transformation-aware map rendering. It is not the right place for heavy vector cleaning, tabular joins, or complex raster I/O pipelines.

## Version-Sensitive Notes For `0.25.0`

- PyPI package version: `0.25.0`
- Stable docs version: `0.25.0.post2`
- `0.25` raised the minimum supported Shapely version to `2.0`
- `0.25` added new projection-related behavior such as Orthographic `rotation` and the Spilhaus projection, with Spilhaus requiring PROJ `9.6+`
- `0.25` also deprecated older path-conversion helpers, so examples written for `0.24` or earlier may need edits even if the broad plotting workflow is unchanged

## Official Sources Used For This Guide

- Stable docs landing page: `https://cartopy.readthedocs.io/stable/`
- Installation: `https://cartopy.readthedocs.io/stable/installing.html`
- Getting started and Matplotlib interface: `https://cartopy.readthedocs.io/stable/getting_started/index.html`
- Matplotlib intro: `https://cartopy.readthedocs.io/stable/matplotlib/intro.html`
- Projection and transform tutorial: `https://cartopy.readthedocs.io/stable/tutorials/understanding_transform.html`
- API reference: `https://cartopy.readthedocs.io/stable/reference/index.html`
- Feature interface: `https://cartopy.readthedocs.io/stable/reference/feature.html`
- I/O reference: `https://cartopy.readthedocs.io/stable/reference/io.html`
- Configuration reference: `https://cartopy.readthedocs.io/stable/reference/config.html`
- `0.25` release notes: `https://cartopy.readthedocs.io/stable/whatsnew/v0.25.html`
- `0.23` release notes for offline feature download utility: `https://cartopy.readthedocs.io/latest/whatsnew/v0.23.html`
- PyPI registry page: `https://pypi.org/project/Cartopy/`

---
name: package
description: "pyproj Python package for CRS parsing, coordinate transformations, geodesic calculations, and PROJ data management"
metadata:
  languages: "python"
  versions: "3.7.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pyproj,proj,gis,geospatial,crs,coordinate-transformations,geodesic"
---

# pyproj Python Package Guide

## Golden Rule

Use `pyproj` through `CRS`, `Transformer`, and `Geod`, not through legacy `Proj` or deprecated `transform()` helpers. For coordinate transforms, decide axis order explicitly with `always_xy=True` when your inputs are longitude/latitude, and treat grid availability as part of your runtime configuration.

## Install

For most Python projects, use the wheel from PyPI:

```bash
python -m pip install "pyproj==3.7.2"
```

Common alternatives:

```bash
uv add "pyproj==3.7.2"
poetry add "pyproj==3.7.2"
```

If you are using conda, prefer `conda-forge` and avoid mixing `pip install` into that environment unless you understand the consequences:

```bash
conda config --prepend channels conda-forge
conda config --set channel_priority strict
conda create -n geo pyproj
conda activate geo
```

Source-build notes:

- Current stable docs say the minimum supported PROJ version is `9.4`
- Building from source requires a working PROJ installation
- The installation docs describe `PROJ_DIR`, `PROJ_LIBDIR`, and `PROJ_INCDIR` for source builds

## Core Workflow

### 1. Parse and inspect CRS objects

Use EPSG codes or WKT where possible. The official gotchas page recommends WKT or SRIDs such as EPSG codes over PROJ strings, and prefers WKT2 over WKT1.

```python
from pyproj import CRS

wgs84 = CRS.from_epsg(4326)
web_mercator = CRS.from_epsg(3857)

print(wgs84)
print(web_mercator.to_epsg())
print(web_mercator.to_wkt())
```

Practical rule:

- Prefer `CRS.from_epsg(...)` for standard systems
- Prefer `crs.to_wkt()` when you need a durable serialized CRS form
- Avoid storing CRS definitions only as PROJ strings when long-term fidelity matters

### 2. Reuse a `Transformer` for coordinate transforms

`Transformer` is the main API for repeated transforms and datum-aware conversions.

```python
from pyproj import Transformer

transformer = Transformer.from_crs(
    "EPSG:4326",
    "EPSG:3857",
    always_xy=True,
)

x, y = transformer.transform(-122.4194, 37.7749)
print(x, y)
```

Why `always_xy=True` matters:

- EPSG axis order can be latitude/longitude for geographic CRS definitions
- Many application inputs are longitude/latitude
- `always_xy=True` forces x/y order so callers can consistently pass `lon, lat`

When you need to inspect the chosen operation:

```python
from pyproj import Transformer

transformer = Transformer.from_crs("EPSG:4326", "EPSG:26917", always_xy=True)

print(transformer.description)
print(transformer.accuracy)
print(transformer.area_of_use)
```

### 3. Choose a local UTM CRS from coordinates

This is often safer than hard-coding a projected CRS for ad hoc point data.

```python
from pyproj import CRS
from pyproj.aoi import AreaOfInterest
from pyproj.database import query_utm_crs_info

utm_candidates = query_utm_crs_info(
    datum_name="WGS 84",
    area_of_interest=AreaOfInterest(
        west_lon_degree=-93.581543,
        south_lat_degree=42.032974,
        east_lon_degree=-93.581543,
        north_lat_degree=42.032974,
    ),
)

utm_crs = CRS.from_epsg(utm_candidates[0].code)
print(utm_crs)
```

### 4. Compute geodesic distances, bearings, and destinations

Use `Geod` for accurate work on the ellipsoid. Do not project to Web Mercator and treat planar distance as geodesic distance.

```python
from pyproj import Geod

geod = Geod(ellps="WGS84")

az12, az21, distance_m = geod.inv(
    -71.1167, 42.25,
    -123.6833, 45.5167,
)

end_lon, end_lat, back_azimuth = geod.fwd(
    -71.1167, 42.25,
    az12,
    distance_m,
)

print(distance_m)
print(end_lon, end_lat, back_azimuth)
```

For polygon measurements without projecting first:

```python
from pyproj import Geod

geod = Geod(ellps="WGS84")

lons = [-74, -102, -102, -131, -163, 163, 172, 140]
lats = [-72.9, -71.9, -74.9, -74.3, -77.5, -77.4, -71.7, -65.9]

area_m2, perimeter_m = geod.polygon_area_perimeter(lons, lats)
print(area_m2, perimeter_m)
```

If your project already uses Shapely, `Geod.geometry_length()` and `Geod.geometry_area_perimeter()` are available for geometry objects.

## Configuration And PROJ Data

### Data directory resolution

`pyproj` resolves its PROJ data directory in this order:

1. A path set with `pyproj.datadir.set_data_dir(...)`
2. The internal PROJ directory, if present
3. `PROJ_DATA` for PROJ 9.1+ or `PROJ_LIB` for older PROJ
4. `sys.prefix`
5. The system `PATH`

Basic inspection and override:

```python
from pyproj import datadir

print(datadir.get_data_dir())
datadir.set_data_dir("/opt/proj/share/proj")
```

### Grid downloads and network access

Important upstream rule: `pyproj 3` wheels do **not** include transformation grids.

Implications:

- If you only use `CRS` or `Geod`, you usually do not need grids
- Datum transformations may need grids for best accuracy
- Missing grids can change which transform PROJ selects or make the best transform unavailable

Enable PROJ network access when on-demand grid fetching is acceptable:

```python
from pyproj import network

network.set_network_enabled(True)
print(network.is_network_enabled())
```

This has the same behavior as the `PROJ_NETWORK` environment variable.

If you need to preflight or pre-download candidate grids:

```python
from pyproj.sync import get_transform_grid_list

grids = get_transform_grid_list(
    source_id="us_noaa",
    include_already_downloaded=False,
)

print(len(grids))
```

For offline or controlled environments:

- Mirror or pre-download the grid files into the PROJ data directory
- Keep `PROJ_NETWORK=OFF` if the runtime must not fetch remote assets
- Use `pyproj.show_versions()` when debugging mismatched PROJ data, wheel, and runtime environments

```bash
python -c "import pyproj; pyproj.show_versions()"
```

## Common Pitfalls

- Do not use `Proj` as a generic latitude/longitude to projected-coordinate converter across datums. The official gotchas page recommends `Transformer` for that job.
- Do not use `+init=EPSG:xxxx` or similar `+init=` syntax. It is deprecated and can fail even when `EPSG:xxxx` works.
- Do not assume axis order is always `lon, lat`. If your caller works in x/y order, pass `always_xy=True` when creating the transformer.
- Do not assume PyPI wheels include datum grids. They do not.
- Do not treat PROJ strings as the best long-term CRS storage format. Prefer EPSG codes or WKT, especially WKT2.
- Do not ignore `SQLite error on SELECT` messages from PROJ. The upstream gotchas page says they usually indicate mixed PROJ versions or an incompatible data directory.
- Do not keep using `pyproj.transformer.transform()` or `itransform()` from old examples. The gotchas page marks them as deprecated in favor of `Transformer`.

## Version-Sensitive Notes For 3.7.2

- As of March 12, 2026, the version used here, PyPI latest version, and stable docs version all align on `3.7.2`.
- The stable docs root for `3.7.2` states the minimum supported Python version is `3.11` and the minimum supported PROJ version is `9.4`.
- `pyproj.set_use_global_context()` is deprecated since `3.7.0` because pyproj now uses one context per thread. Avoid adding new code that depends on the old global-context toggle.
- If you are copying older blog posts, watch for pre-`Transformer` examples, `Proj(init="epsg:...")`, or manual axis-order assumptions. Those patterns are stale for modern pyproj.

## Official Sources

- Stable docs: https://pyproj4.github.io/pyproj/stable/
- Installation: https://pyproj4.github.io/pyproj/stable/installation.html
- Getting started examples: https://pyproj4.github.io/pyproj/stable/examples.html
- Gotchas/FAQ: https://pyproj4.github.io/pyproj/stable/gotchas.html
- Transformation grids: https://pyproj4.github.io/pyproj/stable/transformation_grids.html
- Data directory API: https://pyproj4.github.io/pyproj/stable/api/datadir.html
- PROJ network API: https://pyproj4.github.io/pyproj/stable/api/network.html
- Sync API: https://pyproj4.github.io/pyproj/stable/api/sync.html
- Global context API: https://pyproj4.github.io/pyproj/stable/api/global_context.html
- Geod API: https://pyproj4.github.io/pyproj/stable/api/geod.html
- Show versions API: https://pyproj4.github.io/pyproj/stable/api/show_versions.html
- PyPI package page: https://pypi.org/project/pyproj/

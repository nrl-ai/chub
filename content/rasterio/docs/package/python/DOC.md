---
name: package
description: "Rasterio Python package guide for reading, writing, windowing, masking, reprojection, and cloud raster access"
metadata:
  languages: "python"
  versions: "1.5.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "rasterio,python,gdal,gis,raster,geotiff,geospatial"
---

# Rasterio Python Package Guide

## Golden Rule

Use `rasterio` as the Python layer over GDAL for raster files, keep dataset access inside `with rasterio.open(...)` blocks, and derive output metadata from `src.profile` or `src.meta` instead of rebuilding raster dimensions, CRS, dtype, and transform by hand.

As of 2026-03-12, PyPI and the upstream GitHub release are on `1.5.0`, but `https://rasterio.readthedocs.io/en/stable/` is still labeled `1.4.4`. Treat the docs site as the main usage guide and the `1.5.0` release notes as the source of truth for version-specific changes.

## Install

Pin the version your project expects:

```bash
python -m pip install "rasterio==1.5.0"
```

Common alternatives:

```bash
uv add "rasterio==1.5.0"
poetry add "rasterio==1.5.0"
```

Useful extras published on PyPI:

```bash
python -m pip install "rasterio[s3]==1.5.0"
python -m pip install "rasterio[plot]==1.5.0"
python -m pip install "rasterio[all]==1.5.0"
```

Installation notes:

- PyPI wheels are the easiest path for simple applications and include many built-in drivers, but the official installation guide warns they are not tested against every other binary wheel, conda package, or QGIS stack.
- If your environment already has a GIS stack and binary compatibility matters more than convenience, `conda-forge` or a source build against your existing GDAL may be safer.
- Source builds need a compatible GDAL installation. The docs still show `gdal-config`-based builds; for `1.5.0`, align with the newer package baseline of Python 3.12+, GDAL 3.8+, and NumPy 2+.

## Core Usage

### Open a dataset and inspect metadata

`rasterio.open()` returns a dataset object. Read metadata from attributes and arrays with `read()`.

```python
import rasterio

with rasterio.open("example.tif") as src:
    print(src.name)
    print(src.count, src.width, src.height)
    print(src.crs)
    print(src.transform)
    print(src.bounds)

    band1 = src.read(1)
    masked_band1 = src.read(1, masked=True)
```

Notes:

- Band indexes are 1-based, not 0-based.
- `src.read()` returns an array shaped `(count, height, width)`.
- `masked=True` is usually the safest read mode when nodata handling matters.

### Write a derived raster

The official profiles guide recommends copying `src.profile`, updating only the fields that changed, and passing that profile back into `rasterio.open(..., "w", **profile)`.

```python
import numpy as np
import rasterio

with rasterio.open("input.tif") as src:
    data = src.read().astype("float32")
    mean_band = data.mean(axis=0, dtype="float32")

    profile = src.profile.copy()
    profile.update(
        driver="GTiff",
        count=1,
        dtype="float32",
        compress="deflate",
    )

    with rasterio.open("mean.tif", "w", **profile) as dst:
        dst.write(mean_band, 1)
```

When output dimensions or georeferencing change, also update `width`, `height`, `transform`, and sometimes `crs`.

### Read or write only a window

Use windows to work on rasters larger than RAM or to process chunks in parallel.

```python
import rasterio
from rasterio.windows import Window

window = Window(col_off=0, row_off=0, width=512, height=512)

with rasterio.open("big.tif") as src:
    tile = src.read(1, window=window, masked=True)
    tile_transform = src.window_transform(window)
```

Windowing notes:

- Windows are pixel subsets defined by offsets plus width and height.
- Tiny windows do not guarantee tiny I/O. Rasterio reads full underlying GDAL blocks, so tiled rasters are much more efficient for window-heavy workloads.

### Mask by geometry

Use `rasterio.mask.mask()` when you need to crop to polygons or zero out pixels outside shapes.

```python
import rasterio
from rasterio.mask import mask

geometries = [
    {
        "type": "Polygon",
        "coordinates": [[
            (-123.3, 44.0),
            (-123.3, 43.8),
            (-123.0, 43.8),
            (-123.0, 44.0),
            (-123.3, 44.0),
        ]],
    }
]

with rasterio.open("input.tif") as src:
    data, transform = mask(src, geometries, crop=True)
    profile = src.profile.copy()
    profile.update(
        height=data.shape[1],
        width=data.shape[2],
        transform=transform,
    )

    with rasterio.open("clipped.tif", "w", **profile) as dst:
        dst.write(data)
```

With `crop=True`, shape and transform change. Update them before writing output.

### Reproject to another CRS

`rasterio.warp.calculate_default_transform()` and `reproject()` are the standard path for GeoTIFF reprojection.

```python
import rasterio
from rasterio.warp import calculate_default_transform, reproject, Resampling

dst_crs = "EPSG:4326"

with rasterio.open("input.tif") as src:
    transform, width, height = calculate_default_transform(
        src.crs, dst_crs, src.width, src.height, *src.bounds
    )

    profile = src.profile.copy()
    profile.update(
        crs=dst_crs,
        transform=transform,
        width=width,
        height=height,
    )

    with rasterio.open("reprojected.tif", "w", **profile) as dst:
        for band_index in range(1, src.count + 1):
            reproject(
                source=rasterio.band(src, band_index),
                destination=rasterio.band(dst, band_index),
                src_transform=src.transform,
                src_crs=src.crs,
                dst_transform=transform,
                dst_crs=dst_crs,
                resampling=Resampling.nearest,
            )
```

## Configuration And Cloud Access

### Use `rasterio.Env()` for GDAL and cloud settings

Rasterio wraps GDAL's global state with `rasterio.Env(...)`. Use it when you need GDAL config options, custom sessions, or cloud credentials.

```python
import rasterio

with rasterio.Env(GDAL_CACHEMAX=128_000_000):
    with rasterio.open("example.tif") as src:
        arr = src.read(1)
```

For AWS-backed datasets, `Env` can take a session or AWS credential arguments and profile settings.

```python
import rasterio

with rasterio.Env(profile_name="default"):
    with rasterio.open("s3://my-bucket/path/to/image.tif") as src:
        print(src.profile)
```

### Remote files and virtual filesystems

Rasterio maps normal URI schemes to GDAL virtual filesystems. Prefer standard URIs over raw `/vsi...` paths.

```python
import rasterio

with rasterio.open("https://example.com/data/image.tif") as src:
    arr = src.read(1)

with rasterio.open("zip+https://example.com/archive/files.zip!image.tif") as src:
    arr = src.read(1)
```

S3 support is optional and requires the `s3` extra:

```bash
python -m pip install "rasterio[s3]==1.5.0"
```

### Use `MemoryFile` for bytes in memory

`MemoryFile` is useful when your input is already in bytes from a network response or another in-memory source.

```python
import requests
from rasterio.io import MemoryFile

response = requests.get("https://example.com/image.tif", timeout=30)
response.raise_for_status()

with MemoryFile(response.content) as memfile:
    with memfile.open() as src:
        arr = src.read(1)
```

Important: a `MemoryFile` created from existing bytes cannot be extended later. Empty `MemoryFile()` instances and byte-initialized ones are different modes.

## Common Pitfalls

- Do not rebuild output metadata from scratch unless necessary. Start from `src.profile.copy()` and then update only what changed.
- Remember that Rasterio band indexes are 1-based. `src.read(1)` is the first band.
- `crop=True` in `rasterio.mask.mask()` changes extent, shape, and transform. If you write the result back out, update those fields in the destination profile.
- Windowed reads operate at the raster block level. If the file is not tiled, small windows can still trigger large reads.
- Base `pip install rasterio` does not include S3 extras. Install `rasterio[s3]` before assuming `s3://...` support works cleanly.
- Keep file access inside context managers. Dataset handles own GDAL resources and should be closed promptly.
- If you mix Rasterio wheels with a separately managed GIS stack, binary compatibility issues can look like runtime import errors or missing drivers rather than obvious install failures.
- When writing arrays, ensure `dtype`, `count`, and band shapes match the destination profile. Silent mismatches are a common source of corrupted outputs or write errors.

## Version-Sensitive Notes For `1.5.0`

- `1.5.0` is the current PyPI release as of 2026-03-12 and was released on 2026-01-05.
- The minimum supported versions were raised to Python 3.12+, GDAL 3.8+, and NumPy 2+.
- The `1.5.0` release adds `float16` dtype support.
- `rasterio.open()` gained a `thread_safe` parameter in `1.5.0`. If you are copying concurrency advice from older examples, re-check it against the current API and release notes.
- `rasterio.warp.reproject()` now exposes `tolerance` as an argument, which matters if you tune warp precision.
- Official wheels for `1.5.0` use GDAL `3.12.1`.
- The stable docs site still shows `1.4.4`, so when a detail appears stale, prefer PyPI metadata and the `1.5.0` release notes over the docs page header.

## Official Links

- Docs root: `https://rasterio.readthedocs.io/en/stable/`
- Installation: `https://rasterio.readthedocs.io/en/stable/installation.html`
- Quickstart: `https://rasterio.readthedocs.io/en/stable/quickstart.html`
- Profiles and writing: `https://rasterio.readthedocs.io/en/stable/topics/profiles.html`
- Windowed I/O: `https://rasterio.readthedocs.io/en/stable/topics/windowed-rw.html`
- Masking: `https://rasterio.readthedocs.io/en/stable/topics/masking-by-shapefile.html`
- Reprojection: `https://rasterio.readthedocs.io/en/stable/topics/reproject.html`
- GDAL/AWS environment API: `https://rasterio.readthedocs.io/en/stable/api/rasterio.env.html`
- Virtual filesystems: `https://rasterio.readthedocs.io/en/stable/topics/vsi.html`
- In-memory files: `https://rasterio.readthedocs.io/en/stable/topics/memory-files.html`
- PyPI package: `https://pypi.org/project/rasterio/`
- `1.5.0` release notes: `https://github.com/rasterio/rasterio/releases/tag/1.5.0`

---
name: package
description: "xarray package guide for labeled N-dimensional arrays, datasets, netCDF/Zarr I/O, and chunked scientific workflows in Python"
metadata:
  languages: "python"
  versions: "2026.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "xarray,python,numpy,pandas,dask,zarr,netcdf,scientific-computing"
---

# xarray Python Package Guide

## Golden Rule

Use `xarray` when your arrays have named dimensions, coordinates, or metadata that should survive slicing, alignment, grouping, and serialization. For production code, pass the storage backend and chunking strategy explicitly instead of relying on backend auto-detection.

## Install

Pin the package version your project expects:

```bash
python -m pip install "xarray==2026.2.0"
```

Useful extras from the official install guide:

```bash
python -m pip install "xarray[io]==2026.2.0"
python -m pip install "xarray[parallel]==2026.2.0"
python -m pip install "xarray[viz]==2026.2.0"
python -m pip install "xarray[accel]==2026.2.0"
python -m pip install "xarray[complete]==2026.2.0"
```

Notes:

- `xarray` itself is lightweight; most real workflows also need optional packages for file formats, parallelism, plotting, or performance.
- Scientific environments are often easier to provision with conda or mamba when native dependencies like NetCDF or Zarr backends are involved.
- `2026.2.0` requires newer dependency floors than older blog posts assume, including Python 3.11+, NumPy 1.26+, and pandas 2.2+.

## Initialize And Core Objects

The two main types are:

- `xr.DataArray`: one labeled array with dimensions, coordinates, and attrs
- `xr.Dataset`: a dict-like collection of named `DataArray` objects sharing coordinates

### DataArray

```python
import numpy as np
import pandas as pd
import xarray as xr

times = pd.date_range("2026-01-01", periods=3, freq="D")

temperature = xr.DataArray(
    np.array([15.2, 16.8, 14.9]),
    dims=["time"],
    coords={"time": times},
    name="temperature_c",
    attrs={"units": "degC"},
)

print(temperature.sel(time="2026-01-02").item())
print(temperature.mean(dim="time").item())
```

### Dataset

```python
import numpy as np
import pandas as pd
import xarray as xr

times = pd.date_range("2026-01-01", periods=3, freq="D")

ds = xr.Dataset(
    data_vars={
        "temperature_c": ("time", [15.2, 16.8, 14.9]),
        "humidity_pct": ("time", [70, 65, 80]),
    },
    coords={"time": times},
    attrs={"station": "sfo"},
)

print(ds["temperature_c"])
print(ds.mean(dim="time"))
```

## Core Usage

### Indexing And Selection

Prefer label-based indexing when coordinates are meaningful:

```python
subset = ds.sel(time=slice("2026-01-01", "2026-01-02"))
first_row = ds.isel(time=0)
```

Use `sel()` for coordinates and `isel()` for positional indexing. xarray aligns operations by coordinate labels, not just by shape.

### Grouping And Resampling

```python
monthly = ds.resample(time="MS").mean()
```

If your dimension names are semantic, operations stay readable because every reduction and transformation takes `dim=` names instead of positional axes.

### Converting To Pandas

```python
frame = ds.to_dataframe().reset_index()
```

Use pandas only when you need row-oriented operations. Keep data in xarray for labeled N-dimensional math, broadcasting, and serialization.

## I/O And Storage Backends

### Open a single dataset

```python
import xarray as xr

ds = xr.open_dataset(
    "data/example.nc",
    engine="h5netcdf",
    chunks="auto",
    decode_times=True,
)
```

Important parameters for `xr.open_dataset()`:

- `engine=`: choose the backend explicitly for predictable behavior
- `chunks=`: use `"auto"` or a chunk mapping to get dask-backed lazy arrays
- `cache=`: defaults differ depending on whether dask chunking is enabled
- `create_default_indexes=`: by default, dimension coordinates are loaded into pandas indexes eagerly

### Open many NetCDF files

```python
ds = xr.open_mfdataset(
    "data/daily/*.nc",
    combine="by_coords",
    engine="h5netcdf",
    chunks="auto",
)
```

Use `open_mfdataset()` for collections of files that should combine by coordinates or by nested structure. This is the normal entry point for dask-backed multi-file workflows.

### Open Zarr

```python
ds = xr.open_zarr(
    "s3://bucket/path/to/store",
    chunks="auto",
    consolidated=True,
    zarr_format=3,
)
```

Use `open_zarr()` when data already lives in chunked object storage or when cloud-native lazy access matters more than single-file portability.

### Write datasets

```python
ds.to_netcdf("output.nc", engine="h5netcdf")
ds.to_zarr("output.zarr", mode="w", consolidated=True, zarr_format=3)
```

## Configuration And Environment

xarray itself does not require authentication. Credentials only matter when the storage backend does, for example:

- S3 paths via `fsspec` and `s3fs`
- GCS paths via `gcsfs`
- Azure paths via `adlfs`

Keep cloud credentials in the backend’s normal environment variables or SDK config, then pass backend-specific options through `storage_options=` when needed.

Example:

```python
ds = xr.open_zarr(
    "s3://my-bucket/forecast.zarr",
    storage_options={"anon": False},
    chunks="auto",
)
```

Process-wide options:

```python
import xarray as xr

with xr.set_options(
    keep_attrs=True,
    arithmetic_join="exact",
    netcdf_engine_order=("h5netcdf", "netcdf4", "scipy"),
):
    result = ds1 + ds2
```

Useful options:

- `keep_attrs=True` preserves metadata through many operations
- `arithmetic_join="exact"` fails fast if coordinates do not align exactly
- `netcdf_engine_order=...` makes backend selection deterministic

## Common Pitfalls

- Binary operations align by coordinate labels. Two arrays with the same shape can still produce unexpected `NaN` values or expanded coordinates if labels differ.
- `open_dataset()` may eagerly create pandas indexes for dimension coordinates even when array data stays lazy. Large coordinate indexes can still consume memory.
- The default backend for NetCDF is not a safe thing to guess. Official xarray docs currently contain version-era guidance that can differ between API pages and release notes, so pass `engine=` explicitly.
- Dask chunking is not automatic unless you ask for it. Without `chunks=...`, many reads materialize NumPy arrays eagerly.
- `open_mfdataset()` assumes the files can be combined coherently. Mismatched coordinates, attrs, or chunking often need explicit `combine=`, `compat=`, or preprocessing.
- Zarr v2 and v3 have different behavior around fill values and metadata. Be explicit about `zarr_format=` when writing new stores.
- `attrs` are metadata, not a schema contract. They are easy to drop unless you deliberately preserve them with `xr.set_options(keep_attrs=True)` or operation-specific handling.

## Version-Sensitive Notes For 2026.2.0

- PyPI and the stable docs both identify the current package release as `2026.2.0` on March 12, 2026.
- `2026.2.0` requires Python 3.11 or later.
- The `2026.2.0` release notes document newer minimum supported dependency versions than older examples typically assume, including NumPy 1.26+, pandas 2.2+, packaging 24.1+, zarr 2.18+, and cftime 1.6.2+.
- The Zarr API has moved toward `zarr_format=`; older snippets that still use `zarr_version=` should be updated.
- The release notes also call out changed default handling for `use_zarr_fill_value_as_mask`: for Zarr v2 the default remains `True`, while for Zarr v3 it defaults to `False`.
- Backend-selection guidance is version-sensitive right now. If a workflow depends on `netcdf4`, `h5netcdf`, or `scipy` semantics, set `engine=` or `xr.set_options(netcdf_engine_order=...)` explicitly instead of trusting the default.

## Official Sources

- Docs root: https://docs.xarray.dev/en/stable/
- API reference: https://docs.xarray.dev/en/stable/api.html
- Installation guide: https://docs.xarray.dev/en/stable/getting-started-guide/installing.html
- Quick overview: https://docs.xarray.dev/en/stable/getting-started-guide/quick-overview.html
- Data structures: https://docs.xarray.dev/en/stable/user-guide/data-structures.html
- Indexing: https://docs.xarray.dev/en/stable/user-guide/indexing.html
- I/O guide: https://docs.xarray.dev/en/stable/user-guide/io.html
- `open_dataset()`: https://docs.xarray.dev/en/stable/generated/xarray.open_dataset.html
- `open_zarr()`: https://docs.xarray.dev/en/stable/generated/xarray.open_zarr.html
- `set_options()`: https://docs.xarray.dev/en/stable/generated/xarray.set_options.html
- Release notes: https://docs.xarray.dev/en/stable/whats-new.html
- PyPI: https://pypi.org/project/xarray/

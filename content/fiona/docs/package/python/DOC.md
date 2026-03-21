---
name: package
description: "Fiona Python package guide for reading and writing vector geospatial data with GDAL-backed drivers"
metadata:
  languages: "python"
  versions: "1.10.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "fiona,python,gis,geospatial,vector,gdal,geojson"
---

# Fiona Python Package Guide

## Golden Rule

Use `fiona` for vector dataset I/O, not for geometric analysis. Fiona reads and writes GeoJSON-like feature records through GDAL/OGR-backed drivers; pair it with `shapely` for geometry operations and `pyproj` when you need explicit coordinate transforms.

## Install

Pin the version your project expects:

```bash
python -m pip install "fiona==1.10.1"
```

Common alternatives:

```bash
uv add "fiona==1.10.1"
poetry add "fiona==1.10.1"
```

Useful extras:

```bash
python -m pip install "fiona[s3]==1.10.1"
python -m pip install "fiona[calc]==1.10.1"
```

Notes:

- `fiona[s3]` installs the `boto3` dependencies used by `AWSSession`.
- `fiona[calc]` is required for CLI commands such as `fio calc`, `fio filter`, `fio map`, and `fio reduce`.
- PyPI wheels are convenient, but the upstream docs explicitly warn that they omit many optional GDAL drivers and are not a good fit for every production environment or mixed binary stack.

If you need drivers omitted from the wheels, build from source against an existing GDAL install:

```bash
export GDAL_CONFIG=/path/to/gdal-config
python -m pip install -U pip
python -m pip install --no-binary fiona "fiona==1.10.1"
```

## Initialize And Inspect A Dataset

Collections returned by `fiona.open()` behave like file objects plus iterators over GeoJSON-like feature records:

```python
import fiona

with fiona.open("data/countries.geojson") as src:
    print(src.driver)
    print(src.crs)
    print(src.schema)
    print(src.bounds)
    print(len(src))

    first = next(iter(src))
    print(first.id)
    print(first.geometry.type)
    print(first.properties["name"])
```

Important behavior:

- Use `with fiona.open(...)` so files flush and close cleanly.
- Feature objects are mapping-like, not plain dicts. Convert with `fiona.model.to_dict()` if you need JSON serialization.
- Fiona copies feature data into Python objects. This is simpler than GDAL's raw bindings, but not the fastest option for bulk field-level extraction.

## Core Usage

### Read Features And Apply Spatial Or Attribute Filters

```python
import fiona
from shapely.geometry import shape

with fiona.open("data/parcels.gpkg", layer="parcels") as src:
    for feat in src.filter(
        bbox=(-123.2, 44.0, -123.0, 44.2),
        where="land_use = 'residential'",
    ):
        geom = shape(feat.geometry)
        print(feat.id, geom.area, feat.properties["land_use"])
```

Use `bbox=` for spatial intersection filtering and `where=` for SQL-style attribute filtering. If you need geometry predicates beyond simple bounds checks, load the geometry into Shapely and do the rest there.

### Write A New Dataset

When the output mostly matches the input, start from `src.profile` instead of rebuilding driver, CRS, and schema by hand.

```python
import fiona
from fiona import Feature, Geometry
from shapely.geometry import mapping, shape

def to_centroid(feature):
    centroid = shape(feature.geometry).centroid
    return Feature.from_dict(
        geometry=Geometry.from_dict(mapping(centroid)),
        properties=dict(feature.properties),
    )

with fiona.open("data/input.gpkg", layer="parcels") as src:
    profile = src.profile
    profile["driver"] = "GPKG"
    profile["schema"]["geometry"] = "Point"

    with fiona.open("data/parcel-centroids.gpkg", "w", layer="centroids", **profile) as dst:
        dst.writerecords(to_centroid(feature) for feature in src)
```

When writing from scratch, you must provide:

- `driver`
- `crs`
- `schema`

The schema geometry must match the geometries you write. Fiona will not automatically coerce a polygon schema into a point schema or vice versa.

### Work With Multilayer Datasets

GeoPackage, file geodatabases, archives, and directory-backed datasets can expose multiple layers.

```python
import fiona

dataset = "data/census.gpkg"

print(fiona.listlayers(dataset))

with fiona.open(dataset, layer="tracts") as tracts:
    print(tracts.schema)

with fiona.open(dataset, layer=0) as first_layer:
    print(first_layer.name)
```

If you omit `layer=...`, Fiona opens the first layer. That is convenient for shapefiles and easy demos, but it is brittle for GeoPackage-style datasets.

### Read Zipped Or In-Memory Data

Fiona supports GDAL virtual filesystem URIs and in-memory readers.

```python
from fiona.io import ZipMemoryFile

data = open("uploads/boundaries.zip", "rb").read()

with ZipMemoryFile(data) as archive:
    with archive.open("boundaries.shp") as src:
        print(len(src))
        print(src.schema)
```

This is the right pattern when an agent receives uploaded bytes and should avoid writing a temporary archive to disk.

## Configuration And Auth

### GDAL Environment Options

Use `fiona.Env()` when you need GDAL configuration, debug logging, or cloud credentials.

```python
import fiona

with fiona.Env(CPL_DEBUG=True):
    with fiona.open("data/parcels.gpkg") as src:
        print(src.driver)
```

You can also inspect which drivers are actually enabled in the linked GDAL build:

```python
from fiona.env import Env

with Env() as env:
    print(sorted(env.drivers().keys())[:20])
```

### S3 Access

For S3 URIs, either let Fiona create a session from the environment or pass one explicitly.

```python
import boto3
import fiona
from fiona.session import AWSSession

session = boto3.Session(profile_name="geo-prod")

with fiona.Env(session=AWSSession(session, requester_pays=True)):
    with fiona.open("zip+s3://my-bucket/boundaries.zip") as src:
        print(src.bounds)
```

Notes:

- `zip+s3://...` works for zipped vector datasets stored in S3.
- `fio` exposes matching CLI flags such as `--aws-profile`, `--aws-no-sign-requests`, and `--aws-requester-pays`.
- The docs only document `AWSSession` directly; for other storage systems, treat credentials as GDAL environment configuration unless the driver family documents something more specific.

## CLI Workflow

Fiona ships with `fio`, which is useful for quick inspection, format conversion, and stream-based processing:

```bash
fio info data/parcels.gpkg --layer parcels
fio ls data/parcels.gpkg
fio cat data/parcels.gpkg --layer parcels --where "land_use = 'residential'" | head
fio cat data/parcels.gpkg --layer parcels | fio load /tmp/parcels.geojson --driver GeoJSON
```

Newer 1.10 CLI features:

- `fio filter`
- `fio map`
- `fio reduce`

These require the `calc` extra and use parenthesized expression syntax, not the older Python-style filter syntax.

## Common Pitfalls

- Fiona is vector-only. Use `rasterio` or GDAL raster APIs for rasters.
- Fiona features do not provide geometry methods. Convert `feature.geometry` with `shapely.geometry.shape()` before calling spatial methods like `.buffer()` or `.intersection()`.
- Do not assume PyPI wheels support every format driver. If a format fails unexpectedly, inspect enabled drivers and consider a source build or conda-forge environment.
- Writing one feature at a time with repeated `dst.write()` can be much slower than `dst.writerecords(...)` for larger outputs.
- Be explicit about `layer=` for GeoPackage or archive-based inputs. Relying on the first layer is fragile.
- Schema property widths matter. Strings declared as `str:25` are truncated on write.
- Shapefile schema geometry can still yield `MultiPolygon` or `MultiLineString` features at read time. Validate actual feature geometry types, not just the schema headline.
- Unsupported GDAL drivers can sometimes be opened with `allow_unsupported_drivers=True`, but the manual is explicit that correct access is not guaranteed.
- If you need JSON output from a feature object, convert it first. Since 1.9, features are mappings and are not immediately JSON serializable.

## Version-Sensitive Notes For 1.10.1

- PyPI and the stable docs both point to `1.10.1` as the current stable Fiona release as of March 12, 2026.
- `1.10.1` itself is a small bugfix release. The stable changelog notes a CRS logging fix on September 16, 2024.
- The larger behavior changes are in the `1.10.0` line: new Python openers, new `fio filter`/`map`/`reduce` commands, and more explicit movement toward Fiona 2.0 APIs.
- Older Python-style `fio filter` expressions such as `f.properties.area > 1000.0` are deprecated in `1.10` and documented for removal in `2.0`. Use parenthesized list expressions instead.
- Deprecated helpers such as `fiona.path` and old CRS helper functions are on the 2.0 removal path. Prefer `from fiona.crs import CRS` and methods like `CRS.from_epsg(...)`.
- The manual documents that features became mapping objects in `1.9.0`; `1.10.0rc1` restored mutable item access for `Feature`, `Geometry`, and `Properties`, but code should still avoid depending on mutability if you want a smoother Fiona 2.0 migration path.

## Official URLs Used For This Doc

- Stable docs root: `https://fiona.readthedocs.io/en/stable/`
- Installation and changelog: `https://fiona.readthedocs.io/en/stable/README.html`
- User manual: `https://fiona.readthedocs.io/en/stable/manual.html`
- CLI docs: `https://fiona.readthedocs.io/en/stable/cli.html`
- API reference: `https://fiona.readthedocs.io/en/stable/fiona.html`
- PyPI package page: `https://pypi.org/project/fiona/`
- GitHub releases: `https://github.com/Toblerity/Fiona/releases`

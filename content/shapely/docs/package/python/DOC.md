---
name: package
description: "Shapely Python package for planar geometry creation, predicates, overlays, serialization, and spatial indexing"
metadata:
  languages: "python"
  versions: "2.1.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "shapely,geometry,geospatial,gis,geos,spatial"
---

# Shapely Python Package Guide

## Golden Rule

Use `shapely` for in-process planar geometry work, not for CRS management, map rendering, or file I/O. Distances, areas, and buffers are only as correct as your input coordinates, so reproject to an appropriate projected CRS before doing metric operations.

## Install

Pin the package version your project expects:

```bash
python -m pip install "shapely==2.1.2"
```

Common alternatives:

```bash
uv add "shapely==2.1.2"
poetry add "shapely==2.1.2"
```

Notes:

- Prefer wheels unless you specifically need a source build. The 2.1.2 wheels bundle GEOS for typical Linux, macOS, and Windows installs.
- If you must build from source, the installation docs call out a recent GEOS C library and NumPy as build requirements.

## Initialize And Basic Geometry Creation

Shapely 2.x exposes both geometry classes and vectorized functions from the top-level package.

```python
from shapely import LineString, Point, Polygon

pt = Point(0, 0)
road = LineString([(0, 0), (2, 2), (4, 2)])
parcel = Polygon([(0, 0), (4, 0), (4, 3), (0, 3)])

print(pt.geom_type)      # Point
print(parcel.area)       # 12.0
print(road.length)       # 4.82842712474619
```

Use geometry objects when working with one or a few shapes, and use top-level `shapely.*` functions when you want NumPy-friendly bulk operations.

## Core Usage

### Predicates, buffers, and overlays

```python
from shapely import Point, Polygon

parcel = Polygon([(0, 0), (8, 0), (8, 6), (0, 6)])
well = Point(3, 2)
protected_zone = well.buffer(1.5)

print(parcel.contains(well))               # True
print(parcel.intersects(protected_zone))   # True

overlap = parcel.intersection(protected_zone)
remaining = parcel.difference(protected_zone)

print(round(overlap.area, 2))
print(round(remaining.area, 2))
```

Use methods on geometry objects for one-off operations. Most binary operations also have top-level ufunc-style equivalents such as `shapely.intersection()` and `shapely.contains()`.

### Vectorized operations over many geometries

Shapely 2.x integrates PyGEOS-style ufuncs, which are the right choice when you have arrays of geometries.

```python
import numpy as np
import shapely
from shapely import Point, box

window = box(0, 0, 10, 10)
points = np.array([Point(1, 1), Point(5, 5), Point(11, 11)], dtype=object)

mask = shapely.contains(window, points)
clipped = shapely.intersection(shapely.buffer(points, 0.5), window)

print(mask.tolist())          # [True, True, False]
print(clipped[0].geom_type)   # Polygon
```

For repeated predicate tests against the same geometry, prepare it first:

```python
import shapely
from shapely import Point, box

window = box(0, 0, 10, 10)
shapely.prepare(window)

hits = shapely.contains(window, [Point(1, 1), Point(20, 20)])
print(hits.tolist())          # [True, False]
```

### Read and write common interchange formats

Use WKT, WKB, and GeoJSON helpers when you need to cross process boundaries or store shapes in text/binary form.

```python
from shapely import from_wkt, to_wkt
from shapely.geometry import mapping, shape

geom = from_wkt("POINT (1 2)")
as_wkt = to_wkt(geom)

geojson_mapping = mapping(geom)
roundtrip = shape(geojson_mapping)

print(as_wkt)
print(roundtrip.equals(geom))
```

`shape()` and `mapping()` work with GeoJSON-like Python mappings. If you need CRS transforms, use `pyproj` or another CRS library outside Shapely.

### Build a spatial index with `STRtree`

`STRtree` is the built-in immutable spatial index for many-geometry lookup.

```python
from shapely import Point, STRtree, box

geometries = [Point(0, 0), Point(2, 2), Point(5, 5)]
tree = STRtree(geometries)

query_geom = box(1, 1, 3, 3)
matches = tree.query(query_geom)

print(matches.tolist())  # array of matching geometry indices
for idx in matches:
    print(idx, geometries[idx])
```

Use `predicate=` when you want the tree to filter by a spatial relationship instead of only bounding-box overlap.

### Validate and repair invalid geometries

Invalid polygons are common when input rings self-intersect or collapse. Check validity before assuming overlay results are trustworthy.

```python
import shapely
from shapely import Polygon

geom = Polygon([(0, 0), (2, 2), (0, 2), (2, 0), (0, 0)])

print(shapely.is_valid(geom))         # False
fixed = shapely.make_valid(geom)

print(fixed.geom_type)
print(shapely.is_valid(fixed))        # True
```

`make_valid()` can return a different geometry type than the input, including collections. Keep downstream code defensive.

## Configuration And Environment

Shapely has no service auth, API keys, or network configuration. The main setup decisions are about coordinates, dependencies, and performance.

- Coordinate systems: Shapely does not manage CRS metadata or reprojection. Put geometries into the same CRS yourself before comparing or combining them.
- Units: `length`, `area`, `distance`, and `buffer()` use the raw coordinate units. Latitude/longitude degrees are not meters.
- Native library: Wheels include GEOS for normal installs. Source builds depend on the local GEOS toolchain.
- Bulk workflows: NumPy arrays of geometry objects work well with top-level `shapely.*` functions for vectorized processing.

## Common Pitfalls

- Shapely is planar. Z values may be preserved on coordinates, but most analysis is still 2D and ignores height for predicates and measurements.
- `==` compares object equality semantics, not spatial equality. Use `geom.equals(other)` or `shapely.equals(a, b)` when you mean topological equality.
- Multipart geometries in 2.x are not general Python sequences. Use `.geoms` instead of assuming list-like iteration or indexing behavior.
- Geometry objects are immutable in 2.x. Build new geometries instead of trying to mutate coordinates in place or attach ad hoc attributes.
- `STRtree.query()` returns indices into the original geometry sequence, not geometry objects.
- `make_valid()` may change the geometry type. A broken polygon can become a `MultiPolygon` or `GeometryCollection`.
- GeoJSON helpers operate on geometry content only. They do not preserve feature properties, CRS metadata, or application-specific wrappers.

## Version-Sensitive Notes For 2.1.x

- Shapely 2.1.2 is a patch release. Use it for the packaging fixes and GEOS wheel refresh, but expect most API behavior to match the 2.1.0 and 2.1.1 user-manual content.
- The 2.1.0 release added initial support for geometries with M or ZM values, coverage validation and simplification helpers, and a new `orient_polygons()` helper. These are 2.1-era features that older 2.0-focused examples will miss.
- The 2.1.x docs deprecate passing several optional parameters positionally in top-level functions. Prefer explicit keywords such as `grid_size=...`, `include_z=...`, and `indices=...` for forward compatibility.
- Some Read the Docs manual pages still surface as `2.1.1` even while the installation page and release notes are published under `2.1.2`. For package-version facts, trust the 2.1.2 install and release pages first.

## Official Sources

- Docs root: https://shapely.readthedocs.io/en/2.1.2/
- Installation: https://shapely.readthedocs.io/en/2.1.2/installation.html
- Migration guide: https://shapely.readthedocs.io/en/2.1.2/migration.html
- Release notes: https://shapely.readthedocs.io/en/2.1.2/release/2.x.html
- User manual: https://shapely.readthedocs.io/en/2.1.1/manual.html
- API reference index: https://shapely.readthedocs.io/en/stable/_reference.html
- PyPI: https://pypi.org/project/shapely/2.1.2/

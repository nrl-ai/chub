---
name: package
description: "h3 Python package for hexagonal geospatial indexing, traversal, and polygon coverage"
metadata:
  languages: "python"
  versions: "4.4.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "h3,geospatial,gis,hexagonal-grid,geojson,uber"
---

# h3 Python Package Guide

## Golden Rule

Use `h3` for local geospatial indexing and polygon-to-cell coverage, and keep the coordinate conventions straight: core point and boundary APIs use `(lat, lng)`, while GeoJSON-style `__geo_interface__` objects use `(lng, lat)`. For most Python code, stick with the default string API from `import h3` unless you have a measured reason to switch to an integer or NumPy-oriented API.

## Install

Pin the version your project expects:

```bash
python -m pip install "h3==4.4.2"
```

Common alternatives:

```bash
uv add "h3==4.4.2"
poetry add "h3==4.4.2"
```

Optional extra for NumPy-oriented work:

```bash
python -m pip install "h3[numpy]==4.4.2"
```

If you use conda instead of PyPI, the upstream docs point to `conda-forge` as the package source:

```bash
conda config --add channels conda-forge
conda install h3-py
```

## Initialization And API Choice

The default import uses the string-oriented API:

```python
import h3

print(h3.versions())
```

`h3.versions()` returns both the Python wrapper version and the wrapped C library version. Upstream guarantees that the Python and C versions match on major and minor (`X.Y`) even if the patch versions differ.

`h3-py` exposes multiple APIs with the same function names but different input and output types:

- `import h3` or `import h3.api.basic_str as h3`: default, human-readable hex strings
- `import h3.api.basic_int as h3`: Python integers instead of strings
- `import h3.api.numpy_int as h3`: NumPy-friendly integer arrays
- `import h3.api.memview_int as h3`: memoryview-oriented API for lower-overhead bulk work

Choose one API per module and stay consistent. If your project stores H3 indexes as strings in JSON, databases, or logs, the default API is usually the right choice.

## Core Usage

### Convert a point to a cell and back

```python
import h3

lat, lng = 37.769377, -122.388903
cell = h3.latlng_to_cell(lat, lng, 9)

print(cell)                    # '89283082e73ffff'
print(h3.get_resolution(cell)) # 9
print(h3.cell_to_latlng(cell)) # (lat, lng)
print(h3.cell_to_boundary(cell))
```

Use these functions as your baseline:

- `latlng_to_cell(lat, lng, res)`: point to cell
- `cell_to_latlng(cell)`: cell center
- `cell_to_boundary(cell)`: boundary vertices as `(lat, lng)` tuples
- `cell_area(cell, unit="km^2")` and `average_hexagon_area(res, unit="km^2")`: area calculations

### Traverse the grid

```python
import h3

origin = h3.latlng_to_cell(37.769377, -122.388903, 9)

neighbors = h3.grid_disk(origin, 2)
ring = h3.grid_ring(origin, 1)

print(len(neighbors))
print(h3.are_neighbor_cells(origin, ring[0]))
```

Useful traversal functions:

- `grid_disk(cell, k)`: all cells within grid distance `<= k`
- `grid_ring(cell, k)`: cells at exactly distance `k`
- `grid_distance(a, b)`: number of grid steps between cells
- `grid_path_cells(start, end)`: ordered minimum-length path

`grid_disk`, `grid_ring`, and several hierarchy helpers return unordered collections. Do not write code that depends on a stable iteration order unless the function explicitly documents ordered output.

### Move up and down the hierarchy

```python
import h3

cell = h3.latlng_to_cell(37.769377, -122.388903, 9)
parent = h3.cell_to_parent(cell, 7)
children = h3.cell_to_children(parent, 9)

compacted = h3.compact_cells(children)
expanded = h3.uncompact_cells(compacted, 9)
```

Hierarchy helpers that come up often:

- `cell_to_parent(cell, res)`
- `cell_to_children(cell, res)`
- `cell_to_children_size(cell, res)`
- `compact_cells(cells)` / `uncompact_cells(cells, res)`

### Cover a polygon with cells

For GeoJSON-like objects, use `geo_to_cells`. Geo objects use `(lng, lat)` coordinate order.

```python
import h3

polygon = {
    "type": "Polygon",
    "coordinates": [[
        (-122.4089866999972145, 37.813318999983238),
        (-122.3805436999997056, 37.7866302000007224),
        (-122.3544736999993603, 37.7198061999978478),
        (-122.5123436999983966, 37.7076131999975672),
        (-122.5247187000021967, 37.7835871999971715),
        (-122.4798767000009008, 37.8151571999998453),
        (-122.4089866999972145, 37.813318999983238),
    ]],
}

cells = h3.geo_to_cells(polygon, res=9)
shape = h3.cells_to_h3shape(cells)
geojson = h3.cells_to_geo(cells)
```

If you want the explicit polygon classes, use `LatLngPoly` and `LatLngMultiPoly`, which expect `(lat, lng)` points:

```python
import h3

poly = h3.LatLngPoly(
    [
        (37.804, -122.412),
        (37.778, -122.507),
        (37.733, -122.501),
    ]
)

cells = h3.h3shape_to_cells(poly, res=7)
```

Interoperability functions:

- `geo_to_h3shape(geo)` / `h3shape_to_geo(shape)`
- `geo_to_cells(geo, res)` / `cells_to_geo(cells, tight=False)`
- `h3shape_to_cells(shape, res)` / `cells_to_h3shape(cells, tight=False)`
- `polygon_to_cells(...)`: alias for `h3shape_to_cells(...)`

## Configuration And Environment

There is no API key, network configuration, or service endpoint to set. The configuration choices that matter in practice are:

- Index representation: string API vs integer/NumPy APIs
- Coordinate convention: `(lat, lng)` for core H3 functions and `LatLngPoly`, `(lng, lat)` for GeoJSON-like objects
- Coordinate reference system: convert projected data to WGS84 / EPSG:4326 before calling `geo_to_cells`

For GeoPandas or Shapely workflows, make sure the geometry is in latitude/longitude degrees before passing it into `h3`. Upstream explicitly warns that projected CRS values like EPSG:2263 will produce incorrect results.

## Common Pitfalls

- `import h3` is the default string API, not the integer API. If another part of the code expects `uint64`-like integers, import `h3.api.basic_int` or `h3.api.numpy_int` explicitly.
- `latlng_to_cell()` and `cell_to_boundary()` use `(lat, lng)`, but GeoJSON and `__geo_interface__` use `(lng, lat)`. This mismatch is the easiest way to generate wrong cells.
- `cell_to_children()`, `grid_disk()`, `grid_ring()`, and `compact_cells()` work with unordered collections. Sort only if your application truly needs deterministic output.
- `geo_to_cells()` fills cells whose centroids are contained in the polygon. If you need alternate containment semantics, use `h3shape_to_cells_experimental()` or `polygon_to_cells_experimental()` and check the current containment-mode options in the upstream API reference.
- `cells_to_h3shape()` and `cells_to_geo()` assume a valid cell set. Mixed resolutions and duplicate cells are a common source of downstream errors.
- Shapely or GeoPandas objects may be compatible through `__geo_interface__`, but their CRS is your responsibility. Reproject first.
- If you exchange H3 indexes across services, normalize them early. Mixing string IDs and integer IDs in the same code path creates subtle bugs.

## Version-Sensitive Notes For 4.4.2

- PyPI currently lists `4.4.2` as the latest stable release for `h3`, published on January 29, 2026.
- `4.4.2` is a bugfix release. The changelog notes an error check fix in `cellsToLinkedMultiPolygon`, which matters if you convert cell sets back into polygons.
- The `4.4.0` changelog added `is_valid_index`, `get_index_digit`, `construct_cell`, and `deconstruct_cell`, plus several new concrete error classes such as `H3IndexInvalidError`.
- `polygon_to_cells` is a `4.1.x` alias for `h3shape_to_cells`. Older snippets may still use earlier v4 beta names or the v3-era shape API.
- `polygon_to_cells_experimental` and `h3shape_to_cells_experimental` arrived in `4.2.0`; do not assume they exist in older `4.1.x` environments.
- The v4 line changed function names and introduced the newer error system. If you are upgrading old code or copying blog posts written for `3.x`, verify every function name against the current quick reference.
- Upstream has already published `4.5.0` alpha releases as of February 2026, including a future Python minimum of `3.10`. That does not apply to stable `4.4.2`, but it is relevant if you track pre-releases.

## Official Sources

- Docs root: https://uber.github.io/h3-py/
- API quick reference: https://uber.github.io/h3-py/api_quick.html
- API comparison: https://uber.github.io/h3-py/api_comparison.html
- Polygon tutorial: https://uber.github.io/h3-py/polygon_tutorial.html
- Changelog: https://uber.github.io/h3-py/_changelog.html
- PyPI: https://pypi.org/project/h3/

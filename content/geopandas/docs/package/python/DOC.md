---
name: package
description: "GeoPandas package guide for Python geospatial dataframes, CRS workflows, spatial joins, GeoParquet, and PostGIS"
metadata:
  languages: "python"
  versions: "1.1.3"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "geopandas,geospatial,gis,shapely,pyogrio,geoparquet,postgis"
---

# GeoPandas Python Package Guide

## Golden Rule

Use `import geopandas as gpd`, keep the active geometry column and CRS explicit, and reproject before any distance- or area-based operation. GeoPandas operations are planar: `set_crs()` assigns CRS metadata, while `to_crs()` transforms coordinates.

## Install

The official install guide recommends `conda-forge` when compiled geospatial dependencies are hard to resolve manually.

### pip

```bash
python -m pip install "geopandas==1.1.3"
```

### conda-forge

```bash
conda install -c conda-forge geopandas
```

### Useful extras

```bash
python -m pip install "geopandas[all]==1.1.3"
```

Current stable install docs list these core dependency floors:

- `numpy >= 1.24`
- `pandas >= 2.2`
- `pyproj >= 3.7.0`
- `shapely >= 2.1.0`
- `pyogrio >= 0.8.0`
- `packaging`

Optional extras matter for common workflows:

- `pyarrow` for faster `pyogrio` IO and Parquet
- `sqlalchemy`, `psycopg` or `psycopg2`, and `GeoAlchemy2` for PostGIS
- `matplotlib` for `plot()`
- `folium`, `mapclassify`, and `matplotlib` for `explore()`

## Initialize Geometry

Create a `GeoDataFrame` from tabular lon/lat data with an explicit CRS:

```python
import geopandas as gpd
import pandas as pd

df = pd.DataFrame(
    {
        "city": ["San Francisco", "New York"],
        "lon": [-122.4194, -73.9857],
        "lat": [37.7749, 40.7484],
    }
)

gdf = gpd.GeoDataFrame(
    df,
    geometry=gpd.points_from_xy(df["lon"], df["lat"]),
    crs="EPSG:4326",
)

print(gdf.crs)
print(gdf.geometry.name)
```

GeoPandas can keep multiple geometry columns, but spatial methods operate on the active geometry column. Use `set_geometry(...)` if you need to switch.

## Core Workflows

### Read vector data

`read_file()` is the main entry point for files, URLs, and GDAL-backed data sources:

```python
import geopandas as gpd

gdf = gpd.read_file(
    "data/neighborhoods.gpkg",
    layer="neighborhoods",
    columns=["name", "borough", "geometry"],
)

subset = gpd.read_file(
    "data/neighborhoods.gpkg",
    bbox=(-74.1, 40.68, -73.85, 40.88),
)
```

For larger datasets, use Arrow-backed reads when `pyarrow` is installed:

```python
fast = gpd.read_file("data/neighborhoods.gpkg", use_arrow=True)
```

Notes:

- Since GeoPandas 1.0, the default read/write engine is `pyogrio`, not Fiona.
- With the `pyogrio` engine, `bbox=` must already be in the dataset CRS.
- `gpd.list_layers(...)` is available when `pyogrio` is installed.
- `read_file()` returns a plain `pandas.DataFrame` when the source has no geometry column.

### CRS assignment and reprojection

Assign a CRS only when coordinates are already in that CRS. Reproject before buffering, measuring distance, or nearest-neighbor work:

```python
import geopandas as gpd

gdf = gdf.set_crs("EPSG:4326")
gdf_projected = gdf.to_crs(gdf.estimate_utm_crs())
```

Use `allow_override=True` with `set_crs()` only when you are fixing bad metadata. It does not move coordinates.

### Spatial joins

`sjoin()` combines rows by spatial predicate, while `sjoin_nearest()` finds nearest matches:

```python
parcels = parcels.to_crs(parcels.estimate_utm_crs())
schools = schools.to_crs(parcels.crs)

intersections = parcels.sjoin(
    schools[["school_id", "geometry"]],
    how="left",
    predicate="intersects",
)

nearest = parcels.sjoin_nearest(
    schools[["school_id", "geometry"]],
    how="left",
    max_distance=500,
    distance_col="distance_m",
)
```

Important behavior:

- Valid predicates depend on the spatial index backend; inspect `left_df.sindex.valid_query_predicates`.
- `predicate="dwithin"` requires a `distance=` argument.
- `sjoin_nearest()` distances are in CRS units. Using a geographic CRS such as `EPSG:4326` gives inaccurate distance results.
- Multiple equidistant nearest geometries produce multiple output rows.

### Overlay and clip-style geometry operations

Use `overlay()` when you need geometric set operations between two layers:

```python
result = gpd.overlay(
    parcels,
    flood_zones,
    how="intersection",
    keep_geom_type=True,
)
```

Notes:

- `overlay()` expects uniform geometry types per input layer.
- The default `keep_geom_type=True` drops result geometries that do not match the left-hand geometry type and emits a warning. Set `keep_geom_type=False` if you need those rows.
- `make_valid=True` is the default and repairs invalid inputs before overlay. Set `make_valid=False` if you would rather fail fast.

### GeoParquet and other columnar formats

GeoPandas reads and writes Parquet and Feather through Arrow-backed paths:

```python
gdf.to_parquet(
    "out/places.parquet",
    compression="zstd",
)

round_tripped = gpd.read_parquet("out/places.parquet")
```

Useful notes:

- `to_parquet()` supports both WKB encoding and experimental GeoArrow-based native geometry encoding.
- GeoParquet 1.1 support landed in GeoPandas 1.0 and is still new enough that older tutorials may miss it.

### PostGIS

Read from and write to PostGIS through SQLAlchemy engines or connections:

```python
from sqlalchemy import create_engine
import geopandas as gpd

engine = create_engine("postgresql+psycopg://user:pass@host:5432/geodb")

roads = gpd.read_postgis(
    "SELECT road_id, geom FROM roads",
    engine,
    geom_col="geom",
)

roads.to_postgis(
    "roads_copy",
    engine,
    if_exists="replace",
    index=False,
)
```

`read_postgis()` expects the geometry column to contain WKB values. `to_postgis()` requires SQLAlchemy, a PostgreSQL driver, and GeoAlchemy2.

## Configuration And Backend Notes

GeoPandas does not define package-level authentication of its own. From the official IO APIs, auth is delegated to the file, database, or cloud backend you use.

Practical configuration points:

- Set `PYOGRIO_USE_ARROW=1` to enable Arrow transfers by default for supported `pyogrio` reads and writes.
- Put PostGIS credentials in the SQLAlchemy URL, connection settings, or your secret manager.
- When you pass GDAL-backed file paths or URLs to `read_file()`, backend-specific access rules come from the underlying driver stack rather than GeoPandas itself.

## Common Pitfalls

### Confusing `set_crs()` with `to_crs()`

- `set_crs()` assigns metadata to existing coordinates.
- `to_crs()` transforms coordinates into a new CRS.

If you use `set_crs()` when you meant `to_crs()`, every downstream spatial result will be wrong.

### Measuring in a geographic CRS

Distance, area, and nearest-neighbor operations are planar. Reproject first when you need meters or other projected units.

### Assuming old Fiona or PyGEOS behavior

Many blog posts still describe pre-1.0 behavior. Since 1.0:

- `pyogrio` is the default IO engine
- Shapely 2 is required
- support for PyGEOS and `rtree`-based spatial indexing was removed
- `geopandas.datasets` was removed

### Losing geometries in overlay results

If `overlay()` appears to "drop" rows, check whether `keep_geom_type=True` filtered out line or point results that came from polygon inputs.

### Forgetting the active geometry column

If a frame has multiple geometry columns, `plot`, `sjoin`, `overlay`, `to_crs`, and other geometry-aware methods use the active one. Switch it explicitly with `set_geometry(...)`.

## Version-Sensitive Notes For 1.1.x

- `1.1.3` includes fixes for pandas 3.0 copy-on-write behavior, `GeoSeries.value_counts()`, `GeoSeries.fillna()`, `overlay(..., identity)`, and NA handling in `from_wkt` / `from_wkb`.
- `1.1.2` fixed several regressions, including some `read_file()`, `to_file()`, and `read_parquet()` failures with complex Arrow types.
- Stable install docs now show dependency floors that are stricter than older 0.x and early 1.0 tutorials. If examples mention PyGEOS, `rtree`, or Fiona-as-default, they are outdated for 1.1.3.

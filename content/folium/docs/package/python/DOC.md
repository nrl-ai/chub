---
name: package
description: "Folium Python package for building Leaflet-based interactive maps, layers, GeoJSON overlays, choropleths, and map plugins"
metadata:
  languages: "python"
  versions: "0.20.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "folium,maps,leaflet,geojson,visualization,gis"
---

# Folium Python Package Guide

## Golden Rule

Use `folium` when you need to generate interactive Leaflet maps from Python and deliver them as HTML. Treat the Python object as a map builder, not a GIS engine: prepare your data first, then use Folium to render tiles, markers, GeoJSON, choropleths, and plugins into a notebook or saved HTML file.

## Install

Pin the package version your project expects:

```bash
python -m pip install "folium==0.20.0"
```

Common alternatives:

```bash
uv add "folium==0.20.0"
poetry add "folium==0.20.0"
conda install -c conda-forge folium
```

Upstream getting-started docs list these automatic dependencies for the base install:

- `branca`
- `jinja2`
- `numpy`
- `requests`

PyPI also publishes a `testing` extra for contributor or package-maintainer workflows.

## Initialize And Render A Map

Folium maps are HTML/JavaScript artifacts. In notebooks, display the map object directly. In scripts or web backends, save the output to HTML.

```python
import folium

m = folium.Map(
    location=(45.5236, -122.6750),
    zoom_start=12,
    control_scale=True,
)

folium.Marker(
    location=(45.528, -122.68),
    tooltip="Downtown",
    popup="Portland",
).add_to(m)

m.save("map.html")
```

Key defaults and behaviors:

- `location` is latitude, longitude.
- The default basemap is `OpenStreetMap`.
- The rendered output is not visible in an untrusted Jupyter notebook until the notebook is trusted.
- `LayerControl` should be added after the layers it controls.

## Core Usage

### Create a map with a different tile provider

Folium 0.20.0 supports built-in provider names and `xyzservices` tile providers. If you pass a custom tile URL, also pass attribution with `attr`.

```python
import folium

m = folium.Map(
    location=(37.7749, -122.4194),
    zoom_start=11,
    tiles="CartoDB Positron",
)

folium.TileLayer(
    tiles="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
    attr="&copy; OpenStreetMap contributors",
    name="OSM",
).add_to(m)

folium.LayerControl().add_to(m)
```

If you need a map with no basemap, start with `tiles=None` and add your own layers later.

### Group layers and toggle them

```python
import folium

m = folium.Map((0, 0), zoom_start=3)

cities = folium.FeatureGroup(name="Cities").add_to(m)
routes = folium.FeatureGroup(name="Routes").add_to(m)

folium.Marker((51.5072, -0.1276), tooltip="London").add_to(cities)
folium.Marker((48.8566, 2.3522), tooltip="Paris").add_to(cities)

folium.PolyLine(
    locations=[(51.5072, -0.1276), (48.8566, 2.3522)],
    color="blue",
    weight=3,
).add_to(routes)

folium.LayerControl(collapsed=False).add_to(m)
```

Use `FeatureGroup` for logical toggles. If the layer should appear in the control, keep `control=True` and add the control last.

### Add GeoJSON with styling and tooltips

`folium.GeoJson` is the main entry point for polygon, line, and point feature overlays.

```python
import folium

geojson = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "properties": {"name": "Area A", "value": 10},
            "geometry": {
                "type": "Polygon",
                "coordinates": [[
                    [-122.53, 45.52],
                    [-122.53, 45.55],
                    [-122.48, 45.55],
                    [-122.48, 45.52],
                    [-122.53, 45.52],
                ]],
            },
        }
    ],
}

m = folium.Map(location=(45.53, -122.50), zoom_start=13)

folium.GeoJson(
    geojson,
    name="Areas",
    style_function=lambda feature: {
        "fillColor": "#2a9d8f",
        "color": "#1d3557",
        "weight": 2,
        "fillOpacity": 0.5,
    },
    highlight_function=lambda feature: {
        "weight": 4,
        "fillOpacity": 0.8,
    },
    tooltip=folium.GeoJsonTooltip(fields=["name", "value"]),
    zoom_on_click=True,
).add_to(m)

folium.LayerControl().add_to(m)
```

Prefer `GeoJson` when you want full styling control, hover highlighting, popups/tooltips, or click-to-zoom behavior.

### Build a choropleth from tabular data

`folium.Choropleth` is the fast path for coloring features from a data table. The most common mistake is using the wrong `key_on` expression for your GeoJSON properties or IDs.

```python
import folium
import pandas as pd

geojson = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "id": "A",
            "properties": {"name": "Area A"},
            "geometry": {
                "type": "Polygon",
                "coordinates": [[
                    [-122.53, 45.52],
                    [-122.53, 45.55],
                    [-122.48, 45.55],
                    [-122.48, 45.52],
                    [-122.53, 45.52],
                ]],
            },
        },
        {
            "type": "Feature",
            "id": "B",
            "properties": {"name": "Area B"},
            "geometry": {
                "type": "Polygon",
                "coordinates": [[
                    [-122.48, 45.52],
                    [-122.48, 45.55],
                    [-122.43, 45.55],
                    [-122.43, 45.52],
                    [-122.48, 45.52],
                ]],
            },
        },
    ],
}

data = pd.DataFrame(
    [
        {"region_id": "A", "score": 10},
        {"region_id": "B", "score": 25},
    ]
)

m = folium.Map(location=(45.53, -122.48), zoom_start=13)

folium.Choropleth(
    geo_data=geojson,
    data=data,
    columns=["region_id", "score"],
    key_on="feature.id",
    fill_color="YlGnBu",
    fill_opacity=0.7,
    line_opacity=0.2,
    legend_name="Score",
    highlight=True,
).add_to(m)
```

Useful options from the upstream guide:

- `bins` when the default color scaling is not appropriate
- `nan_fill_color` and `nan_fill_opacity` for missing values
- `name` and `show` if the choropleth should participate in `LayerControl`

### Cluster many markers

For dense point data, use `folium.plugins.MarkerCluster` instead of adding hundreds of plain markers directly to the map.

```python
import folium
from folium.plugins import MarkerCluster

m = folium.Map(location=(44, -73), zoom_start=5)
cluster = MarkerCluster(name="Sites").add_to(m)

points = [
    (40.67, -73.94, "Brooklyn"),
    (44.67, -73.94, "Vermont"),
    (44.67, -71.94, "New Hampshire"),
]

for lat, lon, label in points:
    folium.Marker((lat, lon), popup=label, tooltip=label).add_to(cluster)

folium.LayerControl().add_to(m)
```

### Search GeoJSON or grouped features

The `Search` plugin can search `GeoJson`, `TopoJson`, `FeatureGroup`, or `MarkerCluster` layers. Set `search_label` to a property that actually exists in the target layer.

```python
import folium
from folium.plugins import Search

geojson = {
    "type": "FeatureCollection",
    "features": [
        {
            "type": "Feature",
            "properties": {"name": "Central Park"},
            "geometry": {"type": "Point", "coordinates": [-73.9654, 40.7829]},
        }
    ],
}

m = folium.Map(location=(40.78, -73.97), zoom_start=13)

places = folium.GeoJson(
    geojson,
    name="Places",
    tooltip=folium.GeoJsonTooltip(fields=["name"]),
).add_to(m)

Search(
    layer=places,
    search_label="name",
    placeholder="Search places",
    collapsed=False,
).add_to(m)
```

## Configuration And Integration Notes

### Notebooks

- Display the map object directly in Jupyter: `m`
- If the map area stays blank, trust the notebook first
- Save with `m.save("map.html")` when you need a portable artifact outside the notebook

### Flask, Django, and other server-rendered apps

Folium generates HTML with linked JavaScript and CSS resources. Common patterns are:

- generate a full standalone file with `m.save(...)`
- embed `m.get_root().render()` into a server-rendered template
- write the HTML to object storage or a temp file if another service will host it

Because Folium emits browser code, it is usually simpler to generate maps on the server and let the browser render them than to treat Folium as an API you call from frontend JavaScript.

### Tile providers and API keys

Folium itself has no package-level authentication, but some third-party tile or geocoding providers do. Practical rules:

- if you pass a custom tile URL, set `attr` explicitly
- if a plugin provider needs credentials, pass them in the provider-specific options the plugin accepts
- keep provider tokens outside source control and inject them from environment variables

Example with an environment variable inside a tile URL template:

```python
import os
import folium

tile_token = os.environ["MAP_TILE_TOKEN"]

m = folium.Map(
    location=(37.7749, -122.4194),
    zoom_start=11,
    tiles=f"https://tiles.example.com/{{z}}/{{x}}/{{y}}.png?token={tile_token}",
    attr="Example Maps",
)
```

### Custom JS and CSS resources

Classes that inherit from Folium's JS/CSS mixin can load custom resources. Use this when your deployment needs a private CDN or a patched asset URL.

```python
import folium

m = folium.Map()
m.add_css_link(
    "bootstrap_css",
    "https://example.com/bootstrap/400.5.0/css/bootstrap.min.css",
)
```

## Common Pitfalls

- Folium expects latitude, longitude order for `location`; GeoJSON coordinate arrays stay longitude, latitude.
- The map object is only a renderer. If your data needs joins, projections, cleaning, or aggregation, do that before building the map.
- Blank notebook output usually means the notebook is not trusted or required assets were blocked.
- `LayerControl` must be added after the layers you want it to control.
- Custom tile URLs need attribution via `attr`, and provider terms can require API keys or usage limits.
- `Choropleth` joins fail quietly when `columns` and `key_on` do not line up with GeoJSON identifiers.
- Large GeoJSON payloads can make notebook output or saved HTML files heavy. For large point sets, prefer clustering; for large polygons, simplify the geometry before rendering.
- Search only works against the properties exposed by the target layer. If `search_label` does not exist, the UI appears but results will be wrong or empty.

## Version-Sensitive Notes For 0.20.0

- As of March 12, 2026, PyPI still lists `0.20.0` as the latest Folium release, published on June 16, 2025.
- The official docs root and API reference are aligned to `0.20.0`, so the version used here and upstream version match for this session.
- PyPI now requires Python `>=3.9`; older project environments pinned to Python 3.8 need an older Folium release.
- The upstream `v0.20.0` release notes are short, so treat the docs and API reference as the authoritative source for behavior rather than relying on release-note summaries alone.

## Official Sources

- Folium docs root: `https://python-visualization.github.io/folium/latest/`
- Folium getting started: `https://python-visualization.github.io/folium/latest/getting_started.html`
- Folium API reference: `https://python-visualization.github.io/folium/latest/reference.html`
- Folium user guide: `https://python-visualization.github.io/folium/latest/user_guide.html`
- Folium choropleth guide: `https://python-visualization.github.io/folium/latest/user_guide/geojson/choropleth.html`
- Folium MarkerCluster guide: `https://python-visualization.github.io/folium/latest/user_guide/plugins/marker_cluster.html`
- Folium Search guide: `https://python-visualization.github.io/folium/latest/user_guide/plugins/search.html`
- Folium custom tiles guide: `https://python-visualization.github.io/folium/latest/advanced_guide/custom_tiles.html`
- Folium JS/CSS customization guide: `https://python-visualization.github.io/folium/latest/advanced_guide/customize_javascript_and_css.html`
- PyPI package page: `https://pypi.org/project/folium/`
- GitHub release notes for `v0.20.0`: `https://github.com/python-visualization/folium/releases/tag/v0.20.0`

---
name: package
description: "pygal Python package guide for generating SVG charts, styling them, and exporting them for web or file output"
metadata:
  languages: "python"
  versions: "3.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "pygal,python,charts,svg,plotting,visualization"
---

# pygal Python Package Guide

## Golden Rule

Use `pygal` when you want SVG-first chart generation from Python. Build a chart object, add one or more series, then render to SVG, a file, or PNG. Do not assume maps are bundled in core `pygal`; map charts live in separate plugin packages.

As of March 12, 2026, PyPI lists `pygal 3.1.0`, but the upstream docs site still labels many pages as `3.0.5`. The official changelog says `3.1.0` mainly fixes docs and adds `reverse_direction` for `Gauge`, so most stable docs examples still apply to `3.1.0`.

## Install

Pin the package version your project expects:

```bash
python -m pip install "pygal==3.1.0"
```

Common alternatives:

```bash
uv add "pygal==3.1.0"
poetry add "pygal==3.1.0"
```

Optional dependencies matter for output quality and convenience:

- PNG export needs `cairosvg` for `render_to_png()`
- `lxml` can improve rendering behavior and enables some helpers such as `render_in_browser()`
- The docs note that `tinycss` and `cssselect` can help when rendered images appear black

One practical install for SVG plus PNG export is:

```bash
python -m pip install "pygal==3.1.0" cairosvg lxml tinycss cssselect
```

## Initialize And Render A Basic Chart

The core workflow is always:

1. Create a chart instance such as `Bar`, `Line`, or `Pie`
2. Set chart-level options like `title` or `x_labels`
3. Add one or more series with `add()`
4. Render to bytes, a file, a browser response, or PNG

Minimal example:

```python
import pygal

chart = pygal.Bar()
chart.title = "Monthly signups"
chart.x_labels = ["Jan", "Feb", "Mar", "Apr"]
chart.add("Web", [12, 18, 21, 30])
chart.add("Mobile", [8, 11, 15, 19])

chart.render_to_file("signups.svg")
```

If you need the SVG in memory:

```python
svg_bytes = chart.render()
svg_text = chart.render(is_unicode=True)
data_uri = chart.render_data_uri()
```

`render()` returns bytes by default. Use `is_unicode=True` if your framework or template code expects a string.

## Core Usage Patterns

### Reuse configuration across charts

Use `pygal.Config` when several charts should share the same settings:

```python
import pygal
from pygal import Config

config = Config()
config.show_legend = True
config.explicit_size = True
config.width = 900
config.height = 400
config.x_label_rotation = 20

sales = pygal.Line(config)
sales.title = "Quarterly revenue"
sales.x_labels = ["Q1", "Q2", "Q3", "Q4"]
sales.add("2025", [120, 150, 170, 210])
sales.add("2026", [140, 165, 190, 240])
sales.render_to_file("revenue.svg")
```

You can also pass config values directly to the chart constructor:

```python
chart = pygal.Line(show_legend=False, explicit_size=True, width=700, height=300)
```

### Use styles instead of hand-editing SVG

Built-in and custom styles are the intended customization path:

```python
import pygal
from pygal.style import Style

custom_style = Style(
    background="transparent",
    plot_background="transparent",
    foreground="#2d3142",
    foreground_subtle="#7d8597",
    colors=("#ef8354", "#4f5d75", "#2d3142"),
)

chart = pygal.StackedBar(style=custom_style)
chart.title = "Tickets by team"
chart.x_labels = ["Mon", "Tue", "Wed"]
chart.add("Support", [18, 21, 17])
chart.add("Success", [9, 11, 8])
chart.render_to_file("tickets.svg")
```

If you already know a built-in style fits, use imports like `DarkStyle`, `LightStyle`, `CleanStyle`, or `BlueStyle`.

### Add metadata to individual values

`pygal` accepts dictionaries instead of raw values when you need labels, links, colors, or point-specific formatting:

```python
import pygal

chart = pygal.Bar(print_values=True)
chart.add(
    "Errors",
    [
        {"value": 2, "label": "API timeout", "color": "#d90429"},
        {"value": 5, "label": "Validation", "xlink": {"href": "https://example.com/runbook"}},
        {"value": 1, "style": "fill: #2b2d42; stroke: #000; stroke-width: 2"},
    ],
)

chart.render_to_file("errors.svg")
```

Use this when an agent needs tooltips, point-level links, or exceptions highlighted without splitting the series.

### Export PNG when SVG is not enough

```python
import pygal

chart = pygal.Line()
chart.add("CPU", [20, 35, 42, 33])
chart.render_to_png("cpu.png")
```

This requires `cairosvg`. If PNG output is black or malformed, the docs recommend installing `lxml`, `tinycss`, and `cssselect`.

## Web And Framework Output

`pygal` is SVG-first, so web integration is usually simple.

### Flask

```python
from flask import Flask
import pygal

app = Flask(__name__)

@app.get("/traffic.svg")
def traffic_chart():
    chart = pygal.Line()
    chart.add("Visits", [100, 120, 180, 160])
    return chart.render_response()
```

### Django

```python
import pygal

def traffic_chart(request):
    chart = pygal.Line()
    chart.add("Visits", [100, 120, 180, 160])
    return chart.render_django_response()
```

### Inline HTML

If you embed the raw SVG directly into HTML, disable the XML declaration and include the tooltip JavaScript yourself:

```python
svg = chart.render(is_unicode=True, disable_xml_declaration=True)
```

The docs also expose a configurable `js` setting for external tooltip assets. If tooltips are not appearing, check whether the generated SVG still references the expected JS and whether your page permits loading it.

## Maps And Plugins

Map charts are not shipped inside the core package. Install the plugin you need, then use the `pygal.maps.*` namespace:

```bash
python -m pip install pygal_maps_world
```

```python
import pygal.maps.world

worldmap = pygal.maps.world.World()
worldmap.title = "Requests by country"
worldmap.add("Traffic", {"us": 1200, "fr": 320, "de": 280})
worldmap.render_to_file("traffic-world.svg")
```

The official docs list separate packages such as `pygal_maps_world`, `pygal_maps_fr`, and `pygal_maps_ch`.

## Configuration Notes

- There is no authentication model. The main configuration surface is chart options, style, output format, and optional plugin or rendering dependencies.
- `Config` objects are reusable and are the cleanest way to keep a consistent chart style across a codebase.
- Some options can be supplied at chart creation time, on the chart instance, or during `render(...)`.
- Tooltips depend on JavaScript. Static SVG files still render without it, but interactive hover behavior will be missing.
- `explicit_size=True` is useful when embedding SVGs into layouts that need fixed `width` and `height` instead of just a `viewBox`.

## Common Pitfalls

- `render()` returns bytes, not text. Use `is_unicode=True` if you need a string.
- The docs site version banner is stale. Validate feature additions against the changelog when copying examples.
- Maps require separate packages; `pip install pygal` alone is not enough for `pygal.maps.world`, `pygal.maps.fr`, or `pygal.maps.ch`.
- Some SVG renderers do not handle pygal’s CSS styling well and can show black output. The upstream docs call out `lxml`, `tinycss`, and `cssselect` as fixes to try.
- PNG export is optional. If `render_to_png()` fails, check for missing `cairosvg` and related rendering dependencies first.
- Interactive tooltips need the pygal JavaScript asset. Inline SVG without that JS still draws the chart but loses hover behavior.
- The docs site still contains old examples using `human_readable=True`; since `2.2.0`, the changelog says to use `pygal.formatters.human_readable` via `value_formatter` instead.

## Version-Sensitive Notes For 3.1.0

- PyPI release `3.1.0` was published on December 9, 2025.
- PyPI requires Python `>=3.8`.
- The official changelog says `3.1.0` adds the `reverse_direction` option for `Gauge` and otherwise contains lint and docs fixes.
- The public docs site still labels many pages as `3.0.5`, so use the changelog to sanity-check newer features rather than assuming the page banner matches the installed version.

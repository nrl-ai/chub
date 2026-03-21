---
name: package
description: "Dash package guide for Python projects building reactive web apps with Plotly Dash 4"
metadata:
  languages: "python"
  versions: "4.0.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "dash,plotly,python,web,callbacks,data-visualization,flask"
---

# Dash Python Package Guide

## Golden Rule

Use `dash` as the app shell and callback framework, import components from `dash`, and keep long-running work out of normal request-thread callbacks. For `4.0.0`, also treat component styling and some default values as upgrade-sensitive.

## Install

Pin the version your project expects:

```bash
python -m pip install "dash==4.0.0"
```

Common alternatives:

```bash
uv add "dash==4.0.0"
poetry add "dash==4.0.0"
```

Useful extras from the official package metadata:

```bash
python -m pip install "dash[async]==4.0.0"
python -m pip install "dash[testing]==4.0.0"
python -m pip install "dash[diskcache]==4.0.0"
python -m pip install "dash[celery]==4.0.0"
python -m pip install "dash[cloud]==4.0.0"
```

Notes:

- The installation guide states that `plotly` is installed with `dash`.
- Many tutorial examples also assume `pandas` is installed for data loading and `plotly.express`.
- `dash[testing]` is the supported way to install `dash.testing` and pytest fixtures.

## Initialize A Minimal App

```python
from dash import Dash, dcc, html
import plotly.express as px
import pandas as pd

df = pd.DataFrame(
    {
        "Fruit": ["Apples", "Oranges", "Bananas", "Apples", "Oranges", "Bananas"],
        "Amount": [4, 1, 2, 2, 4, 5],
        "City": ["SF", "SF", "SF", "Montreal", "Montreal", "Montreal"],
    }
)

fig = px.bar(df, x="Fruit", y="Amount", color="City", barmode="group")

app = Dash()
app.layout = html.Div(
    [
        html.H1("Hello Dash"),
        html.Div("Dash: a web application framework for your data."),
        dcc.Graph(id="example-graph", figure=fig),
    ]
)

if __name__ == "__main__":
    app.run(debug=True)
```

Run the app with:

```bash
python app.py
```

The default local dev server listens on `http://127.0.0.1:8050/`.

## Core Usage

### Write callbacks with `@callback`

Dash callbacks connect component properties to Python functions. The standard import pattern in the current docs is:

```python
from dash import Dash, dcc, html, Input, Output, callback

app = Dash()
app.layout = html.Div(
    [
        dcc.Input(id="my-input", value="initial value", type="text"),
        html.Div(id="my-output"),
    ]
)

@callback(
    Output("my-output", "children"),
    Input("my-input", "value"),
)
def update_output(value: str) -> str:
    return f"Output: {value}"

if __name__ == "__main__":
    app.run(debug=True)
```

### Multi-page apps with Dash Pages

For new apps, prefer Dash Pages instead of hand-rolled URL routing.

`app.py`:

```python
from dash import Dash, html, dcc, page_container, page_registry

app = Dash(__name__, use_pages=True)
app.layout = html.Div(
    [
        html.H1("My app"),
        html.Nav(
            [
                dcc.Link(f"{page['name']}", href=page["relative_path"])
                for page in page_registry.values()
            ]
        ),
        page_container,
    ]
)

if __name__ == "__main__":
    app.run(debug=True)
```

`pages/home.py`:

```python
import dash
from dash import html

dash.register_page(__name__, path="/")

layout = html.Div("Home page")
```

Key rules:

- Create a `/pages` directory.
- Call `dash.register_page(__name__)` in each page module.
- Set `use_pages=True` on the app.
- Put `dash.page_container` where routed page content should render.

### Use `ctx`, `PreventUpdate`, and `no_update` for real callback control

These patterns are essential once apps move beyond toy examples:

```python
from dash import Dash, Input, Output, callback, ctx, html, no_update
from dash.exceptions import PreventUpdate

app = Dash()
app.layout = html.Div(
    [
        html.Button("A", id="btn-a"),
        html.Button("B", id="btn-b"),
        html.Div(id="result"),
    ]
)

@callback(
    Output("result", "children"),
    Input("btn-a", "n_clicks"),
    Input("btn-b", "n_clicks"),
)
def show_trigger(btn_a, btn_b):
    if not ctx.triggered_id:
        raise PreventUpdate
    if ctx.triggered_id == "btn-a":
        return "A clicked"
    return no_update
```

Use these intentionally:

- `ctx.triggered_id`: determine which input fired
- `PreventUpdate`: stop all outputs from changing
- `no_update`: keep one output unchanged while updating others
- `prevent_initial_call=True`: avoid callback execution on first page load when that behavior is unwanted

### Use background callbacks for slow work

Normal callbacks run in the web request path. The docs call out 30 second web-server timeouts as a common problem for long jobs. Use background callbacks for expensive work:

```python
import os
from dash import Dash, Input, Output, html, callback, DiskcacheManager, CeleryManager

if "REDIS_URL" in os.environ:
    from celery import Celery

    celery_app = Celery(__name__, broker=os.environ["REDIS_URL"], backend=os.environ["REDIS_URL"])
    background_callback_manager = CeleryManager(celery_app)
else:
    import diskcache

    cache = diskcache.Cache("./cache")
    background_callback_manager = DiskcacheManager(cache)

app = Dash(background_callback_manager=background_callback_manager)
app.layout = html.Div([html.Button("Run", id="run"), html.Div(id="result")])

@callback(
    Output("result", "children"),
    Input("run", "n_clicks"),
    background=True,
    prevent_initial_call=True,
)
def run_job(n_clicks):
    return f"Started job {n_clicks}"
```

Operational guidance:

- Use `dash[diskcache]` for local development.
- Use `dash[celery]` plus Redis or another Celery broker for production.
- Background callbacks replaced long callbacks in Dash 3.0; do not build new code on the removed long-callback API.

### Async callbacks are available

Dash supports `async` and `await` in callbacks when installed with `dash[async]`.

```python
import asyncio
from dash import Dash, Input, Output, html

app = Dash()
app.layout = html.Div(
    [
        html.Button("Run", id="run"),
        html.Div(id="out"),
    ]
)

@app.callback(Output("out", "children"), Input("run", "n_clicks"), prevent_initial_call=True)
async def fetch_data(_):
    await asyncio.sleep(1)
    return "done"
```

The advanced callbacks page notes a deployment caveat: if you use `gunicorn`, use the `gthread` worker class, not `gevent`.

## Assets, Server, And Configuration

### Assets

Dash automatically serves files placed in an `assets/` directory at the app root.

```text
app.py
assets/
  typography.css
  custom-script.js
```

Important details from the official docs:

- The default assets URL path is `/assets`.
- You can change it with the `assets_url_path` argument to `Dash`.
- In Dash `2.14+`, `__name__` is no longer required in the `Dash(...)` constructor for these examples, so `Dash()` is fine in `4.0.0`.

### Server object

If you need Flask-level integration for deployment or middleware, Dash exposes the underlying server:

```python
app = Dash()
server = app.server
```

This is the integration point to use when your deployment stack expects a WSGI app object or when you need to add Flask-side behavior around the Dash app.

### Authentication

Authentication is not a first-class setup step in the Dash OSS package docs. In practice, treat auth as a deployment concern and implement it in the hosting layer, reverse proxy, or the underlying Flask server rather than expecting `dash` itself to provide a complete login system.

## Testing

Dash ships a supported testing surface in `dash.testing`:

```bash
python -m pip install "dash[testing]==4.0.0"
```

The testing docs describe two main layers:

- callback unit tests
- end-to-end browser tests with Dash pytest fixtures

If you use Zsh, escape the bracket form when needed:

```bash
python -m pip install dash\[testing\]
```

## Common Pitfalls

- Forgetting that callbacks fire on initial page load. Use `prevent_initial_call=True` when the first run is undesirable.
- Blocking the request thread with heavy work. Move slow tasks to background callbacks instead of increasing web-server timeouts and hoping for the best.
- Expecting dynamic or conditional components to exist for every callback. For dynamic layouts, use `allow_optional=True` where appropriate, and use `suppress_callback_exceptions=True` only when you truly need it.
- Missing the current input that triggered a callback. Prefer `ctx.triggered_id` over brittle parsing of older `callback_context.triggered` shapes.
- Keeping large mutable state in Python globals. For browser-side JSON-sized state, use `dcc.Store`; for server-side shared state, use a real cache or datastore.
- Continuing to build on `dash_table.DataTable` without a migration plan. The official docs mark Dash DataTable as deprecated and state that it will be removed from the core Dash API in Dash 5.0; new table-heavy work should evaluate `dash-ag-grid`.
- Assuming old CSS will survive the Dash 4 upgrade unchanged. Dash 4 updates the styling of several core components and changes some defaults, especially around sliders and dropdowns.

## Version-Sensitive Notes For `4.0.0`

- PyPI lists `dash 4.0.0` as the latest stable release, released on February 3, 2026. PyPI also shows `4.1.0rc0` on February 23, 2026, so avoid pinning to `4.1` behavior unless you explicitly want a release candidate.
- The Dash 4 release updates styling for many `dcc` components. If your app has custom CSS targeting Dash-generated class names or previous visual defaults, regression-test the UI before upgrading.
- `dcc.Dropdown` defaults changed in Dash 4: `optionHeight` now defaults to `auto`, and `closeOnSelect` defaults to `True` for single-select and `False` for multi-select dropdowns.
- Slider `step` behavior changed in Dash 4. It is still `1` when `min` and `max` are integers, but otherwise it is dynamically computed.
- `dcc.Button` is new in Dash 4 and is styled to match other Dash Core Components.
- Features introduced during the Dash 2.x and 3.x line remain relevant in `4.0.0`, including Pages (`2.5`), partial property updates (`2.9`), `running` callback output updates (`2.16`), `async` callbacks (`3.1`), and optional callback inputs (`3.1`).

## Official Sources

- Dash docs root: `https://dash.plotly.com/`
- Installation: `https://dash.plotly.com/installation`
- Layout: `https://dash.plotly.com/layout`
- Basic callbacks: `https://dash.plotly.com/basic-callbacks`
- Multi-page apps and URLs: `https://dash.plotly.com/urls`
- Advanced callbacks: `https://dash.plotly.com/advanced-callbacks`
- Background callbacks: `https://dash.plotly.com/background-callbacks`
- External resources and assets: `https://dash.plotly.com/external-resources`
- Dash 4 release notes: `https://dash.plotly.com/whats-new-in-dash-4`
- DataTable reference and deprecation notice: `https://dash.plotly.com/datatable/reference`
- Testing: `https://dash.plotly.com/testing`
- PyPI package metadata: `https://pypi.org/project/dash/`

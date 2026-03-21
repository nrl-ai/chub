---
name: package
description: "voila package guide for turning Jupyter notebooks into standalone web apps and dashboards"
metadata:
  languages: "python"
  versions: "0.5.11"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "voila,jupyter,ipywidgets,notebook,dashboard,webapp"
---

# voila Python Package Guide

## Golden Rule

Use `voila` to serve notebooks, not as a generic Python web framework. Keep the notebook, widget libraries, and the Voilà server in the same environment, choose either standalone CLI mode or Jupyter server extension mode deliberately, and configure auth plus reverse-proxy behavior explicitly before exposing the app outside local development.

## Version-Sensitive Notes

- As of March 12, 2026, PyPI and the upstream GitHub releases page both show `0.5.11` as the latest published release.
- The docs URL points at `en/stable`, but some stable pages still render as `0.5.8`. For package-level work, prefer `https://voila.readthedocs.io/en/latest/` until the stable alias is refreshed upstream.
- `0.5.11` adds `extra_labextensions_path` support and fixes server-extension local config merging.
- `0.5.8` added `progressive_rendering`, ipywidgets 7 compatibility, and `page_config_hook`.
- `0.5.0` was the major frontend transition: Voilà moved to a JupyterLab 4 based frontend, gained token auth support with `jupyter-server` 2, and changed tree-page/template behavior. Old blog posts written for `0.3.x` or early `0.4.x` are often misleading now.
- The `classic` template is deprecated and upstream says it will be removed in Voilà `1.0.0`.

## Install

Use an isolated environment unless the notebook server is already isolated:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "voila==0.5.11"
```

Conda and mamba are first-class upstream install paths:

```bash
mamba install -c conda-forge voila
conda install -c conda-forge voila
```

Basic verification:

```bash
voila --help
python -m pip show voila
```

Important install notes:

- `pip install voila` is enough for the CLI itself.
- The notebook's runtime dependencies must also be installed in the same environment as the kernel that Voilà will launch.
- The PyPI project description says the JupyterLab preview extension is automatically installed after `pip install voila` starting with JupyterLab 3.0.

## Create A Minimal Notebook App

Create a notebook such as `hello.ipynb` with a widget-driven cell:

```python
import ipywidgets as widgets
from IPython.display import display

name = widgets.Text(description="Name", value="Voila")
out = widgets.Output()

def render(change=None):
    with out:
        out.clear_output()
        print(f"Hello, {name.value}!")

name.observe(render, names="value")
display(name, out)
render()
```

Run it locally:

```bash
voila hello.ipynb
```

By default, Voilà serves the app on `http://localhost:8866` and hides code cells in the rendered output.

## Core Usage Patterns

### Serve one notebook directly

```bash
voila hello.ipynb --no-browser
```

Use `--no-browser` for remote machines, containers, CI smoke tests, and production launch scripts.

### Serve a notebook directory

If you run `voila` with no notebook argument, it serves the current directory as a notebook picker:

```bash
cd notebooks
voila
```

Each notebook launch gets its own kernel, which is important for memory planning and per-user state isolation.

### Use Voilà from JupyterLab or Jupyter Server

Start your server normally and open the `/voila` route:

```bash
jupyter lab
# then browse to http://localhost:8888/voila
```

If the endpoint is missing in an older or unusual environment, the upstream README still documents enabling the extension explicitly:

```bash
jupyter serverextension enable voila
jupyter server extension enable voila
```

### Show notebook source intentionally

By default, Voilà strips notebook inputs from the rendered page. If you actually want readers to see source code, set `strip_sources=False` in your config instead of assuming notebook cells will appear automatically.

### Programmatic startup exists, but prefer the CLI first

The docs show an embedding path through `voila.app.Voila` and `voila.config.VoilaConfiguration`, mainly for advanced hook-based customization. For most agent-generated work, the CLI is simpler and less error-prone than constructing the application object manually.

## Configuration, Hooks, And Auth

Voilà configuration is spread across CLI flags, `voila.py`, `voila.json`, and Jupyter Server startup flags.

### Pass config through the CLI

Examples:

```bash
voila hello.ipynb --theme=dark --template=gridstack
voila hello.ipynb --VoilaConfiguration.file_allowlist="['.*\\.(png|jpg|svg)']"
voila --MappingKernelManager.cull_interval=60 --MappingKernelManager.cull_idle_timeout=120 hello.ipynb
```

### Configure Voilà inside a Jupyter Server launch

When serving through JupyterLab or Notebook, pass Voilà config through the Jupyter command:

```bash
jupyter lab --VoilaConfiguration.template=distill
```

### `voila.py` for Python hook functions

The official docs place `voila.py` in the directory where you start Voilà. Use it when you need Python hooks such as:

- `prelaunch_hook` to inspect the Tornado request or mutate the notebook before execution
- `page_config_hook` to adjust frontend `page_config`, static URLs, or extension config

Use hooks sparingly. They are powerful, but they make notebook rendering behavior harder to reason about.

### `voila.json` for structured config

The docs use `voila.json` in the launch directory for kernel-pool settings and other nontrivial config, especially:

- preheated kernel pools
- kernel environment variables
- notebook-level preheat deny lists

### Token authentication

Voilà supports token auth when using `jupyter-server` 2, but upstream says it is disabled by default.

Use an explicit token for anything beyond local testing:

```bash
voila hello.ipynb --token=my-secret-token
```

Do not assume that a reverse proxy alone gives you acceptable access control.

### Static files are restricted by default

Unlike JupyterLab, Voilà does not serve every file next to the notebook. It only serves files that match the allowlist and do not match the denylist. This is a common source of broken images, CSS, or downloadable assets when moving from notebook development to deployment.

## Performance And Deployment

### Cull idle kernels

Voilà starts a new kernel for each rendered notebook. If multiple users or repeated refreshes matter, configure culling:

```bash
voila hello.ipynb \
  --MappingKernelManager.cull_interval=60 \
  --MappingKernelManager.cull_idle_timeout=120
```

This is one of the simplest ways to prevent memory growth in multi-user or demo deployments.

### Preheated kernels

For heavy notebooks with slow first render times:

```bash
voila hello.ipynb --preheat_kernel=True --pool_size=5
```

Important limitations:

- `preheated kernels` are incompatible with `prelaunch_hook`
- cached pre-rendered HTML only works when request conditions still match the pre-rendered theme/template expectations
- if you use request-derived data, you may need `voila.utils.wait_for_request()` in the notebook

### Public hosting patterns

The official deploy docs cover Binder, Railway, Google App Engine flexible, private servers behind nginx, Apache reverse proxy, ngrok, and other hosted paths.

Practical deployment rules:

- include every notebook dependency in `requirements.txt` or `environment.yml`
- public deployments need websocket support
- for container or service launches, use `--no-browser`
- for hosted services, bind to `0.0.0.0` when the platform expects it, for example `voila --port=$PORT --Voila.ip=0.0.0.0 --no-browser`
- if serving under a subpath or reverse proxy, configure the base URL correctly and forward websocket traffic

## Common Pitfalls

- Installing `voila` without installing the notebook's widget and data dependencies.
- Running Voilà from a different environment than the Jupyter kernel, then assuming missing widgets are a Voilà bug.
- Starting from the stale `en/stable` docs pages and copying old template or frontend guidance.
- Forgetting that code cells are hidden by default and then debugging the wrong thing.
- Expecting arbitrary local files to be served without adjusting `file_allowlist`, `file_denylist`, or `static_root`.
- Using `prelaunch_hook` and `preheat_kernel` together. Upstream documents them as incompatible.
- Assuming tree-page templates still apply to the default tree view. Since `0.5.0`, the default tree page is JupyterLab-based; old tree templates require `--classic-tree`.
- Treating public sharing as safe because execute requests are blocked. The app can still expose notebook data and business logic.
- Deploying to environments without websocket support. The official Google App Engine docs explicitly require the flexible environment for this reason.
- Using plain `.py` scripts as if they were notebooks without configuring `extension_language_mapping` or using Jupytext.

## Official Source URLs

- `https://voila.readthedocs.io/en/latest/`
- `https://voila.readthedocs.io/en/latest/install.html`
- `https://voila.readthedocs.io/en/latest/using.html`
- `https://voila.readthedocs.io/en/latest/customize.html`
- `https://voila.readthedocs.io/en/latest/deploy.html`
- `https://voila.readthedocs.io/en/latest/changelog.html`
- `https://pypi.org/project/voila/`
- `https://github.com/voila-dashboards/voila`
- `https://github.com/voila-dashboards/voila/releases`

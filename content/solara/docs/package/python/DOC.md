---
name: package
description: "Solara Python package guide for building reactive web apps and Jupyter UIs with pure Python"
metadata:
  languages: "python"
  versions: "1.57.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "solara,python,web,ui,jupyter,ipywidgets,reactive"
---

# Solara Python Package Guide

## Golden Rule

Use `solara` when you want one Python codebase that can run as a Jupyter UI and as a standalone web app. Keep the app entry point centered on a `Page` component, use reactive state instead of imperative DOM-style updates, and run production deployments with `solara run ... --production` or the Starlette/Flask integration paths from the official deployment docs.

As of March 12, 2026, the version used here `1.57.3` does not appear on the official registry. PyPI and the official changelog both show `1.57.2` as the latest published release, so this guide is pinned to `1.57.2`.

## Install

Use a virtual environment unless you already have an isolated Python environment:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "solara==1.57.2"
```

Common alternatives:

```bash
uv add "solara==1.57.2"
poetry add "solara==1.57.2"
```

Important install details from the official docs:

- `solara` is a meta package that pins compatible `solara-ui` and `solara-server` versions.
- If you only need Solara inside Jupyter, `solara-ui` can be enough.
- If you only need server deployment control, install `solara-server[...]` directly.
- For air-gapped or firewalled environments, use `solara[assets]` so static assets do not need to be fetched from a CDN at runtime.

Examples:

```bash
python -m pip install "solara-ui[all]"
python -m pip install "solara-server[starlette]"
python -m pip install "solara[assets]==1.57.2"
```

## Initialize A Solara App

The usual entry point is a Python file with a `Page` component:

```python
import solara

sentence = solara.reactive("Solara makes Python UIs simpler.")
word_limit = solara.reactive(8)

@solara.component
def Page():
    word_count = len(sentence.value.split())

    solara.SliderInt("Word limit", value=word_limit, min=2, max=20)
    solara.InputText("Sentence", value=sentence, continuous_update=True)

    if word_count > word_limit.value:
        solara.Error(f"{word_count} words is over the limit of {word_limit.value}.")
    else:
        solara.Success("Within the limit.")
```

Run it locally:

```bash
solara run myapp.py
```

For production mode:

```bash
solara run myapp.py --production
```

Solara can generate a starter file for you:

```bash
solara create button
```

## Reuse The Same UI In Jupyter

If the app file exposes `Page`, you can render it inside a notebook:

```python
from myapp import Page

Page()
```

If notebook rendering fails, check whether the notebook server and the kernel are using different Python environments. The official troubleshooting docs call this the most common Jupyter issue, and `solara.check_jupyter()` can help detect it.

## Core Usage Patterns

### Use reactive state intentionally

Use `solara.reactive(...)` for app-level or shared state and `solara.use_reactive(...)` or `solara.use_state(...)` inside components.

`use_reactive` is usually the most ergonomic hook when you want a reactive object:

```python
import solara

@solara.component
def Counter():
    count = solara.use_reactive(0)

    def increment():
        count.value += 1

    solara.Button("Increment", on_click=increment)
    solara.Text(f"Count: {count.value}")
```

Use `use_state` when you explicitly want a `(value, setter)` tuple:

```python
import solara

@solara.component
def Counter():
    count, set_count = solara.use_state(0)
    solara.Button("Increment", on_click=lambda: set_count(lambda prev: prev + 1))
    solara.Text(f"Count: {count}")
```

### Avoid blocking the UI thread

For medium or long running work, use `solara.use_thread(...)` so the render loop stays responsive:

```python
import solara
import time

@solara.component
def Page():
    result = solara.use_thread(lambda: (time.sleep(2), "done")[1], dependencies=[])

    if result.state.name in {"STARTING", "WAITING", "RUNNING"}:
        solara.Info("Working...")
    elif result.error:
        solara.Error(str(result.error))
    else:
        solara.Success(result.value)
```

### Routing and multipage apps

For a simple app, exporting a single `Page` is enough. For multipage apps:

- define a `routes = [...]` tree with `solara.Route(...)`, or
- point Solara at a directory or package and let it generate routes automatically

Example manual route setup:

```python
import solara

@solara.component
def Home():
    solara.Markdown("Home")

@solara.component
def About():
    solara.Markdown("About")

routes = [
    solara.Route(path="/", component=Home, label="Home"),
    solara.Route(path="about", component=About, label="About"),
]
```

When navigating, use `solara.Link(...)`, `solara.resolve_path(...)`, or `solara.use_router().push(...)`. Do not build navigation from `route.path` alone because route paths are relative.

## Deployment, Config, And Auth

### Default deployment path

The most direct deployment path is:

```bash
solara run myapp.py --production
```

Under the hood this uses Starlette. For framework-managed hosting, set `SOLARA_APP` and run your server normally:

```bash
export SOLARA_APP=myapp.py
uvicorn solara.server.starlette:app
```

Or:

```bash
export SOLARA_APP=myapp.py
gunicorn solara.server.flask:app
```

Solara can also be mounted inside existing Flask, Starlette, or FastAPI apps.

### Reverse proxies and subpaths

If Solara is behind Nginx, Traefik, Caddy, or another proxy:

- forward `Host` and `X-Forwarded-Proto`
- if the app is mounted under a subpath, use `X-Script-Name` or uvicorn `--root-path`
- configure `FORWARDED_ALLOW_IPS` so uvicorn trusts the proxy headers

These settings matter for URL generation, cookies, and OAuth redirects.

### Auth

Open-source Solara itself is a UI framework, not a general auth system. Official OAuth support is documented under Solara Enterprise and currently centers on `solara-enterprise[auth]`.

Typical enterprise auth setup uses environment variables such as:

```bash
SOLARA_SESSION_SECRET_KEY=change-me
SOLARA_OAUTH_CLIENT_ID=...
SOLARA_OAUTH_CLIENT_SECRET=...
SOLARA_OAUTH_API_BASE_URL=...
SOLARA_OAUTH_PRIVATE=True
```

If redirects come back to the wrong URL, the official docs point to `SOLARA_BASE_URL`, `SOLARA_ROOT_PATH`, forwarded `Host`, and forwarded `X-Forwarded-Proto` as the first things to verify.

## Testing And Debugging

The official testing guidance is to prefer tests without a browser when possible. Use plain `pytest` for application logic and only bring in browser testing when you are validating new frontend behavior, CSS, or custom components.

Relevant packages:

```bash
python -m pip install pytest
python -m pip install "pytest-ipywidgets"
```

Debugging options from the official docs:

- `breakpoint()` works inside Solara callbacks
- PyCharm and VS Code debugger launch configs are supported
- development mode gives hot reloading; production mode disables file watching

## Common Pitfalls

### Do not mutate reactive values in place

Lists, dicts, and custom objects can be mutated without Solara noticing. Reassign a new value instead of mutating in place.

Prefer:

```python
items.value = [*items.value, "new"]
```

Avoid:

```python
items.value.append("new")
```

### Keep hooks at the top level

Hooks such as `use_state`, `use_reactive`, and `use_thread` must not be called inside loops, conditions, or nested functions. Solara warns today, and the docs say Solara 2.0 will turn these cases into errors.

### Be careful with `use_reactive` and objects without equality

The official `use_reactive` docs warn that objects without usable equality semantics can trigger render loops. If you need a stable object instance, combine `use_reactive(...)` with `use_memo(...)`.

### Notebook server and kernel environments can differ

If Solara works in one environment but not another, confirm the Python executable used by the notebook server and the kernel. This mismatch is a frequent cause of missing widget assets in Jupyter environments.

### Multi-worker deployments need planning

## Version-Sensitive Notes

- `1.57.2` includes a security fix for a path-boundary traversal issue and a memory leak fix.
- `1.56.0` dropped Python 3.7 support, so do not assume older runtime compatibility if you are upgrading from older Solara deployments.
- The roadmap says Solara 2.0 is expected to tighten state-mutation and hook-misuse enforcement. Keep current code free of those warnings now to avoid avoidable upgrade churn later.

## Official Sources

- Docs root: https://solara.dev/documentation
- Quickstart: https://solara.dev/documentation/getting_started
- Installation: https://solara.dev/docs/installing
- State management: https://solara.dev/documentation/getting_started/fundamentals/state-management
- Hooks: https://solara.dev/documentation/api/hooks/use_reactive and https://solara.dev/documentation/api/hooks/use_state
- Routing: https://solara.dev/documentation/advanced/understanding/routing
- Deployment: https://solara.dev/documentation/getting_started/deploying/self-hosted
- Testing: https://solara.dev/documentation/advanced/howto/testing
- Debugging: https://solara.dev/documentation/advanced/howto/debugging
- OAuth: https://solara.dev/documentation/advanced/enterprise/oauth
- Changelog: https://solara.dev/changelog/
- Registry: https://pypi.org/project/solara/

---
name: package
description: "NiceGUI package guide for building browser-based and native Python UIs with NiceGUI 3.8.0"
metadata:
  languages: "python"
  versions: "3.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "nicegui,python,ui,web,fastapi,desktop"
---

# NiceGUI Python Package Guide

## Golden Rule

Use `nicegui` when you want a Python-first UI that runs in the browser or in NiceGUI's native window mode. Pick one app structure per entry point:

- a simple NiceGUI script with UI elements in module scope
- a single root function passed to `ui.run(...)`
- explicit `@ui.page(...)` route functions

Since NiceGUI `3.0.0`, those patterns are no longer interchangeable in one file. If you mix script mode with `@ui.page`, routing and reload behavior become the first thing to debug.

## Install

Pin the package version your project expects:

```bash
python -m pip install "nicegui==3.8.0"
```

Common alternatives:

```bash
uv add "nicegui==3.8.0"
poetry add "nicegui==3.8.0"
```

Useful extras from the published package metadata:

```bash
python -m pip install "nicegui[native]==3.8.0"
python -m pip install "nicegui[redis]==3.8.0"
```

Use `native` when you want a desktop-style window via `pywebview`. Use `redis` only when you actually need Redis-backed persistence or coordination.

## Quick Start

The upstream package README still shows the core happy path:

```python
from nicegui import ui

ui.label('Hello NiceGUI!')
ui.button('BUTTON', on_click=lambda: ui.notify('button was pressed'))

ui.run()
```

Run it:

```bash
python main.py
```

By default NiceGUI serves the app on `http://localhost:8080/` and reloads the page when the source changes.

## Choose An App Structure

### Single-page script

For a very small app, keep the UI in module scope and finish with `ui.run()`:

```python
from nicegui import ui

ui.markdown('## Build a small internal tool')
ui.button('Ping', on_click=lambda: ui.notify('pong'))

ui.run(host='0.0.0.0', port=8080)
```

Use this when the app is effectively one page and you do not need `@ui.page`.

### Root function for a larger single-page app

NiceGUI `3.x` added a `root` page parameter for `ui.run(...)`. This is the cleanest way to keep one routed entry point without relying on global script mode:

```python
from nicegui import ui

def root() -> None:
    ui.label('Dashboard')
    ui.button('Refresh', on_click=lambda: ui.notify('refresh requested'))

ui.run(root, host='0.0.0.0', port=8080)
```

Use this when you want one main page with reusable functions and clearer structure than a module-level script.

### Routed pages

Use `@ui.page` when the app has multiple pages or login/protected routes:

```python
from nicegui import ui

@ui.page('/')
def index() -> None:
    ui.label('Home')
    ui.link('Settings', '/settings')

@ui.page('/settings')
def settings() -> None:
    ui.label('Settings page')

ui.run()
```

Do not mix this with module-scope script UI in the same file unless you have verified the behavior against current `3.x` docs and examples.

## Attach NiceGUI To FastAPI

NiceGUI is built on FastAPI/Starlette, and the supported integration path is `ui.run_with(app)`:

```python
from fastapi import FastAPI
from nicegui import ui

fastapi_app = FastAPI()

@fastapi_app.get('/healthz')
def healthz() -> dict[str, bool]:
    return {'ok': True}

@ui.page('/')
def index() -> None:
    ui.label('NiceGUI mounted into an existing FastAPI app')

ui.run_with(fastapi_app, storage_secret='replace-me')
```

Practical notes:

- Register plain FastAPI routes before `ui.run_with(...)`.
- If you depend on session-backed NiceGUI storage, use a real `storage_secret`.
- If your FastAPI app already has `SessionMiddleware`, keep the session secret aligned with the NiceGUI storage secret. There is an open upstream bug report showing storage persistence breaks when they differ.

## Authentication And Per-user Storage

The upstream authentication example uses `app.storage.user` plus middleware that redirects unauthenticated requests away from protected pages.

Minimal pattern:

```python
from fastapi import Request
from fastapi.responses import RedirectResponse
from starlette.middleware.base import BaseHTTPMiddleware

from nicegui import app, ui

unrestricted_page_routes = {'/login'}

@app.add_middleware
class AuthMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next):
        if not app.storage.user.get('authenticated', False):
            if not request.url.path.startswith('/_nicegui') and request.url.path not in unrestricted_page_routes:
                return RedirectResponse(f'/login?redirect_to={request.url.path}')
        return await call_next(request)

@ui.page('/login')
def login(redirect_to: str = '/') -> None:
    def do_login() -> None:
        app.storage.user.update({'authenticated': True, 'username': username.value})
        ui.navigate.to(redirect_to)

    username = ui.input('Username')
    ui.button('Log in', on_click=do_login)

@ui.page('/')
def index() -> None:
    ui.label(f'Hello {app.storage.user.get("username", "user")}')

if __name__ in {'__main__', '__mp_main__'}:
    ui.run(storage_secret='replace-this-secret')
```

Important details agents often miss:

- `app.storage.user` is session-backed per-user storage, so it needs `storage_secret` to persist safely.
- Exclude `/_nicegui` from auth redirects or the frontend runtime breaks.
- Replace the demo login with a real auth provider for production. The upstream example explicitly says to use OAuth2/Authlib or another real authentication system instead of plain passwords in code.

## Native Mode

NiceGUI can run in a desktop-style native window instead of only in the browser:

```python
from nicegui import ui

ui.label('Desktop window')
ui.run(native=True, reload=False)
```

Use the `native` extra first:

```bash
python -m pip install "nicegui[native]==3.8.0"
```

Practical caveats:

- Treat native mode as `pywebview` integration, not as a separate GUI toolkit.
- Keep `reload=False` in native mode unless you have validated a different setup.
- Platform-specific `pywebview` renderer issues still show up in upstream issue reports, especially on Windows and Linux desktop environments.

## Deployment And Runtime Notes

- NiceGUI uses WebSockets after the initial page load, so proxy and ingress config must forward WebSocket traffic correctly.
- The project README describes NiceGUI as a single-worker async setup. Do not assume a multi-worker deployment is safe without validating session, storage, and websocket behavior.
- For containers or remote development, bind explicitly:

```python
ui.run(host='0.0.0.0', port=8080)
```

- When you need custom API endpoints, mount files, or extra ASGI behavior, prefer attaching NiceGUI to a FastAPI app instead of fighting the default script entry point.

## Common Pitfalls

- Mixing script mode and `@ui.page` after the `3.0.0` routing changes.
- Forgetting `storage_secret` and then wondering why `app.storage.user` data disappears after refresh.
- Protecting all routes with middleware and accidentally redirecting requests under `/_nicegui`.
- Calling `ui.run_with(app)` before defining the FastAPI routes you still need.
- Assuming browser examples will work unchanged in native mode.
- Copying old `2.x` snippets that rely on the old shared auto-index page semantics.

## Version-Sensitive Notes

### `3.8.0`

- `3.8.0` is the current PyPI release as of `2026-03-12`.
- The `3.8.0` release hardens `run_method()` and `run_*_method()` against XSS by no longer accepting arbitrary JavaScript expressions as method names. If you find older examples that pass a JS lambda string into those helpers, rewrite them to use `run_javascript(...)` instead.
- The same release added a security best-practices section to the official docs. Prefer that guidance over blog posts for HTML/Markdown sanitization and client-side method execution patterns.

### `3.1.x`

- `3.1.0` added session cookie attribute configuration from `ui.run()`. If you need stricter cookie settings, check the `3.1+` API instead of older examples.

### `3.0.x`

- `3.0.0` introduced NiceGUI script mode and the `root` parameter for `ui.run(...)`.
- It also removed the old shared auto-index client behavior. Global-scope UI now behaves as a script page that is recreated on each visit.
- Old snippets that assume global UI state is shared across clients are no longer reliable in `3.x`.

## Official Sources

- Documentation: `https://nicegui.io/documentation`
- Package registry: `https://pypi.org/project/nicegui/`
- Repository: `https://github.com/zauberzeug/nicegui`
- Releases: `https://github.com/zauberzeug/nicegui/releases`
- Authentication example: `https://github.com/zauberzeug/nicegui/blob/main/examples/authentication/main.py`

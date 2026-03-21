---
name: debug-toolbar
description: "django-debug-toolbar package guide for Django request inspection, SQL debugging, and development-only diagnostics"
metadata:
  languages: "python"
  versions: "6.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,debugging,toolbar,sql,profiling,development"
---

# django-debug-toolbar Python Package Guide

## What It Is

`django-debug-toolbar` adds an in-browser toolbar to Django development pages so you can inspect SQL, templates, headers, cache calls, signals, request data, and request history.

This entry is for `django-debug-toolbar==6.2.0`, the current stable PyPI release as of March 12, 2026. Upstream states that `6.2.0` requires Python `>=3.10`, works with Django `>=4.2`, and still has only experimental async support without concurrent-request support.

## Install

Install it only in development environments:

```bash
python -m pip install django-debug-toolbar==6.2.0
```

Common alternatives:

```bash
uv add --dev django-debug-toolbar==6.2.0
poetry add --group dev django-debug-toolbar==6.2.0
```

Prerequisites the upstream install guide assumes are already present:

- `django.contrib.staticfiles` is enabled
- your `TEMPLATES` setting uses a `DjangoTemplates` backend with `APP_DIRS = True`
- you are rendering HTML responses with a closing `</body>` tag

## Minimal Setup

Add the app:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "django.contrib.staticfiles",
    "debug_toolbar",
]
```

Add the middleware as early as possible, but after middleware that encodes response content such as `GZipMiddleware`:

```python
# settings.py
MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "debug_toolbar.middleware.DebugToolbarMiddleware",
    # ...
]
```

Allow local requests:

```python
# settings.py
INTERNAL_IPS = [
    "127.0.0.1",
    "::1",
]
```

Add the URLs:

```python
# urls.py
from django.urls import path
from debug_toolbar.toolbar import debug_toolbar_urls

urlpatterns = [
    # ...
] + debug_toolbar_urls()
```

The helper above uses the default `__debug__` prefix. If you want an explicit route, this also works:

```python
from django.urls import include, path

urlpatterns = [
    path("__debug__/", include("debug_toolbar.urls")),
    # ...
]
```

## Recommended Development-Only Pattern

Keep the toolbar out of production and test settings.

```python
# settings/dev.py
import os
import sys

from .base import *  # noqa: F403

TESTING = "test" in sys.argv or "PYTEST_VERSION" in os.environ

if not TESTING:
    INSTALLED_APPS += ["debug_toolbar"]  # noqa: F405
    MIDDLEWARE = [  # noqa: F405
        "debug_toolbar.middleware.DebugToolbarMiddleware",
        *MIDDLEWARE,
    ]
    INTERNAL_IPS = ["127.0.0.1", "::1"]
```

```python
# urls.py
from django.conf import settings

urlpatterns = [
    # ...
]

if not getattr(settings, "TESTING", False):
    from debug_toolbar.toolbar import debug_toolbar_urls

    urlpatterns += debug_toolbar_urls()
```

If your project has a nonstandard test command, set `DEBUG_TOOLBAR_CONFIG["IS_RUNNING_TESTS"]` explicitly.

## Core Usage

Open a normal HTML page in development and use the toolbar to inspect the current request.

Panels most useful in day-to-day work:

- `SQL`: query count, duplicate queries, timings, and `EXPLAIN`
- `Templates`: rendered templates and context
- `Headers`: request/response headers
- `Request`: GET, POST, cookies, and session data
- `Cache`: cache hits and misses
- `History`: switch back to prior requests
- `Signals`: fired signals and receivers

Typical workflow:

1. Load the slow or incorrect page.
2. Open `SQL` first for N+1 queries or bad joins.
3. Open `Templates` when the wrong context or fragment renders.
4. Open `Headers` and `Request` for cookies, CSRF, auth, or proxy/header issues.
5. Use `History` to compare a redirect target or the last few requests.

The package also ships a useful management command:

```bash
python manage.py debugsqlshell
```

Use `debugsqlshell` when you want ORM queries printed interactively while experimenting with queryset changes.

## Visibility and Access Control

There is no auth layer inside the toolbar. Access is controlled by Django settings, mainly `DEBUG`, `INTERNAL_IPS`, and `SHOW_TOOLBAR_CALLBACK`.

For most projects, a callback is the safest place to centralize visibility rules:

```python
# project/debug_toolbar.py
from django.conf import settings

def show_toolbar(request):
    return (
        settings.DEBUG
        and request.user.is_staff
        and request.META.get("REMOTE_ADDR") in {"127.0.0.1", "::1"}
    )
```

```python
# settings.py
DEBUG_TOOLBAR_CONFIG = {
    "SHOW_TOOLBAR_CALLBACK": "project.debug_toolbar.show_toolbar",
}
```

Do not use a lambda that closes over your module-level `DEBUG` variable. Upstream explicitly warns that this behaves badly under tests because Django forces `settings.DEBUG = False` while your module variable may stay `True`.

The upstream configuration docs also warn that the toolbar is not hardened for production use. If `SECRET_KEY` leaks, the SQL panel can become especially risky.

## Docker, WSL, and Remote Dev

The most common reason the toolbar does not appear in containers or remote dev is that the incoming IP is not in `INTERNAL_IPS`.

For `6.0+`, upstream added a Docker helper callback:

```python
DEBUG_TOOLBAR_CONFIG = {
    "SHOW_TOOLBAR_CALLBACK": "debug_toolbar.middleware.show_toolbar_with_docker",
}
```

Use that when developing inside Docker and the gateway IP changes often. If that still does not fit your setup, write your own callback and inspect `request.META["REMOTE_ADDR"]`.

## Useful Configuration

Keep configuration minimal. The upstream docs explicitly say not to copy the entire default config into your settings.

### Reduce noise or cost

```python
DEBUG_TOOLBAR_CONFIG = {
    "RESULTS_CACHE_SIZE": 10,
}
```

Lower `RESULTS_CACHE_SIZE` if you want to reduce memory use during development.

### HTMX or Turbo

If you use boosted page updates, preserve the toolbar root element and optionally update the toolbar after fetches:

```python
DEBUG_TOOLBAR_CONFIG = {
    "ROOT_TAG_EXTRA_ATTRS": "hx-preserve",
    "UPDATE_ON_FETCH": True,
}
```

For Hotwire Turbo, switch the root tag attribute:

```python
DEBUG_TOOLBAR_CONFIG = {
    "ROOT_TAG_EXTRA_ATTRS": "data-turbo-permanent",
    "UPDATE_ON_FETCH": True,
}
```

`UPDATE_ON_FETCH` defaults to `False`, so AJAX or boosted navigation can otherwise leave the toolbar showing stale request data.

### Store request data in the database

`6.0+` added `TOOLBAR_STORE_CLASS`:

```python
DEBUG_TOOLBAR_CONFIG = {
    "TOOLBAR_STORE_CLASS": "debug_toolbar.store.DatabaseStore",
}
```

If you use `DatabaseStore`, run the app migrations:

```bash
python manage.py migrate debug_toolbar
```

Use `DatabaseStore` only when you actually need persistence across request rendering boundaries. The default `MemoryStore` is simpler for normal local development.

### Disable expensive panels

```python
DEBUG_TOOLBAR_CONFIG = {
    "DISABLE_PANELS": {
        "debug_toolbar.panels.profiling.ProfilingPanel",
    },
}
```

You can also trim `DEBUG_TOOLBAR_PANELS` entirely if some panels are irrelevant to your project.

## Profiling Notes

`ProfilingPanel` is included but disabled by default. On Python `3.12+`, upstream says you must run the development server with `--nothreading` for profiling to work reliably:

```bash
python -m manage runserver --nothreading
```

The profiling panel and parts of async support are still limited around concurrent requests. Do not treat it as reliable under high-concurrency or Channels-based development flows.

## Async and HTML Limitations

Upstream now supports Django async views and ASGI environments experimentally, but still documents important limits:

- concurrent requests are not supported
- `TimerPanel`, `RequestPanel`, and `ProfilingPanel` do not work in async contexts
- Django Channels is not supported

The toolbar only injects itself into HTML responses with MIME type `text/html` or `application/xhtml+xml` that include a closing `</body>` tag. Do not expect it on JSON APIs.

## Common Pitfalls

### The toolbar never appears

Check these first:

- `DEBUG` is `True`
- the response is HTML and contains `</body>`
- `debug_toolbar` is in `INSTALLED_APPS`
- `DebugToolbarMiddleware` is in `MIDDLEWARE`
- the request IP is allowed by `INTERNAL_IPS` or `SHOW_TOOLBAR_CALLBACK`
- toolbar URLs are included

### Middleware order is wrong

Put the middleware early. If another middleware returns a response before the toolbar runs, the toolbar will not be injected.

### Static files are broken

If `toolbar.js` fails to load, check the browser console. Upstream calls out two recurring causes:

- incorrect MIME types for `.js` files on some platforms
- cross-origin static file hosting without the right CORS headers

### History or stored panels do not behave as expected

`HistoryPanel` is disabled when `RENDER_PANELS = True` or when the server runs with multiple processes. That behavior is upstream, not a misconfiguration in your project.

### Tests break after adding the toolbar

Do not leave the toolbar enabled during tests. Gate it behind `TESTING` or configure `IS_RUNNING_TESTS`.

## Version-Sensitive Notes for 6.2.0

- `6.2.0` deprecates `RedirectsPanel` in favor of `HistoryPanel` for redirected requests.
- `6.2.0` drops Python `3.9`; use Python `3.10+`.
- `6.1.0` added `CommunityPanel` to the default built-in panel set.
- `6.0.0` added `show_toolbar_with_docker` and `TOOLBAR_STORE_CLASS`.
- `UPDATE_ON_FETCH` exists in current releases but still defaults to `False`.
- If you are upgrading from `5.0.x`, re-check async, Docker, and request-store behavior instead of copying an old settings snippet unchanged.

## Official Links

- Documentation root: https://django-debug-toolbar.readthedocs.io/en/latest/
- Installation: https://django-debug-toolbar.readthedocs.io/en/latest/installation.html
- Configuration: https://django-debug-toolbar.readthedocs.io/en/latest/configuration.html
- Tips: https://django-debug-toolbar.readthedocs.io/en/latest/tips.html
- Panels: https://django-debug-toolbar.readthedocs.io/en/latest/panels.html
- Commands: https://django-debug-toolbar.readthedocs.io/en/latest/commands.html
- Change log: https://django-debug-toolbar.readthedocs.io/en/latest/changes.html
- PyPI: https://pypi.org/project/django-debug-toolbar/

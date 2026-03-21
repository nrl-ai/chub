---
name: package
description: "WhiteNoise static-file serving for Django and WSGI apps with practical setup, compression, caching, and deployment notes"
metadata:
  languages: "python"
  versions: "6.12.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "whitenoise,django,wsgi,staticfiles,compression,caching,cdn"
---

# WhiteNoise Python Package Guide

## What It Is

`whitenoise` serves static files directly from Python web apps.

The main supported patterns are:

- Django static file serving through `whitenoise.middleware.WhiteNoiseMiddleware`
- WSGI wrapping with `from whitenoise import WhiteNoise`
- Precompressed static assets (`gzip`, optional `brotli`)
- Cache-friendly immutable asset URLs for CDN deployment

Use it for application static assets. Do not use it for user-uploaded media files.

## Version Covered

- Package: `whitenoise`
- Ecosystem: `pypi`
- Version: `6.12.0`
- Python requirement on PyPI: `>=3.10`
- Django classifiers on PyPI: `4.2`, `5.0`, `5.1`, `5.2`, `6.0`
- Registry: https://pypi.org/project/whitenoise/
- Docs root used for this guide: https://whitenoise.readthedocs.io/en/stable/

## Install

Pin the package version explicitly:

```bash
python -m pip install "whitenoise==6.12.0"
```

If you want Brotli-compressed assets:

```bash
python -m pip install "whitenoise[brotli]==6.12.0"
```

## Django Setup

WhiteNoise's primary integration path is Django.

Add it directly after `SecurityMiddleware`, configure `STATIC_ROOT`, and use the WhiteNoise storage backend:

```python
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent

INSTALLED_APPS = [
    "whitenoise.runserver_nostatic",
    "django.contrib.staticfiles",
    # ...
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "whitenoise.middleware.WhiteNoiseMiddleware",
    # ...
]

STATIC_URL = "/static/"
STATIC_ROOT = BASE_DIR / "staticfiles"

STORAGES = {
    "staticfiles": {
        "BACKEND": "whitenoise.storage.CompressedManifestStaticFilesStorage",
    },
}
```

Then build the static tree:

```bash
python manage.py collectstatic
```

Use `runserver --nostatic` in development when you want Django to route static requests through WhiteNoise instead of Django's built-in static serving:

```bash
python manage.py runserver --nostatic
```

### Storage Backend Choice

Use this in most production Django apps:

```python
"whitenoise.storage.CompressedManifestStaticFilesStorage"
```

This gives you:

- hashed asset filenames for long-lived caching
- gzip output
- Brotli output when the `brotli` extra is installed

Use this only if you want compression without manifest hashing:

```python
"whitenoise.storage.CompressedStaticFilesStorage"
```

### Common Django Settings

```python
WHITENOISE_AUTOREFRESH = DEBUG
WHITENOISE_USE_FINDERS = DEBUG
WHITENOISE_KEEP_ONLY_HASHED_FILES = True
WHITENOISE_MANIFEST_STRICT = True
WHITENOISE_ROOT = BASE_DIR / "public"
```

What matters:

- `WHITENOISE_AUTOREFRESH`: rechecks files on each request; development only
- `WHITENOISE_USE_FINDERS`: serves from Django finders instead of only `STATIC_ROOT`; useful in development
- `WHITENOISE_KEEP_ONLY_HASHED_FILES`: keeps only hashed copies to shrink deployment artifacts
- `WHITENOISE_MANIFEST_STRICT`: raises when referenced manifest entries are missing
- `WHITENOISE_ROOT`: serves files at site root such as `robots.txt` or `favicon.ico`

### CDN Pattern

The documented pattern is to prepend an optional static host:

```python
import os

STATIC_HOST = os.environ.get("DJANGO_STATIC_HOST", "")
STATIC_URL = STATIC_HOST + "/static/"
```

Operational notes:

- WhiteNoise automatically sends cache headers for immutable versioned files
- `WHITENOISE_ALLOW_ALL_ORIGINS` defaults to `True`, which is useful for fonts and other static assets behind a CDN
- Browsers only request Brotli over HTTPS
- Your CDN must preserve and cache on `Accept-Encoding` if you want precompressed assets to work correctly

## WSGI And Flask Setup

For non-Django WSGI apps, wrap the WSGI application:

```python
from whitenoise import WhiteNoise

from myapp import application

application = WhiteNoise(application, root="/absolute/path/to/static")
```

Add more static directories if needed:

```python
application.add_files("/absolute/path/to/more-static", prefix="assets/")
```

Important behavior:

- WhiteNoise scans static directories at startup
- matching requests are served directly
- non-static requests are passed through to the wrapped WSGI app

For Flask:

```python
from flask import Flask
from whitenoise import WhiteNoise

app = Flask(__name__)
app.wsgi_app = WhiteNoise(app.wsgi_app, root="static/")
```

If you need multiple static roots:

```python
from flask import Flask
from whitenoise import WhiteNoise

app = Flask(__name__)
app.wsgi_app = WhiteNoise(app.wsgi_app)

for path in ("static/one/", "static/two/"):
    app.wsgi_app.add_files(path)
```

## Compression And Customization

Outside Django's storage backend, you can precompress files yourself:

```bash
python -m whitenoise.compress static/
```

That generates `.gz` files and, when Brotli support is installed, `.br` files when compression reduces size.

Useful constructor-level customization:

```python
import re
from whitenoise import WhiteNoise

def add_headers(headers, path, url):
    if path.endswith(".pdf"):
        headers["Content-Disposition"] = "attachment"

def immutable_file_test(path, url):
    return re.match(r"^.+\\.[0-9a-f]{12}\\..+$", url) is not None

application = WhiteNoise(
    application,
    root="/absolute/path/to/static",
    add_headers_function=add_headers,
    immutable_file_test=immutable_file_test,
)
```

Useful knobs for WSGI usage:

- `prefix`: URL prefix for mounted static files
- `index_file`: serve directory indexes such as `index.html`
- `max_age`: cache lifetime for non-immutable files
- `autorefresh`: development mode file reloading

## Config And Auth Notes

WhiteNoise does not have an authentication or credential model. Configuration is entirely local:

- Django: settings and storage backend selection
- WSGI/Flask: `WhiteNoise(...)` constructor options and `add_files(...)`
- Deployment: CDN host, cache behavior, and optional Brotli support

If a static route needs access control, enforce that outside WhiteNoise. WhiteNoise itself is designed to serve public static files.

## Common Pitfalls

- Put `WhiteNoiseMiddleware` immediately after Django `SecurityMiddleware`. Middleware ordering is part of the documented setup.
- Do not skip `collectstatic` in production Django deploys. The manifest storage backend depends on collected assets.
- Do not hardcode `/static/...` paths in templates. Use Django static helpers so hashed filenames resolve correctly.
- Do not use WhiteNoise for user-uploaded media. The docs call out media files as a separate concern.
- Do not leave `WHITENOISE_AUTOREFRESH=True` in production.
- Do not expect WhiteNoise to serve files created after process startup unless you are deliberately using autorefresh development behavior.
- If Brotli appears not to work behind a CDN, check HTTPS delivery and `Accept-Encoding` caching behavior first.

## Version-Sensitive Notes For 6.12.0

- `6.12.0` drops Python `3.9` support and adds support for Python `3.14`.
- `6.12.0` adds support for Django `6.0`.
- `6.12.0` fixes a security issue in `WHITENOISE_AUTOREFRESH` mode that affected path traversal when a symlink appeared inside a served directory tree. Production deployments are not affected because autorefresh is off by default.
- The stable docs landing page still includes an older compatibility summary (`Python 3.8 to 3.14`), so for `6.12.0` version support you should trust the changelog and current PyPI metadata instead of that summary line.

## Official Sources

- Docs root: https://whitenoise.readthedocs.io/en/stable/
- Django guide: https://whitenoise.readthedocs.io/en/stable/django.html
- WSGI guide: https://whitenoise.readthedocs.io/en/stable/base.html
- Flask guide: https://whitenoise.readthedocs.io/en/stable/flask.html
- Changelog: https://whitenoise.readthedocs.io/en/stable/changelog.html
- Registry: https://pypi.org/project/whitenoise/

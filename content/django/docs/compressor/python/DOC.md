---
name: compressor
description: "django-compressor package guide for Django asset compression, offline builds, and static storage integration"
metadata:
  languages: "python"
  versions: "4.6.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,staticfiles,assets,compression,css,javascript,python,pypi"
---

# django-compressor Python Package Guide

`django-compressor` combines and filters CSS and JavaScript declared in Django templates, then emits cacheable output files or inline blocks. Use it when a Django project still serves template-owned assets and needs minification, URL rewriting, offline asset generation, or static storage/CDN integration without adding a separate frontend bundler.

This entry is pinned to `django-compressor==4.6.0`.

## Install

Use the normalized PyPI package name and pin the version that matches the project:

```bash
pip install "django-compressor==4.6.0"
```

Common alternatives:

```bash
uv add "django-compressor==4.6.0"
poetry add "django-compressor==4.6.0"
```

Optional extras from the official docs, depending on your parser and filter choices:

```bash
pip install beautifulsoup4
pip install lxml
pip install html5lib
pip install calmjs.parse
pip install csscompressor
pip install brotli
```

## Minimal Django Setup

Add the app:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "compressor",
]
```

If the project uses `django.contrib.staticfiles`, add the finder so generated files can be discovered:

```python
# settings.py
STATICFILES_FINDERS = [
    "django.contrib.staticfiles.finders.FileSystemFinder",
    "django.contrib.staticfiles.finders.AppDirectoriesFinder",
    "compressor.finders.CompressorFinder",
]
```

Align Compressor with your static file location:

```python
# settings.py
STATIC_URL = "/static/"
STATIC_ROOT = BASE_DIR / "staticfiles"

COMPRESS_URL = STATIC_URL
COMPRESS_ROOT = STATIC_ROOT
```

Defaults that matter:

- `COMPRESS_ENABLED` defaults to `not DEBUG`
- `COMPRESS_URL` defaults to `STATIC_URL`
- `COMPRESS_ROOT` defaults to `STATIC_ROOT`
- `COMPRESS_OUTPUT_DIR` defaults to `"CACHE"`
- `COMPRESS_STORAGE` defaults to `compressor.storage.CompressorFileStorage`

If you want compression to run while `DEBUG = True`, enable it explicitly:

```python
COMPRESS_ENABLED = True
```

## Core Template Usage

Wrap static assets inside `{% compress %}` blocks:

```django
{% load compress %}
{% load static %}

{% compress css %}
<link rel="stylesheet" href="{% static 'css/site.css' %}" type="text/css">
<link rel="stylesheet" href="{% static 'css/theme.css' %}" type="text/css">
{% endcompress %}
```

Typical output:

```html
<link rel="stylesheet" href="/static/CACHE/css/output.<hash>.css" type="text/css">
```

JavaScript works the same way:

```django
{% load compress %}
{% load static %}

{% compress js %}
<script src="{% static 'js/vendor.js' %}"></script>
<script src="{% static 'js/app.js' %}"></script>
{% endcompress %}
```

Use `inline` only when you deliberately want the rendered content injected into the page:

```django
{% load compress %}
{% load static %}

{% compress js inline %}
<script src="{% static 'js/app.js' %}"></script>
<script>
  window.APP_ENV = "prod";
</script>
{% endcompress %}
```

For most projects, file output is the safer default because browsers and CDNs can cache it independently.

## Recommended Base Configuration

This is the simplest production-friendly starting point for a standard Django app with local static output:

```python
# settings.py
INSTALLED_APPS += ["compressor"]

STATIC_URL = "/static/"
STATIC_ROOT = BASE_DIR / "staticfiles"

STATICFILES_FINDERS = [
    "django.contrib.staticfiles.finders.FileSystemFinder",
    "django.contrib.staticfiles.finders.AppDirectoriesFinder",
    "compressor.finders.CompressorFinder",
]

COMPRESS_URL = STATIC_URL
COMPRESS_ROOT = STATIC_ROOT
COMPRESS_ENABLED = not DEBUG
COMPRESS_OFFLINE = not DEBUG
```

For single-server development, request-time compression is fine. For real deployments, upstream still treats offline compression as the default recommendation.

## Offline Compression

Offline compression precomputes the output and manifest during deploy instead of during the first request:

```python
# settings.py
COMPRESS_OFFLINE = True
```

Build assets during deployment:

```bash
python manage.py collectstatic --noinput
python manage.py compress
```

If a `{% compress %}` block depends on template variables, provide those values before running `compress`:

```python
# settings.py
COMPRESS_OFFLINE_CONTEXT = {
    "STATIC_URL": STATIC_URL,
    "theme_name": "default",
}
```

`COMPRESS_OFFLINE_CONTEXT` can also be a list of contexts or a dotted callable path when you need multiple combinations such as locale, tenant, or theme variants.

Why offline mode is usually the right production path:

- app servers do not need to create compressed files during requests
- generated files can be built once and published with the rest of static assets
- multi-server or CDN-backed deployments avoid inconsistent first-hit behavior
- external precompilers only need to exist on the build host

At runtime, missing offline manifest entries raise `OfflineGenerationError`. That usually means the deploy skipped `python manage.py compress` or the offline context did not cover every rendered block variant.

## Static Storage And CDN Configuration

`django-compressor` does not have its own auth system. Any credentials come from the storage backend used for static files, such as S3 via `django-storages`.

Simple CDN pattern:

```python
STATIC_URL = "https://cdn.example.com/"
COMPRESS_URL = STATIC_URL
```

For Django 4.2+ projects using `STORAGES`, define explicit aliases instead of relying on fallback behavior:

```python
STORAGES = {
    "default": {
        "BACKEND": "django.core.files.storage.FileSystemStorage",
    },
    "staticfiles": {
        "BACKEND": "django.contrib.staticfiles.storage.ManifestStaticFilesStorage",
    },
    "compressor": {
        "BACKEND": "compressor.storage.CompressorFileStorage",
    },
    "compressor-offline": {
        "BACKEND": "compressor.storage.OfflineManifestFileStorage",
    },
}

COMPRESS_STORAGE_ALIAS = "compressor"
COMPRESS_OFFLINE_MANIFEST_STORAGE_ALIAS = "compressor-offline"
```

If your static assets ultimately live in remote storage, Compressor still needs a local filesystem view while building output. The official docs recommend a storage that writes locally and then mirrors to the remote backend:

```python
from django.core.files.storage import storages
from storages.backends.s3boto3 import S3Boto3Storage

class CachedS3Boto3Storage(S3Boto3Storage):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.local_storage = storages.create_storage(
            {"BACKEND": "compressor.storage.CompressorFileStorage"}
        )

    def save(self, name, content):
        self.local_storage.save(name, content)
        super().save(name, self.local_storage._open(name))
        return name
```

Then wire it up:

```python
STATIC_ROOT = BASE_DIR / "staticfiles"
COMPRESS_ROOT = STATIC_ROOT

STATICFILES_STORAGE = "project.storage.CachedS3Boto3Storage"
COMPRESS_STORAGE = STATICFILES_STORAGE
COMPRESS_OFFLINE_MANIFEST_STORAGE = STATICFILES_STORAGE

STATIC_URL = "https://cdn.example.com/"
COMPRESS_URL = STATIC_URL
```

Practical rule: if your storage backend does not provide a working local `path()` for the build process, do not point Compressor at it directly without the cached-local pattern.

## Filters, Parsers, And Precompilers

Default filters remain the main path for standard CSS and JS blocks:

```python
COMPRESS_FILTERS = {
    "css": [
        "compressor.filters.css_default.CssAbsoluteFilter",
        "compressor.filters.cssmin.rCSSMinFilter",
    ],
    "js": [
        "compressor.filters.jsmin.rJSMinFilter",
    ],
}
```

Important behavior:

- `CssAbsoluteFilter` rewrites relative `url(...)` paths because the generated CSS file moves to a different output directory
- `AutoSelectParser` is the default parser and prefers `LxmlParser` when `lxml` is installed
- malformed or unusual markup can require an explicit parser such as `BeautifulSoupParser` or `Html5LibParser`

Example parser override:

```python
COMPRESS_PARSER = "compressor.parser.BeautifulSoupParser"
```

Precompilers let Compressor transform asset types before filtering:

```python
COMPRESS_PRECOMPILERS = (
    ("text/coffeescript", "coffee --compile --stdio"),
    ("text/less", "lessc {infile} {outfile}"),
    ("text/x-scss", "sass --scss {infile} {outfile}"),
)
```

Every binary referenced there must be available anywhere you run request-time compression or `python manage.py compress`.

## Jinja2 And django-sekizai

Jinja2 support uses `compressor.contrib.jinja2ext.CompressorExtension`:

```python
import jinja2
from compressor.contrib.jinja2ext import CompressorExtension

env = jinja2.Environment(extensions=[CompressorExtension])
```

For offline compression with Jinja2 templates:

```python
COMPRESS_JINJA2_GET_ENVIRONMENT = "project.jinja2.environment"
```

Run:

```bash
python manage.py compress --engine jinja2
```

Upstream limitations worth remembering:

- no support for `{% import %}` and similar blocks inside `{% compress %}`
- no support for `{{ super() }}`
- avoid mixing Django and Jinja2 template engines in the same template locations unless you are deliberate about loader order

`django-sekizai` integration exists, but it is not compatible with offline compression. If a project depends on sekizai-style asset collection, do not assume the normal offline deployment flow will work unchanged.

## Common Pitfalls

### Compression is off in development

With `DEBUG = True`, Compressor disables itself unless `COMPRESS_ENABLED = True`.

### Generated files are missing from staticfiles

If you use `django.contrib.staticfiles`, missing `compressor.finders.CompressorFinder` is the most common cause.

### `OfflineGenerationError` appears after deploy

Usually one of these is true:

- `python manage.py compress` was never run
- template variables inside `{% compress %}` were not covered by `COMPRESS_OFFLINE_CONTEXT`
- new templates shipped without rebuilding the offline manifest

### Remote storage builds fail

Compressor still needs local filesystem access while resolving and writing files. Purely remote storage backends are the common failure mode.

### CSS asset URLs break after compression

If you replace the default CSS filter chain and remove `CssAbsoluteFilter`, relative `url(...)` references often stop resolving correctly.

### Parser errors show up on malformed templates

Use a different parser instead of forcing the default parser through bad markup.

### Precompiler commands work locally but fail in CI

Your build host needs the same `sass`, `lessc`, `coffee`, or other binaries that the config references.

### sekizai and offline compression are mixed together

That combination is unsupported upstream.

## Version-Sensitive Notes For 4.6.0

- `4.6.0` adds official Django `5.2` and `6.0` support.
- `4.6.0` drops Django `5.0` support.
- `4.6.0` adds Python `3.13` and `3.14` support.
- `4.6.0` drops Python `3.8` and `3.9` support, so plan on Python `3.10+` for this line.
- `4.5` introduced `COMPRESS_STORAGE_ALIAS` and `COMPRESS_OFFLINE_MANIFEST_STORAGE_ALIAS`; keep using them in `4.6.0` if the project uses Django `STORAGES`.
- `4.5.1` fixed remote storage behavior when the backend did not implement `path()`. `4.6.0` includes that fix, so older workarounds written specifically for `4.5.0` may no longer be needed.
- `4.6.0` also removes the top-end pin on `rcssmin` and `rjsmin` and fixes concurrent compression of the same node during offline generation.

## Official Sources

- Documentation root: https://django-compressor.readthedocs.io/en/stable/
- Quickstart: https://django-compressor.readthedocs.io/en/stable/quickstart.html
- Usage: https://django-compressor.readthedocs.io/en/stable/usage.html
- Common deployment scenarios: https://django-compressor.readthedocs.io/en/stable/scenarios.html
- Settings: https://django-compressor.readthedocs.io/en/stable/settings.html
- Remote storages: https://django-compressor.readthedocs.io/en/stable/remote-storages.html
- Changelog: https://django-compressor.readthedocs.io/en/stable/changelog.html
- Jinja2 support: https://django-compressor.readthedocs.io/en/stable/jinja2.html
- django-sekizai support: https://django-compressor.readthedocs.io/en/stable/django-sekizai.html
- PyPI registry: https://pypi.org/project/django-compressor/

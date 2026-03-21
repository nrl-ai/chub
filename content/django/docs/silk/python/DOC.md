---
name: silk
description: "django-silk package guide for Python with safe Django profiling setup, core Silk workflows, and version-drift notes"
metadata:
  languages: "python"
  versions: "5.5.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-silk,django,python,profiling,performance,sql,debugging,middleware"
---

# django-silk Python Package Guide

Use `django-silk` for request, SQL, and Python profiling inside Django apps when you need to inspect slow endpoints or repeated queries from inside the app itself.

## Golden Rule

Enable Silk only for local development, staging, or tightly controlled internal environments. Treat `/silk/` as sensitive operational tooling, not as a public feature.

## Version Drift Note

An earlier version reference pointed to `5.5.0`, but the official public sources I could verify on 2026-03-12 still showed `5.4.3` as the latest visible release on both PyPI and GitHub releases.

Practical implication:

- keep `5.5.0` in frontmatter for queue traceability
- use the public `5.4.3` docs and release history as the verified baseline
- if a project lockfile or private package mirror already pins `5.5.0`, verify the exact release artifact before assuming behavior beyond the `5.4.x` line

## Install

Install the current verified public line:

```bash
python -m pip install "django-silk==5.4.3"
```

Basic install without a version pin:

```bash
python -m pip install django-silk
```

Optional formatting extras for generated Python snippets:

```bash
python -m pip install "django-silk[formatting]"
```

Common project tools:

```bash
poetry add django-silk
uv add django-silk
```

If your project is already pinned to version used here `5.5.0`, keep that project pin, but do not assume the public PyPI page or GitHub release notes already document it.

## Compatibility

Current public docs on PyPI list support for:

- Django `4.2`, `5.1`, `5.2`
- Python `3.9`, `3.10`, `3.11`, `3.12`, `3.13`

The current GitHub README is newer than the public `5.4.3` release notes and lists a slightly newer support matrix. Treat the PyPI package page and tagged releases as the safer baseline when your project is version-sensitive.

## Minimal Setup

Add the app:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "silk",
]
```

Add Silk middleware near the top so it sees most of the request lifecycle. If you use gzip, keep `GZipMiddleware` before Silk:

```python
# settings.py
MIDDLEWARE = [
    "django.middleware.gzip.GZipMiddleware",
    "silk.middleware.SilkyMiddleware",
    # ...
]
```

If you subclass Silk middleware, route the setting through `SILKY_MIDDLEWARE_CLASS`:

```python
# settings.py
SILKY_MIDDLEWARE_CLASS = "myproject.middleware.MyCustomSilkyMiddleware"

MIDDLEWARE = [
    # ...
    SILKY_MIDDLEWARE_CLASS,
    # ...
]
```

Silk expects the request object in Django templates:

```python
# settings.py
TEMPLATES = [
    {
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "OPTIONS": {
            "context_processors": [
                # ...
                "django.template.context_processors.request",
            ],
        },
    },
]
```

Expose the UI:

```python
# urls.py
from django.urls import include, path

urlpatterns = [
    # ...
    path("silk/", include("silk.urls", namespace="silk")),
]
```

Run database and static setup:

```bash
python manage.py migrate
python manage.py collectstatic --noinput
```

Then hit a few requests and open `/silk/`.

## Safe Default Configuration

Start with auth enabled, sampling turned on, and request/response storage bounded:

```python
# settings.py
SILKY_AUTHENTICATION = True
SILKY_AUTHORISATION = True

SILKY_INTERCEPT_PERCENT = 10
SILKY_MAX_REQUEST_BODY_SIZE = 1024 * 50
SILKY_MAX_RESPONSE_BODY_SIZE = 1024 * 50
SILKY_MAX_RECORDED_REQUESTS = 10**4
SILKY_MAX_RECORDED_REQUESTS_CHECK_PERCENT = 10
SILKY_SENSITIVE_KEYS = {
    "authorization",
    "cookie",
    "csrftoken",
    "password",
    "secret",
    "session",
    "token",
}
```

If you want conditional capture, use `SILKY_INTERCEPT_FUNC` instead of `SILKY_INTERCEPT_PERCENT`:

```python
# settings.py
def record_selected_requests(request):
    return request.user.is_staff and "record_requests" in request.session

SILKY_INTERCEPT_FUNC = record_selected_requests
```

## Auth And Permissions

By default, enabling `SILKY_AUTHORISATION` restricts the UI to staff users. Override that with a custom permission callable when your project needs something narrower:

```python
# settings.py
def can_view_silk(user):
    return user.is_authenticated and user.is_superuser

SILKY_PERMISSIONS = can_view_silk
```

Operationally, `/silk/` should usually sit behind at least one of:

- Django auth and staff or superuser checks
- an internal admin network or VPN
- a temporary feature flag during an investigation window

## Core Usage

### Request and SQL inspection

Silk stores request and response metadata, timing, SQL query lists, and per-request totals in your Django database. The common starting views are:

- `/silk/requests/` for slow endpoints and per-request totals
- `/silk/sql/` for query shape, table involvement, joins, and timings

This is where you usually spot N+1-style behavior, duplicated queries, or views spending too much time in ORM work.

### Enable Python profiling

For request-level `cProfile` output:

```python
# settings.py
SILKY_PYTHON_PROFILER = True
SILKY_PYTHON_PROFILER_BINARY = True
SILKY_PYTHON_PROFILER_EXTENDED_FILE_NAME = True
```

For Django `4.2+` and Silk `5.1.0+`, configure profiler result storage through `STORAGES` instead of `SILKY_STORAGE_CLASS`:

```python
# settings.py
STORAGES = {
    "default": {
        "BACKEND": "django.core.files.storage.FileSystemStorage",
    },
    "SILKY_STORAGE": {
        "BACKEND": "myproject.storage.ProfilerStorage",
    },
}
```

If you use the default profiler storage, you can choose a result directory:

```python
# settings.py
SILKY_PYTHON_PROFILER_RESULT_PATH = "/tmp/silk-profiles"
```

The saved `.prof` files work with tools such as `snakeviz`.

### Profile specific code

Decorator:

```python
from silk.profiling.profiler import silk_profile

@silk_profile(name="checkout flow")
def build_checkout(user, cart):
    ...
```

Context manager:

```python
from silk.profiling.profiler import silk_profile

def create_invoice(order):
    with silk_profile(name=f"invoice #{order.pk}"):
        return serialize_order(order)
```

### Dynamic profiling

Use `SILKY_DYNAMIC_PROFILING` when you need to patch a function or code block without editing its source:

```python
# settings.py
SILKY_DYNAMIC_PROFILING = [
    {
        "module": "payments.service",
        "function": "build_quote",
    }
]
```

Point the config at the symbol actually imported and called by runtime code, not just the function's original source module.

### Conditional Python profiling

If full request profiling is too expensive, gate it with `SILKY_PYTHON_PROFILER_FUNC`:

```python
# settings.py
def should_profile_request(request):
    return request.user.is_staff and "profile_requests" in request.session

SILKY_PYTHON_PROFILER_FUNC = should_profile_request
```

This setting takes precedence over `SILKY_PYTHON_PROFILER` and still works with dynamic profiling.

## Useful Configuration Flags

- `SILKY_META = True` records how much extra time Silk itself adds while saving profiling data.
- `SILKY_ANALYZE_QUERIES = True` enables DB-backed query analysis when supported.
- `SILKY_EXPLAIN_FLAGS = {"format": "JSON", "costs": True}` passes extra explain flags to supported backends.
- `SILKY_MAX_RECORDED_REQUESTS` caps stored request rows.
- `SILKY_MAX_RECORDED_REQUESTS_CHECK_PERCENT` controls how often automatic garbage collection runs.

## Cleanup And Operations

Silk persists captured requests in your application database, so clean it up regularly:

```bash
python manage.py silk_request_garbage_collect
python manage.py silk_clear_request_log
```

If you want cleanup decoupled from normal request handling, set `SILKY_MAX_RECORDED_REQUESTS_CHECK_PERCENT = 0` and run `silk_request_garbage_collect` from cron or another scheduled job.

## Common Pitfalls

- Middleware order matters. If middleware before Silk returns a response without calling `get_response`, Silk never sees that request.
- Keep `django.middleware.gzip.GZipMiddleware` before Silk or you can hit encoding issues.
- Missing `django.template.context_processors.request` breaks parts of the UI.
- `SILKY_ANALYZE_QUERIES = True` can execute a query twice on some backends, especially with `EXPLAIN ANALYZE`, so do not enable it casually against write-heavy code paths.
- Request and response bodies are stored by default and can grow the database quickly on high-volume or large-payload systems.
- Python `3.12+` profiling is concurrency-sensitive because `cProfile` cannot be enabled multiple times concurrently.
- Silk itself adds overhead. Use it to diagnose an issue, then reduce capture or disable it again.

## Version-Sensitive Notes

- Public upstream sources observed on 2026-03-12 still show `5.4.3` as the latest visible release, not the version used here `5.5.0`.
- `5.4.3` fixed double `EXPLAIN` behavior and serialization issues for binary and JSON fields.
- `5.3.2` refreshed middleware-order documentation.
- `5.3.0` removed support for Django `3.2` and Python `3.8`, added Python `3.13`, and upgraded jQuery UI to address an XSS issue.
- `5.2.0` added Django `5.1` support.
- `5.1.0` deprecated `SILKY_STORAGE_CLASS` in favor of Django `STORAGES` and documented Python `3.12` profiler concurrency limits.

## Recommended Agent Workflow

1. Confirm whether the target project is using the public `5.4.x` line or an internal/private `5.5.0` artifact.
2. Install Silk and wire `silk` plus `SilkyMiddleware` into settings.
3. Lock down `/silk/` before generating real traffic.
4. Start with request sampling and bounded body sizes.
5. Use `/silk/requests/` and `/silk/sql/` first, then add `@silk_profile` or `SILKY_DYNAMIC_PROFILING` only for specific hotspots.
6. Clear captured data after the investigation so stale traces do not keep accumulating.

## Official Sources

- Repository: https://github.com/jazzband/django-silk
- README: https://github.com/jazzband/django-silk/blob/master/README.md
- Releases: https://github.com/jazzband/django-silk/releases
- PyPI: https://pypi.org/project/django-silk/

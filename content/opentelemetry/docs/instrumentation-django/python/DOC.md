---
name: instrumentation-django
description: "OpenTelemetry Django instrumentation for tracing Django requests, middleware, templates, and database activity in Python apps"
metadata:
  languages: "python"
  versions: "0.61b0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opentelemetry,django,observability,tracing,otlp,instrumentation"
---

# OpenTelemetry Django Instrumentation for Python

## Golden Rule

Use `opentelemetry-instrumentation-django` only as one piece of a complete OpenTelemetry setup: install the Django instrumentation package, configure an SDK and exporter or `opentelemetry-distro`, and instrument the app exactly once during process startup. This package is on a beta contrib release line, so pin exact versions and keep related OpenTelemetry contrib packages on the same release line.

## Install

For programmatic instrumentation, install the package plus an SDK and exporter:

```bash
python -m pip install "opentelemetry-instrumentation-django==0.61b0" \
  opentelemetry-sdk \
  opentelemetry-exporter-otlp
```

If you want zero-code or mostly zero-code setup, install the distro and bootstrap dependencies:

```bash
python -m pip install "opentelemetry-distro" "opentelemetry-instrumentation-django==0.61b0"
opentelemetry-bootstrap -a install
```

Practical versioning rule:

- Pin this package explicitly because `0.61b0` is a pre-release.
- Avoid mixing `0.61b0` with older `0.60b1` or main-branch `0.62b0.dev` contrib packages.
- If your project already pins other `opentelemetry-instrumentation-*` packages, keep them on the same beta line unless you have verified compatibility.

## Initialize Programmatically

Instrument before Django builds the WSGI or ASGI application so the middleware is inserted early enough.

Example `mysite/wsgi.py`:

```python
import os

from django.core.wsgi import get_wsgi_application

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.django import DjangoInstrumentor
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

resource = Resource.create(
    {
        "service.name": os.getenv("OTEL_SERVICE_NAME", "mysite"),
        "deployment.environment.name": os.getenv("DEPLOY_ENV", "dev"),
    }
)

provider = TracerProvider(resource=resource)
provider.add_span_processor(
    BatchSpanProcessor(
        OTLPSpanExporter(
            endpoint=os.getenv("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317"),
        )
    )
)
trace.set_tracer_provider(provider)

def response_hook(span, request, response):
    if span and getattr(request, "user", None) and request.user.is_authenticated:
        span.set_attribute("enduser.id", str(request.user.pk))

DjangoInstrumentor().instrument(response_hook=response_hook)

application = get_wsgi_application()
```

Notes:

- Call `DjangoInstrumentor().instrument()` once per process.
- Use `response_hook` when you need middleware-populated request data such as `request.user`; the maintainer docs note that `request_hook` runs before middleware executes.
- The package exposes `DjangoInstrumentor().uninstrument()` if you need to disable it in tests or teardown code.

## Zero-Code Startup

OpenTelemetry's Python zero-code flow can instrument Django from the command line. Do not combine this with a manual `DjangoInstrumentor().instrument()` call in the same process.

Typical setup:

```bash
export OTEL_SERVICE_NAME=mysite
export OTEL_TRACES_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf

opentelemetry-instrument python manage.py runserver
```

For OTLP backends that require auth, add headers such as:

```bash
export OTEL_EXPORTER_OTLP_HEADERS="authorization=Bearer <token>"
```

Practical note:

- If the Django development autoreloader causes duplicate initialization or duplicate spans, run the underlying app entrypoint without reloading or instrument the production server command instead. This is an operational safeguard rather than a Django-package-specific API rule.

## Core Django-Specific Configuration

### Exclude noisy URLs

The maintainer docs support a Django-specific excluded-URLs environment variable:

```bash
export OTEL_PYTHON_DJANGO_EXCLUDED_URLS="healthz,metrics,admin/jsi18n/"
```

You can also pass `excluded_urls=` directly when instrumenting.

### Capture selected request attributes

Set `OTEL_PYTHON_DJANGO_TRACED_REQUEST_ATTRS` to a comma-separated list of request attribute names:

```bash
export OTEL_PYTHON_DJANGO_TRACED_REQUEST_ATTRS="resolver_match,user.is_authenticated"
```

Use this carefully. Only trace small, stable attributes that are safe to emit.

### Control middleware placement

If middleware ordering matters, the package supports both:

- `middleware_position=` in `DjangoInstrumentor().instrument(...)`
- `OTEL_PYTHON_DJANGO_MIDDLEWARE_POSITION`

This is useful when custom middleware must run before or after OpenTelemetry's middleware.

### Temporarily disable Django instrumentation

If the package is installed but should not instrument a process, set:

```bash
export OTEL_PYTHON_DJANGO_INSTRUMENT=False
```

### Request and response hooks

The Django instrumentor accepts `request_hook` and `response_hook` callbacks:

```python
from opentelemetry.instrumentation.django import DjangoInstrumentor

def request_hook(span, request):
    if span:
        span.set_attribute("http.request_id", request.headers.get("X-Request-ID", ""))

def response_hook(span, request, response):
    if span:
        span.set_attribute("http.response.template_status", response.status_code)

DjangoInstrumentor().instrument(
    request_hook=request_hook,
    response_hook=response_hook,
)
```

The official docs explicitly call out that response hooks are the right place to read middleware-generated values like authenticated user data or `django.contrib.sites` information.

## Headers And SQL Commenter

The maintainer docs support the standard Python instrumentation environment variables for HTTP header capture and sanitization:

- `OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_REQUEST`
- `OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_RESPONSE`
- `OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SANITIZE_FIELDS`

Use these sparingly and sanitize aggressively because headers often contain secrets or PII.

SQLCommenter support is built in and can be enabled with:

```python
DjangoInstrumentor().instrument(
    is_sql_commentor_enabled=True,
)
```

Or in `settings.py`:

```python
SQLCOMMENTER_WITH_CONTROLLER = True
SQLCOMMENTER_WITH_ROUTE = True
SQLCOMMENTER_WITH_APP_NAME = True
SQLCOMMENTER_WITH_OPENTELEMETRY = True
```

Only enable SQL commenting when your database path and privacy requirements allow it. If you already use SQL commenter features elsewhere in your stack, avoid duplicating them.

## Common Pitfalls

- Do not mix zero-code startup and manual `DjangoInstrumentor().instrument()` in the same process.
- Instrument before `get_wsgi_application()` or `get_asgi_application()` so middleware insertion happens early enough.
- Prefer `response_hook` over `request_hook` for data populated by Django middleware.
- Be conservative with traced request attributes and header capture to avoid leaking secrets or high-cardinality data.
- Exclude health, readiness, metrics, and similar endpoints early so they do not dominate trace volume.
- Treat the ReadTheDocs pages as maintainer guidance for the current contrib line, not as a perfect snapshot of `0.61b0`.

## Version-Sensitive Notes

- The version used here and the PyPI release page align on `0.61b0`.
- The maintainer docs page is versionless `latest`, and the public repository already advertises `0.62b0.dev`; that means examples in `latest` docs can drift ahead of the exact release you have pinned.
- When there is any ambiguity, prefer the exact PyPI release page for package version identity and the package docs page for behavioral guidance.

## Official Sources Used

- PyPI release page: `https://pypi.org/project/opentelemetry-instrumentation-django/0.61b0/`
- Django instrumentation docs: `https://opentelemetry-python-contrib.readthedocs.io/en/latest/instrumentation/django/django.html`
- OpenTelemetry Python zero-code example docs: `https://opentelemetry.io/docs/zero-code/python/example/`
- Maintainer repo package metadata: `https://raw.githubusercontent.com/open-telemetry/opentelemetry-python-contrib/main/instrumentation/opentelemetry-instrumentation-django/pyproject.toml`
- Maintainer repo instrumentor code: `https://raw.githubusercontent.com/open-telemetry/opentelemetry-python-contrib/main/instrumentation/opentelemetry-instrumentation-django/src/opentelemetry/instrumentation/django/__init__.py`
- Maintainer repo environment variable definitions: `https://raw.githubusercontent.com/open-telemetry/opentelemetry-python-contrib/main/instrumentation/opentelemetry-instrumentation-django/src/opentelemetry/instrumentation/django/environment_variables.py`

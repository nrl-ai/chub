---
name: opentelemetry
description: "OpenTelemetry Flask instrumentation for tracing Flask request handling, hooks, header capture, and OTLP export setup"
metadata:
  languages: "python"
  versions: "0.61b0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,opentelemetry,otel,tracing,observability,otlp"
---

# OpenTelemetry Flask Instrumentation

## Golden Rule

Configure a tracer provider and exporter first, then instrument the Flask app exactly once with `FlaskInstrumentor().instrument_app(app)`. Set `service.name` explicitly. Do not mix programmatic `instrument_app(app)` with separate global `FlaskInstrumentor().instrument()` patching on the same app object.

## Install

For explicit in-app instrumentation:

```bash
python -m pip install "Flask>=1.0" \
  "opentelemetry-instrumentation-flask==0.61b0" \
  opentelemetry-sdk \
  opentelemetry-exporter-otlp-proto-http
```

For zero-code or bootstrap-driven instrumentation:

```bash
python -m pip install "Flask>=1.0" opentelemetry-distro
opentelemetry-bootstrap -a install
```

Notes:

- `opentelemetry-instrumentation-flask` is a contrib instrumentation package, not the full SDK.
- Keep OpenTelemetry packages on the same release train where practical. This package depends on matching contrib instrumentation components internally.
- PyPI marks `0.61b0` as a beta pre-release. Pin if you need reproducible behavior.

## Manual Setup For A Flask App

This is the safest pattern when you control app startup and want predictable instrumentation:

```python
from flask import Flask

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.flask import FlaskInstrumentor
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

app = Flask(__name__)

resource = Resource.create(
    {
        SERVICE_NAME: "orders-api",
    }
)

trace_provider = TracerProvider(resource=resource)
trace_provider.add_span_processor(
    BatchSpanProcessor(
        OTLPSpanExporter(endpoint="http://localhost:4318/v1/traces")
    )
)
trace.set_tracer_provider(trace_provider)

FlaskInstrumentor().instrument_app(app, tracer_provider=trace_provider)

@app.get("/healthz")
def healthz():
    return {"ok": True}

if __name__ == "__main__":
    app.run(debug=True)
```

What this package adds for Flask:

- server spans around incoming requests
- span names based on the Flask URL rule
- `http.route` populated from the matched Flask route
- request/response hooks for custom span attributes
- optional SQLCommenter context enrichment for instrumented database stacks

## Core Usage

### Exclude noisy routes

Use this for health checks, metrics endpoints, and other low-value traffic:

```python
FlaskInstrumentor().instrument_app(
    app,
    excluded_urls="healthz,metrics,/static/.*",
)
```

Equivalent environment variable:

```bash
export OTEL_PYTHON_FLASK_EXCLUDED_URLS="healthz,metrics,/static/.*"
```

You can also use `OTEL_PYTHON_EXCLUDED_URLS` to apply exclusions across all Python instrumentations.

### Add request and response hooks

Hooks let you attach app-specific attributes without manually managing spans:

```python
from opentelemetry.trace import Span

def request_hook(span: Span, environ) -> None:
    if span and span.is_recording():
        tenant_id = environ.get("HTTP_X_TENANT_ID")
        if tenant_id:
            span.set_attribute("app.tenant_id", tenant_id)

def response_hook(span: Span, status: str, response_headers) -> None:
    if span and span.is_recording():
        span.set_attribute("app.http_status_line", status)

FlaskInstrumentor().instrument_app(
    app,
    request_hook=request_hook,
    response_hook=response_hook,
)
```

### Capture selected headers

```bash
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_REQUEST="x-request-id,user-agent"
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_RESPONSE="content-type"
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SANITIZE_FIELDS=".*session.*,set-cookie,authorization"
```

Important:

- request header names are case-insensitive
- response header names are case-insensitive
- captured header attributes use normalized lowercase names with `-` changed to `_`
- the header capture environment variable names are still experimental upstream

### Enable SQLCommenter when you also instrument your DB stack

```python
FlaskInstrumentor().instrument_app(
    app,
    enable_commenter=True,
    commenter_options={
        "controller": False,
    },
)
```

This only helps if your database driver or ORM instrumentation also supports SQLCommenter. Flask instrumentation enriches that downstream SQL comment with framework, route, and controller metadata.

### Uninstrument in tests or app reinitialization code

```python
FlaskInstrumentor().uninstrument_app(app)
```

This restores the original WSGI app wrapper and removes the registered Flask hooks. It is useful in test suites that reuse or rebuild the same app object.

## Zero-Code Instrumentation

If you want automatic instrumentation around a Flask process without editing startup code:

```bash
python -m pip install "Flask>=1.0" opentelemetry-distro
opentelemetry-bootstrap -a install

export OTEL_SERVICE_NAME=orders-api
opentelemetry-instrument \
  --traces_exporter console \
  --metrics_exporter none \
  --logs_exporter none \
  python app.py
```

Notes:

- `opentelemetry-bootstrap -a install` installs default and detected instrumentation packages for the current environment.
- `opentelemetry-instrument` defaults to OTLP exporters unless you override exporters explicitly.
- If you need to suppress Flask instrumentation in a larger auto-instrumented process, use `OTEL_PYTHON_DISABLED_INSTRUMENTATIONS=flask`.

## Configuration And OTLP Auth

OpenTelemetry backends usually need three things from you:

1. A stable service identity
2. An exporter endpoint
3. Auth headers or collector-side auth

Common environment variables:

```bash
export OTEL_SERVICE_NAME=orders-api
export OTEL_RESOURCE_ATTRIBUTES=deployment.environment.name=prod,service.namespace=payments
export OTEL_EXPORTER_OTLP_ENDPOINT=https://otel.example.com
export OTEL_EXPORTER_OTLP_HEADERS="authorization=Bearer <token>,x-tenant-id=acme"
```

Key points:

- `service.name` defaults to `unknown_service` if you do not set it, so set `OTEL_SERVICE_NAME` or create a `Resource` in code.
- `OTEL_RESOURCE_ATTRIBUTES` is the usual place for deployment environment and other resource metadata.
- The agent configuration docs map `exporter_otlp_endpoint` to `OTEL_EXPORTER_OTLP_ENDPOINT`.
- Upstream docs note default collector endpoints of `0.0.0.0:4317` for gRPC and `0.0.0.0:4318` for HTTP when you do not override them.
- `OTEL_EXPORTER_OTLP_HEADERS` is commonly required for vendor backends that expect API keys or bearer tokens.

## Common Pitfalls

- Do not expect this package to create spans in your backend by itself. You still need an SDK and exporter or zero-code distro setup.
- Do not instrument the same app twice. The source warns if an app is already instrumented.
- Prefer `instrument_app(app)` in normal Flask application code. `FlaskInstrumentor().instrument()` patches `flask.Flask` globally.
- Set `OTEL_SERVICE_NAME` or an explicit `Resource`. Otherwise your service appears as `unknown_service`.
- Be conservative with header capture. Capturing `.*` can leak secrets and produce high-cardinality attributes.
- If you enable SQLCommenter, also enable it in the database instrumentation you actually use; Flask alone is not enough.
- Excluded URL patterns are regexes matched against the request URL. Test them before using broad patterns in production.

## Version-Sensitive Notes

- `0.61b0` is the current PyPI release for this package as of March 12, 2026, and it is still a beta pre-release.
- This release requires Python 3.9+.
- The package metadata exposes an `instruments` extra that pulls in `flask >= 1.0`.
- The package entry point name for disable-lists is `flask`, which is why `OTEL_PYTHON_DISABLED_INSTRUMENTATIONS=flask` works.
- The header-capture environment variables are explicitly marked experimental in the upstream docs and may change across future contrib releases.

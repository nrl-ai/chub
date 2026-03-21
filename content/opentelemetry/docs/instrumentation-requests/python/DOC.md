---
name: instrumentation-requests
description: "OpenTelemetry requests instrumentation for tracing and metrics on outbound Python requests calls"
metadata:
  languages: "python"
  versions: "0.61b0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opentelemetry,requests,tracing,metrics,observability,http"
---

# opentelemetry-instrumentation-requests Python Package Guide

## Golden Rule

`opentelemetry-instrumentation-requests` only patches `requests`; it does not configure telemetry export by itself. Set up the OpenTelemetry SDK or distro first, instrument once during process startup, and avoid combining manual `RequestsInstrumentor().instrument()` with `opentelemetry-instrument` for the same process.

## Install

Install the instrumented library plus the OpenTelemetry pieces your app actually uses.

Minimal manual setup:

```bash
python -m pip install "requests" \
  "opentelemetry-api" \
  "opentelemetry-sdk" \
  "opentelemetry-instrumentation-requests==0.61b0"
```

Common production setup with OTLP export:

```bash
python -m pip install "requests" \
  "opentelemetry-distro" \
  "opentelemetry-exporter-otlp" \
  "opentelemetry-instrumentation-requests==0.61b0"
```

If you are using auto-instrumentation broadly, the OpenTelemetry docs recommend:

```bash
python -m pip install opentelemetry-distro opentelemetry-exporter-otlp
opentelemetry-bootstrap -a install
```

`opentelemetry-bootstrap -a install` installs matching instrumentation packages for libraries already present in the active environment.

## Manual Setup

For code-driven setup, initialize the SDK before instrumenting `requests`:

```python
import requests

from opentelemetry import trace
from opentelemetry.instrumentation.requests import RequestsInstrumentor
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor, ConsoleSpanExporter

resource = Resource.create({"service.name": "billing-client"})

provider = TracerProvider(resource=resource)
provider.add_span_processor(BatchSpanProcessor(ConsoleSpanExporter()))
trace.set_tracer_provider(provider)

RequestsInstrumentor().instrument()

response = requests.get("https://httpbin.org/get", timeout=10)
print(response.status_code)
```

Use a real exporter such as OTLP in production; `ConsoleSpanExporter` is mainly useful to confirm that spans are being created.

If your app manages providers explicitly, `instrument()` also accepts `tracer_provider=` and `meter_provider=` keyword arguments.

## Auto-Instrumentation

If the process already uses the OpenTelemetry Python agent, do not also call `RequestsInstrumentor().instrument()` in your code.

CLI example:

```bash
opentelemetry-instrument \
  --traces_exporter otlp \
  --metrics_exporter console \
  --service_name billing-client \
  python app.py
```

Environment-variable example:

```bash
export OTEL_SERVICE_NAME="billing-client"
export OTEL_TRACES_EXPORTER="otlp"
export OTEL_METRICS_EXPORTER="console"
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4318/v1/traces"

opentelemetry-instrument python app.py
```

Auto-instrumentation uses monkey patching, so start the process through `opentelemetry-instrument` from the beginning rather than enabling it halfway through a long-running worker.

## Core Usage

### Instrument all outbound `requests` traffic

After `RequestsInstrumentor().instrument()`, normal `requests.get()`, `post()`, and `Session` calls emit client telemetry automatically:

```python
import requests
from opentelemetry.instrumentation.requests import RequestsInstrumentor

RequestsInstrumentor().instrument()

session = requests.Session()
session.headers["Authorization"] = "Bearer secret-token"

response = session.post(
    "https://api.example.com/orders",
    json={"id": 123},
    timeout=10,
)
```

### Add request and response hooks

Hooks are the main extension point for attaching application-specific attributes:

```python
import requests

from opentelemetry.instrumentation.requests import RequestsInstrumentor

def request_hook(span, request_obj):
    if span and span.is_recording():
        request_id = request_obj.headers.get("X-Request-ID")
        if request_id:
            span.set_attribute("app.request_id", request_id)

def response_hook(span, request_obj, response):
    if span and span.is_recording():
        span.set_attribute("app.status_family", f"{response.status_code // 100}xx")

RequestsInstrumentor().instrument(
    request_hook=request_hook,
    response_hook=response_hook,
)

requests.get("https://httpbin.org/status/200", timeout=10)
```

`request_obj` is a `requests.PreparedRequest`, and `response` is a `requests.Response`.

### Exclude noisy endpoints

Use regex-based exclude lists for health checks, metadata calls, or other low-value traffic:

```bash
export OTEL_PYTHON_REQUESTS_EXCLUDED_URLS="healthcheck,client/.*/info"
```

You can also pass `excluded_urls=` directly to `instrument()` if you want the rule in code instead of environment config.

### Capture selected request and response headers

`0.61b0` documents client request and client response header capture through environment variables:

```bash
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_CLIENT_REQUEST="x-request-id,content-type"
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_CLIENT_RESPONSE="content-type,x-ratelimit-remaining"
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SANITIZE_FIELDS="authorization,cookie,set-cookie"
```

Header capture is useful for debugging, but sanitize aggressively. These environment variable names are still marked experimental in the upstream docs.

### Customize request-duration histogram boundaries

If you are tuning metrics buckets, pass explicit boundaries:

```python
from opentelemetry.instrumentation.requests import RequestsInstrumentor

RequestsInstrumentor().instrument(
    duration_histogram_boundaries=[0.0, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
)
```

Be careful here during semantic-convention migration: the source code for `0.61b0` still supports both the old `http.client.duration` metric in milliseconds and the newer `http.client.request.duration` metric in seconds, depending on the active semantic-convention mode.

## Configuration And Auth Notes

- This package has no package-specific authentication flow.
- Outbound request authentication is whatever your `requests` code sets, such as `Authorization` headers, cookies, or client certificates.
- Telemetry exporter authentication is configured on the SDK/exporter or distro side, not on `RequestsInstrumentor`.
- The instrumentor injects tracing context headers into outbound requests automatically. Do not confuse those propagation headers with application auth headers.

## Common Pitfalls

- No SDK, no telemetry. Installing this package alone does not export spans anywhere.
- Do not double instrument. Pick either manual `RequestsInstrumentor().instrument()` or the `opentelemetry-instrument` launcher for a given process.
- Instrument as early as possible in startup so all worker traffic is patched consistently.
- This package instruments outbound HTTP clients only. It does not create inbound server spans for Django, Flask, FastAPI, or ASGI apps.
- If you capture headers, sanitize `authorization`, `cookie`, `set-cookie`, and any tenant- or user-identifying custom headers.
- Exclude-list patterns are regexes, not plain path prefixes. Test them before assuming traffic is filtered.
- If you need to remove instrumentation from a single `requests.Session`, use `RequestsInstrumentor.uninstrument_session(session)`.

## Version-Sensitive Notes For `0.61b0`

- PyPI lists `0.61b0` as a pre-release published on March 4, 2026. This package is still in the contrib `0.x` beta line.
- The contrib repository states these instrumentation libraries are currently beta and generally should not be treated like fully stable `1.0` APIs.
- The contrib repository also notes a monthly release cadence, so version drift is common. Re-check PyPI before pinning examples in generated code.
- HTTP semantic-convention migration is active across Python HTTP-related instrumentations. Attribute names and metric names can differ across environments depending on semantic-convention opt-in settings.

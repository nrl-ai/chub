---
name: instrumentation-fastapi
description: "OpenTelemetry FastAPI instrumentation for tracing incoming FastAPI requests in Python applications"
metadata:
  languages: "python"
  versions: "0.61b0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opentelemetry,otel,fastapi,asgi,tracing,observability"
---

# OpenTelemetry FastAPI Instrumentation Python Package Guide

## Golden Rule

`opentelemetry-instrumentation-fastapi` instruments incoming FastAPI and ASGI request handling, but it does not become useful until you also configure an OpenTelemetry SDK/exporter or run under `opentelemetry-instrument`. As of `2026-03-12`, the PyPI project page publishes `0.61b0` for this package. The canonical docs page is the FastAPI instrumentation page under `opentelemetry-python-contrib.readthedocs.io`, not the broader OpenTelemetry Python docs root.


## Install

For manual setup, install the FastAPI instrumentation package plus an SDK and exporter:

```bash
python -m pip install \
  "opentelemetry-sdk" \
  "opentelemetry-exporter-otlp-proto-http" \
  "opentelemetry-instrumentation-fastapi==0.61b0"
```

If you plan to use zero-code auto-instrumentation:

```bash
python -m pip install \
  "opentelemetry-distro" \
  "opentelemetry-exporter-otlp" \
  "opentelemetry-instrumentation-fastapi==0.61b0"
opentelemetry-bootstrap -a install
```

Practical version note:

- Keep contrib instrumentation packages on the same beta release train in one environment, for example `opentelemetry-instrumentation-fastapi`, `opentelemetry-instrumentation`, and `opentelemetry-instrumentation-asgi`. That is an inference from the official shared source tree and release cadence, and it avoids resolver churn across mixed `0.xx` prereleases.

## Manual Setup

Create a tracer provider, attach an exporter, then instrument the FastAPI app before the app starts serving traffic:

```python
from fastapi import FastAPI

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

resource = Resource.create({SERVICE_NAME: "orders-api"})
tracer_provider = TracerProvider(resource=resource)
tracer_provider.add_span_processor(
    BatchSpanProcessor(
        OTLPSpanExporter(endpoint="http://localhost:4318/v1/traces")
    )
)
trace.set_tracer_provider(tracer_provider)

app = FastAPI()

FastAPIInstrumentor.instrument_app(
    app,
    tracer_provider=tracer_provider,
    excluded_urls="^/healthz$,^/metrics$",
    exclude_spans=["receive", "send"],
)

@app.get("/orders/{order_id}")
async def get_order(order_id: str) -> dict[str, str]:
    return {"order_id": order_id}
```

Why these choices matter:

- `SERVICE_NAME` keeps spans grouped correctly in your backend.
- `BatchSpanProcessor` is the normal production choice; avoid per-span exporting.
- `exclude_spans=["receive", "send"]` suppresses lower-level ASGI internal spans when you only want server spans.
- `excluded_urls` is a regex list for endpoints like health checks and metrics that would otherwise dominate trace volume.

## Zero-Code Setup

If you want automatic instrumentation at process launch, configure the exporter with environment variables and run the server through `opentelemetry-instrument`:

```bash
export OTEL_SERVICE_NAME=orders-api
export OTEL_TRACES_EXPORTER=otlp
export OTEL_METRICS_EXPORTER=none
export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318

opentelemetry-instrument uvicorn app:app --host 0.0.0.0 --port 8000
```

Use zero-code instrumentation when you want broad framework coverage with minimal code changes. Use manual setup when you need explicit SDK construction, custom resources, or programmatic control over sampling and span processors.

## Core Usage

### Add hooks to customize spans

The FastAPI instrumentation supports server hooks plus ASGI client hooks for the internal `receive` and `send` spans:

```python
from typing import Any

from fastapi import FastAPI
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.trace import Span

app = FastAPI()

def server_request_hook(span: Span, scope: dict[str, Any]) -> None:
    if span and span.is_recording():
        route = scope.get("path", "")
        span.set_attribute("app.route_path", route)

def client_request_hook(span: Span, scope: dict[str, Any], message: dict[str, Any]) -> None:
    if span and span.is_recording():
        span.set_attribute("app.asgi.event.type", message.get("type", ""))

FastAPIInstrumentor.instrument_app(
    app,
    server_request_hook=server_request_hook,
    client_request_hook=client_request_hook,
)
```

Use hooks for request-derived attributes, tenant IDs, feature flags, or other context that is not automatically captured.

### Capture selected headers

The FastAPI instrumentation docs support header capture via environment variables:

```bash
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_REQUEST="x-request-id,authorization"
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_RESPONSE="content-type"
export OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SANITIZE_FIELDS="authorization,set-cookie"
```

Notes:

- Header capture is explicitly marked experimental in the official docs.
- Always sanitize secrets such as `authorization`, cookies, or internal tokens before enabling broad capture.

### Exclude URLs from instrumentation

You can exclude URLs in code with `excluded_urls=...`, or by environment variable:

```bash
export OTEL_PYTHON_FASTAPI_EXCLUDED_URLS="^/healthz$,^/metrics$"
```

The docs also note the generic fallback:

```bash
export OTEL_PYTHON_EXCLUDED_URLS="^/healthz$,^/metrics$"
```

## Config And Auth

The FastAPI instrumentation package itself does not require an API key. Auth usually matters in one of two places:

1. Your exporter to the collector or backend, for example OTLP headers such as `OTEL_EXPORTER_OTLP_HEADERS`.
2. Your application code, which may need its own auth middleware or dependency injection and can attach request metadata through hooks.

Useful config knobs:

- `OTEL_SERVICE_NAME` or `OTEL_RESOURCE_ATTRIBUTES`: service identity
- `OTEL_EXPORTER_OTLP_ENDPOINT`: collector base URL
- `OTEL_EXPORTER_OTLP_PROTOCOL`: usually `http/protobuf` or `grpc`
- `OTEL_EXPORTER_OTLP_HEADERS`: auth or tenant headers for the collector
- `OTEL_PYTHON_FASTAPI_EXCLUDED_URLS`: FastAPI-specific instrumentation exclusions
- `OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_REQUEST`: opt-in request header capture
- `OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SERVER_RESPONSE`: opt-in response header capture
- `OTEL_INSTRUMENTATION_HTTP_CAPTURE_HEADERS_SANITIZE_FIELDS`: redact sensitive headers

## Common Pitfalls

- Installing only `opentelemetry-instrumentation-fastapi` is not enough. Without an SDK/exporter or distro, you will not see useful telemetry.
- Instrument the app once. The official source keeps an `_is_instrumented_by_opentelemetry` flag and warns on duplicate instrumentation attempts.
- Instrument before the app handles traffic. Delayed instrumentation can miss startup-time behavior or interact poorly with test fixtures and reload flows.
- This package covers inbound FastAPI request handling. Add separate instrumentation for outbound HTTP clients, database libraries, or background task queues you also use.
- Excluding nothing can create noisy traces from `/healthz`, `/metrics`, docs routes, and probes.
- Capturing headers without sanitization can leak credentials into telemetry backends.
- In tests or hot-reload scenarios, call `FastAPIInstrumentor.uninstrument_app(app)` before rebuilding instrumentation state.

## Version-Sensitive Notes

- The opened PyPI project page currently lists `opentelemetry-instrumentation-fastapi 0.61b0` with `Requires: Python >=3.9`.
- The live package page currently lists `0.61b0`, but search-result snippets briefly lagged and suggested `0.60b1`.
- Use the FastAPI instrumentation page under `opentelemetry-python-contrib.readthedocs.io` as the canonical docs page.

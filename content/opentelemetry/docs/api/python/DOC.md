---
name: api
description: "OpenTelemetry Python API package guide for manual instrumentation, context propagation, metrics, and logs"
metadata:
  languages: "python"
  versions: "1.40.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opentelemetry,otel,python,observability,tracing,metrics,logs,context-propagation"
---

# OpenTelemetry Python API Package Guide

## Golden Rule

Use `opentelemetry-api` as the dependency surface for reusable libraries, and pair it with `opentelemetry-sdk` in applications that must actually emit telemetry. Keep all OpenTelemetry Python package versions aligned across `opentelemetry-api`, `opentelemetry-sdk`, exporters, and instrumentation packages.

## Install

Library-only dependency:

```bash
python -m pip install "opentelemetry-api==1.40.0"
```

Application setup that will emit telemetry:

```bash
python -m pip install \
  "opentelemetry-api==1.40.0" \
  "opentelemetry-sdk==1.40.0"
```

If the app needs OTLP export, add a matching exporter package:

```bash
python -m pip install "opentelemetry-exporter-otlp-proto-http==1.40.0"
```

## Setup Patterns

### Libraries: depend on API only

Libraries should create tracers, meters, or loggers but should not install global providers. That keeps the host application in control of exporters, batching, sampling, and credentials.

```python
from opentelemetry import trace

tracer = trace.get_tracer("acme.widgets", "2.3.0")

def render_widget(widget_id: str) -> str:
    with tracer.start_as_current_span("acme.widgets.render") as span:
        span.set_attribute("widget.id", widget_id)
        return f"widget:{widget_id}"
```

### Applications: install providers once

Applications should configure the SDK exactly once near process startup, then retrieve tracers or meters from the API.

```python
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor, ConsoleSpanExporter

provider = TracerProvider()
provider.add_span_processor(BatchSpanProcessor(ConsoleSpanExporter()))
trace.set_tracer_provider(provider)

tracer = trace.get_tracer("myapp.checkout", "1.0.0")

with tracer.start_as_current_span("checkout") as span:
    span.set_attribute("cart.items", 3)
```

`trace.set_tracer_provider(...)` can only be done once. If no provider is installed, `get_tracer(...)` still returns an object, but it is backed by the default proxy or no-op behavior and nothing is exported.

## Core Usage

### Create spans

Use a stable instrumentation scope name such as your package or service name. Add attributes rather than encoding data into span names.

```python
from opentelemetry import trace

tracer = trace.get_tracer("payments.api", "2026.3")

def charge(order_id: str, amount_cents: int) -> None:
    with tracer.start_as_current_span("payments.charge") as span:
        span.set_attribute("order.id", order_id)
        span.set_attribute("payment.amount_cents", amount_cents)
```

### Propagate context and baggage

`propagate.inject(...)` writes tracing context and baggage into a carrier such as HTTP headers. `propagate.extract(...)` reconstructs context from an incoming carrier.

```python
from opentelemetry import baggage, propagate
from opentelemetry.context import attach, detach

headers: dict[str, str] = {}

ctx = baggage.set_baggage("tenant.id", "acme")
token = attach(ctx)
try:
    propagate.inject(headers)
finally:
    detach(token)

incoming_context = propagate.extract(headers)
tenant_id = baggage.get_baggage("tenant.id", incoming_context)
```

Use `attach(...)` and `detach(...)` as a pair. Forgetting to detach leaves the current context polluted for later work on the same thread or task.

### Create metrics

The metrics API comes from `opentelemetry-api`, but exporting requires an SDK meter provider and at least one metric reader.

```python
from opentelemetry import metrics
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import ConsoleMetricExporter, PeriodicExportingMetricReader

reader = PeriodicExportingMetricReader(ConsoleMetricExporter())
metrics.set_meter_provider(MeterProvider(metric_readers=[reader]))

meter = metrics.get_meter("myapp.checkout", "1.0.0")
request_counter = meter.create_counter(
    "checkout.requests",
    unit="1",
    description="Number of checkout requests",
)

request_counter.add(1, {"route": "/checkout", "method": "POST"})
```

Like tracing, `metrics.set_meter_provider(...)` should happen once during startup.

### Emit logs through the OpenTelemetry logging pipeline

For application code, the practical path is usually standard `logging` plus the OpenTelemetry SDK logging handler.

```python
import logging

from opentelemetry._logs import set_logger_provider
from opentelemetry.sdk._logs import LoggerProvider, LoggingHandler
from opentelemetry.sdk._logs.export import BatchLogRecordProcessor, ConsoleLogRecordExporter

provider = LoggerProvider()
provider.add_log_record_processor(BatchLogRecordProcessor(ConsoleLogRecordExporter()))
set_logger_provider(provider)

handler = LoggingHandler(level=logging.INFO, logger_provider=provider)
logging.basicConfig(level=logging.INFO, handlers=[handler])

logging.getLogger("myapp.checkout").info("checkout started", extra={"order_id": "ord_123"})
```

If you only install `opentelemetry-api`, loggers can be created from the API, but no provider means no records are exported.

## Configuration Notes

There is no authentication in `opentelemetry-api` itself. Authentication, endpoints, headers, and backend-specific settings belong to SDK exporters such as OTLP exporters.

Useful environment variables from the official Python docs:

- `OTEL_PROPAGATORS`: selects global propagators; the default is `tracecontext,baggage`
- `OTEL_PYTHON_CONTEXT`: selects the runtime context implementation
- `OTEL_PYTHON_TRACER_PROVIDER`: loads a tracer provider implementation from an entry point
- `OTEL_PYTHON_METER_PROVIDER`: loads a meter provider implementation from an entry point

Practical guidance:

- Prefer explicit startup code for provider registration in application code unless your deployment already standardizes environment-driven setup.
- Keep exporter endpoint and credential configuration in the SDK/exporter layer, not in library code that only depends on `opentelemetry-api`.
- When using custom carriers for propagation, implement the appropriate getter or setter instead of assuming your object behaves like a plain dict.

## Common Pitfalls

- Installing `opentelemetry-api` alone does not make spans, metrics, or logs appear in a backend. You need the SDK plus exporter or readers.
- Global provider setters are one-time operations. If multiple modules try to set them, later attempts log warnings and are ignored.
- Do not make reusable libraries call `set_tracer_provider(...)` or `set_meter_provider(...)`. That steals control from the host app.
- Avoid unstable instrumentation scope names such as `__name__` in scripts. Use a durable package or service name so telemetry stays grouped sensibly.
- `propagate.inject(...)` only writes context to a carrier. It does not send network requests or automatically attach headers to your HTTP client.
- Baggage is for small cross-service key-value context, not for large payloads or secrets.
- Logs in OpenTelemetry Python are still marked as development status. Treat logging internals as more version-sensitive than traces and metrics.

## Version-Sensitive Notes For 1.40.0

- As of March 12, 2026, PyPI lists `opentelemetry-api 1.40.0` as the current release.
- `opentelemetry-api 1.40.0` requires Python `>=3.9`.
- The OpenTelemetry Python repository marks tracing and metrics as stable, while logging remains in development status.
- The current Python instrumentation docs use `ConsoleLogRecordExporter`. Older examples from versions earlier than `1.39.0` may still show `ConsoleLogExporter`; update those snippets before copying them.

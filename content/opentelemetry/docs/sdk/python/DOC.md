---
name: sdk
description: "OpenTelemetry SDK package guide for Python applications using manual instrumentation, resources, exporters, metrics, and logs"
metadata:
  languages: "python"
  versions: "1.40.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opentelemetry,observability,telemetry,tracing,metrics,logs,otlp"
---

# OpenTelemetry SDK Python Package Guide

## Golden Rule

Use `opentelemetry-sdk` in application code, not in reusable libraries. Application code installs and configures the SDK, exporters, and resources; libraries should depend on `opentelemetry-api` only and emit telemetry only when the host application has configured an SDK.

For production, set a real `service.name`, export through OTLP to an OpenTelemetry Collector, and treat console exporters as local debugging tools.

## Install

For manual instrumentation, install the SDK and usually pin the matching API version too:

```bash
python -m pip install "opentelemetry-api==1.40.0" "opentelemetry-sdk==1.40.0"
```

Common alternatives:

```bash
uv add "opentelemetry-api==1.40.0" "opentelemetry-sdk==1.40.0"
poetry add "opentelemetry-api==1.40.0" "opentelemetry-sdk==1.40.0"
```

The SDK does not include every exporter or instrumentation package. Install what you actually use:

```bash
python -m pip install "opentelemetry-exporter-otlp-proto-http==1.40.0"
python -m pip install "opentelemetry-exporter-otlp-proto-grpc==1.40.0"
python -m pip install "opentelemetry-instrumentation-requests==0.61b0"
```

Notes:

- `opentelemetry-api` and `opentelemetry-sdk` are separate packages by design.
- Exporters and instrumentation libraries are separate extension packages.
- If you want zero-code or low-code auto-instrumentation, use the Python distro and contrib instrumentations instead of trying to make `opentelemetry-sdk` do everything on its own.

## Initialize Tracing

This is the shortest reliable manual setup for local development:

```python
from opentelemetry import trace
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor, ConsoleSpanExporter

resource = Resource.create(
    {
        "service.name": "checkout-api",
        "service.version": "1.40.0",
        "deployment.environment": "dev",
    }
)

trace_provider = TracerProvider(resource=resource)
trace_provider.add_span_processor(BatchSpanProcessor(ConsoleSpanExporter()))
trace.set_tracer_provider(trace_provider)

tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("create-order") as span:
    span.set_attribute("app.order_id", "ord_123")
    span.add_event("validating_request")
```

Why this shape:

- `Resource.create(...)` is where service identity belongs.
- `BatchSpanProcessor` is the normal production-oriented processor; `SimpleSpanProcessor` is mostly for debugging or very small programs.
- `trace.set_tracer_provider(...)` configures the process-wide provider. Do this once during startup.

In short-lived scripts and tests, flush before exit:

```python
trace.get_tracer_provider().force_flush()
trace.get_tracer_provider().shutdown()
```

## Add Metrics

Metrics require a `MeterProvider` plus at least one `MetricReader`:

```python
from opentelemetry import metrics
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import (
    ConsoleMetricExporter,
    PeriodicExportingMetricReader,
)
from opentelemetry.sdk.resources import Resource

metric_reader = PeriodicExportingMetricReader(
    ConsoleMetricExporter(),
    export_interval_millis=5000,
)

meter_provider = MeterProvider(
    metric_readers=[metric_reader],
    resource=Resource.create({"service.name": "checkout-api"}),
)
metrics.set_meter_provider(meter_provider)

meter = metrics.get_meter(__name__)
request_counter = meter.create_counter("http.server.requests")
request_counter.add(1, {"http.method": "GET", "http.route": "/healthz"})
```

Notes:

- `PeriodicExportingMetricReader` handles timed collection and export.
- Console metrics are diagnostic output only.
- For unit tests, `InMemoryMetricReader` is often a better fit than waiting for timed exports.

## Export To A Collector With OTLP

Collector-first OTLP is the safest default. Install one OTLP exporter package and configure it with environment variables instead of hard-coding endpoints in application code.

OTLP over HTTP:

```bash
python -m pip install "opentelemetry-exporter-otlp-proto-http==1.40.0"

export OTEL_SERVICE_NAME="checkout-api"
export OTEL_RESOURCE_ATTRIBUTES="deployment.environment=prod,service.version=1.40.0"
export OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
export OTEL_EXPORTER_OTLP_HEADERS="authorization=Bearer <token>"
```

OTLP over gRPC:

```bash
python -m pip install "opentelemetry-exporter-otlp-proto-grpc==1.40.0"

export OTEL_SERVICE_NAME="checkout-api"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export OTEL_EXPORTER_OTLP_HEADERS="authorization=Bearer <token>"
```

Important endpoint behavior:

- With `OTEL_EXPORTER_OTLP_ENDPOINT` and OTLP/HTTP, the exporter appends per-signal paths such as `/v1/traces`, `/v1/metrics`, and `/v1/logs`.
- With per-signal variables such as `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, the URL is used as-is.
- `OTEL_EXPORTER_OTLP_HEADERS` is the standard place for backend auth headers.
- Certificate, compression, timeout, and per-signal overrides are also available through OTLP exporter environment variables.

## Logs

Python logs support is still in development, so keep log-specific code conservative and version-aware.

Current basic setup:

```python
import logging

from opentelemetry._logs import set_logger_provider
from opentelemetry.sdk._logs import LoggerProvider, LoggingHandler
from opentelemetry.sdk._logs.export import (
    BatchLogRecordProcessor,
    ConsoleLogRecordExporter,
)
from opentelemetry.sdk.resources import Resource

logger_provider = LoggerProvider(
    resource=Resource.create({"service.name": "checkout-api"})
)
logger_provider.add_log_record_processor(
    BatchLogRecordProcessor(ConsoleLogRecordExporter())
)
set_logger_provider(logger_provider)

handler = LoggingHandler(level=logging.INFO, logger_provider=logger_provider)
logging.basicConfig(handlers=[handler], level=logging.INFO)

logging.getLogger(__name__).info("payment accepted", extra={"order_id": "ord_123"})
```

Use logs only if you specifically need them. Traces and metrics have a much more stable Python surface today.

## Auto-Instrumentation And Third-Party Libraries

Two rules matter:

- If you are instrumenting your application, install the SDK and then add instrumentation packages for frameworks and clients you use.
- If you are writing a reusable library, do not take a hard dependency on `opentelemetry-sdk`; depend on `opentelemetry-api` only.

Typical application add-ons:

```bash
python -m pip install "opentelemetry-instrumentation-fastapi==0.61b0"
python -m pip install "opentelemetry-instrumentation-sqlalchemy==0.61b0"
python -m pip install "opentelemetry-instrumentation-requests==0.61b0"
```

## Common Pitfalls

- Forgetting to set `service.name`. Telemetry without a stable service identity is much harder to use.
- Installing only `opentelemetry-sdk` and expecting OTLP export to work without an OTLP exporter package.
- Initializing providers multiple times. The process-wide tracer and meter providers should normally be set once at startup.
- Using console exporters in production. Send data to a Collector instead.
- Ending the process immediately after emitting spans or metrics. Call `force_flush()` or `shutdown()` in short-lived processes.
- Treating logs as equally stable with traces and metrics in Python. They are not.
- Putting `opentelemetry-sdk` into a reusable library dependency tree. That makes SDK selection harder for the final application.
- Mixing incompatible package series. Keep `opentelemetry-api`, `opentelemetry-sdk`, OTLP exporters, and semantic-convention-sensitive integrations aligned to the same release family when possible.
- Copying old logging examples that use pre-1.39 log class names.

## Version-Sensitive Notes For 1.40.0

- PyPI lists `1.40.0` as the latest `opentelemetry-sdk` release on March 12, 2026, released on March 4, 2026.
- Python support in the official docs is `3.9+`, and the `1.40.0` changelog adds Python `3.14` support.
- The `1.40.0` changelog deprecates `LoggingHandler` in favor of `opentelemetry-instrumentation-logging`. Existing `LoggingHandler` examples still work for now, but prefer newer logging guidance when wiring fresh code.
- The `1.39.0` line made breaking log-signal renames from `Log` to `LogRecord` types. If a snippet uses names such as `ConsoleLogExporter`, it is from an older package line; current code should use `ConsoleLogRecordExporter`.
- The `1.40.0` changelog also drops old Jaeger exporter environment variables that were already obsolete after the Jaeger exporter removal in `1.22.0`. Prefer OTLP-based export paths.

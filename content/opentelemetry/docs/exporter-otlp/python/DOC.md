---
name: exporter-otlp
description: "OpenTelemetry OTLP exporter package for sending Python traces, metrics, and logs to an OpenTelemetry Collector or OTLP backend"
metadata:
  languages: "python"
  versions: "1.40.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opentelemetry,otel,otlp,observability,tracing,metrics,logging,collector"
---

# opentelemetry-exporter-otlp Python Package Guide

## Golden Rule

`opentelemetry-exporter-otlp` is the convenience meta-package. Install it when you want both OTLP transports available, but write code against the concrete HTTP or gRPC exporter modules. The package does not create providers, processors, or resources for you; you still need `opentelemetry-sdk` setup, and OTLP is usually sent to an OpenTelemetry Collector or a vendor endpoint that speaks OTLP.

## Install

If you want the convenience bundle:

```bash
python -m pip install "opentelemetry-exporter-otlp==1.40.0" "opentelemetry-sdk==1.40.0"
```

If you already know the transport, prefer the smaller concrete package:

```bash
python -m pip install "opentelemetry-exporter-otlp-proto-http==1.40.0" "opentelemetry-sdk==1.40.0"
python -m pip install "opentelemetry-exporter-otlp-proto-grpc==1.40.0" "opentelemetry-sdk==1.40.0"
```

Common alternatives:

```bash
uv add "opentelemetry-exporter-otlp==1.40.0" "opentelemetry-sdk==1.40.0"
poetry add "opentelemetry-exporter-otlp==1.40.0" "opentelemetry-sdk==1.40.0"
```

If you are using auto-instrumentation instead of manual SDK setup, keep in mind that the instrumentation release train is separate. For the `1.40.0` core release, the matching contrib line is `0.61b0`.

## Choose A Transport First

Use HTTP/protobuf when:

- your backend documents OTLP/HTTP endpoints
- you want explicit per-signal URLs such as `/v1/traces`
- proxies and load balancers behave better with HTTP than long-lived gRPC

Use gRPC when:

- your backend or collector expects OTLP/gRPC
- you want a single endpoint instead of signal-specific URLs
- your environment already handles gRPC/TLS well

The OpenTelemetry OTLP defaults are:

- HTTP base endpoint: `http://localhost:4318`
- gRPC endpoint: `http://localhost:4317`

## Minimal Manual Setup

All signals should share a `Resource` so telemetry carries the same service identity.

```python
from opentelemetry.sdk.resources import Resource

resource = Resource.create(
    {
        "service.name": "billing-api",
        "service.version": "2026.03.12",
        "deployment.environment": "dev",
    }
)
```

### Traces Over OTLP/HTTP

```python
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

resource = Resource.create({"service.name": "billing-api"})

provider = TracerProvider(resource=resource)
provider.add_span_processor(
    BatchSpanProcessor(
        OTLPSpanExporter(
            endpoint="http://localhost:4318/v1/traces",
            headers={"authorization": "Bearer <token>"},
            timeout=10,
        )
    )
)
trace.set_tracer_provider(provider)

tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("checkout"):
    pass
```

### Traces Over OTLP/gRPC

For gRPC, use the gRPC exporter import and point it at the collector/backend endpoint without `/v1/traces`:

```python
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

resource = Resource.create({"service.name": "billing-api"})

provider = TracerProvider(resource=resource)
provider.add_span_processor(
    BatchSpanProcessor(
        OTLPSpanExporter(
            endpoint="http://localhost:4317",
            insecure=True,
            headers={"authorization": "Bearer <token>"},
            timeout=10,
        )
    )
)
trace.set_tracer_provider(provider)
```

Notes:

- `insecure=True` is for plain local or trusted network gRPC. For TLS, use an `https://` endpoint or pass credentials instead.
- In `1.40.0`, OTLP exporter `headers` also accept a string as well as a dict, which helps when reusing env-style header values.

### Metrics Over OTLP/HTTP

```python
from opentelemetry import metrics
from opentelemetry.exporter.otlp.proto.http.metric_exporter import OTLPMetricExporter
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.sdk.resources import Resource

resource = Resource.create({"service.name": "billing-api"})

metric_reader = PeriodicExportingMetricReader(
    OTLPMetricExporter(
        endpoint="http://localhost:4318/v1/metrics",
        headers={"authorization": "Bearer <token>"},
        timeout=10,
    ),
    export_interval_millis=5000,
)

provider = MeterProvider(resource=resource, metric_readers=[metric_reader])
metrics.set_meter_provider(provider)

meter = metrics.get_meter(__name__)
requests_counter = meter.create_counter("http.server.requests")
requests_counter.add(1, {"route": "/checkout", "method": "POST"})
```

### Logs

The OTLP package includes `OTLPLogExporter` for HTTP and gRPC, but log setup in Python is still the least stable signal. The reliable shape is:

- create a `LoggerProvider`
- attach `BatchLogRecordProcessor`
- attach `OTLPLogExporter`
- bridge application logging carefully

For `1.40.0`, treat log examples as version-sensitive and verify them against the current upstream logging docs before copying older snippets. The `1.40.0` release notes also deprecate `LoggingHandler`, so do not anchor new code on it unless you have confirmed the current replacement path in the docs for your stack.

## Auto-Instrumentation

For zero-code or low-code instrumentation, the exporter is usually configured by environment variables instead of constructor arguments:

```bash
python -m pip install "opentelemetry-distro==0.61b0" "opentelemetry-exporter-otlp==1.40.0"
opentelemetry-bootstrap -a install

export OTEL_SERVICE_NAME="billing-api"
export OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"

opentelemetry-instrument python app.py
```

When using OTLP/HTTP with the base endpoint variable above, the SDK builds signal-specific URLs automatically:

- traces -> `/v1/traces`
- metrics -> `/v1/metrics`
- logs -> `/v1/logs`

If you set `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`, or `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT`, those values are already full per-signal endpoints.

## Configuration And Authentication

Most real backends use one or more of these patterns:

- Authorization or tenant headers
- TLS certificates
- Separate per-signal endpoints
- A collector in front of the vendor backend

Useful environment variables:

- `OTEL_EXPORTER_OTLP_PROTOCOL`: `grpc` or `http/protobuf`
- `OTEL_EXPORTER_OTLP_ENDPOINT`: one base endpoint for all signals
- `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`
- `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`
- `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT`
- `OTEL_EXPORTER_OTLP_HEADERS`: comma-separated key/value pairs for auth or tenancy
- `OTEL_EXPORTER_OTLP_TIMEOUT`
- `OTEL_EXPORTER_OTLP_COMPRESSION`

Signal-specific variants exist too, such as `OTEL_EXPORTER_OTLP_TRACES_HEADERS` and `OTEL_EXPORTER_OTLP_METRICS_TIMEOUT`.

Typical header-based auth:

```bash
export OTEL_EXPORTER_OTLP_HEADERS="authorization=Bearer <token>,x-tenant-id=<tenant>"
```

HTTP exporters also support certificate and client-cert file arguments. gRPC exporters support channel credentials and TLS-aware endpoints.

## Common Pitfalls

- Installing the meta-package does not mean you import `opentelemetry_exporter_otlp` directly. Import the concrete HTTP or gRPC exporter modules.
- OTLP exporter setup alone is not enough. Without `TracerProvider`, `MeterProvider`, processors, and a `Resource`, you will not emit useful telemetry.
- Missing `service.name` is one of the fastest ways to get hard-to-find telemetry in the backend.
- OTLP/HTTP base endpoints and per-signal endpoints behave differently. `OTEL_EXPORTER_OTLP_ENDPOINT=http://host:4318` is correct; `OTEL_EXPORTER_OTLP_ENDPOINT=http://host:4318/v1/traces` is wrong if you expect metrics and logs to derive cleanly from the same base.
- gRPC endpoints do not use `/v1/traces`, `/v1/metrics`, or `/v1/logs`.
- Vendor backends often require headers even when the collector on localhost does not. Local success does not prove production auth is correct.
- Batch processors export asynchronously. Short-lived scripts should call provider shutdown or force-flush paths before exiting.
- Logs remain the most version-sensitive signal in Python OpenTelemetry. Re-check upstream examples before pasting older log bridge code.

## Version-Sensitive Notes For 1.40.0

- PyPI currently lists `1.40.0` for `opentelemetry-exporter-otlp`, so the version used here is current as of March 12, 2026.
- The package is production/stable on PyPI and advertises Python `3.9` through `3.14`.
- The `1.40.0` release train for the core Python packages is paired with contrib/instrumentation `0.61b0`.
- The release notes for `1.40.0` call out ongoing logging changes, including deprecation of `LoggingHandler`; treat log integrations as higher-churn than traces and metrics.

## Official Sources

- OpenTelemetry Python exporter docs: `https://opentelemetry-python.readthedocs.io/en/latest/exporter/otlp/otlp.html`
- OpenTelemetry Python exporter guide: `https://opentelemetry.io/docs/languages/python/exporters/`
- OTLP exporter configuration: `https://opentelemetry.io/docs/languages/sdk-configuration/otlp-exporter/`
- OTLP protocol exporter spec: `https://opentelemetry.io/docs/specs/otel/protocol/exporter/`
- PyPI package page: `https://pypi.org/project/opentelemetry-exporter-otlp/`
- PyPI JSON metadata: `https://pypi.org/pypi/opentelemetry-exporter-otlp/json`

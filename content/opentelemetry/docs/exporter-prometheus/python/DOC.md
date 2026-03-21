---
name: exporter-prometheus
description: "OpenTelemetry Prometheus exporter for Python applications that expose scraped metrics endpoints"
metadata:
  languages: "python"
  versions: "0.61b0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "opentelemetry,prometheus,metrics,observability,monitoring"
---

# opentelemetry-exporter-prometheus Python Package Guide

## Golden Rule

Use `opentelemetry-exporter-prometheus` when your Python process must expose a Prometheus scrape endpoint itself. If you already run an OpenTelemetry Collector or Prometheus with the OTLP receiver enabled, OpenTelemetry recommends OTLP for production because it preserves more of the OTel data model and fits Collector-based deployments better.

## Install

Pin the exporter version your project expects:

```bash
python -m pip install "opentelemetry-exporter-prometheus==0.61b0"
```

Common alternatives:

```bash
uv add "opentelemetry-exporter-prometheus==0.61b0"
poetry add "opentelemetry-exporter-prometheus==0.61b0"
```

What gets installed with it:

- `opentelemetry-sdk`
- `opentelemetry-api`
- `prometheus_client`

Do not try to force the exporter package version to match the SDK numerically. In this release train, the exporter is `0.61b0` while the package metadata depends on the `1.x` OpenTelemetry API/SDK line.

## Initialize The Exporter

This exporter is a `MetricReader`. Your app still needs a normal OpenTelemetry `MeterProvider`, a `Resource`, and metric instruments.

```python
from prometheus_client import start_http_server

from opentelemetry import metrics
from opentelemetry.exporter.prometheus import PrometheusMetricReader
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.resources import SERVICE_NAME, Resource

resource = Resource.create(
    {
        SERVICE_NAME: "checkout-api",
        "deployment.environment": "dev",
    }
)

start_http_server(port=9464, addr="localhost")

reader = PrometheusMetricReader()
provider = MeterProvider(resource=resource, metric_readers=[reader])
metrics.set_meter_provider(provider)

meter = metrics.get_meter("checkout-api")
request_counter = meter.create_counter(
    "http.server.requests",
    unit="1",
    description="HTTP requests handled",
)
latency = meter.create_histogram(
    "http.server.duration",
    unit="ms",
    description="HTTP request latency",
)

request_counter.add(1, {"method": "GET", "route": "/health"})
latency.record(12.7, {"method": "GET", "route": "/health"})
```

After the process starts, Prometheus-format metrics are exposed at `http://localhost:9464/metrics`.

## Scrape From Prometheus

Minimal Prometheus config:

```yaml
scrape_configs:
  - job_name: checkout-api
    scrape_interval: 5s
    static_configs:
      - targets: ["host.docker.internal:9464"]
```

If Prometheus is not in Docker, replace `host.docker.internal` with the actual hostname or IP that can reach the Python process.

## Auto-Instrumented / Zero-Code Setup

The package registers a `prometheus` metrics exporter entry point for OpenTelemetry Python auto-configuration. If you already run your app through `opentelemetry-instrument`, this package can start the scrape server for you.

```bash
python -m pip install "opentelemetry-distro" "opentelemetry-exporter-prometheus==0.61b0"

export OTEL_SERVICE_NAME="checkout-api"
export OTEL_METRICS_EXPORTER="prometheus"
export OTEL_EXPORTER_PROMETHEUS_HOST="0.0.0.0"
export OTEL_EXPORTER_PROMETHEUS_PORT="9464"

opentelemetry-instrument python app.py
```

Use this path only when you are already relying on the OpenTelemetry auto-instrumentation tooling. For normal code-based instrumentation, instantiate `PrometheusMetricReader()` yourself and call `start_http_server(...)` explicitly.

## Configuration And Exposure

Supported Prometheus-specific environment variables:

- `OTEL_EXPORTER_PROMETHEUS_HOST`: bind host, default `localhost`
- `OTEL_EXPORTER_PROMETHEUS_PORT`: bind port, default `9464`

Authentication and transport:

- This exporter does not add authentication, TLS, or authorization to the metrics endpoint.
- Treat the endpoint as internal infrastructure.
- Bind to `localhost` in development, or put the endpoint behind a trusted reverse proxy, service mesh, or private network path in shared environments.

## Common Pitfalls

- Do not copy the older `PrometheusMetricReader(prefix)` example from the docs page. In `0.61b0`, the actual constructor is `PrometheusMetricReader(disable_target_info: bool = False)`.
- Do not start two scrape servers. If auto-instrumentation starts the HTTP server for you, do not also call `start_http_server(...)` in application code.
- This package is pull-based. Nothing is pushed anywhere; Prometheus or a Collector with a Prometheus receiver must scrape `/metrics`.
- No multiprocessing support. The official docs explicitly call this out; do not assume Gunicorn-style multi-worker setups will aggregate metrics correctly through this exporter.
- Set a service name in the `Resource`. OpenTelemetry’s Python exporter docs note that a service name is required for most backends.
- Keep label cardinality under control. Prometheus labels created from metric attributes can explode in size if you include request IDs, user IDs, or other unbounded values.

## Version-Sensitive Notes

- `0.61b0` is a pre-release version from the OpenTelemetry Python beta exporter line. Pin it deliberately; do not assume it behaves like a stable `1.x` package.
- The package metadata for this release requires Python `>=3.9`.
- The current Read the Docs page mixes an older `prefix` example with the newer API signature. Prefer the API signature and source code over that stale example.
- If your deployment already has an OpenTelemetry Collector, OTLP is usually the better long-term production path. Use this Prometheus exporter when you specifically need a scrape endpoint from the app process.

## Official Sources

- Prometheus exporter docs: `https://opentelemetry-python.readthedocs.io/en/latest/exporter/prometheus/prometheus.html`
- OpenTelemetry Python exporters guide: `https://opentelemetry.io/docs/languages/python/exporters/`
- OpenTelemetry Python zero-code guide: `https://opentelemetry.io/docs/zero-code/python/`
- PyPI package page: `https://pypi.org/project/opentelemetry-exporter-prometheus/`
- Upstream package metadata: `https://raw.githubusercontent.com/open-telemetry/opentelemetry-python/main/exporter/opentelemetry-exporter-prometheus/pyproject.toml`
- Upstream exporter source: `https://raw.githubusercontent.com/open-telemetry/opentelemetry-python/main/exporter/opentelemetry-exporter-prometheus/src/opentelemetry/exporter/prometheus/__init__.py`

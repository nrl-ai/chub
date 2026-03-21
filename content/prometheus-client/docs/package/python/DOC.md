---
name: package
description: "prometheus-client Python package for instrumenting code, exposing Prometheus metrics, and handling ASGI/WSGI, multiprocess, and Pushgateway workflows"
metadata:
  languages: "python"
  versions: "0.24.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "prometheus,monitoring,metrics,instrumentation,observability,python"
---

# prometheus-client Python Package Guide

## Golden Rule

Use `prometheus-client` as the package name, import it as `prometheus_client`, and keep instrumentation simple: create metrics once at module scope, expose them on `/metrics`, and switch to the documented multiprocess pattern before running under multiple worker processes. For Pushgateway and batch jobs, use a dedicated `CollectorRegistry` rather than the default global registry.

## Install

Pin the version your project expects:

```bash
python -m pip install "prometheus-client==0.24.1"
```

Common alternatives:

```bash
uv add "prometheus-client==0.24.1"
poetry add "prometheus-client==0.24.1"
```

Optional extras published on PyPI:

```bash
python -m pip install "prometheus-client[aiohttp]==0.24.1"
python -m pip install "prometheus-client[django]==0.24.1"
python -m pip install "prometheus-client[twisted]==0.24.1"
```

## Quick Start

The simplest pattern is the library quickstart: define metrics once, start the HTTP exporter, then update metrics from application code.

```python
from prometheus_client import Summary, start_http_server
import random
import time

REQUEST_TIME = Summary(
    "request_processing_seconds",
    "Time spent processing a request",
)

@REQUEST_TIME.time()
def process_request(delay_seconds: float) -> None:
    time.sleep(delay_seconds)

if __name__ == "__main__":
    start_http_server(8000)
    while True:
        process_request(random.random())
```

Prometheus can then scrape `http://localhost:8000/`.

## Core Metric Types

### Counter

Use `Counter` for values that only increase.

```python
from prometheus_client import Counter

REQUESTS = Counter(
    "myapp_requests_total",
    "Total requests handled",
    ["method", "endpoint"],
)

REQUESTS.labels("GET", "/health").inc()
REQUESTS.labels("POST", "/jobs").inc()
```

Useful helper:

```python
from prometheus_client import Counter

FAILURES = Counter("myapp_failures_total", "Unhandled failures")

@FAILURES.count_exceptions()
def run_job() -> None:
    raise RuntimeError("boom")
```

### Gauge

Use `Gauge` for values that can go up or down.

```python
from prometheus_client import Gauge

IN_PROGRESS = Gauge("myapp_inprogress_requests", "Requests currently in progress")

@IN_PROGRESS.track_inprogress()
def handle_request() -> None:
    ...
```

For callback-driven gauges:

```python
from prometheus_client import Gauge

QUEUE_SIZE = Gauge("myapp_queue_size", "Items waiting in memory")
queue = []
QUEUE_SIZE.set_function(lambda: len(queue))
```

### Histogram

Use `Histogram` for latency or size distributions that you want to aggregate across instances.

```python
from prometheus_client import Histogram

LATENCY = Histogram(
    "myapp_request_latency_seconds",
    "Request latency",
    buckets=(0.01, 0.05, 0.1, 0.25, 0.5, 1, 2, 5, 10),
)

@LATENCY.time()
def query_backend() -> None:
    ...
```

### Summary

Use `Summary` when you want count and sum in-process and do not need histogram buckets.

```python
from prometheus_client import Summary

REQUEST_SIZE = Summary("myapp_request_size_bytes", "Request payload size")
REQUEST_SIZE.observe(512)
```

### Info and Enum

These exist and are useful for build metadata or state-style values:

```python
from prometheus_client import Enum, Info

BUILD = Info("myapp_build", "Build metadata")
BUILD.info({"version": "1.2.3", "git_sha": "abc123"})

STATE = Enum("myapp_state", "Worker state", states=["starting", "running", "stopped"])
STATE.state("running")
```

Do not use `Info` or `Enum` in multiprocess mode.

## Labels And Registry Basics

Labeled metrics are not initialized until you call `.labels(...)`. If you expect a fixed set of label values, pre-create them during startup so your dashboards do not look empty until the first request.

```python
from prometheus_client import Counter

REQUESTS = Counter(
    "myapp_requests_total",
    "Requests handled",
    ["method", "endpoint"],
)

for method, endpoint in [
    ("GET", "/health"),
    ("POST", "/jobs"),
]:
    REQUESTS.labels(method, endpoint)
```

The library uses the default global `REGISTRY` unless you pass `registry=...`. Keep using the default registry for long-running services unless you have a concrete reason to isolate metrics. Use a separate `CollectorRegistry()` for batch jobs, Pushgateway pushes, tests, or custom exporter endpoints.

## Configuration And Auth

Key environment-driven settings:

- `PROMETHEUS_MULTIPROC_DIR=/path/to/dir`: required for multiprocess mode; set it before Python starts and wipe the directory between runs.
- `PROMETHEUS_DISABLE_CREATED_SERIES=True`: disables `_created` time series for counters, histograms, and summaries if you do not want them.

Equivalent runtime call for `_created` series:

```python
from prometheus_client import disable_created_metrics

disable_created_metrics()
```

Authentication is not built into the plain `/metrics` endpoint by default. If you need auth or network restrictions, enforce them in your ASGI/WSGI app, reverse proxy, ingress, or by enabling HTTPS or mTLS on the standalone exporter.

## Export Metrics

### Standalone HTTP or HTTPS exporter

`start_http_server()` is the easiest way to expose metrics from a process:

```python
from prometheus_client import Counter, start_http_server

REQUESTS = Counter("myapp_requests_total", "Requests handled")

server, thread = start_http_server(8000)

REQUESTS.inc()

# Later, for graceful shutdown:
# server.shutdown()
# server.server_close()
# thread.join()
```

For TLS:

```python
from prometheus_client import start_http_server

start_http_server(
    8443,
    certfile="server.crt",
    keyfile="server.key",
)
```

The exporter only serves `GET` and `OPTIONS`; other methods are rejected.

### Mount into ASGI apps

Use `make_asgi_app()` for FastAPI, Starlette, or other ASGI apps:

```python
from fastapi import FastAPI
from prometheus_client import make_asgi_app

app = FastAPI()
app.mount("/metrics", make_asgi_app())
```

Compression is enabled when the scraper sends `Accept-Encoding: gzip`. Disable it only if you have a reason:

```python
metrics_app = make_asgi_app(disable_compression=True)
```

### Mount into WSGI apps

Use `make_wsgi_app()` with Flask or other WSGI frameworks:

```python
from flask import Flask
from prometheus_client import make_wsgi_app
from werkzeug.middleware.dispatcher import DispatcherMiddleware

app = Flask(__name__)
app.wsgi_app = DispatcherMiddleware(app.wsgi_app, {
    "/metrics": make_wsgi_app(),
})
```

If you just want a thread-backed WSGI exporter, `start_wsgi_server(8000)` is also available.

### Custom endpoint with `generate_latest`

If you need full control over the response path:

```python
from prometheus_client import CONTENT_TYPE_LATEST, Counter, generate_latest
from starlette.responses import Response

REQUESTS = Counter("myapp_requests_total", "Requests handled")

async def metrics_endpoint(request) -> Response:
    return Response(generate_latest(), media_type=CONTENT_TYPE_LATEST)
```

## Pushgateway For Batch Jobs

Use Pushgateway only for short-lived or batch jobs that Prometheus cannot scrape directly. Use a separate registry so you do not accidentally push process-level defaults from the global registry.

```python
from prometheus_client import CollectorRegistry, Gauge, push_to_gateway

registry = CollectorRegistry()
last_success = Gauge(
    "job_last_success_unixtime",
    "Last successful completion timestamp",
    registry=registry,
)

last_success.set_to_current_time()
push_to_gateway("pushgateway.internal:9091", job="nightly-report", registry=registry)
```

Authentication handlers are built in:

```python
from prometheus_client import CollectorRegistry, Gauge, push_to_gateway
from prometheus_client.exposition import basic_auth_handler

def auth_handler(url, method, timeout, headers, data):
    return basic_auth_handler(
        url,
        method,
        timeout,
        headers,
        data,
        username="metrics",
        password="secret123",
    )

registry = CollectorRegistry()
Gauge("job_last_success_unixtime", "Last success time", registry=registry).set_to_current_time()
push_to_gateway(
    "pushgateway.internal:9091",
    job="nightly-report",
    registry=registry,
    handler=auth_handler,
    compression="gzip",
)
```

TLS auth handlers are also available. In multiprocess mode, Pushgateway is not supported.

## Multiprocess Mode

Prometheus Python clients assume shared in-process metrics by default. That breaks under pre-fork worker models such as Gunicorn with multiple workers. When you run multiple processes, switch to the documented multiprocess flow.

### Required environment

Set `PROMETHEUS_MULTIPROC_DIR` in the startup environment, and wipe that directory between runs:

```bash
export PROMETHEUS_MULTIPROC_DIR=/tmp/prometheus-multiproc
rm -rf "$PROMETHEUS_MULTIPROC_DIR"
mkdir -p "$PROMETHEUS_MULTIPROC_DIR"
```

Set the variable in the shell or process manager before Python starts so child workers inherit it.

### Build the registry per scrape

```python
from prometheus_client import CollectorRegistry, CONTENT_TYPE_LATEST, generate_latest, multiprocess
from starlette.responses import Response

async def metrics_endpoint(request) -> Response:
    registry = CollectorRegistry()
    multiprocess.MultiProcessCollector(registry)
    data = generate_latest(registry)
    return Response(data, media_type=CONTENT_TYPE_LATEST)
```

### Gunicorn cleanup hook

```python
from prometheus_client import multiprocess

def child_exit(server, worker):
    multiprocess.mark_process_dead(worker.pid)
```

### Gauge tuning

Pick a `multiprocess_mode` explicitly for gauges so you get the aggregation behavior you want:

```python
from prometheus_client import Gauge

IN_PROGRESS = Gauge(
    "myapp_inprogress_requests",
    "Requests in progress",
    multiprocess_mode="livesum",
)
```

Useful modes include `sum`, `max`, `mostrecent`, and the `live...` variants.

## Default Collectors

The default registry includes collector metrics by default:

- `process_*` metrics for CPU, memory, file descriptors, and start time
- `python_info`
- garbage-collection metrics

Important caveat: `process_*` metrics are only available on Linux.

If these defaults are not appropriate for a service, unregister them explicitly:

```python
import prometheus_client

prometheus_client.REGISTRY.unregister(prometheus_client.GC_COLLECTOR)
prometheus_client.REGISTRY.unregister(prometheus_client.PLATFORM_COLLECTOR)
prometheus_client.REGISTRY.unregister(prometheus_client.PROCESS_COLLECTOR)
```

## Advanced Features

### Exemplars

Counters and histograms support exemplars:

```python
from prometheus_client import Counter

REQUESTS = Counter("myapp_requests_total", "Requests handled", ["path"])
REQUESTS.labels("/checkout").inc(exemplar={"trace_id": "abc123"})
```

Exemplars are only emitted in OpenMetrics exposition, not plain Prometheus text exposition.

### Parse Prometheus text format

For advanced ingestion or bridging workflows:

```python
from prometheus_client.parser import text_string_to_metric_families

payload = "my_gauge 1.0\n"

for family in text_string_to_metric_families(payload):
    for sample in family.samples:
        print(sample)
```

### Custom collectors

Use custom collectors when you need to proxy or transform metrics from another source:

```python
from prometheus_client.core import GaugeMetricFamily, REGISTRY

class QueueCollector:
    def collect(self):
        yield GaugeMetricFamily("external_queue_depth", "Queue depth", value=7)

REGISTRY.register(QueueCollector())
```

Custom collectors do not work in multiprocess mode.

## Common Pitfalls

- The package name is `prometheus-client`, but the import path is `prometheus_client`.
- If you create a `Counter` named with a `_total` suffix, the client normalizes it internally and exports it with `_total` for compatibility.
- Labeled metrics do not exist until `.labels(...)` is called for a given label set.
- Use `Histogram` instead of `Summary` when you need aggregation across multiple instances or workers.
- Use a dedicated `CollectorRegistry()` for Pushgateway jobs instead of pushing the default registry.
- `Gauge.set_function()`, `Info`, `Enum`, exemplars, custom collectors, and label `remove`/`clear` are not supported in multiprocess mode.
- Do not reuse a registry that already has registered metrics when wrapping it with `multiprocess.MultiProcessCollector`, or you can export duplicates.
- The default `process_*` collectors are Linux-only, so code and dashboards should tolerate them being absent on macOS or Windows.
- The built-in HTTP exporter rejects methods other than `GET` and `OPTIONS`.

## Version-Sensitive Notes

- PyPI lists `prometheus-client 0.24.1` as the current release on January 14, 2026.
- PyPI metadata for `0.24.1` requires Python `>=3.9`; Python 3.8 support was removed in `0.22.0`.
- PyPI marks `0.23.0` as yanked because it accidentally included an unlisted dependency. Prefer `0.23.1+` if you encounter older pins.
- `0.20.0` added graceful shutdown support by returning the server and thread objects from `start_http_server()` and `start_wsgi_server()`.
- `0.22.0` added UTF-8 metric support and parser/exposition improvements, so very old examples that assume ASCII-only names or labels are stale.

## Official Sources

- Docs root: `https://prometheus.github.io/client_python/`
- Instrumenting guide: `https://prometheus.github.io/client_python/instrumenting/`
- Labels: `https://prometheus.github.io/client_python/instrumenting/labels/`
- Counter: `https://prometheus.github.io/client_python/instrumenting/counter/`
- Gauge: `https://prometheus.github.io/client_python/instrumenting/gauge/`
- Histogram: `https://prometheus.github.io/client_python/instrumenting/histogram/`
- Exemplars: `https://prometheus.github.io/client_python/instrumenting/exemplars/`
- Collector guide: `https://prometheus.github.io/client_python/collector/`
- Custom collectors: `https://prometheus.github.io/client_python/collector/custom/`
- HTTP/HTTPS exporter: `https://prometheus.github.io/client_python/exporting/http/`
- WSGI exporter: `https://prometheus.github.io/client_python/exporting/http/wsgi/`
- ASGI exporter: `https://prometheus.github.io/client_python/exporting/http/asgi/`
- FastAPI + Gunicorn: `https://prometheus.github.io/client_python/exporting/http/fastapi-gunicorn/`
- Pushgateway: `https://prometheus.github.io/client_python/exporting/pushgateway/`
- Multiprocess mode: `https://prometheus.github.io/client_python/multiprocess/`
- Parser: `https://prometheus.github.io/client_python/parser/`
- PyPI package metadata and release history: `https://pypi.org/project/prometheus-client/`
- GitHub releases: `https://github.com/prometheus/client_python/releases`

---
name: monitor-opentelemetry
description: "Azure Monitor OpenTelemetry distro for Python package guide"
metadata:
  languages: "python"
  versions: "1.8.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-monitor,application-insights,opentelemetry,observability,python"
---

# azure-monitor-opentelemetry Python Package Guide

## Golden Rule

Call `configure_azure_monitor()` once during process startup, before your app begins serving requests or creating SDK clients, and let the distro own the Azure Monitor exporter and bundled OpenTelemetry instrumentations.

This guide is pinned to `azure-monitor-opentelemetry==1.8.6`, which matches the version used here and the current Learn/PyPI package version as of `2026-03-12`.

## What This Package Does

`azure-monitor-opentelemetry` is Microsoft's Python distro for sending traces, metrics, logs, and bundled dependency telemetry to Azure Monitor / Application Insights with one startup call.

Bundled instrumentations enabled by default:

- `azure_sdk`
- `django`
- `fastapi`
- `flask`
- `psycopg2`
- `requests`
- `urllib`
- `urllib3`

Important boundary:

- Use `configure_azure_monitor()` for manual application bootstrap.
- Automatic instrumentation is not supported by this package itself.
- You can still manually instrument unsupported libraries through normal OpenTelemetry APIs.

## Install

Pin the package if you want behavior that matches this guide:

```bash
pip install "azure-monitor-opentelemetry==1.8.6"
```

If you want Microsoft Entra ID authentication, also install Azure Identity:

```bash
pip install azure-identity
```

Set the Application Insights connection string before startup:

```bash
export APPLICATIONINSIGHTS_CONNECTION_STRING="InstrumentationKey=...;IngestionEndpoint=https://..."
export OTEL_SERVICE_NAME="billing-api"
export OTEL_RESOURCE_ATTRIBUTES="service.namespace=payments,service.instance.id=worker-1"
```

## Minimal Setup

```python
import logging
import os

from azure.monitor.opentelemetry import configure_azure_monitor
from opentelemetry import metrics, trace

logging.basicConfig(level=logging.INFO)

configure_azure_monitor(
    connection_string=os.environ["APPLICATIONINSIGHTS_CONNECTION_STRING"],
    logger_name="app.telemetry",
)

tracer = trace.get_tracer("billing-api")
meter = metrics.get_meter("billing-api")
jobs_counter = meter.create_counter("jobs_processed")
logger = logging.getLogger("app.telemetry")

with tracer.start_as_current_span("sync-job") as span:
    span.set_attribute("job.name", "sync")
    logger.info("sync started", extra={"component": "worker"})
    jobs_counter.add(1, {"job": "sync"})
```

What this gives you:

- Azure Monitor export for traces, metrics, and logs
- auto-instrumentation for supported libraries used after startup
- custom spans through the OpenTelemetry tracing API
- custom metrics through the OpenTelemetry metrics API
- log records and custom event emission through Python logging

## Initialization Rules

- Call `configure_azure_monitor()` only once per process.
- Call it before creating framework apps, Azure SDK clients, HTTP clients, or database connections you expect to be auto-instrumented.
- Keep telemetry bootstrap in a startup module, not inside request handlers, Celery tasks, or function bodies that run repeatedly.
- If you need custom processors, views, or resource metadata, pass them during the initial call instead of mutating providers later.

## Configuration And Auth

### Connection string and resource identity

`connection_string` is the standard way to point the distro at an Application Insights resource. If you omit it in code, the package reads `APPLICATIONINSIGHTS_CONNECTION_STRING`.

Use resource metadata so Azure Monitor displays useful service names and instances:

```python
import os

from azure.monitor.opentelemetry import configure_azure_monitor
from opentelemetry.sdk.resources import Resource

configure_azure_monitor(
    connection_string=os.environ["APPLICATIONINSIGHTS_CONNECTION_STRING"],
    resource=Resource.create(
        {
            "service.name": "billing-api",
            "service.namespace": "payments",
            "service.instance.id": os.getenv("HOSTNAME", "local"),
        }
    ),
)
```

### Microsoft Entra ID authentication

The Azure Monitor auth guide documents Python examples that pass an Azure Identity credential to `configure_azure_monitor()`.

Managed identity example:

```python
import os

from azure.identity import ManagedIdentityCredential
from azure.monitor.opentelemetry import configure_azure_monitor

credential = ManagedIdentityCredential(client_id=os.getenv("AZURE_CLIENT_ID"))

configure_azure_monitor(
    connection_string=os.environ["APPLICATIONINSIGHTS_CONNECTION_STRING"],
    credential=credential,
)
```

Use managed identity in Azure-hosted workloads when possible. The auth guide also shows `ClientSecretCredential` for service principal flows.

### Signal, sampling, and exporter options

Useful `configure_azure_monitor()` options in `1.8.6`:

- `logger_name`: collect logs from a named Python logger and avoid SDK self-log capture.
- `instrumentation_options`: enable or disable bundled instrumentations selectively.
- `enable_live_metrics`: defaults to `True`.
- `sampling_ratio`: fixed-percentage trace sampling, `0.0..1.0`.
- `traces_per_second`: rate-limited trace sampling.
- `enable_trace_based_sampling_for_logs`: defaults to `True`.
- `span_processors`, `log_record_processors`, `views`, `metric_readers`: advanced OpenTelemetry customization hooks.
- `storage_directory`: override offline retry storage path.
- `disable_offline_storage`: disable disk buffering.

Example with selective instrumentation and rate-limited sampling:

```python
import os

from azure.monitor.opentelemetry import configure_azure_monitor

configure_azure_monitor(
    connection_string=os.environ["APPLICATIONINSIGHTS_CONNECTION_STRING"],
    logger_name="billing.telemetry",
    instrumentation_options={
        "azure_sdk": {"enabled": True},
        "requests": {"enabled": True},
        "urllib3": {"enabled": False},
    },
    traces_per_second=2.0,
    enable_trace_based_sampling_for_logs=True,
    storage_directory="/tmp/azure-monitor",
)
```

Environment variables still matter:

- `OTEL_PYTHON_DISABLED_INSTRUMENTATIONS="requests,urllib3"` disables selected bundled instrumentations.
- `OTEL_TRACES_SAMPLER` and `OTEL_TRACES_SAMPLER_ARG` configure sampler behavior globally.
- `OTEL_LOGS_EXPORTER=None`, `OTEL_METRICS_EXPORTER=None`, or `OTEL_TRACES_EXPORTER=None` disable a signal.
- `OTEL_BLRP_SCHEDULE_DELAY` and `OTEL_BSP_SCHEDULE_DELAY` control log and trace export intervals.
- `OTEL_METRIC_EXPORT_INTERVAL` defaults to `60000` ms.

## Core Usage Patterns

### Manual spans

```python
from opentelemetry import trace
from opentelemetry.trace import SpanKind

tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("reconcile-invoice", kind=SpanKind.SERVER) as span:
    span.set_attribute("invoice.id", invoice_id)
    process_invoice(invoice_id)
```

By default, manual spans appear as in-process dependencies. If a span represents a server-side job or request-like operation that should show up in the `requests` table, set `kind=SpanKind.SERVER`.

### Custom metrics

```python
from opentelemetry import metrics

meter = metrics.get_meter("payments")
counter = meter.create_counter("invoices_processed")
counter.add(1, {"status": "success"})
```

If you want metric namespaces visible in Application Insights Metrics Explorer, Microsoft Learn shows opting in with:

```bash
export APPLICATIONINSIGHTS_METRIC_NAMESPACE_OPT_IN="true"
```

### Custom events and log dimensions

Python logging is autoinstrumented. Use `extra=` to attach custom properties, or set `microsoft.custom_event.name` to emit a custom event:

```python
import logging

logger = logging.getLogger("app.telemetry")

logger.warning(
    "invoice workflow completed",
    extra={
        "microsoft.custom_event.name": "invoice-workflow",
        "invoice.status": "success",
        "tenant.id": tenant_id,
    },
)
```

Normal log enrichment works the same way:

```python
logger.info("payment captured", extra={"order.id": order_id, "amount": amount})
```

### Recording exceptions on spans

```python
from opentelemetry import trace

tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("charge-card") as span:
    try:
        charge_customer()
    except Exception as exc:
        span.record_exception(exc)
        raise
```

## Common Pitfalls

- Do not expect this package to behave like `opentelemetry-instrument`. The package overview explicitly says automatic instrumentation is not supported by the distro itself; bootstrap it in code.
- Do not call `configure_azure_monitor()` repeatedly. Duplicate initialization leads to duplicate telemetry, confusing logging behavior, and harder-to-debug provider state.
- Sampling defaults changed in the current line. In `1.8.6`, if you set neither `sampling_ratio` nor `traces_per_second` nor the sampler environment variables, `configure_azure_monitor()` uses the rate-limited sampler by default with `5.0` traces per second.
- For sampling, environment variables can override code configuration. If `OTEL_TRACES_SAMPLER` or `OTEL_TRACES_SAMPLER_ARG` is set in the runtime environment, verify those values before assuming your `sampling_ratio=` or `traces_per_second=` call is effective.
- Trace-based log sampling is enabled by default. Logs associated with unsampled traces can be dropped even though standalone logs without trace context are unaffected.
- Offline retry storage writes to a temp directory by default. In containers or restricted runtimes, set `storage_directory` explicitly or disable offline storage if disk writes are undesirable.
- Azure Functions has two Python-specific caveats in the official docs: incoming requests are not automatically trace-correlated with worker telemetry, and the Functions worker can emit duplicate logs unless you clear handlers or disable host-side logging before calling `configure_azure_monitor()`.
- The Python version floor is not presented consistently across official pages for `1.8.6`. Validate runtime support before promising Python `3.8`.

## Version-Sensitive Notes

- Version used here: `1.8.6`
- Learn package overview version observed: `1.8.6`
- PyPI latest version observed: `1.8.6`
- PyPI release date observed: `2026-02-04`

Practical consequences for `1.8.6`:

- Current docs and the package page agree on the version.
- The current Learn overview exposes modern kwargs such as `resource`, `span_processors`, `log_record_processors`, `metric_readers`, `sampling_ratio`, and `traces_per_second`.
- The current Azure Monitor configuration guide states that Python `1.8.6` defaults to rate-limited sampling if you do not configure a sampler.
- Older blog posts and earlier package guides for `1.0.x` often describe a smaller configuration surface and older sampling behavior. Re-check official docs before copying those examples.

## Official Sources

- Package overview: https://learn.microsoft.com/en-us/python/api/overview/azure/monitor-opentelemetry-readme?view=azure-python
- Azure Monitor configuration guide: https://learn.microsoft.com/en-us/azure/azure-monitor/app/opentelemetry-configuration
- Azure Monitor add/modify guide: https://learn.microsoft.com/en-us/azure/azure-monitor/app/opentelemetry-add-modify
- Microsoft Entra authentication guide: https://learn.microsoft.com/en-us/azure/azure-monitor/app/azure-ad-authentication
- PyPI project page: https://pypi.org/project/azure-monitor-opentelemetry/
- Version-pinned PyPI JSON: https://pypi.org/pypi/azure-monitor-opentelemetry/1.8.6/json

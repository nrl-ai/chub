---
name: monitoring
description: "Google Cloud Monitoring Python client library for metrics, alerting, uptime checks, and SLO automation"
metadata:
  languages: "python"
  versions: "2.29.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,monitoring,observability,metrics,alerting,slo"
---

# Google Cloud Monitoring Python Client Library

## Golden Rule

Use the official `google-cloud-monitoring` package with `from google.cloud import monitoring_v3`, and authenticate with Application Default Credentials (ADC).

For `2.29.1`, prefer the generated `*ServiceClient` classes such as `MetricServiceClient`, `AlertPolicyServiceClient`, and `NotificationChannelServiceClient`. Avoid old blog posts that import `google.cloud.monitoring` or use `monitoring.Client()` unless you are intentionally maintaining legacy code.

## Install

Pin the package version if you want behavior to match this document exactly:

```bash
python -m pip install "google-cloud-monitoring==2.29.1"
```

If you want the optional dataframe helper from `monitoring_v3.query.Query`, install `pandas` too:

```bash
python -m pip install "google-cloud-monitoring==2.29.1" pandas
```

## Authentication And Project Setup

This library uses Google Cloud credentials, not API keys.

For local development, the usual path is:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
export GOOGLE_CLOUD_PROJECT=YOUR_PROJECT_ID
```

For service-to-service or production workloads, attach a service account to the runtime or set:

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
export GOOGLE_CLOUD_PROJECT=YOUR_PROJECT_ID
```

You also need the Cloud Monitoring API enabled in the target project.

## Initialize Clients

Most work starts with a fully qualified project resource name:

```python
from google.cloud import monitoring_v3

project_id = "your-project-id"
project_name = monitoring_v3.MetricServiceClient.common_project_path(project_id)

metric_client = monitoring_v3.MetricServiceClient()
alert_client = monitoring_v3.AlertPolicyServiceClient()
channel_client = monitoring_v3.NotificationChannelServiceClient()
uptime_client = monitoring_v3.UptimeCheckServiceClient()
service_client = monitoring_v3.ServiceMonitoringServiceClient()
query_client = monitoring_v3.QueryServiceClient()
```

Use the client matching the API surface you need:

- `MetricServiceClient`: read and write metrics/time series, descriptors, groups
- `AlertPolicyServiceClient`: alert policies
- `NotificationChannelServiceClient`: email, Pub/Sub, webhook, PagerDuty, and other channel config
- `UptimeCheckServiceClient`: uptime checks and configs
- `ServiceMonitoringServiceClient`: services and SLOs
- `QueryServiceClient`: Monitoring Query Language (MQL) time-series queries

## Core Usage

### Read time series with `MetricServiceClient`

```python
from datetime import datetime, timedelta, timezone

from google.cloud import monitoring_v3
from google.protobuf.timestamp_pb2 import Timestamp

project_id = "your-project-id"
client = monitoring_v3.MetricServiceClient()
project_name = client.common_project_path(project_id)

end = Timestamp()
end.FromDatetime(datetime.now(tz=timezone.utc))

start = Timestamp()
start.FromDatetime(datetime.now(tz=timezone.utc) - timedelta(minutes=10))

interval = monitoring_v3.TimeInterval(
    {
        "end_time": end,
        "start_time": start,
    }
)

results = client.list_time_series(
    request={
        "name": project_name,
        "filter": 'metric.type = "compute.googleapis.com/instance/cpu/utilization"',
        "interval": interval,
        "view": monitoring_v3.ListTimeSeriesRequest.TimeSeriesView.FULL,
    }
)

for series in results:
    print(series.metric.type)
    print(series.resource.type)
    print(series.metric.labels)
    print(series.resource.labels)
```

Notes:

- `name` is usually `projects/{project_id}`.
- `filter` must match Cloud Monitoring filter syntax exactly; a wrong metric type or resource label returns no data rather than a helpful error.
- Use `HEADERS` view if you only need metadata and want to reduce payload size.

### Write a custom metric with `create_time_series`

```python
import time

from google.cloud import monitoring_v3

project_id = "your-project-id"
client = monitoring_v3.MetricServiceClient()
project_name = client.common_project_path(project_id)

series = monitoring_v3.TimeSeries()
series.metric.type = "custom.googleapis.com/myapp/request_latency_ms"
series.metric.labels["endpoint"] = "/healthz"
series.resource.type = "global"
series.resource.labels["project_id"] = project_id

now = time.time()
seconds = int(now)
nanos = int((now - seconds) * 1_000_000_000)

point = series.points.add()
point.value.double_value = 123.4
point.interval.end_time.seconds = seconds
point.interval.end_time.nanos = nanos

client.create_time_series(name=project_name, time_series=[series])
```

Notes:

- The metric descriptor for a custom metric is created automatically on first write if the metric type is new.
- Your metric kind and value type are inferred from the first writes. Be consistent so later writes do not fail with type mismatches.
- The monitored resource must match the labels required by that resource type. `global` is a simple starting point for app-level custom metrics.

### Use the query helper for simple aggregations

```python
from google.cloud import monitoring_v3
from google.cloud.monitoring_v3.query import Query

project_id = "your-project-id"
metric_client = monitoring_v3.MetricServiceClient()

query = Query(
    metric_client,
    project=project_id,
    metric_type="compute.googleapis.com/instance/cpu/utilization",
    minutes=30,
)

query = query.select_resources(resource_type="gce_instance").align(
    monitoring_v3.Aggregation.Aligner.ALIGN_MEAN,
    minutes=5,
)

for time_series in query:
    print(time_series.metric.labels, len(time_series.points))
```

If you call `query.as_dataframe()`, install `pandas` first.

### List alert policies

```python
from google.cloud import monitoring_v3

project_id = "your-project-id"
client = monitoring_v3.AlertPolicyServiceClient()
project_name = client.common_project_path(project_id)

for policy in client.list_alert_policies(request={"name": project_name}):
    print(policy.name, policy.display_name, policy.enabled)
```

Before creating notification channels, inspect the descriptor for the channel type you want so you know which labels are required:

```python
from google.cloud import monitoring_v3

project_id = "your-project-id"
client = monitoring_v3.NotificationChannelServiceClient()
project_name = client.common_project_path(project_id)

for descriptor in client.list_notification_channel_descriptors(name=project_name):
    print(descriptor.type, descriptor.display_name)
```

## Configuration, Retries, And Runtime Behavior

### Explicit credentials or project

If ADC is not enough, pass explicit credentials:

```python
from google.cloud import monitoring_v3
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = monitoring_v3.MetricServiceClient(credentials=credentials)
```

Most constructors also accept `client_options`, which is where you override the API endpoint or universe domain when you need non-default routing:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import monitoring_v3

client = monitoring_v3.MetricServiceClient(
    client_options=ClientOptions(api_endpoint="monitoring.googleapis.com")
)
```

### Retry and timeout control

RPC methods accept `retry=` and `timeout=` parameters. Use them when a polling loop or a high-volume metrics pipeline needs predictable behavior:

```python
results = client.list_time_series(
    request=request,
    timeout=30,
)
```

### Library logging

Google's Python client libraries support environment-based logging configuration. For debugging client behavior, set a Google logging scope before starting the process:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google.cloud.monitoring_v3
```

### Multiprocessing

Google's generated docs call out a multiprocessing caveat: when you use `multiprocessing` with the gRPC transport, create client instances after `os.fork()`.

## Common Pitfalls

- Package name and import path differ. Install `google-cloud-monitoring`, but import `monitoring_v3`.
- Modern code should use service clients, not the older `monitoring.Client()` examples you may find on historical release pages or blogs.
- Many methods want full resource names such as `projects/{project_id}` or `projects/{project_id}/alertPolicies/{policy_id}`. Use the path helper methods on the client when available.
- `QueryServiceClient` and the `query.Query` helper are for Monitoring Query Language. Google has deprecated MQL for new dashboard and alert creation, so prefer PromQL for new query authoring where the product surface supports it.
- Writing custom metrics fails if the monitored resource labels are incomplete or if later points change value type or metric kind.
- `list_time_series` filter strings are strict. Start with a known metric type from the console or `list_metric_descriptors` output before adding more clauses.
- `query.as_dataframe()` needs `pandas`; it is not bundled by default with this package.
- In long-running ingestion jobs, do not assume the docs alias `/latest` stays pinned to `2.29.1`. Store the package version separately.

## Version-Sensitive Notes For `2.29.1`

- The official docs and PyPI both show `2.29.1` as the current package version on 2026-03-12.
- The upstream changelog lists `2.29.1` on 2026-02-05 as a bugfix release for mypy-related errors.
- The changelog for `2.29.0` notes automatic mTLS enablement when the runtime provides the required certificates and Google APIs use the default host. If endpoint selection matters in your environment, set `client_options` and the related Google API endpoint environment variables explicitly instead of relying on defaults.
- The docs root in Google Cloud uses `/latest`, which is a moving alias. Treat the package version in frontmatter as the actual pin, not the URL path.

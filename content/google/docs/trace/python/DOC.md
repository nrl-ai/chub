---
name: trace
description: "Google Cloud Trace Python client library for reading traces with trace_v1 and writing spans with trace_v2"
metadata:
  languages: "python"
  versions: "1.18.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,cloud-trace,trace,tracing,observability,gcp"
---

# Google Cloud Trace Python Client Library

## Golden Rule

Use `google-cloud-trace` when you need direct Cloud Trace API access from Python.

- Import `trace_v1` to list, read, and patch traces.
- Import `trace_v2` to create or batch-write spans.
- If you need application instrumentation rather than low-level API calls, Google recommends OpenTelemetry for Cloud Trace instrumentation.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-trace==1.18.0"
```

Common alternatives:

```bash
uv add "google-cloud-trace==1.18.0"
poetry add "google-cloud-trace==1.18.0"
```

If you use explicit service-account credentials in code, install the auth helper too:

```bash
python -m pip install "google-auth"
```

## Authentication And Setup

Cloud Trace client libraries use Application Default Credentials (ADC).

Local development:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

Service-account file flow:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

In Google Cloud runtimes, prefer an attached service account over a downloaded key file.

Before sending or reading trace data, make sure:

1. The Cloud Trace API is enabled in the target project.
2. The caller has the right IAM role.
3. `GOOGLE_CLOUD_PROJECT` or the explicit project argument matches the project that owns the traces.

Useful IAM roles from the Cloud Trace IAM docs:

- `roles/cloudtrace.user`: read trace data
- `roles/cloudtrace.agent`: write spans
- `roles/cloudtrace.admin`: full administrative access

## Client Initialization

### Read traces with `trace_v1`

```python
from google.cloud import trace_v1

client = trace_v1.TraceServiceClient()
project_id = "your-project-id"
```

`trace_v1.TraceServiceClient` is the client for `list_traces`, `get_trace`, and `patch_traces`.

### Write spans with `trace_v2`

```python
from google.cloud import trace_v2

client = trace_v2.TraceServiceClient()
project_name = "projects/your-project-id"
```

`trace_v2.TraceServiceClient` is the client for `batch_write_spans` and `create_span`.

### Explicit credentials

```python
from google.cloud import trace_v2
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = trace_v2.TraceServiceClient(credentials=credentials)
```

## Core Usage

### Write spans with `batch_write_spans`

Use `batch_write_spans` for normal ingest. It accepts a parent resource name in the form `projects/{project_id}` and a list of spans.

```python
from datetime import datetime, timedelta, timezone

from google.cloud import trace_v2
from google.cloud.trace_v2.types import Span
from google.protobuf.timestamp_pb2 import Timestamp

def to_timestamp(value: datetime) -> Timestamp:
    ts = Timestamp()
    ts.FromDatetime(value)
    return ts

project_id = "your-project-id"
trace_id = "0123456789abcdef0123456789abcdef"
span_id = "0123456789abcdef"

now = datetime.now(timezone.utc)
end = now + timedelta(milliseconds=150)

span = Span(
    name=f"projects/{project_id}/traces/{trace_id}/spans/{span_id}",
    span_id=span_id,
    display_name={"value": "process-order"},
    start_time=to_timestamp(now),
    end_time=to_timestamp(end),
)

client = trace_v2.TraceServiceClient()
client.batch_write_spans(
    name=f"projects/{project_id}",
    spans=[span],
)
```

Use `create_span` only when you really want a single-span RPC. For regular ingestion, batch writes reduce request overhead.

### List traces with `trace_v1`

`trace_v1` reads traces by project id, not by `projects/{project_id}` resource name.

```python
from datetime import datetime, timedelta, timezone

from google.cloud import trace_v1
from google.protobuf.timestamp_pb2 import Timestamp

def to_timestamp(value: datetime) -> Timestamp:
    ts = Timestamp()
    ts.FromDatetime(value)
    return ts

client = trace_v1.TraceServiceClient()
project_id = "your-project-id"

end = datetime.now(timezone.utc)
start = end - timedelta(hours=1)

for trace in client.list_traces(
    project_id=project_id,
    view=trace_v1.ListTracesRequest.ViewType.MINIMAL,
    page_size=20,
    start_time=to_timestamp(start),
    end_time=to_timestamp(end),
):
    print(trace.project_id, trace.trace_id)
```

Use a larger view only when you need more than IDs:

- `MINIMAL`: cheapest listing
- `ROOTSPAN`: include root-span data
- `COMPLETE`: include full span details

### Get one trace

```python
from google.cloud import trace_v1

client = trace_v1.TraceServiceClient()

trace = client.get_trace(
    project_id="your-project-id",
    trace_id="0123456789abcdef0123456789abcdef",
)

print(trace.project_id)
print(trace.trace_id)
print(len(trace.spans))
```

## Configuration Notes

- `trace_v1` and `trace_v2` are separate generated clients. Pick the client by operation, not by package name.
- ADC is the default path. Only pass explicit credentials when your runtime cannot provide ADC cleanly.
- If your runtime does not infer the project automatically, pass the project id explicitly and keep `GOOGLE_CLOUD_PROJECT` aligned with it.
- The generated clients support `client_options` if you need to override the API endpoint or other transport options.
- For library debug logging, Google client libraries support the `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` environment variable.
- `trace_v1` also exposes `patch_traces` for updating trace resources, but new write flows should usually be modeled as v2 span writes.

## Common Pitfalls

- Do not import `trace`; the Python package exposes `trace_v1` and `trace_v2`.
- Do not use the v2 client for reads. `trace_v2` is for writing spans, while `trace_v1` is where `list_traces` and `get_trace` live.
- Do not mix resource formats. `trace_v2` methods use names like `projects/my-project`, while `trace_v1` request methods use `project_id="my-project"`.
- For `batch_write_spans`, every span still needs a fully qualified span `name` such as `projects/{project_id}/traces/{trace_id}/spans/{span_id}`.
- If you are instrumenting an app or service rather than calling the Trace API directly, prefer OpenTelemetry. This package is the low-level client, not the main instrumentation path.
- In production, prefer attached service accounts over JSON key files.
- Some generated reference pages can lag the newest release label. Validate package version against PyPI if a `latest` doc page and a generated class page disagree.

## Version-Sensitive Notes For `1.18.0`

- The version used here `1.18.0` matches the current PyPI release as of March 12, 2026.
- The canonical Google Python docs family for this package is `cloudtrace`, not the older `trace` path variant.
- The top-level `latest` reference resolves to `1.18.0`, but some generated class pages can still display `1.17.0`. Treat PyPI as the release authority when there is a temporary docs-generation mismatch.
- PyPI still declares `Requires: Python >=3.7`, but Python 3.7 and 3.8 are upstream-EOL interpreters. If you are choosing a new runtime, use a currently supported Python version even though the wheel metadata remains broad.

## Official Sources

- Python reference root: `https://cloud.google.com/python/docs/reference/cloudtrace/latest`
- `trace_v1` service docs: `https://cloud.google.com/python/docs/reference/cloudtrace/latest/google.cloud.trace_v1.services.trace_service.TraceServiceClient`
- `trace_v2` service docs: `https://cloud.google.com/python/docs/reference/cloudtrace/latest/google.cloud.trace_v2.services.trace_service.TraceServiceClient`
- Python instrumentation guidance: `https://cloud.google.com/trace/docs/setup/python-ot`
- Authentication: `https://cloud.google.com/docs/authentication/client-libraries`
- ADC setup: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- Cloud Trace IAM: `https://cloud.google.com/trace/docs/iam`
- PyPI package: `https://pypi.org/project/google-cloud-trace/`

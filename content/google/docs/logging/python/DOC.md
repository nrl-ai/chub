---
name: logging
description: "google-cloud-logging package guide for Python with ADC setup, stdlib integration, direct API usage, and 3.x version notes"
metadata:
  languages: "python"
  versions: "3.14.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud-logging,google-cloud,logging,gcp,observability,python"
---

# google-cloud-logging Python Package Guide

## What It Is

`google-cloud-logging` is the official Python client for Google Cloud Logging.

Use it in two main ways:

- integrate Cloud Logging with Python's standard `logging` module
- call the Cloud Logging API directly to write, read, and manage log entries

For most application code, start with standard-library integration. Use the direct client APIs when you need explicit control over log names, labels, resources, batching, or entry retrieval.

## Version Covered

- Ecosystem: `pypi`
- Package: `google-cloud-logging`
- Version covered: `3.14.0`
- Import path: `google.cloud.logging`
- Registry: `https://pypi.org/project/google-cloud-logging/`
- Docs root used for this guide: `https://cloud.google.com/python/docs/reference/logging/latest`

Version drift to know about:

- The version used here for this entry is `3.14.0`.
- PyPI lists `3.14.0` as the current package release.
- As of March 12, 2026, the Google Cloud `latest` reference selector and changelog pages still top out at `3.13.0`.
- Inference from those sources: the practical `3.x` API surface used here remains valid for `3.14.0`, but the Cloud docs are not fully version-pinned to that exact package release. Do not assume a method is new in `3.14.0` unless you verify it in release notes or package metadata.

## Install

Pin the package when you need to match the version used here exactly:

```bash
python -m pip install google-cloud-logging==3.14.0
```

The package name and import path differ:

```python
import google.cloud.logging
```

PyPI metadata for `3.14.0` declares `Requires-Python >=3.7`.

## Authentication And Project Setup

Before writing logs, you need:

1. a Google Cloud project
2. the Cloud Logging API enabled
3. Application Default Credentials (ADC)

For local development, use the standard Google Cloud ADC flow:

```bash
gcloud auth application-default login
```

ADC search order matters when auth fails:

1. `GOOGLE_APPLICATION_CREDENTIALS`
2. the local ADC file created by `gcloud auth application-default login`
3. the attached service account from the metadata server

If you must use a service account key locally:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

When the code runs on Google Cloud, prefer an attached service account instead of a key file. The runtime identity needs permission to write logs, typically:

- `roles/logging.logWriter`

## Standard Library Integration

This is the default path for application logging.

```python
import logging
import google.cloud.logging

client = google.cloud.logging.Client(project="my-project-id")
client.setup_logging()

logging.info("service started")
logging.warning("cache miss", extra={"json_fields": {"key": "user:123"}})

client.close()
```

What `client.setup_logging()` does:

- attaches a Cloud Logging handler to the root logger
- by default captures logs at `INFO` level and above
- picks a handler based on the current runtime environment

Runtime behavior to know:

- in managed Google Cloud environments, the library may choose `StructuredLogHandler`
- in other environments, it commonly uses `CloudLoggingHandler`

If you need predictable behavior, configure the handler directly.

### Manual `CloudLoggingHandler`

Use this when you want API-backed delivery from the Python logging stack.

```python
import logging
import google.cloud.logging
from google.cloud.logging_v2.handlers import CloudLoggingHandler

client = google.cloud.logging.Client(project="my-project-id")
handler = CloudLoggingHandler(client, name="application")

logger = logging.getLogger("myapp")
logger.setLevel(logging.INFO)
logger.addHandler(handler)
logger.propagate = False

logger.error("payment failed")

client.close()
```

Important details:

- `CloudLoggingHandler` writes through the Cloud Logging API
- the default transport is `BackgroundThreadTransport`
- use `SyncTransport` only when you need synchronous writes and accept the latency cost

### Manual `StructuredLogHandler`

Use this when the runtime already captures stdout or stderr and turns JSON lines into Cloud Logging entries.

```python
import logging
from google.cloud.logging_v2.handlers.structured_log import StructuredLogHandler

handler = StructuredLogHandler()

logger = logging.getLogger("myapp")
logger.setLevel(logging.INFO)
logger.addHandler(handler)
logger.propagate = False

logger.info(
    "user signed in",
    extra={"json_fields": {"user_id": "123", "plan": "pro"}},
)
```

`StructuredLogHandler` emits structured JSON to standard output. It does not use the transport classes used by `CloudLoggingHandler`.

### Adding Cloud Logging Metadata Through `extra`

The stdlib integration supports Cloud Logging fields through `extra=`.

```python
import logging

logging.info(
    "request completed",
    extra={
        "labels": {"component": "billing", "region": "us-central1"},
        "json_fields": {"order_id": "ord-123", "amount_cents": 4999},
        "trace": "projects/my-project-id/traces/0123456789abcdef",
        "http_request": {"requestMethod": "POST", "requestUrl": "/checkout"},
    },
)
```

Useful fields supported by the official docs include:

- `labels`
- `trace`
- `span_id`
- `trace_sampled`
- `http_request`
- `source_location`
- `resource`
- `json_fields`

Values you pass explicitly through `extra` override auto-detected metadata.

## Direct API Usage

Use the direct client APIs when you want to control the log name or work with entries yourself instead of routing through `logging`.

### Create A Client And Logger

```python
import google.cloud.logging

client = google.cloud.logging.Client(project="my-project-id")
logger = client.logger("application")
```

If you need HTTP instead of gRPC:

```python
import google.cloud.logging

client = google.cloud.logging.Client(
    project="my-project-id",
    _use_grpc=False,
)
```

### Write Text And Structured Entries

```python
import google.cloud.logging

client = google.cloud.logging.Client(project="my-project-id")
logger = client.logger("application")

logger.log_text("plain text entry", severity="INFO")

logger.log_struct(
    {
        "message": "structured entry",
        "job": "daily-sync",
        "success": True,
        "items_processed": 42,
    },
    severity="INFO",
    labels={"component": "sync-worker"},
)

client.close()
```

`Logger.log()` can infer the entry type from the payload. Use `log_text()` and `log_struct()` when you want the entry type to be explicit.

### Batch Writes

Batching reduces per-entry network overhead.

```python
import google.cloud.logging

client = google.cloud.logging.Client(project="my-project-id")
logger = client.logger("application")

with logger.batch() as batch:
    batch.log_text("batch start")
    batch.log_struct({"step": "transform", "rows": 128})
    batch.log_text("batch done", severity="NOTICE")

client.close()
```

### Read Back Entries

```python
import google.cloud.logging

client = google.cloud.logging.Client(project="my-project-id")

for entry in client.list_entries(
    filter_='logName="projects/my-project-id/logs/application"',
    order_by="DESCENDING",
    max_results=20,
):
    print(type(entry).__name__, entry.severity, entry.timestamp)
```

The client applies a 24-hour filter by default unless you override it.

## Resource And Label Control

The library usually infers the monitored resource from the runtime environment.

If you need to set a specific resource:

```python
import google.cloud.logging
from google.cloud.logging_v2.resource import Resource

client = google.cloud.logging.Client(project="my-project-id")

resource = Resource(
    type="global",
    labels={},
)

logger = client.logger(
    "application",
    labels={"service": "billing"},
    resource=resource,
)
```

This matters most in local development, hybrid environments, or custom runtimes where resource detection may fall back to `global`.

## Web Framework Integration

The official integration can enrich entries with `trace`, `span_id`, `trace_sampled`, and `http_request` for supported frameworks such as Flask and Django.

If trace correlation matters:

- configure logging before constructing the Flask app
- add the documented middleware for Django
- let framework integration populate request metadata unless you need to override fields manually

## Common Pitfalls

- Install with `pip install google-cloud-logging`, but import `google.cloud.logging`.
- `client.setup_logging()` changes the root logger. If the app already has logging config, decide whether to use the root logger or attach a handler to a specific logger.
- `setup_logging()` defaults to `INFO`. Lower the threshold explicitly if you need `DEBUG`.
- `BackgroundThreadTransport` is asynchronous. In short-lived scripts, call `client.close()` before exit so buffered logs flush.
- If logs are missing on Google Cloud, check the runtime service account and project selection before debugging the handler.
- If local auth behaves unexpectedly, inspect the ADC search order before changing code.
- `json_fields` and `log_struct()` payloads must be JSON- or protobuf-struct-serializable.
- `CloudLoggingHandler` and `StructuredLogHandler` solve different transport problems. Do not attach both to the same logger unless duplicate output is intentional.

## Version-Sensitive Notes

For current `3.x` projects:

- Prefer `CloudLoggingHandler` or `StructuredLogHandler`. Older product-specific handlers such as `AppEngineHandler` and `ContainerEngineHandler` were deprecated in the `3.0.0` migration.
- The `3.0.0` upgrade also changed resource-detection behavior and expanded structured logging support, so older `2.x` examples are not always the best reference.
- Because the Google Cloud `latest` docs lag the current PyPI release as of March 12, 2026, keep examples anchored to documented `3.x` methods and verify any release-specific additions separately.

Practical upgrade takeaway:

- if upgrading from `2.x`, replace older environment-specific handlers
- retest log formatting, metadata enrichment, and resource attribution after the upgrade
- for projects pinned close to `3.14.0`, treat the Cloud docs as the API reference and PyPI as the authoritative package-version signal

## Recommended Workflow For Agents

1. Set up ADC first and confirm the target project ID.
2. Start with `client.setup_logging()` unless the app already has strict logging configuration.
3. Use `CloudLoggingHandler` when you want direct API delivery from Python.
4. Use `StructuredLogHandler` when the platform already ingests stdout JSON.
5. Switch to direct `client.logger(...).log_*()` calls only when you need named logs, batching, reads, or explicit resources.
6. If the project is pinned to a specific `3.x` release, verify docs drift before assuming a feature is tied to that exact patch or minor version.

## Official Sources

- Cloud Logging Python docs: `https://cloud.google.com/python/docs/reference/logging/latest`
- Standard library integration: `https://cloud.google.com/python/docs/reference/logging/latest/std-lib-integration`
- Direct library usage: `https://cloud.google.com/python/docs/reference/logging/latest/direct-lib-usage`
- Logger reference: `https://cloud.google.com/python/docs/reference/logging/latest/logger`
- Changelog: `https://cloud.google.com/python/docs/reference/logging/latest/changelog`
- Cloud Logging setup guide: `https://cloud.google.com/logging/docs/setup/python`
- ADC setup for local development: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- ADC search order: `https://cloud.google.com/docs/authentication/application-default-credentials`
- 3.0.0 migration guide: `https://cloud.google.com/python/docs/reference/logging/3.3.0/upgrading`
- PyPI package page: `https://pypi.org/project/google-cloud-logging/`

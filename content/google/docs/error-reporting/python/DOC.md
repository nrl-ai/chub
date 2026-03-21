---
name: error-reporting
description: "Google Cloud Error Reporting Python client library for reporting handled exceptions and custom error events"
metadata:
  languages: "python"
  versions: "1.14.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,error-reporting,observability,exceptions,logging"
---

# Google Cloud Error Reporting Python Client

## Golden Rule

Use `google-cloud-error-reporting` for Python code that needs to report handled exceptions or custom error events into Google Cloud Error Reporting.

- PyPI package: `google-cloud-error-reporting`
- Common import: `from google.cloud import error_reporting`
- Covered version: `1.14.0`

If your app already runs on managed Google Cloud platforms that automatically surface uncaught exceptions, do not add this client just to duplicate those same unhandled errors. Use the library when you need explicit reporting, custom grouping metadata, or reporting from environments that are not auto-integrated.

## Install

Pin the version your project expects:

```bash
python -m pip install "google-cloud-error-reporting==1.14.0"
```

Common alternatives:

```bash
uv add "google-cloud-error-reporting==1.14.0"
poetry add "google-cloud-error-reporting==1.14.0"
```

## Project Setup And Auth

Before the client can send events:

1. Enable the Error Reporting API for the target Google Cloud project.
2. Use credentials that can write error events, typically `roles/errorreporting.writer`.
3. Prefer Application Default Credentials (ADC).

Local development with user credentials:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

Service account based setup:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

The client will usually infer the project from ADC. Set `GOOGLE_CLOUD_PROJECT` or pass `project=` if the runtime cannot infer it correctly.

## Initialize The Client

Minimal setup:

```python
from google.cloud import error_reporting

client = error_reporting.Client()
```

Explicit project and service grouping:

```python
from google.cloud import error_reporting

client = error_reporting.Client(
    project="my-project",
    service="checkout-api",
    version="2026.03.12",
)
```

`service` and `version` affect how errors are grouped and displayed in Error Reporting. Set them intentionally for services with multiple deploys or components.

## Core Usage

### Report the current exception

Use `report_exception()` inside an `except` block:

```python
from google.cloud import error_reporting

client = error_reporting.Client(service="checkout-api", version="2026.03.12")

try:
    1 / 0
except ZeroDivisionError:
    client.report_exception()
    raise
```

### Report a handled error message

Use `report()` when you want an Error Reporting event without re-raising:

```python
from google.cloud import error_reporting

client = error_reporting.Client(service="worker")

try:
    process_job()
except TimeoutError as exc:
    client.report(f"job processing timed out: {exc}")
```

### Attach user context

Both reporting methods support a `user` identifier. Use a stable internal identifier, not raw secrets:

```python
try:
    charge_customer(customer)
except Exception:
    client.report_exception(user=customer.id)
    raise
```

## Flask Request Context

For Flask apps, use the helper that converts the current request into the HTTP context Error Reporting expects:

```python
from flask import Flask, request
from google.cloud import error_reporting
from google.cloud.error_reporting import build_flask_context

app = Flask(__name__)
client = error_reporting.Client(service="frontend")

@app.post("/checkout")
def checkout():
    try:
        run_checkout(request.json)
        return {"ok": True}
    except Exception:
        client.report_exception(
            http_context=build_flask_context(request),
            user=request.headers.get("X-User-Id"),
        )
        raise
```

If you are not using Flask, provide the HTTP metadata expected by the library rather than trying to fake a Flask request object.

## Multiprocessing

The library has dedicated multiprocessing guidance because clients created before `fork()` are a bad fit for worker processes. Create the client in each worker after process startup:

```python
from multiprocessing import Pool
from google.cloud import error_reporting

client = None

def init_worker():
    global client
    client = error_reporting.Client(service="batch-worker")

def handle_job(job):
    try:
        process_job(job)
    except Exception:
        client.report_exception()
        raise

with Pool(initializer=init_worker) as pool:
    pool.map(handle_job, jobs)
```

Do not construct one global client in the parent process and share it across forked workers.

## Common Pitfalls

- Package and import names differ: install `google-cloud-error-reporting`, import `google.cloud.error_reporting`.
- Enabling the library is not enough by itself. The target project still needs the Error Reporting API enabled and a credential with write access.
- If `Client()` reports to the wrong project, pass `project=` explicitly or set `GOOGLE_CLOUD_PROJECT`.
- `report_exception()` should run inside an active exception handler. Use `report()` for handled, non-exception events.
- Set `service` and `version` deliberately. If you omit them, grouping in the Error Reporting UI is less useful for multi-service systems.
- On Cloud Run, App Engine, and similar managed environments, uncaught exceptions may already appear through integrated logging. Avoid double-reporting the same failure path.
- In multiprocess workers, initialize the client after the worker starts.

## Version-Sensitive Notes

- PyPI lists `1.14.0` as the current package release used for this doc.
- The Google reference site is published under a `latest` docs root, so use PyPI for exact package-version pinning and release verification.
- Older search results for this package referenced stale version information. As of `2026-03-12`, the earlier version reference `1.14.0` matches the live PyPI package page.

## Official Links

- Python client reference: https://cloud.google.com/python/docs/reference/clouderrorreporting/latest
- Usage guide: https://cloud.google.com/python/docs/reference/clouderrorreporting/latest/usage
- API reference for `Client`: https://cloud.google.com/python/docs/reference/clouderrorreporting/latest/google.cloud.error_reporting.client.Client
- Multiprocessing guidance: https://cloud.google.com/python/docs/reference/clouderrorreporting/latest/multiprocessing
- Product setup guide: https://cloud.google.com/error-reporting/docs/setup/python
- ADC setup: https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment
- PyPI: https://pypi.org/project/google-cloud-error-reporting/

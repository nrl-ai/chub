---
name: scheduler
description: "Google Cloud Scheduler Python client for creating and managing scheduled HTTP, Pub/Sub, and App Engine jobs"
metadata:
  languages: "python"
  versions: "2.18.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,scheduler,gcp,cron,jobs,http,pubsub,app-engine"
---

# google-cloud-scheduler Python Package Guide

## What This Package Is

`google-cloud-scheduler` is the official Python client library for Google Cloud Scheduler.

Use it when your Python code needs to create or manage scheduled jobs that deliver work to exactly one of these target types:

- HTTP endpoints
- Pub/Sub topics
- App Engine HTTP handlers

The generated client gives you sync and async clients for the Cloud Scheduler API, plus typed request and resource objects such as `Job`, `HttpTarget`, `PubsubTarget`, `RetryConfig`, and `OidcToken`.

## Install

The maintainer README and PyPI package page currently publish `2.18.0`.

```bash
pip install google-cloud-scheduler==2.18.0
```

Unpinned install:

```bash
pip install google-cloud-scheduler
```

With `uv`:

```bash
uv add google-cloud-scheduler
```

The package metadata in the maintainer repo lists `Python >= 3.7`.

## Setup And Authentication

Before the client works, you need a Google Cloud project with Cloud Scheduler enabled and Application Default Credentials (ADC) configured for the code that calls the API.

Enable the API:

```bash
gcloud services enable cloudscheduler.googleapis.com
```

Set up local ADC:

```bash
gcloud auth application-default login
```

If you need ADC backed by service account impersonation:

```bash
gcloud auth application-default login \
  --impersonate-service-account=SERVICE_ACCT_EMAIL
```

On Google Cloud runtimes, prefer workload-attached service accounts and let ADC resolve credentials automatically.

## Imports And Client Creation

```python
from google.cloud import scheduler_v1

client = scheduler_v1.CloudSchedulerClient()
```

Async client:

```python
from google.cloud import scheduler_v1

client = scheduler_v1.CloudSchedulerAsyncClient()
```

Build resource names explicitly:

```python
project_id = "my-project"
location_id = "us-central1"
parent = f"projects/{project_id}/locations/{location_id}"
job_name = f"{parent}/jobs/daily-report"
```

## Core Pattern: Create An HTTP Job

HTTP jobs are the common case for Cloud Run, custom HTTPS services, and internal webhooks.

```python
from google.cloud import scheduler_v1
from google.protobuf.duration_pb2 import Duration

client = scheduler_v1.CloudSchedulerClient()

project_id = "my-project"
location_id = "us-central1"
parent = f"projects/{project_id}/locations/{location_id}"

job = scheduler_v1.Job(
    name=f"{parent}/jobs/daily-report",
    description="Call the daily report endpoint every morning",
    schedule="0 9 * * *",
    time_zone="Etc/UTC",
    http_target=scheduler_v1.HttpTarget(
        uri="https://reports-abc123-uc.a.run.app/run",
        http_method=scheduler_v1.HttpMethod.POST,
        headers={"Content-Type": "application/json"},
        body=b'{"source":"cloud-scheduler"}',
        oidc_token=scheduler_v1.OidcToken(
            service_account_email="scheduler-invoker@my-project.iam.gserviceaccount.com",
            audience="https://reports-abc123-uc.a.run.app/run",
        ),
    ),
    retry_config=scheduler_v1.RetryConfig(
        retry_count=3,
        min_backoff_duration=Duration(seconds=10),
        max_backoff_duration=Duration(seconds=300),
        max_doublings=5,
    ),
    attempt_deadline=Duration(seconds=180),
)

created = client.create_job(request={"parent": parent, "job": job})
print(created.name)
```

Use `oidc_token` for services that validate OpenID Connect identity tokens, including Cloud Run and most custom HTTPS targets.

## Pub/Sub Job Example

Use a Pub/Sub target when the scheduled action should enqueue work rather than invoke an HTTP service directly.

```python
import json

from google.cloud import scheduler_v1

client = scheduler_v1.CloudSchedulerClient()

project_id = "my-project"
location_id = "us-central1"
parent = f"projects/{project_id}/locations/{location_id}"

job = scheduler_v1.Job(
    name=f"{parent}/jobs/hourly-refresh",
    schedule="0 * * * *",
    time_zone="Etc/UTC",
    pubsub_target=scheduler_v1.PubsubTarget(
        topic_name=f"projects/{project_id}/topics/hourly-jobs",
        data=json.dumps({"task": "refresh-cache"}).encode("utf-8"),
        attributes={"source": "cloud-scheduler"},
    ),
)

client.create_job(request={"parent": parent, "job": job})
```

The RPC reference states that the Pub/Sub topic must be in the same project as the Cloud Scheduler job.

## App Engine Job Example

Use an App Engine target only when you actually want App Engine routing behavior.

```python
from google.cloud import scheduler_v1

client = scheduler_v1.CloudSchedulerClient()

project_id = "my-project"
location_id = "us-central1"
parent = f"projects/{project_id}/locations/{location_id}"

job = scheduler_v1.Job(
    name=f"{parent}/jobs/app-engine-cleanup",
    schedule="0 3 * * *",
    time_zone="Etc/UTC",
    app_engine_http_target=scheduler_v1.AppEngineHttpTarget(
        relative_uri="/tasks/cleanup",
        http_method=scheduler_v1.HttpMethod.POST,
        headers={"Content-Type": "application/json"},
        body=b'{"action":"cleanup"}',
    ),
)

client.create_job(request={"parent": parent, "job": job})
```

## Read, List, And Update Jobs

```python
from google.cloud import scheduler_v1
from google.protobuf.field_mask_pb2 import FieldMask

client = scheduler_v1.CloudSchedulerClient()

job_name = "projects/my-project/locations/us-central1/jobs/daily-report"

job = client.get_job(request={"name": job_name})
print(job.state.name)
print(job.schedule)

parent = "projects/my-project/locations/us-central1"
for item in client.list_jobs(request={"parent": parent, "page_size": 100}):
    print(item.name, item.state.name)

patch = scheduler_v1.Job(
    name=job_name,
    schedule="30 9 * * *",
    time_zone="America/Los_Angeles",
)

mask = FieldMask(paths=["schedule", "time_zone"])
updated = client.update_job(request={"job": patch, "update_mask": mask})
print(updated.schedule)
```

Important update behavior from the generated reference:

- `list_jobs` is paginated and `page_size` has a documented maximum of `500`.
- `update_job` is a partial update. Use `FieldMask`; do not assume omitted fields are preserved safely by accident.
- If an update fails, the job can enter `UPDATE_FAILED`. Retry `update_job` until it succeeds.

## Pause, Resume, Run, And Delete

```python
from google.cloud import scheduler_v1

client = scheduler_v1.CloudSchedulerClient()
job_name = "projects/my-project/locations/us-central1/jobs/daily-report"

client.pause_job(request={"name": job_name})
client.resume_job(request={"name": job_name})
client.run_job(request={"name": job_name})
client.delete_job(request={"name": job_name})
```

Operational behavior to remember:

- `pause_job` applies to an enabled job.
- `resume_job` applies to a paused job.
- `run_job` forces an immediate execution and is not a dry run.

## Target Authentication Rules

There are two separate auth problems:

1. Your Python code must authenticate to the Cloud Scheduler API.
2. Cloud Scheduler may need to authenticate from the job to the target service.

For the first, use ADC.

For HTTP job targets:

- Use `oidc_token` for services that expect an OIDC identity token.
- Use `oauth_token` when the target is a Google API on `*.googleapis.com`.
- The service account configured in `oidc_token` or `oauth_token` must belong to the same project as the job.
- If you set `oidc_token` or `oauth_token`, Cloud Scheduler overrides any `Authorization` header you put in `http_target.headers`.

For Cloud Run or another protected backend, grant the service account permission to invoke the target before you schedule the job.

## Retry, Deadline, And Scheduling Semantics

Important runtime behavior from the RPC and generated reference docs:

- Cloud Scheduler does not allow two outstanding executions of the same job at the same time.
- If one execution is still running when the next schedule time arrives, the next execution is delayed.
- `retry_count=0` means failures are not retried before the next scheduled time.
- For HTTP targets, `attempt_deadline` must be between 15 seconds and 30 minutes.
- For Pub/Sub targets, `attempt_deadline` is ignored.

## Common Pitfalls

- HTTP request bodies are valid only for `POST`, `PUT`, or `PATCH`.
- If you send a body and omit `Content-Type`, Cloud Scheduler defaults it to `application/octet-stream`.
- Cloud Scheduler computes or overwrites several HTTP headers, including `Host`, `Content-Length`, `User-Agent`, and `X-CloudScheduler-*`.
- Total HTTP header size must stay under 80 KB.
- `AppEngineHttpTarget` requires `relative_uri` to start with `/`.
- `PATCH` and `OPTIONS` are not allowed for `AppEngineHttpTarget`.
- Always set `time_zone` intentionally. Do not rely on local machine timezone assumptions.
- Prefer explicit `request={...}` calls or typed request objects. That matches the generated library surface and is less brittle than positional-argument examples copied from old posts.

## Version-Sensitive Notes

- This guide covers package version `2.18.0`, which is the version currently shown on PyPI and in the maintainer package README.
- The docs URL for this package points at the maintainer repo directory on GitHub. That repo is useful for install metadata and package-level release context, but operational API details are clearer in the generated Python reference and Cloud Scheduler product docs.
- As of `2026-03-12`, Google’s generated `latest` reference pages appear to mix `2.17.0` and `2.18.0` labels across pages. When that happens, trust the installed package version in your environment and the actual generated method signatures over stale page chrome.

## Official Sources

- Maintainer package directory: https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-scheduler
- Maintainer README: https://raw.githubusercontent.com/googleapis/google-cloud-python/main/packages/google-cloud-scheduler/README.rst
- Python client reference: https://docs.cloud.google.com/python/docs/reference/cloudscheduler/latest
- Cloud Scheduler setup: https://docs.cloud.google.com/scheduler/docs/setup
- Cloud Scheduler authentication: https://docs.cloud.google.com/scheduler/docs/authentication
- Secure cron jobs with HTTP targets: https://docs.cloud.google.com/scheduler/docs/http-target-auth
- RPC reference: https://docs.cloud.google.com/scheduler/docs/reference/rpc/google.cloud.scheduler.v1
- PyPI package page: https://pypi.org/project/google-cloud-scheduler/

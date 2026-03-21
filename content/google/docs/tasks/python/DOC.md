---
name: tasks
description: "Google Cloud Tasks Python client library for creating and managing HTTP and App Engine tasks"
metadata:
  languages: "python"
  versions: "2.21.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,cloud-tasks,queues,async,jobs,http"
---

# Google Cloud Tasks Python Client

## Golden Rule

Use `google-cloud-tasks` for Python and authenticate with Application Default Credentials (ADC) unless you have a strong reason to construct credentials manually. For application handlers, treat queue location, IAM, and request authentication as part of the API contract, not deployment details.

As of March 12, 2026, PyPI lists `google-cloud-tasks 2.21.0`, which matches the version used here for this session.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-tasks==2.21.0"
```

Common alternatives:

```bash
uv add "google-cloud-tasks==2.21.0"
poetry add "google-cloud-tasks==2.21.0"
```

## Authentication And Setup

Cloud Tasks client libraries are meant to run with Google Cloud credentials, not anonymous requests. For local development, the standard path is:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

If you must use a service account key file locally, point ADC at it:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

Minimal IAM to enqueue HTTP tasks is usually:

- `roles/cloudtasks.enqueuer` on the queue or project for the caller creating tasks
- `roles/iam.serviceAccountUser` on the service account attached to the task when you use `oidc_token` or `oauth_token`
- A target-service role on the receiving service, such as Cloud Run invocation permission, for the service account that will sign the outbound token

Queue administration uses broader roles such as `roles/cloudtasks.queueAdmin` or `roles/cloudtasks.admin`.

## Initialize The Client

Most code only needs a default client:

```python
from google.cloud import tasks_v2

client = tasks_v2.CloudTasksClient()
queue_name = client.queue_path("my-project", "us-central1", "default")
```

For async code:

```python
from google.cloud import tasks_v2

client = tasks_v2.CloudTasksAsyncClient()
queue_name = client.queue_path("my-project", "us-central1", "default")
```

If you need explicit credentials instead of ADC:

```python
from google.cloud import tasks_v2
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "service-account.json"
)
client = tasks_v2.CloudTasksClient(credentials=credentials)
```

The generated client also supports `client_options`, including explicit `api_endpoint`, if you need non-default endpoint or mTLS behavior.

## Create A Basic HTTP Task

This is the common pattern for pushing JSON work to an HTTP handler:

```python
import datetime
import json

from google.cloud import tasks_v2
from google.protobuf import timestamp_pb2

def enqueue_http_json_task(
    project_id: str,
    location: str,
    queue_id: str,
    url: str,
    payload: dict,
) -> str:
    client = tasks_v2.CloudTasksClient()
    parent = client.queue_path(project_id, location, queue_id)

    task = tasks_v2.Task(
        http_request=tasks_v2.HttpRequest(
            http_method=tasks_v2.HttpMethod.POST,
            url=url,
            headers={"Content-Type": "application/json"},
            body=json.dumps(payload).encode(),
        )
    )

    schedule_time = timestamp_pb2.Timestamp()
    schedule_time.FromDatetime(
        datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(seconds=30)
    )
    task.schedule_time = schedule_time

    response = client.create_task(
        request={"parent": parent, "task": task}
    )
    return response.name
```

Use an explicit `task.name` only when you need deduplication or idempotency. The API docs note that explicit task IDs introduce an extra lookup, can increase latency, and sequential prefixes increase storage hot-spotting. If a task with the same ID was recently deleted or executed, deduplication may still reject the create request.

## Create An Authenticated HTTP Task

For Cloud Run, Cloud Functions, or your own HTTPS handler protected by IAM, attach an OIDC token:

```python
import json

from google.cloud import tasks_v2

def enqueue_oidc_task(
    project_id: str,
    location: str,
    queue_id: str,
    url: str,
    service_account_email: str,
    payload: dict,
) -> str:
    client = tasks_v2.CloudTasksClient()
    parent = client.queue_path(project_id, location, queue_id)

    task = tasks_v2.Task(
        http_request=tasks_v2.HttpRequest(
            http_method=tasks_v2.HttpMethod.POST,
            url=url,
            headers={"Content-Type": "application/json"},
            body=json.dumps(payload).encode(),
            oidc_token=tasks_v2.OidcToken(
                service_account_email=service_account_email,
                audience=url,
            ),
        )
    )

    response = client.create_task(request={"parent": parent, "task": task})
    return response.name
```

Use `oidc_token` for handlers that validate Google-signed ID tokens. Use `oauth_token` only when the target expects an OAuth access token for Google APIs such as `*.googleapis.com`.

## App Engine Tasks

Cloud Tasks also supports App Engine targets through `app_engine_http_request`. Prefer that field only when the target really is an App Engine service. For Cloud Run or arbitrary HTTPS endpoints, use `http_request`.

```python
from google.cloud import tasks_v2

client = tasks_v2.CloudTasksClient()
parent = client.queue_path("my-project", "us-central1", "default")

task = tasks_v2.Task(
    app_engine_http_request=tasks_v2.AppEngineHttpRequest(
        http_method=tasks_v2.HttpMethod.POST,
        relative_uri="/tasks/process",
        body=b'{\"job\": \"sync\"}',
        headers={"Content-Type": "application/json"},
    )
)

response = client.create_task(request={"parent": parent, "task": task})
```

## Inspect And Manage Queues

Queue operations you will use most often:

```python
from google.cloud import tasks_v2

client = tasks_v2.CloudTasksClient()
queue_name = client.queue_path("my-project", "us-central1", "default")

queue = client.get_queue(name=queue_name)
print(queue.rate_limits)

for task in client.list_tasks(
    request={"parent": queue_name, "response_view": tasks_v2.Task.View.FULL}
):
    print(task.name, task.schedule_time)

client.pause_queue(name=queue_name)
client.resume_queue(name=queue_name)
client.purge_queue(name=queue_name)
```

Useful methods on `CloudTasksClient` include:

- `create_queue`, `get_queue`, `list_queues`, `update_queue`
- `pause_queue`, `resume_queue`, `purge_queue`, `delete_queue`
- `create_task`, `get_task`, `list_tasks`, `delete_task`, `run_task`

`run_task` is for manual execution/testing of a task now. It is not a replacement for normal dispatch configuration.

## Configuration Notes

- Queue names are regional. The path is always `projects/{project}/locations/{location}/queues/{queue}`.
- Create the queue before enqueuing work. `create_task` does not auto-create missing queues.
- Queue-level throttling and retry behavior live on the queue, not the individual `create_task` call.
- Task payload size is capped at `100KB`.
- Queue creation docs note that tasks can live in a queue for up to `31 days`, so Cloud Tasks is not a long-term durable archive.
- If you need to inspect task payloads or headers, request `response_view=FULL`; the default view is lighter.

## Common Pitfalls

- Do not confuse Cloud Tasks with Pub/Sub. Cloud Tasks is for controlled dispatch to a specific handler with per-queue retry and rate settings.
- Do not omit the location when building queue paths. Region mismatch is a common cause of `NOT_FOUND`.
- If you use OIDC or OAuth tokens, the caller creating the task needs permission to act as the signing service account, and the target service must also trust that account.
- `purge_queue` is destructive and takes time to propagate. Do not call it in tests unless the queue is isolated.
- `delete_queue` is also slow to clear fully. The Python client reference says deletion can take up to `7 days`, and a queue with the same name cannot be recreated until deletion finishes.
- The Python library does not cover every REST surface. The product docs explicitly call out `BufferTask` as not supported in client libraries, so use the REST API directly if you need that endpoint.
- There is no "update task" workflow for mutating an existing enqueued task. In practice you usually create a replacement task and delete the old one if it has not executed yet.
- Avoid sequential explicit task IDs such as timestamps or incrementing integers. The client reference warns they can increase latency and error rates because the backend expects roughly uniform distribution.

## Version-Sensitive Notes

- `2.21.0` is the current PyPI release for this package as of March 12, 2026.
- The upstream changelog for `2.21.0` notes automatic mTLS enablement when supported client certificates are detected. If your environment has unexpected client certs, pin `client_options` or your Google API mTLS environment variables deliberately instead of relying on defaults.
- The changelog for `2.20.0` deprecated the generated `credentials_file` constructor argument. Prefer ADC, `credentials=...`, or explicit `service_account.Credentials.from_service_account_file(...)`.
- The generated reference pages still contain some stale-looking older version labels in page chrome. Use PyPI as the source of truth for the published package version and use the reference docs for API surface and examples.

## Official Sources Used For This Doc

- PyPI package page: `https://pypi.org/project/google-cloud-tasks/`
- Python reference root: `https://docs.cloud.google.com/python/docs/reference/cloudtasks/latest`
- CloudTasksClient reference: `https://docs.cloud.google.com/python/docs/reference/cloudtasks/latest/google.cloud.tasks_v2.services.cloud_tasks.CloudTasksClient`
- Changelog: `https://docs.cloud.google.com/python/docs/reference/cloudtasks/latest/changelog`
- Create HTTP target tasks guide: `https://cloud.google.com/tasks/docs/creating-http-target-tasks`
- ADC local development guide: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- IAM roles for Cloud Tasks: `https://cloud.google.com/iam/docs/roles-permissions/cloudtasks`

---
name: run
description: "Google Cloud Run Python client library for managing services, jobs, executions, tasks, revisions, and worker pools"
metadata:
  languages: "python"
  versions: "0.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,cloud-run,serverless,containers,jobs,google-cloud"
---

# Google Cloud Run Python Client

## Golden Rule

Use `google-cloud-run` for Cloud Run control-plane automation in Python.

- This package manages Cloud Run resources such as services, jobs, executions, tasks, revisions, and worker pools.
- It does not replace the Cloud Run runtime contract inside your container. Your deployed app still needs to listen on the expected port and handle requests normally.
- Authenticate with Application Default Credentials (ADC). For local development, use `gcloud auth application-default login` or set `GOOGLE_APPLICATION_CREDENTIALS` to a service account key file if you explicitly need file-based credentials.

## Install

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "google-cloud-run==0.15.0"
```

Common alternatives:

```bash
uv add "google-cloud-run==0.15.0"
poetry add "google-cloud-run==0.15.0"
```

## Authentication And Setup

Cloud Run uses Google Cloud authentication, so start with ADC instead of hand-rolling tokens.

Local development:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="my-project"
export GOOGLE_CLOUD_LOCATION="us-central1"
```

Service account key file when needed:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

Basic client setup:

```python
import os
from google.cloud import run_v2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.environ.get("GOOGLE_CLOUD_LOCATION", "us-central1")
parent = f"projects/{project_id}/locations/{location}"

services_client = run_v2.ServicesClient()
jobs_client = run_v2.JobsClient()
```

If you need a custom endpoint or universe domain, pass `client_options` when creating the client. The generated clients expose `api_endpoint` and `universe_domain` settings through `client_options`.

## Client Surface

The package is organized around API-specific clients:

- `ServicesClient` / `ServicesAsyncClient`
- `JobsClient` / `JobsAsyncClient`
- `ExecutionsClient`
- `TasksClient`
- `RevisionsClient`
- `WorkerPoolsClient`
- `BuildsClient`

Use `ServicesClient` for service CRUD, `JobsClient` for job CRUD and job execution, and the other clients when you need to inspect the resources created by those operations.

## Core Usage

### List and fetch services

```python
import os
from google.cloud import run_v2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.environ.get("GOOGLE_CLOUD_LOCATION", "us-central1")

client = run_v2.ServicesClient()
parent = f"projects/{project_id}/locations/{location}"

for service in client.list_services(parent=parent):
    print(service.name, list(service.urls))

service_name = client.service_path(project_id, location, "hello-service")
service = client.get_service(name=service_name)
print(service.name)
print(list(service.urls))
```

### Create a service

Cloud Run mutations are long-running operations. Call `.result()` before assuming the resource exists.

```python
import os
from google.cloud import run_v2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.environ.get("GOOGLE_CLOUD_LOCATION", "us-central1")
parent = f"projects/{project_id}/locations/{location}"

client = run_v2.ServicesClient()

service = run_v2.Service(
    template=run_v2.RevisionTemplate(
        service_account="runtime-sa@my-project.iam.gserviceaccount.com",
        containers=[
            run_v2.Container(
                image="us-docker.pkg.dev/cloudrun/container/hello",
                env=[run_v2.EnvVar(name="APP_ENV", value="prod")],
            )
        ],
    )
)

operation = client.create_service(
    parent=parent,
    service=service,
    service_id="hello-service",
)

created = operation.result()
print(created.name)
print(list(created.urls))
```

### Update a service with an update mask

Patch updates need a `FieldMask`. Only the paths in the update mask are applied.

```python
import os
from google.cloud import run_v2
from google.protobuf import field_mask_pb2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.environ.get("GOOGLE_CLOUD_LOCATION", "us-central1")

client = run_v2.ServicesClient()
service_name = client.service_path(project_id, location, "hello-service")
service = client.get_service(name=service_name)

service.template.containers[0].env.append(
    run_v2.EnvVar(name="FEATURE_FLAG", value="on")
)

update_mask = field_mask_pb2.FieldMask(paths=["template.containers"])
updated = client.update_service(
    service=service,
    update_mask=update_mask,
).result()

print(updated.name)
```

### Create and run a job

Jobs also return long-running operations. `run_job()` returns an operation whose result is an `Execution`.

```python
import os
from google.cloud import run_v2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.environ.get("GOOGLE_CLOUD_LOCATION", "us-central1")
parent = f"projects/{project_id}/locations/{location}"

client = run_v2.JobsClient()

job = run_v2.Job(
    template=run_v2.ExecutionTemplate(
        task_count=1,
        template=run_v2.TaskTemplate(
            service_account="runtime-sa@my-project.iam.gserviceaccount.com",
            containers=[
                run_v2.Container(
                    image="us-docker.pkg.dev/my-project/batch/worker:latest",
                    args=["--mode", "daily"],
                    env=[run_v2.EnvVar(name="APP_ENV", value="prod")],
                )
            ],
        ),
    )
)

created = client.create_job(
    parent=parent,
    job=job,
    job_id="daily-job",
).result()

execution = client.run_job(name=created.name).result()
print(execution.name)
```

### Inspect executions and tasks

Use the follow-on clients instead of trying to infer execution state from the original job resource.

```python
from google.cloud import run_v2

executions = run_v2.ExecutionsClient()
tasks = run_v2.TasksClient()

job_name = "projects/my-project/locations/us-central1/jobs/daily-job"
execution_parent = f"{job_name}/executions"

for execution in executions.list_executions(parent=execution_parent):
    print(execution.name)

task_parent = f"{job_name}/executions/my-execution"
for task in tasks.list_tasks(parent=task_parent):
    print(task.name)
```

## Async Usage

If the rest of your application is already async, use the async variants rather than wrapping sync clients in thread pools.

```python
import os
from google.cloud import run_v2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
location = os.environ.get("GOOGLE_CLOUD_LOCATION", "us-central1")

client = run_v2.ServicesAsyncClient()
parent = f"projects/{project_id}/locations/{location}"

async def list_services() -> None:
    async for service in client.list_services(parent=parent):
        print(service.name)
```

## Configuration Notes

- Resource names follow the usual Google Cloud pattern, such as `projects/{project}/locations/{location}/services/{service}` and `projects/{project}/locations/{location}/jobs/{job}`.
- Prefer the client helper methods like `service_path()` and `job_path()` when you need a resource name string.
- The generated clients support `client_options`, `credentials`, `transport`, and custom retry/timeout values. Use those instead of patching transport internals.
- For local or CI automation, export `GOOGLE_CLOUD_PROJECT` and the location you deploy into so your scripts do not silently mutate the wrong region.

## Common Pitfalls

- This package manages Cloud Run resources; it does not deploy source code directly for you. You still need a container image or another supported deployment artifact in the resource template.
- Always wait on long-running operations with `.result()`. Create, update, delete, and run calls do not complete synchronously.
- Do not mix regions. A service or job name built for `us-central1` will not resolve in `europe-west1`.
- Use an update mask for partial updates. Without it, agents often assume a mutated local object automatically patches server-side state.
- ADC is the normal auth path. Hard-coded access tokens and checked-in service account keys are the wrong default.
- If you override job execution arguments or environment variables, verify the container names and override fields against the generated request types before sending the request.
- The client surface includes worker-pool APIs. Do not assume worker pools exist in every project or region your code targets.

## Version-Sensitive Notes

- As of March 12, 2026, PyPI and the generated reference both show `google-cloud-run 0.15.0`.
- The published changelog page under the `latest` docs root currently lists entries through `0.14.0`, so release-note coverage appears to lag the package version. If you need exact `0.15.0` change details, double-check PyPI release history or the package repository before assuming the changelog page is complete.
- The generated changelog records notable recent behavior changes:
  - `0.14.0` added automatic mTLS enablement when a client certificate is available and mTLS is not explicitly disabled.
  - `0.12.0` deprecated the `credentials_file` helper argument in client constructors.
  - `0.11.0` included breaking changes around worker pools and build worker pools.

## Official Sources

- Cloud Run Python reference root: `https://cloud.google.com/python/docs/reference/run/latest`
- Services client reference: `https://cloud.google.com/python/docs/reference/run/latest/google.cloud.run_v2.services.services.ServicesClient`
- Jobs client reference: `https://cloud.google.com/python/docs/reference/run/latest/google.cloud.run_v2.services.jobs.JobsClient`
- Changelog: `https://cloud.google.com/python/docs/reference/run/latest/changelog`
- PyPI package page: `https://pypi.org/project/google-cloud-run/`
- Google Cloud ADC guide: `https://cloud.google.com/docs/authentication/provide-credentials-adc`

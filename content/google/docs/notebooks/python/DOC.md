---
name: notebooks
description: "Google Cloud Notebooks Python client library for notebook instances, managed runtimes, and Workbench instances"
metadata:
  languages: "python"
  versions: "1.15.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,notebooks,jupyter,workbench,ml"
---

# Google Cloud Notebooks Python Client

## Golden Rule

Use `google-cloud-notebooks` for Google Cloud notebook instance automation, authenticate with Application Default Credentials (ADC), and pick the client surface that matches the resource you are managing:

- `notebooks_v1.NotebookServiceClient`: classic AI Platform notebook instances, executions, schedules
- `notebooks_v1.ManagedNotebookServiceClient`: managed notebook runtimes
- `notebooks_v2.NotebookServiceClient`: Workbench notebook instances

As of March 12, 2026, PyPI lists `google-cloud-notebooks 1.15.0`. The Google-hosted `latest` reference pages are current enough to use, but some generated subpages still display older minor versions, so pin package versions from PyPI rather than from the page chrome.

## Install

Pin the package version your project expects:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "google-cloud-notebooks==1.15.0"
```

Common alternatives:

```bash
uv add "google-cloud-notebooks==1.15.0"
poetry add "google-cloud-notebooks==1.15.0"
```

## Authentication And Project Setup

The official quick start says you must:

1. Select or create a Google Cloud project.
2. Enable billing for the project.
3. Enable AI Platform Notebooks for the project.
4. Set up authentication.

Use ADC so the same code works locally and on Google Cloud:

```bash
gcloud auth application-default login
```

For production on Google Cloud, prefer an attached service account with the least-privileged IAM roles instead of distributing service account keys.

If you must use a credentials file explicitly, set:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/credentials.json"
```

## Choose The Right Client

### `notebooks_v1.NotebookServiceClient`

Use this for classic notebook instances and related resources such as executions and schedules.

```python
from google.cloud import notebooks_v1

client = notebooks_v1.NotebookServiceClient()
parent = "projects/my-project/locations/us-central1"

for instance in client.list_instances(parent=parent):
    print(instance.name, instance.state.name)
```

Key operations:

- `list_instances`, `get_instance`
- `create_instance`, `delete_instance`
- `start_instance`, `stop_instance`
- `create_execution`, `list_executions`
- `create_schedule`, `list_schedules`

The generated client docs require `parent` in the form `projects/{project_id}/locations/{location}`. `create_instance` returns a long-running operation.

### `notebooks_v1.ManagedNotebookServiceClient`

Use this for managed notebook runtimes.

```python
from google.cloud import notebooks_v1

client = notebooks_v1.ManagedNotebookServiceClient()
parent = "projects/my-project/locations/us-central1"

for runtime in client.list_runtimes(parent=parent):
    print(runtime.name, runtime.health_state.name)
```

Useful operations:

- `list_runtimes`, `get_runtime`
- `create_runtime`, `delete_runtime`
- `start_runtime`, `stop_runtime`
- IAM helpers such as `get_iam_policy` and `set_iam_policy`

The generated docs explicitly note that `get_runtime` expects a regional location rather than a zonal endpoint.

### `notebooks_v2.NotebookServiceClient`

Use this for Workbench notebook instances.

```python
from google.cloud import notebooks_v2

client = notebooks_v2.NotebookServiceClient()
parent = "projects/my-project/locations/us-central1"

for instance in client.list_instances(parent=parent):
    print(instance.name, instance.state.name)
```

Useful operations:

- `list_instances`, `get_instance`
- `create_instance`, `delete_instance`
- `check_instance_upgradability`

The v2 generated sample shows `Instance.gce_setup.vm_image.name` and `Instance.gce_setup.vm_image.project` when building a creation request, which is a good hint that Workbench instance creation uses a different schema than classic v1 instances.

## Common Request Patterns

### Get one instance by resource name

```python
from google.cloud import notebooks_v2

client = notebooks_v2.NotebookServiceClient()
name = "projects/my-project/locations/us-central1/instances/my-instance"

instance = client.get_instance(name=name)
print(instance.name)
```

### Start and stop a classic v1 instance

```python
from google.cloud import notebooks_v1

client = notebooks_v1.NotebookServiceClient()
name = "projects/my-project/locations/us-central1/instances/my-instance"

client.start_instance(name=name).result()
client.stop_instance(name=name).result()
```

### Create a classic v1 instance

The generated docs show the minimum shape: set a VM image, set a machine type, send `parent`, `instance_id`, and `instance`, then wait on the operation.

```python
from google.cloud import notebooks_v1

client = notebooks_v1.NotebookServiceClient()

instance = notebooks_v1.Instance()
instance.vm_image.image_name = "tf-latest-cpu"
instance.vm_image.project = "deeplearning-platform-release"
instance.machine_type = "e2-standard-4"

operation = client.create_instance(
    parent="projects/my-project/locations/us-central1",
    instance_id="my-notebook",
    instance=instance,
)

created = operation.result(timeout=1800)
print(created.name)
```

### Create a managed runtime

`create_runtime` is also a long-running operation. The request must include `parent`, `runtime_id`, and a populated `Runtime`.

```python
from google.cloud import notebooks_v1

client = notebooks_v1.ManagedNotebookServiceClient()

runtime = notebooks_v1.Runtime()

operation = client.create_runtime(
    parent="projects/my-project/locations/us-central1",
    runtime_id="my-runtime",
    runtime=runtime,
)

created = operation.result(timeout=1800)
print(created.name)
```

In real code, fill the required runtime configuration fields from the `Runtime` schema before sending the request.

## Operations, Timeouts, And Endpoints

- Most create, delete, start, and stop methods return `google.api_core.operation.Operation`. Call `.result()` and budget for long timeouts.
- The generated samples repeatedly warn that you might need to specify a regional API endpoint with `client_options` for your location.
- If you are looping over many resources, prefer pager-returning methods such as `list_instances` and `list_runtimes` instead of manually managing page tokens.
- Use request objects when the method has many optional fields; flattened keyword arguments are convenient for small calls but easier to misuse in larger create/update flows.

## Common Pitfalls

- Do not mix the three service surfaces. `notebooks_v1.NotebookServiceClient`, `notebooks_v1.ManagedNotebookServiceClient`, and `notebooks_v2.NotebookServiceClient` manage different resource models.
- Resource names are location-scoped. Build `parent` as `projects/{project}/locations/{location}` and full resource names as `projects/{project}/locations/{location}/instances/{instance}` or `.../runtimes/{runtime}`.
- `get_runtime` in the managed-runtime client expects a regional location. Zonal resource naming and regional endpoints are easy to confuse.
- Creation calls are not synchronous. Agents often forget to wait on the returned long-running operation before reading the new resource.
- ADC and `gcloud auth login` are not the same thing. Local client-library auth should use `gcloud auth application-default login` when you want user credentials for ADC.
- Prefer attached service accounts in Google Cloud environments. Service account keys are a fallback, not the default.

## Version-Sensitive Notes

- PyPI currently publishes `1.15.0`, uploaded on January 15, 2026.
- The official Google Cloud reference root still points at `latest`, but some generated package pages under that tree show older minor versions in their titles. Treat the reference tree as authoritative for API shapes, and PyPI as authoritative for `metadata.versions`.
- PyPI says the package supports Python `>=3.7`, including `3.14`. If your project has already dropped Python 3.7 or 3.8, that is your application policy, not a library requirement from this package.

---
name: batch
description: "Google Cloud Batch Python client library for submitting, inspecting, and managing batch jobs"
metadata:
  languages: "python"
  versions: "0.20.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,batch,jobs,compute,scheduler"
---

# Google Cloud Batch Python Client

## Golden Rule

Use `google-cloud-batch` with `from google.cloud import batch_v1`, authenticate with Application Default Credentials (ADC), and treat Batch as a regional control plane over Compute Engine resources. Most real failures come from missing IAM, unsupported region choice, or VM/network/service-account configuration in the job definition, not from the Python client itself.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-batch==0.20.0"
```

Common alternatives:

```bash
uv add "google-cloud-batch==0.20.0"
poetry add "google-cloud-batch==0.20.0"
```

## Authentication And Setup

Before writing code:

1. Enable the Google Cloud Batch API on the target project.
2. Make sure billing is enabled for the project.
3. Pick a supported Batch region and keep using that same region in resource names.
4. Confirm the caller can create Batch jobs and act as the job's VM service account.

For local development, use ADC:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

Then construct the client normally:

```python
from google.cloud import batch_v1

client = batch_v1.BatchServiceClient()
```

If you must use a service account key file, set the standard ADC environment variable before creating the client:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

Practical auth notes:

- `gcloud auth login` is not enough for this library by itself; the Python client uses ADC.
- On Google Cloud runtimes, prefer attached service accounts or workload identity over long-lived key files.
- If your organization has removed the default VPC network or restricts images and service accounts with org policies, your job spec must set compatible networking and identity explicitly.

## Core Usage

### Submit a basic script job

This is the smallest practical job shape: one task group, one runnable script, fixed compute, Cloud Logging enabled.

```python
from google.cloud import batch_v1

def create_script_job(project_id: str, region: str, job_id: str) -> batch_v1.Job:
    client = batch_v1.BatchServiceClient()

    runnable = batch_v1.Runnable()
    runnable.script = batch_v1.Runnable.Script()
    runnable.script.text = (
        "echo Hello from task ${BATCH_TASK_INDEX}/${BATCH_TASK_COUNT}; "
        "sleep 5"
    )

    resources = batch_v1.ComputeResource()
    resources.cpu_milli = 2000
    resources.memory_mib = 512

    task = batch_v1.TaskSpec()
    task.runnables = [runnable]
    task.compute_resource = resources
    task.max_retry_count = 2
    task.max_run_duration = "3600s"

    group = batch_v1.TaskGroup()
    group.task_count = 4
    group.task_spec = task

    instance_policy = batch_v1.AllocationPolicy.InstancePolicy()
    instance_policy.machine_type = "e2-standard-4"

    instance = batch_v1.AllocationPolicy.InstancePolicyOrTemplate()
    instance.policy = instance_policy

    allocation_policy = batch_v1.AllocationPolicy()
    allocation_policy.instances = [instance]

    job = batch_v1.Job()
    job.task_groups = [group]
    job.allocation_policy = allocation_policy
    job.labels = {"env": "dev"}

    job.logs_policy = batch_v1.LogsPolicy()
    job.logs_policy.destination = batch_v1.LogsPolicy.Destination.CLOUD_LOGGING

    request = batch_v1.CreateJobRequest()
    request.parent = f"projects/{project_id}/locations/{region}"
    request.job_id = job_id
    request.job = job

    return client.create_job(request=request)

created = create_script_job(
    project_id="my-project",
    region="us-central1",
    job_id="example-job-001",
)

print(created.name)
```

Notes:

- `job_id` must be unique within the project and region.
- The parent resource is always `projects/{project}/locations/{region}`.
- The task runtime gets useful built-in environment variables such as `BATCH_TASK_INDEX`, `BATCH_TASK_COUNT`, `BATCH_HOSTS_FILE`, and `BATCH_TASK_RETRY_ATTEMPT`.

### Run a container instead of an inline script

Replace the runnable setup when your workload is already packaged as a container image:

```python
from google.cloud import batch_v1

runnable = batch_v1.Runnable()
runnable.container = batch_v1.Runnable.Container()
runnable.container.image_uri = "us-docker.pkg.dev/cloudrun/container/hello"
runnable.container.entrypoint = "/bin/sh"
runnable.container.commands = ["-c", "python -V && echo batch container started"]
```

Keep the rest of the job structure the same.

### Inspect, list, cancel, or delete jobs

```python
from google.cloud import batch_v1

client = batch_v1.BatchServiceClient()
job_name = "projects/my-project/locations/us-central1/jobs/example-job-001"
parent = "projects/my-project/locations/us-central1"

job = client.get_job(name=job_name)
print(job.status.state)

for item in client.list_jobs(parent=parent):
    print(item.name, item.status.state)

cancel_op = client.cancel_job(name=job_name)
cancel_op.result()

delete_op = client.delete_job(name=job_name)
delete_op.result()
```

`cancel_job()` and `delete_job()` return long-running operations, so wait for them instead of assuming the state changed immediately.

### Task groups and tasks

`list_tasks()` needs the task group resource name, not just the job name. If you need per-task inspection, keep the task group name returned by the created job or the later job lookup and pass that full resource path to `list_tasks(parent=...)`.

## Configuration Notes

### Service account and permissions

Batch jobs run on VMs, so there are usually two identities involved:

- the caller creating the job
- the service account attached to the job's VMs

The official setup docs call out two common requirements:

- the caller needs permission to create jobs
- the caller needs permission to act as the job service account

If you need a non-default VM service account, set it in the job's allocation policy rather than relying on ambient defaults.

### Region and endpoint handling

Batch is regional. Keep these aligned:

- the `parent` location in `create_job()` and `list_jobs()`
- the fully-qualified job name in `get_job()`, `cancel_job()`, and `delete_job()`
- the product region where Batch is supported

The generated client also supports endpoint overrides through `client_options.api_endpoint`. This matters for custom endpoint selection and mTLS behavior.

### Logging

For job output, send logs to Cloud Logging with `job.logs_policy.destination = CLOUD_LOGGING`.

For SDK-internal Python logging, Google's generated clients support the `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` environment variable. Example:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

## Common Pitfalls

- Package name and import path differ: install `google-cloud-batch`, import `batch_v1`.
- Do not confuse CLI credentials with ADC. If local auth works in `gcloud` but fails in Python, run `gcloud auth application-default login`.
- Batch is regional. A valid project with the wrong region string still fails.
- A job can be syntactically valid but still fail at runtime because the VM service account, network, subnet, or image permissions are wrong.
- If your project no longer has a usable default network, the simplest examples from the docs are not enough; add explicit networking or an instance template.
- `create_job()` returns the job resource immediately, not proof that all tasks succeeded. Check job state and logs afterward.
- `cancel_job()` and `delete_job()` are operations; wait for them.
- The generated clients warn against using the client object as a context manager if the transport is shared elsewhere, because exiting closes the transport.

## Version-Sensitive Notes For 0.20.0

- PyPI and the official changelog both show `0.20.0` as the current release line covered here as of `2026-03-12`.
- The `0.20.0` changelog notes auto-enabled mTLS support when a supported client certificate is present. If endpoint behavior changes unexpectedly, check `GOOGLE_API_USE_MTLS_ENDPOINT`, `GOOGLE_API_USE_CLIENT_CERTIFICATE`, and any explicit `client_options.api_endpoint` override.
- The `0.19.0` release added new provisioning-model support. If you are adapting older 2025 examples around instance provisioning, confirm the current Batch fields and enums in the latest API reference.
- PyPI still marks this package as beta, so generated client details can change faster than long-established Google Cloud libraries. Prefer the current reference and changelog over blog posts.

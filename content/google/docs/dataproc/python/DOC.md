---
name: dataproc
description: "Google Cloud Dataproc Python client for cluster management, job submission, and serverless batch workloads"
metadata:
  languages: "python"
  versions: "5.25.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,dataproc,gcp,python,spark,hadoop,serverless"
---

# Google Cloud Dataproc Python Package Guide

## Golden Rule

Use the official `google-cloud-dataproc` client library and treat Dataproc as a regional API surface. Install `google-cloud-dataproc`, authenticate with Application Default Credentials (ADC), and create clients with a regional endpoint such as `us-central1-dataproc.googleapis.com:443`.

As of March 12, 2026, PyPI lists `google-cloud-dataproc 5.25.0`. The `latest` reference site is still version-skewed across individual pages, so use PyPI for package-version truth and the reference docs for API shape.

## Install

```bash
python -m pip install --upgrade pip
python -m pip install "google-cloud-dataproc==5.25.0"
```

Common alternatives:

```bash
uv add "google-cloud-dataproc==5.25.0"
poetry add "google-cloud-dataproc==5.25.0"
```

Dataproc jobs usually reference code and data in Google Cloud Storage, so many projects also install `google-cloud-storage` separately when they need to upload artifacts before submission.

## Authentication And Setup

Dataproc uses Google Cloud credentials, not API keys.

Recommended local setup:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
gcloud services enable dataproc.googleapis.com
```

Service account setup for CI or servers:

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
export GOOGLE_CLOUD_PROJECT=your-project-id
export DATAPROC_REGION=us-central1
```

Prefer workload identity or an attached service account in production instead of long-lived JSON keys.

## Initialize Clients

The package exposes multiple service clients. For most coding tasks, the important ones are:

- `ClusterControllerClient` for cluster lifecycle
- `JobControllerClient` for jobs that run on an existing cluster
- `BatchControllerClient` for Dataproc Serverless batches

Create them with a regional endpoint:

```python
import os

from google.api_core.client_options import ClientOptions
from google.cloud import dataproc_v1

PROJECT_ID = os.environ["GOOGLE_CLOUD_PROJECT"]
REGION = os.getenv("DATAPROC_REGION", "us-central1")

def regional_client_options(region: str) -> ClientOptions:
    return ClientOptions(api_endpoint=f"{region}-dataproc.googleapis.com:443")

cluster_client = dataproc_v1.ClusterControllerClient(
    client_options=regional_client_options(REGION)
)
job_client = dataproc_v1.JobControllerClient(
    client_options=regional_client_options(REGION)
)
batch_client = dataproc_v1.BatchControllerClient(
    client_options=regional_client_options(REGION)
)
```

If you need explicit credentials:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import dataproc_v1
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = dataproc_v1.JobControllerClient(
    credentials=credentials,
    client_options=ClientOptions(
        api_endpoint="us-central1-dataproc.googleapis.com:443"
    ),
)
```

## Core Usage Patterns

### 1. Create Or List Clusters

Use `ClusterControllerClient` when you manage classic Dataproc clusters yourself.

```python
from google.api_core.client_options import ClientOptions
from google.cloud import dataproc_v1

region = "us-central1"
project_id = "your-project-id"

cluster_client = dataproc_v1.ClusterControllerClient(
    client_options=ClientOptions(
        api_endpoint=f"{region}-dataproc.googleapis.com:443"
    )
)

cluster = {
    "cluster_name": "analytics-dev",
    "config": {
        "gce_cluster_config": {
            "zone_uri": "us-central1-a",
        },
        "master_config": {
            "num_instances": 1,
            "machine_type_uri": "e2-standard-4",
        },
        "worker_config": {
            "num_instances": 2,
            "machine_type_uri": "e2-standard-4",
        },
    },
}

operation = cluster_client.create_cluster(
    request={
        "project_id": project_id,
        "region": region,
        "cluster": cluster,
    }
)

created_cluster = operation.result(timeout=1800)
print(created_cluster.cluster_uuid)

for item in cluster_client.list_clusters(
    request={"project_id": project_id, "region": region}
):
    print(item.cluster_name, item.status.state)
```

Use `.result()` on long-running operations. `create_cluster`, `update_cluster`, and `delete_cluster` do not complete immediately.

### 2. Submit A Job To An Existing Cluster

Use `JobControllerClient` when you already have a cluster and want to run a Spark, PySpark, Hive, or Hadoop job on it.

```python
from google.api_core.client_options import ClientOptions
from google.cloud import dataproc_v1

region = "us-central1"
project_id = "your-project-id"
cluster_name = "analytics-dev"

job_client = dataproc_v1.JobControllerClient(
    client_options=ClientOptions(
        api_endpoint=f"{region}-dataproc.googleapis.com:443"
    )
)

job = {
    "placement": {
        "cluster_name": cluster_name,
    },
    "pyspark_job": {
        "main_python_file_uri": "gs://your-bucket/jobs/main.py",
        "args": ["--date", "2026-03-12"],
    },
}

operation = job_client.submit_job_as_operation(
    request={
        "project_id": project_id,
        "region": region,
        "job": job,
    }
)

submitted_job = operation.result(timeout=1800)
print(submitted_job.reference.job_id)
print(submitted_job.driver_output_resource_uri)
```

Use `submit_job()` if you want the `Job` resource back immediately and will poll status yourself. Use `submit_job_as_operation()` when you want the operation flow.

### 3. Run A Dataproc Serverless Batch

Use `BatchControllerClient` for Dataproc Serverless. This is a separate API surface from jobs on a long-lived cluster.

```python
from google.api_core.client_options import ClientOptions
from google.cloud import dataproc_v1

region = "us-central1"
project_id = "your-project-id"

batch_client = dataproc_v1.BatchControllerClient(
    client_options=ClientOptions(
        api_endpoint=f"{region}-dataproc.googleapis.com:443"
    )
)

batch = {
    "pyspark_batch": {
        "main_python_file_uri": "gs://your-bucket/jobs/etl.py",
        "args": ["--env", "dev"],
    },
    "runtime_config": {
        "version": "2.2",
        "properties": {
            "spark.executor.instances": "2",
        },
    },
    "environment_config": {
        "execution_config": {
            "service_account": "dataproc-runner@your-project-id.iam.gserviceaccount.com",
            "subnetwork_uri": (
                "projects/your-project-id/regions/us-central1/subnetworks/default"
            ),
        },
    },
}

operation = batch_client.create_batch(
    request={
        "parent": f"projects/{project_id}/regions/{region}",
        "batch": batch,
        "batch_id": "daily-etl-20260312",
        "request_id": "c1d9b0b1-2b4e-4a47-a03d-9b6d5d118df8",
    }
)

created_batch = operation.result(timeout=1800)
print(created_batch.name)
print(created_batch.state)
```

The `request_id` field is worth setting when retries are possible. It helps make create requests safer to replay after transport failures.

### 4. Poll Or Enumerate Existing Batches

```python
parent = f"projects/{project_id}/regions/{region}"

for batch in batch_client.list_batches(request={"parent": parent}):
    print(batch.name, batch.state)
```

## Configuration Notes

- Dataproc is region-scoped. Keep `region`, `parent`, and the endpoint aligned.
- GCS URIs such as `gs://bucket/path/file.py` are common in request payloads. Local file paths are not uploaded automatically.
- Many request parameters can be passed as typed protobuf objects or plain Python dictionaries. Dictionaries are usually faster for agents to assemble correctly.
- Client methods accept retry and timeout options through `google-api-core` if you need stricter operational behavior.

## Common Pitfalls

- Install name and import name differ. Use `pip install google-cloud-dataproc`, then `from google.cloud import dataproc_v1`.
- Do not use the global endpoint by accident. The reference docs explicitly call out regional endpoints for Dataproc clients.
- Do not mix cluster jobs and serverless batches. `JobControllerClient` targets an existing cluster; `BatchControllerClient` targets Dataproc Serverless.
- Long-running operations are common. Call `.result()` or store the operation name and poll deliberately.
- The package does not upload your local code. Put scripts, jars, and dependency archives in GCS first.
- `latest` reference pages are not perfectly version-synchronized. Cross-check version-sensitive behavior against PyPI and the Dataproc changelog before copying snippets blindly.
- Some batch and cluster features are region- or image-version-sensitive. If a field looks valid in the reference docs but the API rejects it, confirm the Dataproc image/runtime version and region support.

## Version-Sensitive Notes For 5.25.0

- PyPI shows `5.25.0` as the current package version, released on February 19, 2026.
- The official changelog for the `latest` docs includes 5.25.0 changes, including `ClusterType` support for zero-scale clusters.
- Individual API reference pages under the `latest` docs root still show older page-version labels on some services. Treat the package version and the page heading as separate signals.

## Official Sources

- Dataproc Python reference: `https://cloud.google.com/python/docs/reference/dataproc/latest`
- Dataproc changelog: `https://cloud.google.com/python/docs/reference/dataproc/latest/changelog`
- `ClusterControllerClient` reference: `https://cloud.google.com/python/docs/reference/dataproc/latest/google.cloud.dataproc_v1.services.cluster_controller.ClusterControllerClient`
- `JobControllerClient` reference: `https://cloud.google.com/python/docs/reference/dataproc/latest/google.cloud.dataproc_v1.services.job_controller.JobControllerClient`
- `BatchControllerClient` reference: `https://cloud.google.com/python/docs/reference/dataproc/latest/google.cloud.dataproc_v1.services.batch_controller.BatchControllerClient`
- Google Cloud ADC guide: `https://cloud.google.com/docs/authentication/application-default-credentials`
- PyPI project page: `https://pypi.org/project/google-cloud-dataproc/`

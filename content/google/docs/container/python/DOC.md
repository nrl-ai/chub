---
name: container
description: "Google Kubernetes Engine Python client guide with ADC setup, ClusterManagerClient usage, operation polling, and location/name pitfalls"
metadata:
  languages: "python"
  versions: "2.63.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,gke,kubernetes,container,gcp,adc"
---

# google-cloud-container Python Package Guide

## What This Package Is

`google-cloud-container` is the official Google Cloud Python client for the Google Kubernetes Engine control-plane API.

Use it when Python code needs to:

- list, inspect, create, update, or delete GKE clusters
- manage node pools and server config
- poll GKE operations from application code

Do not use this package for normal Kubernetes object access inside a cluster. For Deployments, Services, Pods, and other Kubernetes resources, use the Kubernetes Python client against the cluster endpoint after you obtain cluster credentials.

## Version Note

This guide covers package version `2.63.0`, which PyPI lists as the latest visible release as of `2026-03-12`.

Official upstream sources checked on `2026-03-12` do not fully agree on package freshness:

- PyPI lists `2.63.0` as the latest release
- the Google Cloud Python docs overview page under `latest` currently renders the package as `2.62.0`
- the `latest` docs changelog page currently stops at `2.61.0`

Practical implication:

- the examples below are grounded in the current GA `container_v1` client surface
- if you depend on a field added very recently, verify it against the installed wheel, not just the rolling docs site
- treat the docs site as slightly lagging behind the published PyPI package for this library

Official sources used for this guide:

- Docs root: `https://cloud.google.com/python/docs/reference/container/latest`
- Cluster manager reference: `https://cloud.google.com/python/docs/reference/container/latest/google.cloud.container_v1.services.cluster_manager.ClusterManagerClient`
- Changelog: `https://cloud.google.com/python/docs/reference/container/latest/changelog`
- Authentication: `https://cloud.google.com/docs/authentication/provide-credentials-adc`
- Registry: `https://pypi.org/project/google-cloud-container/`

## Install

Install the library:

```bash
python -m pip install google-cloud-container
```

If you want the current upstream release that was visible on that date:

```bash
python -m pip install "google-cloud-container==2.63.0"
```

PyPI currently declares `Requires: Python >=3.7`.

## Required Setup

Before calling the client, Google’s package docs and authentication docs require the usual Google Cloud setup:

1. Select or create a Google Cloud project.
2. Enable billing for the project.
3. Enable the Kubernetes Engine API.
4. Set up Application Default Credentials.

Enable the API:

```bash
gcloud services enable container.googleapis.com
```

Local development with user ADC:

```bash
gcloud auth application-default login
```

Service account credentials via environment variable:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

Your credential needs permission to call GKE cluster-management APIs in the target project.

## Imports And Client Creation

Use the GA surface by default:

```python
from google.cloud import container_v1

client = container_v1.ClusterManagerClient()
```

Async client:

```python
from google.cloud import container_v1

client = container_v1.ClusterManagerAsyncClient()
```

Explicit service account file:

```python
from google.cloud import container_v1

client = container_v1.ClusterManagerClient.from_service_account_file(
    "/absolute/path/service-account.json"
)
```

REST transport instead of gRPC:

```python
from google.cloud import container_v1

client = container_v1.ClusterManagerClient(transport="rest")
```

Use `container_v1beta1` only if you intentionally need beta-only fields. For stable automation, prefer `container_v1`.

## Resource Naming

The current generated client expects fully qualified resource names.

Use these formats:

- parent location: `projects/{project_id}/locations/{location}`
- cluster name: `projects/{project_id}/locations/{location}/clusters/{cluster_id}`
- operation name: `projects/{project_id}/locations/{location}/operations/{operation_id}`

`location` can be a region such as `us-central1` or a zone such as `us-central1-a`, depending on whether the cluster is regional or zonal.

Important compatibility note:

- older request shapes used separate `project_id`, `zone`, and `cluster_id` fields
- the current reference marks those older fields as deprecated in favor of `parent` and `name`

For new code, always build `parent` and `name` explicitly.

## Core Usage

### List Clusters In A Location

```python
from google.cloud import container_v1

def list_clusters(project_id: str, location: str) -> list[container_v1.Cluster]:
    client = container_v1.ClusterManagerClient()
    parent = f"projects/{project_id}/locations/{location}"
    response = client.list_clusters(request={"parent": parent})
    return list(response.clusters)

for cluster in list_clusters("my-project", "us-central1"):
    print(cluster.name, cluster.endpoint)
```

`list_clusters` returns a `ListClustersResponse`, not a pager object. Read `response.clusters` directly.

### Get One Cluster

```python
from google.cloud import container_v1

def get_cluster(project_id: str, location: str, cluster_id: str) -> container_v1.Cluster:
    client = container_v1.ClusterManagerClient()
    name = f"projects/{project_id}/locations/{location}/clusters/{cluster_id}"
    return client.get_cluster(request={"name": name})

cluster = get_cluster("my-project", "us-central1", "primary")
print(cluster.name)
print(cluster.endpoint)
print(cluster.current_master_version)
```

### Inspect Available Control-Plane Versions

Use `get_server_config` before hard-coding Kubernetes versions in automation:

```python
from google.cloud import container_v1

def get_server_config(project_id: str, location: str) -> container_v1.ServerConfig:
    client = container_v1.ClusterManagerClient()
    name = f"projects/{project_id}/locations/{location}"
    return client.get_server_config(request={"name": name})

config = get_server_config("my-project", "us-central1")
print(config.default_cluster_version)
print(list(config.valid_master_versions)[:5])
```

### Create A Cluster

`create_cluster` returns a GKE operation resource. It does not block until the cluster is ready.

```python
from google.cloud import container_v1

def create_cluster(project_id: str, location: str, cluster_id: str) -> str:
    client = container_v1.ClusterManagerClient()
    parent = f"projects/{project_id}/locations/{location}"

    operation = client.create_cluster(
        request={
            "parent": parent,
            "cluster": container_v1.Cluster(
                name=cluster_id,
                initial_node_count=1,
                node_config=container_v1.NodeConfig(
                    machine_type="e2-standard-4",
                ),
            ),
        }
    )
    return operation.name
```

Real production cluster creation usually needs more than the minimal example above, such as networking, release channel, workload identity, autoscaling, or private-cluster settings. Start with the minimal request shape, then add product-specific cluster configuration deliberately.

### Poll A Long-Running Operation

```python
import time

from google.cloud import container_v1

def wait_for_operation(operation_name: str, poll_seconds: int = 10) -> container_v1.Operation:
    client = container_v1.ClusterManagerClient()

    while True:
        operation = client.get_operation(request={"name": operation_name})
        if operation.status == container_v1.Operation.Status.DONE:
            return operation
        time.sleep(poll_seconds)
```

If you need to cancel an operation, the same client also exposes `cancel_operation`.

### Delete A Cluster

```python
from google.cloud import container_v1

def delete_cluster(project_id: str, location: str, cluster_id: str) -> str:
    client = container_v1.ClusterManagerClient()
    name = f"projects/{project_id}/locations/{location}/clusters/{cluster_id}"
    operation = client.delete_cluster(request={"name": name})
    return operation.name
```

## Authentication And Configuration Notes

### ADC Is The Default

This client is designed around Application Default Credentials. In practice, the safest patterns are:

- local development: `gcloud auth application-default login`
- deployed workloads on Google Cloud: attach a service account and let ADC discover it
- external workloads: point `GOOGLE_APPLICATION_CREDENTIALS` at a service account key only when workload identity federation is not available

### Endpoint Overrides

The generated client constructor accepts `client_options=` if you need to override the API endpoint for a specialized environment:

```python
from google.cloud import container_v1
from google.api_core.client_options import ClientOptions

client = container_v1.ClusterManagerClient(
    client_options=ClientOptions(
        api_endpoint="container.googleapis.com",
    )
)
```

For normal public Google Cloud usage, the default endpoint is the right choice.

### Logging

The PyPI package page documents standard Google client-library logging support through `GOOGLE_SDK_PYTHON_LOGGING_SCOPE`.

Example:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google.cloud.container_v1
```

## Common Pitfalls

### Confusing GKE Control-Plane Calls With Kubernetes API Calls

`google-cloud-container` manages clusters and node pools through Google Cloud APIs. It does not replace the Kubernetes client for in-cluster resource CRUD.

### Using Deprecated Request Fields

If you copy old examples that pass `project_id`, `zone`, or `cluster_id` separately, you can end up mixing old and new request shapes. Prefer the current `parent` and `name` fields everywhere.

### Forgetting To Poll Operations

Cluster creation, updates, and deletes are asynchronous. Treat the returned `Operation` as a long-running status object and poll `get_operation` until it is done.

### Mixing Zonal And Regional Names

Use the exact location where the cluster lives. A regional cluster name such as `projects/my-project/locations/us-central1/clusters/my-cluster` is different from a zonal cluster name under `us-central1-a`.

### Assuming `latest` Docs Match Installed Code

For this package, PyPI currently shows `2.63.0`, the docs overview page shows `2.62.0`, and the published changelog page stops at `2.61.0`. If a method or field appears new, verify it against the installed package before relying on it in generated code.

### Reaching For `container_v1beta1` Without A Reason

The beta surface exists, but GA automation should start on `container_v1`. Only switch to `container_v1beta1` when you have confirmed that the needed field or method is not available in GA.

## Practical Defaults For Agents

- Import `from google.cloud import container_v1`.
- Build resource names explicitly with `projects/.../locations/...`.
- Use `ClusterManagerClient()` with ADC.
- Treat create, update, and delete calls as asynchronous and poll `get_operation`.
- Use the Kubernetes Python client separately for workload objects inside the cluster.

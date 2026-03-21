---
name: alloydb
description: "google-cloud-alloydb Python package guide for managing AlloyDB clusters, instances, backups, users, and admin SQL through the AlloyDB Admin API"
metadata:
  languages: "python"
  versions: "0.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,alloydb,gcp,database,postgresql,admin"
---

# google-cloud-alloydb Python Package Guide

## What This Package Is For

`google-cloud-alloydb` is the generated Python client library for the AlloyDB Admin API.

Use it when your code needs to:

- list or inspect AlloyDB clusters, instances, backups, users, databases, and operations
- create, update, or delete AlloyDB resources through Google Cloud APIs
- fetch instance connection metadata with `get_connection_info`
- run administrative SQL with `execute_sql`

Do not use this package as your normal PostgreSQL application driver. It is the control-plane SDK. For application connections to an AlloyDB instance, use the separate `google-cloud-alloydb-connector` package together with a driver such as `pg8000` or `asyncpg`.

The import surface is:

```python
from google.cloud import alloydb_v1
```

## Version-Sensitive Notes

- This entry keeps `0.8.0` in frontmatter because that was the version used here provided for review.
- Public upstream sources checked on 2026-03-12 did not verify a public `0.8.0` release. PyPI currently shows `0.7.0`.
- The hosted Google Cloud reference also lags PyPI: the `latest` changelog still tops out at `0.6.0`.
- Treat the docs root as the canonical API reference, but verify the exact installed package version before pinning dependencies or depending on release-specific behavior.
- PyPI metadata says the package supports Python `>=3.7`.

## Install

Use the public package name from PyPI:

```bash
pip install google-cloud-alloydb
```

Pin only after checking what your index actually serves:

```bash
pip install "google-cloud-alloydb==0.7.0"
```

If your project genuinely depends on a private or mirrored `0.8.0`, confirm that in your artifact registry before copying the pin above.

## Authentication And Environment Setup

This library uses Application Default Credentials (ADC).

Local development:

```bash
gcloud init
gcloud services enable alloydb.googleapis.com
gcloud auth application-default login
```

Service account based auth:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="my-project"
```

Practical permission guidance:

- for admin operations, start with `roles/alloydb.admin`
- to create a cluster, Google Cloud also requires `compute.networks.list`; the docs call out `roles/compute.networkUser` as the least-privilege way to get that permission
- cluster creation also depends on private services access and other AlloyDB networking prerequisites already being configured in the target project and region

## Initialize A Client

Use the GA `v1` surface unless you specifically need a method that only exists on a preview surface:

```python
from google.cloud import alloydb_v1

client = alloydb_v1.AlloyDBAdminClient()
```

Prefer full resource names in requests:

```python
project = "my-project"
region = "us-central1"
cluster = "my-cluster"
instance = "my-primary"

location_name = f"projects/{project}/locations/{region}"
cluster_name = f"{location_name}/clusters/{cluster}"
instance_name = f"{cluster_name}/instances/{instance}"
```

Most request fields expect these full resource names, not short IDs.

## Core Read Workflows

### List Clusters

```python
from google.cloud import alloydb_v1

client = alloydb_v1.AlloyDBAdminClient()
request = alloydb_v1.ListClustersRequest(
    parent="projects/my-project/locations/us-central1"
)

for cluster in client.list_clusters(request=request):
    print(cluster.name, cluster.state)
```

`list_*` methods return pagers. Iterate directly unless you need custom page-token handling.

### List Instances In A Cluster

```python
from google.cloud import alloydb_v1

client = alloydb_v1.AlloyDBAdminClient()
request = alloydb_v1.ListInstancesRequest(
    parent="projects/my-project/locations/us-central1/clusters/my-cluster"
)

for instance in client.list_instances(request=request):
    print(instance.name, instance.instance_type, instance.state)
```

### Fetch Connection Metadata

Use `get_connection_info` when you need admin-plane connection details for an instance:

```python
from google.cloud import alloydb_v1

client = alloydb_v1.AlloyDBAdminClient()
request = alloydb_v1.GetConnectionInfoRequest(
    parent=(
        "projects/my-project/locations/us-central1/"
        "clusters/my-cluster/instances/my-primary"
    )
)

info = client.get_connection_info(request=request)
print(info)
```

### Run Administrative SQL

`execute_sql` is for controlled admin tasks, not for normal application query traffic.

```python
import os
from google.cloud import alloydb_v1

client = alloydb_v1.AlloyDBAdminClient()
request = alloydb_v1.ExecuteSqlRequest(
    instance=(
        "projects/my-project/locations/us-central1/"
        "clusters/my-cluster/instances/my-primary"
    ),
    database="postgres",
    user="postgres",
    password=os.environ["ALLOYDB_DB_PASSWORD"],
    sql_statement="SELECT current_database();",
)

response = client.execute_sql(request=request)
print(response)
```

`database` and `user` are plain database identifiers, not resource paths.

## Mutating Operations

Create, update, and delete calls typically return long-running operations. Wait on `.result()` before assuming the resource is ready.

```python
import uuid
from google.cloud import alloydb_v1

client = alloydb_v1.AlloyDBAdminClient()
request = alloydb_v1.CreateClusterRequest(
    parent="projects/my-project/locations/us-central1",
    cluster_id="dev-cluster",
    cluster=alloydb_v1.Cluster(
        display_name="dev-cluster",
        # Fill in the required network, initial user, and config fields.
    ),
    request_id=str(uuid.uuid4()),
)

operation = client.create_cluster(request=request)
created_cluster = operation.result(timeout=1800)
print(created_cluster.name)
```

Use a `request_id` on mutating calls when your caller might retry after a timeout or transport failure.

## Configuration Notes

### Endpoints

If your environment requires an endpoint override, pass `client_options` explicitly:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import alloydb_v1

client = alloydb_v1.AlloyDBAdminClient(
    client_options=ClientOptions(api_endpoint="YOUR_ALLOYDB_ENDPOINT")
)
```

### Async Usage

If your service is already async, use `AlloyDBAdminAsyncClient` instead of pushing the sync client through thread executors.

### Logging

The PyPI package page documents environment-based logging for Google client libraries:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google.cloud.alloydb_v1
```

Only enable this in environments where you are comfortable with RPC metadata appearing in logs.

## Common Pitfalls

- `google-cloud-alloydb` manages AlloyDB resources; it is not your regular Postgres connection library.
- Use full resource names like `projects/.../locations/.../clusters/...`, not bare IDs.
- Expect long-running operations for provisioning and destructive changes.
- `execute_sql` needs database credentials; it does not replace connection pooling or ordinary query execution.
- Cluster creation also depends on private networking prerequisites outside this Python package.
- Google-hosted reference pages can lag PyPI releases, so check both the installed version and the reference docs before relying on version-specific behavior.
- If you need app-level connections, use `google-cloud-alloydb-connector` with `pg8000` or `asyncpg` instead of trying to route runtime traffic through the admin client.

## Official Sources

- Google Cloud AlloyDB Python reference root: https://cloud.google.com/python/docs/reference/alloydb/latest
- AlloyDB changelog: https://cloud.google.com/python/docs/reference/alloydb/latest/changelog
- `AlloyDBAdminClient` reference: https://cloud.google.com/python/docs/reference/alloydb/latest/google.cloud.alloydb_v1.services.alloy_d_b_admin.AlloyDBAdminClient
- AlloyDB cluster creation guide: https://cloud.google.com/alloydb/docs/cluster-create
- Google Cloud client-library authentication guide: https://cloud.google.com/docs/authentication/client-libraries
- PyPI package page: https://pypi.org/project/google-cloud-alloydb/
- AlloyDB Python Connector package page: https://pypi.org/project/google-cloud-alloydb-connector/

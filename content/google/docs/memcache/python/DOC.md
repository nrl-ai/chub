---
name: memcache
description: "google-cloud-memcache package guide for Python with ADC setup, instance lifecycle operations, and docs-version drift notes"
metadata:
  languages: "python"
  versions: "1.14.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,memorystore,memcached,gcp,python,admin"
---

# google-cloud-memcache Python Package Guide

## What This Package Is

`google-cloud-memcache` is the official Google Cloud Python client library for the Memorystore for Memcached admin API.

Use it when your code needs to:

- create Memcached instances
- list or inspect instances
- update instance metadata or staged Memcached parameters
- apply staged parameter changes
- reschedule maintenance
- delete instances

Do not use this package for Memcached key/value traffic. After the instance exists, your application talks to the cache through a normal Memcached protocol client and the instance's discovery or node endpoints.

Primary import:

```python
from google.cloud import memcache_v1
```

Legacy surface also exists:

```python
from google.cloud import memcache_v1beta2
```

Prefer `memcache_v1` for new code.

## Version Note

This entry is pinned to the version used here `1.14.0`, and that version is confirmed by the official PyPI project page as the current release published on January 15, 2026.

Google's current `latest` Python reference pages for this library are lagging or internally inconsistent:

- the package landing page and several `memcache_v1` pages are still labeled `1.13.0`
- some deeper generated pages still render as `1.12.2`

The API surface in this guide is based on the current official `latest` reference pages plus the official PyPI package page for `1.14.0`. Where those sources disagree on displayed doc version labels, this guide favors the validated package version from PyPI and treats the reference-site version labels as documentation lag.

## Install

```bash
python -m pip install "google-cloud-memcache==1.14.0"
```

Unpinned install:

```bash
python -m pip install google-cloud-memcache
```

With `uv`:

```bash
uv add google-cloud-memcache
```

With Poetry:

```bash
poetry add google-cloud-memcache
```

The official PyPI page currently lists support for Python `>=3.7`.

## Setup And Authentication

Before calling the client, you need:

- a Google Cloud project with billing enabled
- the Memorystore for Memcached API enabled
- Application Default Credentials (ADC)
- a VPC network that can host the instance
- private services access configured for that network

Enable the API:

```bash
gcloud services enable memcache.googleapis.com
```

Set up ADC for local development:

```bash
gcloud auth application-default login
```

If you need service account impersonation for local work:

```bash
gcloud auth application-default login \
  --impersonate-service-account=SERVICE_ACCOUNT_EMAIL
```

If you must point ADC at a credential file:

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

Google's auth docs still recommend attached service accounts for production workloads on Google Cloud rather than shipping service account keys with your code.

## Client Creation

Default client with ADC:

```python
from google.cloud import memcache_v1

client = memcache_v1.CloudMemcacheClient()
```

Async client:

```python
from google.cloud import memcache_v1

client = memcache_v1.CloudMemcacheAsyncClient()
```

Explicit credentials object:

```python
from google.cloud import memcache_v1
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = memcache_v1.CloudMemcacheClient(credentials=credentials)
```

The generated reference examples warn that some environments may need explicit `client_options` endpoint configuration. Start with the default client and only add `client_options` when your environment actually requires it.

## Resource Naming

Memcached instances are regional resources:

```python
project_id = "my-project"
region = "us-central1"
parent = f"projects/{project_id}/locations/{region}"
instance_name = f"{parent}/instances/cache-prod"
```

The reference docs also document `projects/{project}/locations/-` for listing across all regions.

## Core Workflow

### List instances

```python
from google.cloud import memcache_v1

client = memcache_v1.CloudMemcacheClient()

for instance in client.list_instances(
    request={"parent": "projects/my-project/locations/us-central1"}
):
    print(instance.name, instance.state.name)
```

`list_instances()` returns a pager. Iterate over it directly.

### Get one instance

```python
from google.cloud import memcache_v1

client = memcache_v1.CloudMemcacheClient()

instance = client.get_instance(
    request={
        "name": "projects/my-project/locations/us-central1/instances/cache-prod"
    }
)

print(instance.discovery_endpoint)
print(instance.memcache_nodes)
print(instance.maintenance_schedule)
```

Useful fields from `Instance`:

- `authorized_network`
- `discovery_endpoint`
- `labels`
- `maintenance_policy`
- `maintenance_schedule`
- `memcache_nodes`
- `memcache_version`
- `node_config`
- `node_count`
- `state`

### Create an instance

`create_instance()` is a long-running operation. Wait for `operation.result()` before assuming the instance is usable.

```python
from google.cloud import memcache_v1

client = memcache_v1.CloudMemcacheClient()
parent = "projects/my-project/locations/us-central1"

instance = memcache_v1.Instance(
    display_name="Primary cache",
    authorized_network="projects/my-project/global/networks/default",
    node_count=3,
    node_config=memcache_v1.Instance.NodeConfig(
        cpu_count=2,
        memory_size_mb=4096,
    ),
    labels={"env": "prod"},
)

operation = client.create_instance(
    request={
        "parent": parent,
        "instance_id": "cache-prod",
        "instance": instance,
    }
)

created = operation.result(timeout=1800)
print(created.name)
print(created.discovery_endpoint)
```

Practical constraints from the official docs:

- `instance_id` must be 1 to 40 characters
- it must use lowercase letters, numbers, and hyphens
- it must start with a letter
- it must end with a letter or number
- `node_count` and `node_config` are required
- `authorized_network` should point at the VPC network meant to reach the cache

The product docs also note that a private services access connection must exist for the selected network before instance creation succeeds.

### Update instance metadata

Use a field mask. Do not assume the whole resource is safely patchable.

```python
from google.cloud import memcache_v1
from google.protobuf.field_mask_pb2 import FieldMask

client = memcache_v1.CloudMemcacheClient()

patch = memcache_v1.Instance(
    name="projects/my-project/locations/us-central1/instances/cache-prod",
    display_name="Cache production",
)

operation = client.update_instance(
    request={
        "update_mask": FieldMask(paths=["display_name"]),
        "instance": patch,
    }
)

updated = operation.result(timeout=1800)
print(updated.display_name)
```

### Stage Memcached parameter changes

`update_parameters()` stages parameter changes. It does not apply them to nodes by itself.

```python
from google.cloud import memcache_v1
from google.protobuf.field_mask_pb2 import FieldMask

client = memcache_v1.CloudMemcacheClient()

operation = client.update_parameters(
    request={
        "name": "projects/my-project/locations/us-central1/instances/cache-prod",
        "update_mask": FieldMask(paths=["params"]),
        "parameters": memcache_v1.MemcacheParameters(
            params={
                "max-item-size": "1048576",
                "lru_crawler": "yes",
            }
        ),
    }
)

operation.result(timeout=1800)
```

### Apply staged parameters

The generated client docs state that `ApplyParameters` restarts the specified nodes so they pick up the current staged parameter set.

```python
from google.cloud import memcache_v1

client = memcache_v1.CloudMemcacheClient()

operation = client.apply_parameters(
    request={
        "name": "projects/my-project/locations/us-central1/instances/cache-prod"
    }
)

operation.result(timeout=1800)
```

The product docs warn that applying parameter updates can flush cache data on affected nodes. Treat this as an operational change, not a harmless metadata edit.

### Reschedule maintenance

The reference types include `RescheduleMaintenanceRequest`, `MaintenancePolicy`, and `MaintenanceSchedule`.

```python
from datetime import datetime, timezone

from google.cloud import memcache_v1
from google.protobuf.timestamp_pb2 import Timestamp

client = memcache_v1.CloudMemcacheClient()

schedule_time = Timestamp()
schedule_time.FromDatetime(datetime(2026, 3, 20, 3, 0, tzinfo=timezone.utc))

operation = client.reschedule_maintenance(
    request={
        "instance": "projects/my-project/locations/us-central1/instances/cache-prod",
        "reschedule_type": (
            memcache_v1.RescheduleMaintenanceRequest.RescheduleType.SPECIFIC_TIME
        ),
        "schedule_time": schedule_time,
    }
)

operation.result(timeout=1800)
```

### Delete an instance

```python
from google.cloud import memcache_v1

client = memcache_v1.CloudMemcacheClient()

operation = client.delete_instance(
    request={
        "name": "projects/my-project/locations/us-central1/instances/cache-prod"
    }
)

operation.result(timeout=1800)
```

## Connecting Your Application To The Cache

This package stops at the admin API.

Typical split:

1. Use `google-cloud-memcache` to provision or inspect the Memorystore instance.
2. Read `discovery_endpoint` or node addresses from the returned `Instance`.
3. Configure a real Memcached client in your application to talk to those endpoints.

If generated code starts calling `client.get()` or `client.set()` on `CloudMemcacheClient`, it is using the wrong API surface.

## Logging

The official PyPI package page documents optional structured logging support through `GOOGLE_SDK_PYTHON_LOGGING_SCOPE`.

Example:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

Use that only when you explicitly want Google client library debug logging, and treat emitted RPC metadata as potentially sensitive.

## Common Pitfalls

### Using the admin client as a Memcached data client

This package manages instances. It does not implement Memcached `get`, `set`, `delete`, or CAS operations.

### Missing private services access

Creating an instance can fail even with correct IAM if the target VPC network does not already have private services access configured.

### Forgetting to wait on long-running operations

`create_instance()`, `update_instance()`, `update_parameters()`, `apply_parameters()`, `reschedule_maintenance()`, and `delete_instance()` are not immediate state changes.

### Assuming parameter updates are harmless

The product docs explicitly warn that applying updated parameters can flush cache data on affected nodes.

### Blindly trusting `latest` doc version labels

For this package on March 12, 2026, the official reference pages lag the official PyPI release. Pin generated install commands to `1.14.0`, but validate method availability against the current reference pages when you rely on newly added features.

## Official Source URLs

- `https://cloud.google.com/python/docs/reference/memcache/latest`
- `https://cloud.google.com/python/docs/reference/memcache/latest/google.cloud.memcache_v1.services.cloud_memcache.CloudMemcacheClient`
- `https://cloud.google.com/python/docs/reference/memcache/latest/google.cloud.memcache_v1.types.Instance`
- `https://cloud.google.com/python/docs/reference/memcache/latest/google.cloud.memcache_v1.types`
- `https://cloud.google.com/memorystore/docs/memcached`
- `https://cloud.google.com/memorystore/docs/memcached/configure-memcached`
- `https://cloud.google.com/memorystore/docs/memcached/establish-connection`
- `https://cloud.google.com/docs/authentication/client-libraries`
- `https://cloud.google.com/docs/authentication/application-default-credentials`
- `https://pypi.org/project/google-cloud-memcache/`

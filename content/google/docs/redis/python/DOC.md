---
name: redis
description: "google-cloud-redis package guide for Python with ADC setup, instance administration, long-running operations, and Memorystore-specific pitfalls"
metadata:
  languages: "python"
  versions: "2.20.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,memorystore,redis,gcp,python,admin-sdk"
---

# google-cloud-redis Python Package Guide

`google-cloud-redis` is the Python admin SDK for Google Cloud Memorystore for Redis. Use it for control-plane tasks such as listing instances, creating instances, updating configuration, exporting data, importing data, failing over, and deleting instances.

It is not the Redis protocol client you use inside application request paths. For reads, writes, pipelines, pub/sub, and Lua scripts against the Redis endpoint itself, use the separate `redis` package.

## Golden Rule

- Use `google-cloud-redis` for Memorystore administration.
- Use Application Default Credentials (ADC) instead of hardcoded credentials whenever possible.
- Treat create, update, export, import, failover, reschedule maintenance, upgrade, and delete calls as long-running operations.
- Use the library path helpers to build resource names instead of concatenating `projects/...` strings by hand.
- Use the separate `redis` package for application data-plane commands.

## Install

Pin to the package version you intend to use:

```bash
python -m pip install google-cloud-redis==2.20.0
```

If you also need to connect to the Redis endpoint from Python code, install `redis` separately:

```bash
python -m pip install google-cloud-redis==2.20.0 redis
```

PyPI currently lists Python `>=3.7` for this package. The official changelog also notes Python 3.14 support in the 2.19.0 line.

## Authentication And Project Setup

Google Cloud client libraries use ADC by default. For local development, the normal path is:

```bash
gcloud auth application-default login
gcloud config set project MY_PROJECT_ID
gcloud services enable redis.googleapis.com
```

In Google Cloud runtimes, prefer attaching a service account to the workload instead of shipping JSON key files.

If you must use an explicit credential file, point ADC at it:

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/absolute/path/service-account.json
export GOOGLE_CLOUD_PROJECT=MY_PROJECT_ID
```

Practical notes:

- The admin API is `redis.googleapis.com`.
- Most requests need a project ID and region.
- The region is part of the parent resource name: `projects/<project>/locations/<region>`.
- Avoid baking credentials into code. Let ADC resolve them from the environment or runtime metadata server.

## Initialize The Client

Start with the synchronous client unless the rest of your codebase is already async:

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
project_id = "my-project"
region = "us-central1"

parent = client.common_location_path(project_id, region)

for instance in client.list_instances(request={"parent": parent}):
    print(instance.name)
    print(instance.host, instance.port)
    print(instance.state.name)
```

Useful path helpers:

- `client.common_location_path(project_id, region)`
- `client.instance_path(project_id, region, instance_id)`

If you need a non-default endpoint or other transport configuration, pass `client_options` when constructing the client. The generated reference explicitly supports overriding `api_endpoint` that way.

## Core Usage

## List Instances

`list_instances()` returns a pager. Iterate over it directly:

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
parent = client.common_location_path("my-project", "us-central1")

for instance in client.list_instances(request={"parent": parent}):
    print(instance.name)
    print(instance.tier.name)
    print(instance.redis_version)
    print(instance.host, instance.port)
```

## Get One Instance

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
name = client.instance_path("my-project", "us-central1", "cache-dev")

instance = client.get_instance(request={"name": name})

print(instance.name)
print(instance.host)
print(instance.port)
print(instance.memory_size_gb)
print(instance.redis_version)
print(instance.authorized_network)
```

`Instance` exposes operationally useful fields including:

- `host` and `port` for runtime client connections
- `authorized_network`
- `connect_mode`
- `transit_encryption_mode`
- `redis_version`
- `read_endpoint` and `read_endpoint_port` when read replicas are available
- `available_maintenance_versions`

## Create An Instance

Create calls return a long-running operation. Wait for completion before using the instance.

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
project_id = "my-project"
region = "us-central1"

instance = redis_v1.Instance(
    tier=redis_v1.Instance.Tier.BASIC,
    memory_size_gb=1,
    authorized_network=f"projects/{project_id}/global/networks/default",
    redis_version="REDIS_7_0",
)

operation = client.create_instance(
    request={
        "parent": client.common_location_path(project_id, region),
        "instance_id": "cache-dev",
        "instance": instance,
    }
)

created = operation.result(timeout=1800)
print(created.name)
print(created.host, created.port)
```

Choose these fields deliberately:

- `tier`: `BASIC` vs `STANDARD_HA`
- `memory_size_gb`
- `authorized_network`
- `redis_version`
- `connect_mode`
- `transit_encryption_mode`
- `labels`, `display_name`, maintenance settings, and Redis config knobs for production instances

## Update An Instance

Use an update mask. Without it, partial updates are easy to get wrong.

```python
from google.cloud import redis_v1
from google.protobuf import field_mask_pb2

client = redis_v1.CloudRedisClient()
name = client.instance_path("my-project", "us-central1", "cache-dev")

instance = redis_v1.Instance(
    name=name,
    display_name="cache-dev-updated",
)

operation = client.update_instance(
    request={
        "update_mask": field_mask_pb2.FieldMask(paths=["display_name"]),
        "instance": instance,
    }
)

updated = operation.result(timeout=1800)
print(updated.display_name)
```

## Export And Import

Export and import are also long-running operations and usually require Cloud Storage permissions in addition to Memorystore permissions.

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
name = client.instance_path("my-project", "us-central1", "cache-prod")

operation = client.export_instance(
    request={
        "name": name,
        "output_config": {
            "gcs_destination": {
                "uri": "gs://my-backups/redis/cache-prod.rdb",
            }
        },
    }
)

operation.result(timeout=1800)
```

For import, the structure is similar but uses `input_config`.

## Fail Over And Maintenance Operations

High-availability operational methods also return long-running operations:

- `failover_instance()`
- `reschedule_maintenance()`
- `upgrade_instance()`

Example:

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
name = client.instance_path("my-project", "us-central1", "cache-prod")

operation = client.failover_instance(
    request={
        "name": name,
        "data_protection_mode": (
            redis_v1.FailoverInstanceRequest.DataProtectionMode.LIMITED_DATA_LOSS
        ),
    }
)

operation.result(timeout=1800)
```

Use failover only for instances and runbooks that are prepared for it.

## Delete An Instance

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
name = client.instance_path("my-project", "us-central1", "cache-dev")

operation = client.delete_instance(request={"name": name})
operation.result(timeout=1800)
```

## Get The AUTH String

If AUTH is enabled, retrieve it from the admin API instead of assuming it is embedded in `get_instance()` output:

```python
from google.cloud import redis_v1

client = redis_v1.CloudRedisClient()
name = client.instance_path("my-project", "us-central1", "cache-prod")

response = client.get_instance_auth_string(request={"name": name})
print(response.auth_string)
```

## Connect To The Redis Endpoint From Application Code

Use the admin SDK to discover the connection endpoint, then use `redis` for actual Redis commands:

```python
from google.cloud import redis_v1
import redis

admin = redis_v1.CloudRedisClient()
name = admin.instance_path("my-project", "us-central1", "cache-prod")
instance = admin.get_instance(request={"name": name})

r = redis.Redis(host=instance.host, port=instance.port, decode_responses=True)
print(r.ping())
```

That split matters:

- `google-cloud-redis` manages the Memorystore resource.
- `redis` talks Redis protocol to the running instance.

## Async Client

If your code is already async, the library exposes `CloudRedisAsyncClient` with the same request shapes:

```python
import asyncio
from google.cloud import redis_v1

async def main() -> None:
    client = redis_v1.CloudRedisAsyncClient()
    parent = client.common_location_path("my-project", "us-central1")

    async for instance in client.list_instances(request={"parent": parent}):
        print(instance.name)

asyncio.run(main())
```

## Configuration And Authentication Pitfalls

- Do not confuse the package with `redis`. This library does not expose Redis data commands like `get`, `set`, or pipelines.
- Use full resource names for `parent` and `name`. Path helpers are safer than manual string assembly.
- Wait on `operation.result()` for every long-running call. A returned operation object does not mean the underlying change is finished.
- Use an explicit update mask for `update_instance()`.
- `authorized_network` expects a full VPC network resource path such as `projects/<project>/global/networks/<network>`.
- Export and import need Cloud Storage access as well as Memorystore access. Instance metadata also exposes `persistence_iam_identity`, which matters when troubleshooting storage permissions.
- `read_endpoint` is not the same thing as the primary write endpoint. Keep your runtime client routing aligned with the instance topology you actually provisioned.
- The generated client can be used as a context manager, but doing so closes the underlying transport. Do not use `with CloudRedisClient()` if that transport will be shared elsewhere.
- In multiprocessing scenarios, create the client after `os.fork()`, not before.

## Version-Sensitive Notes

- Frontmatter tracks `2.20.0` because that is the version used here and the current PyPI release.
- Official upstream pages are not perfectly in sync on `2026-03-12`. PyPI lists `2.20.0`, some reference pages such as the `Instance` type page also show `2.20.0`, while the generated `CloudRedisClient` page and the official changelog page still show `2.19.0 (latest)`.
- The official changelog records these recent changes:
  - `2.20.0`: automatic mTLS enablement when Google API client certificates are available and the environment opts in, plus generated client updates around version handling
  - `2.19.0`: Python 3.14 support and deprecation of the old `credentials_file` argument in generated clients
  - `2.18.1`: protobuf 6.x compatibility
  - `2.17.0`: opt-in debug logging support through `GOOGLE_SDK_PYTHON_LOGGING_SCOPE`
- Because the public reference pages are mixed, verify the exact constructor arguments and generated helper behavior against the live reference if you are relying on very recent `2.20.x` behavior.

## Official Sources Used For This Entry

- Google Cloud Python reference root: `https://cloud.google.com/python/docs/reference/redis/latest`
- `CloudRedisClient` reference: `https://cloud.google.com/python/docs/reference/redis/latest/google.cloud.redis_v1.services.cloud_redis.CloudRedisClient`
- `Instance` reference: `https://cloud.google.com/python/docs/reference/redis/latest/google.cloud.redis_v1.types.Instance`
- Google Cloud ADC overview: `https://cloud.google.com/docs/authentication/application-default-credentials`
- Google Cloud client library auth guidance: `https://cloud.google.com/docs/authentication/client-libraries`
- Official changelog: `https://cloud.google.com/python/docs/reference/redis/latest/changelog`
- PyPI package page: `https://pypi.org/project/google-cloud-redis/`

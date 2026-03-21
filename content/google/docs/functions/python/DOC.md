---
name: functions
description: "Google Cloud Functions Python client for managing and deploying 1st gen and 2nd gen Cloud Functions resources"
metadata:
  languages: "python"
  versions: "1.22.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,functions,cloud-functions,gcp,serverless"
---

# Google Cloud Functions Python Client

Use `google-cloud-functions` when you need the Google Cloud control-plane client for Cloud Functions from Python: list functions, inspect config, discover runtimes, deploy updates, and manage IAM. Prefer `functions_v2` for current 2nd gen workflows. Use `functions_v1` only when you are intentionally working with legacy 1st gen functions or the v1-only `call_function` API.

```python
from google.cloud import functions_v2
```

## What This Package Is For

This package is an admin SDK, not a runtime helper for function handler code.

Use it to:

- list and inspect deployed functions
- create, update, and delete functions
- discover supported runtimes
- manage IAM and deployment metadata
- generate signed upload or download URLs for source bundles

Do not use it to call an HTTP-triggered function URL. For that, send a normal HTTP request to the deployed endpoint. The one direct invocation helper in this package, `call_function`, is part of the v1 client surface for legacy 1st gen functions.

## Install

Pin the version your project expects:

```bash
python -m pip install "google-cloud-functions==1.22.0"
```

Common alternatives:

```bash
uv add "google-cloud-functions==1.22.0"
poetry add "google-cloud-functions==1.22.0"
```

## Setup And Authentication

Typical prerequisites:

- a Google Cloud project with billing enabled
- the Cloud Functions API enabled
- IAM permissions for the functions and locations you need to manage
- a chosen region such as `us-central1`

The client uses Application Default Credentials (ADC). For local development:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="my-project"
export GOOGLE_CLOUD_REGION="us-central1"
```

If you must use a credential file explicitly:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="my-project"
export GOOGLE_CLOUD_REGION="us-central1"
```

For production on Google Cloud, prefer an attached service account instead of shipping key files. ADC checks, in order, `GOOGLE_APPLICATION_CREDENTIALS`, the local ADC file created by `gcloud auth application-default login`, then the attached service account from the metadata server.

Explicit credentials also work:

```python
from google.cloud import functions_v2
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = functions_v2.FunctionServiceClient(credentials=credentials)
```

## Client Structure

The package exposes both generations:

- `functions_v2.FunctionServiceClient`: preferred for current deployments and admin workflows
- `functions_v2.FunctionServiceAsyncClient`: async variant
- `functions_v1.CloudFunctionsServiceClient`: legacy 1st gen surface, including `call_function`

Important behavior:

- Most mutating methods return a long-running operation. Call `operation.result(...)` and wait.
- Resource names are usually full paths like `projects/my-project/locations/us-central1/functions/my-fn`.
- The client has helper builders such as `common_location_path()` and `function_path()`.
- Use either flattened arguments or `request=...`, not both in the same call.

## Core Usage

### Create a v2 client and location parent

```python
import os
from google.cloud import functions_v2

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
region = os.environ.get("GOOGLE_CLOUD_REGION", "us-central1")

client = functions_v2.FunctionServiceClient()
parent = client.common_location_path(project_id, region)
```

### List functions in a region

```python
for fn in client.list_functions(parent=parent):
    uri = fn.service_config.uri if fn.service_config else None
    print(fn.name, fn.environment, fn.state, uri)
```

### Get one function by resource name

```python
function_name = client.function_path(project_id, region, "hello-function")
fn = client.get_function(name=function_name)

print(fn.name)
print(fn.build_config.runtime)
print(fn.service_config.service_account_email)
print(fn.service_config.uri)
```

### Discover supported runtimes

Use this instead of hard-coding a runtime string when you are generating deploy code:

```python
for runtime in client.list_runtimes(parent=parent):
    print(runtime.name, runtime.environment, runtime.stage)
```

### Update a deployed function safely

`update_function` accepts a `FieldMask`. Use it. Without a mask, the request updates all fields present on the function object.

```python
from google.cloud import functions_v2
from google.protobuf.field_mask_pb2 import FieldMask

function_name = client.function_path(project_id, region, "hello-function")
fn = client.get_function(name=function_name)

fn.service_config.environment_variables["APP_ENV"] = "prod"
fn.service_config.max_instance_count = 5

operation = client.update_function(
    request=functions_v2.UpdateFunctionRequest(
        function=fn,
        update_mask=FieldMask(
            paths=[
                "service_config.environment_variables",
                "service_config.max_instance_count",
            ]
        ),
    )
)

updated = operation.result(timeout=1800)
print(updated.update_time)
```

### Create a function from source already in Cloud Storage

For `StorageSource`, the referenced object is a gzipped archive (`.tar.gz`), not a local source tree.

```python
from google.cloud import functions_v2

function_id = "hello-function"

function = functions_v2.Function(
    build_config=functions_v2.BuildConfig(
        runtime="python312",
        entry_point="hello_http",
        source=functions_v2.Source(
            storage_source=functions_v2.StorageSource(
                bucket="my-source-bucket",
                object_="hello-function.tar.gz",
            )
        ),
    ),
    service_config=functions_v2.ServiceConfig(
        available_memory="512M",
        timeout_seconds=60,
        environment_variables={"APP_ENV": "prod"},
    ),
)

operation = client.create_function(
    parent=parent,
    function_id=function_id,
    function=function,
)

created = operation.result(timeout=1800)
print(created.name)
print(created.service_config.uri)
```

### Generate an upload URL for local source packaging

If your source bundle is local instead of already in Cloud Storage, generate a signed upload URL first:

```python
upload = client.generate_upload_url(parent=parent)
print(upload.upload_url)
```

Important constraints from the official docs:

- upload a zip file to the signed URL
- send `content-type: application/zip`
- do not send an `Authorization` header with that upload request

After the upload completes, use the returned upload URL in the create or update flow.

### Delete a function

```python
function_name = client.function_path(project_id, region, "hello-function")
operation = client.delete_function(name=function_name)
operation.result(timeout=1800)
```

## Legacy V1 Surface

Use `functions_v1` only if you need the 1st gen API shape:

```python
from google.cloud import functions_v1

client = functions_v1.CloudFunctionsServiceClient()
```

Reasons to stay on v1:

- you are managing existing 1st gen functions
- you need `call_function`
- you are working with older deployment fields or migration flows

Do not mix v1 and v2 request objects in the same workflow. Pick the surface that matches the deployed function generation.

## Configuration Notes

- Keep `GOOGLE_CLOUD_PROJECT` and `GOOGLE_CLOUD_REGION` in config, not inline strings.
- If you accept customer-supplied credential JSON or files, review Google’s security guidance before loading them directly.
- `service_config` controls execution-side settings such as memory, timeout, instance counts, service account, ingress, and secret environment variables.
- `build_config` controls build-time concerns such as runtime, entry point, source location, worker pool, and build-time environment variables.
- 2nd gen functions are backed by Cloud Run service configuration. Expect important fields such as the deployed URI to live under `service_config`.
- If you enable library logging, Google’s client library logging can be activated with `GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google`.

## Common Pitfalls

- The package name is `google-cloud-functions`, but the main imports are `google.cloud.functions_v2` and `google.cloud.functions_v1`.
- This library manages Cloud Functions resources. It does not replace plain HTTP requests to a deployed function URL.
- For `get`, `update`, and `delete`, use full resource names like `projects/.../locations/.../functions/...`, not just `my-function`.
- `update_function` without a `FieldMask` updates everything present in the request object.
- `generate_upload_url()` is for a signed upload flow; the upload request must not include an `Authorization` header.
- `StorageSource.object_` expects a `.tar.gz` archive in Cloud Storage, while the signed upload URL flow expects a zip upload.
- Long-running operations are the normal path for create, update, and delete. Do not assume the resource is ready until `operation.result()` returns.
- If your code forks processes, create client instances after `os.fork()` or after `multiprocessing` starts the child process.
- 2nd gen and 1st gen fields are different enough that copying old blog examples blindly usually fails.

## Version-Sensitive Notes

- PyPI currently lists `1.22.0` as the latest release, published on January 15, 2026.
- As of March 12, 2026, some Google Cloud docs pages still show `1.21.0` as the latest on overview or changelog pages even though the class reference pages and PyPI already expose `1.22.0`. Trust PyPI and the class pages over the lagging overview when pinning the package version.
- The changelog page for `latest` currently tops out at `1.21.0`, where Google notes Python 3.14 support and deprecates the `credentials_file` argument.
- The `BuildConfig.docker_registry` field is effectively Artifact Registry only for 2nd gen. The docs mark `CONTAINER_REGISTRY` as deprecated as of March 2025.
- `ServiceConfig.security_level` only matters for 1st gen HTTP-triggered functions. 2nd gen functions are HTTPS-only.

## Official Sources

- Python reference root: https://cloud.google.com/python/docs/reference/cloudfunctions/latest
- v2 client reference: https://docs.cloud.google.com/python/docs/reference/cloudfunctions/latest/google.cloud.functions_v2.services.function_service.FunctionServiceClient
- v2 type reference: https://docs.cloud.google.com/python/docs/reference/cloudfunctions/latest/google.cloud.functions_v2.types.Function
- REST reference: https://cloud.google.com/functions/docs/reference/rest/v2/projects.locations.functions
- Authentication: https://cloud.google.com/docs/authentication/provide-credentials-adc
- ADC search order: https://cloud.google.com/docs/authentication/application-default-credentials
- PyPI package page: https://pypi.org/project/google-cloud-functions/

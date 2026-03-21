---
name: compute
description: "Python package guide for google-cloud-compute, the Google Cloud Compute Engine client library"
metadata:
  languages: "python"
  versions: "1.45.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud-compute,gcp,google-cloud,compute-engine,vm,python"
---

# google-cloud-compute Python Package Guide

`google-cloud-compute` is the generated Python client for Google Cloud Compute Engine.
Install the package from PyPI, import `compute_v1`, authenticate with Application Default Credentials (ADC), and use the resource-specific client for the thing you are managing.

```python
from google.cloud import compute_v1
```

## Install

```bash
pip install google-cloud-compute==1.45.0
```

Use the PyPI package name in dependency files and the `google.cloud.compute_v1` import path in code.

PyPI currently lists:

- Package version: `1.45.0`
- Python requirement: `>=3.7`

## Setup And Authentication

Typical prerequisites:

- a Google Cloud project ID
- the Compute Engine API enabled for that project
- credentials with the required IAM permissions
- the correct zone, region, or resource scope for the API you are calling

For local development, the standard path is ADC:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="my-project"
export GOOGLE_CLOUD_ZONE="us-central1-a"
```

For automation, use a service account key only when ADC from the runtime environment is not available:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

Explicit credentials also work:

```python
from google.cloud import compute_v1
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

instances = compute_v1.InstancesClient(credentials=credentials)
```

## Client Model

This library is organized as generated clients by resource family. Common ones include:

- `InstancesClient`
- `ImagesClient`
- `FirewallsClient`
- `NetworksClient`
- `SubnetworksClient`
- `DisksClient`
- `ZoneOperationsClient`
- `RegionOperationsClient`
- `GlobalOperationsClient`

Common rules that matter in practice:

- Use the client that matches the resource scope: zonal, regional, or global.
- Read methods usually return response objects or pagers.
- Mutating methods such as `insert()`, `delete()`, `start()`, and `stop()` usually return an extended operation that you must wait on.
- The generated methods accept either flattened keyword arguments or a `request=` object. Do not mix both in the same call.

## Basic Read Operations

List instances in a zone:

```python
import os
from google.cloud import compute_v1

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
zone = os.environ["GOOGLE_CLOUD_ZONE"]

instances = compute_v1.InstancesClient()

for vm in instances.list(project=project_id, zone=zone):
    print(vm.name, vm.status)
```

Fetch one instance:

```python
import os
from google.cloud import compute_v1

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
zone = os.environ["GOOGLE_CLOUD_ZONE"]

instances = compute_v1.InstancesClient()
vm = instances.get(project=project_id, zone=zone, instance="example-vm")

print(vm.name)
print(vm.machine_type)
print(vm.status)
```

Resolve an image family before instance creation:

```python
from google.cloud import compute_v1

images = compute_v1.ImagesClient()
image = images.get_from_family(project="debian-cloud", family="debian-12")

print(image.self_link)
```

## Waiting For Mutating Operations

Compute Engine mutations are long-running operations. The official samples use `google.api_core.extended_operation.ExtendedOperation` and wait on `result()`.

```python
from google.api_core.extended_operation import ExtendedOperation

def wait_for_extended_operation(
    operation: ExtendedOperation, *, verbose_name: str, timeout: int = 600
):
    result = operation.result(timeout=timeout)

    if operation.error_code:
        raise RuntimeError(
            f"{verbose_name} failed with code {operation.error_code}: "
            f"{operation.error_message}"
        )

    for warning in operation.warnings or []:
        print(f"{verbose_name} warning: {warning.code}: {warning.message}")

    return result
```

Start, stop, and delete all follow the same pattern:

```python
stop_op = instances.stop(project=project_id, zone=zone, instance="example-vm")
wait_for_extended_operation(stop_op, verbose_name="stop instance")

start_op = instances.start(project=project_id, zone=zone, instance="example-vm")
wait_for_extended_operation(start_op, verbose_name="start instance")

delete_op = instances.delete(project=project_id, zone=zone, instance="example-vm")
wait_for_extended_operation(delete_op, verbose_name="delete instance")
```

## Creating An Instance

The official Compute sample uses `ImagesClient.get_from_family()` to find a boot image, builds a `compute_v1.Instance`, then calls `InstancesClient.insert()`.

```python
import os
from google.cloud import compute_v1

project_id = os.environ["GOOGLE_CLOUD_PROJECT"]
zone = os.environ["GOOGLE_CLOUD_ZONE"]
instance_name = "example-vm"

images = compute_v1.ImagesClient()
source_image = images.get_from_family(
    project="debian-cloud",
    family="debian-12",
)

disk = compute_v1.AttachedDisk()
disk.boot = True
disk.auto_delete = True
disk.initialize_params = compute_v1.AttachedDiskInitializeParams(
    source_image=source_image.self_link,
)

network_interface = compute_v1.NetworkInterface()
network_interface.name = "global/networks/default"

instance = compute_v1.Instance()
instance.name = instance_name
instance.machine_type = f"zones/{zone}/machineTypes/e2-micro"
instance.disks = [disk]
instance.network_interfaces = [network_interface]

instances = compute_v1.InstancesClient()
operation = instances.insert(
    project=project_id,
    zone=zone,
    instance_resource=instance,
)

wait_for_extended_operation(operation, verbose_name="create instance")
```

## Configuration Patterns

- Prefer environment variables or configuration files for `project`, `zone`, and `region`; avoid hard-coding them in library helpers.
- Use the `request=` object style when the method has many optional fields or when you need request flags such as filters or partial-success behavior.
- If you need custom client options, pass them to the client constructor rather than patching transport internals.
- Create clients after `os.fork()`. The official multiprocessing guidance says to instantiate client objects in the child process, not before forking.

## Common Pitfalls

- `google-cloud-compute` is the package name, but the import is `from google.cloud import compute_v1`.
- Zonal, regional, and global resources are handled by different clients and different operation services. Use the wrong scope and the request fails.
- Many fields need Compute Engine resource paths, not shorthand names. Common examples are `zones/us-central1-a/machineTypes/e2-micro`, `global/networks/default`, and full image `self_link` values.
- Mutating calls are not complete when the initial method returns. Wait for the operation result and check `error_code`, `error_message`, and warnings.
- List methods return pagers. Iterate them directly instead of assuming a single page.
- The generated clients are large enough that JetBrains IDEs may need a larger maximum IntelliSense file size to fully index the stubs.

## Version-Sensitive Notes

As of `2026-03-12`, the official sources are not fully aligned:

- PyPI lists `google-cloud-compute 1.45.0`.
- The Google Cloud Python reference root and changelog pages still show `1.41.0`.

Treat `1.45.0` as the package version for dependency management, but verify any newly added methods or fields against the version of the Google-hosted reference you are reading. The docs URL in this entry is a moving `latest` page, not a version-pinned reference tree.

## Official Sources

- PyPI: https://pypi.org/project/google-cloud-compute/
- Google Cloud Python reference root: https://cloud.google.com/python/docs/reference/compute/latest
- `InstancesClient` reference: https://cloud.google.com/python/docs/reference/compute/latest/google.cloud.compute_v1.services.instances.InstancesClient
- Multiprocessing guidance: https://cloud.google.com/python/docs/reference/compute/latest/multiprocessing
- Compute instance creation sample: https://cloud.google.com/compute/docs/samples/compute-instances-create
- ADC setup: https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment
- Changelog: https://cloud.google.com/python/docs/reference/compute/latest/changelog

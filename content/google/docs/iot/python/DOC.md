---
name: iot
description: "Legacy Google Cloud IoT Core Python client library for maintaining existing device manager code and resource models"
metadata:
  languages: "python"
  versions: "2.9.2"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,iot,cloudiot,devices,pubsub,legacy"
---

# Google Cloud IoT Python Client Library

## Golden Rule

`google-cloud-iot` is a legacy client for Google Cloud IoT Core. The official repository is archived, and the upstream README says Cloud IoT Core was retired in August 2023. Use this package only to read, maintain, or migrate older codebases. Do not choose it for new systems. When you do touch existing code, import `from google.cloud import iot_v1`, authenticate with Application Default Credentials (ADC), and pass request objects or request dicts to client methods.

## Install

Pin the exact version your legacy project expects:

```bash
python -m pip install "google-cloud-iot==2.9.2"
```

Common alternatives:

```bash
uv add "google-cloud-iot==2.9.2"
poetry add "google-cloud-iot==2.9.2"
```

Practical note: there have been no new releases after `2.9.2`, so treat newer Python runtimes as unvalidated unless your own test matrix proves otherwise.

## Authentication And Setup

The client uses Google authentication plumbing, so use ADC first:

1. Local development: `gcloud auth application-default login`
2. Google Cloud runtime with an attached service account
3. `GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json` only when the first two are not possible

Basic setup:

```python
from google.cloud import iot_v1

client = iot_v1.DeviceManagerClient()

parent = "projects/my-project/locations/us-central1"
registry_name = f"{parent}/registries/my-registry"
device_name = f"{registry_name}/devices/device-1"
```

If legacy code needs a non-default endpoint or transport settings, use the generated GAPIC knobs such as `client_options` and `transport`. Keep that configuration explicit in tests because this package is old and no longer evolving.

## Core Usage

These examples are for maintaining old code and understanding the current 2.x request shapes. They are not a recommendation to build new Cloud IoT Core workflows.

### List registries in a location

```python
from google.cloud import iot_v1

client = iot_v1.DeviceManagerClient()
parent = "projects/my-project/locations/us-central1"

for registry in client.list_device_registries(request={"parent": parent}):
    print(registry.id)
```

### List devices in a registry

```python
from google.cloud import iot_v1

client = iot_v1.DeviceManagerClient()
registry_name = "projects/my-project/locations/us-central1/registries/my-registry"

for device in client.list_devices(request={"parent": registry_name}):
    print(device.id, device.num_id)
```

### Read one device and its config history

```python
from google.cloud import iot_v1

client = iot_v1.DeviceManagerClient()
device_name = "projects/my-project/locations/us-central1/registries/my-registry/devices/device-1"

device = client.get_device(request={"name": device_name})
print(device.id)

for version in client.list_device_config_versions(request={"name": device_name}):
    print(version.version, version.cloud_update_time)
```

### Update persistent cloud-to-device config

Use `modify_cloud_to_device_config` when the device should fetch a versioned config payload later.

```python
import json
from google.cloud import iot_v1

client = iot_v1.DeviceManagerClient()
device_name = "projects/my-project/locations/us-central1/registries/my-registry/devices/device-1"

payload = json.dumps({"sampling_interval_s": 60}).encode("utf-8")

config = client.modify_cloud_to_device_config(
    request={
        "name": device_name,
        "binary_data": payload,
    }
)

print(config.version)
```

### Send an immediate command

Use `send_command_to_device` for transient commands rather than durable config.

```python
from google.cloud import iot_v1

client = iot_v1.DeviceManagerClient()
device_name = "projects/my-project/locations/us-central1/registries/my-registry/devices/device-1"

client.send_command_to_device(
    request={
        "name": device_name,
        "binary_data": b"reboot",
        "subfolder": "ops",
    }
)
```

### Create or inspect registry definitions in legacy code

Registry objects usually wire Pub/Sub topics for telemetry and state:

```python
from google.cloud import iot_v1

client = iot_v1.DeviceManagerClient()
parent = "projects/my-project/locations/us-central1"

registry = {
    "id": "my-registry",
    "event_notification_configs": [
        {
            "pubsub_topic_name": "projects/my-project/topics/device-events",
        }
    ],
    "state_notification_config": {
        "pubsub_topic_name": "projects/my-project/topics/device-state",
    },
}

created = client.create_device_registry(
    request={
        "parent": parent,
        "device_registry": registry,
    }
)

print(created.name)
```

## Configuration And Data Model Notes

- Resource names are full paths, not short IDs. Use strings like `projects/.../locations/.../registries/.../devices/...`.
- `DeviceManagerClient` is the main service client. Most legacy code only touches this one client.
- Device credentials use `DeviceCredential` and `PublicKeyCredential`; there is no API-key-style auth for devices.
- Registry routing is Pub/Sub-based. Event and state topics must already exist, and the relevant Google-managed service identities need permission to publish.
- `binary_data` fields are bytes. Encode JSON manually with `json.dumps(...).encode("utf-8")`.
- The generated 2.x client accepts either strongly typed request objects or plain request dicts. Request dicts are usually simpler for maintenance work.

## Common Pitfalls

- The service is retired. Official reference pages still exist, but live Cloud IoT Core workflows are no longer a valid choice for new Google Cloud systems.
- The repo is archived, so do not expect bugfixes, new features, or updated Python-version classifiers.
- Older blog posts often show 1.x positional-call patterns. The 2.x client follows GAPIC request-object style instead.
- Device, registry, and location strings are easy to mix up. Most method failures in old code come from malformed resource names.
- `modify_cloud_to_device_config` stores durable config versions; `send_command_to_device` is for immediate commands. They are not interchangeable.
- Pub/Sub topic wiring is registry-level configuration. If legacy code creates registries, missing topics or IAM permissions will break event/state delivery.

## Version-Sensitive Notes

- Official PyPI metadata and the Google Cloud Python reference both show `2.9.2` as the current published package version as of `2026-03-12`.
- The important upgrade boundary is `2.0.0`. If you are maintaining pre-2.x code, check the upstream migration guide before translating older samples.
- Upstream metadata still advertises `>=3.7`, but the archived package has no active release train for modern Python versions. Verify `3.12+` behavior in your own environment before depending on it.

## Official Sources

- Reference docs: `https://cloud.google.com/python/docs/reference/cloudiot/latest`
- Main client reference: `https://cloud.google.com/python/docs/reference/cloudiot/latest/google.cloud.iot_v1.services.device_manager.DeviceManagerClient`
- Migration guide: `https://cloud.google.com/python/docs/reference/cloudiot/latest/migration`
- ADC guide: `https://cloud.google.com/docs/authentication/application-default-credentials`
- GitHub repo and archived README: `https://github.com/googleapis/python-iot`
- PyPI package page: `https://pypi.org/project/google-cloud-iot/`

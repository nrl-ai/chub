---
name: eventarc
description: "Google Cloud Eventarc Python client for managing triggers, providers, channels, Google API sources, message buses, and pipelines"
metadata:
  languages: "python"
  versions: "1.19.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,eventarc,google-cloud,events,cloudevents,cloud-run,pubsub"
---

# Google Cloud Eventarc Python Client Library

## Golden Rule

Use `google-cloud-eventarc` to manage Eventarc resources from Python. This package is the control-plane client for Eventarc resources such as triggers, providers, channels, message buses, pipelines, and Google API sources. It is not the library that receives HTTP events in your Cloud Run service, and it is not the publishing client for custom events; Google publishes `google-cloud-eventarc-publishing` separately for publishing flows.

## Install

Pin the version your project expects:

```bash
python -m pip install "google-cloud-eventarc==1.19.0"
```

Common alternatives:

```bash
uv add "google-cloud-eventarc==1.19.0"
poetry add "google-cloud-eventarc==1.19.0"
```

## Authentication And Setup

Eventarc clients use Application Default Credentials (ADC). Prefer these credential sources in this order:

1. Local development: `gcloud auth application-default login`
2. Google Cloud runtimes with an attached service account or workload identity
3. `GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json` only when you cannot use the first two

Project prerequisites agents commonly miss:

- Enable the Eventarc API before creating or listing resources.
- Use the same `location` as the resources you are routing events from or to; providers and triggers are location-scoped.
- Make sure the caller can manage Eventarc resources and impersonate the service account used by the trigger when needed.
- For Cloud Run destinations, the trigger's service account typically needs `roles/eventarc.eventReceiver`, and the destination service may need `roles/run.invoker` when it requires authenticated invocation.
- Avoid default service accounts for production setups unless you have reviewed the permissions explicitly.

If you must use a key file locally:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

## Initialize A Client

Generated client snippets warn that some environments may require specifying a regional API endpoint. Start with the default client, and set `client_options` if your environment or upstream guidance requires a location-specific endpoint.

```python
from google.cloud import eventarc_v1

project_id = "my-project"
location = "us-central1"
parent = f"projects/{project_id}/locations/{location}"

client = eventarc_v1.EventarcClient()
```

If you need custom endpoint selection:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import eventarc_v1

client = eventarc_v1.EventarcClient(
    client_options=ClientOptions(
        api_endpoint="YOUR_EVENTARC_ENDPOINT"
    )
)
```

## Core Usage

### List providers for a location

Provider discovery is the safest way to confirm which event sources and filters are valid in a given region.

```python
from google.cloud import eventarc_v1

project_id = "my-project"
location = "us-central1"
parent = f"projects/{project_id}/locations/{location}"

client = eventarc_v1.EventarcClient()

for provider in client.list_providers(parent=parent):
    print(provider.name)
```

Use the provider metadata in the official docs or `gcloud eventarc providers describe ...` when you need the exact filtering attributes for a specific event type.

### Create a trigger that sends Cloud Storage events to Cloud Run

Trigger creation is a long-running operation, so wait for `.result()` instead of assuming the resource is ready immediately.

```python
from google.cloud import eventarc_v1

project_id = "my-project"
location = "us-central1"
parent = f"projects/{project_id}/locations/{location}"

client = eventarc_v1.EventarcClient()

trigger = eventarc_v1.Trigger(
    service_account="eventarc-trigger@my-project.iam.gserviceaccount.com",
    destination=eventarc_v1.Destination(
        cloud_run=eventarc_v1.CloudRun(
            service="image-handler",
            region=location,
            path="/events",
        )
    ),
    event_filters=[
        eventarc_v1.EventFilter(
            attribute="type",
            value="google.cloud.storage.object.v1.finalized",
        ),
        eventarc_v1.EventFilter(
            attribute="bucket",
            value="my-upload-bucket",
        ),
    ],
)

operation = client.create_trigger(
    parent=parent,
    trigger=trigger,
    trigger_id="storage-object-finalized",
)

created = operation.result(timeout=300)
print(created.name)
```

Notes:

- Every trigger must include an event filter for the `type` attribute.
- The valid additional filters depend on the provider and event type.
- Eventarc propagation is not instant; the product docs note that new triggers can take up to about two minutes to become active.

### List existing triggers

`list_triggers()` returns a pager, so iterate directly over it.

```python
from google.cloud import eventarc_v1

project_id = "my-project"
location = "us-central1"
parent = f"projects/{project_id}/locations/{location}"

client = eventarc_v1.EventarcClient()

for trigger in client.list_triggers(parent=parent):
    print(trigger.name)
    print(trigger.service_account)
```

### Delete a trigger

Delete operations are also long-running:

```python
from google.cloud import eventarc_v1

client = eventarc_v1.EventarcClient()

name = "projects/my-project/locations/us-central1/triggers/storage-object-finalized"
operation = client.delete_trigger(name=name)
operation.result(timeout=300)
```

## Working With Advanced Resources

The same client also exposes methods for Eventarc Advanced resources such as channels, channel connections, message buses, pipelines, enrollments, and Google API sources. The request pattern is the same:

- build the resource name under `projects/{project}/locations/{location}/...`
- create the resource object from `eventarc_v1`
- call the matching `create_*`, `update_*`, `get_*`, `list_*`, or `delete_*` method
- wait on long-running operations with `.result()`

If you are writing code against these newer surfaces, check the `1.18.x` and `1.19.x` changelog entries first because several fields for Google API sources, Eventarc triggers, and Python version support were updated recently.

## Configuration Notes

- Resource names are location-scoped. Keep a single `location` variable and build all `parent` and `name` strings from it.
- Use explicit project IDs in resource names rather than relying on ambient defaults when automation spans multiple projects.
- Logging can be enabled with the standard Google client logging controls, including `GOOGLE_SDK_PYTHON_LOGGING_SCOPE`.
- Prefer passing an explicit `credentials=` object only when ADC is not viable. The changelog notes that the older `credentials_file` argument is deprecated.

## Common Pitfalls

- This package manages Eventarc resources. It does not replace your Cloud Run, GKE, or HTTP application code that actually receives delivered CloudEvents.
- Do not guess event filters. Event types and supported filtering attributes vary by provider and location.
- `type` is mandatory on every trigger, and Eventarc does not let you change the event filter type after the trigger is created.
- Long-running operations still need propagation time after `.result()` returns; allow for eventual consistency in tests and automation.
- Path-pattern matching is not generally available for all trigger types; the type docs mark it as only supported for GCFv1 triggers.
- `Trigger.retry_policy` is version-sensitive and only applies to Cloud Run destinations.
- If you provide your own Pub/Sub topic for transport, Eventarc will not delete that topic when the trigger is deleted.
- IAM failures are often split across two identities: the caller creating the trigger and the service account attached to the trigger itself.

## Version-Sensitive Notes For 1.19.0

- PyPI and the official Google Cloud reference both show `1.19.0` as the current package version on March 12, 2026.
- The `1.19.0` changelog adds Python 3.14 support.
- The `1.18.0` changelog added wider `GoogleApiSource` flags and trigger retry policy support, so examples written against older `1.17.x` docs may miss newer fields.
- The `1.17.0` changelog deprecates `credentials_file`; prefer ADC or an explicit `credentials=` object.

## Official Sources

- Python client reference: `https://cloud.google.com/python/docs/reference/eventarc/latest`
- `EventarcClient` reference: `https://cloud.google.com/python/docs/reference/eventarc/latest/google.cloud.eventarc_v1.services.eventarc.EventarcClient`
- Package changelog: `https://cloud.google.com/python/docs/reference/eventarc/latest/changelog`
- Eventarc create-trigger guidance: `https://cloud.google.com/eventarc/docs/creating-triggers`
- Eventarc IAM guidance: `https://cloud.google.com/eventarc/docs/roles-permissions`
- ADC guidance: `https://cloud.google.com/docs/authentication/application-default-credentials`
- PyPI package page: `https://pypi.org/project/google-cloud-eventarc/`

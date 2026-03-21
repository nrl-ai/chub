---
name: deploy
description: "Google Cloud Deploy Python client for listing delivery pipelines and targets, inspecting releases and rollouts, and driving approval or promotion flows"
metadata:
  languages: "python"
  versions: "2.9.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,cloud-deploy,gcp,cicd,deployments,rollouts,adc,python"
---

# google-cloud-deploy Python Package Guide

Use `google-cloud-deploy` for Python automation around Google Cloud Deploy control-plane resources: delivery pipelines, targets, releases, rollouts, rollout approvals, promotions, and canary advancement.

Import surface:

```python
from google.cloud import deploy_v1
```

## Golden Rule

- Install and import the official client: `from google.cloud import deploy_v1`.
- Authenticate with Application Default Credentials (ADC), not ad hoc API-key patterns.
- Treat Cloud Deploy resource names as regional: `projects/{project}/locations/{location}/...`.
- Use generated path helpers such as `release_path(...)` and `rollout_path(...)` instead of hand-building resource strings.
- Read current release or rollout state before approving, promoting, or advancing it.
- Wait for long-running operations with `operation.result(...)` when later steps depend on completion.

## Version Covered

- Package: `google-cloud-deploy`
- Ecosystem: `pypi`
- Version: `2.9.0`
- Python requirement from PyPI: `>=3.7`
- Registry: https://pypi.org/project/google-cloud-deploy/
- Canonical package docs URL from the task input: https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-deploy
- Python reference root used for method shapes: https://docs.cloud.google.com/python/docs/reference/clouddeploy/latest

Version drift note:

- PyPI lists `2.9.0` as the current package version.
- The generated Python reference root also renders `google-cloud-deploy 2.9.0`.
- Some generated subpages, including the changelog and some class pages, still display `2.7.1`.
- Practical rule: use PyPI for package-version truth, and use the generated reference pages for client classes, request types, and helper methods.

## Install

Pin the package version you want the agent to target:

```bash
python -m pip install google-cloud-deploy==2.9.0
```

If you only need the latest published build:

```bash
python -m pip install google-cloud-deploy
```

## Authentication And Setup

This client uses Google Cloud credentials through ADC.

Local development:

```bash
gcloud auth application-default login
gcloud services enable clouddeploy.googleapis.com
```

Service account credentials:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
export GOOGLE_CLOUD_PROJECT="my-project-id"
```

Minimum setup:

1. Enable the Cloud Deploy API in the target project.
2. Make sure ADC resolves to the principal you expect.
3. Grant the IAM role that matches the action being automated.
4. Keep the project, location, delivery pipeline, release, and rollout names in the same region.

Common documented Cloud Deploy roles:

- `roles/clouddeploy.admin`
- `roles/clouddeploy.operator`
- `roles/clouddeploy.releaser`
- `roles/clouddeploy.approver`

If your automation promotes or approves rollouts, validate both Cloud Deploy IAM and the downstream runtime permissions needed by the actual target environment.

## Initialize The Client

Standard sync client:

```python
from google.cloud import deploy_v1

client = deploy_v1.CloudDeployClient()
```

Async client:

```python
from google.cloud import deploy_v1

client = deploy_v1.CloudDeployAsyncClient()
```

The generated client also supports `transport` and `client_options` when you need REST transport or custom endpoint behavior:

```python
from google.cloud import deploy_v1

client = deploy_v1.CloudDeployClient(
    transport="rest",
)
```

## Resource Names And Regional Scope

Cloud Deploy is regional. Delivery pipelines, targets, releases, and rollouts all live under a location-qualified resource name.

Prefer generated helpers:

```python
from google.cloud import deploy_v1

client = deploy_v1.CloudDeployClient()

pipeline_name = client.delivery_pipeline_path(
    "my-project",
    "us-central1",
    "web-app",
)
target_name = client.target_path(
    "my-project",
    "us-central1",
    "staging",
)
release_name = client.release_path(
    "my-project",
    "us-central1",
    "web-app",
    "release-20260312",
)
rollout_name = client.rollout_path(
    "my-project",
    "us-central1",
    "web-app",
    "release-20260312",
    "release-20260312-to-staging",
)
```

This is safer than concatenating strings and helps avoid cross-region `NotFound` and permission errors.

## Core Usage

### List Delivery Pipelines

```python
from google.cloud import deploy_v1

def list_delivery_pipelines(project_id: str, location: str) -> list[str]:
    client = deploy_v1.CloudDeployClient()
    parent = f"projects/{project_id}/locations/{location}"

    return [
        pipeline.name
        for pipeline in client.list_delivery_pipelines(
            request={"parent": parent}
        )
    ]
```

### List Targets

```python
from google.cloud import deploy_v1

def list_targets(project_id: str, location: str) -> list[str]:
    client = deploy_v1.CloudDeployClient()
    parent = f"projects/{project_id}/locations/{location}"

    return [
        target.name
        for target in client.list_targets(
            request={"parent": parent}
        )
    ]
```

### List Releases For A Pipeline

```python
from google.cloud import deploy_v1

def list_releases(project_id: str, location: str, pipeline_id: str) -> list[str]:
    client = deploy_v1.CloudDeployClient()
    parent = client.delivery_pipeline_path(project_id, location, pipeline_id)

    return [
        release.name
        for release in client.list_releases(
            request={"parent": parent}
        )
    ]
```

### Read Release And Rollout State Before Acting

Use this before promotion, approval, or canary advancement so automation branches on actual state instead of assuming the rollout is ready.

```python
from google.cloud import deploy_v1

def get_release_and_rollout(
    project_id: str,
    location: str,
    pipeline_id: str,
    release_id: str,
    rollout_id: str,
):
    client = deploy_v1.CloudDeployClient()

    release_name = client.release_path(
        project_id,
        location,
        pipeline_id,
        release_id,
    )
    rollout_name = client.rollout_path(
        project_id,
        location,
        pipeline_id,
        release_id,
        rollout_id,
    )

    release = client.get_release(request={"name": release_name})
    rollout = client.get_rollout(request={"name": rollout_name})
    return release, rollout
```

### Create A Release

`create_release` is usually the most version-sensitive write path. The official request type still centers on three fields:

- `parent`: delivery pipeline resource name
- `release_id`: release identifier
- `release`: `deploy_v1.Release` payload

```python
from google.cloud import deploy_v1

def create_release(project_id: str, location: str, pipeline_id: str, release_id: str):
    client = deploy_v1.CloudDeployClient()
    parent = client.delivery_pipeline_path(project_id, location, pipeline_id)

    release = deploy_v1.Release(
        description="Created from Python automation",
    )

    operation = client.create_release(
        request={
            "parent": parent,
            "release_id": release_id,
            "release": release,
        }
    )
    return operation.result(timeout=900)
```

The current `Release` reference includes fields such as `skaffold_config_path`, `skaffold_version`, `build_artifacts`, `target_artifacts`, and `deploy_parameters`. Which fields you actually need depends on how the delivery pipeline and render source are configured, so verify the installed client version's `Release` schema before generating release bodies.

### Approve A Rollout

Use `approve_rollout` only when the rollout is paused for approval.

```python
from google.cloud import deploy_v1

def approve_rollout(
    project_id: str,
    location: str,
    pipeline_id: str,
    release_id: str,
    rollout_id: str,
) -> None:
    client = deploy_v1.CloudDeployClient()
    rollout_name = client.rollout_path(
        project_id,
        location,
        pipeline_id,
        release_id,
        rollout_id,
    )

    operation = client.approve_rollout(
        request={
            "name": rollout_name,
            "approved": True,
        }
    )
    operation.result(timeout=600)
```

### Promote A Release

Use `promote_release` to move an existing release forward in the delivery pipeline.

```python
from google.cloud import deploy_v1

def promote_release(
    project_id: str,
    location: str,
    pipeline_id: str,
    release_id: str,
    target_id: str,
    rollout_id: str,
) -> None:
    client = deploy_v1.CloudDeployClient()
    release_name = client.release_path(
        project_id,
        location,
        pipeline_id,
        release_id,
    )

    operation = client.promote_release(
        request={
            "name": release_name,
            "target_id": target_id,
            "rollout_id": rollout_id,
        }
    )
    operation.result(timeout=900)
```

### Advance A Canary Rollout Phase

For multi-phase rollouts, use `advance_rollout` with the phase identifier.

```python
from google.cloud import deploy_v1

def advance_rollout(
    project_id: str,
    location: str,
    pipeline_id: str,
    release_id: str,
    rollout_id: str,
    phase_id: str,
) -> None:
    client = deploy_v1.CloudDeployClient()
    rollout_name = client.rollout_path(
        project_id,
        location,
        pipeline_id,
        release_id,
        rollout_id,
    )

    operation = client.advance_rollout(
        request={
            "name": rollout_name,
            "phase_id": phase_id,
        }
    )
    operation.result(timeout=900)
```

## Configuration Notes

- The generated client uses gRPC by default; use `transport="rest"` only if your environment needs REST.
- The list methods return pagers, so ordinary iteration handles pagination.
- For library logging, the PyPI package page documents the standard Google client-library logging controls, including the `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` environment variable.

## Common Pitfalls

- The repo URL from the task is a package source location, not the best day-to-day API reference. Use the generated Python reference for class names and request types.
- The old docs path pattern `.../python/docs/reference/deploy/latest` is stale for this package. The canonical path is `.../python/docs/reference/clouddeploy/latest`.
- Cloud Deploy is regional. A correct project with the wrong `location` still fails.
- `approve_rollout`, `promote_release`, and `advance_rollout` are long-running operations; do not assume success until `operation.result(...)` returns.
- Approval only works when the rollout is actually waiting for approval.
- Promotion can fail if the release, pipeline, or target state no longer matches what your automation assumed.
- Release creation depends on your pipeline configuration, Skaffold setup, render source, and artifacts. Do not generate a large `Release` body from memory alone.

## Version-Sensitive Notes

- This doc targets `2.9.0`, which is the package version currently published on PyPI.
- The generated reference site is partially version-lagged: some subpages still render `2.7.1` even though the reference root and PyPI show `2.9.0`.
- `CloudDeployClient` and `CloudDeployAsyncClient` are both present in the current reference.
- Before using lesser-known request fields, confirm them against the installed client's generated types for `2.9.0` rather than relying on older blog posts or stale snippets.

## Official Links

- Package repo path: `https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-deploy`
- PyPI: `https://pypi.org/project/google-cloud-deploy/`
- Python reference root: `https://docs.cloud.google.com/python/docs/reference/clouddeploy/latest`
- `CloudDeployClient` reference: `https://docs.cloud.google.com/python/docs/reference/clouddeploy/latest/google.cloud.deploy_v1.services.cloud_deploy.CloudDeployClient`
- `Release` reference: `https://docs.cloud.google.com/python/docs/reference/clouddeploy/latest/google.cloud.deploy_v1.types.Release`
- Changelog: `https://docs.cloud.google.com/python/docs/reference/clouddeploy/latest/changelog`
- Cloud Deploy product docs: `https://cloud.google.com/deploy/docs`
- Promote release guide: `https://cloud.google.com/deploy/docs/promote-release`
- Approvals and promotions guide: `https://cloud.google.com/deploy/docs/approve-promote`

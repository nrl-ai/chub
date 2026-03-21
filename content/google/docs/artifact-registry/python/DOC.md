---
name: artifact-registry
description: "Google Cloud Artifact Registry Python client for repository management, package metadata, tags, and version operations"
metadata:
  languages: "python"
  versions: "1.20.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,gcp,artifact-registry,packages,python,registry,google-cloud"
---

# Google Cloud Artifact Registry Python Client

## Golden Rule

Use `google-cloud-artifact-registry` when you need to manage Artifact Registry resources from Python code. Import it as `from google.cloud import artifactregistry_v1`, authenticate with Application Default Credentials (ADC), and build resource names with the client helper methods instead of string-concatenating them by hand.

This package is the Artifact Registry control-plane client. If your job is configuring `pip`, `twine`, or keyring-based auth for private Python package publishing and installs, use the Artifact Registry Python package product docs for that workflow; this SDK is for repository, package, version, tag, rule, file, and attachment APIs.

## Install

Pin the version your project expects:

```bash
python -m pip install "google-cloud-artifact-registry==1.20.0"
```

Common alternatives:

```bash
uv add "google-cloud-artifact-registry==1.20.0"
poetry add "google-cloud-artifact-registry==1.20.0"
```

## Authentication And Setup

Before you create a client:

1. Enable the Artifact Registry API in the target Google Cloud project.
2. Use an identity with the Artifact Registry IAM permissions your operation needs.
3. Prefer ADC over embedding service-account keys in code.

Local development with ADC:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

Basic client setup:

```python
from google.cloud import artifactregistry_v1

client = artifactregistry_v1.ArtifactRegistryClient()
```

If your environment does not make the project obvious, keep the project ID explicit in resource names:

```python
project_id = "my-project"
location = "us-central1"
parent = f"projects/{project_id}/locations/{location}"
```

Service-account key fallback:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/absolute/path/service-account.json"
```

### Endpoint And Client Options

The client constructor accepts `credentials`, `transport`, and `client_options`. Use `client_options` when you need a specific API endpoint:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import artifactregistry_v1

client = artifactregistry_v1.ArtifactRegistryClient(
    client_options=ClientOptions(api_endpoint="artifactregistry.googleapis.com")
)
```

The package also supports structured logging configuration through the `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` environment variable, for example:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google.cloud.artifactregistry_v1
```

## Core Usage

### Build Resource Names With Helpers

Artifact Registry methods expect fully qualified resource names. Use the helper methods exposed on the client:

```python
from google.cloud import artifactregistry_v1

client = artifactregistry_v1.ArtifactRegistryClient()

repository_name = client.repository_path(
    project="my-project",
    location="us-central1",
    repository="python-internal",
)

package_name = client.package_path(
    project="my-project",
    location="us-central1",
    repository="python-internal",
    package="my-package",
)

python_package_name = client.python_package_path(
    project="my-project",
    location="us-central1",
    repository="python-internal",
    python_package="my-package",
)
```

These helpers prevent common mistakes around location-scoped names such as `projects/{project}/locations/{location}/repositories/{repository}`.

### List Repositories In A Location

```python
from google.cloud import artifactregistry_v1

client = artifactregistry_v1.ArtifactRegistryClient()
parent = "projects/my-project/locations/us-central1"

for repo in client.list_repositories(parent=parent):
    print(repo.name, repo.format_, repo.mode, repo.registry_uri)
```

The `Repository` type exposes repository `format_` and `mode`, including standard, remote, and virtual repository modes.

### Create A Standard Python Repository

`create_repository` is a long-running operation, so wait for the result:

```python
from google.cloud import artifactregistry_v1

client = artifactregistry_v1.ArtifactRegistryClient()
parent = "projects/my-project/locations/us-central1"

repository = artifactregistry_v1.Repository(
    format_=artifactregistry_v1.Repository.Format.PYTHON,
    mode=artifactregistry_v1.Repository.Mode.STANDARD_REPOSITORY,
    description="Internal Python packages",
)

operation = client.create_repository(
    parent=parent,
    repository_id="python-internal",
    repository=repository,
)

created = operation.result()
print(created.name)
print(created.registry_uri)
```

Use the returned `registry_uri` when wiring clients or publishing tools to the repository endpoint.

### List Packages And Python Package Metadata

Use `list_packages()` for general package inventory and `list_python_packages()` when you want Python-specific package resources:

```python
from google.cloud import artifactregistry_v1

client = artifactregistry_v1.ArtifactRegistryClient()
repository = client.repository_path("my-project", "us-central1", "python-internal")

for pkg in client.list_packages(parent=repository):
    print(pkg.name)

for py_pkg in client.list_python_packages(parent=repository):
    print(py_pkg.name)
```

### Inspect Versions And Tags

Versions and tags use package-scoped parent resources:

```python
from google.cloud import artifactregistry_v1

client = artifactregistry_v1.ArtifactRegistryClient()
package_name = client.package_path(
    project="my-project",
    location="us-central1",
    repository="python-internal",
    package="my-package",
)

for version in client.list_versions(parent=package_name):
    print(version.name)

for tag in client.list_tags(parent=package_name):
    print(tag.name, tag.version)
```

This is the right surface when you need metadata, retention tooling, or promotion logic around stored artifacts.

## Common Pitfalls

- Do not confuse this SDK with the Artifact Registry `pip` and `twine` setup guides. Publishing or installing Python packages from private repositories usually needs product-doc auth steps in addition to, or instead of, this client library.
- Resource names are always fully qualified and location-scoped. Prefer `repository_path()`, `package_path()`, `python_package_path()`, `tag_path()`, and `version_path()` helpers.
- `create_repository`, `update_repository`, and similar mutating methods can return long-running operations. Call `.result()` before assuming the change is usable.
- Artifact Registry is regional or multi-regional. A wrong `location` in the resource name is a common reason for confusing `NotFound` and permission errors.
- IAM matters at both the project and repository levels. Failing auth often means the identity lacks Artifact Registry permissions, not that the SDK call shape is wrong.
- If you override endpoints or transports, keep them aligned with the resource location and the rest of your Google Cloud environment.

## Version-Sensitive Notes

- PyPI lists `1.20.0` as the current package version covered here.
- The main client reference resolves on the `latest` docs track and showed `ArtifactRegistryClient (1.20.0)` when validated on March 12, 2026.
- Some adjacent generated reference pages under the same docs root were still indexed at `1.19.0` on March 12, 2026. If a type page looks behind the client page or PyPI, trust the package version on PyPI and re-check the docs version selector before copying details.
- PyPI lists support for Python `>=3.7`; if your project is on an older interpreter, upgrade Python before trying to pin this release line.

## Official Sources

- Python client reference root: `https://docs.cloud.google.com/python/docs/reference/artifactregistry/latest`
- `ArtifactRegistryClient` reference: `https://docs.cloud.google.com/python/docs/reference/artifactregistry/latest/google.cloud.artifactregistry_v1.services.artifact_registry.ArtifactRegistryClient`
- Client changelog: `https://docs.cloud.google.com/python/docs/reference/artifactregistry/latest/changelog`
- Artifact Registry Python package product docs: `https://docs.cloud.google.com/artifact-registry/docs/python`
- PyPI package page: `https://pypi.org/project/google-cloud-artifact-registry/`

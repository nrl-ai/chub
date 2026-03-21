---
name: resource-manager
description: "Google Cloud Resource Manager Python client library for projects, folders, organizations, and tag bindings with ADC-based auth"
metadata:
  languages: "python"
  versions: "1.16.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,gcp,google-cloud,resource-manager,projects,folders,organizations,tags,python"
---

# Google Cloud Resource Manager Python Client Library

## Golden Rule

Use `google-cloud-resource-manager` with `from google.cloud import resourcemanager_v3`, authenticate with Application Default Credentials (ADC), and pass full Google Cloud resource names instead of bare IDs. The package version is `1.16.0`, but the main client surface is still the `resourcemanager_v3` namespace.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-resource-manager==1.16.0"
```

Common alternatives:

```bash
uv add "google-cloud-resource-manager==1.16.0"
poetry add "google-cloud-resource-manager==1.16.0"
```

For local examples and most production workloads, you will also want the Google Cloud CLI available for ADC setup:

```bash
gcloud --version
```

## Authentication And Setup

Use ADC unless you have a hard requirement to provide credentials explicitly.

Recommended credential flow:

1. Local development: `gcloud auth application-default login`
2. Production on Google Cloud: attached service account or workload identity
3. Only if you cannot use the first two: `GOOGLE_APPLICATION_CREDENTIALS=/path/to/key.json`

Local development setup:

```bash
gcloud auth application-default login
gcloud services enable cloudresourcemanager.googleapis.com
```

Service account key fallback:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

Explicit credentials example:

```python
from google.cloud import resourcemanager_v3
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/path/to/service-account.json"
)

client = resourcemanager_v3.ProjectsClient(credentials=credentials)
```

Notes:

- Successful authentication is not enough by itself; the caller also needs the correct IAM permissions on the organization, folder, project, or tag resources it touches.
- For local development, Google recommends user ADC or service account impersonation instead of downloading long-lived key files.
- The Cloud Resource Manager API must be enabled on the project used for these calls.

## Initialize Clients

Projects are usually the starting point:

```python
from google.cloud import resourcemanager_v3

projects = resourcemanager_v3.ProjectsClient()
```

Other surfaces in the same package:

```python
from google.cloud import resourcemanager_v3

folders = resourcemanager_v3.FoldersClient()
organizations = resourcemanager_v3.OrganizationsClient()
tag_bindings = resourcemanager_v3.TagBindingsClient()
tag_keys = resourcemanager_v3.TagKeysClient()
tag_values = resourcemanager_v3.TagValuesClient()
```

Async variants also exist, for example `ProjectsAsyncClient` and `TagBindingsAsyncClient`, if your codebase is already async.

## Core Usage

### Read a project

Use a full resource name like `projects/my-project-id`:

```python
from google.cloud import resourcemanager_v3

client = resourcemanager_v3.ProjectsClient()
project_name = resourcemanager_v3.ProjectsClient.common_project_path("my-project-id")

project = client.get_project(name=project_name)

print(project.name)
print(project.project_id)
print(project.display_name)
print(project.state)
```

### Search projects when you do not know the exact parent

`search_projects()` accepts a query language and is the practical starting point for discovery:

```python
from google.cloud import resourcemanager_v3

client = resourcemanager_v3.ProjectsClient()

pager = client.search_projects(
    request={
        "query": "parent.type:folder parent.id:123456789012 state:ACTIVE labels.env:prod"
    }
)

for project in pager:
    print(project.project_id, project.display_name)
```

Use `search_projects()` for ad hoc discovery. Use `list_projects()` only when you already know the parent container.

### List projects under a folder or organization

```python
from google.cloud import resourcemanager_v3

client = resourcemanager_v3.ProjectsClient()

for project in client.list_projects(request={"parent": "folders/123456789012"}):
    print(project.project_id, project.display_name)
```

### Create a project

Project creation is a long-running operation. Wait for `.result()` instead of assuming the project exists immediately.

```python
from google.cloud import resourcemanager_v3

client = resourcemanager_v3.ProjectsClient()

operation = client.create_project(
    request={
        "project": {
            "project_id": "example-project-123",
            "display_name": "Example Project",
            "parent": "folders/123456789012",
        }
    }
)

project = operation.result(timeout=300)
print(project.name)
```

### Move a project

```python
from google.cloud import resourcemanager_v3

client = resourcemanager_v3.ProjectsClient()

operation = client.move_project(
    request={
        "name": "projects/my-project-id",
        "destination_parent": "folders/987654321098",
    }
)

project = operation.result(timeout=300)
print(project.parent)
```

### Delete and undelete a project

```python
from google.cloud import resourcemanager_v3

client = resourcemanager_v3.ProjectsClient()

delete_op = client.delete_project(name="projects/my-project-id")
delete_op.result(timeout=300)

undelete_op = client.undelete_project(name="projects/my-project-id")
undelete_op.result(timeout=300)
```

### Inspect effective tags on a project

Tag APIs use a different parent resource format than `ProjectsClient`. Use the double-slash URI-style resource name:

```python
from google.cloud import resourcemanager_v3

client = resourcemanager_v3.TagBindingsClient()

for tag in client.list_effective_tags(
    parent="//cloudresourcemanager.googleapis.com/projects/123456789012"
):
    print(tag.namespaced_tag_key, tag.namespaced_tag_value)
```

## Configuration Notes

### Resource names are strict

Common forms:

- Projects client methods: `projects/my-project-id`
- Folder methods: `folders/123456789012`
- Organization methods: `organizations/123456789012`
- Tag binding parent values: `//cloudresourcemanager.googleapis.com/projects/123456789012`

Passing a bare ID where the API expects a full resource name is a common cause of `InvalidArgument` errors.

### Search and list are not interchangeable

- `search_projects()` supports queries and is appropriate for discovery
- `list_projects()` requires a known parent and is better for enumerating that container

Agents often guess one when the other is required and end up debugging the wrong thing.

### Long-running operations are normal

`create_project()`, `move_project()`, `delete_project()`, `undelete_project()`, and some tag operations return long-running operations. Always wait on `.result()` and handle timeout, permission, and conflict errors explicitly.

### Endpoint overrides are specialized

These clients accept `client_options`, including custom `api_endpoint`. Leave the default endpoint alone unless you are working with mTLS, Private Service Connect, or a specialized Google Cloud environment.

### SDK logging is opt-in

Google's Python client libraries support environment-based logging configuration. If you need transport-level visibility while debugging:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

That is useful for auth and request debugging, but do not leave verbose logging enabled by default in production.

## Common Pitfalls

- Do not import `google.cloud.resource_manager`; use `google.cloud.resourcemanager_v3`.
- Do not assume the package version and API namespace match. This package is `1.16.0`, but the API surface is still `v3`.
- Do not pass bare project IDs where a full resource name is required.
- Do not reuse the `projects/...` format for tag-binding parents; tag APIs use the `//cloudresourcemanager.googleapis.com/...` form.
- Do not assume successful ADC means the operation is authorized. Most failures here are IAM or org-policy issues, not client-construction bugs.
- Do not fire-and-forget project create, move, delete, or undelete calls. They are long-running operations.
- Do not reach for service account key files first in local development when user ADC or impersonation will work.

## Version-Sensitive Notes

- PyPI lists `1.16.0` for `google-cloud-resource-manager` as of March 12, 2026.
- The canonical reference pages for clients such as `ProjectsClient` and `TagBindingsClient` show the `1.16.0` library surface.
- The docs root and changelog navigation still show `1.15.0` in some places. Treat that as documentation-site lag, not as proof that `1.16.0` is unsupported.
- The package still exposes the `resourcemanager_v3` namespace. Do not rewrite imports or sample code to a hypothetical `v1_16` or `v4` namespace based on the package version.

## Official Source URLs Used

- PyPI package: `https://pypi.org/project/google-cloud-resource-manager/`
- Google Cloud reference root: `https://cloud.google.com/python/docs/reference/cloudresourcemanager/latest`
- `ProjectsClient` reference: `https://cloud.google.com/python/docs/reference/cloudresourcemanager/latest/google.cloud.resourcemanager_v3.services.projects.ProjectsClient`
- `TagBindingsClient` reference: `https://cloud.google.com/python/docs/reference/cloudresourcemanager/latest/google.cloud.resourcemanager_v3.services.tag_bindings.TagBindingsClient`
- Changelog: `https://cloud.google.com/python/docs/reference/cloudresourcemanager/latest/changelog`
- ADC overview: `https://cloud.google.com/docs/authentication/provide-credentials-adc`
- ADC local development: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- Repository path: `https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-resource-manager`

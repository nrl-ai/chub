---
name: containerregistry
description: "Azure Container Registry client library for Python: authenticate, inspect repositories, tags, manifests, and blobs"
metadata:
  languages: "python"
  versions: "1.2.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,acr,container-registry,oci,artifacts,python"
---

# azure-containerregistry Python Package Guide

## Golden Rule

Use `azure-containerregistry` for Azure Container Registry data-plane work in Python: browse repositories, inspect tags and manifests, download blobs, and manage artifact metadata. Do not use it to create or configure registries in Azure Resource Manager.

## Installation

Pin to the package version you are targeting:

```bash
python -m pip install "azure-containerregistry==1.2.0"
```

For Azure AD authentication with `DefaultAzureCredential`, install `azure-identity` too:

```bash
python -m pip install "azure-containerregistry==1.2.0" "azure-identity>=1.12.0"
```

PyPI metadata for `1.2.0` lists Python `>=3.7`. Microsoft Learn's current quickstart says Python `3.8+`; treat that as current-sample guidance rather than a version-pinned rule for `1.2.0`.

## Authentication And Setup

Use the registry login server URL, not a portal URL and not an image reference:

```python
import os

from azure.containerregistry import ContainerRegistryClient
from azure.identity import DefaultAzureCredential

account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]  # https://<registry>.azurecr.io
credential = DefaultAzureCredential()

with ContainerRegistryClient(account_url, credential) as client:
    for repository in client.list_repository_names():
        print(repository)
```

Local development usually works with `az login` plus `DefaultAzureCredential`. For non-interactive use, set the standard service principal environment variables:

- `AZURE_CLIENT_ID`
- `AZURE_TENANT_ID`
- `AZURE_CLIENT_SECRET`
- `CONTAINERREGISTRY_ENDPOINT`

### Anonymous Access

The client constructor allows `credential=None` for anonymous pulls. That only works when the registry and repository are configured for anonymous access.

```python
from azure.containerregistry import ContainerRegistryClient

account_url = "https://myregistry.azurecr.io"

with ContainerRegistryClient(account_url, credential=None) as client:
    for tag in client.list_tag_properties("library/hello-world"):
        print(tag.name, tag.digest)
```

## Core Usage

### List Repositories

```python
import os

from azure.containerregistry import ContainerRegistryClient
from azure.identity import DefaultAzureCredential

account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]

with ContainerRegistryClient(account_url, DefaultAzureCredential()) as client:
    for repository in client.list_repository_names():
        print(repository)
```

### Inspect Repository Properties

```python
import os

from azure.containerregistry import ContainerRegistryClient
from azure.identity import DefaultAzureCredential

account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]
repository = "hello-world"

with ContainerRegistryClient(account_url, DefaultAzureCredential()) as client:
    props = client.get_repository_properties(repository)
    print(props.name, props.tag_count, props.manifest_count)
```

### List Tags

```python
import os

from azure.containerregistry import ContainerRegistryClient
from azure.identity import DefaultAzureCredential

account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]
repository = "hello-world"

with ContainerRegistryClient(account_url, DefaultAzureCredential()) as client:
    for tag in client.list_tag_properties(repository):
        print(tag.name, tag.digest, tag.last_updated_on)
```

### List Manifests

```python
import os

from azure.containerregistry import ContainerRegistryClient
from azure.identity import DefaultAzureCredential

account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]
repository = "hello-world"

with ContainerRegistryClient(account_url, DefaultAzureCredential()) as client:
    for manifest in client.list_manifest_properties(repository):
        print(manifest.digest, manifest.tags, manifest.last_updated_on)
```

### Read A Specific Tag

```python
import os

from azure.containerregistry import ContainerRegistryClient
from azure.identity import DefaultAzureCredential

account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]

with ContainerRegistryClient(account_url, DefaultAzureCredential()) as client:
    tag = client.get_tag_properties("hello-world", "latest")
    print(tag.name, tag.digest, tag.created_on)
```

### Download A Blob

Use blob digests when pulling layer or config content.

```python
import os

from azure.containerregistry import ContainerRegistryClient
from azure.identity import DefaultAzureCredential

account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]
repository = "hello-world"
digest = "sha256:..."

with ContainerRegistryClient(account_url, DefaultAzureCredential()) as client:
    stream = client.download_blob(repository, digest)
    with open("blob.bin", "wb") as fh:
        for chunk in stream.chunks():
            fh.write(chunk)
```

## Async Usage

The async client lives under `azure.containerregistry.aio`. Close the async credential explicitly.

```python
import asyncio
import os

from azure.containerregistry.aio import ContainerRegistryClient
from azure.identity.aio import DefaultAzureCredential

async def main() -> None:
    account_url = os.environ["CONTAINERREGISTRY_ENDPOINT"]
    credential = DefaultAzureCredential()

    try:
        async with ContainerRegistryClient(account_url, credential) as client:
            async for repository in client.list_repository_names():
                print(repository)
    finally:
        await credential.close()

asyncio.run(main())
```

## Configuration Notes

- The constructor default `audience` is `https://containerregistry.azure.net`.
- The current class reference also documents alternate audiences for Azure public management, China cloud, and US Gov cloud.
- The current default service API version in the class reference is `2021-07-01`.
- Client and per-call kwargs support the usual Azure SDK pipeline options such as transport and logging.

Start with the default audience unless you have a sovereign-cloud or token-audience requirement. This is an inference from the official sources because some Microsoft samples override `audience` while the class reference documents a different default.

## Write And Delete Operations

The package supports upload and mutation flows too, including:

- `upload_blob`
- `set_manifest`
- `update_manifest_properties`
- `update_tag_properties`
- `update_repository_properties`
- `delete_manifest`
- `delete_tag`
- `delete_blob`
- `delete_repository`

Check the exact signature in the current API reference before copying mutation examples. Microsoft Learn currently mixes quickstart snippets that pass permission flags directly with API reference signatures that use property objects such as `ArtifactManifestProperties` and `ArtifactTagProperties`.

## Common Pitfalls

- `azure-containerregistry` is a data-plane SDK, not the Azure management SDK. Use `azure-mgmt-containerregistry` for registry provisioning and ARM operations.
- The endpoint must be the registry login server with `https://`, for example `https://myregistry.azurecr.io`.
- A repository name like `hello-world` and an image reference like `myregistry.azurecr.io/hello-world:latest` are not interchangeable inputs.
- `DefaultAzureCredential` comes from `azure-identity`; installing only `azure-containerregistry` is not enough for AAD auth.
- Anonymous access is opt-in at the registry or repository level. `credential=None` is not a fallback for private registries.
- `download_blob` and manifest retrieval workflows are digest-based. Keep tag names and digests separate in your code.
- The class reference documents that `delete_repository` does not raise on `404`, so successful deletion does not prove the repository existed.
- The class reference documents digest validation and response-size edge cases around manifest retrieval. If you are reading large manifests or verifying digests, handle those exceptions explicitly.

## Version-Sensitive Notes

- The version used here and current PyPI latest version both point to `1.2.0`, so this entry is aligned to the package release you asked for.
- The current Microsoft Learn quickstart still imports `ManifestOrder` and shows mutation helpers with direct permission kwargs.
- The current Microsoft Learn API reference documents `ArtifactManifestOrder` and property-object-based mutation signatures.
- The PyPI project page for `1.2.0` still links its API reference to an older `1.0.0b1` Azure SDK docs URL.

Practical rule: if a Microsoft sample and the API reference disagree, trust the API reference for exact symbol names and signatures, then verify against your installed package in a REPL before writing production code.

## Official Sources

- Microsoft Learn package reference: https://learn.microsoft.com/en-us/python/api/azure-containerregistry/?view=azure-python
- Microsoft Learn `ContainerRegistryClient` reference: https://learn.microsoft.com/en-us/python/api/azure-containerregistry/azure.containerregistry.containerregistryclient?view=azure-python
- Microsoft Learn quickstart and samples: https://learn.microsoft.com/en-us/azure/container-registry/quickstart-client-libraries
- PyPI project page: https://pypi.org/project/azure-containerregistry/
- PyPI release page for `1.2.0`: https://pypi.org/project/azure-containerregistry/1.2.0/

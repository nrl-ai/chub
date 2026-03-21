---
name: common
description: "azure-common package guide for Python legacy Azure SDK auth helpers, client factories, cloud helpers, and compatibility exceptions"
metadata:
  languages: "python"
  versions: "1.1.28"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,azure-common,python,legacy,authentication,management"
---

# azure-common Python Package Guide

## What It Is

`azure-common` is a legacy support package from the older Azure SDK for Python stack. In `1.1.28`, the source tree still exposes four main areas:

- `azure.common.credentials`
- `azure.common.client_factory`
- `azure.common.cloud`
- `azure.common.exceptions`

It also defines package-level compatibility exceptions in `azure.common` such as `AzureHttpError`, `AzureConflictHttpError`, and `AzureMissingResourceHttpError`.

Use this package only when you are maintaining older Azure SDK code that already imports `azure.common.*`. For new Azure Python work, Microsoft directs you to service-specific modern SDKs plus `azure-identity`.

## Version Scope

- Ecosystem: `pypi`
- Package: `azure-common`
- Version covered: `1.1.28`
- Release state: final published release in the `1.x` line
- Support state: deprecated; Azure's deprecated-packages index lists support end on `2023-03-31`
- Docs root: `https://learn.microsoft.com/en-us/python/api/azure-common/`
- Registry URL: `https://pypi.org/project/azure-common/`

The version used here for this session is `1.1.28`, and that matches the tagged upstream source, the PyPI simple index, and Azure's deprecated-packages page.

## Install

Install only when the target project already depends on the legacy Azure SDK generation:

```bash
python -m pip install azure-common==1.1.28
```

With `uv`:

```bash
uv pip install azure-common==1.1.28
```

With Poetry:

```bash
poetry add azure-common==1.1.28
```

Practical install notes:

- `azure-common` is not a standalone Azure client. It is a support package used by older Azure SDK libraries.
- The upstream `setup.py` explicitly rejects the old `azure` metapackage (`azure==0.x`). If that metapackage is installed, remove it before troubleshooting imports.
- The credentials and exception helpers import `msrest` and `msrestazure`. In older projects, those usually arrive as transitive dependencies from the service client package.

## When You Still Need It

Keep `azure-common` only if the project already uses older Azure packages that expect one of these patterns:

- `ServicePrincipalCredentials`, `UserPassCredentials`, or `InteractiveCredentials`
- `get_client_from_cli_profile(...)`
- `get_azure_cli_credentials(...)`
- `get_client_from_json_dict(...)` or `get_client_from_auth_file(...)`
- `azure.common.exceptions.CloudError` or package-root `AzureHttpError` compatibility handling

If the target library expects an `azure-identity` credential object with `get_token`, `azure-common` is the wrong auth layer.

## Initialization And Auth

### Service principal credentials for older clients

The legacy credential type is `ServicePrincipalCredentials`:

```python
from azure.common.credentials import ServicePrincipalCredentials

tenant_id = "<tenant-id>"
client_id = "<client-id>"
client_secret = "<client-secret>"

credentials = ServicePrincipalCredentials(
    client_id=client_id,
    secret=client_secret,
    tenant=tenant_id,
)
```

Older management-plane clients usually pair this with a separate subscription ID argument:

```python
from azure.common.credentials import ServicePrincipalCredentials
from azure.mgmt.resource import ResourceManagementClient

subscription_id = "<subscription-id>"

credentials = ServicePrincipalCredentials(
    client_id="<client-id>",
    secret="<client-secret>",
    tenant="<tenant-id>",
)

resource_client = ResourceManagementClient(credentials, subscription_id)
```

This is the legacy pattern. If the client constructor expects a modern `TokenCredential`, switch to `azure-identity` instead of forcing `azure-common` into new code.

### Azure CLI credentials

`1.1.28` still ships `get_azure_cli_credentials`, but the source marks it deprecated and explicitly states it does not work with `azure-cli-core>=2.21.0`.

```python
from azure.common.credentials import get_azure_cli_credentials

credentials, subscription_id = get_azure_cli_credentials()
```

Important behavior in `1.1.28`:

- it emits a deprecation warning
- it raises `NotImplementedError` when the installed CLI core version is `2.21.0` or newer
- Microsoft recommends `azure-identity` with `AzureCliCredential` instead

### Client factory from CLI login

For older management clients, `get_client_from_cli_profile(...)` can still instantiate a client from the local Azure CLI profile:

```python
from azure.common.client_factory import get_client_from_cli_profile
from azure.mgmt.resource import SubscriptionClient

subscription_client = get_client_from_cli_profile(SubscriptionClient)
```

This helper is also deprecated in `1.1.28` and has the same `azure-cli-core<2.21.0` limitation.

### JSON and auth-file based helpers

The client-factory module still includes:

- `get_client_from_json_dict(...)`
- `get_client_from_auth_file(...)`

These are legacy migration/maintenance helpers, not good defaults for new code. The source and migration guide both direct new work toward `azure-identity` instead of JSON-file credential loading.

If you must keep the old pattern during maintenance, `get_client_from_auth_file(...)` reads JSON from either:

- an explicit `auth_path`, or
- `AZURE_AUTH_LOCATION`

The JSON shape expected by the helper includes fields such as:

- `clientId`
- `clientSecret`
- `subscriptionId`
- `tenantId`
- `activeDirectoryEndpointUrl`
- `resourceManagerEndpointUrl`

## Core Usage Surface

### Credentials module

`azure.common.credentials` is mostly a re-export layer around older auth classes:

- `ServicePrincipalCredentials`
- `UserPassCredentials`
- `InteractiveCredentials`
- `BasicAuthentication`
- `BasicTokenAuthentication`
- `OAuthTokenAuthentication`

The CLI helpers live in the same module:

- `get_cli_profile()`
- `get_azure_cli_credentials(...)`

### Client factory module

`azure.common.client_factory` is meant for older autogenerated management clients. In `1.1.28`, its helpers still try to auto-fill arguments such as:

- `credentials` or `credential`
- `subscription_id`
- `base_url`
- `tenant_id`

This is useful when reading legacy code because it explains why older client construction sometimes looks incomplete.

### Cloud helper

`azure.common.cloud.get_cli_active_cloud()` exists only to read the active Azure CLI cloud. In `1.1.28`, it is deprecated and subject to the same `azure-cli-core>=2.21.0` breakage as the other CLI helpers.

Treat it as a maintenance-only helper for older sovereign-cloud logic. Do not build new cloud-selection code on top of it.

### Exception surface

There are two exception layers to keep straight:

- `azure.common.exceptions` re-exports exception types from `msrest` plus `msrestazure.azure_exceptions.CloudError`
- package-root `azure.common` defines `AzureException`, `AzureHttpError`, `AzureConflictHttpError`, and `AzureMissingResourceHttpError`

For example:

```python
from azure.common import AzureHttpError, AzureMissingResourceHttpError
from azure.common.exceptions import CloudError

try:
    do_legacy_call()
except AzureMissingResourceHttpError:
    handle_missing()
except CloudError as exc:
    print(exc)
except AzureHttpError as exc:
    print(exc.status_code)
```

Do not assume newer Azure SDK clients will raise these types.

## Configuration Notes

`azure-common` has no central config file. Legacy setups usually depend on one of these sources:

- service-principal secrets from environment variables or secret stores
- local Azure CLI profile state for development-time helpers
- auth JSON passed to `get_client_from_json_dict(...)` or read from `AZURE_AUTH_LOCATION`
- cloud/base URL values inferred from the active CLI cloud

Practical guidance:

- keep the subscription ID separate from credential construction unless the specific helper returns it for you
- never commit auth JSON files; if they must exist temporarily, keep them out of version control
- verify the expected resource or base URL for older clients, especially in sovereign-cloud environments

## Common Pitfalls

### Mixing `azure-common` credentials with modern clients

Microsoft's migration guide calls out the API mismatch directly:

- modern clients expect an `azure-identity` credential with `get_token`
- older clients expect `azure-common` / `msrest` style auth with `signed_session`

If you see errors like missing `get_token` or missing `signed_session`, you are probably mixing SDK generations.

### Expecting CLI helpers to work with a current Azure CLI core

In `1.1.28`, the CLI-based helpers are deprecated and intentionally fail on `azure-cli-core>=2.21.0`. If an old script depends on them, you either need an older CLI core in a controlled compatibility environment or a migration to `AzureCliCredential`.

### Treating auth-file helpers as a long-term design

`get_client_from_json_dict(...)` and `get_client_from_auth_file(...)` are legacy helpers. They are useful for keeping old code running, but they are not the recommended auth path anymore.

### Importing from the wrong exception module

`azure.common.exceptions` does not define the package-root `AzureHttpError` classes. If legacy code imports `AzureHttpError`, it should come from `azure.common`, not `azure.common.exceptions`.

### Installing the old `azure` metapackage

The upstream package setup explicitly rejects the legacy `azure==0.x` metapackage. If import behavior looks inconsistent, check for that package first.

## Version-Sensitive Notes

### `1.1.28`

- final tagged package version
- release date in the upstream changelog: `2022-02-03`
- deprecated all helpers that need access to Azure CLI internals
- raises `NotImplementedError` when CLI credential helpers are used with `azure-cli-core>=2.21.0`

### `1.1.27`

- deprecated the JSON-dict and auth-file client-factory approach

### Support lifecycle

- Azure's deprecated Python SDK index lists `azure-common` support ending on `2023-03-31`
- treat any remaining usage as legacy maintenance, not as a stable foundation for new Azure integrations

## Migration Boundary

When you touch code that imports `azure.common.*`, use this decision rule:

1. Keep `azure-common` only if the surrounding service client is still from the older Azure SDK generation.
2. If the client is modern, migrate the auth layer to `azure-identity` instead of adapting legacy credentials.
3. Replace CLI helpers with `AzureCliCredential` when the client supports modern credentials.
4. Replace `ServicePrincipalCredentials` with `ClientSecretCredential` when moving to newer clients.

## Recommended Agent Workflow

1. Confirm whether the target Azure package is legacy or modern before writing code.
2. Match the installed `azure-common` version to `1.1.28` if you are debugging a current pinned environment.
3. Check for `azure-cli-core` usage before relying on CLI helper functions.
4. Keep auth-file and JSON-dict helpers only for compatibility work.
5. Prefer a staged migration to `azure-identity` instead of adding new `azure-common` usage.

## Official Sources

- Microsoft Learn package URL: `https://learn.microsoft.com/en-us/python/api/azure-common/`
- Azure deprecated Python packages index: `https://azure.github.io/azure-sdk/releases/deprecated/python.html`
- Azure SDK for Python `azure-common` source tree at tag `azure-common_1.1.28`: `https://github.com/Azure/azure-sdk-for-python/tree/azure-common_1.1.28/sdk/core/azure-common/`
- `azure-common` `1.1.28` source files:
  - `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/azure-common_1.1.28/sdk/core/azure-common/azure/common/credentials.py`
  - `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/azure-common_1.1.28/sdk/core/azure-common/azure/common/client_factory.py`
  - `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/azure-common_1.1.28/sdk/core/azure-common/azure/common/cloud.py`
  - `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/azure-common_1.1.28/sdk/core/azure-common/azure/common/exceptions.py`
  - `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/azure-common_1.1.28/sdk/core/azure-common/azure/common/__init__.py`
  - `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/azure-common_1.1.28/sdk/core/azure-common/setup.py`
  - `https://raw.githubusercontent.com/Azure/azure-sdk-for-python/azure-common_1.1.28/sdk/core/azure-common/CHANGELOG.md`
- Azure Identity migration guide: `https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/identity/azure-identity/migration_guide.md`
- PyPI simple index for package release files: `https://pypi.org/simple/azure-common/`

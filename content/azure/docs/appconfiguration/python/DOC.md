---
name: appconfiguration
description: "azure-appconfiguration package guide for Python covering client setup, authentication, CRUD operations, filtering, snapshots, async usage, and 1.8.0-specific notes"
metadata:
  languages: "python"
  versions: "1.8.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,app-configuration,configuration,entra-id,async,snapshots"
---

# azure-appconfiguration Python Package Guide

## What It Is

`azure-appconfiguration` is the low-level Azure SDK client for reading and writing data in an Azure App Configuration store.

Use it when your code needs to:

- create, update, delete, or query configuration settings directly
- work with labels, tags, revision history, and point-in-time reads
- create or inspect snapshots
- integrate App Configuration access into service code or automation

Do not confuse it with `azure-appconfiguration-provider`:

- `azure-appconfiguration` is the data-plane CRUD client
- `azure-appconfiguration-provider` is the higher-level config loader for app startup, `load(...)`, `SettingSelector`, prefix trimming, Key Vault reference resolution, and feature-flag loading

## Version Context

- Ecosystem: `pypi`
- Package: `azure-appconfiguration`
- Version covered: `1.8.0`
- PyPI release date: `2026-01-27`
- Python requirement on PyPI: `>=3.8`
- Docs root: `https://learn.microsoft.com/en-us/python/api/azure-appconfiguration/`
- Registry: `https://pypi.org/project/azure-appconfiguration/`
- Repository: `https://github.com/Azure/azure-sdk-for-python/tree/main/sdk/appconfiguration/azure-appconfiguration`

This guide is written for the `1.8.0` line reflected by the current Microsoft Learn package guide and PyPI release.

## Install

Basic install:

```bash
python -m pip install azure-appconfiguration==1.8.0
```

If you want Microsoft Entra ID authentication:

```bash
python -m pip install azure-appconfiguration==1.8.0 azure-identity
```

## Authentication And Setup

The SDK has two normal entry paths:

- connection string via `AzureAppConfigurationClient.from_connection_string(...)`
- Microsoft Entra ID via `AzureAppConfigurationClient(base_url=..., credential=...)`

Prefer Entra ID for deployed applications. Use connection strings mainly for local development, bootstrap flows, or controlled automation.

### Recommended Environment Variables

```bash
export AZURE_APPCONFIG_ENDPOINT="https://<store-name>.azconfig.io"
export AZURE_APPCONFIG_CONNECTION_STRING="Endpoint=https://<store-name>.azconfig.io;Id=...;Secret=..."
```

Upstream examples also use `APPCONFIGURATION_CONNECTION_STRING`. The package does not require a specific env var name, but your codebase should pick one convention and stick to it.

### Connection String Client

```python
import os

from azure.appconfiguration import AzureAppConfigurationClient

client = AzureAppConfigurationClient.from_connection_string(
    os.environ["AZURE_APPCONFIG_CONNECTION_STRING"]
)
```

To retrieve the connection string with Azure CLI:

```bash
az appconfig credential list --name <config-store-name>
```

### Microsoft Entra ID Client

```python
import os

from azure.appconfiguration import AzureAppConfigurationClient
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential()
client = AzureAppConfigurationClient(
    base_url=os.environ["AZURE_APPCONFIG_ENDPOINT"],
    credential=credential,
)
```

Important auth notes:

- `azure-identity` is a separate dependency
- Entra auth needs data-plane roles, not just control-plane roles
- role propagation can take up to 15 minutes after assignment

Required App Configuration roles:

- `App Configuration Data Reader` for read-only access
- `App Configuration Data Owner` for read/write/delete access

Store-level `Reader`, `Contributor`, or `Owner` does not by itself grant data-plane access through Entra ID.

### Sovereign Cloud Audience

`1.8.0` adds an `audience=` keyword argument on `AzureAppConfigurationClient(...)` for non-public Azure clouds.

```python
client = AzureAppConfigurationClient(
    base_url=os.environ["AZURE_APPCONFIG_ENDPOINT"],
    credential=DefaultAzureCredential(),
    audience="<supported audience for your cloud>",
)
```

Only set `audience` when the default public-cloud audience is wrong for your environment.

## Core Model

The main resource type is `ConfigurationSetting`.

Common fields you will use:

- `key`
- `label`
- `value`
- `content_type`
- `tags`
- `read_only`
- `etag`
- `last_modified`

A setting is identified by `(key, label)`. Unlabeled and labeled values are different records.

## Core CRUD Usage

### Create Only

Use `add_configuration_setting(...)` when the setting must not already exist.

```python
from azure.appconfiguration import AzureAppConfigurationClient, ConfigurationSetting

client = AzureAppConfigurationClient.from_connection_string(
    "Endpoint=https://<store>.azconfig.io;Id=...;Secret=..."
)

setting = ConfigurationSetting(
    key="app:message",
    label="prod",
    value="hello",
    content_type="text/plain",
    tags={"team": "api"},
)

created = client.add_configuration_setting(setting)
print(created.key, created.label, created.value)
```

### Upsert

Use `set_configuration_setting(...)` when create-or-replace behavior is acceptable.

```python
setting.value = "hello again"
saved = client.set_configuration_setting(setting)
print(saved.etag)
```

### Read One Setting

```python
current = client.get_configuration_setting(
    key="app:message",
    label="prod",
)

print(current.value)
print(current.content_type)
print(current.tags)
```

### Delete One Setting

```python
deleted = client.delete_configuration_setting(
    key="app:message",
    label="prod",
)

print(deleted)
```

## Safe Updates With ETags

Use `etag` plus `match_condition` when you do not want concurrent writers to overwrite each other silently.

```python
from azure.core import MatchConditions

current = client.get_configuration_setting(key="app:message", label="prod")
current.value = "new value"

updated = client.set_configuration_setting(
    current,
    etag=current.etag,
    match_condition=MatchConditions.IfNotModified,
)
```

The same pattern matters for deletes and read-only changes.

## Filters, History, And Labels

### List Settings

```python
items = client.list_configuration_settings(
    key_filter="app:*",
    label_filter="prod",
    tags_filter=["team=api"],
)

for item in items:
    print(item.key, item.label, item.value)
```

Useful points:

- `*` works as a wildcard at the beginning or end of supported filters
- `tags_filter` expects strings like `"key=value"`
- results are paged iterators

### List Labels

```python
for label in client.list_labels(name="prod*"):
    print(label)
```

This is useful when you need to discover environment labels before reading settings.

### Revision History

```python
for item in client.list_revisions(
    key_filter="app:message",
    label_filter="prod",
):
    print(item.last_modified, item.etag, item.value)
```

### Point-In-Time Reads

```python
from datetime import datetime, timedelta, timezone

point_in_time = datetime.now(timezone.utc) - timedelta(hours=1)

historical = client.get_configuration_setting(
    key="app:message",
    label="prod",
    accept_datetime=point_in_time,
)

print(historical.value)
```

`accept_datetime` is available on current `get_configuration_setting(...)`, `list_configuration_settings(...)`, `list_labels(...)`, and `list_revisions(...)` flows.

## Read-Only State

```python
current = client.get_configuration_setting(key="app:message", label="prod")

client.set_read_only(current)
client.set_read_only(current, read_only=False)
```

This locks or unlocks the server-side key-value. It is a useful safeguard, but it does not replace RBAC.

## Snapshots

`1.8.0` includes snapshot APIs for point-in-time configuration sets.

```python
from azure.appconfiguration import ConfigurationSettingsFilter

filters = [
    ConfigurationSettingsFilter(key="app:*", label="prod"),
]

poller = client.begin_create_snapshot(
    name="release-2026-03-12",
    filters=filters,
    tags={"release": "2026-03-12"},
)
snapshot = poller.result()

print(snapshot.name, snapshot.status)
```

Related APIs in the current line:

- `get_snapshot(...)`
- `list_snapshots(...)`
- `archive_snapshot(...)`
- `recover_snapshot(...)`

Use snapshots when you need a stable set of configuration values that should not drift while a deployment or rollout is in progress.

## Async Usage

Async support lives under `azure.appconfiguration.aio`.

```python
import asyncio
import os

from azure.appconfiguration.aio import AzureAppConfigurationClient

async def main() -> None:
    client = AzureAppConfigurationClient.from_connection_string(
        os.environ["AZURE_APPCONFIG_CONNECTION_STRING"]
    )
    try:
        setting = await client.get_configuration_setting(
            key="app:message",
            label="prod",
        )
        print(setting.value)

        async for item in client.list_revisions(key_filter="app:*"):
            print(item.key, item.value)
    finally:
        await client.close()

asyncio.run(main())
```

Use the async client if the rest of your service is already async. Iterate paged results with `async for`, and close the client when you are done.

## When To Use `azure-appconfiguration-provider` Instead

Choose `azure-appconfiguration-provider` instead of this package when your main goal is application configuration loading rather than low-level key-value management.

The provider package adds higher-level behaviors such as:

- `load(...)`
- `SettingSelector`
- prefix trimming
- Key Vault reference resolution
- feature-flag loading

That package is better for startup-time config hydration. `azure-appconfiguration` is better for direct SDK-style reads, writes, history queries, and snapshot management.

## Common Pitfalls

- Installing only `azure-appconfiguration` and then using `DefaultAzureCredential` without also installing `azure-identity`
- Using Entra ID with the wrong IAM role; data-plane access needs `App Configuration Data Reader` or `App Configuration Data Owner`
- Treating an unlabeled key and a labeled key as the same setting
- Using `set_configuration_setting(...)` when you really needed create-only behavior from `add_configuration_setting(...)`
- Skipping `etag` checks in multi-writer code paths
- Using the low-level client where the provider package would handle selectors, trimming, and feature-flag loading more cleanly

## Version-Sensitive Notes For 1.8.0

- PyPI lists `1.8.0` as the latest stable release on `2026-01-27`
- PyPI requires Python `>=3.8`
- The constructor docs for the current line show default `api_version="2023-11-01"`; overriding it may lead to unsupported behavior
- Azure SDK February 2026 release notes call out new `1.8.0` support for constructor `audience=...` and `by_page(match_conditions=...)` on the iterator returned by `list_configuration_settings()`
- If your project is pinned to an older `1.x` release, re-check snapshot support and any pagination or audience-related code against that exact version before copying examples

## Official Sources Used

- Microsoft Learn package guide: `https://learn.microsoft.com/en-us/python/api/overview/azure/appconfiguration-readme?view=azure-python`
- Microsoft Learn client API reference: `https://learn.microsoft.com/en-us/python/api/azure-appconfiguration/azure.appconfiguration.azureappconfigurationclient?view=azure-python`
- Microsoft Learn `ConfigurationSetting` reference: `https://learn.microsoft.com/en-us/python/api/azure-appconfiguration/azure.appconfiguration.configurationsetting?view=azure-python`
- Microsoft Learn RBAC guidance: `https://learn.microsoft.com/en-us/azure/azure-app-configuration/concept-enable-rbac`
- Microsoft Learn provider reference: `https://learn.microsoft.com/en-us/azure/azure-app-configuration/reference-python-provider`
- PyPI package page: `https://pypi.org/project/azure-appconfiguration/`
- Azure SDK release notes: `https://azure.github.io/azure-sdk/releases/2026-02/python.html`

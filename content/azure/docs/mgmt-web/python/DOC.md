---
name: mgmt-web
description: "Azure App Service management SDK for Python for App Service plans, web apps, app settings, source control, and related control-plane operations"
metadata:
  languages: "python"
  versions: "10.1.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "azure,app-service,azure-mgmt-web,management,web-apps,app-service-plan"
---

# Azure App Service Management SDK for Python

## Golden Rule

Use `azure-mgmt-web` for Azure App Service control-plane work: creating and updating App Service plans, web apps, app settings, deployment source control, certificates, and related Microsoft.Web resources. Install `azure-identity` with it, authenticate with `DefaultAzureCredential` or a narrower Entra credential, and do not treat this package as the runtime deployment SDK for your application code.

This package manages Azure resources. Your app code deployment flow usually still involves Azure CLI, ZipDeploy, GitHub Actions, or another CI/CD path.

## Install

Pin the management package version your project expects and install `azure-identity` alongside it:

```bash
python -m pip install "azure-mgmt-web==10.1.0" azure-identity
```

Common alternatives:

```bash
uv add "azure-mgmt-web==10.1.0" azure-identity
poetry add "azure-mgmt-web==10.1.0" azure-identity
```

If you plan to use the async client surface, install an async transport too:

```bash
python -m pip install "azure-mgmt-web==10.1.0" azure-identity aiohttp
```

## Authentication And Setup

The current official package description shows this baseline setup:

- `AZURE_SUBSCRIPTION_ID`
- `AZURE_TENANT_ID`
- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`

For most real projects, prefer one of these credential patterns:

1. `DefaultAzureCredential()` for code that must run locally, in CI, and in Azure.
2. `AzureCliCredential()` for local scripts after `az login`.
3. `ManagedIdentityCredential()` or workload identity once the runtime environment is fixed.

Basic sync client setup:

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.web import WebSiteManagementClient

subscription_id = os.environ["AZURE_SUBSCRIPTION_ID"]
credential = DefaultAzureCredential()

client = WebSiteManagementClient(
    credential=credential,
    subscription_id=subscription_id,
)
```

Local CLI-driven scripts can be explicit:

```python
import os

from azure.identity import AzureCliCredential
from azure.mgmt.web import WebSiteManagementClient

client = WebSiteManagementClient(
    credential=AzureCliCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)
```

The official `WebSiteManagementClient` docs also expose `cloud_setting=`. Use that when you target sovereign clouds so the credential authority and ARM endpoint stay aligned with the cloud you are using.

## Core Client Surface

The current client exposes these high-value operation groups:

- `app_service_plans`
- `web_apps`
- `static_sites`
- `site_certificates`
- `certificates`
- `domains`
- `deleted_web_apps`
- `diagnostics`

For ordinary App Service automation, most code lives in `client.app_service_plans` and `client.web_apps`.

## Core Usage

### Create an App Service plan

The official docs show `app_service_plans.begin_create_or_update(...)` taking an `AppServicePlan` payload. For Linux plans, set `reserved=True`.

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.web import WebSiteManagementClient
from azure.mgmt.web.models import AppServicePlan, SkuDescription

resource_group = "example-rg"
plan_name = "example-plan"
location = "westus2"

client = WebSiteManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)

poller = client.app_service_plans.begin_create_or_update(
    resource_group,
    plan_name,
    AppServicePlan(
        location=location,
        reserved=True,
        sku=SkuDescription(
            name="B1",
            tier="Basic",
            size="B1",
            capacity=1,
        ),
    ),
)

plan = poller.result()
print(plan.id)
```

Important details:

- `AppServicePlan.location` is required.
- `reserved=True` is what marks the plan as Linux in the current model.
- `begin_create_or_update()` returns a poller. Call `.result()` when you need completion before the next step.

### Create a web app on that plan

The `Site` model requires `location`; the `server_farm_id` should point at the App Service plan resource ID.

```python
import os

from azure.identity import DefaultAzureCredential
from azure.mgmt.web import WebSiteManagementClient
from azure.mgmt.web.models import Site, SiteConfig

resource_group = "example-rg"
plan_name = "example-plan"
app_name = "example-app-12345"
location = "westus2"

client = WebSiteManagementClient(
    credential=DefaultAzureCredential(),
    subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
)

plan = client.app_service_plans.get(resource_group, plan_name)

poller = client.web_apps.begin_create_or_update(
    resource_group,
    app_name,
    Site(
        location=location,
        server_farm_id=plan.id,
        https_only=True,
        site_config=SiteConfig(
            linux_fx_version="PYTHON|3.11",
            always_on=True,
            ftps_state="Disabled",
            min_tls_version="1.2",
        ),
    ),
)

site = poller.result()
print(site.default_host_name)
```

Practical notes:

- `server_farm_id` should come from the actual plan object instead of being string-built by hand.
- `linux_fx_version` configures the runtime stack. Keep it aligned with a runtime that App Service currently supports.
- `https_only=True` and `min_tls_version="1.2"` are safer defaults than leaving those unset.

### List or inspect apps

`web_apps.list_by_resource_group(...)` is the direct way to enumerate sites in a resource group.

```python
for app in client.web_apps.list_by_resource_group("example-rg", include_slots=True):
    print(app.name, app.state, app.default_host_name)

app = client.web_apps.get("example-rg", "example-app-12345")
print(app.kind)
print(app.server_farm_id)
```

### Read and replace app settings

The current API surface uses `StringDictionary` for app settings. `update_application_settings(...)` replaces the application settings payload, so merge with the current values when you only want to change one key.

```python
from azure.mgmt.web.models import StringDictionary

current = client.web_apps.list_application_settings("example-rg", "example-app-12345")

settings = dict(current.properties or {})
settings["DJANGO_SETTINGS_MODULE"] = "mysite.settings"
settings["SCM_DO_BUILD_DURING_DEPLOYMENT"] = "1"

updated = client.web_apps.update_application_settings(
    "example-rg",
    "example-app-12345",
    StringDictionary(properties=settings),
)

print(updated.properties["DJANGO_SETTINGS_MODULE"])
```

Do not assume this is a patch operation. The official method description says it replaces the application settings of an app.

### Update site configuration after creation

For runtime-level settings such as startup commands, websockets, health checks, or runtime stack values, use the configuration operations rather than rewriting the full site resource.

```python
from azure.mgmt.web.models import SiteConfigResource

config = client.web_apps.get_configuration("example-rg", "example-app-12345")

config.linux_fx_version = "PYTHON|3.11"
config.always_on = True
config.health_check_path = "/healthz"
config.app_command_line = (
    "gunicorn -w 2 -k uvicorn.workers.UvicornWorker "
    "-b 0.0.0.0:8000 main:app"
)

client.web_apps.create_or_update_configuration(
    "example-rg",
    "example-app-12345",
    SiteConfigResource(
        linux_fx_version=config.linux_fx_version,
        always_on=config.always_on,
        health_check_path=config.health_check_path,
        app_command_line=config.app_command_line,
    ),
)
```

This is the safer pattern when you only need to manage configuration. It avoids rebuilding a large `Site` object with many server-populated fields.

### Configure deployment source control

`begin_create_or_update_source_control(...)` is the management-side way to connect a site to a repository.

```python
from azure.mgmt.web.models import SiteSourceControl

poller = client.web_apps.begin_create_or_update_source_control(
    "example-rg",
    "example-app-12345",
    SiteSourceControl(
        repo_url="https://github.com/example-org/example-app",
        branch="main",
        is_manual_integration=False,
        is_git_hub_action=True,
    ),
)

source_control = poller.result()
print(source_control.repo_url)
```

When you are driving GitHub Actions or another deployment system outside the SDK, prefer treating source control configuration as deployment metadata rather than assuming it deploys the app content by itself.

### Restart an app or inspect publishing credentials

```python
client.web_apps.restart(
    "example-rg",
    "example-app-12345",
    soft_restart=True,
    synchronous=True,
)

publishing_user = client.web_apps.begin_list_publishing_credentials(
    "example-rg",
    "example-app-12345",
).result()

print(publishing_user.publishing_user_name)
```

The current docs say `restart(...)` can be asynchronous unless you pass `synchronous=True`.

### Async client

The package also publishes `azure.mgmt.web.aio.WebSiteManagementClient`.

```python
import os
import asyncio

from azure.identity.aio import DefaultAzureCredential
from azure.mgmt.web.aio import WebSiteManagementClient

async def main():
    credential = DefaultAzureCredential()
    client = WebSiteManagementClient(
        credential=credential,
        subscription_id=os.environ["AZURE_SUBSCRIPTION_ID"],
    )
    try:
        async for app in client.web_apps.list_by_resource_group("example-rg"):
            print(app.name)
    finally:
        await client.close()
        await credential.close()

asyncio.run(main())
```

Keep sync and async stacks consistent. Do not mix `azure.identity.aio` credentials into the sync client.

## App Service Runtime Notes That Affect SDK Automation

`azure-mgmt-web` manages the Azure resources, but App Service runtime behavior still matters when you automate configuration:

- Microsoft Learn's current Python App Service guidance says Python on App Service is Linux-only.
- The runtime config article says App Service expects your Python dependencies to be available during deployment, and in practice `requirements.txt` at the project root still matters for build automation.
- If you are automating FastAPI deployment, the current quickstart says you must set a custom startup command such as Gunicorn with `uvicorn.workers.UvicornWorker`.

That is why many useful `azure-mgmt-web` automation flows combine:

1. `app_service_plans.begin_create_or_update(...)`
2. `web_apps.begin_create_or_update(...)`
3. `web_apps.update_application_settings(...)`
4. `web_apps.create_or_update_configuration(...)`
5. a separate deployment step outside this SDK

## Configuration Notes

- `subscription_id` is mandatory; the client does not infer it from the credential.
- The `base_url` and `cloud_setting` constructor parameters matter for sovereign clouds or custom ARM endpoints.
- App Service plan and site creation are often long-running operations; expect pollers from `begin_*` methods.
- `list_application_settings()` returns a `StringDictionary` whose actual key-values live under `.properties`.
- Several resource models contain many server-populated fields. When you only want to update configuration, use the narrow operation or patch model instead of sending back a whole fetched object unchanged.

## Version-Sensitive Notes

### `10.1.0`

PyPI release history for `10.1.0` shows:

- new `AppServicePlansOperations` methods for instance details and managed-instance worker operations
- new `AppServicePlan` fields such as `identity`, `install_scripts`, `network`, `storage_mounts`, and related custom-mode fields
- new `SitePatchResource.public_network_access`

If you are automating private networking or custom-mode plans, prefer current `10.1.0` docs over older examples.

### `10.0.0`

PyPI marks `10.0.0` as a breaking change: the package now targets only the latest available API version and removes older API-version folders. If your code depends on a specific older Microsoft.Web API version, pin an earlier `azure-mgmt-web` release instead of assuming `10.x` still exposes those modules.

### `9.0.0`

PyPI release history for `9.0.0` added fields such as:

- `Site.ssh_enabled`
- `Site.outbound_vnet_routing`
- `Site.client_affinity_proxy_enabled`
- `SiteConfig.http20_proxy_flag`

It also removed several older `Site` VNet-related parameters. Do not trust pre-`9.0.0` blog posts for current model fields.

### `8.0.0`

PyPI release history for `8.0.0` also warns that older API-version subfolders were removed for package-size reduction. That matters if you are copying imports from older automation code.

## Common Pitfalls

- Installing `azure-mgmt-web` without `azure-identity`
- Using this package for app-code deployment instead of resource management
- Forgetting `reserved=True` when creating a Linux App Service plan
- Hard-coding `server_farm_id` strings instead of reading the plan resource ID returned by Azure
- Treating `update_application_settings()` as a patch instead of a replacement
- Sending back whole fetched models with server-populated fields when a narrower update call exists
- Forgetting `.result()` on `begin_create_or_update(...)` and other long-running operations
- Assuming App Service Python apps run on Windows; current Microsoft guidance says Python on App Service is Linux-only
- Setting a Python runtime in `SiteConfig.linux_fx_version` that App Service no longer supports

## Official Sources Used

- https://pypi.org/project/azure-mgmt-web/
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.websitemanagementclient?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.aio.websitemanagementclient?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.operations.appserviceplansoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.operations.webappsoperations?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.models.appserviceplan?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.models.site?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.models.siteconfig?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.models.stringdictionary?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.models.sitesourcecontrol?view=azure-python
- https://learn.microsoft.com/en-us/python/api/azure-mgmt-web/azure.mgmt.web.models.siteconfigresource?view=azure-python
- https://learn.microsoft.com/en-us/azure/app-service/configure-language-python
- https://learn.microsoft.com/en-us/azure/app-service/quickstart-python

---
name: billing
description: "Google Cloud Billing Python client for billing accounts, project billing links, and Cloud Catalog SKU metadata"
metadata:
  languages: "python"
  versions: "1.18.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud-billing,google-cloud,gcp,billing,cloud-catalog,pricing,python"
---

# google-cloud-billing Python Package Guide

## Golden Rule

Use `google-cloud-billing` for control-plane billing operations and catalog metadata, not for invoice analysis or budget management.

- Import from `google.cloud import billing_v1`.
- Use `CloudBillingClient` for billing accounts and project billing links.
- Use `CloudCatalogClient` for services, SKUs, and price metadata.
- Prefer Application Default Credentials (ADC) unless your runtime already manages credentials another way.

## Version Notes

- The version used here for this package was `1.18.0`. That matches the current PyPI package version shown on March 12, 2026.
- Google's `latest` reference pages for some `billing_v1` classes still display `1.17.0` in their headers. Treat the reference site as the authoritative API surface, but confirm your installed version when exact generated signatures matter.
- This package covers Cloud Billing account/project linkage and Cloud Catalog data. If you need budgets, BigQuery billing export analysis, or invoices, you need adjacent Google Cloud services or separate libraries.

## Installation

Install the package directly:

```bash
python -m pip install google-cloud-billing
```

Pin the package when your project already uses a lockfile or needs reproducible CI:

```bash
python -m pip install "google-cloud-billing==1.18.0"
```

With `uv`:

```bash
uv add google-cloud-billing
```

With Poetry:

```bash
poetry add google-cloud-billing
```

## Authentication And Setup

Prefer ADC for local development and deployed workloads.

Local development:

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

Service-account fallback:

```bash
export GOOGLE_APPLICATION_CREDENTIALS=/abs/path/service-account.json
```

Typical prerequisites:

1. The target Google Cloud project exists and has billing enabled where required.
2. The Cloud Billing API is enabled for the project that will call the API.
3. The caller has the right IAM permissions to view billing accounts or update project billing links.

Minimal initialization:

```python
from google.cloud import billing_v1

billing_client = billing_v1.CloudBillingClient()
catalog_client = billing_v1.CloudCatalogClient()
```

Explicit credentials:

```python
from google.cloud import billing_v1
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "/abs/path/service-account.json"
)

client = billing_v1.CloudBillingClient(credentials=credentials)
```

Async variants also exist if your app already uses async Google client patterns:

- `billing_v1.CloudBillingAsyncClient`
- `billing_v1.CloudCatalogAsyncClient`

## Core Usage

### List Billing Accounts

Use this to discover the billing account resource names your code will reuse later.

```python
from google.cloud import billing_v1

client = billing_v1.CloudBillingClient()

for account in client.list_billing_accounts():
    print(account.name, account.display_name, account.open)
```

Resource names look like `billingAccounts/000000-000000-000000`.

### Read A Project's Billing State

`get_project_billing_info()` expects a project resource name, not a bare project ID.

```python
from google.cloud import billing_v1

client = billing_v1.CloudBillingClient()

project_name = "projects/my-project-id"
info = client.get_project_billing_info(name=project_name)

print(info.name)
print(info.project_id)
print(info.billing_account_name)
print(info.billing_enabled)
```

Use this before linking or unlinking a billing account.

### Link Or Unlink A Project Billing Account

To attach a project to a billing account:

```python
from google.cloud import billing_v1

client = billing_v1.CloudBillingClient()

updated = client.update_project_billing_info(
    name="projects/my-project-id",
    project_billing_info=billing_v1.ProjectBillingInfo(
        billing_account_name="billingAccounts/000000-000000-000000",
    ),
)

print(updated.billing_account_name)
print(updated.billing_enabled)
```

To disable billing for a project, send an empty billing account name:

```python
from google.cloud import billing_v1

client = billing_v1.CloudBillingClient()

updated = client.update_project_billing_info(
    name="projects/my-project-id",
    project_billing_info=billing_v1.ProjectBillingInfo(
        billing_account_name="",
    ),
)
```

### List Projects Linked To A Billing Account

Use this when you need inventory for one billing account.

```python
from google.cloud import billing_v1

client = billing_v1.CloudBillingClient()

billing_account_name = "billingAccounts/000000-000000-000000"

for project in client.list_project_billing_info(name=billing_account_name):
    print(project.project_id, project.billing_enabled)
```

### List Billable Services

`CloudCatalogClient` exposes published Cloud Catalog metadata.

```python
from google.cloud import billing_v1

client = billing_v1.CloudCatalogClient()

for service in client.list_services():
    print(service.name, service.display_name)
```

Service names look like `services/6F81-5844-456A`.

### List SKUs For A Service

This is the common path for pricing metadata lookup.

```python
from google.cloud import billing_v1

client = billing_v1.CloudCatalogClient()

service_name = "services/6F81-5844-456A"

for sku in client.list_skus(parent=service_name):
    print(sku.name)
    print(sku.description)
    print(sku.category.resource_family)
```

If you need the price metadata effective for a specific window, use the request object and pass `start_time` and `end_time`.

```python
from datetime import datetime, timedelta, timezone

from google.cloud import billing_v1

client = billing_v1.CloudCatalogClient()

end_time = datetime.now(timezone.utc)
start_time = end_time - timedelta(days=30)

request = billing_v1.ListSkusRequest(
    parent="services/6F81-5844-456A",
    start_time=start_time,
    end_time=end_time,
)

for sku in client.list_skus(request=request):
    print(sku.name)
```

Treat catalog data as published SKU metadata, not as live usage or invoice data.

## Request Configuration

Generated Google Cloud clients follow the standard GAPIC pattern:

- Most methods accept `retry`, `timeout`, and `metadata` keyword arguments.
- List methods return pagers, so direct iteration is the normal path.
- Resource names are strings like `projects/my-project-id`, `billingAccounts/...`, and `services/...`.

Example with an explicit timeout:

```python
from google.cloud import billing_v1

client = billing_v1.CloudBillingClient()

info = client.get_project_billing_info(
    name="projects/my-project-id",
    timeout=30,
)
```

## Configuration And Debugging Notes

- ADC is the default credential flow. Avoid hard-coding credentials into application code.
- Set `GOOGLE_APPLICATION_CREDENTIALS` only when you intentionally want file-based service-account credentials.
- The package README documents environment-based logging control through `GOOGLE_SDK_PYTHON_LOGGING_SCOPE`. Use `google` or a narrower namespace when debugging client-library RPC behavior.
- Reuse long-lived clients in app code instead of creating a new client for every request.

## Common Pitfalls

- The package installs as `google-cloud-billing`, but the import path is `google.cloud.billing_v1`.
- `CloudBillingClient` and `CloudCatalogClient` are separate surfaces. Project billing methods are not on the catalog client, and SKU methods are not on the billing client.
- `get_project_billing_info()` and `update_project_billing_info()` require `projects/PROJECT_ID`, not just `PROJECT_ID`.
- `list_project_billing_info()` takes a billing account resource like `billingAccounts/...`, not a project resource.
- Cloud Catalog results can be large. Cache service and SKU lookups if you call them repeatedly.
- Do not use catalog prices as a substitute for usage export, invoice reconciliation, or budget enforcement.
- Be careful with context-manager patterns around generated transports. Shared transports can be closed unexpectedly if you wrap clients in short-lived `with` blocks.

## Version-Sensitive Notes

- This guide targets PyPI version `1.18.0`.
- The package README and PyPI metadata are already on `1.18.0`, but some Google Cloud reference pages still render `1.17.0` in class headings as of March 12, 2026. If you hit a signature mismatch, inspect your installed package with `python -m pip show google-cloud-billing` and prefer the generated docs for your exact version when available.
- Older links may still point to `.../python/docs/reference/billing/latest`. Prefer the `cloudbilling` path for current references.

## Official Sources

- PyPI package page: `https://pypi.org/project/google-cloud-billing/`
- Package README in the official monorepo: `https://raw.githubusercontent.com/googleapis/google-cloud-python/main/packages/google-cloud-billing/README.rst`
- Repository package directory: `https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-billing`
- Cloud Billing Python reference root: `https://cloud.google.com/python/docs/reference/cloudbilling/latest`
- `CloudBillingClient` reference: `https://cloud.google.com/python/docs/reference/cloudbilling/latest/google.cloud.billing_v1.services.cloud_billing.CloudBillingClient`
- `CloudCatalogClient` reference: `https://cloud.google.com/python/docs/reference/cloudbilling/latest/google.cloud.billing_v1.services.cloud_catalog.CloudCatalogClient`
- `ListSkusRequest` reference: `https://cloud.google.com/python/docs/reference/cloudbilling/latest/google.cloud.billing_v1.types.ListSkusRequest`
- ADC setup: `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`

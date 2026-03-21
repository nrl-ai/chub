---
name: retail
description: "Google Cloud Retail Python client for catalog import, search, recommendations, user events, and autocomplete"
metadata:
  languages: "python"
  versions: "2.9.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-cloud,retail,ecommerce,search,recommendations"
---

# Google Cloud Retail Python Client

## Golden Rule

Use the official `google-cloud-retail` package with Application Default Credentials (ADC), and prefer the GA `retail_v2` namespace unless a feature is only surfaced in `retail_v2beta`.

As of March 12, 2026, PyPI lists `google-cloud-retail 2.9.0`, but Google's `latest` Python reference and changelog still show `2.7.0`. Treat the docs site as the API shape reference, but pin installs from PyPI and call out the drift in reviews.

## Install

```bash
python -m pip install "google-cloud-retail==2.9.0"
```

Common alternatives:

```bash
uv add "google-cloud-retail==2.9.0"
poetry add "google-cloud-retail==2.9.0"
```

## Authentication And Setup

This client library uses Google Cloud authentication, so the normal setup is:

1. Create or select a Google Cloud project.
2. Enable billing.
3. Enable the Retail API for that project.
4. Configure ADC for local development or attach a service account in production.

### Local development

Google's ADC guidance says the best local option is your user credentials:

```bash
gcloud auth application-default login
```

Google also documents that ADC checks credentials in this order:

1. `GOOGLE_APPLICATION_CREDENTIALS`
2. A local ADC file created by `gcloud auth application-default login`
3. The attached service account from the metadata server

### Production on Google Cloud

For production workloads running on Google Cloud, Google recommends an attached user-managed service account rather than baking keys into the app.

### Service account key fallback

Only use a service account key if you cannot use user credentials, service account impersonation, or an attached service account:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

## Import Surface

GA and generated helpers live under `retail_v2`:

```python
from google.cloud import retail_v2
```

The docs overview also exposes `retail_v2alpha` and `retail_v2beta`. In practice:

- Use `retail_v2` for product import, search, recommendations, and user events.
- Check `retail_v2beta` when you need autocomplete and the GA page for `CompletionServiceClient` is missing or stale.

## Resource Names You Will Reuse

Most requests fail because of bad resource names rather than bad Python syntax. Build them explicitly:

```python
from google.cloud import retail_v2

project_id = "my-project"
location = "global"
catalog = "default_catalog"
branch = "default_branch"
serving_config = "default_serving_config"

search_client = retail_v2.SearchServiceClient()

branch_name = search_client.branch_path(project_id, location, catalog, branch)
serving_config_name = search_client.serving_config_path(
    project_id,
    location,
    catalog,
    serving_config,
)
catalog_name = f"projects/{project_id}/locations/{location}/catalogs/{catalog}"
```

Useful formats from the official docs:

- Search placement: `projects/*/locations/global/catalogs/default_catalog/servingConfigs/default_serving_config`
- Legacy search placement: `projects/*/locations/global/catalogs/default_catalog/placements/default_search`
- Branch: `projects/*/locations/global/catalogs/default_catalog/branches/default_branch`
- Catalog: `projects/*/locations/global/catalogs/default_catalog`

## Search Products

`SearchRequest` requires a `placement`. `visitor_id` is effectively mandatory for real applications if you want search quality and later event correlation.

```python
from google.cloud import retail_v2

project_id = "my-project"
location = "global"
catalog = "default_catalog"
branch = "default_branch"
serving_config = "default_serving_config"

client = retail_v2.SearchServiceClient()

request = retail_v2.SearchRequest(
    placement=client.serving_config_path(
        project_id,
        location,
        catalog,
        serving_config,
    ),
    branch=client.branch_path(project_id, location, catalog, branch),
    query="running shoes",
    visitor_id="anon-session-7d6b4d3c",
)

results = client.search(request=request)

for result in results:
    product = result.product
    print(product.id, product.title)
```

Version-sensitive behavior from the reference:

- If `query` is empty, Retail treats the call as category browsing.
- `search()` returns a pager and automatically resolves additional pages while you iterate.
- `placements/*` still works, but `servingConfigs/*` is the recommended resource.

## Import Or Update Products

`import_products()` is the normal bulk-ingestion API. The request parent is the branch resource, and the operation can create missing products.

```python
from google.cloud import retail_v2

project_id = "my-project"
location = "global"
catalog = "default_catalog"
branch = "default_branch"

client = retail_v2.ProductServiceClient()
parent = client.branch_path(project_id, location, catalog, branch)

products = [
    retail_v2.Product(
        id="sku-123",
        title="Trail Runner 2",
        categories=["Shoes", "Running"],
    ),
    retail_v2.Product(
        id="sku-456",
        title="Daily Trainer",
        categories=["Shoes", "Training"],
    ),
]

input_config = retail_v2.ProductInputConfig(
    product_inline_source=retail_v2.ProductInlineSource(products=products)
)

operation = client.import_products(
    request=retail_v2.ImportProductsRequest(
        parent=parent,
        input_config=input_config,
    )
)

response = operation.result()
print(response)
```

Important request behavior from the official docs:

- `parent` must look like `projects/.../locations/global/catalogs/default_catalog/branches/default_branch`
- `reconciliation_mode` defaults to incremental
- If you set `update_mask`, missing products are not created
- `notification_pubsub_topic` is only supported with `FULL` reconciliation

## Get Recommendations

Recommendations are served through `PredictionServiceClient.predict()`. The official `PredictRequest` docs say `placement` can be either a `servingConfigs/*` resource or the legacy `placements/*` form, with `servingConfigs/*` preferred.

```python
from google.cloud import retail_v2

project_id = "my-project"
location = "global"
catalog = "default_catalog"

client = retail_v2.PredictionServiceClient()

request = retail_v2.PredictRequest(
    placement=(
        f"projects/{project_id}/locations/{location}/"
        f"catalogs/{catalog}/servingConfigs/default_recommendation_config"
    ),
    user_event=retail_v2.UserEvent(
        event_type="detail-page-view",
        visitor_id="anon-session-7d6b4d3c",
    ),
    page_size=10,
)

response = client.predict(request=request)

for prediction in response.results:
    print(prediction.id)
```

The request docs also warn that the inline `user_event` in `PredictRequest` is not written to Retail's event logs. If you need logging and training data, send a separate `write_user_event()` call too.

## Record User Events

Use `UserEventServiceClient.write_user_event()` for real-time clickstream or behavioral signals.

```python
from google.cloud import retail_v2

project_id = "my-project"
location = "global"
catalog = "default_catalog"

client = retail_v2.UserEventServiceClient()
parent = f"projects/{project_id}/locations/{location}/catalogs/{catalog}"

response = client.write_user_event(
    request=retail_v2.WriteUserEventRequest(
        parent=parent,
        user_event=retail_v2.UserEvent(
            event_type="search",
            visitor_id="anon-session-7d6b4d3c",
            search_query="running shoes",
        ),
    )
)

print(response)
```

Practical notes from the official request docs:

- `parent` is the catalog resource, not the branch
- `user_event` is required
- `write_async=True` can return success before the event is fully written; silent failures then show up in Cloud Logging

## Autocomplete

The package overview lists completion support in the Retail family, and `CompleteQueryRequest` is documented in the reference. The directly indexed `CompletionServiceClient` page currently resolves under `retail_v2beta`, so verify the namespace in your installed version before committing autocomplete code.

```python
from google.cloud import retail_v2beta

project_id = "my-project"
location = "global"
catalog = "default_catalog"

client = retail_v2beta.CompletionServiceClient()

response = client.complete_query(
    request=retail_v2beta.CompleteQueryRequest(
        catalog=f"projects/{project_id}/locations/{location}/catalogs/{catalog}",
        query="run",
        visitor_id="anon-session-7d6b4d3c",
    )
)

print(response)
```

The documented request shape matters:

- `catalog` is required
- `query` is required and limited to 255 characters
- `visitor_id` is recommended, must be UTF-8, and has a 128-character limit
- Completion is only available when Retail Search is enabled

## Logging And Debugging

The PyPI package page documents built-in Python logging support for Google client libraries.

Environment-based logging:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

Code-based logging:

```python
import logging

base_logger = logging.getLogger("google")
base_logger.addHandler(logging.StreamHandler())
base_logger.setLevel(logging.DEBUG)
```

The package notes also say logging is not configured by default and may include sensitive information.

## Common Pitfalls

- Do not invent resource names. Use helper methods like `branch_path()` and `serving_config_path()` where available.
- Do not omit `visitor_id`. Search, recommendations, and autocomplete all rely on it for useful personalization and event attribution.
- Do not assume `PredictRequest.user_event` writes an event log entry. It does not.
- Do not mix up parent scopes:
  - product import uses a branch parent
  - user-event writes use a catalog parent
  - search uses `placement` plus an optional branch
- Do not trust the docs site version label as the package version. The reference currently says `2.7.0 (latest)` while PyPI serves `2.9.0`.
- Do not default to service account keys in production when an attached service account is available.

## Version-Sensitive Notes

- PyPI shows `google-cloud-retail 2.9.0`, released on January 29, 2026.
- The canonical Google Python reference root redirects to `https://docs.cloud.google.com/python/docs/reference/retail/latest`, but its visible `latest` label and changelog still stop at `2.7.0` as of March 12, 2026.
- The generated client reference pages used here were last updated on October 30, 2025, while the Google Cloud authentication pages were updated on March 11, 2026.
- Completion/autocomplete documentation currently appears split: the package overview exposes completion under Retail V2, while the directly indexed `CompletionServiceClient` page resolves under `retail_v2beta`.

## Official Sources

- PyPI package page: https://pypi.org/project/google-cloud-retail/
- Python reference root: https://cloud.google.com/python/docs/reference/retail/latest
- Changelog: https://cloud.google.com/python/docs/reference/retail/latest/changelog
- Search client: https://cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.services.search_service.SearchServiceClient
- Search request: https://cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.types.SearchRequest
- Product client: https://cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.services.product_service.ProductServiceClient
- Import products request: https://cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.types.ImportProductsRequest
- Prediction client: https://cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.services.prediction_service.PredictionServiceClient
- Predict request: https://docs.cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.types.PredictRequest
- User event client: https://cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.services.user_event_service.UserEventServiceClient
- Write user event request: https://docs.cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.types.WriteUserEventRequest
- Complete query request: https://cloud.google.com/python/docs/reference/retail/latest/google.cloud.retail_v2.types.CompleteQueryRequest
- ADC overview: https://cloud.google.com/docs/authentication/provide-credentials-adc
- ADC local development: https://docs.cloud.google.com/docs/authentication/set-up-adc-local-dev-environment
- ADC search order: https://cloud.google.com/docs/authentication/application-default-credentials

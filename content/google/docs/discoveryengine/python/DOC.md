---
name: discoveryengine
description: "Google Cloud Discovery Engine Python client for search, answer generation, and document operations"
metadata:
  languages: "python"
  versions: "0.17.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,discoveryengine,vertex-ai-search,agent-builder,search,rag,recommendations"
---

# Google Cloud Discovery Engine Python Client

## Install And Prerequisites

Google's own package quickstart still expects the standard Google Cloud setup before client code will work:

1. Select or create a Google Cloud project.
2. Enable billing for that project.
3. Enable the Discovery Engine API.
4. Set up authentication.

Then install the client in an isolated environment:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install "google-cloud-discoveryengine==0.17.0"
```

Common alternatives:

```bash
uv add "google-cloud-discoveryengine==0.17.0"
poetry add "google-cloud-discoveryengine==0.17.0"
```

## Authentication And Endpoint Setup

Google Cloud client libraries use Application Default Credentials (ADC). For local development, Google recommends setting up ADC with your user credentials or service account impersonation via the `gcloud` CLI. For production, attach a service account to the runtime instead of baking JSON keys into the app.

Typical local setup:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="my-project"
export GOOGLE_CLOUD_LOCATION="global"
export DISCOVERY_ENGINE_DATA_STORE_ID="my-data-store"
```

Use a regional endpoint when your resources are not in `global`. The generated Discovery Engine client snippets explicitly warn that you may need to specify regional endpoints when creating the service client.

```python
import os

from google.api_core.client_options import ClientOptions
from google.cloud import discoveryengine_v1 as discoveryengine

PROJECT_ID = os.environ["GOOGLE_CLOUD_PROJECT"]
LOCATION = os.getenv("GOOGLE_CLOUD_LOCATION", "global")
DATA_STORE_ID = os.environ["DISCOVERY_ENGINE_DATA_STORE_ID"]

client_options = None
if LOCATION != "global":
    client_options = ClientOptions(
        api_endpoint=f"{LOCATION}-discoveryengine.googleapis.com"
    )

search_client = discoveryengine.SearchServiceClient(client_options=client_options)
document_client = discoveryengine.DocumentServiceClient(client_options=client_options)
answer_client = discoveryengine.ConversationalSearchServiceClient(
    client_options=client_options
)

serving_config = search_client.serving_config_path(
    project=PROJECT_ID,
    location=LOCATION,
    data_store=DATA_STORE_ID,
    serving_config="default_serving_config",
)

branch = document_client.branch_path(
    project=PROJECT_ID,
    location=LOCATION,
    data_store=DATA_STORE_ID,
    branch="default_branch",
)
```

## Core Search

`SearchServiceClient.search()` is the default entry point when you need ranked search results from an indexed data store.

```python
from google.cloud import discoveryengine_v1 as discoveryengine

request = discoveryengine.SearchRequest(
    serving_config=serving_config,
    query="refund policy",
    page_size=5,
)

pager = search_client.search(request=request)

for result in pager:
    document = result.document
    title = document.derived_struct_data.get("title")
    print(document.id, title or document.name)
```

Useful notes from the reference docs:

- `SearchRequest.serving_config` accepts either an engine serving config or a data store serving config.
- Leaving `branch` empty searches the default branch; otherwise use a branch resource name such as `default_branch`.
- `page_size` limits depend on the indexed data type, so do not assume every corpus supports large pages.
- The returned pager handles additional pages for you when you iterate it.

### API key-only search

`SearchServiceClient.search_lite()` exists, but the reference docs restrict it to public website search and explicitly recommend normal `search()` with OAuth/IAM checks for better security. Treat `search_lite()` as a narrow onboarding path, not the default for private or enterprise content.

## Answer Generation

Use `ConversationalSearchServiceClient.answer_query()` when you need synthesized answers over retrieved content instead of raw ranked results.

```python
from google.cloud import discoveryengine_v1 as discoveryengine

session = answer_client.session_path(
    project=PROJECT_ID,
    location=LOCATION,
    data_store=DATA_STORE_ID,
    session="-",
)

request = discoveryengine.AnswerQueryRequest(
    serving_config=serving_config,
    query=discoveryengine.Query(text="Summarize the return policy."),
    session=session,
    user_pseudo_id="user-123",
    answer_generation_spec=discoveryengine.AnswerQueryRequest.AnswerGenerationSpec(
        include_citations=True,
        ignore_adversarial_query=True,
        ignore_non_answer_seeking_query=True,
        model_spec=discoveryengine.AnswerQueryRequest.AnswerGenerationSpec.ModelSpec(
            model_version="stable"
        ),
    ),
)

response = answer_client.answer_query(request=request)
print(response.answer.answer_text)
```

Important behavior from the request docs:

- `session` is optional. If you want auto-session mode, Google documents `-` as a wildcard session ID and the service will generate a session for you.
- `stream_answer_query()` is available if you need streamed answer output.
- `user_pseudo_id` should be stable per visitor/device and should not be a fixed placeholder like `unknown_visitor`.

## Inspect Documents

When search results are unexpectedly empty, inspect the indexed content before debugging query logic.

```python
pager = document_client.list_documents(request={"parent": branch})

for document in pager:
    print(document.name)
```

Useful helpers from `DocumentServiceClient`:

- `branch_path(project, location, data_store, branch)`
- `document_path(project, location, data_store, branch, document)`

These helpers are safer than hand-assembling resource names when you are writing validation, ingestion, or cleanup tools.

## Configuration And Operational Notes

- Prefer ADC over manually loading credentials into every client constructor. The auth docs say client libraries automatically check ADC and use those credentials.
- In production, attach a service account to the workload instead of distributing long-lived JSON keys.
- Keep `project`, `location`, `data_store`, `branch`, and `serving_config` in environment or typed config. Most `INVALID_ARGUMENT` and `NOT_FOUND` failures come from a wrong resource path, not a broken query.
- If you need request-level logging while debugging, PyPI documents the `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` environment variable. `export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google` enables default debug logging for Google client libraries.

## Common Pitfalls

- Hosted docs are currently inconsistent across pages. Do not assume every `latest` page reflects the same package build.
- Set `client_options.api_endpoint` for non-`global` locations. The generated client snippets repeat this warning across search, document, and conversational clients.
- Search and answer generation are separate APIs. `search()` returns ranked results; `answer_query()` generates synthesized responses over retrieved content.
- Be consistent about resource naming. Request types accept both engine-based and data-store-based serving configs, while many helper methods generate data-store-based paths.
- Older examples may use preview modules or older product branding such as Generative AI App Builder. Prefer `discoveryengine_v1` and current resource names unless you have a documented reason to use preview APIs.
- `search_lite()` supports API keys only for public website search. It is not a general replacement for OAuth/IAM-backed search.

## Version-Sensitive Notes

- PyPI currently lists `google-cloud-discoveryengine 0.17.0`, released on February 12, 2026.
- The official reference changelog page still reports `0.16.0 (latest)` and was last updated on December 18, 2025.
- The hosted reference is in a mixed rollout state:
  - `SearchServiceClient` renders as `0.17.0`.
  - `ConversationalSearchServiceClient` and `SearchRequest` render as `0.16.0`.
  - `DocumentServiceClient` still renders as `0.15.0`.
- If you are using a very new symbol from the installed package, confirm it exists locally with Python introspection or your editor's type stubs instead of trusting a single hosted page.

## Official Sources

- Python reference root: `https://cloud.google.com/python/docs/reference/discoveryengine/latest`
- Search client reference: `https://cloud.google.com/python/docs/reference/discoveryengine/latest/google.cloud.discoveryengine_v1.services.search_service.SearchServiceClient`
- Search request reference: `https://cloud.google.com/python/docs/reference/discoveryengine/latest/google.cloud.discoveryengine_v1.types.SearchRequest`
- Conversational search client reference: `https://cloud.google.com/python/docs/reference/discoveryengine/latest/google.cloud.discoveryengine_v1.services.conversational_search_service.ConversationalSearchServiceClient`
- Answer query request reference: `https://cloud.google.com/python/docs/reference/discoveryengine/latest/google.cloud.discoveryengine_v1.types.AnswerQueryRequest`
- Document service reference: `https://cloud.google.com/python/docs/reference/discoveryengine/latest/google.cloud.discoveryengine_v1.services.document_service.DocumentServiceClient`
- Discovery Engine changelog: `https://cloud.google.com/python/docs/reference/discoveryengine/latest/changelog`
- Google Cloud authentication for client libraries: `https://cloud.google.com/docs/authentication/client-libraries`
- PyPI package page: `https://pypi.org/project/google-cloud-discoveryengine/`

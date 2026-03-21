---
name: dialogflow
description: "Google Cloud Dialogflow ES Python client library for sessions, intents, contexts, and agent management"
metadata:
  languages: "python"
  versions: "2.46.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,dialogflow,dialogflow-es,gcp,chatbot,conversation,nlp"
---

# Google Cloud Dialogflow Python Client

## Golden Rule

Use `google-cloud-dialogflow` for Dialogflow ES work in Python, and treat the official Google Cloud reference as the source of truth for client classes and request shapes.

- Use Application Default Credentials (ADC) instead of hardcoding credentials in code.
- Do not mix Dialogflow CX examples into this package. This package documents the Dialogflow ES / `dialogflow_v2` surface.
- If your agent is regional, set both the API endpoint and the session resource path to the same location.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-cloud-dialogflow==2.46.0"
```

Common alternatives:

```bash
uv add "google-cloud-dialogflow==2.46.0"
poetry add "google-cloud-dialogflow==2.46.0"
```

## Authentication And Setup

For local development, prefer ADC with the Google Cloud CLI:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="your-gcp-project-id"
export DIALOGFLOW_LOCATION="global"
```

If you must use a service account key file, point ADC at it:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GOOGLE_CLOUD_PROJECT="your-gcp-project-id"
```

Practical notes:

- The Google Cloud project must have Dialogflow enabled and the caller must have Dialogflow permissions.
- For regional agents, set `DIALOGFLOW_LOCATION` to the region where the agent lives, for example `us-central1`.
- Prefer attached service accounts or impersonation over long-lived JSON key files when you control the runtime environment.

## Initialize A Sessions Client

Use the default global endpoint unless your agent is regional:

```python
import os
from typing import Sequence

from google.api_core.client_options import ClientOptions
from google.cloud import dialogflow

PROJECT_ID = os.environ["GOOGLE_CLOUD_PROJECT"]
LOCATION = os.getenv("DIALOGFLOW_LOCATION", "global")

def make_sessions_client(location: str) -> dialogflow.SessionsClient:
    if location == "global":
        return dialogflow.SessionsClient()

    return dialogflow.SessionsClient(
        client_options=ClientOptions(
            api_endpoint=f"{location}-dialogflow.googleapis.com:443"
        )
    )

def build_session_name(
    client: dialogflow.SessionsClient, project_id: str, session_id: str, location: str
) -> str:
    if location == "global":
        return client.session_path(project_id, session_id)

    return f"projects/{project_id}/locations/{location}/agent/sessions/{session_id}"
```

If ADC is not available in the current process, you can initialize directly from a service account file:

```python
from google.cloud import dialogflow

client = dialogflow.SessionsClient.from_service_account_file(
    "/path/to/service-account.json"
)
```

## Core Usage: Detect Intent From Text

This is the core request flow from the official text detect-intent sample, with regional endpoint handling added:

```python
import os

from google.api_core.client_options import ClientOptions
from google.cloud import dialogflow

def detect_intent_texts(
    project_id: str,
    session_id: str,
    texts: Sequence[str],
    language_code: str = "en-US",
    location: str = "global",
) -> None:
    client_options = None
    if location != "global":
        client_options = ClientOptions(
            api_endpoint=f"{location}-dialogflow.googleapis.com:443"
        )

    session_client = dialogflow.SessionsClient(client_options=client_options)

    if location == "global":
        session = session_client.session_path(project_id, session_id)
    else:
        session = (
            f"projects/{project_id}/locations/{location}/agent/sessions/{session_id}"
        )

    for text in texts:
        text_input = dialogflow.TextInput(text=text, language_code=language_code)
        query_input = dialogflow.QueryInput(text=text_input)

        response = session_client.detect_intent(
            request={"session": session, "query_input": query_input}
        )

        result = response.query_result
        matched_intent = result.intent.display_name if result.intent else "<no match>"

        print(f"user: {text}")
        print(f"intent: {matched_intent}")
        print(f"confidence: {result.intent_detection_confidence:.3f}")
        print(f"reply: {result.fulfillment_text}")
        print("---")

detect_intent_texts(
    project_id=os.environ["GOOGLE_CLOUD_PROJECT"],
    session_id="web-demo-session",
    texts=["hello", "book a table for two tonight"],
    language_code="en-US",
    location=os.getenv("DIALOGFLOW_LOCATION", "global"),
)
```

Use one stable `session_id` per end-user conversation so Dialogflow can reuse context across turns.

## Other Common Operations

Reach for these clients for the next layer of work:

- `IntentsClient`: create, update, list, and delete intents
- `ContextsClient`: inspect or clear active contexts for a session
- `EntityTypesClient`: manage custom entities and synonyms
- `AgentsClient`: inspect agent settings and supported languages
- `KnowledgeBasesClient` and `DocumentsClient`: manage knowledge connectors

The package reference root lists these services and their request/response types under `google.cloud.dialogflow_v2`.

## Configuration Notes

### Regional agents

Regional Dialogflow ES agents require both of these changes:

- API endpoint: `"{location}-dialogflow.googleapis.com:443"`
- Session resource: `projects/{project}/locations/{location}/agent/sessions/{session}`

Changing only one of them is a common cause of `NOT_FOUND` and routing errors.

### Logging

The library supports opt-in standard logging. Set a logging scope before import or process startup:

```bash
export GOOGLE_SDK_PYTHON_LOGGING_SCOPE=google
```

Treat client logs as sensitive. Request metadata and payload details may not be appropriate for production logs.

## Common Pitfalls

- Dialogflow ES vs CX: this package is for Dialogflow ES. CX examples and resource names do not map cleanly here.
- Global vs regional mismatch: if the agent is regional, do not keep using the default global endpoint.
- Post-fork gRPC clients: create client instances after `os.fork()` or worker process start. Do not share a pre-fork client across processes.
- Session handling: reuse a session ID for one conversation, but do not share one session ID across unrelated users.
- Empty `fulfillment_text`: some agents rely more on rich response messages, output contexts, or intent parameters than on a single text reply.
- Explicit close behavior: using the client as a context manager closes the underlying transport. Do not do that if the transport is shared elsewhere in the process.

## Version-Sensitive Notes

- PyPI lists `google-cloud-dialogflow 2.46.0`, which is the version tracked in this doc.
- As of `2026-03-12`, Google Cloud reference pages under the `latest` docs tree still show `2.45.0` in page titles and the published changelog.
- Use PyPI as the authoritative package-release source for pinning, and use the Google Cloud `latest` reference tree for current API shapes until Google publishes a clearly version-aligned `2.46.0` reference set.

## Official Sources

- `https://cloud.google.com/python/docs/reference/dialogflow/latest`
- `https://cloud.google.com/python/docs/reference/dialogflow/latest/google.cloud.dialogflow_v2.services.sessions.SessionsClient`
- `https://cloud.google.com/python/docs/reference/dialogflow/latest/changelog`
- `https://cloud.google.com/dialogflow/es/docs/how/detect-intent-text`
- `https://cloud.google.com/dialogflow/es/docs/how/region`
- `https://cloud.google.com/docs/authentication/set-up-adc-local-dev-environment`
- `https://pypi.org/project/google-cloud-dialogflow/`

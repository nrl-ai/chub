---
name: auth
description: "Google authentication library for Python covering ADC, service accounts, workload identity federation, impersonation, ID tokens, and authenticated transports"
metadata:
  languages: "python"
  versions: "2.49.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,google-auth,authentication,oauth2,adc,service-account,workload-identity,id-token"
---

# Google Auth Python Library

## Golden Rule

Use `google-auth` for Google credential objects and authenticated transports in Python. Prefer Application Default Credentials (ADC) first, prefer short-lived credentials over private key files, and use `google-auth-oauthlib` for browser-based OAuth consent flows because `google-auth` does not perform that login flow by itself.

## Install

Pin the package version your project expects:

```bash
python -m pip install "google-auth==2.49.0"
```

Common alternatives:

```bash
uv add "google-auth==2.49.0"
poetry add "google-auth==2.49.0"
```

If you are writing higher-level Google Cloud client code, you usually also install the specific client library you need, such as `google-cloud-storage` or `google-cloud-bigquery`. `google-auth` is the shared authentication layer underneath those libraries.

## Authentication And Setup

### Use ADC for the common case

`google.auth.default()` is the standard entry point. In practice it commonly resolves credentials from:

1. `GOOGLE_APPLICATION_CREDENTIALS`
2. Local Application Default Credentials created by `gcloud auth application-default login`
3. An attached service account on Google Cloud runtimes
4. External account configuration for workload identity federation

For direct HTTP calls, pair ADC with `AuthorizedSession`:

```python
import google.auth
from google.auth.transport.requests import AuthorizedSession

SCOPES = ["https://www.googleapis.com/auth/cloud-platform"]

credentials, project_id = google.auth.default(scopes=SCOPES)
session = AuthorizedSession(credentials)

response = session.get(
    "https://storage.googleapis.com/storage/v1/b",
    params={"project": project_id},
    timeout=30,
)
response.raise_for_status()

for bucket in response.json().get("items", []):
    print(bucket["name"])
```

Notes:

- `project_id` may be `None`; do not assume ADC always discovers one.
- Pass scopes explicitly when you are using raw transports instead of a higher-level client library.
- `AuthorizedSession` refreshes access tokens automatically, but you still need to set request timeouts yourself.

### Refresh a token explicitly

Refresh manually when you need the current bearer token value:

```python
import google.auth
from google.auth.transport.requests import Request

credentials, _ = google.auth.default(
    scopes=["https://www.googleapis.com/auth/cloud-platform"]
)
credentials.refresh(Request())

print(credentials.token)
```

## Core Credential Flows

### Service account credentials

Use a service account key file only when ADC, workload identity federation, or impersonation is not viable.

```python
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "service-account.json",
    scopes=["https://www.googleapis.com/auth/cloud-platform"],
)

credentials = credentials.with_quota_project("billing-project-id")
```

For Google Workspace domain-wide delegation:

```python
delegated = credentials.with_subject("user@example.com")
```

Important points:

- Keep private key files out of source control.
- Prefer deriving copies with `with_scopes(...)`, `with_quota_project(...)`, and `with_subject(...)` instead of mutating shared state.
- Domain-wide delegation only works when the Workspace admin has explicitly authorized the service account.

### Authorized user credentials

If an OAuth flow has already produced an authorized-user JSON file, load it with `google.oauth2.credentials`:

```python
from google.auth.transport.requests import AuthorizedSession
from google.oauth2.credentials import Credentials

credentials = Credentials.from_authorized_user_file(
    "authorized-user.json",
    scopes=["openid", "https://www.googleapis.com/auth/userinfo.email"],
)

session = AuthorizedSession(credentials)
response = session.get(
    "https://openidconnect.googleapis.com/v1/userinfo",
    timeout=30,
)
response.raise_for_status()
print(response.json()["email"])
```

`google-auth` can use these credentials, but it does not run the browser login flow that creates them.

### Workload identity federation and external account configs

For AWS, Azure, or OIDC federation into Google Cloud, load the external account config instead of distributing service account keys:

```python
import google.auth

credentials, project_id = google.auth.load_credentials_from_file(
    "wif-config.json",
    scopes=["https://www.googleapis.com/auth/cloud-platform"],
)
```

The official docs call the generic credential-loading helpers security-sensitive. Validate any externally supplied credential JSON before using it, and prefer config files generated by Google Cloud tooling rather than user-uploaded files.

### Service account impersonation

Use impersonation when one trusted credential should mint short-lived credentials for another service account:

```python
import google.auth
from google.auth import impersonated_credentials

source_credentials, _ = google.auth.default(
    scopes=["https://www.googleapis.com/auth/cloud-platform"]
)

target_credentials = impersonated_credentials.Credentials(
    source_credentials=source_credentials,
    target_principal="sa-name@project-id.iam.gserviceaccount.com",
    target_scopes=["https://www.googleapis.com/auth/cloud-platform"],
    lifetime=300,
)
```

This is usually safer than distributing long-lived key files.

### Fetch ID tokens for Cloud Run or IAP

Use ID tokens, not OAuth access tokens, when the target expects an audience-bound identity token:

```python
from google.auth.transport.requests import Request
from google.oauth2 import id_token

audience = "https://my-service-abc-uc.a.run.app"
token = id_token.fetch_id_token(Request(), audience)
print(token)
```

Rule of thumb:

- Google APIs usually expect OAuth access tokens.
- Cloud Run, IAP, and similar identity-aware endpoints often expect ID tokens with the correct audience.

## Direct HTTP Usage

`AuthorizedSession` is the practical default when you need raw HTTP instead of a higher-level Google client library:

```python
from google.auth.transport.requests import AuthorizedSession
from google.oauth2 import service_account

credentials = service_account.Credentials.from_service_account_file(
    "service-account.json",
    scopes=["https://www.googleapis.com/auth/devstorage.read_only"],
)

session = AuthorizedSession(credentials)
response = session.get(
    "https://storage.googleapis.com/storage/v1/b/my-bucket/o",
    timeout=30,
)
response.raise_for_status()
print(response.json())
```

Other transports exist, including `google.auth.transport.urllib3` and gRPC helpers, but `requests`-based `AuthorizedSession` is usually the fastest path for agent-written code.

## Configuration Notes

- `GOOGLE_APPLICATION_CREDENTIALS` points ADC at a credential file when you need to override the default environment.
- `scopes` matter for raw REST usage and some service account flows; many higher-level Google client libraries set defaults for you.
- `quota_project_id` is important when billing or quota should be charged to a different project than the resource project.
- Metadata server availability affects ADC behavior on GCE, GKE, and Cloud Run.
- Always pass `timeout=` on HTTP requests; auth helpers do not define your network timeout policy.

## Common Pitfalls

### Package name and import names differ

Install `google-auth`, but import from `google.auth` and `google.oauth2`.

### `google-auth` is not the browser OAuth flow

If your code needs to open a browser, redirect a user, or complete an OAuth consent screen, use `google-auth-oauthlib` together with `google-auth`.

### ADC can resolve an unexpected identity

`google.auth.default()` may choose local CLI credentials, an attached runtime service account, or a workload identity config depending on the environment. If the wrong account is being used, inspect the resolved credential source before debugging the API call itself.

### `project_id` may be missing

Authentication can succeed even when no project ID is returned. Pass project IDs explicitly when the downstream API requires them.

### Access tokens and ID tokens are not interchangeable

If you get audience or issuer errors, verify whether the target service expects an ID token instead of a bearer access token.

### Credential files are sensitive configuration

The official docs explicitly warn against blindly trusting externally supplied credential configuration files. Do not accept uploaded JSON credentials across a trust boundary without validation.

## Version-Sensitive Notes

### Hosted documentation lags the package release

As of 2026-03-12, PyPI has `2.49.0`, while the official `googleapis.dev` API docs still render as `2.47.0` and the Read the Docs user guide renders as `2.38.0`. If a symbol or parameter seems missing from the hosted docs, check the installed package in your environment before assuming the API is unavailable.

### Python 3.8 is the minimum supported version on PyPI

If you are pinned to Python `3.7`, you need an older `google-auth` release line.

## Official Sources

- PyPI: `https://pypi.org/project/google-auth/`
- API docs root: `https://googleapis.dev/python/google-auth/latest/`
- User guide: `https://google-auth.readthedocs.io/en/latest/user-guide.html`
- Core API reference: `https://googleapis.dev/python/google-auth/latest/reference/google.auth.html`
- Requests transport reference: `https://googleapis.dev/python/google-auth/latest/reference/google.auth.transport.requests.html`
- Service account reference: `https://googleapis.dev/python/google-auth/latest/reference/google.oauth2.service_account.html`
- Impersonation reference: `https://googleapis.dev/python/google-auth/latest/reference/google.auth.impersonated_credentials.html`
- ID token reference: `https://googleapis.dev/python/google-auth/latest/reference/google.oauth2.id_token.html`
- Repository: `https://github.com/googleapis/google-auth-library-python`

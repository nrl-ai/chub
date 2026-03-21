---
name: iam
description: "Google Cloud IAM Python client library for service account admin, short-lived credentials, and deny policies"
metadata:
  languages: "python"
  versions: "2.21.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google,iam,python,service-accounts,credentials,deny-policies"
---

# Google Cloud IAM Python Client Library

## Golden Rule

Use `google-cloud-iam` for Google Cloud IAM Admin, IAM Credentials, and IAM v2 deny-policy operations, but do not treat it as the entire Google Cloud auth stack. In practice this package exposes three main client surfaces:

- `google.cloud.iam_admin_v1` for service accounts, keys, and custom roles
- `google.cloud.iam_credentials_v1` for short-lived access tokens, ID tokens, and signing
- `google.cloud.iam_v2` for deny policies

For authentication, prefer Application Default Credentials (ADC). For credential minting and request signing, prefer `IAMCredentialsClient` over the deprecated signing methods on `IAMClient`.

## Install

```bash
python -m pip install --upgrade google-cloud-iam
```

Common alternatives:

```bash
uv add google-cloud-iam
poetry add google-cloud-iam
```

## Authentication And Setup

Google Cloud client libraries use Application Default Credentials. For local development:

```bash
gcloud auth application-default login
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

For non-interactive workloads, point ADC at a service account key file only when you actually need a key-based flow:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

Basic imports:

```python
from google.cloud import iam_admin_v1
from google.cloud import iam_credentials_v1
from google.cloud import iam_v2
```

Basic client initialization:

```python
from google.cloud import iam_admin_v1

client = iam_admin_v1.IAMClient()
```

You can also use the generated helper constructors if you need to pin a credential file directly:

```python
from google.cloud import iam_credentials_v1

client = iam_credentials_v1.IAMCredentialsClient.from_service_account_json(
    "/path/to/service-account.json"
)
```

## Core Usage

### Manage service accounts with `IAMClient`

Create a service account:

```python
from google.cloud import iam_admin_v1

project_id = "your-project-id"
client = iam_admin_v1.IAMClient()

request = iam_admin_v1.CreateServiceAccountRequest(
    name=f"projects/{project_id}",
    account_id="automation-runner",
    service_account=iam_admin_v1.ServiceAccount(
        display_name="Automation runner",
        description="Service account used by deployment jobs",
    ),
)

service_account = client.create_service_account(request=request)
print(service_account.email)
```

List service accounts in a project:

```python
from google.cloud import iam_admin_v1

project_id = "your-project-id"
client = iam_admin_v1.IAMClient()

pager = client.list_service_accounts(
    request=iam_admin_v1.ListServiceAccountsRequest(
        name=f"projects/{project_id}",
    )
)

for service_account in pager:
    print(service_account.name, service_account.email)
```

Patch only selected fields with an update mask:

```python
from google.cloud import iam_admin_v1
from google.protobuf.field_mask_pb2 import FieldMask

client = iam_admin_v1.IAMClient()

service_account = iam_admin_v1.ServiceAccount(
    name="projects/your-project-id/serviceAccounts/automation-runner@your-project-id.iam.gserviceaccount.com",
    display_name="Automation runner v2",
)

updated = client.patch_service_account(
    request=iam_admin_v1.PatchServiceAccountRequest(
        service_account=service_account,
        update_mask=FieldMask(paths=["display_name"]),
    )
)

print(updated.display_name)
```

`create_service_account_key` still exists, but treat long-lived key creation as an exception. Prefer short-lived tokens or signing via `IAMCredentialsClient` when you can.

### Generate short-lived credentials with `IAMCredentialsClient`

Generate an OAuth access token for a target service account:

```python
from google.cloud import iam_credentials_v1
from google.protobuf.duration_pb2 import Duration

service_account_email = "target-sa@your-project-id.iam.gserviceaccount.com"
client = iam_credentials_v1.IAMCredentialsClient()

request = iam_credentials_v1.GenerateAccessTokenRequest(
    name=f"projects/-/serviceAccounts/{service_account_email}",
    scope=["https://www.googleapis.com/auth/cloud-platform"],
    lifetime=Duration(seconds=900),
)

response = client.generate_access_token(request=request)
print(response.access_token)
print(response.expire_time)
```

Generate an ID token for Cloud Run or another audience-bound service:

```python
from google.cloud import iam_credentials_v1

service_account_email = "target-sa@your-project-id.iam.gserviceaccount.com"
client = iam_credentials_v1.IAMCredentialsClient()

request = iam_credentials_v1.GenerateIdTokenRequest(
    name=f"projects/-/serviceAccounts/{service_account_email}",
    audience="https://your-service-xyz.a.run.app",
    include_email=True,
)

response = client.generate_id_token(request=request)
print(response.token)
```

Sign a JWT payload with the target service account:

```python
import json
from google.cloud import iam_credentials_v1

service_account_email = "target-sa@your-project-id.iam.gserviceaccount.com"
client = iam_credentials_v1.IAMCredentialsClient()

payload = json.dumps(
    {
        "iss": service_account_email,
        "sub": service_account_email,
        "aud": "https://example.com",
    }
)

response = client.sign_jwt(
    request=iam_credentials_v1.SignJwtRequest(
        name=f"projects/-/serviceAccounts/{service_account_email}",
        payload=payload,
    )
)

print(response.signed_jwt)
```

For impersonation flows, the caller needs `roles/iam.serviceAccountTokenCreator` on the target service account. The direct credential-creation guide also documents that access tokens default to one hour and can be extended to twelve hours only when the org policy for lifetime extension allows it.

### Manage deny policies with `PoliciesClient`

Create a deny policy attached to a project:

```python
from urllib.parse import quote

from google.cloud import iam_v2
from google.cloud.iam_v2 import types

project_id = "your-project-id"
client = iam_v2.PoliciesClient()

attachment_point = quote(
    f"cloudresourcemanager.googleapis.com/projects/{project_id}",
    safe="",
)

request = types.CreatePolicyRequest(
    parent=f"policies/{attachment_point}/denypolicies",
    policy=types.Policy(
        display_name="Restrict project deletion",
        rules=[
            types.PolicyRule(
                deny_rule=types.DenyRule(
                    denied_principals=["principalSet://goog/public:all"],
                    denied_permissions=[
                        "cloudresourcemanager.googleapis.com/projects.delete"
                    ],
                    exception_principals=[
                        "principal://iam.googleapis.com/projects/-/serviceAccounts/admin@your-project-id.iam.gserviceaccount.com"
                    ],
                )
            )
        ],
    ),
    policy_id="deny-project-delete",
)

operation = client.create_policy(request=request)
policy = operation.result()
print(policy.name)
```

List attached deny policies:

```python
from urllib.parse import quote

from google.cloud import iam_v2

project_id = "your-project-id"
client = iam_v2.PoliciesClient()

attachment_point = quote(
    f"cloudresourcemanager.googleapis.com/projects/{project_id}",
    safe="",
)

for policy in client.list_policies(
    request={"parent": f"policies/{attachment_point}/denypolicies"}
):
    print(policy.name, policy.display_name)
```

`list_policies` is for discovery. Fetch the specific policy with `get_policy` before assuming you have the full rule contents.

## Configuration Notes

- The generated clients accept `client_options`, including `api_endpoint`, and document the standard Google mTLS environment variables `GOOGLE_API_USE_MTLS_ENDPOINT` and `GOOGLE_API_USE_CLIENT_CERTIFICATE`.
- The docs also mention `universe_domain` support on client construction. Most projects should leave this alone unless they already run in a non-default Google Cloud universe.
- The PyPI page documents standard `GOOGLE_SDK_PYTHON_LOGGING_SCOPE` support for Google client library logging if you need request-level diagnostics.

## Common Pitfalls

- Do not use `IAMClient.sign_blob` or `IAMClient.sign_jwt` for new code. The generated docs mark them deprecated and point you to `IAMCredentialsClient`.
- `IAMCredentialsClient` resource names commonly use the wildcard form `projects/-/serviceAccounts/<email-or-unique-id>`. Do not assume a concrete project segment is interchangeable in every flow.
- `generate_access_token` requires at least one OAuth scope. Missing scopes is a common source of confusing permission errors.
- Do not create service account keys unless a downstream system truly requires a private key file. Short-lived access tokens, ID tokens, or signing APIs are usually safer.
- Deny policy attachment points must be URL-encoded under `policies/{attachment_point}/denypolicies`; passing an unescaped resource name will fail.
- The IAM deny-policy API uses long-running operations for create, update, and delete. Wait on `operation.result()` before assuming the change is active.
- If you use a client as a context manager, do not share its transport with other clients unless you intend to close that shared transport on exit.

## Version-Sensitive Notes

- PyPI currently lists `google-cloud-iam 2.21.0`, which matches the version used here for this session.
- The rolling docs root is correct for current reference lookup, but some generated class pages can lag slightly in the version stamp. During this session the main docs root and IAM Admin/IAM Credentials pages showed `2.21.0`, while the IAM v2 `PoliciesClient` page still displayed `2.20.0`.
- The package changelog shows API-key support plumbing added in the 2.x line, but Google Cloud IAM coding flows in the official docs still center on ADC, service accounts, and delegated IAM permissions. Treat API keys as an edge configuration detail, not the default auth path.
- Deny policies are still the IAM v2 surface in this package. Older blog posts often cover only IAM Admin or IAM Credentials and omit deny-policy attachment-point requirements.

## Official Sources

- Google Cloud Python reference: `https://cloud.google.com/python/docs/reference/iam/latest`
- IAM Admin client reference: `https://cloud.google.com/python/docs/reference/iam/latest/google.cloud.iam_admin_v1.services.iam.IAMClient`
- IAM Credentials client reference: `https://cloud.google.com/python/docs/reference/iam/latest/google.cloud.iam_credentials_v1.services.iam_credentials.IAMCredentialsClient`
- IAM v2 Policies client reference: `https://cloud.google.com/python/docs/reference/iam/latest/google.cloud.iam_v2.services.policies.PoliciesClient`
- Google Cloud client-library auth guide: `https://cloud.google.com/docs/authentication/client-libraries`
- IAM short-lived credentials guide: `https://cloud.google.com/iam/docs/create-short-lived-credentials-direct`
- IAM deny policy sample: `https://cloud.google.com/iam/docs/samples/iam-create-deny-policy`
- PyPI package page: `https://pypi.org/project/google-cloud-iam/`

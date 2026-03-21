---
name: securitycenter
description: "google-cloud-securitycenter package guide for Python covering ADC setup, v1/v2 clients, findings workflows, and endpoint/version caveats"
metadata:
  languages: "python"
  versions: "1.42.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "google-cloud,security-command-center,security-center,findings,google-cloud-securitycenter,python"
---

# google-cloud-securitycenter Python Package Guide

## Golden Rule

Use `google-cloud-securitycenter` for Security Command Center API access from Python, and authenticate with Application Default Credentials (ADC) unless you have a strong reason to inject explicit credentials.

This package exposes multiple generated client surfaces under different modules, most commonly `google.cloud.securitycenter_v1` and `google.cloud.securitycenter_v2`. Choose the module that matches the API resource names and request shapes your project is already using.

## Version Notes

The version used here for this session is `1.42.0`.

Google's package reference landing page is current for the package, but some deep class-reference pages still render with `1.41.0` in the page header. Treat the package version in this entry as authoritative for dependency pinning, and use the latest class docs for signatures and request types.

The package includes several API namespaces (`v1`, `v1beta1`, `v1p1beta1`, `v2`). For new work, prefer `v1` or `v2` unless you are matching an older codebase or a feature that only exists in a beta namespace.

## Install

Pin the version when you need behavior aligned with this entry:

```bash
python -m pip install "google-cloud-securitycenter==1.42.0"
```

If your environment does not already have Google auth tooling available, install the Cloud SDK separately so local ADC setup is straightforward.

## Authentication And Setup

Security Command Center clients use Google auth and gRPC transports by default. The standard setup is:

1. Enable the Security Command Center API for the Google Cloud project or organization you are targeting.
2. Grant the calling identity the required Security Command Center IAM roles.
3. Provide ADC locally with `gcloud auth application-default login`, or in deployed environments with an attached service account / workload identity.

Local development:

```bash
gcloud auth application-default login
```

Service account key fallback:

```bash
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
```

Minimal client creation:

```python
from google.cloud import securitycenter_v1

client = securitycenter_v1.SecurityCenterClient()
```

If you need a non-default endpoint, pass `client_options`:

```python
from google.api_core.client_options import ClientOptions
from google.cloud import securitycenter_v1

client = securitycenter_v1.SecurityCenterClient(
    client_options=ClientOptions(api_endpoint="securitycenter.googleapis.com")
)
```

Use a region-specific endpoint only when your Security Command Center setup requires it. The generated client docs explicitly support overriding `api_endpoint` through `client_options`.

## Choose The API Module

Use this rule of thumb:

- `securitycenter_v1`: safest default for established findings and source-management flows.
- `securitycenter_v2`: use when your workflow is location-aware and the resource names include `/locations/{location}`.
- `securitycenter_v1beta1` or `securitycenter_v1p1beta1`: only use when you are maintaining existing beta-based code or need an API surface not yet promoted.

The biggest practical difference is the resource path shape:

- `v1` commonly uses `organizations/{organization}/sources/{source}`.
- `v2` commonly uses `organizations/{organization}/sources/{source}/locations/{location}`.

Do not mix request objects and resource names across modules.

## Core Usage

### List Findings With `v1`

`list_findings` is a common read path. In `v1`, the `parent` is typically a source resource such as `organizations/123456789/sources/-`.

```python
from google.cloud import securitycenter_v1

client = securitycenter_v1.SecurityCenterClient()

request = securitycenter_v1.ListFindingsRequest(
    parent="organizations/123456789/sources/-",
)

for finding_result in client.list_findings(request=request):
    finding = finding_result.finding
    print(finding.name)
    print(finding.category)
```

Notes:

- `sources/-` is the aggregate form when you want findings across all sources in the organization.
- The iterator is paged for you; iterate directly over the result unless you need manual page control.
- Add a server-side `filter` only when you know the exact Security Command Center filter syntax you need.

### Get One Finding

```python
from google.cloud import securitycenter_v1

client = securitycenter_v1.SecurityCenterClient()

finding = client.get_finding(
    request=securitycenter_v1.GetFindingRequest(
        name=(
            "organizations/123456789/sources/12345/"
            "findings/finding-id"
        )
    )
)

print(finding.name)
print(finding.state)
```

Use `get_finding` when you already have a canonical finding resource name and need the current server state before updating or triaging it.

### List Findings With `v2`

If your deployment and existing resource names are location-scoped, use `securitycenter_v2` instead:

```python
from google.cloud import securitycenter_v2

client = securitycenter_v2.SecurityCenterClient()

request = securitycenter_v2.ListFindingsRequest(
    parent="organizations/123456789/sources/-/locations/us",
)

for finding_result in client.list_findings(request=request):
    print(finding_result.finding.name)
```

Do not silently downgrade a location-scoped resource name to `v1`; keep the client version aligned with the resource path.

### Async Client

Use the async client only when the rest of the application is already async:

```python
import asyncio

from google.cloud import securitycenter_v1

async def main() -> None:
    client = securitycenter_v1.SecurityCenterAsyncClient()
    request = securitycenter_v1.ListFindingsRequest(
        parent="organizations/123456789/sources/-",
    )

    async for finding_result in client.list_findings(request=request):
        print(finding_result.finding.name)

asyncio.run(main())
```

The async surface mirrors the sync API closely. Reuse the same module-selection rules (`v1` vs `v2`) for async code.

## Resource Naming Patterns

Expect to work with full resource names, not short IDs. The most common shapes are:

```text
organizations/{organization}/sources/{source}
organizations/{organization}/sources/{source}/findings/{finding}
organizations/{organization}/sources/{source}/locations/{location}
organizations/{organization}/sources/{source}/locations/{location}/findings/{finding}
```

If you only have numeric IDs, build the full resource name before calling the client. Most generated request types expect canonical names, not separate `organization_id` and `finding_id` fields.

## Configuration Notes

- Reuse client instances when possible. The generated clients manage transports and channels internally, so recreating them per call is unnecessary overhead.
- Prefer explicit request objects such as `ListFindingsRequest(...)` over positional parameters. That makes version/module mismatches easier to catch.
- The default transport is gRPC. Stay on the default unless your environment has a concrete reason to force REST transport.
- Timeout and retry behavior comes from `google-api-core`; override only for known latency or idempotency needs.

## Common Pitfalls

- Importing the wrong module. `securitycenter_v1` and `securitycenter_v2` are both valid, but their request types and resource paths are not interchangeable.
- Using the wrong parent format. `v1` examples typically omit `/locations/...`; `v2` examples include it.
- Forgetting ADC. A plain `SecurityCenterClient()` call will fail if the environment is not authenticated.
- Passing partial IDs instead of full resource names. Most methods want canonical `name` or `parent` strings.
- Assuming every blog post matches the current generated client. For this package, older examples often use beta modules or stale page versions.
- Ignoring IAM. Authentication alone is not enough; the caller also needs the right Security Command Center permissions on the organization, folder, or project scope involved.

## Practical Workflow For Agents

When writing or editing code against this package:

1. Confirm whether the codebase is using `securitycenter_v1` or `securitycenter_v2`.
2. Confirm whether existing resource names contain `/locations/{location}`.
3. Set up ADC first and verify access before debugging request shapes.
4. Start with `list_findings` or `get_finding` to prove connectivity and permissions.
5. Only then add write or triage operations, using request classes from the same module as the read path.

## Official Sources

- Package reference landing page: `https://cloud.google.com/python/docs/reference/securitycenter/latest`
- API class docs (`v1`): `https://cloud.google.com/python/docs/reference/securitycenter/latest/google.cloud.securitycenter_v1.services.security_center.SecurityCenterClient`
- API class docs (`v2`): `https://cloud.google.com/python/docs/reference/securitycenter/latest/google.cloud.securitycenter_v2.services.security_center.SecurityCenterClient`
- Async client docs: `https://cloud.google.com/python/docs/reference/securitycenter/latest/google.cloud.securitycenter_v1.services.security_center.SecurityCenterAsyncClient`
- Google Cloud auth / ADC setup: `https://cloud.google.com/docs/authentication/provide-credentials-adc`
- Source repository package directory: `https://github.com/googleapis/google-cloud-python/tree/main/packages/google-cloud-securitycenter`
- PyPI package page: `https://pypi.org/project/google-cloud-securitycenter/`

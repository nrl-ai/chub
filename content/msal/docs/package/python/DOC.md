---
name: package
description: "MSAL for Python package guide covering app registration, token acquisition flows, caching, broker support, and version-sensitive notes for 1.35.1"
metadata:
  languages: "python"
  versions: "1.35.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "msal,microsoft-entra,azure-ad,oauth,openid-connect,authentication,tokens,broker,managed-identity,python"
---

# msal Python Package Guide

## What This Package Is

`msal` is the Microsoft Authentication Library for Python. Use it when Python code needs to sign in users or apps with Microsoft identities and acquire tokens for Microsoft Graph, Microsoft APIs, or your own APIs protected by Microsoft Entra ID.

For `1.35.1`, the PyPI release requires Python `>=3.8` and provides an optional `broker` extra. As of `2026-03-12`, the official Read the Docs landing page still shows `MSAL Python 1.35.0 documentation`, so treat the docs site as slightly behind the package version.

## Install

```bash
pip install "msal==1.35.1"
```

With broker support:

```bash
pip install "msal[broker]==1.35.1"
```

## Before You Write Code

You usually need a Microsoft Entra app registration with:

- an application (client) ID
- a tenant-specific authority or an explicit multi-tenant authority
- the correct redirect URI for the flow you are using
- delegated scopes or application permissions for the downstream API
- a client secret or certificate for confidential clients

Common authorities:

- Single tenant: `https://login.microsoftonline.com/<tenant-id>`
- Multi-tenant workforce accounts: `https://login.microsoftonline.com/organizations`
- Workforce plus personal Microsoft accounts: `https://login.microsoftonline.com/common`

Use a tenant-specific authority for daemon apps, on-behalf-of flows, and most production web apps. `common` is mainly for interactive user sign-in.

## Pick The Right Application Type

- `msal.PublicClientApplication`: desktop apps, CLIs, device-code flows, and other apps that cannot safely hold a secret
- `msal.ConfidentialClientApplication`: web apps, web APIs, daemons, and background jobs that can safely hold a client secret or certificate
- `msal.ManagedIdentityClient`: Azure-hosted workloads using managed identity instead of an Entra app credential

MSAL returns a dictionary. Check for `access_token` first and treat anything else as an error payload.

```python
def require_access_token(result: dict) -> str:
    if "access_token" in result:
        return result["access_token"]
    raise RuntimeError(
        f"{result.get('error')}: {result.get('error_description')}"
    )
```

## Public Client Apps

Use `PublicClientApplication` for user sign-in flows in CLIs, desktop apps, and local tools.

### Interactive Sign-In

```python
import msal

CLIENT_ID = "your-client-id"
AUTHORITY = "https://login.microsoftonline.com/common"
SCOPES = ["User.Read"]

app = msal.PublicClientApplication(
    CLIENT_ID,
    authority=AUTHORITY,
)

accounts = app.get_accounts()
result = None

if accounts:
    result = app.acquire_token_silent(SCOPES, account=accounts[0])

if not result:
    result = app.acquire_token_interactive(
        scopes=SCOPES,
        redirect_uri="http://localhost",
    )

access_token = require_access_token(result)
```

Notes:

- `acquire_token_interactive()` uses PKCE automatically.
- `http://localhost` must be registered as a redirect URI in the app registration.
- For delegated user flows, request named scopes like `User.Read`, not `resource/.default`.

### Device Code Flow

Use this for terminals, remote shells, or headless hosts.

```python
import json
import msal

app = msal.PublicClientApplication(
    "your-client-id",
    authority="https://login.microsoftonline.com/common",
)

flow = app.initiate_device_flow(scopes=["User.Read"])
if "user_code" not in flow:
    raise RuntimeError(json.dumps(flow, indent=2))

print(flow["message"])
result = app.acquire_token_by_device_flow(flow)
access_token = require_access_token(result)
```

### Brokered Interactive Sign-In

Broker support is optional and platform-specific. The API reference for the current docs exposes `enable_broker_on_windows`, `enable_broker_on_mac`, `enable_broker_on_linux`, and `enable_broker_on_wsl`.

```python
import msal

app = msal.PublicClientApplication(
    "your-client-id",
    authority="https://login.microsoftonline.com/common",
    enable_broker_on_windows=True,
)

result = app.acquire_token_interactive(
    scopes=["User.Read"],
    parent_window_handle=app.CONSOLE_WINDOW_HANDLE,
)
```

Broker-specific notes:

- Install the package with `msal[broker]`.
- Windows broker requires a broker redirect URI of the form `ms-appx-web://microsoft.aad.brokerplugin/<client_id>`.
- Do not keep using `allow_broker`; the API reference marks it deprecated in favor of explicit platform flags.
- The Learn WAM article is Windows-focused. Use the API reference for the cross-platform flags that are present in the current package line.

### Avoid Username/Password

Do not build new code on `acquire_token_by_username_password()`. The official docs mark it deprecated for public client flows.

## Confidential Client Apps

Use `ConfidentialClientApplication` when your code can securely hold credentials.

### Client Credentials Flow

Use this when the app acts as itself.

```python
import msal

app = msal.ConfidentialClientApplication(
    client_id="your-client-id",
    client_credential="your-client-secret",
    authority="https://login.microsoftonline.com/your-tenant-id",
)

result = app.acquire_token_for_client(
    scopes=["https://graph.microsoft.com/.default"]
)

access_token = require_access_token(result)
```

Notes:

- Client credentials use `resource/.default` scopes.
- Since MSAL Python `1.23`, `acquire_token_for_client()` checks the token cache before making a network call.
- `client_credential` can be a client secret, a certificate dictionary, or another advanced credential form supported by the API docs.

### Authorization Code Flow For Web Apps

```python
import msal

app = msal.ConfidentialClientApplication(
    client_id="your-client-id",
    client_credential="your-client-secret",
    authority="https://login.microsoftonline.com/your-tenant-id",
)

flow = app.initiate_auth_code_flow(
    scopes=["User.Read"],
    redirect_uri="https://app.example.com/callback",
    response_mode="form_post",
)

# Redirect the browser to flow["auth_uri"]
```

Then handle the callback:

```python
try:
    result = app.acquire_token_by_auth_code_flow(
        auth_code_flow=session["flow"],
        auth_response=request.form or request.args,
    )
except ValueError as exc:
    raise RuntimeError("State or CSRF validation failed") from exc

access_token = require_access_token(result)
```

Notes:

- `response_mode="form_post"` is safer than query parameters for web apps and is supported in the current MSAL docs.
- Keep redirect URI, authority, and app registration settings exactly aligned.

### On-Behalf-Of Flow

Use this when your API receives a user token and needs a downstream token.

```python
import msal

app = msal.ConfidentialClientApplication(
    client_id="your-client-id",
    client_credential="your-client-secret",
    authority="https://login.microsoftonline.com/your-tenant-id",
)

incoming_access_token = request.headers["Authorization"].split()[1]

result = app.acquire_token_on_behalf_of(
    user_assertion=incoming_access_token,
    scopes=["api://downstream-api-id/.default"],
)

downstream_access_token = require_access_token(result)
```

## Token Cache And Persistence

MSAL caches tokens in memory by default. If you want persistence, you must supply it.

```python
import atexit
import os

import msal

CACHE_FILE = ".msal_cache.bin"

cache = msal.SerializableTokenCache()
if os.path.exists(CACHE_FILE):
    with open(CACHE_FILE, "r", encoding="utf-8") as f:
        cache.deserialize(f.read())

def persist_cache() -> None:
    if cache.has_state_changed:
        with open(CACHE_FILE, "w", encoding="utf-8") as f:
            f.write(cache.serialize())

atexit.register(persist_cache)

app = msal.PublicClientApplication(
    "your-client-id",
    authority="https://login.microsoftonline.com/common",
    token_cache=cache,
)
```

Practical rules:

- Try `acquire_token_silent()` before any interactive prompt.
- Use `acquire_token_silent_with_error()` when you need to tell "cache miss" from "refresh failed".
- `SerializableTokenCache` does not encrypt or persist to disk by itself.
- If you need encrypted persistence, use the Microsoft-maintained `msal-extensions` helper library.

### `http_cache`

MSAL also supports a separate `http_cache` for non-token HTTP metadata. It can reduce repeated instance-discovery and OpenID metadata fetches in long-lived CLIs, but its format is intentionally not stable across versions.

## Managed Identity

Use `ManagedIdentityClient` only on supported Azure hosts and only when you actually want managed identity semantics instead of a normal app registration.

```python
import msal
import requests

managed_identity = msal.SystemAssignedManagedIdentity()
client = msal.ManagedIdentityClient(
    managed_identity,
    http_client=requests.Session(),
)

result = client.acquire_token_for_client(
    resource="https://vault.azure.net"
)

access_token = require_access_token(result)
```

Notes:

- Managed identity support is available in MSAL Python `1.29.0+`.
- Managed identity uses `resource=...`, not a `scopes=[...]` list.
- Pass a reusable `requests.Session()` as `http_client`.
- If your code already uses Azure Identity successfully, stay there unless you specifically need MSAL-level control.

## Configuration Options Worth Knowing

### `client_capabilities`

Use `client_capabilities=["CP1"]` when your app must declare support for claims challenges such as Continuous Access Evaluation.

### `exclude_scopes`

MSAL historically adds `offline_access`. If you do not want refresh-token style behavior:

```python
app = msal.PublicClientApplication(
    "your-client-id",
    exclude_scopes=["offline_access"],
)
```

### `oidc_authority`

Use `oidc_authority` for a custom OIDC provider outside the standard Microsoft Entra authority patterns.

```python
app = msal.PublicClientApplication(
    "your-client-id",
    oidc_authority="https://contoso.example/tenant",
)
```

Notes:

- This was added before the `1.35.x` line and is available in the current package version.
- The API docs explicitly say broker does not work with `oidc_authority`.

### `instance_discovery`

Leave `instance_discovery` enabled unless you know exactly which authorities are valid in your environment. Disabling it blindly can weaken authority validation.

### Certificate Credentials

If you use certificate auth, verify the current credential shape against the API reference. The current docs line supports PFX-based inputs and notes that a SHA-256 thumbprint can be calculated automatically when `public_certificate` is available and `thumbprint` is omitted.

## Common Pitfalls

- Do not use `resource/.default` for normal interactive delegated sign-in unless that consent model is intentional.
- Do not skip cache lookups. Repeated sign-in is usually a flow bug.
- Do not mix broker redirect URIs and localhost redirect URIs; they solve different problems.
- Do not treat managed identity like a normal `scopes=[...]` flow.
- Do not use `common` for daemon apps and most production web APIs.
- Do not disable authority validation or instance discovery unless you control the authority list.
- Do not assume the docs site is fully version-pinned. The official docs root currently shows `1.35.0`, while the target package version here is `1.35.1`.
- Do not keep using `allow_broker`; use the explicit platform flags instead.

## Version-Sensitive Notes For 1.35.1

- Target version version: `1.35.1`
- PyPI release date: `2026-03-04`
- Python requirement on that release: `>=3.8`
- Optional extra on that release: `broker`
- Official docs root status on `2026-03-12`: Read the Docs landing page still shows `MSAL Python 1.35.0 documentation`
- Official `1.35.1` release note: instance discovery remains cloud-local on known clouds
- Official `1.35.0` release notes and API docs highlight:
  - Python `3.14` support
  - support for `response_mode="form_post"` in auth-code flow helpers
  - OIDC issuer validation support for managed identity
  - a Linux broker silent-flow redirect fix
  - certificate handling updates around PFX inputs and SHA-256 thumbprints

Inference from official sources:

- `1.35.1` appears to be a patch release on top of the `1.35.0` docs line, not a separate new docs set.
- If your code depends on authority discovery behavior, broker behavior, or certificate auth details, prefer the package version page plus GitHub release notes over assuming the Read the Docs landing page is fully current.

## Official Sources

- PyPI package page: https://pypi.org/project/msal/
- PyPI `1.35.1` page: https://pypi.org/project/msal/1.35.1/
- API docs root: https://msal-python.readthedocs.io/en/latest/
- Token acquisition guide: https://learn.microsoft.com/en-us/entra/msal/python/getting-started/acquiring-tokens
- Managed identity guide: https://learn.microsoft.com/en-us/entra/msal/python/advanced/managed-identity
- WAM and broker guide: https://learn.microsoft.com/en-us/entra/msal/python/advanced/wam
- Releases page: https://github.com/AzureAD/microsoft-authentication-library-for-python/releases

---
name: oauth-toolkit
description: "django-oauth-toolkit package guide for building OAuth2 and OpenID Connect providers in Django"
metadata:
  languages: "python"
  versions: "3.2.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,oauth2,openid-connect,drf,authentication,authorization"
---

# django-oauth-toolkit Python Package Guide

## What It Does

`django-oauth-toolkit` (DOT) turns a Django project into an OAuth 2.0 authorization server. It provides authorization, token, revocation, introspection, application management, Django REST framework integration, and optional OpenID Connect support.

Use it when your Django app must issue and validate OAuth tokens for first-party or third-party clients. Do not use it as a social-login client library.

## Version Context

- Ecosystem: `pypi`
- Package: `django-oauth-toolkit`
- Version covered: `3.2.0`
- Docs root: `https://django-oauth-toolkit.readthedocs.io/en/latest/`
- PyPI URL: `https://pypi.org/project/django-oauth-toolkit/`

The official docs currently publish a `3.2.0` snapshot, so the package version and docs version are aligned for this entry.

## Install

```bash
pip install django-oauth-toolkit==3.2.0
```

If you are protecting Django REST framework APIs, install DRF too:

```bash
pip install django-oauth-toolkit==3.2.0 djangorestframework
```

Official compatibility signals are slightly inconsistent across upstream surfaces:

- Read the Docs `3.2.0` docs say DOT supports Python `3.10` through `3.14`, Django `4.2` through `6.0`, and `oauthlib>=3.2.2`.
- The PyPI project page still shows older requirement text and classifiers in some sections.

For upgrade decisions, prefer the docs root plus changelog over stale PyPI prose.

## Minimal Provider Setup

Add the app, define scopes, and keep PKCE on.

```python
# settings.py

INSTALLED_APPS = [
    # Django apps...
    "oauth2_provider",
]

OAUTH2_PROVIDER = {
    "SCOPES": {
        "read": "Read access",
        "write": "Write access",
    },
    "PKCE_REQUIRED": True,
}
```

Expose the OAuth endpoints:

```python
# urls.py
from django.urls import include, path
from oauth2_provider import urls as oauth2_urls

urlpatterns = [
    path("o/", include(oauth2_urls)),
]
```

Run migrations:

```bash
python manage.py migrate
python manage.py createsuperuser
```

After that, create applications in Django admin under `Applications`.

## Register a Client Application

For most browser, SPA, and mobile clients:

- Grant type: `Authorization code`
- Client type: `Public` for SPA/native clients, `Confidential` for server-side clients
- Redirect URIs: exact callback URIs
- Allowed origins: needed for browser-based clients calling the token endpoint from another origin
- PKCE: expected by default

For confidential clients, copy the generated client secret before saving if you need it outside the server. DOT hashes client secrets on save in modern releases, so you cannot recover the plain secret later from Django admin.

If you want to create applications programmatically, DOT ships a `createapplication` management command:

```bash
python manage.py createapplication \
  "web-app" \
  admin \
  authorization-code \
  confidential \
  "https://app.example.com/callback"
```

## Authorization Code Flow

The common flow is:

1. Redirect the user to `/o/authorize/`.
2. Receive an authorization code on your redirect URI.
3. Exchange the code at `/o/token/`.
4. Send the bearer token to protected APIs.

Example authorization URL:

```python
from urllib.parse import urlencode

params = urlencode(
    {
        "client_id": "YOUR_CLIENT_ID",
        "response_type": "code",
        "redirect_uri": "https://app.example.com/callback",
        "scope": "read write",
        "code_challenge": "BASE64URL_SHA256_CODE_VERIFIER",
        "code_challenge_method": "S256",
    }
)

authorize_url = f"https://auth.example.com/o/authorize/?{params}"
```

Example code exchange for a public PKCE client:

```python
import requests

response = requests.post(
    "https://auth.example.com/o/token/",
    data={
        "grant_type": "authorization_code",
        "client_id": "YOUR_CLIENT_ID",
        "code": "CODE_FROM_CALLBACK",
        "redirect_uri": "https://app.example.com/callback",
        "code_verifier": "ORIGINAL_CODE_VERIFIER",
    },
    timeout=30,
)
response.raise_for_status()
token = response.json()
```

For confidential clients, authenticate the token request with the client secret instead of treating the app like a public PKCE-only client.

## Protect Django REST Framework APIs

DOT integrates directly with DRF through `OAuth2Authentication` and scope-aware permission classes.

```python
# settings.py
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": (
        "oauth2_provider.contrib.rest_framework.OAuth2Authentication",
    ),
}
```

```python
# views.py
from oauth2_provider.contrib.rest_framework import (
    OAuth2Authentication,
    TokenHasReadWriteScope,
)
from rest_framework.response import Response
from rest_framework.views import APIView

class DocumentView(APIView):
    authentication_classes = [OAuth2Authentication]
    permission_classes = [TokenHasReadWriteScope]
    required_scopes = ["documents"]

    def get(self, request):
        return Response({"ok": True})
```

`TokenHasReadWriteScope` requires the configured read or write scope depending on the HTTP method, plus any `required_scopes` you declare on the view. Use `TokenHasScope` when you want exact scope checks without read/write semantics.

## Settings That Usually Matter

Start with these:

- `SCOPES`: define every scope your tokens may request.
- `DEFAULT_SCOPES`: narrow the defaults if you do not want `__all__`.
- `PKCE_REQUIRED`: defaults to `True`; keep it that way unless you have a controlled legacy client that cannot support PKCE.
- `ALLOWED_REDIRECT_URI_SCHEMES`: set to `["https"]` in production. Loopback `http` redirects are acceptable for native apps.
- `REFRESH_TOKEN_REUSE_PROTECTION`: use this with rotated refresh tokens to detect reuse and revoke the related session.
- `REFRESH_TOKEN_EXPIRE_SECONDS`: controls cleanup eligibility, not actual refresh-token validation by itself.
- `CLEAR_EXPIRED_TOKENS_BATCH_SIZE` and `CLEAR_EXPIRED_TOKENS_BATCH_INTERVAL`: tune large cleanup jobs.
- `ALLOW_URI_WILDCARDS`: added in `3.1.0`; avoid enabling it in production unless you fully understand the redirect-origin risk.

DOT also ships `cleartokens`, which you should schedule if you issue refresh tokens:

```bash
python manage.py cleartokens
```

Without regular cleanup, expired refresh tokens can accumulate in the database. The docs are explicit that `REFRESH_TOKEN_EXPIRE_SECONDS` alone does not make refresh-token validation expire on its own.

## OpenID Connect Setup

OIDC is optional and disabled by default. Prefer `RS256` for signing ID tokens.

```bash
openssl genrsa -out oidc.key 4096
```

```python
# settings.py
import os

OAUTH2_PROVIDER = {
    "OIDC_ENABLED": True,
    "OIDC_RSA_PRIVATE_KEY": os.environ["OIDC_RSA_PRIVATE_KEY"],
    "SCOPES": {
        "openid": "OpenID Connect scope",
        "profile": "Profile access",
        "email": "Email access",
    },
}
```

Important OIDC constraints:

- Keep the RSA private key out of source control.
- If you already customized `OAUTH2_SERVER_CLASS`, OIDC requires a class derived from `oauthlib.openid.Server`.
- `HS256` uses the application client secret for signing. That is simpler, but it is less flexible and does not work for public clients or implicit/hybrid flows. Prefer `RS256`.
- Use `OIDC_RSA_PRIVATE_KEYS_INACTIVE` for key rotation instead of replacing keys abruptly.

## Separate Resource Server Pattern

If your API and authorization server are separate services, DOT supports token introspection.

On the authorization server:

- include the introspection endpoint under `/o/introspect/`
- add an `introspection` scope
- issue a token the resource server can use

On the resource server:

- set `RESOURCE_SERVER_INTROSPECTION_URL`
- set either `RESOURCE_SERVER_AUTH_TOKEN` or `RESOURCE_SERVER_INTROSPECTION_CREDENTIALS`

Use this only when you actually split the auth server and resource server. For a single Django app, standard local token validation is simpler.

## Common Pitfalls

- DOT is an authorization server toolkit, not a third-party OAuth client or social-login package.
- Older blog posts often show outdated setup details. Follow the current `3.2.0` docs for URL wiring and settings names.
- Public clients will fail authorization-code token exchange if you omit `code_challenge` and `code_verifier`, because `PKCE_REQUIRED` is on by default.
- The plain client secret is not recoverable after save when hashing is enabled. Copy it at creation time if a confidential client needs it.
- `REFRESH_TOKEN_EXPIRE_SECONDS` does not automatically invalidate refresh tokens during validation. It mainly drives `cleartokens` cleanup unless you customize validation.
- `ALLOW_URI_WILDCARDS` is a security-sensitive escape hatch. Treat it as a last resort for controlled preview environments.
- Upgrading from pre-`3.0.0` requires extra care if you use swappable DOT models or code that assumed token model primary keys were named `id`.

## Version-Sensitive Notes

- `3.2.0` adds device authorization grant support and adds support for Django `5.2` and Python `3.14`.
- `3.1.0` adds wildcard redirect/origin support through `ALLOW_URI_WILDCARDS` and fixes several migration and introspection issues.
- `3.0.1` fixes a migration error when pre-existing access tokens are already in the database.
- `3.0.0` is the important breaking line:
  - run `python manage.py migrate` after upgrading
  - update custom swappable models and generate migrations if needed
  - Django versions below `4.2` are no longer supported
  - access-token storage changed to use `TextField` plus `token_checksum`
  - `REFRESH_TOKEN_REUSE_PROTECTION` was introduced as a security control
  - minimum `oauthlib` is `3.2.2+`

## Official Sources Used

- Docs root: https://django-oauth-toolkit.readthedocs.io/en/latest/
- Installation: https://django-oauth-toolkit.readthedocs.io/en/latest/install.html
- Tutorial part 1: https://django-oauth-toolkit.readthedocs.io/en/latest/tutorial/tutorial_01.html
- Django REST framework integration: https://django-oauth-toolkit.readthedocs.io/en/latest/rest-framework/getting_started.html
- DRF permissions: https://django-oauth-toolkit.readthedocs.io/en/latest/rest-framework/permissions.html
- Settings: https://django-oauth-toolkit.readthedocs.io/en/latest/settings.html
- OpenID Connect: https://django-oauth-toolkit.readthedocs.io/en/latest/oidc.html
- Separate resource server: https://django-oauth-toolkit.readthedocs.io/en/latest/resource_server.html
- Management commands: https://django-oauth-toolkit.readthedocs.io/en/latest/management_commands.html
- Changelog: https://django-oauth-toolkit.readthedocs.io/en/latest/changelog.html
- PyPI package page: https://pypi.org/project/django-oauth-toolkit/

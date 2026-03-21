---
name: rest-framework-simplejwt
description: "Simple JWT for Django REST Framework in Python projects"
metadata:
  languages: "python"
  versions: "5.5.1"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,django-rest-framework,jwt,authentication,tokens"
---

# Django REST Framework Simple JWT for Python

`djangorestframework-simplejwt` adds JWT authentication views, token classes, and DRF authentication backends. Use it when your Django app issues bearer tokens and your DRF API needs short-lived access tokens plus refresh-token flows.

## Install

Pin the package version your project expects:

```bash
python -m pip install "djangorestframework-simplejwt==5.5.1"
```

If you will sign or verify tokens with RSA or ECDSA algorithms, install the crypto extra:

```bash
python -m pip install "djangorestframework-simplejwt[crypto]==5.5.1"
```

Do not add `rest_framework_simplejwt` to `INSTALLED_APPS` for normal auth wiring. The maintainer docs only call that out for translations. The blacklist feature is separate and uses `rest_framework_simplejwt.token_blacklist`.

## Minimal Setup

Configure DRF to use JWT auth in `settings.py`:

```python
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": (
        "rest_framework_simplejwt.authentication.JWTAuthentication",
    ),
}
```

Expose token endpoints in `urls.py`:

```python
from django.urls import path
from rest_framework_simplejwt.views import (
    TokenObtainPairView,
    TokenRefreshView,
    TokenVerifyView,
)

urlpatterns = [
    path("api/token/", TokenObtainPairView.as_view(), name="token_obtain_pair"),
    path("api/token/refresh/", TokenRefreshView.as_view(), name="token_refresh"),
    path("api/token/verify/", TokenVerifyView.as_view(), name="token_verify"),
]
```

Protect DRF views with normal DRF permissions:

```python
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response
from rest_framework.views import APIView

class MeView(APIView):
    permission_classes = [IsAuthenticated]

    def get(self, request):
        return Response({"user_id": request.user.pk})
```

Typical client flow:

1. `POST /api/token/` with username and password to get `access` and `refresh`.
2. Send `Authorization: Bearer <access-token>` to protected views.
3. `POST /api/token/refresh/` with the refresh token when the access token expires.
4. Optionally `POST /api/token/verify/` when clients need a validity check for HMAC-signed tokens.

`TokenVerifyView` only answers whether a token is valid. It is not an authorization decision point.

## Core `SIMPLE_JWT` Settings

The stable `5.5.1` docs show these defaults:

```python
from datetime import timedelta
from django.conf import settings

SIMPLE_JWT = {
    "ACCESS_TOKEN_LIFETIME": timedelta(minutes=5),
    "REFRESH_TOKEN_LIFETIME": timedelta(days=1),
    "ROTATE_REFRESH_TOKENS": False,
    "BLACKLIST_AFTER_ROTATION": False,
    "UPDATE_LAST_LOGIN": False,
    "ALGORITHM": "HS256",
    "SIGNING_KEY": settings.SECRET_KEY,
    "VERIFYING_KEY": "",
    "AUDIENCE": None,
    "ISSUER": None,
    "JSON_ENCODER": None,
    "JWK_URL": None,
    "LEEWAY": 0,
    "AUTH_HEADER_TYPES": ("Bearer",),
    "AUTH_HEADER_NAME": "HTTP_AUTHORIZATION",
    "USER_ID_FIELD": "id",
    "USER_ID_CLAIM": "user_id",
    "AUTH_TOKEN_CLASSES": ("rest_framework_simplejwt.tokens.AccessToken",),
}
```

Practical guidance:

- Keep access tokens short-lived. The default `5` minutes is a reasonable starting point.
- Pair `ROTATE_REFRESH_TOKENS` with `BLACKLIST_AFTER_ROTATION` only when the blacklist app is installed and migrated.
- Leave `UPDATE_LAST_LOGIN` off unless you explicitly need it. The maintainer docs warn that enabling it substantially increases database writes and should be paired with DRF throttling on login endpoints.
- If you use HMAC signing, prefer a dedicated JWT signing key instead of reusing Django `SECRET_KEY`.
- If other services validate your tokens, set `ISSUER`, `AUDIENCE`, `ALGORITHM`, and verifying material consistently across those services.
- `AUTH_HEADER_TYPES` and `AUTH_HEADER_NAME` must match whatever your gateway or client actually sends.
- `USER_ID_FIELD` should be stable over time. Avoid fields like username or email if those values can change.

## Login, Refresh, And Auth Headers

Request a token pair:

```bash
curl -X POST http://localhost:8000/api/token/ \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"password"}'
```

Example response:

```json
{
  "refresh": "<refresh-token>",
  "access": "<access-token>"
}
```

Use the access token on protected requests:

```bash
curl http://localhost:8000/api/me/ \
  -H "Authorization: Bearer <access-token>"
```

Refresh it:

```bash
curl -X POST http://localhost:8000/api/token/refresh/ \
  -H "Content-Type: application/json" \
  -d '{"refresh":"<refresh-token>"}'
```

## Logout And Revocation

If you need server-side revocation for refresh or sliding tokens, enable the blacklist app:

```python
INSTALLED_APPS = [
    # ...
    "rest_framework_simplejwt.token_blacklist",
]
```

Then run migrations:

```bash
python manage.py migrate
```

Optionally expose the blacklist endpoint:

```python
from django.urls import path
from rest_framework_simplejwt.views import TokenBlacklistView

urlpatterns = [
    path("api/token/blacklist/", TokenBlacklistView.as_view(), name="token_blacklist"),
]
```

You can also blacklist a refresh token directly:

```python
from rest_framework_simplejwt.tokens import RefreshToken

def revoke_refresh_token(token_string: str) -> None:
    token = RefreshToken(token_string)
    token.blacklist()
```

Operational note: the blacklist app stores `OutstandingToken` and `BlacklistedToken` rows. Run `python manage.py flushexpiredtokens` on a schedule, typically daily, to remove expired entries.

## Custom Claims

Customize claims by subclassing the serializer used by the obtain-token view:

```python
from rest_framework_simplejwt.serializers import TokenObtainPairSerializer
from rest_framework_simplejwt.views import TokenObtainPairView

class MyTokenObtainPairSerializer(TokenObtainPairSerializer):
    @classmethod
    def get_token(cls, user):
        token = super().get_token(user)
        token["tenant_id"] = str(user.tenant_id)
        token["is_staff"] = user.is_staff
        return token

class MyTokenObtainPairView(TokenObtainPairView):
    serializer_class = MyTokenObtainPairSerializer
```

If you want the built-in view wiring to use your serializer by default:

```python
SIMPLE_JWT = {
    "TOKEN_OBTAIN_SERIALIZER": "my_app.serializers.MyTokenObtainPairSerializer",
}
```

Important behavior: changes made in `get_token()` affect both refresh and access tokens because the access token is derived from the refresh token.

## Creating Tokens Manually

For service-side issuance, use `RefreshToken.for_user(user)`:

```python
from rest_framework_simplejwt.exceptions import AuthenticationFailed
from rest_framework_simplejwt.tokens import RefreshToken

def issue_tokens_for_user(user):
    if not user.is_active:
        raise AuthenticationFailed("User is not active")

    refresh = RefreshToken.for_user(user)
    return {
        "refresh": str(refresh),
        "access": str(refresh.access_token),
    }
```

Do not skip the `is_active` check. The maintainer docs explicitly warn that `for_user()` does not validate it for you.

## Token Types

Simple JWT supports these token families:

- Access tokens: the default for request authentication.
- Refresh tokens: used to mint new access tokens, not to authenticate protected views.
- Sliding tokens: combined auth-plus-refresh tokens with a separate refresh expiration claim.

If you want sliding tokens, include `SlidingToken` in `AUTH_TOKEN_CLASSES` and use the sliding views:

```python
from django.urls import path
from rest_framework_simplejwt.views import (
    TokenObtainSlidingView,
    TokenRefreshSlidingView,
)

urlpatterns = [
    path("api/token/", TokenObtainSlidingView.as_view(), name="token_obtain"),
    path("api/token/refresh/", TokenRefreshSlidingView.as_view(), name="token_refresh"),
]
```

Sliding tokens are more convenient for clients, but the docs call them less secure and, when the blacklist app is enabled, less performant because each authenticated request checks the blacklist.

## Stateless User Authentication

If your stack needs token-backed SSO behavior across multiple Django services without a database lookup on every request, use `JWTStatelessUserAuthentication`:

```python
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": (
        "rest_framework_simplejwt.authentication.JWTStatelessUserAuthentication",
    ),
}
```

This returns a `TokenUser` backed by the validated token rather than a database-loaded user object. In `5.1.0`, `JWTTokenUserAuthentication` was renamed to `JWTStatelessUserAuthentication`, but both names remain supported for backward compatibility.

## Version-Sensitive Notes

- An older docs link pointed to `/en/latest/`, but `/en/stable/` is the safer canonical docs root for package-specific `5.5.1` guidance.
- PyPI `5.5.1` is still the latest release as of March 12, 2026.
- The `5.5.1` changelog notes a previously missing `0013_blacklist` migration for `rest_framework_simplejwt.token_blacklist`. If you generated your own `0013_blacklist` migration from the old master branch state, read the maintainer changelog before upgrading.
- The stable docs still describe Django support through `5.1`, while the PyPI classifiers already include `5.2`. Prefer the docs for behavior and migration guidance until the package docs catch up.

## Common Pitfalls

- Installing the package without adding `JWTAuthentication` or `JWTStatelessUserAuthentication` to DRF auth classes.
- Sending refresh tokens to protected endpoints instead of access tokens.
- Enabling blacklist-related settings without installing `rest_framework_simplejwt.token_blacklist` and running migrations.
- Treating `TokenVerifyView` as an authorization check.
- Using asymmetric algorithms without installing the `crypto` extra.
- Enabling `UPDATE_LAST_LOGIN` on a public login endpoint without DRF throttling.
- Assuming `RefreshToken.for_user()` blocks inactive users.

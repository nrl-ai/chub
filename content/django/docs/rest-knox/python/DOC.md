---
name: rest-knox
description: "django-rest-knox package guide for token authentication in Django REST Framework projects"
metadata:
  languages: "python"
  versions: "5.0.4"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,drf,authentication,token,api,security,knox"
---

# django-rest-knox Python Package Guide

## What This Package Does

`django-rest-knox` adds expiring, server-managed authentication tokens to Django REST Framework.

- PyPI package: `django-rest-knox`
- Django app and import namespace: `knox`
- Version covered here: `5.0.4`
- Docs URL for this session: `https://github.com/jazzband/django-rest-knox`
- Canonical docs root used for setup details: `https://jazzband.github.io/django-rest-knox/`

Use Knox when DRF's built-in token auth is too limited and you need:

- more than one token per user
- token expiry and optional auto-refresh
- logout for only the current token or for all user tokens
- database storage of token digests instead of recoverable plaintext tokens

The live docs site is a current-project guide, not a version-frozen `5.0.4` snapshot. For version-sensitive behavior, cross-check the changelog and PyPI metadata before copying older examples.

## Install And Compatibility

Install the version used here explicitly:

```bash
pip install "django-rest-knox==5.0.4"
```

For a new DRF project:

```bash
pip install "django>=4.2" "djangorestframework" "django-rest-knox==5.0.4"
```

Official package metadata for `5.0.4` shows:

- Python: `>=3.8`
- Django dependency: `>=4.2`
- Django REST framework dependency: `>=3.14`

`5.0.4` is a support-matrix release. The official changelog says it adds support for Django `5.2` and `6.0`, plus Python `3.13` and `3.14`, without changing the minimum supported baselines.

## Minimal Django Setup

Add Knox to `INSTALLED_APPS` and configure DRF to use Knox tokens for authenticated API requests:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "rest_framework",
    "knox",
]

REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": (
        "knox.auth.TokenAuthentication",
    ),
    "DEFAULT_PERMISSION_CLASSES": (
        "rest_framework.permissions.IsAuthenticated",
    ),
}
```

Run migrations before testing login or protected routes:

```bash
python manage.py migrate
```

If the project previously used DRF's built-in `rest_framework.authtoken`, remove that token system cleanly instead of running both in parallel.

## Login, Logout, And A Protected Endpoint

The main integration trap is login. If Knox token auth is the only default DRF authentication class, the stock `knox.views.LoginView` is not enough on its own because the request must already be authenticated before Knox can mint a token.

The official docs recommend authenticating the username and password first, then delegating token creation to Knox:

```python
# views.py
from django.contrib.auth import login
from rest_framework import permissions
from rest_framework.authtoken.serializers import AuthTokenSerializer
from rest_framework.response import Response
from rest_framework.views import APIView

from knox.auth import TokenAuthentication
from knox.views import LoginView as KnoxLoginView

class LoginView(KnoxLoginView):
    permission_classes = [permissions.AllowAny]

    def post(self, request, format=None):
        serializer = AuthTokenSerializer(data=request.data)
        serializer.is_valid(raise_exception=True)
        user = serializer.validated_data["user"]
        login(request, user)
        return super().post(request, format=None)

class MeView(APIView):
    authentication_classes = [TokenAuthentication]
    permission_classes = [permissions.IsAuthenticated]

    def get(self, request):
        return Response(
            {
                "user_id": request.user.pk,
                "username": request.user.get_username(),
            }
        )
```

Wire the endpoints:

```python
# urls.py
from django.urls import path

from knox import views as knox_views

from .views import LoginView, MeView

urlpatterns = [
    path("api/auth/login/", LoginView.as_view(), name="knox_login"),
    path("api/auth/logout/", knox_views.LogoutView.as_view(), name="knox_logout"),
    path("api/auth/logoutall/", knox_views.LogoutAllView.as_view(), name="knox_logoutall"),
    path("api/me/", MeView.as_view(), name="me"),
]
```

Typical request flow:

```http
POST /api/auth/login/
Content-Type: application/json

{"username": "alice", "password": "secret"}
```

Then send the returned token on later requests:

```http
Authorization: Token <token-value>
```

`POST /api/auth/logout/` deletes only the current token. `POST /api/auth/logoutall/` deletes every token for the authenticated user.

## Using `knox.urls`

Knox ships a small URLconf:

```python
from django.urls import include, path

urlpatterns = [
    path("api/auth/", include("knox.urls")),
]
```

Two details matter:

- Use string-based `include("knox.urls")`, not a direct import of the module.

If the project wants JSON username/password login while global auth defaults to Knox tokens, use the custom `LoginView` pattern above.

## Core Configuration

Knox settings live under `REST_KNOX`:

```python
# settings.py
from datetime import timedelta

REST_KNOX = {
    "TOKEN_TTL": timedelta(hours=10),
    "AUTO_REFRESH": False,
    "AUTO_REFRESH_MAX_TTL": timedelta(days=7),
    "MIN_REFRESH_INTERVAL": 60,
    "TOKEN_LIMIT_PER_USER": None,
    "AUTH_HEADER_PREFIX": "Token",
    "TOKEN_PREFIX": "",
}
```

Settings that usually matter first:

- `TOKEN_TTL`: token lifetime. `None` means tokens do not expire.
- `AUTO_REFRESH`: extend token expiry on use.
- `AUTO_REFRESH_MAX_TTL`: cap total token lifetime when auto-refresh is enabled.
- `MIN_REFRESH_INTERVAL`: reduce database writes by not refreshing expiry on every request.
- `TOKEN_LIMIT_PER_USER`: cap concurrent valid tokens per user.
- `AUTH_HEADER_PREFIX`: defaults to `Token`.
- `TOKEN_PREFIX`: add a prefix to generated token values if you need token-type namespacing.

The package code for `5.0.4` sets `USER_SERIALIZER` to `None` by default. Only set it if you want the login response to include serialized user data.

## Customizing The Login Response

Knox `5.x` exposes hooks for customizing the login payload, including `create_token`, `get_token_ttl`, and `get_post_response_data`.

Example:

```python
# settings.py
REST_KNOX = {
    "USER_SERIALIZER": "accounts.api.UserSerializer",
}
```

```python
# views.py
from knox.views import LoginView as KnoxLoginView

class CustomLoginView(KnoxLoginView):
    def get_post_response_data(self, request, token, instance):
        return {
            "expiry": self.format_expiry_datetime(instance.expiry),
            "token": token,
            "user": self.get_user_serializer_class()(request.user).data,
        }
```

If the project has a custom user model, set a serializer that matches the actual fields you want returned instead of assuming the default docs example fits the model.

## Common Pitfalls

- Using Knox as the only DRF default authentication class and also relying on the stock Knox login view. That creates a circular login flow.
- Forgetting `python manage.py migrate`. Knox stores token records in the database.
- Keeping `rest_framework.authtoken` enabled while switching to Knox. Pick one token system and migrate cleanly.
- Importing `knox.urls` directly instead of `include("knox.urls")`.
- Treating logout endpoints as `GET` or `DELETE`. Knox logout endpoints are `POST`.
- Assuming the login response will automatically include serialized user data. In `5.0.4`, `USER_SERIALIZER` is not enabled by default.
- Setting `TOKEN_TTL` to `timedelta(0)` or a negative duration, which makes tokens immediately unusable.
- Enabling `AUTO_REFRESH` without deciding whether total lifetime must be capped with `AUTO_REFRESH_MAX_TTL`.

## Version-Sensitive Notes For `5.0.4`

- `5.0.4` was released on 2026-03-10.
- The `5.0.4` changelog entry is support-focused: Django `5.2` and `6.0`, Python `3.13` and `3.14`.
- `5.0.0` invalidated tokens created before the `5.x` line. If a project is upgrading from `4.x` or older, plan a forced re-authentication event.
- `5.0.0` also dropped Django `4.0` support and made Django `4.2+` the supported baseline.
- The docs site is maintained separately from the PyPI release pin. Treat it as current upstream guidance and use the changelog when a detail looks newer than the package version you are installing.

## Recommended Agent Workflow

1. Confirm whether the project wants username/password login, session login, or another upstream DRF authentication method for minting Knox tokens.
2. Install the pinned package version, add `"knox"` to `INSTALLED_APPS`, and run migrations before wiring auth endpoints.
3. If Knox tokens are the default DRF auth mechanism, implement the custom login view immediately instead of relying on the stock login view.
4. Decide the token policy early: fixed expiry, auto-refresh, max lifetime, and per-user token limits.
5. If the project already uses DRF token auth or an older Knox major version, treat token migration and forced logout behavior as part of the rollout.

## Official Sources Used

- Docs root: `https://jazzband.github.io/django-rest-knox/`
- Installation: `https://jazzband.github.io/django-rest-knox/installation/`
- Auth guide: `https://jazzband.github.io/django-rest-knox/auth/`
- URL routing: `https://jazzband.github.io/django-rest-knox/urls/`
- Views API: `https://jazzband.github.io/django-rest-knox/views/`
- Settings reference: `https://jazzband.github.io/django-rest-knox/settings/`
- Changelog: `https://jazzband.github.io/django-rest-knox/changelog/`
- PyPI package page: `https://pypi.org/project/django-rest-knox/`
- PyPI JSON metadata: `https://pypi.org/pypi/django-rest-knox/json`
- Source repository: `https://github.com/jazzband/django-rest-knox`
- Package defaults source: `https://raw.githubusercontent.com/jazzband/django-rest-knox/develop/knox/settings.py`

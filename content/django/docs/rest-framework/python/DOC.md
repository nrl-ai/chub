---
name: rest-framework
description: "djangorestframework package guide for Python projects using Django REST framework 3.16.1"
metadata:
  languages: "python"
  versions: "3.16.1"
  revision: 2
  updated-on: "2026-03-11"
  source: maintainer
  tags: "django,drf,rest,api,python,web"
---

# djangorestframework Python Package Guide

## What This Package Is

`djangorestframework` is the PyPI package for Django REST framework. Install it as `djangorestframework`, but import from `rest_framework` in code.

This guide covers package version `3.16.1`. The official docs site is a latest-docs site rather than a frozen versioned snapshot, so treat release notes and the 3.16 announcement as the source of truth for version-sensitive behavior.

## Install And Compatibility Baseline

Pin the package when you need 3.16.1-specific behavior:

```bash
pip install "djangorestframework==3.16.1"
```

For a new project, install Django explicitly too:

```bash
pip install "django>=4.2" "djangorestframework==3.16.1"
```

Relevant upstream compatibility facts for `3.16.1`:

- PyPI declares `Requires: Python >=3.9`.
- The official 3.16 announcement says the minimum supported versions are Django `4.2` and Python `3.9`.
- DRF 3.16 adds support for Django `5.1`, the upcoming Django `5.2` LTS, and Python `3.13`.

If your project is on Django `<4.2` or Python `<3.9`, do not upgrade to `3.16.1` without first moving the runtime baseline.

Optional packages from the official docs:

```bash
pip install markdown django-filter pyyaml uritemplate
```

Use them only if you need browsable API Markdown rendering, filtering, or schema-generation extras.

## Project Setup

Add DRF to `INSTALLED_APPS`:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "rest_framework",
]
```

If you want the browsable API login/logout views:

```python
# urls.py
from django.urls import include, path

urlpatterns = [
    path("api-auth/", include("rest_framework.urls", namespace="rest_framework")),
]
```

If you want DRF's built-in token authentication, also add the token app and run migrations:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "rest_framework",
    "rest_framework.authtoken",
]
```

```bash
python manage.py migrate
```

## Minimal CRUD API

For standard model CRUD, the shortest reliable path is `ModelSerializer` plus `ModelViewSet` plus `DefaultRouter`.

```python
# models.py
from django.db import models

class Widget(models.Model):
    name = models.CharField(max_length=100, unique=True)
    is_active = models.BooleanField(default=True)
    created_at = models.DateTimeField(auto_now_add=True)
```

```python
# serializers.py
from rest_framework import serializers

from .models import Widget

class WidgetSerializer(serializers.ModelSerializer):
    class Meta:
        model = Widget
        fields = ["id", "name", "is_active", "created_at"]
        read_only_fields = ["id", "created_at"]
```

```python
# views.py
from rest_framework import permissions, viewsets

from .models import Widget
from .serializers import WidgetSerializer

class WidgetViewSet(viewsets.ModelViewSet):
    queryset = Widget.objects.all().order_by("-created_at")
    serializer_class = WidgetSerializer
    permission_classes = [permissions.IsAuthenticated]
```

```python
# urls.py
from django.urls import include, path
from rest_framework.routers import DefaultRouter

from .views import WidgetViewSet

router = DefaultRouter()
router.register("widgets", WidgetViewSet, basename="widget")

urlpatterns = [
    path("api/", include(router.urls)),
    path("api-auth/", include("rest_framework.urls", namespace="rest_framework")),
]
```

Use `APIView` or the generic class-based views when you need a custom request flow. Use `ModelViewSet` when the endpoint is mostly CRUD and you want routers, pagination hooks, and consistent action wiring.

## Global DRF Configuration

The most important DRF defaults are not safe for a typical private API. Upstream defaults include:

- authentication: `SessionAuthentication` and `BasicAuthentication`
- permissions: `AllowAny`
- pagination: off unless you set a pagination class and `PAGE_SIZE`
- schema generation: `rest_framework.schemas.openapi.AutoSchema`

Set project-wide defaults explicitly:

```python
# settings.py
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": [
        "rest_framework.authentication.SessionAuthentication",
        "rest_framework.authentication.TokenAuthentication",
    ],
    "DEFAULT_PERMISSION_CLASSES": [
        "rest_framework.permissions.IsAuthenticated",
    ],
    "DEFAULT_PARSER_CLASSES": [
        "rest_framework.parsers.JSONParser",
        "rest_framework.parsers.FormParser",
        "rest_framework.parsers.MultiPartParser",
    ],
    "DEFAULT_RENDERER_CLASSES": [
        "rest_framework.renderers.JSONRenderer",
        "rest_framework.renderers.BrowsableAPIRenderer",
    ],
    "DEFAULT_SCHEMA_CLASS": "rest_framework.schemas.openapi.AutoSchema",
    "DEFAULT_PAGINATION_CLASS": "rest_framework.pagination.PageNumberPagination",
    "PAGE_SIZE": 50,
}
```

If you are building an API-only service, remove the browsable API renderer:

```python
REST_FRAMEWORK = {
    "DEFAULT_RENDERER_CLASSES": [
        "rest_framework.renderers.JSONRenderer",
    ],
}
```

If you need query-parameter filtering, install `django-filter` and opt in explicitly:

```python
# settings.py
INSTALLED_APPS += ["django_filters"]

REST_FRAMEWORK = {
    "DEFAULT_FILTER_BACKENDS": [
        "django_filters.rest_framework.DjangoFilterBackend",
    ],
}
```

## Authentication Patterns

### reviewuthentication

Use this when DRF is part of a Django-rendered app or you want the browsable API to share Django sessions.

```python
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": [
        "rest_framework.authentication.SessionAuthentication",
    ],
    "DEFAULT_PERMISSION_CLASSES": [
        "rest_framework.permissions.IsAuthenticated",
    ],
}
```

Important behavior:

- unsafe methods still require a valid CSRF token
- failed permission checks may produce `403` instead of `401`
- it fits browser-based flows better than machine-to-machine API clients

### Token Authentication

Use DRF's built-in token auth only for simple first-party clients.

```python
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": [
        "rest_framework.authentication.TokenAuthentication",
    ],
    "DEFAULT_PERMISSION_CLASSES": [
        "rest_framework.permissions.IsAuthenticated",
    ],
}
```

Expose the built-in token endpoint:

```python
# urls.py
from django.urls import path
from rest_framework.authtoken.views import obtain_auth_token

urlpatterns = [
    path("api/token/", obtain_auth_token),
]
```

Clients send:

```http
Authorization: Token <token-value>
```

Quick smoke test:

```bash
curl http://127.0.0.1:8000/api/widgets/ \
  -H "Authorization: Token <token-value>"
```

Built-in token auth is intentionally minimal. If you need multiple tokens per user, rotation, or expiry, use a dedicated auth package instead of stretching `rest_framework.authtoken`.

### Per-View Overrides

Override auth and permissions when one endpoint differs from the project default:

```python
from rest_framework.authentication import TokenAuthentication
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response
from rest_framework.views import APIView

class MeView(APIView):
    authentication_classes = [TokenAuthentication]
    permission_classes = [IsAuthenticated]

    def get(self, request):
        return Response({"username": request.user.username})
```

When you set `authentication_classes` or `permission_classes` on the view, that view stops using the corresponding global default list.

## Core Usage Decisions

### Serializer Choice

- Use `ModelSerializer` for normal Django model CRUD.
- Use plain `Serializer` for non-model payloads, composite responses, or request validation that is not backed directly by a model.
- Use `HyperlinkedModelSerializer` only when you explicitly want hyperlink fields in the output.

If you instantiate a `HyperlinkedModelSerializer` manually, pass request context so URL fields can render correctly:

```python
serializer = WidgetHyperlinkedSerializer(
    widget,
    context={"request": request},
)
```

### ViewSet And Router Choice

- Use `DefaultRouter` for standard CRUD APIs and `@action` routes.
- Use `GenericAPIView` plus mixins when you want only part of CRUD.
- Use `APIView` when you need full control over the request/response flow.

Do not wire `@action` methods via `.as_view()` manually. Let a router register the viewset so action-specific settings such as `permission_classes` are honored.

### Pagination

Pagination is not automatic everywhere. DRF only auto-paginates generic views and viewsets. If you write a plain `APIView`, you must call the pagination API yourself or you will return an unpaginated response.

The common baseline is page-number pagination:

```python
REST_FRAMEWORK = {
    "DEFAULT_PAGINATION_CLASS": "rest_framework.pagination.PageNumberPagination",
    "PAGE_SIZE": 50,
}
```

## Common Pitfalls

- The package name is `djangorestframework`, but imports come from `rest_framework`.
- Authentication and authorization are separate. DRF defaults to `AllowAny` unless you set permissions explicitly.
- `BasicAuthentication` is fine for testing, not a good default for production clients.
- `SessionAuthentication` on unsafe methods requires CSRF protection even if the user is logged in.
- DRF views opt out of Django 5.1+ `LoginRequiredMiddleware`; enforce login with DRF auth and permission settings instead.
- `DjangoModelPermissions` requires a queryset or `get_queryset()` because DRF infers the model from it.
- `AllowAny` and an empty permission list are effectively open access. Use `AllowAny` only when you want that openness to be explicit.

## Version-Sensitive Notes For 3.16.x

Official upstream notes to keep in mind for `3.16.1`:

- `3.16.0` was released on March 28, 2025 and raised the minimum supported versions to Django `4.2` and Python `3.9`.
- `3.16.0` added support for Django `5.1`, the upcoming Django `5.2` LTS, and Python `3.13`.
- `3.16.0` improved generated validator support for `UniqueConstraint`, especially for nullable fields and conditional constraints.
- `3.16.1` was released on August 6, 2025 and is a bugfix release, not a new feature line.
- `3.16.1` fixed regressions around `unique_together` validation with `SerializerMethodField` and `UniqueTogetherValidator` handling of fields with `source`.

Practical implication:

- if you upgrade from `3.15.x`, re-test serializer validation on models that use `UniqueConstraint`, `unique_together`, nullable unique fields, or serializer fields with `source=...`
- if you copied compatibility assumptions from older guides, update them to the `Django >=4.2`, `Python >=3.9` baseline before debugging downstream failures
- if you are reading the current docs home page, remember that it lists actively supported series and may be stricter than the exact `3.16.1` PyPI requirement

## Official Source URLs

- Docs root: https://www.django-rest-framework.org/
- Quickstart: https://www.django-rest-framework.org/tutorial/quickstart/
- Settings: https://www.django-rest-framework.org/api-guide/settings/
- Authentication: https://www.django-rest-framework.org/api-guide/authentication/
- Permissions: https://www.django-rest-framework.org/api-guide/permissions/
- Pagination: https://www.django-rest-framework.org/api-guide/pagination/
- 3.16 announcement: https://www.django-rest-framework.org/community/3.16-announcement/
- Release notes: https://www.django-rest-framework.org/community/release-notes/
- PyPI package page: https://pypi.org/project/djangorestframework/3.16.1/

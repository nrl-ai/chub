---
name: rest-framework-spectacular
description: "drf-spectacular OpenAPI schema generation for Django REST Framework in Python projects"
metadata:
  languages: "python"
  versions: "0.29.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django,django-rest-framework,openapi,swagger,redoc,schema"
---

# drf-spectacular Python Package Guide

`drf-spectacular` generates OpenAPI 3 schemas for Django REST Framework and includes schema, Swagger UI, and Redoc views. Use it when you need DRF schemas that are explicit enough for generated clients, API docs, contract review, or downstream tooling.

## Package Snapshot

- Package: `drf-spectacular`
- Main import paths: `drf_spectacular.openapi`, `drf_spectacular.views`, `drf_spectacular.utils`, `drf_spectacular.extensions`
- Version covered: `0.29.0`
- Docs root: `https://drf-spectacular.readthedocs.io/en/latest/`
- Registry URL: `https://pypi.org/project/drf-spectacular/`

Version-sensitive note:

- The published docs still describe Python `>=3.7`, but the `0.29.0` changelog drops Python `3.7` from the test suite and calls out unresolved pydantic issues on `<=3.8`.
- Treat Python `3.9+` as the practical floor for new work unless your project already proves an older matrix.

## Install

Install the package into a Django project that already uses Django REST Framework:

```bash
python -m pip install "drf-spectacular==0.29.0"
```

If you want Swagger UI and Redoc assets served locally instead of loading from a CDN, install the sidecar extra:

```bash
python -m pip install "drf-spectacular[sidecar]==0.29.0"
```

Typical companion packages:

```bash
python -m pip install django djangorestframework
```

## Minimal Django And DRF Setup

Add `drf_spectacular` to `INSTALLED_APPS`, point DRF at `AutoSchema`, and define basic schema metadata:

```python
# settings.py
INSTALLED_APPS = [
    # ...
    "rest_framework",
    "drf_spectacular",
]

REST_FRAMEWORK = {
    "DEFAULT_SCHEMA_CLASS": "drf_spectacular.openapi.AutoSchema",
}

SPECTACULAR_SETTINGS = {
    "TITLE": "My API",
    "DESCRIPTION": "Internal API for my service",
    "VERSION": "1.0.0",
}
```

Expose the schema and documentation views:

```python
# urls.py
from django.urls import path
from drf_spectacular.views import (
    SpectacularAPIView,
    SpectacularRedocView,
    SpectacularSwaggerView,
)

urlpatterns = [
    path("api/schema/", SpectacularAPIView.as_view(), name="schema"),
    path(
        "api/schema/swagger-ui/",
        SpectacularSwaggerView.as_view(url_name="schema"),
        name="swagger-ui",
    ),
    path(
        "api/schema/redoc/",
        SpectacularRedocView.as_view(url_name="schema"),
        name="redoc",
    ),
]
```

Generate and validate the schema from the command line before wiring docs into CI or client generation:

```bash
python manage.py spectacular --color --file schema.yaml --validate
```

Upstream guidance: warnings are signals that your schema needs overrides. Some warnings are harmless fallbacks, but do not ignore them blindly.

## Core Workflow

The generator works best when DRF views expose real serializer and queryset information.

### Start with explicit view metadata

Even on `APIView`, set `serializer_class` and `queryset` when you can. `drf-spectacular` uses them even in places where plain DRF does not.

```python
from myapp.models import Widget
from rest_framework import serializers, viewsets

class WidgetSerializer(serializers.Serializer):
    id = serializers.IntegerField()
    name = serializers.CharField()

class WidgetViewSet(viewsets.ReadOnlyModelViewSet):
    queryset = Widget.objects.all()
    serializer_class = WidgetSerializer
```

### Refine operations with `@extend_schema`

Use `@extend_schema` for request bodies, query parameters, examples, and response overrides.

```python
from drf_spectacular.utils import OpenApiParameter, extend_schema
from rest_framework.response import Response
from rest_framework.viewsets import ViewSet

class SearchViewSet(ViewSet):
    @extend_schema(
        parameters=[
            OpenApiParameter("q", str, OpenApiParameter.QUERY, required=True),
        ],
        responses={200: WidgetSerializer(many=True)},
    )
    def list(self, request):
        return Response([])
```

Use `@extend_schema_view` when you need to annotate inherited mixin methods you are not redefining yourself.

### Refine custom fields and serializers

Use `@extend_schema_field` when a custom field or `SerializerMethodField` would otherwise resolve to the wrong type. Use `@extend_schema_serializer` to exclude fields, attach examples, or override `many`.

```python
from drf_spectacular.utils import extend_schema_field, extend_schema_serializer
from rest_framework import serializers

@extend_schema_serializer(exclude_fields=("internal_token",))
class WidgetSerializer(serializers.Serializer):
    name = serializers.CharField()
    internal_token = serializers.CharField()
    computed = serializers.SerializerMethodField()

    @extend_schema_field(str)
    def get_computed(self, obj):
        return "value"
```

### Use extensions for third-party code you cannot decorate

When a library supplies authentication classes, fields, serializers, filters, or views that you cannot edit directly, register an extension instead of patching generated schema output by hand.

Authentication example:

```python
# myapp/schema.py
from drf_spectacular.extensions import OpenApiAuthenticationExtension

class ApiKeyAuthenticationScheme(OpenApiAuthenticationExtension):
    target_class = "myapp.auth.ApiKeyAuthentication"
    name = "ApiKeyAuth"

    def get_security_definition(self, auto_schema):
        return {
            "type": "apiKey",
            "in": "header",
            "name": "X-API-Key",
        }
```

Import the module from `apps.py.ready()` so the extension registers exactly once during Django startup:

```python
# myapp/apps.py
from django.apps import AppConfig

class MyAppConfig(AppConfig):
    name = "myapp"

    def ready(self):
        import myapp.schema  # noqa: F401
```

## Schema Serving, Auth, And Settings That Matter First

Start with a small `SPECTACULAR_SETTINGS` surface and only add overrides when the schema or docs UI needs them.

```python
SPECTACULAR_SETTINGS = {
    "TITLE": "My API",
    "DESCRIPTION": "Internal API",
    "VERSION": "1.0.0",
    "SERVE_INCLUDE_SCHEMA": False,
    "SERVE_PUBLIC": False,
    "SERVE_PERMISSIONS": ["rest_framework.permissions.IsAdminUser"],
    "SERVE_AUTHENTICATION": [
        "rest_framework.authentication.SessionAuthentication",
    ],
}
```

Practical guidance:

- `SERVE_INCLUDE_SCHEMA = False` keeps the schema endpoint from listing itself in the schema.
- `SERVE_PUBLIC = False` makes schema generation for served docs respect the requesting user.
- `SERVE_PERMISSIONS` and `SERVE_AUTHENTICATION` control access to the schema views themselves. Use them when docs must be private.
- `AUTHENTICATION_WHITELIST` limits which authentication classes appear in the schema. Leave it `None` to show all discovered auth schemes.
- `OAS_VERSION` defaults to OpenAPI `3.0.3`. Only move to `3.1.0` if downstream tooling actually expects it.
- `SCHEMA_PATH_PREFIX` and related trimming/insertion settings help when your deployed API path differs from router paths, for example behind a reverse proxy or shared `/api/v1` prefix.
- `COMPONENT_SPLIT_REQUEST = True` is often worth enabling when request and response shapes differ, especially for upload endpoints and `FileField` handling.
- `ENABLE_DJANGO_DEPLOY_CHECK` is enabled by default and runs schema checks during Django's deploy checks. Keep it on unless you have a deliberate reason to suppress it.

Avoid relying on the global `SECURITY` setting for new work. Upstream marks it as strongly discouraged except for rare cases. For auth schemes, prefer `OpenApiAuthenticationExtension` or targeted auth whitelisting.

## Versioning And Multiple API Variants

If your DRF project uses versioning, the default schema view only includes unversioned endpoints plus endpoints for the requested API version.

For CLI generation:

```bash
python manage.py spectacular --api-version v1 --file schema-v1.yaml
```

For a served schema view:

```python
path(
    "api/schema/v1/",
    SpectacularAPIView.as_view(api_version="v1"),
    name="schema-v1",
)
```

If endpoints seem to disappear from the schema, check DRF versioning before assuming the generator is broken.

## Common Pitfalls

### Empty or incomplete schema

Usually caused by one of these:

- `REST_FRAMEWORK["DEFAULT_SCHEMA_CLASS"]` still points somewhere else.
- Views do not expose a serializer or queryset.
- Endpoints are hidden behind DRF versioning and no version was requested.
- The schema view is filtered by request user because `SERVE_PUBLIC` is `False`.

### Duplicate operations from `{format}` suffix routes

If your project uses `format_suffix_patterns`, the FAQ recommends the built-in preprocessing hook:

```python
SPECTACULAR_SETTINGS = {
    "PREPROCESSING_HOOKS": [
        "drf_spectacular.hooks.preprocess_exclude_path_format",
    ],
}
```

### `FileField` or upload endpoints look wrong

`FileField` cannot be represented perfectly as one shared component because request and response forms differ. Upstream recommends:

- enable `COMPONENT_SPLIT_REQUEST = True`
- use `MultiPartParser` or another parser that matches the upload endpoint

### Unsupported authentication class

If your schema does not show a custom auth mechanism correctly, do not hard-code OpenAPI fragments into every view. Add an `OpenApiAuthenticationExtension` for the authentication class instead.

### Blank Swagger UI or Redoc under Content Security Policy

If the docs page renders blank under CSP:

- use the `sidecar` extra to self-host assets
- update CSP to allow the required scripts, styles, fonts, and worker sources for the UI you serve
- if you use `SpectacularSwaggerSplitView`, validate it carefully behind path-rewriting proxies because the split request can break in that setup

### Enum naming warnings

Postprocessing tries to create stable enum component names, but warnings can still appear when names collide or differ only by suffix. Use `ENUM_NAME_OVERRIDES` if generated enum names must stay stable across schema revisions.

## Practical Pattern For Agent Work

When you need a reliable schema fast:

1. Configure `DEFAULT_SCHEMA_CLASS` and a minimal `SPECTACULAR_SETTINGS`.
2. Run `python manage.py spectacular --validate`.
3. Fix missing serializer/queryset metadata on views.
4. Add `@extend_schema` only where auto-discovery is insufficient.
5. Add extensions for third-party code or custom auth.
6. Re-run validation before using the schema for client generation or publishing docs.

This order matches upstream guidance and prevents premature schema hand-editing.

## Official Sources

- Docs root: `https://drf-spectacular.readthedocs.io/en/latest/`
- Package overview and install: `https://drf-spectacular.readthedocs.io/en/latest/readme.html`
- Customization workflow: `https://drf-spectacular.readthedocs.io/en/latest/customization.html`
- Settings reference: `https://drf-spectacular.readthedocs.io/en/latest/settings.html`
- FAQ: `https://drf-spectacular.readthedocs.io/en/latest/faq.html`
- Changelog: `https://drf-spectacular.readthedocs.io/en/latest/changelog.html`
- PyPI: `https://pypi.org/project/drf-spectacular/`

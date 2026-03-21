---
name: rest-framework-yasg
description: "drf-yasg package guide for Python - Swagger/OpenAPI 2.0 schema generation for Django REST Framework"
metadata:
  languages: "python"
  versions: "1.21.15"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "drf-yasg,django,django-rest-framework,swagger,openapi,schema"
---

# drf-yasg Python Package Guide

## Golden Rule

`drf-yasg` is a Django REST Framework schema generator for **Swagger 2.0 / OpenAPI 2.0**.

Use it when a project already depends on `drf-yasg` or when you specifically need Swagger 2.0 output plus its bundled Swagger UI and ReDoc views. Do not treat it as an OpenAPI 3 generator.

## Version-Sensitive Notes

- PyPI shows `1.21.15` released on `2026-02-24`.
- PyPI metadata for `1.21.15` declares support for Python `>=3.9` and classifiers through Python `3.13`, Django `4.0` through `5.2`, and Django REST Framework `3.13` through `3.16`.
- The Read the Docs site is behind the package release line:
  - `/en/stable/` renders documentation for `1.21.7`
  - `/en/latest/` renders `1.20.4.dev2+g...`
- Practical setup and decorator usage from the docs still matches the `1.21.x` line, but compatibility claims should come from PyPI for `1.21.15`, not from the docs site banners.
- The package description on PyPI still documents a few constraints that matter in current versions:
  - only `URLPathVersioning` and `NamespaceVersioning` are supported for generated versioned endpoints
  - codec support depends on `coreapi`

## Install

Pin the version when you need behavior that matches this doc:

```bash
pip install drf-yasg==1.21.15
```

Optional extras from PyPI:

```bash
pip install "drf-yasg[validation]==1.21.15"
```

Typical project dependencies:

- `Django`
- `djangorestframework`
- `drf-yasg`

## Minimal Setup

Add `drf_yasg` and DRF to `INSTALLED_APPS`:

```python
# settings.py
INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
    "rest_framework",
    "drf_yasg",
]
```

Define a schema view and expose raw schema plus UI routes:

```python
# urls.py
from django.urls import path, re_path
from rest_framework import permissions
from drf_yasg import openapi
from drf_yasg.views import get_schema_view

schema_view = get_schema_view(
    openapi.Info(
        title="My API",
        default_version="v1",
        description="Internal API documentation",
    ),
    public=True,
    permission_classes=(permissions.AllowAny,),
)

urlpatterns = [
    re_path(
        r"^swagger(?P<format>\.json|\.yaml)$",
        schema_view.without_ui(cache_timeout=0),
        name="schema-json",
    ),
    path(
        "swagger/",
        schema_view.with_ui("swagger", cache_timeout=0),
        name="schema-swagger-ui",
    ),
    path(
        "redoc/",
        schema_view.with_ui("redoc", cache_timeout=0),
        name="schema-redoc",
    ),
]
```

This gives you:

- `/swagger.json` or `/swagger.yaml` for machine-readable schema output
- `/swagger/` for Swagger UI
- `/redoc/` for ReDoc

## Core Usage

### Override schema generation for one operation

Use `@swagger_auto_schema` when DRF inference is incomplete:

```python
from drf_yasg import openapi
from drf_yasg.utils import swagger_auto_schema
from rest_framework import serializers, viewsets
from rest_framework.response import Response

class UserSerializer(serializers.Serializer):
    id = serializers.IntegerField()
    email = serializers.EmailField()

class UserViewSet(viewsets.ViewSet):
    @swagger_auto_schema(
        operation_summary="List users",
        manual_parameters=[
            openapi.Parameter(
                "email",
                openapi.IN_QUERY,
                description="Filter by exact email",
                type=openapi.TYPE_STRING,
            ),
        ],
        responses={200: UserSerializer(many=True)},
    )
    def list(self, request):
        return Response([{"id": 1, "email": "dev@example.com"}])
```

Use this decorator for:

- custom query parameters
- explicit request bodies
- non-default response shapes
- better operation summaries and descriptions

### Use serializer-backed query parameters

When query params match a serializer, prefer `query_serializer=`:

```python
from drf_yasg.utils import swagger_auto_schema
from rest_framework import serializers

class UserFilterSerializer(serializers.Serializer):
    email = serializers.EmailField(required=False)
    is_active = serializers.BooleanField(required=False)

@swagger_auto_schema(query_serializer=UserFilterSerializer)
def list(self, request, *args, **kwargs):
    ...
```

### Hide views or methods

Hide an entire view:

```python
class InternalView(APIView):
    swagger_schema = None
```

Hide one method only:

```python
@swagger_auto_schema(auto_schema=None)
def delete(self, request, *args, **kwargs):
    ...
```

### Document computed serializer fields

Use `swagger_serializer_method` for `SerializerMethodField` values that DRF cannot infer correctly:

```python
from drf_yasg.utils import swagger_serializer_method
from rest_framework import serializers

class ReportSerializer(serializers.Serializer):
    total = serializers.SerializerMethodField()

    @swagger_serializer_method(serializer_or_field=serializers.IntegerField())
    def get_total(self, obj):
        return obj.total
```

### Generate a static schema in CI or release jobs

Set `DEFAULT_INFO` when using the management command:

```python
# urls.py
from drf_yasg import openapi

api_info = openapi.Info(
    title="My API",
    default_version="v1",
)

SWAGGER_SETTINGS = {
    "DEFAULT_INFO": "myproject.urls.api_info",
}
```

```bash
python manage.py generate_swagger /tmp/swagger.json
```

Use this when you want to publish a checked-in schema artifact or validate schema generation in CI.

## Config And Auth

`drf-yasg` uses `SWAGGER_SETTINGS` for schema generation and UI behavior:

```python
# settings.py
SWAGGER_SETTINGS = {
    "USE_SESSION_AUTH": False,
    "SECURITY_DEFINITIONS": {
        "Bearer": {
            "type": "apiKey",
            "name": "Authorization",
            "in": "header",
            "description": "Format: Bearer <token>",
        },
    },
    "SECURITY_REQUIREMENTS": [{"Bearer": []}],
    "PERSIST_AUTH": False,
    "REFETCH_SCHEMA_WITH_AUTH": False,
    "REFETCH_SCHEMA_ON_LOGOUT": False,
}
```

Practical guidance:

- `USE_SESSION_AUTH`: set this to `False` for token-only APIs so Swagger UI does not render Django login/logout controls.
- `SECURITY_DEFINITIONS`: define auth mechanisms once so operations inherit them.
- `SECURITY_REQUIREMENTS`: apply auth globally unless a view overrides it.
- `PERSIST_AUTH`: leave this `False` unless you intentionally want tokens stored in browser local storage.
- `REFETCH_SCHEMA_WITH_AUTH`: enable only if schema visibility actually changes by user or token.

If you need stricter validation of generated schemas in development or CI, install the `validation` extra and use the packaged validators documented upstream.

## Common Pitfalls

- `drf-yasg` generates Swagger 2.0, not OpenAPI 3. Features that assume OpenAPI 3 should be modeled elsewhere.
- Package name and import path differ: install `drf-yasg`, import `drf_yasg`.
- Keep `django.contrib.staticfiles` enabled or the bundled UI assets will not render in standard Django deployments.
- If you decorate multi-method actions, you often need method-specific `swagger_auto_schema` decoration instead of one shared decorator.
- The docs site is stale relative to `1.21.15`. Treat compatibility matrices and current supported versions on PyPI as authoritative for this package version.
- If your project uses DRF versioning, only `URLPathVersioning` and `NamespaceVersioning` are documented as supported by `drf-yasg`.
- When schema output differs by request user, host, or URL configuration, avoid caching assumptions copied from examples; verify your actual schema view settings and deployment path.

## Official Source URLs Used For This Doc

- Docs landing page: `https://drf-yasg.readthedocs.io/en/latest/`
- Stable docs readme/setup: `https://drf-yasg.readthedocs.io/en/stable/readme.html`
- Custom schema generation: `https://drf-yasg.readthedocs.io/en/latest/custom_spec.html`
- Settings reference: `https://drf-yasg.readthedocs.io/en/latest/settings.html`
- Security/auth guidance: `https://drf-yasg.readthedocs.io/en/latest/security.html`
- Rendering and management command reference: `https://drf-yasg.readthedocs.io/en/latest/rendering.html`
- PyPI project page and metadata: `https://pypi.org/project/drf-yasg/`
- Repository: `https://github.com/axnsan12/drf-yasg`

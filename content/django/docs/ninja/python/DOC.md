---
name: ninja
description: "django-ninja package guide for Python - typed Django APIs with validation, auth, and OpenAPI docs"
metadata:
  languages: "python"
  versions: "1.5.3"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "django-ninja,django,python,api,openapi,pydantic"
---

# django-ninja Python Package Guide

## When To Use It

Use `django-ninja` when a project already uses Django and you need:

- typed request parsing from path, query, headers, forms, files, or JSON bodies
- response serialization and output filtering
- generated OpenAPI schema plus interactive docs
- a smaller, type-hint-first API layer than Django REST Framework

The PyPI package is `django-ninja`, but imports come from `ninja`.

## Version Scope

- This doc targets `django-ninja==1.5.3`.
- The official docs root and current PyPI page both reflect `1.5.3`.
- PyPI metadata for `1.5.3` requires Python `>=3.9`.

## Install

```bash
pip install "django-ninja==1.5.3"
```

Common variants:

```bash
poetry add django-ninja==1.5.3
uv add django-ninja==1.5.3
```

You do not need to add `"ninja"` to `INSTALLED_APPS` for normal API usage. Add it only if you want the docs UI assets served by Django `staticfiles` instead of the default CDN-hosted assets.

## Minimal Setup

Create `api.py` next to your Django `urls.py`:

```python
from ninja import NinjaAPI, Schema

api = NinjaAPI(title="Example API", version="1.0.0")

class AddIn(Schema):
    a: int
    b: int

class AddOut(Schema):
    result: int

@api.post("/add", response=AddOut)
def add(request, payload: AddIn):
    return {"result": payload.a + payload.b}
```

Wire it into Django URLs:

```python
from django.contrib import admin
from django.urls import path

from .api import api

urlpatterns = [
    path("admin/", admin.site.urls),
    path("api/", api.urls),
]
```

By default:

- interactive docs are served at `/api/docs`
- the OpenAPI schema is served at `/api/openapi.json`

## Request Parsing And Response Schemas

Django Ninja infers input locations from the function signature:

- path params: declared in the route path
- query params: simple typed function arguments
- JSON body: `Schema` arguments
- forms/files: `Form(...)` and `File(...)`

```python
from typing import Optional

from ninja import File, Form, NinjaAPI, Schema
from ninja.files import UploadedFile

api = NinjaAPI()

class UserOut(Schema):
    id: int
    username: str
    email: str

@api.post("/users", response=UserOut)
def create_user(
    request,
    username: str = Form(...),
    email: str = Form(...),
    avatar: Optional[UploadedFile] = File(None),
):
    user = User.objects.create(username=username, email=email)
    if avatar is not None:
        user.avatar.save(avatar.name, avatar)
    return user
```

Practical rules:

- Always declare `response=...` for non-trivial endpoints so output is filtered and validated.
- Use `list[SchemaType]` or `{status_code: SchemaType}` for list and multi-status responses.
- Return Django models only when the response schema is explicit.
- Prefer `payload.model_dump()` over hand-parsing `request.body`.

## Model-Backed Schemas

`ModelSchema` is the shortest path when your API shape mostly mirrors a Django model:

```python
from ninja import ModelSchema

class UserSchema(ModelSchema):
    class Meta:
        model = User
        fields = ["id", "username", "email"]
```

Prefer explicit field lists. Avoid `fields = "__all__"` unless you fully control the model and are sure no sensitive fields can leak.

For nested relations, use `select_related()` or `prefetch_related()` before returning querysets so serialization does not trigger avoidable database work.

## Pagination

Use built-in pagination for list endpoints instead of returning unbounded querysets:

```python
from ninja.pagination import PageNumberPagination, paginate

@api.get("/users", response=list[UserSchema])
@paginate(PageNumberPagination, page_size=50)
def list_users(request):
    return User.objects.order_by("id")
```

Notes:

- `@paginate` works well with Django querysets and standard iterables.
- `PageNumberPagination` can expose a `page_size` query parameter in current v1 releases when you enable it in the paginator config.
- Set a global pagination class in Django settings if most endpoints should behave the same way.

## Authentication And Docs Configuration

```python
from ninja.security import django_auth

@api.get("/me", auth=django_auth)
def me(request):
    return {"username": request.auth.username}
```

Bearer token auth:

```python
from ninja.security import HttpBearer

class AuthBearer(HttpBearer):
    def authenticate(self, request, token):
        if token == "supersecret":
            return token
        return None

api = NinjaAPI(auth=AuthBearer())

@api.get("/protected")
def protected(request):
    return {"token": request.auth}
```

Docs and schema controls:

```python
from django.contrib.admin.views.decorators import staff_member_required
from ninja import NinjaAPI, Redoc, Swagger

api = NinjaAPI(
    title="Example API",
    docs=Swagger(settings={"persistAuthorization": True}),
    docs_decorator=staff_member_required,
)

public_readonly_api = NinjaAPI(docs=Redoc(), openapi_url="/openapi.json")
private_api = NinjaAPI(docs_url=None, openapi_url=None)
```

Important behavior:

- Cookie-based auth enables CSRF protection automatically in v1.
- `docs_url=None` hides the interactive docs page.
- `openapi_url=None` disables the schema endpoint and therefore the docs UI as well.
- Protect docs with `docs_decorator` if the schema should not be public.

## Django Settings That Matter In Production

These settings are easy to miss and affect behavior directly:

```python
NINJA_FIX_REQUEST_FILES_METHODS = ["PUT", "PATCH", "DELETE"]
NINJA_NUM_PROXIES = 1
```

Use them when:

- you need `request.FILES` to work on non-`POST` methods such as `PUT` or `PATCH`
- the app runs behind one or more trusted reverse proxies and client IP handling must use forwarded headers correctly

## Async Usage

Async views are useful for network-bound work, but Django Ninja does not make synchronous ORM calls magically safe inside `async def` views.

```python
from asgiref.sync import sync_to_async

@api.get("/tasks/{task_id}")
async def get_task(request, task_id: int):
    task = await sync_to_async(Task.objects.get)(id=task_id)
    return {"id": task.id, "title": task.title}
```

Practical rules:

- Use Django's async ORM methods where available.
- Wrap synchronous ORM or other blocking work with `sync_to_async(...)`.
- Materialize lazy querysets before returning from async endpoints if evaluation could cross sync/async boundaries unexpectedly.

## Testing

For endpoint-level tests without full Django URL resolution, use `ninja.testing.TestClient`:

```python
from django.test import TestCase
from ninja.testing import TestClient

class APITest(TestCase):
    def test_add(self):
        client = TestClient(api)
        response = client.post("/add", json={"a": 2, "b": 3})

        assert response.status_code == 200
        assert response.json() == {"result": 5}
        assert response.data == {"result": 5}
```

`response.data` is convenient when you want parsed payloads directly in assertions.

## Common Pitfalls

- The install name is `django-ninja`, but imports come from `ninja`.
- Returning a raw model or queryset without `response=...` is a common way to leak fields accidentally.
- Old `0.x` snippets often use patterns that are wrong for current v1 releases, especially around Pydantic config, CSRF, and docs settings.
- Hiding `/docs` is not enough if `/openapi.json` still exposes the full schema.
- File uploads on `PUT`, `PATCH`, or `DELETE` need `NINJA_FIX_REQUEST_FILES_METHODS` if you rely on `request.FILES`.
- An `async def` view still needs async-safe database and I/O access patterns.

## Version-Sensitive Notes

Current `1.5.3` guidance assumes Django Ninja v1 behavior:

- Pydantic 2 is the baseline.
- `Schema` and `ModelSchema` customization uses `class Meta` instead of the older inner `Config` pattern.
- Serializer and validator examples should use Pydantic 2 decorators such as `@field_serializer` and `@field_validator`.
- Cookie auth auto-enables CSRF protection; do not copy old `csrf=True` guidance from pre-v1 examples.

If a project is pinned older than `1.5.x`, re-check pagination configuration, non-`POST` file upload handling, and any docs UI examples against that installed version before copying code directly.

## Official Sources

- Docs root: https://django-ninja.dev/
- Tutorial and installation: https://django-ninja.dev/tutorial/
- What is new in v1: https://django-ninja.dev/whatsnew_v1/
- Authentication guide: https://django-ninja.dev/guides/authentication/
- CSRF guide: https://django-ninja.dev/guides/csrf/
- API docs guide: https://django-ninja.dev/guides/api-docs/
- Settings reference: https://django-ninja.dev/reference/settings/
- Django model schemas: https://django-ninja.dev/guides/response/django-pydantic/
- Pagination guide: https://django-ninja.dev/guides/response/pagination/
- Async support: https://django-ninja.dev/guides/async-support/
- Testing guide: https://django-ninja.dev/guides/testing/
- PyPI package page: https://pypi.org/project/django-ninja/
- GitHub releases: https://github.com/vitalik/django-ninja/releases

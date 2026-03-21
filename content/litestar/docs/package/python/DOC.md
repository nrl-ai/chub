---
name: package
description: "Litestar ASGI framework for Python APIs with typed routing, DI, DTOs, auth, OpenAPI, and testing."
metadata:
  languages: "python"
  versions: "2.21.1"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "litestar,python,asgi,api,web,openapi,async"
---

# `litestar` for Python

Litestar is a typed ASGI framework for building APIs and web apps. Reach for it when you want typed route handlers, layered routing, dependency injection, DTO-based input/output control, built-in OpenAPI, and first-party test helpers without stitching multiple libraries together yourself.

## Install

Base package:

```bash
pip install litestar==2.21.1
```

Recommended local development install if you want the CLI and a bundled ASGI server:

```bash
pip install 'litestar[standard]==2.21.1'
```

If you only need JWT helpers, PyPI publishes an explicit extra for that too:

```bash
pip install 'litestar[jwt]==2.21.1'
```

PyPI also lists extras such as `full`, `sqlalchemy`, `pydantic`, `redis`, `structlog`, `prometheus`, `jinja`, and `cryptography`. Install only the extras your app actually uses.

## Minimal App

```python
from litestar import Litestar, get

@get("/")
async def hello_world() -> dict[str, str]:
    return {"hello": "world"}

app = Litestar(route_handlers=[hello_world])
```

Run it:

```bash
litestar run
```

If autodiscovery does not find your app, pass it explicitly:

```bash
litestar --app=my_project.app:app run
```

The CLI autodiscovers canonical modules such as `app.py`, `asgi.py`, `app/__init__.py`, `application.py`, and related submodules. It looks for a `Litestar` instance named `app` or `application`, any `Litestar` instance, or a callable named `create_app`.

## Core Application Structure

Every Litestar app starts with a `Litestar(...)` instance whose `route_handlers` can include:

- standalone route handler functions
- `Router` instances
- `Controller` classes

```python
from litestar import Litestar, Router, get
from litestar.controller import Controller

@get("/health")
def healthcheck() -> dict[str, str]:
    return {"status": "ok"}

class UserController(Controller):
    path = "/users"

    @get("/{user_id:int}")
    async def get_user(self, user_id: int) -> dict[str, int]:
        return {"user_id": user_id}

admin_router = Router(path="/admin", route_handlers=[UserController])

app = Litestar(route_handlers=[healthcheck, admin_router])
```

Operational notes:

- Route paths support typed converters such as `/{user_id:int}` and `/{order_id:uuid}`.
- Controllers and standalone handlers can be registered multiple times under different routers.
- Mounted ASGI apps are supported when you need to delegate a sub-path to another ASGI application.
- Litestar uses trie-based routing. That matters mostly for large route trees, not for day-to-day handler code.

## Lifespan and Shared State

Use app lifespan hooks for long-lived resources such as DB engines, caches, HTTP clients, or message brokers. Store those resources on `app.state`.

```python
from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager

from litestar import Litestar

@asynccontextmanager
async def db_lifespan(app: Litestar) -> AsyncGenerator[None, None]:
    app.state.db = object()
    try:
        yield
    finally:
        app.state.db = None

app = Litestar(route_handlers=[], lifespan=[db_lifespan])
```

This keeps initialization out of route handlers and gives you one place to manage startup and shutdown cleanup.

## Request Parsing and Typed Handlers

Litestar is strict about typing because it uses annotations for validation, parsing, and OpenAPI generation. Missing or vague handler types cause problems quickly, so annotate inputs and return values deliberately.

Basic JSON body parsing:

```python
from litestar import Litestar, post

@post("/users")
async def create_user(data: dict[str, str]) -> dict[str, str]:
    return data

app = Litestar(route_handlers=[create_user])
```

Use `Body(...)` when you need explicit validation or OpenAPI metadata:

```python
from dataclasses import dataclass
from typing import Annotated

from litestar import Litestar, post
from litestar.params import Body

@dataclass
class UserCreate:
    id: int
    name: str

@post("/users")
async def create_user(
    data: Annotated[
        UserCreate,
        Body(title="Create User", description="Create a new user."),
    ],
) -> UserCreate:
    return data

app = Litestar(route_handlers=[create_user])
```

Use explicit request encodings for forms and uploads:

```python
from dataclasses import dataclass
from typing import Annotated

from litestar import Litestar, post
from litestar.enums import RequestEncodingType
from litestar.params import Body

@dataclass
class LoginForm:
    username: str
    password: str

@post("/login")
async def login(
    data: Annotated[LoginForm, Body(media_type=RequestEncodingType.URL_ENCODED)],
) -> dict[str, str]:
    return {"username": data.username}

app = Litestar(route_handlers=[login])
```

Important limit: `request_max_body_size` defaults to `10MB`. If a request exceeds the limit, Litestar returns `413 Request Entity Too Large`. Setting `request_max_body_size=None` is explicitly discouraged upstream because it can expose the app to DoS-style memory exhaustion.

## Dependency Injection

Dependencies are declared by key and wrapped with `Provide(...)`. They can be attached at the app, router, controller, or route-handler layer.

```python
from litestar import Litestar, get
from litestar.di import Provide

async def provide_request_id() -> str:
    return "req-123"

@get("/", dependencies={"request_id": Provide(provide_request_id)})
async def index(request_id: str) -> dict[str, str]:
    return {"request_id": request_id}

app = Litestar(route_handlers=[index])
```

What matters in practice:

- The dependency dictionary key must match the handler parameter name.
- Dependencies can depend on other dependencies.
- Lower scopes override higher scopes cleanly because dependencies are string-keyed at each layer.
- `Provide.use_cache=True` memoizes the first result for the current request only. Do not treat it as a cross-request cache.

For resources that need cleanup, use generator dependencies and always wrap `yield` in `try` / `finally`:

```python
from collections.abc import Generator

from litestar import Litestar, get
from litestar.di import Provide

def open_resource() -> Generator[dict[str, bool], None, None]:
    resource = {"open": True}
    try:
        yield resource
    finally:
        resource["open"] = False

@get("/", dependencies={"resource": Provide(open_resource)})
def handler(resource: dict[str, bool]) -> dict[str, bool]:
    return resource

app = Litestar(route_handlers=[handler])
```

Upstream notes that cleanup runs after the handler returns and that cleanup exceptions are re-raised in an `ExceptionGroup`. Also use `Dependency(skip_validation=True)` sparingly; the docs explicitly warn to use it with caution.

## DTOs for Create and Update Flows

Use DTOs when you need different read/write contracts, partial update behavior, field exclusion, or tighter control over generated schemas.

```python
from dataclasses import dataclass

from litestar import Litestar, patch
from litestar.dto import DataclassDTO, DTOConfig, DTOData

@dataclass
class User:
    id: int
    name: str
    email: str

class UserReadDTO(DataclassDTO[User]):
    config = DTOConfig(exclude={"email"})

class UserPatchDTO(DataclassDTO[User]):
    config = DTOConfig(exclude={"id"}, partial=True)

@patch("/users/{user_id:int}", dto=UserPatchDTO, return_dto=UserReadDTO, sync_to_thread=False)
def update_user(user_id: int, data: DTOData[User]) -> User:
    user = User(id=user_id, name="Jane", email="jane@example.com")
    return data.update_instance(user)

app = Litestar(route_handlers=[update_user])
```

This is the common pattern for CRUD handlers:

- `dto=` controls request parsing
- `return_dto=` controls response serialization
- `DTOData[T]` gives you update helpers such as `update_instance(...)`
- `partial=True` is the switch you want for `PATCH`

## JWT Authentication

Litestar ships a JWT auth backend that can inject middleware and OpenAPI config during app initialization.

```python
from os import environ
from typing import Any

from litestar import Litestar, Request, get, post
from litestar.openapi import OpenAPIConfig
from litestar.security.jwt import JWTAuth, Token

class User:
    def __init__(self, user_id: str) -> None:
        self.id = user_id

async def retrieve_user_handler(token: Token, connection: Any) -> User | None:
    return User(token.sub) if token.sub else None

jwt_auth = JWTAuth[User](
    retrieve_user_handler=retrieve_user_handler,
    token_secret=environ["JWT_SECRET"],
    exclude=["/login", "/schema"],
)

@post("/login")
async def login_handler() -> dict[str, str]:
    token = jwt_auth.create_token(identifier="user-123")
    return {"access_token": token}

@get("/me")
def me(request: Request[User, Token, Any]) -> dict[str, str]:
    return {"user_id": request.user.id}

app = Litestar(
    route_handlers=[login_handler, me],
    on_app_init=[jwt_auth.on_app_init],
    openapi_config=OpenAPIConfig(title="Example API", version="1.0.0"),
)
```

Auth notes:

- `retrieve_user_handler` is required for `JWTAuth`.
- `jwt_auth.on_app_init` wires in middleware and OpenAPI integration.
- Excluding `/login` and `/schema` is a common upstream pattern.
- Authenticated handlers can read both `request.user` and `request.auth`.
- If you need revocation, the official docs show a `revoked_token_handler` pattern.

## Testing

Litestar’s test client is built on top of `httpx`. Use the regular in-process clients for most handlers, and only switch to subprocess clients when you need real server behavior.

Simple isolated route test:

```python
from litestar import get
from litestar.testing import create_test_client

@get("/health-check")
def health_check() -> str:
    return "healthy"

def test_health_check() -> None:
    with create_test_client(route_handlers=[health_check]) as client:
        response = client.get("/health-check")
        assert response.status_code == 200
        assert response.text == "healthy"
```

Async fixture style:

```python
from collections.abc import AsyncIterator

import pytest
from litestar import Litestar, get
from litestar.testing import AsyncTestClient

@get("/ping", sync_to_thread=False)
def ping() -> str:
    return "pong"

app = Litestar(route_handlers=[ping], debug=True)

@pytest.fixture
async def test_client() -> AsyncIterator[AsyncTestClient[Litestar]]:
    async with AsyncTestClient(app=app) as client:
        yield client
```

Testing notes:

- `create_test_client(...)` is the fastest way to test isolated handlers.
- `websocket_connect(...)` is available on the test client for websocket tests.
- For infinite SSE streams or cases where HTTPX’s in-process emulation is insufficient, Litestar provides `subprocess_sync_client()` and `subprocess_async_client()`.

## OpenAPI and Schema Defaults

Litestar generates OpenAPI automatically from your typed handlers, DTOs, parameters, and response models. During tests and local inspection, `/schema/openapi.json` is the default machine-readable endpoint you will most often use.

If you are adding auth, custom titles, or other schema settings, keep an explicit `OpenAPIConfig(...)` near app initialization so docs generation stays predictable.

## Common Pitfalls

- The docs URL `https://docs.litestar.dev/latest/` is a moving target. This guide covers `2.21.1`, so verify version-sensitive behavior against PyPI and the 2.x changelog before copying examples later.
- Litestar expects strong typing. Missing return annotations or vague body types often break validation or schema generation.
- The `litestar run` command does not scan arbitrary filenames. Use canonical `app` / `application` modules or pass `--app`.
- Dependency keys and handler argument names must match exactly.
- Do not disable request body limits casually. Upstream explicitly warns that `request_max_body_size=None` can enable memory-exhaustion attacks unless another layer enforces limits.
- Use `try` / `finally` around generator dependencies so cleanup still runs on errors.
- If you test SSE or other streaming behavior with infinite generators, do not assume the normal test client is enough; use subprocess test helpers.

## Version-Sensitive Notes for `2.21.1`

- PyPI lists `2.21.1` as the latest Litestar release and marks it as released on `2026-03-07`.
- The official 2.x changelog for `2.21.1` is a bugfix release focused on DTO behavior:
  - nested DTO schema references with custom `__schema_name__`
  - `TypeError` cases involving deeply nested optional DTO fields
- If your code depends on advanced DTO schema generation or deeply nested optional payloads, prefer `2.21.1` examples and avoid older blog posts or issues that predate these fixes.
- The docs root is still correct, but it is not version-frozen. Re-check the changelog if `latest` has moved beyond `2.21.1`.

## Official Sources

- Docs landing page: https://docs.litestar.dev/latest/
- Applications: https://docs.litestar.dev/latest/usage/applications.html
- Routing: https://docs.litestar.dev/latest/usage/routing/overview.html
- Requests: https://docs.litestar.dev/latest/usage/requests.html
- Dependency injection: https://docs.litestar.dev/latest/usage/dependency-injection.html
- JWT auth: https://docs.litestar.dev/latest/usage/security/jwt.html
- CLI: https://docs.litestar.dev/latest/usage/cli.html
- DTO updating tutorial: https://docs.litestar.dev/2/tutorials/dto-tutorial/09-updating.html
- Testing: https://docs.litestar.dev/2/usage/testing.html
- 2.x changelog: https://docs.litestar.dev/2/release-notes/changelog.html
- PyPI package page: https://pypi.org/project/litestar/2.21.1/

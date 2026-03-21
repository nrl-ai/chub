---
name: restx
description: "Flask-RESTX package guide for Python REST APIs with namespaces, validation, and Swagger UI"
metadata:
  languages: "python"
  versions: "1.3.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask-restx,flask,rest,api,swagger,openapi"
---

# Flask-RESTX Python Package Guide

## What It Is

`flask-restx` is a Flask extension for class-based REST APIs with:

- `Resource` handlers for HTTP verbs
- `Namespace` composition for larger projects
- model definitions plus response marshalling
- request payload validation
- generated Swagger UI and `swagger.json`
- documented error handlers and auth metadata

For `1.3.2`, treat it as a stable Flask extension for Python `3.9+`. The current `1.3.x` line is also the compatibility boundary for modern Flask `3.x`.

## Install

```bash
pip install "flask-restx==1.3.2"
```

Typical setup for a new project:

```bash
python -m venv .venv
source .venv/bin/activate
pip install "flask>=2" "flask-restx==1.3.2"
```

If you need a reproducible lockfile, pin both Flask and Flask-RESTX together. Upstream notes that `>=1.3.0` is the line that supports Flask `>=2.0.0`, including Flask `3.x`.

## Minimal Setup

Use `Api(app)` for a small app or `Api().init_app(app)` for an app factory.

```python
from flask import Flask
from flask_restx import Api, Resource, fields

app = Flask(__name__)
app.config["RESTX_VALIDATE"] = True

api = Api(
    app,
    version="1.0",
    title="Todo API",
    description="Small example API",
    doc="/docs",
)

ns = api.namespace("todos", path="/todos", description="Todo operations")

todo_input = api.model(
    "TodoInput",
    {
        "title": fields.String(required=True, description="Short task title"),
        "done": fields.Boolean(required=True, description="Completion flag"),
    },
)

todo_model = api.inherit(
    "Todo",
    todo_input,
    {
        "id": fields.Integer(readonly=True, description="Server-assigned ID"),
    },
)

TODOS = [{"id": 1, "title": "Ship docs", "done": False}]

@ns.route("/")
class TodoList(Resource):
    @ns.marshal_list_with(todo_model)
    def get(self):
        return TODOS

    @ns.expect(todo_input, validate=True)
    @ns.marshal_with(todo_model, code=201)
    def post(self):
        payload = api.payload
        item = {
            "id": len(TODOS) + 1,
            "title": payload["title"],
            "done": payload["done"],
        }
        TODOS.append(item)
        return item, 201

@ns.route("/<int:todo_id>")
@ns.response(404, "Todo not found")
class TodoItem(Resource):
    @ns.marshal_with(todo_model)
    def get(self, todo_id):
        for item in TODOS:
            if item["id"] == todo_id:
                return item
        api.abort(404, f"Todo {todo_id} not found")

if __name__ == "__main__":
    app.run(debug=True)
```

Important behavior:

- Swagger UI is served from `/docs` here; by default it is served from the API root.
- Flask-RESTX also exposes the generated schema through `api.__schema__`.
- `api.payload` gives you the parsed JSON body after `@api.expect(...)`.

## App Factory and Blueprints

Use `Api()` plus `init_app()` when the Flask app is created later:

```python
from flask import Flask
from flask_restx import Api

api = Api(
    version="1.0",
    title="Example API",
    description="Factory-based app",
    doc="/docs",
)

def create_app():
    app = Flask(__name__)
    api.init_app(app)
    return app
```

For larger services, mount the API on a blueprint:

```python
from flask import Blueprint
from flask_restx import Api

api_bp = Blueprint("api", __name__, url_prefix="/api/v1")
api = Api(
    api_bp,
    title="Service API",
    version="1.0",
    description="Versioned API",
    doc="/docs",
)
```

Then register the blueprint:

```python
app.register_blueprint(api_bp)
```

When using blueprints:

- `url_for()` endpoint names are prefixed with the blueprint name, for example `url_for("api.some_endpoint")`
- registering the blueprint is enough; you do not also call `api.init_app(app)`

## Organizing Larger APIs

The upstream scaling guide uses one namespace module per surface and adds each namespace to a shared `Api`:

```python
from flask_restx import Api

from .users import api as users_ns
from .projects import api as projects_ns

api = Api(title="My API", version="1.0", description="Service API")
api.add_namespace(users_ns, path="/users")
api.add_namespace(projects_ns, path="/projects")
```

This is the default pattern to use when the API grows beyond one file or when you need multiple versioned blueprints.

## Core Usage Patterns

### Models and marshalling

Use `api.model()`, `api.inherit()`, `@marshal_with()`, and `@marshal_list_with()` to keep response shapes explicit and documented.

```python
project_model = api.model(
    "Project",
    {
        "id": fields.Integer(required=True),
        "name": fields.String(required=True),
        "owner_name": fields.String(attribute="owner.display_name"),
        "url": fields.Url("project_detail", absolute=True),
    },
)
```

Useful details:

- only declared fields are serialized
- `attribute=` remaps object attributes or dict keys
- `fields.Nested(...)` and `fields.List(fields.Nested(...))` document structured output
- `fields.Url(...)` builds URLs from endpoint names
- `ordered=True` on `Api`, `Namespace`, or `marshal()` preserves field order

### Request validation

For JSON request bodies, prefer model-based validation:

```python
create_user = api.model(
    "CreateUser",
    {
        "email": fields.String(required=True),
        "active": fields.Boolean(required=True),
    },
)

@ns.route("/users")
class UserList(Resource):
    @ns.expect(create_user, validate=True)
    def post(self):
        payload = api.payload
        return payload, 201
```

Validation can be enabled in three places:

- per endpoint with `@api.expect(model, validate=True)`
- globally with `app.config["RESTX_VALIDATE"] = True`
- globally with `Api(..., validate=True)`

### `reqparse` still exists, but treat it as legacy

Official docs mark the request-parser system as deprecated and say it is maintained only until `2.0`. Keep it for older codebases, query/header/file parsing, or very small forms, but do not build new large validation flows around it.

```python
from flask_restx import reqparse

parser = reqparse.RequestParser(bundle_errors=True)
parser.add_argument("page", type=int, location="args")
parser.add_argument("picture", location="files")
```

If you keep using `reqparse`:

- `strict=True` on `parse_args()` rejects unknown inputs
- `BUNDLE_ERRORS = True` returns all parser errors instead of only the first one
- if `location=["headers", ...]` is used, header names must match title case

## Docs, Auth, and Configuration

Flask-RESTX generates Swagger UI plus a `swagger.json`-style schema from routes, models, decorators, and namespaces.

Document auth schemes with `authorizations` and `security`:

```python
authorizations = {
    "apikey": {
        "type": "apiKey",
        "in": "header",
        "name": "X-API-Key",
    }
}

api = Api(
    app,
    authorizations=authorizations,
    security="apikey",
    doc="/docs",
)
```

Per-method overrides:

```python
@ns.route("/private")
class PrivateResource(Resource):
    @ns.doc(security="apikey")
    def get(self):
        return {"ok": True}

@ns.route("/public")
class PublicResource(Resource):
    @ns.doc(security=[])
    def get(self):
        return {"ok": True}
```

Important caveat: these settings document security for Swagger UI, but they do not enforce authentication. You still need Flask auth middleware, decorators, or request checks.

Configuration keys worth checking before production rollout:

- `RESTX_JSON`
- `RESTX_VALIDATE`
- `RESTX_MASK_HEADER`
- `RESTX_MASK_SWAGGER`
- `RESTX_INCLUDE_ALL_MODELS`
- `BUNDLE_ERRORS`
- `ERROR_404_HELP`
- `SWAGGER_VALIDATOR_URL`
- `SWAGGER_UI_DOC_EXPANSION`
- `SWAGGER_UI_OPERATION_ID`
- `SWAGGER_UI_REQUEST_DURATION`
- `SWAGGER_UI_OAUTH_APP_NAME`
- `SWAGGER_UI_OAUTH_CLIENT_ID`
- `SWAGGER_UI_OAUTH_REALM`
- `SWAGGER_SUPPORTED_SUBMIT_METHODS`

Disable the built-in docs UI with:

```python
api = Api(app, doc=False)
```

## Error Handling

Werkzeug `HTTPException` instances are serialized automatically. Use `@api.errorhandler` or namespace-level handlers when you need a structured API error shape.

```python
from werkzeug.exceptions import BadRequest

@api.errorhandler(BadRequest)
def handle_bad_request(error):
    return {"message": str(error), "code": "bad_request"}, 400
```

Useful behavior:

- `api.abort(code, message, extra="value")` adds extra keys to the response
- namespace-level handlers override API-level handlers
- `ERROR_INCLUDE_MESSAGE = False` removes the default `message` field

## Common Pitfalls

### The root endpoint is reserved early

Initializing `Api` registers `/` even if you move Swagger UI to another path. If your app needs `/` for something else, register that route before creating `Api`, or mount the API on a blueprint/prefix.

### Swagger docs are Swagger/OpenAPI 2-era, not OpenAPI 3-native

The official docs and generated schema use Swagger terminology such as `securityDefinitions` and `swagger.json`. If a tool expects OpenAPI 3 features directly, verify compatibility before generating clients from the schema.

### `reqparse` is deprecated

Do not start new complex validation layers with `reqparse`. Prefer model-based docs/validation plus a dedicated validation library if your payload rules are non-trivial.

### Docs auth is documentation, not enforcement

`authorizations`, `security`, and Swagger UI settings only describe auth to humans and tooling. They do not check headers or users for you.

### The docs site version label is stale

As of `2026-03-12`, the official Read the Docs pages under `https://flask-restx.readthedocs.io/en/latest/` still render a `1.1.1.dev` title even though PyPI lists `1.3.2` as the latest release. Use PyPI and GitHub releases as the authoritative version source, and treat the docs site as a rolling usage guide.

## Version-Sensitive Notes for `1.3.2`

For the requested version used here `1.3.2`:

- PyPI shows `1.3.2` as the latest release on `2025-09-23`
- Python requirement is still `>=3.9`
- the `1.3.x` line remains the boundary for Flask `3.x` support

Official `1.3.2` release notes add important detail beyond the older `1.3.0` compatibility jump:

- fixes for Flask `3.1.0` test changes
- migration away from `jsonschema.RefResolver` to the newer `referencing` library
- a fix for nullable `fields.Nested` input validation
- a thread lock around first-time schema construction

If you are upgrading older code:

- jumping from `1.2.0` to `1.3.x` is the important compatibility step for Flask `3.x`
- jumping from `1.3.0` to `1.3.2` is mostly about bug fixes and runtime/schema robustness

## Migration Notes

If the codebase still uses Flask-RESTPlus, the official quickstart says Flask-RESTX kept API compatibility and the normal migration is:

- replace `flask_restplus` imports with `flask_restx`
- rename config keys from `RESTPLUS_...` to `RESTX_...`
- retest validation, error responses, and generated docs

## Official Sources

- Docs root: https://flask-restx.readthedocs.io/en/latest/
- Quick start: https://flask-restx.readthedocs.io/en/latest/quickstart.html
- Swagger docs: https://flask-restx.readthedocs.io/en/latest/swagger.html
- Request parsing: https://flask-restx.readthedocs.io/en/latest/parsing.html
- Error handling: https://flask-restx.readthedocs.io/en/latest/errors.html
- Configuration: https://flask-restx.readthedocs.io/en/latest/configuration.html
- Scaling patterns: https://flask-restx.readthedocs.io/en/latest/scaling.html
- PyPI package: https://pypi.org/project/flask-restx/
- GitHub repo: https://github.com/python-restx/flask-restx
- GitHub releases: https://github.com/python-restx/flask-restx/releases

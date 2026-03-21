---
name: smorest
description: "flask-smorest package guide for Python with Flask setup, Blueprint patterns, OpenAPI config, pagination, ETag, auth boundaries, and version-sensitive notes"
metadata:
  languages: "python"
  versions: "0.46.2"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask-smorest,flask,marshmallow,openapi,rest,python"
---

# flask-smorest Python Package Guide

## What It Is

`flask-smorest` is a Flask extension for building JSON REST APIs with:

- Marshmallow-based request parsing and validation
- Marshmallow-based response serialization
- generated OpenAPI documentation
- optional pagination helpers
- optional ETag helpers for conditional requests

The main objects you use are:

- `Api` to bind the extension to a Flask app
- `Blueprint` to group routes and attach request and response metadata
- decorators such as `@blp.arguments`, `@blp.response`, `@blp.paginate`, and `@blp.etag`
- `abort(...)` for consistent JSON HTTP errors

## Version Covered

- Package: `flask-smorest`
- Ecosystem: `pypi`
- Import: `from flask_smorest import Api, Blueprint, abort`
- Version covered: `0.46.2`
- Python requirement: `>=3.9`
- Registry URL: `https://pypi.org/project/flask-smorest/`
- Docs root used for this guide: `https://flask-smorest.readthedocs.io/en/latest/`

The official docs root and PyPI both identify `0.46.2` as the current documented release.

## Install

Pin the package version when you need behavior that matches this guide.

```bash
python -m pip install "flask-smorest==0.46.2"
```

For current releases, assume a modern Flask stack:

- `flask-smorest 0.46.2` requires Python `>=3.9`
- `0.43.0` raised the floor to `Flask>=3.0.2` and `Werkzeug>=3.0.1`
- `0.46.0` dropped marshmallow `<3.24.1`
- `0.46.1` added marshmallow 4 support
- `0.46.2` added Flask async support

## Minimal Setup

`flask-smorest` expects a Flask app, Marshmallow schemas, and routes organized around a `Blueprint`.

```python
from flask import Flask
from flask.views import MethodView
import marshmallow as ma
from flask_smorest import Api, Blueprint, abort

class ItemSchema(ma.Schema):
    id = ma.fields.Int(dump_only=True)
    name = ma.fields.Str(required=True)

class ItemQuerySchema(ma.Schema):
    name = ma.fields.Str()

ITEMS = [
    {"id": 1, "name": "hammer"},
    {"id": 2, "name": "nails"},
]

blp = Blueprint(
    "items",
    __name__,
    url_prefix="/items",
    description="Operations on items",
)

@blp.route("/")
class ItemsResource(MethodView):
    @blp.arguments(ItemQuerySchema, location="query")
    @blp.response(200, ItemSchema(many=True))
    def get(self, query_args):
        if "name" not in query_args:
            return ITEMS
        return [item for item in ITEMS if item["name"] == query_args["name"]]

    @blp.arguments(ItemSchema)
    @blp.response(201, ItemSchema)
    def post(self, item_data):
        item = {"id": len(ITEMS) + 1, **item_data}
        ITEMS.append(item)
        return item

@blp.route("/<int:item_id>")
class ItemByIdResource(MethodView):
    @blp.response(200, ItemSchema)
    def get(self, item_id):
        for item in ITEMS:
            if item["id"] == item_id:
                return item
        abort(404, message="Item not found")

def create_app():
    app = Flask(__name__)
    app.config.update(
        API_TITLE="Example API",
        API_VERSION="v1",
        OPENAPI_VERSION="3.0.2",
    )

    api = Api(app)
    api.register_blueprint(blp)
    return app

app = create_app()
```

## App Factory Pattern

If your project uses Flask's app factory pattern, initialize lazily:

```python
from flask import Flask
from flask_smorest import Api

api = Api()

def create_app():
    app = Flask(__name__)
    app.config.update(
        API_TITLE="Example API",
        API_VERSION="v1",
        OPENAPI_VERSION="3.0.2",
    )
    api.init_app(app)
    api.register_blueprint(blp)
    return app
```

Use `api.init_app(app)` when extension setup happens before the Flask app exists.

## Core Usage Patterns

### Parse Request Data With `@arguments`

`@blp.arguments(...)` deserializes request data with a Marshmallow schema and injects the validated data into the view.

```python
@blp.arguments(ItemSchema)
def post(self, item_data):
    ...
```

Important defaults:

- default location is `"json"`
- validated data is passed as one positional `dict`
- use `location="query"` for query parameters
- use `as_kwargs=True` if you want keyword arguments instead of one `dict`

```python
@blp.arguments(ItemQuerySchema, location="query", as_kwargs=True)
def get(self, **filters):
    ...
```

Allowed locations include `json`, `query`, `path`, `form`, `headers`, `cookies`, `files`, and `json_or_form`.

You can stack `@arguments(...)` decorators, but the order matters because it controls the order of injected parameters.

### Control Unknown-Field Behavior Explicitly

For JSON and other nested body payloads, marshmallow's schema-level `Meta.unknown` setting is the reliable place to define whether unknown fields are rejected, excluded, or included.

```python
import marshmallow as ma

class BaseSchema(ma.Schema):
    class Meta:
        unknown = ma.EXCLUDE
```

For non-body locations such as query parameters, `flask-smorest` already uses `unknown=EXCLUDE` by default through its webargs integration.

### Serialize Responses With `@response`

Use `@blp.response(status_code, schema)` to keep return values and OpenAPI docs aligned.

```python
@blp.response(200, ItemSchema)
def get(self, item_id):
    return load_item(item_id)

@blp.response(204)
def delete(self, item_id):
    delete_item(item_id)
```

Use `many=True` when returning lists:

```python
@blp.response(200, ItemSchema(many=True))
def get(self):
    return list_items()
```

If the view returns a real Flask or Werkzeug `Response`, `flask-smorest` leaves it unchanged and only uses the decorator for documentation.

Avoid returning tuples with a different status code than the one documented by `@blp.response(...)`; it works at runtime, but it makes the generated OpenAPI spec inaccurate.

### Document Alternative Flows

`@blp.response(...)` should describe the normal success path. Use `@blp.alt_response(...)` for non-primary flows you still want in the spec.

```python
@blp.alt_response(401, description="Unauthorized")
@blp.alt_response(404, description="Item not found")
```

### Use `abort(...)` For JSON Error Payloads

`flask_smorest.abort(...)` raises an HTTP error and lets you attach structured details.

```python
abort(400, message="Invalid state transition")
abort(404, message="Item not found")
```

Validation errors from request parsing also flow through the JSON error handler.

## OpenAPI And Documentation Config

You must provide these settings, either through `app.config` or `Api(..., spec_kwargs=...)`:

- `API_TITLE`
- `API_VERSION`
- `OPENAPI_VERSION`

The docs pages can also be served directly from the app:

```python
app.config.update(
    API_TITLE="Example API",
    API_VERSION="v1",
    OPENAPI_VERSION="3.0.2",
    OPENAPI_URL_PREFIX="/docs",
    OPENAPI_JSON_PATH="openapi.json",
    OPENAPI_SWAGGER_UI_PATH="/swagger-ui",
    OPENAPI_SWAGGER_UI_URL="https://cdn.jsdelivr.net/npm/swagger-ui-dist/",
)
```

Useful notes:

- `OPENAPI_URL_PREFIX` enables serving docs from the Flask app
- `OPENAPI_JSON_PATH` controls the JSON spec route below that prefix
- ReDoc, Swagger UI, and RapiDoc are supported if you set both a path and a script URL
- `app.config` overrides overlapping `spec_kwargs`
- `API_SPEC_OPTIONS` adds or overrides root OpenAPI document fields

To export the generated spec during deployment:

```bash
flask openapi print --format=json
flask openapi write --format=json openapi.json
```

If one Flask app hosts multiple APIs, `Api(config_prefix="V1_")` lets each `Api` instance read a separate configuration namespace.

### Custom OpenAPI Mappings

If your API uses custom Flask path converters or custom Marshmallow fields, register them before documenting routes and schemas:

```python
api.register_converter(MyConverter, converter_to_schema)
api.register_field(MyField, "string", "custom-format")
```

## Auth And Security Boundaries

`flask-smorest` does not implement authentication or authorization for you. Handle auth with your normal Flask stack, such as:

- request hooks
- custom decorators
- Flask-Login
- JWT or API-key middleware

Then keep the generated spec honest:

- document `401` and `403` with `@blp.alt_response(...)`
- use `@blp.doc(...)` when the generated operation documentation needs extra auth metadata
- use `api.spec.components...` when you need extra top-level OpenAPI components

Treat auth as an application concern, not a `flask-smorest` feature.

## Pagination

`@blp.paginate()` helps standardize paged list endpoints.

### Pagination In The View

```python
@blp.route("/")
class ItemsResource(MethodView):
    @blp.response(200, ItemSchema(many=True))
    @blp.paginate()
    def get(self, pagination_parameters):
        pagination_parameters.item_count = len(ITEMS)
        return ITEMS[
            pagination_parameters.first_item:pagination_parameters.last_item
        ]
```

Default pagination parameters are effectively:

```python
{"page": 1, "page_size": 10, "max_page_size": 100}
```

When pagination is enabled, responses include an `X-Pagination` header with counts and page navigation data unless you customize or disable that behavior on the blueprint class.

### Post-Pagination With A Pager

If the view returns a lazy collection or query object, pass a pager class to `@blp.paginate(...)` and let the decorator slice after the view returns.

## ETag Support

`@blp.etag` enables conditional request support, but mutation protection is not fully automatic.

```python
@blp.route("/<int:item_id>")
@blp.etag
class ItemByIdResource(MethodView):
    @blp.response(200, ItemSchema)
    def get(self, item_id):
        return load_item(item_id)

    @blp.arguments(ItemSchema)
    @blp.response(200, ItemSchema)
    def put(self, item_data, item_id):
        item = load_item(item_id)
        blp.check_etag(item, ItemSchema)
        item.update(item_data)
        return item
```

Important behavior:

- `ETAG_DISABLED` disables ETag support globally
- GET and HEAD responses can derive ETags from serialized response data
- PUT, PATCH, and DELETE handlers must call `blp.check_etag(...)` themselves
- if you forget `check_etag(...)`, `flask-smorest` warns at runtime
- `blp.set_etag(...)` lets you compute ETags from arbitrary data instead of the response schema

## Common Pitfalls

- Missing `API_TITLE`, `API_VERSION`, or `OPENAPI_VERSION` breaks or weakens spec generation.
- `@blp.arguments(...)` defaults to JSON body parsing. Query parameters need `location="query"`.
- `@blp.arguments(...)` passes one positional `dict` unless you opt into `as_kwargs=True`.
- Stacked `@blp.arguments(...)` decorators inject parameters in decorator order.
- For nested body schemas, configure unknown-field handling on the schema `Meta`; changing parser defaults does not propagate cleanly into nested schemas.
- Returning a tuple with a different status code than the one documented by `@blp.response(...)` creates doc and runtime drift.
- Returning a real `Response` bypasses schema dumping and the decorator's status-code behavior.
- `@blp.etag` does not automatically enforce lost-update protection on writes; you still need `blp.check_etag(...)`.
- Custom converters and custom Marshmallow fields should be registered before routes and schemas are documented.
- The OpenAPI UI is not served unless you configure both the base prefix and the selected UI path and script URL.
- Multipart requests with mixed body locations are not documented correctly; do not stack more than one request-body location on the same view.

## Version-Sensitive Notes

- `0.46.2` adds Flask async support. If you adopt async view functions, validate the behavior in your actual Flask deployment stack because most upstream examples are still synchronous.
- `0.46.1` adds marshmallow 4 support.
- `0.46.0` relies on marshmallow for schema ordering and drops marshmallow versions earlier than `3.24.1`.
- `0.45.0` removed `flask_smorest.__version__`. If you need the installed package version at runtime, use `importlib.metadata.version("flask-smorest")`.
- `0.45.0` also officially supports Python 3.13 and drops Python 3.8.
- `0.43.0` raised the minimum supported Flask and Werkzeug versions to `3.0.2` and `3.0.1`.

## Official Sources

- Docs root: `https://flask-smorest.readthedocs.io/en/latest/`
- Quickstart: `https://flask-smorest.readthedocs.io/en/latest/quickstart.html`
- Arguments guide: `https://flask-smorest.readthedocs.io/en/latest/arguments.html`
- Response guide: `https://flask-smorest.readthedocs.io/en/latest/response.html`
- Pagination guide: `https://flask-smorest.readthedocs.io/en/latest/pagination.html`
- ETag guide: `https://flask-smorest.readthedocs.io/en/latest/etag.html`
- OpenAPI guide: `https://flask-smorest.readthedocs.io/en/latest/openapi.html`
- API reference: `https://flask-smorest.readthedocs.io/en/latest/api_reference.html`
- Changelog: `https://flask-smorest.readthedocs.io/en/latest/changelog.html`
- PyPI: `https://pypi.org/project/flask-smorest/`
- Repository: `https://github.com/marshmallow-code/flask-smorest`

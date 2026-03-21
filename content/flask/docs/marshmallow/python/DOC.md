---
name: marshmallow
description: "flask-marshmallow 1.4.0 package guide for Flask-aware Marshmallow schemas, URL fields, file validation, and SQLAlchemy integration"
metadata:
  languages: "python"
  versions: "1.4.0"
  revision: 2
  updated-on: "2026-03-12"
  source: maintainer
  tags: "flask,marshmallow,serialization,validation,sqlalchemy,uploads"
---

# flask-marshmallow Python Package Guide

## Install

Pin the package version the project expects:

```bash
python -m venv .venv
source .venv/bin/activate
pip install "flask-marshmallow==1.4.0"
```

For ORM-backed schemas, install the SQLAlchemy packages explicitly too:

```bash
pip install "flask-marshmallow==1.4.0" Flask-SQLAlchemy marshmallow-sqlalchemy
```

If you are upgrading an older app, check the changelog before assuming compatibility with old `marshmallow` or Flask versions.

## Initialize In An App Factory

Use the normal Flask extension pattern:

```python
from flask import Flask
from flask_marshmallow import Marshmallow

ma = Marshmallow()

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["API_TITLE"] = "Example API"

    ma.init_app(app)
    return app
```

What `ma` exposes:

- `ma.Schema` for your base schema class
- standard marshmallow fields through the extension
- Flask-specific fields such as `URLFor`, `AbsoluteURLFor`, `Hyperlinks`, and `Config`
- SQLAlchemy schema classes when the SQLAlchemy integration packages are installed

## Core Schema Usage

Use `ma.Schema` like a normal marshmallow schema, then add Flask-specific fields when you need URLs or config-backed values.

```python
from flask import Flask
from flask_marshmallow import Marshmallow
from marshmallow import fields

ma = Marshmallow()
app = Flask(__name__)
ma.init_app(app)

class UserSchema(ma.Schema):
    id = fields.Int(dump_only=True)
    email = fields.Email(required=True)
    profile_url = ma.URLFor("user_detail", values={"user_id": "<id>"})
    _links = ma.Hyperlinks(
        {
            "self": ma.URLFor("user_detail", values={"user_id": "<id>"}),
            "collection": ma.URLFor("user_list"),
        }
    )

user_schema = UserSchema()
```

In a view:

```python
@app.get("/users")
def user_list():
    users = [{"id": 1, "email": "a@example.com"}]
    return {"items": user_schema.dump(users, many=True)}

@app.get("/users/<int:user_id>")
def user_detail(user_id: int):
    return user_schema.dump({"id": user_id, "email": "a@example.com"})
```

Practical notes:

- URL fields build Flask endpoints with `url_for`, so the endpoint name must match an actual route.
- Route parameters belong in `values={...}` and can reference object attributes with placeholders such as `"<id>"`.
- If you serialize outside a request handler, create a request context first:

```python
with app.test_request_context():
    payload = user_schema.dump({"id": 1, "email": "a@example.com"})
```

## SQLAlchemy Integration

If you use `Flask-SQLAlchemy`, initialize the database extension before `Marshmallow`.

```python
from flask import Flask
from flask_marshmallow import Marshmallow
from flask_sqlalchemy import SQLAlchemy

db = SQLAlchemy()
ma = Marshmallow()

class Author(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(120), nullable=False)

class AuthorSchema(ma.SQLAlchemyAutoSchema):
    class Meta:
        model = Author
        load_instance = True

def create_app() -> Flask:
    app = Flask(__name__)
    app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///app.db"

    db.init_app(app)
    ma.init_app(app)
    return app
```

Use this pattern when you want schema generation from Flask-SQLAlchemy models. `flask-marshmallow` does not replace query planning or relationship loading; keep eager-loading and N+1 concerns explicit in your model and query code.

## File Upload Validation

The package includes a `File` field plus validators in `flask_marshmallow.validate`.

```python
from flask import request
from flask_marshmallow import Marshmallow
from flask_marshmallow.validate import FileSize, FileType

ma = Marshmallow()

class UploadSchema(ma.Schema):
    image = ma.File(
        required=True,
        validate=[
            FileType([".png", ".jpg", ".jpeg"]),
            FileSize(max="5 MiB"),
        ],
    )

def validate_upload():
    return UploadSchema().load({"image": request.files["image"]})
```

Use file validation with `request.files` objects, not JSON payloads.

## Config And Auth

`flask-marshmallow` can expose Flask config values through schemas, but it is not an auth library.

```python
class AppInfoSchema(ma.Schema):
    api_title = ma.Config("API_TITLE")
```

Important behavior:

- `Config` reads from `app.config` during serialization.
- Missing config keys raise `ValueError`.
- Do not serialize secrets such as API keys, signing keys, or OAuth client secrets.
- For authentication and authorization, use Flask itself or a dedicated auth extension; `flask-marshmallow` only handles serialization and validation concerns.

## Common Pitfalls

- Forgetting `ma.init_app(app)`: schemas and Flask-aware fields will not be wired to the application.
- Using URL fields for endpoints that are not registered: dumps fail when Flask cannot build the route.
- Serializing URL fields outside a request context: create a test request context for background tasks or tests.
- Expecting SQLAlchemy helpers without `Flask-SQLAlchemy` and `marshmallow-sqlalchemy`: install both packages when you need model-backed schemas.
- Reading missing config keys with `ma.Config`: this raises `ValueError` instead of returning `None`.
- Validating uploads from JSON request bodies: file validators operate on uploaded file objects from `request.files`.
- Copying old examples that use removed legacy schema helpers: prefer `SQLAlchemySchema` and `SQLAlchemyAutoSchema`.

## Version-Sensitive Notes

- This entry is pinned to `1.4.0`, which matches the version used here and the current upstream docs and PyPI release line as checked on 2026-03-12.
- The official changelog says `1.4.0` adds support for `Flask-SQLAlchemy 3`.
- The official changelog says `1.3.0` adds support for `marshmallow 4`; do not assume that compatibility if your app is pinned below `1.3.0`.
- The official changelog says `1.2.0` adds support for `Flask 3.0` and `webargs 8`.
- Since `1.0.0`, the package no longer exposes `flask_marshmallow.__version__`; use `importlib.metadata.version("flask-marshmallow")` if you need the installed package version at runtime.

## Official Sources

- Docs root: `https://flask-marshmallow.readthedocs.io/en/latest/`
- Changelog: `https://flask-marshmallow.readthedocs.io/en/latest/changelog.html`
- PyPI project: `https://pypi.org/project/flask-marshmallow/`
- Repository: `https://github.com/marshmallow-code/flask-marshmallow`
